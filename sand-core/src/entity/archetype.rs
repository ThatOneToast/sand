//! Reusable entity archetypes and lifecycle compilation.
//!
//! An archetype describes state initialization, natural/external adoption,
//! ordered migrations, reconciliation, and cleanup for one statically known
//! entity kind. Runtime identity is a Sand-owned tag plus version score, not a
//! durable Rust entity reference. Every generated function rescans currently
//! loaded entities and binds the match to `@s`.
//!
//! Initialization is idempotent and ordered: missing score dependencies are
//! provisioned first, native bindings are applied by the property compiler,
//! the optional typed callback runs, and the version/initialized marker is
//! written last. Unloaded entities are not scanned; their scoreboard state
//! remains attached and reconciliation resumes when they are observed again.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use sand_components::ResourceLocation;

use crate::entity::curve::{
    FixedPoint, LoweredCurve, LoweredCurveOperation, OverflowPolicy, RoundingPolicy, StatCurve,
};
use crate::entity::diagnostic::EntityDiagnostic;
use crate::entity::kind::{KnownEntityKind, MutableLivingEntityKind, SafeEntityDataWriteKind};
use crate::entity::property::{
    AttributeBinding, AttributeModifierBinding, CurrentHealthSync, EffectBinding, EntityNbtBinding,
    EntityNbtValue, EntityTextSegment, EquipmentBinding, HealthBinding, HealthResizePolicy,
    NameBinding, NativePropertyKey, NumericPropertySource, OwnershipPolicy, RefreshPolicy,
    TagBinding, TeamBinding, validate_native_ownership,
};
use crate::entity::state::{
    EntityFlag, EntityState, EntityStateField, StateSchema, dirty_name, objective_name,
};
use crate::resource_ref::FunctionId;
use crate::state::Ticks;

/// Which externally existing entities an adoption scan may initialize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AdoptionSource {
    /// Unmarked entities not carrying this archetype's external provenance tag.
    ///
    /// Vanilla exposes no general spawn-provenance predicate, so commands
    /// from other packs that omit the explicit external tag are
    /// indistinguishable from natural spawns.
    Natural,
    /// Entities carrying the archetype's Sand-owned external provenance tag.
    ///
    /// Other datapacks can obtain the tag from
    /// [`EntityArchetype::external_adoption_tag`].
    External,
    /// Both natural and externally created entities.
    NaturalAndExternal,
}

/// Treatment of named, tamed, owned, or otherwise special entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpecialEntityPolicy {
    /// Exclude special entities from automatic adoption.
    Exclude,
    /// Adopt them while preserving unrelated name/owner/taming state.
    Preserve,
    /// Include them and allow explicitly owned properties to reconcile.
    Include,
}

/// A typed adoption scan.
///
/// The entity type comes from the archetype's `K`; callers cannot create an
/// unconstrained `@e` scan. Scans see loaded chunks only. `every` bounds work
/// by running one type-constrained scan every N server ticks.
#[derive(Debug, Clone, PartialEq)]
pub struct Adoption {
    source: AdoptionSource,
    every: Ticks,
    max_distance: Option<f64>,
    special: SpecialEntityPolicy,
    required_tags: Vec<crate::entity::property::EntityTag>,
    excluded_tags: Vec<crate::entity::property::EntityTag>,
    state_predicates: Vec<crate::entity::state::StatePredicate>,
}

impl Adoption {
    /// Adopt natural and external entities once per tick.
    #[must_use]
    pub fn natural_and_external() -> Self {
        Self {
            source: AdoptionSource::NaturalAndExternal,
            every: Ticks::new(1),
            max_distance: None,
            special: SpecialEntityPolicy::Preserve,
            required_tags: Vec::new(),
            excluded_tags: Vec::new(),
            state_predicates: Vec::new(),
        }
    }

    /// Restrict adoption to the natural-spawn policy.
    #[must_use]
    pub fn natural() -> Self {
        Self {
            source: AdoptionSource::Natural,
            ..Self::natural_and_external()
        }
    }

    /// Restrict adoption to externally summoned entities.
    #[must_use]
    pub fn external() -> Self {
        Self {
            source: AdoptionSource::External,
            ..Self::natural_and_external()
        }
    }

    /// Set the scan cadence.
    ///
    /// A zero interval is rejected during archetype compilation.
    #[must_use]
    pub fn every(mut self, ticks: Ticks) -> Self {
        self.every = ticks;
        self
    }

    /// Limit adoption to a radius around each scan executor.
    ///
    /// Global type-constrained scans remain supported; this option is useful
    /// for packs that deliberately run the coordinator at player positions.
    #[must_use]
    pub fn within_blocks(mut self, blocks: f64) -> Self {
        self.max_distance = Some(blocks);
        self
    }

    /// Choose how named/tamed/owned entities are treated.
    #[must_use]
    pub fn special_entities(mut self, policy: SpecialEntityPolicy) -> Self {
        self.special = policy;
        self
    }

    /// Require a validated entity tag in addition to the typed entity kind.
    #[must_use]
    pub fn requiring_tag(mut self, tag: crate::entity::property::EntityTag) -> Self {
        self.required_tags.push(tag);
        self
    }

    /// Exclude a validated entity tag from adoption.
    #[must_use]
    pub fn excluding_tag(mut self, tag: crate::entity::property::EntityTag) -> Self {
        self.excluded_tags.push(tag);
        self
    }

    /// Restrict adoption with a typed state predicate.
    ///
    /// Multiple fields are merged into one selector `scores` map, avoiding
    /// handwritten score syntax and additional outer scans.
    #[must_use]
    pub fn where_state(mut self, predicate: crate::entity::state::StatePredicate) -> Self {
        self.state_predicates.push(predicate);
        self
    }
}

/// When an initialized entity is checked against its archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReconcilePolicy {
    /// Initialize once and never automatically reapply owned properties.
    InitializeOnly,
    /// Reconcile after a schema/archetype version changes.
    WhenSchemaChanges,
    /// Reconcile only when a state dependency is dirty.
    WhenDirty,
    /// Reconcile at an explicit interval.
    Every(Ticks),
    /// Reconciliation occurs only through a generated manual function.
    Manual,
}

/// One contiguous ordered archetype migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// Version this step accepts.
    pub from: u32,
    /// Version written only after the callback completes.
    pub to: u32,
    /// Canonical typed migration function.
    pub action: FunctionId,
}

impl Migration {
    /// Construct a migration step.
    #[must_use]
    pub fn new(from: u32, to: u32, action: FunctionId) -> Self {
        Self { from, to, action }
    }
}

/// Cost and generated-resource summary for one archetype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntityRuntimeReport {
    /// Archetype identifier.
    pub archetype: String,
    /// Generated objectives, sorted.
    pub objectives: Vec<String>,
    /// Sand-owned lifecycle tags, sorted.
    pub tags: Vec<String>,
    /// Generated function resource paths, sorted.
    pub functions: Vec<String>,
    /// Maximum outer entity scans on a tick where this archetype is scheduled.
    pub outer_scans_per_cycle: usize,
    /// Human-readable adoption selector.
    pub adoption_selector: Option<String>,
}

/// Compiled records and lifecycle-tag contributions for one archetype.
#[derive(Debug, Clone)]
pub(crate) struct CompiledArchetype {
    pub records: Vec<crate::component::ComponentRecord>,
    pub load_functions: Vec<String>,
    pub tick_functions: Vec<String>,
    pub report: EntityRuntimeReport,
}

/// Reusable lifecycle definition for entity kind `K` and state schema `S`.
#[derive(Debug, Clone)]
pub struct EntityArchetype<K, S> {
    id: ResourceLocation,
    version: u32,
    adoption: Option<Adoption>,
    reconcile: ReconcilePolicy,
    initialize: Option<FunctionId>,
    cleanup: Option<FunctionId>,
    migrations: Vec<Migration>,
    derivations: Vec<EntityDerivation>,
    transitions: Vec<EntityTransitionRule>,
    properties: Vec<ArchetypeProperty>,
    _kind: PhantomData<fn() -> K>,
    _state: PhantomData<fn() -> S>,
}

impl<K, S> EntityArchetype<K, S>
where
    K: KnownEntityKind,
    S: EntityState,
{
    /// Create an archetype.
    ///
    /// The identifier namespaces every generated objective, marker, storage
    /// path, and helper function. The default version is the state schema
    /// version and reconciliation is version-driven.
    #[must_use]
    pub fn new(id: ResourceLocation) -> Self {
        let version = S::schema().version;
        Self {
            id,
            version,
            adoption: None,
            reconcile: ReconcilePolicy::WhenSchemaChanges,
            initialize: None,
            cleanup: None,
            migrations: Vec::new(),
            derivations: Vec::new(),
            transitions: Vec::new(),
            properties: Vec::new(),
            _kind: PhantomData,
            _state: PhantomData,
        }
    }

    /// Override the archetype version independently of the schema version.
    #[must_use]
    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Discover and initialize existing loaded entities.
    #[must_use]
    pub fn adopt(mut self, adoption: Adoption) -> Self {
        self.adoption = Some(adoption);
        self
    }

    /// Choose automatic reconciliation behavior.
    #[must_use]
    pub fn reconcile(mut self, policy: ReconcilePolicy) -> Self {
        self.reconcile = policy;
        self
    }

    /// Run a typed function after state/native setup and before completion is marked.
    #[must_use]
    pub fn initialize_with(mut self, function: FunctionId) -> Self {
        self.initialize = Some(function);
        self
    }

    /// Run a best-effort typed cleanup callback before Sand-owned state is cleared.
    ///
    /// Vanilla provides no reliable callback for every external removal or
    /// unloaded entity. This callback is guaranteed only when the generated
    /// cleanup function is explicitly invoked while the entity is loaded.
    #[must_use]
    pub fn cleanup_with(mut self, function: FunctionId) -> Self {
        self.cleanup = Some(function);
        self
    }

    /// Add one ordered migration.
    #[must_use]
    pub fn migration(mut self, migration: Migration) -> Self {
        self.migrations.push(migration);
        self
    }

    /// Add a cached derived score and its typed dependency declaration.
    ///
    /// The exporter lowers the curve to entity-scoped scoreboard arithmetic
    /// and marks the result dirty only when one of its source scores changes.
    /// Cycles among derived targets are rejected before resources are written.
    #[must_use]
    pub fn derive(mut self, derivation: EntityDerivation) -> Self {
        self.derivations.push(derivation);
        self
    }

    /// Run a typed action when entity-bound state crosses a declared boundary.
    #[must_use]
    pub fn on(mut self, transition: EntityTransition, action: EntityAction) -> Self {
        self.transitions
            .push(EntityTransitionRule { transition, action });
        self
    }

    /// Add an archetype-owned tag while preserving unrelated tags.
    #[must_use]
    pub fn tag(mut self, binding: TagBinding) -> Self {
        self.properties.push(ArchetypeProperty::Tag(binding));
        self
    }

    /// Add/remove an owned tag when a typed flag changes.
    #[must_use]
    pub fn tag_when(mut self, flag: EntityFlag, binding: TagBinding) -> Self {
        let binding = binding.refresh(RefreshPolicy::WhenSourceChanges);
        self.properties
            .push(ArchetypeProperty::ConditionalTag { flag, binding });
        self
    }

    /// Add typed team membership while leaving team configuration external.
    #[must_use]
    pub fn team(mut self, binding: TeamBinding) -> Self {
        self.properties.push(ArchetypeProperty::Team(binding));
        self
    }

    /// Join/leave an owned team when a typed flag changes.
    #[must_use]
    pub fn team_when(mut self, flag: EntityFlag, binding: TeamBinding) -> Self {
        let binding = binding.refresh(RefreshPolicy::WhenSourceChanges);
        self.properties
            .push(ArchetypeProperty::ConditionalTeam { flag, binding });
        self
    }

    /// Archetype identifier.
    #[must_use]
    pub fn id(&self) -> &ResourceLocation {
        &self.id
    }

    /// Sand-owned initialized marker, deterministic across exports.
    #[must_use]
    pub fn initialized_tag(&self) -> String {
        initialized_tag(&self.id.to_string())
    }

    /// Sand-owned tag used to opt an externally summoned entity into an
    /// [`Adoption::external`] scan.
    #[must_use]
    pub fn external_adoption_tag(&self) -> crate::entity::property::EntityTag {
        crate::entity::property::EntityTag::generated(external_tag(&self.id.to_string()))
    }

    /// Call the generated attach/initialize function for the current `@s`.
    ///
    /// This command is execution-scoped. It does not create or return a
    /// persistent entity reference.
    #[must_use]
    pub fn attach(&self) -> String {
        format!(
            "function {}:{}/initialize",
            self.id.namespace(),
            generated_root(&self.id.to_string())
        )
    }

    /// Summon this archetype and initialize the newly created entity.
    ///
    /// Vanilla's `execute summon` binds the new entity directly to `@s`, so
    /// no selector, temporary tag, or global scratch identity is required.
    #[must_use]
    pub fn summon(&self) -> Vec<String> {
        let entity_type = K::entity_type();
        vec![format!(
            "execute summon {entity_type} run {}",
            self.attach()
        )]
    }

    /// Erase Rust marker types for inventory-based exporter registration.
    #[doc(hidden)]
    #[must_use]
    pub(crate) fn definition(&self) -> ArchetypeDefinition {
        ArchetypeDefinition {
            id: self.id.clone(),
            version: self.version,
            entity_type: K::entity_type(),
            kind_label: K::LABEL,
            living: is_living::<K>(),
            mutable_living: is_mutable_living::<K>(),
            schema: S::schema(),
            adoption: self.adoption.clone(),
            reconcile: self.reconcile,
            initialize: self.initialize.clone(),
            cleanup: self.cleanup.clone(),
            migrations: self.migrations.clone(),
            derivations: self.derivations.clone(),
            transitions: self.transitions.clone(),
            properties: self.properties.clone(),
        }
    }
}

/// Erases an archetype's marker types for proc-macro registration.
///
/// This is public only so generated code can cross the crate boundary through
/// `sand::__private`; it is not part of the author-facing entity API.
#[doc(hidden)]
pub fn registered_definition<K, S>(archetype: &EntityArchetype<K, S>) -> ArchetypeDefinition
where
    K: KnownEntityKind,
    S: EntityState,
{
    archetype.definition()
}

impl<K, S> EntityArchetype<K, S>
where
    K: KnownEntityKind + SafeEntityDataWriteKind,
    S: EntityState,
{
    /// Bind a state-aware custom name and visibility.
    #[must_use]
    pub fn name(mut self, binding: NameBinding) -> Self {
        self.properties.push(ArchetypeProperty::Name(binding));
        self
    }

    /// Bind a stable typed native-NBT field.
    #[must_use]
    pub fn native_data(mut self, binding: EntityNbtBinding) -> Self {
        self.properties.push(ArchetypeProperty::Nbt(binding));
        self
    }
}

impl<K, S> EntityArchetype<K, S>
where
    K: KnownEntityKind + MutableLivingEntityKind,
    S: EntityState,
{
    /// Synchronize current/max health according to an explicit resize policy.
    #[must_use]
    pub fn health(mut self, binding: HealthBinding) -> Self {
        self.properties.push(ArchetypeProperty::Health(binding));
        self
    }

    /// Bind an attribute base value.
    #[must_use]
    pub fn attribute(mut self, binding: AttributeBinding) -> Self {
        self.properties.push(ArchetypeProperty::Attribute(binding));
        self
    }

    /// Bind one idempotent namespaced attribute modifier.
    #[must_use]
    pub fn attribute_modifier(mut self, binding: AttributeModifierBinding) -> Self {
        self.properties
            .push(ArchetypeProperty::AttributeModifier(binding));
        self
    }

    /// Apply an archetype-owned status effect on refresh.
    #[must_use]
    pub fn effect(mut self, binding: EffectBinding) -> Self {
        self.properties.push(ArchetypeProperty::Effect(binding));
        self
    }

    /// Apply/remove an effect only when `flag` is enabled/disabled.
    #[must_use]
    pub fn effect_when(mut self, flag: EntityFlag, binding: EffectBinding) -> Self {
        self.properties
            .push(ArchetypeProperty::ConditionalEffect { flag, binding });
        self
    }

    /// Own one typed equipment slot using Sand's canonical item stack model.
    #[must_use]
    pub fn equipment(mut self, binding: EquipmentBinding) -> Self {
        self.properties
            .push(ArchetypeProperty::Equipment(Box::new(binding)));
        self
    }

    /// Equip/clear one owned slot when a typed flag changes.
    #[must_use]
    pub fn equipment_when(mut self, flag: EntityFlag, binding: EquipmentBinding) -> Self {
        let binding = binding.refresh(RefreshPolicy::WhenSourceChanges);
        self.properties
            .push(ArchetypeProperty::ConditionalEquipment {
                flag,
                binding: Box::new(binding),
            });
        self
    }
}

/// One typed native-property declaration retained by an archetype.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ArchetypeProperty {
    /// Current/max health synchronization.
    Health(HealthBinding),
    /// Attribute base binding.
    Attribute(AttributeBinding),
    /// Idempotent named attribute modifier.
    AttributeModifier(AttributeModifierBinding),
    /// Unconditional status effect.
    Effect(EffectBinding),
    /// Flag-driven add/remove effect transition.
    ConditionalEffect {
        /// Source flag.
        flag: EntityFlag,
        /// Effect declaration.
        binding: EffectBinding,
    },
    /// Typed equipment slot.
    Equipment(Box<EquipmentBinding>),
    /// Flag-driven equipment tier.
    ConditionalEquipment {
        /// Source flag.
        flag: EntityFlag,
        /// Owned slot and enabled stack.
        binding: Box<EquipmentBinding>,
    },
    /// Dynamic custom name.
    Name(NameBinding),
    /// Archetype-owned tag.
    Tag(TagBinding),
    /// Flag-driven tag membership.
    ConditionalTag {
        /// Source flag.
        flag: EntityFlag,
        /// Owned tag.
        binding: TagBinding,
    },
    /// Team membership.
    Team(TeamBinding),
    /// Flag-driven team membership.
    ConditionalTeam {
        /// Source flag.
        flag: EntityFlag,
        /// Owned team.
        binding: TeamBinding,
    },
    /// Stable typed NBT property.
    Nbt(EntityNbtBinding),
}

fn is_living<K: KnownEntityKind>() -> bool {
    // Kept type-erased without unstable specialization; the maintained
    // capability registry is the export-time authority.
    matches!(K::LABEL, "player" | "zombie")
}

fn is_mutable_living<K: KnownEntityKind>() -> bool {
    K::LABEL == "zombie"
}

/// Type-erased archetype definition consumed by the export pipeline.
#[derive(Debug, Clone)]
pub struct ArchetypeDefinition {
    /// Archetype identifier.
    pub id: ResourceLocation,
    /// Current archetype version.
    pub version: u32,
    /// Typed entity type.
    pub entity_type: sand_components::EntityTypeId,
    /// Entity kind used in diagnostics.
    pub kind_label: &'static str,
    /// Whether the kind has living capabilities.
    pub living: bool,
    /// Whether direct non-player living mutation is legal.
    pub mutable_living: bool,
    /// State schema.
    pub schema: StateSchema,
    /// Adoption configuration.
    pub adoption: Option<Adoption>,
    /// Reconciliation policy.
    pub reconcile: ReconcilePolicy,
    /// Optional initialization callback.
    pub initialize: Option<FunctionId>,
    /// Optional cleanup callback.
    pub cleanup: Option<FunctionId>,
    /// Ordered migrations.
    pub migrations: Vec<Migration>,
    /// Cached, dirty-driven stat derivations.
    pub derivations: Vec<EntityDerivation>,
    /// State-driven transition subscriptions.
    transitions: Vec<EntityTransitionRule>,
    /// Typed native property declarations.
    pub properties: Vec<ArchetypeProperty>,
}

/// A named numeric derivation cached in a typed entity score.
///
/// Curves use integer fixed-point arithmetic. For native properties whose
/// Minecraft command accepts whole values, choose scale `1`; higher scales
/// preserve fractional values for later score arithmetic.
#[derive(Debug, Clone)]
pub struct EntityDerivation {
    name: String,
    target: crate::entity::state::EntityScore<i32>,
    curve: StatCurve,
    fixed: FixedPoint,
    output: DerivedScoreEncoding,
}

/// Representation written to a derivation's target score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DerivedScoreEncoding {
    /// Divide the fixed-point result by its scale before caching it.
    Whole,
    /// Keep scaled fixed-point units in the target score.
    FixedPoint,
}

/// Direction used by threshold-crossing transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThresholdDirection {
    /// Fire when the score moves from below to at least the threshold.
    Rising,
    /// Fire when the score moves from above to at most the threshold.
    Falling,
}

/// A state change observed for one loaded archetyped entity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityTransition {
    /// Any numeric, enum, or flag value change.
    Changed(EntityTransitionField),
    /// A flag changed from disabled to enabled.
    FlagEnabled(EntityTransitionField),
    /// A flag changed from enabled to disabled.
    FlagDisabled(EntityTransitionField),
    /// An enum changed to one stable encoding.
    EnumChangedTo {
        /// Enum score field.
        field: EntityTransitionField,
        /// Stable enum encoding.
        encoding: i32,
    },
    /// A whole-score threshold was crossed.
    Threshold {
        /// Source score.
        field: EntityTransitionField,
        /// Inclusive boundary.
        value: i32,
        /// Crossing direction.
        direction: ThresholdDirection,
    },
    /// A current/max-health percentage boundary was crossed.
    HealthPercentage {
        /// Current-health score.
        current: EntityTransitionField,
        /// Maximum-health score.
        maximum: EntityTransitionField,
        /// Inclusive percentage in basis points (`10_000 == 100%`).
        basis_points: u16,
        /// Crossing direction.
        direction: ThresholdDirection,
    },
    /// A timer reached zero from a positive value.
    TimerElapsed(EntityTransitionField),
    /// A cooldown reached its ready state.
    CooldownReady(EntityTransitionField),
}

/// Type-erased identity of a typed state field used by a transition plan.
///
/// Construct this through [`EntityTransition`] helpers; the stored objective
/// is generated from schema metadata, never accepted as a raw string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTransitionField {
    objective: String,
}

impl EntityTransitionField {
    fn typed<F: EntityStateField>(field: F) -> Self {
        Self {
            objective: field.objective(),
        }
    }

    fn objective(&self) -> &str {
        &self.objective
    }
}

impl EntityTransition {
    /// Observe any change to a typed field.
    #[must_use]
    pub fn changed<F: EntityStateField>(field: F) -> Self {
        Self::Changed(EntityTransitionField::typed(field))
    }

    /// Observe a flag becoming enabled.
    #[must_use]
    pub fn flag_enabled(field: EntityFlag) -> Self {
        Self::FlagEnabled(EntityTransitionField::typed(field))
    }

    /// Observe a flag becoming disabled.
    #[must_use]
    pub fn flag_disabled(field: EntityFlag) -> Self {
        Self::FlagDisabled(EntityTransitionField::typed(field))
    }

    /// Observe a typed enum becoming one variant.
    #[must_use]
    pub fn enum_changed_to<T: crate::entity::state::EntityEnumValue>(
        field: crate::entity::state::EntityEnum<T>,
        value: T,
    ) -> Self {
        Self::EnumChangedTo {
            field: EntityTransitionField::typed(field),
            encoding: value.encode(),
        }
    }

    /// Observe an inclusive threshold crossing.
    #[must_use]
    pub fn threshold(
        field: impl EntityStateField,
        value: i32,
        direction: ThresholdDirection,
    ) -> Self {
        Self::Threshold {
            field: EntityTransitionField::typed(field),
            value,
            direction,
        }
    }

    /// Observe a health-ratio crossing in basis points.
    #[must_use]
    pub fn health_percentage(
        current: impl EntityStateField,
        maximum: impl EntityStateField,
        basis_points: u16,
        direction: ThresholdDirection,
    ) -> Self {
        Self::HealthPercentage {
            current: EntityTransitionField::typed(current),
            maximum: EntityTransitionField::typed(maximum),
            basis_points,
            direction,
        }
    }

    /// Observe a timer reaching zero.
    #[must_use]
    pub fn timer_elapsed(field: crate::entity::state::EntityTimer) -> Self {
        Self::TimerElapsed(EntityTransitionField::typed(field))
    }

    /// Observe a cooldown becoming ready.
    #[must_use]
    pub fn cooldown_ready(field: crate::entity::state::EntityCooldown) -> Self {
        Self::CooldownReady(EntityTransitionField::typed(field))
    }

    fn field(&self) -> Option<&EntityTransitionField> {
        match self {
            Self::Changed(field)
            | Self::EnumChangedTo { field, .. }
            | Self::Threshold { field, .. }
            | Self::TimerElapsed(field)
            | Self::CooldownReady(field) => Some(field),
            Self::FlagEnabled(field) | Self::FlagDisabled(field) => Some(field),
            Self::HealthPercentage { .. } => None,
        }
    }
}

/// Typed work dispatched by an entity transition.
///
/// [`Self::Run`] composes with the existing event/function infrastructure:
/// the registered function runs with the transitioning entity bound to `@s`
/// and can mutate state, update properties, summon, dispatch VFX, or transform
/// the entity using normal Sand authoring APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityAction {
    /// Call a canonical registered datapack function.
    Run(FunctionId),
    /// Dispatch a typed event function.
    Dispatch(crate::entity::property::EntityEventId),
    /// Add or refresh a typed status effect.
    ApplyEffect(EffectBinding),
    /// Remove a typed status effect.
    RemoveEffect(sand_components::StatusEffectId),
    /// Add a validated entity tag.
    AddTag(crate::entity::property::EntityTag),
    /// Remove a validated entity tag.
    RemoveTag(crate::entity::property::EntityTag),
    /// Remove the current non-player entity.
    Despawn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityTransitionRule {
    transition: EntityTransition,
    action: EntityAction,
}

impl EntityDerivation {
    /// Create a derivation using Sand's default scale of 1000.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        target: crate::entity::state::EntityScore<i32>,
        curve: StatCurve,
    ) -> Self {
        Self {
            name: name.into(),
            target,
            curve,
            fixed: FixedPoint::default(),
            output: DerivedScoreEncoding::Whole,
        }
    }

    /// Select fixed-point scale, rounding, and overflow semantics.
    #[must_use]
    pub fn fixed_point(mut self, fixed: FixedPoint) -> Self {
        self.fixed = fixed;
        self
    }

    /// Keep fixed-point units instead of converting the cached target to a
    /// whole scoreboard value.
    #[must_use]
    pub fn store_fixed_point(mut self) -> Self {
        self.output = DerivedScoreEncoding::FixedPoint;
        self
    }

    /// Stable diagnostic/resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Typed score receiving the cached value.
    #[must_use]
    pub const fn target(&self) -> crate::entity::state::EntityScore<i32> {
        self.target
    }

    /// Curve expression.
    #[must_use]
    pub fn curve(&self) -> &StatCurve {
        &self.curve
    }

    /// Fixed-point representation.
    #[must_use]
    pub const fn fixed(&self) -> FixedPoint {
        self.fixed
    }

    /// Target score representation.
    #[must_use]
    pub const fn output_encoding(&self) -> DerivedScoreEncoding {
        self.output
    }
}

/// Link-time archetype factory submitted by `#[entity_archetype]`.
pub struct EntityArchetypeDescriptor {
    /// Build a fresh type-erased definition for one export.
    pub make: fn() -> Result<ArchetypeDefinition, EntityDiagnostic>,
}

inventory::collect!(EntityArchetypeDescriptor);

pub(crate) fn compile_registered(
    profile: &crate::version::VersionProfile,
) -> Result<Vec<CompiledArchetype>, EntityDiagnostic> {
    let mut definitions = inventory::iter::<EntityArchetypeDescriptor>()
        .map(|descriptor| (descriptor.make)())
        .collect::<Result<Vec<_>, _>>()?;
    definitions.sort_by_key(|definition| definition.id.to_string());
    let mut ids = BTreeSet::new();
    for definition in &definitions {
        if !ids.insert(definition.id.to_string()) {
            return Err(EntityDiagnostic::ResourceCollision {
                resource: definition.id.to_string(),
                first: definition.id.to_string(),
                second: definition.id.to_string(),
            });
        }
    }
    definitions
        .iter()
        .map(|definition| compile_definition(definition, profile))
        .collect()
}

fn compile_definition(
    definition: &ArchetypeDefinition,
    _profile: &crate::version::VersionProfile,
) -> Result<CompiledArchetype, EntityDiagnostic> {
    definition.schema.validate()?;
    validate_definition(definition)?;

    let id = definition.id.to_string();
    let root = generated_root(&id);
    let marker = initialized_tag(&id);
    let external_marker = external_tag(&id);
    let version_objective =
        sand_commands::ObjectiveName::logical(format!("{id}.archetype_version"))
            .as_str()
            .to_string();

    let mut objectives = BTreeSet::new();
    objectives.insert(version_objective.clone());
    for field in definition.schema.fields {
        objectives.insert(objective_name(
            definition.schema.namespace,
            definition.schema.name,
            field.name,
        ));
        objectives.insert(dirty_name(
            definition.schema.namespace,
            definition.schema.name,
            field.name,
        ));
    }

    let mut records = Vec::new();
    let mut functions = BTreeSet::new();
    let load_path = format!("{root}/load");
    functions.insert(load_path.clone());
    records.push(function_record(
        definition.id.namespace(),
        &load_path,
        objectives
            .iter()
            .map(|objective| format!("scoreboard objectives add {objective} dummy"))
            .collect(),
    ));

    let mut provision_commands = Vec::new();
    for (index, field) in definition.schema.fields.iter().enumerate() {
        let objective = objective_name(
            definition.schema.namespace,
            definition.schema.name,
            field.name,
        );
        let dirty = dirty_name(
            definition.schema.namespace,
            definition.schema.name,
            field.name,
        );
        let helper = format!("{root}/initialize/field_{index}");
        functions.insert(helper.clone());
        records.push(function_record(
            definition.id.namespace(),
            &helper,
            vec![
                format!("scoreboard players set @s {objective} {}", field.default),
                format!("scoreboard players set @s {dirty} 1"),
            ],
        ));
        provision_commands.push(format!(
            "execute unless score @s {objective} matches -2147483648.. run function {}:{helper}",
            definition.id.namespace()
        ));
    }
    let provision_path = format!("{root}/provision");
    functions.insert(provision_path.clone());
    records.push(function_record(
        definition.id.namespace(),
        &provision_path,
        provision_commands,
    ));
    let mut initialize_commands = vec![format!(
        "function {}:{provision_path}",
        definition.id.namespace()
    )];

    let derivations = compile_derivations(definition, &root)?;
    objectives.extend(derivations.objectives);
    functions.extend(derivations.functions);
    records.extend(derivations.records);
    if let Some(path) = &derivations.refresh_function {
        initialize_commands.push(format!("function {path}"));
    }

    let mut refresh_sources: Vec<(String, String)> = Vec::new();
    let mut refresh_outputs: Vec<(String, String)> = Vec::new();
    let mut periodic_refreshes: Vec<(String, String, u32)> = Vec::new();
    for (index, property) in definition.properties.iter().enumerate() {
        let compiled = compile_property(definition, property, index, &root, _profile)?;
        objectives.extend(compiled.objectives);
        functions.extend(compiled.functions);
        records.extend(compiled.records);
        if let Some(function) = compiled.initialize_function {
            initialize_commands.push(format!("function {function}"));
        }
        for source in compiled.source_dirty {
            refresh_sources.push((source, compiled.output_dirty.clone()));
        }
        if let Some(function) = compiled.refresh_function {
            refresh_outputs.push((compiled.output_dirty, function));
        }
        if let Some((clock, output, interval)) = compiled.periodic {
            objectives.insert(clock.clone());
            periodic_refreshes.push((clock, output, interval));
        }
    }

    let refresh_path = format!("{root}/refresh");
    if !refresh_outputs.is_empty() {
        refresh_sources.sort();
        refresh_sources.dedup();
        refresh_outputs.sort();
        refresh_outputs.dedup();
        let mut commands = Vec::new();
        for (clock, output, interval) in &periodic_refreshes {
            commands.push(format!("scoreboard players add @s {clock} 1"));
            commands.push(format!(
                "execute if score @s {clock} matches {interval}.. run scoreboard players set @s {output} 1"
            ));
            commands.push(format!(
                "execute if score @s {clock} matches {interval}.. run scoreboard players set @s {clock} 0"
            ));
        }
        for (source, output) in &refresh_sources {
            commands.push(format!(
                "execute if score @s {source} matches 1 run scoreboard players set @s {output} 1"
            ));
        }
        for source in refresh_sources
            .iter()
            .map(|(source, _)| source)
            .collect::<BTreeSet<_>>()
        {
            commands.push(format!("scoreboard players set @s {source} 0"));
        }
        for (output, function) in &refresh_outputs {
            commands.push(format!(
                "execute if score @s {output} matches 1 run function {function}"
            ));
            commands.push(format!("scoreboard players set @s {output} 0"));
        }
        functions.insert(refresh_path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &refresh_path,
            commands,
        ));
    }
    let transitions = compile_transitions(definition, &root)?;
    objectives.extend(transitions.objectives);
    functions.extend(transitions.functions);
    records.extend(transitions.records);
    initialize_commands.extend(transitions.initialize_commands);
    for field in definition.schema.fields {
        initialize_commands.push(format!(
            "scoreboard players set @s {} 0",
            dirty_name(
                definition.schema.namespace,
                definition.schema.name,
                field.name
            )
        ));
    }
    if let Some(callback) = &definition.initialize {
        initialize_commands.push(format!("function {callback}"));
    }
    initialize_commands.push(format!(
        "scoreboard players set @s {version_objective} {}",
        definition.version
    ));
    initialize_commands.push(format!("tag @s add {marker}"));
    let initialize_path = format!("{root}/initialize");
    functions.insert(initialize_path.clone());
    records.push(function_record(
        definition.id.namespace(),
        &initialize_path,
        initialize_commands,
    ));

    let migration_path = format!("{root}/migrate");
    if !definition.migrations.is_empty() {
        let mut commands = Vec::new();
        let mut migrations: Vec<_> = definition.migrations.iter().collect();
        migrations.sort_by_key(|migration| (migration.from, migration.to));
        for migration in migrations {
            let step = format!("{root}/migrate/{}_{}", migration.from, migration.to);
            functions.insert(step.clone());
            records.push(function_record(
                definition.id.namespace(),
                &step,
                vec![
                    format!("function {}", migration.action),
                    format!(
                        "scoreboard players set @s {version_objective} {}",
                        migration.to
                    ),
                ],
            ));
            commands.push(format!(
                "execute if score @s {version_objective} matches {} run function {}:{step}",
                migration.from,
                definition.id.namespace()
            ));
        }
        functions.insert(migration_path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &migration_path,
            commands,
        ));
    }

    let reconcile_path = format!("{root}/reconcile");
    functions.insert(reconcile_path.clone());
    let mut reconcile_commands = vec![format!(
        "function {}:{provision_path}",
        definition.id.namespace()
    )];
    for field in definition.schema.fields {
        if matches!(
            field.kind,
            crate::entity::state::StateFieldKind::Timer
                | crate::entity::state::StateFieldKind::Cooldown
        ) {
            let objective = objective_name(
                definition.schema.namespace,
                definition.schema.name,
                field.name,
            );
            let dirty = dirty_name(
                definition.schema.namespace,
                definition.schema.name,
                field.name,
            );
            reconcile_commands.push(format!(
                "execute if score @s {objective} matches 1.. run scoreboard players set @s {dirty} 1"
            ));
            reconcile_commands.push(format!(
                "execute if score @s {objective} matches 1.. run scoreboard players remove @s {objective} 1"
            ));
        }
    }
    if !definition.migrations.is_empty() {
        reconcile_commands.push(format!(
            "execute unless score @s {version_objective} matches {} run function {}:{migration_path}",
            definition.version,
            definition.id.namespace()
        ));
    }
    reconcile_commands.push(format!(
        "execute unless score @s {version_objective} matches {} run function {}:{initialize_path}",
        definition.version,
        definition.id.namespace()
    ));
    if !refresh_outputs.is_empty() {
        if let Some(path) = &derivations.refresh_function {
            reconcile_commands.push(format!("function {path}"));
        }
        reconcile_commands.push(format!(
            "function {}:{refresh_path}",
            definition.id.namespace()
        ));
    } else if let Some(path) = &derivations.refresh_function {
        reconcile_commands.push(format!("function {path}"));
    }
    if let Some(path) = &transitions.check_function {
        reconcile_commands.push(format!("function {path}"));
    }
    for field in definition.schema.fields {
        reconcile_commands.push(format!(
            "scoreboard players set @s {} 0",
            dirty_name(
                definition.schema.namespace,
                definition.schema.name,
                field.name
            )
        ));
    }
    records.push(function_record(
        definition.id.namespace(),
        &reconcile_path,
        reconcile_commands,
    ));

    let cleanup_path = format!("{root}/cleanup");
    functions.insert(cleanup_path.clone());
    let mut cleanup_commands = Vec::new();
    if let Some(callback) = &definition.cleanup {
        cleanup_commands.push(format!("function {callback}"));
    }
    for property in &definition.properties {
        cleanup_commands.extend(property_cleanup_commands(property));
    }
    for objective in &objectives {
        cleanup_commands.push(format!("scoreboard players reset @s {objective}"));
    }
    cleanup_commands.push(format!("tag @s remove {marker}"));
    cleanup_commands.push(format!("tag @s remove {external_marker}"));
    records.push(function_record(
        definition.id.namespace(),
        &cleanup_path,
        cleanup_commands,
    ));

    let mut tick_functions = Vec::new();
    let mut adoption_selector = None;
    if let Some(adoption) = &definition.adoption {
        let mut selector = format!("@e[type={},tag=!{}", definition.entity_type, marker);
        match adoption.source {
            AdoptionSource::Natural => {
                selector.push_str(&format!(",tag=!{external_marker}"));
            }
            AdoptionSource::External => {
                selector.push_str(&format!(",tag={external_marker}"));
            }
            AdoptionSource::NaturalAndExternal => {}
        }
        let mut required_tags: Vec<_> = adoption
            .required_tags
            .iter()
            .map(|tag| tag.as_str())
            .collect();
        required_tags.sort_unstable();
        required_tags.dedup();
        for tag in required_tags {
            selector.push_str(&format!(",tag={tag}"));
        }
        let mut excluded_tags: Vec<_> = adoption
            .excluded_tags
            .iter()
            .map(|tag| tag.as_str())
            .collect();
        excluded_tags.sort_unstable();
        excluded_tags.dedup();
        for tag in excluded_tags {
            selector.push_str(&format!(",tag=!{tag}"));
        }
        let mut predicates: Vec<_> = adoption.state_predicates.iter().collect();
        predicates.sort_by_key(|predicate| predicate.objective());
        let mut seen_predicates = BTreeSet::new();
        for predicate in &predicates {
            if !seen_predicates.insert(predicate.objective()) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema: definition.schema.id(),
                    field: predicate.objective().into(),
                    detail: "adoption query contains two ranges for one state field".into(),
                });
            }
        }
        if !predicates.is_empty() {
            selector.push_str(",scores={");
            selector.push_str(
                &predicates
                    .iter()
                    .map(|predicate| {
                        format!("{}={}", predicate.objective(), predicate.selector_range())
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            );
            selector.push('}');
        }
        if let Some(distance) = adoption.max_distance {
            selector.push_str(&format!(",distance=..{distance}"));
        }
        selector.push(']');
        adoption_selector = Some(selector.clone());
        let scan_path = format!("{root}/scan");
        functions.insert(scan_path.clone());
        let scan_command = if adoption.special == SpecialEntityPolicy::Exclude {
            format!(
                "execute as {selector} at @s unless data entity @s CustomName run function {}:{initialize_path}",
                definition.id.namespace()
            )
        } else {
            format!(
                "execute as {selector} at @s run function {}:{initialize_path}",
                definition.id.namespace()
            )
        };
        records.push(function_record(
            definition.id.namespace(),
            &scan_path,
            vec![scan_command],
        ));
        let coordinator_path = format!("{root}/tick");
        functions.insert(coordinator_path.clone());
        let interval = adoption.every.get();
        let commands = if interval == 1 {
            vec![format!(
                "function {}:{scan_path}",
                definition.id.namespace()
            )]
        } else {
            let clock = sand_commands::ObjectiveName::logical(format!("{id}.adoption_clock"))
                .as_str()
                .to_string();
            objectives.insert(clock.clone());
            // Add to the already-built load function deterministically.
            if let Some(load) = records
                .iter_mut()
                .find(|record| record.path == load_path && record.dir == "function")
            {
                load.content.push_str(&format!(
                    "\nscoreboard objectives add {clock} dummy\nscoreboard players set #clock {clock} 0"
                ));
            }
            vec![
                format!("scoreboard players add #clock {clock} 1"),
                format!(
                    "execute if score #clock {clock} matches {interval}.. run function {}:{scan_path}",
                    definition.id.namespace()
                ),
                format!(
                    "execute if score #clock {clock} matches {interval}.. run scoreboard players set #clock {clock} 0"
                ),
            ]
        };
        records.push(function_record(
            definition.id.namespace(),
            &coordinator_path,
            commands,
        ));
        tick_functions.push(format!("{}:{coordinator_path}", definition.id.namespace()));
    }

    let has_timers = definition.schema.fields.iter().any(|field| {
        matches!(
            field.kind,
            crate::entity::state::StateFieldKind::Timer
                | crate::entity::state::StateFieldKind::Cooldown
        )
    });
    let property_schedules_scan = !refresh_outputs.is_empty() || has_timers;
    let needs_reconcile_scan = (!matches!(
        definition.reconcile,
        ReconcilePolicy::InitializeOnly | ReconcilePolicy::Manual
    ) && (!definition.properties.is_empty()
        || !definition.derivations.is_empty()
        || !definition.transitions.is_empty()
        || !definition.migrations.is_empty()
        || definition.version > 1))
        || property_schedules_scan;
    if needs_reconcile_scan {
        let path = format!("{root}/reconcile_scan");
        functions.insert(path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &path,
            vec![format!(
                "execute as @e[type={},tag={marker}] at @s run function {}:{reconcile_path}",
                definition.entity_type,
                definition.id.namespace()
            )],
        ));
        if let ReconcilePolicy::Every(interval) = definition.reconcile
            && interval.get() > 1
        {
            let clock = sand_commands::ObjectiveName::logical(format!("{id}.reconciliation_clock"))
                .as_str()
                .to_string();
            objectives.insert(clock.clone());
            if let Some(load) = records
                .iter_mut()
                .find(|record| record.path == load_path && record.dir == "function")
            {
                load.content.push_str(&format!(
                    "\nscoreboard objectives add {clock} dummy\nscoreboard players set #clock {clock} 0"
                ));
            }
            let coordinator = format!("{root}/reconcile_tick");
            functions.insert(coordinator.clone());
            records.push(function_record(
                definition.id.namespace(),
                &coordinator,
                vec![
                    format!("scoreboard players add #clock {clock} 1"),
                    format!(
                        "execute if score #clock {clock} matches {}.. run function {}:{path}",
                        interval.get(),
                        definition.id.namespace()
                    ),
                    format!(
                        "execute if score #clock {clock} matches {}.. run scoreboard players set #clock {clock} 0",
                        interval.get()
                    ),
                ],
            ));
            tick_functions.push(format!("{}:{coordinator}", definition.id.namespace()));
        } else {
            tick_functions.push(format!("{}:{path}", definition.id.namespace()));
        }
    }

    // Property/derivation compilation can add objectives after the load
    // record is first reserved. Materialize the complete sorted set last.
    if let Some(load) = records
        .iter_mut()
        .find(|record| record.path == load_path && record.dir == "function")
    {
        let clock_initializers: Vec<_> = load
            .content
            .lines()
            .filter(|line| line.starts_with("scoreboard players set #clock "))
            .map(str::to_owned)
            .collect();
        load.content = objectives
            .iter()
            .map(|objective| format!("scoreboard objectives add {objective} dummy"))
            .chain(clock_initializers)
            .collect::<Vec<_>>()
            .join("\n");
    }

    records.sort_by(|left, right| {
        (&left.dir, &left.path, &left.namespace).cmp(&(&right.dir, &right.path, &right.namespace))
    });
    Ok(CompiledArchetype {
        records,
        load_functions: vec![format!("{}:{load_path}", definition.id.namespace())],
        tick_functions,
        report: EntityRuntimeReport {
            archetype: id,
            objectives: objectives.into_iter().collect(),
            tags: vec![marker, external_marker],
            functions: functions.into_iter().collect(),
            outer_scans_per_cycle: usize::from(definition.adoption.is_some())
                + usize::from(needs_reconcile_scan),
            adoption_selector,
        },
    })
}

struct TransitionCompilation {
    records: Vec<crate::component::ComponentRecord>,
    functions: Vec<String>,
    objectives: Vec<String>,
    initialize_commands: Vec<String>,
    check_function: Option<String>,
}

fn compile_transitions(
    definition: &ArchetypeDefinition,
    root: &str,
) -> Result<TransitionCompilation, EntityDiagnostic> {
    let mut records = Vec::new();
    let mut functions = Vec::new();
    let mut objectives = Vec::new();
    let mut initialize_commands = Vec::new();
    let mut check_commands = Vec::new();
    for (index, rule) in definition.transitions.iter().enumerate() {
        let action_path = format!("{root}/transition/{index}");
        let action_commands = lower_action(definition, &rule.action, index)?;
        functions.push(action_path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &action_path,
            action_commands,
        ));
        let action = format!("function {}:{action_path}", definition.id.namespace());

        if let EntityTransition::HealthPercentage {
            current,
            maximum,
            basis_points,
            direction,
        } = &rule.transition
        {
            if *basis_points > 10_000 {
                return Err(EntityDiagnostic::InvalidRange {
                    schema: definition.schema.id(),
                    field: format!("transition[{index}]"),
                    range: format!("{basis_points} basis points"),
                });
            }
            for field in [current, maximum] {
                if dirty_for_objective(definition.schema, field.objective()).is_none() {
                    return Err(EntityDiagnostic::InvalidRawExtension {
                        archetype: definition.id.to_string(),
                        extension: format!("transition[{index}]"),
                        detail: format!(
                            "health field objective `{}` is not in this schema",
                            field.objective()
                        ),
                    });
                }
            }
            let percentage = sand_commands::ObjectiveName::logical(format!(
                "{}.transition.{index}.percentage",
                definition.id
            ))
            .as_str()
            .to_string();
            let previous = sand_commands::ObjectiveName::logical(format!(
                "{}.transition.{index}.previous",
                definition.id
            ))
            .as_str()
            .to_string();
            let scale = sand_commands::ObjectiveName::logical(format!(
                "{}.transition.percentage_scale",
                definition.id
            ))
            .as_str()
            .to_string();
            objectives.extend([percentage.clone(), previous.clone(), scale.clone()]);
            let calculate = vec![
                format!("scoreboard players set @s {percentage} 0"),
                format!("scoreboard players set #value {scale} 10000"),
                format!(
                    "execute if score @s {} matches 1.. run scoreboard players operation @s {percentage} = @s {}",
                    maximum.objective(),
                    current.objective()
                ),
                format!(
                    "execute if score @s {} matches 1.. run scoreboard players operation @s {percentage} *= #value {scale}",
                    maximum.objective()
                ),
                format!(
                    "execute if score @s {} matches 1.. run scoreboard players operation @s {percentage} /= @s {}",
                    maximum.objective(),
                    maximum.objective()
                ),
            ];
            initialize_commands.extend(calculate.clone());
            initialize_commands.push(format!(
                "scoreboard players operation @s {previous} = @s {percentage}"
            ));
            check_commands.extend(calculate);
            match direction {
                ThresholdDirection::Rising => check_commands.push(format!(
                    "execute if score @s {percentage} matches {basis_points}.. if score @s {previous} matches ..{} run {action}",
                    basis_points.saturating_sub(1)
                )),
                ThresholdDirection::Falling => check_commands.push(format!(
                    "execute if score @s {percentage} matches ..{basis_points} if score @s {previous} matches {}.. run {action}",
                    basis_points.saturating_add(1)
                )),
            }
            check_commands.push(format!(
                "scoreboard players operation @s {previous} = @s {percentage}"
            ));
            continue;
        }

        let Some(field) = rule.transition.field() else {
            return Err(EntityDiagnostic::InvalidRawExtension {
                archetype: definition.id.to_string(),
                extension: format!("transition[{index}]"),
                detail: "transition has no state field".into(),
            });
        };
        let objective = field.objective();
        if dirty_for_objective(definition.schema, objective).is_none() {
            return Err(EntityDiagnostic::InvalidRawExtension {
                archetype: definition.id.to_string(),
                extension: format!("transition[{index}]"),
                detail: format!("state field objective `{objective}` is not in this schema"),
            });
        }
        let previous = sand_commands::ObjectiveName::logical(format!(
            "{}.transition.{index}.previous",
            definition.id
        ))
        .as_str()
        .to_string();
        objectives.push(previous.clone());
        initialize_commands.push(format!(
            "scoreboard players operation @s {previous} = @s {objective}"
        ));
        match rule.transition {
            EntityTransition::Changed(_) => check_commands.push(format!(
                "execute unless score @s {objective} = @s {previous} run {action}"
            )),
            EntityTransition::FlagEnabled(_) => check_commands.push(format!(
                "execute if score @s {objective} matches 1 unless score @s {previous} matches 1 run {action}"
            )),
            EntityTransition::FlagDisabled(_) => check_commands.push(format!(
                "execute if score @s {objective} matches 0 unless score @s {previous} matches 0 run {action}"
            )),
            EntityTransition::EnumChangedTo { encoding, .. } => check_commands.push(format!(
                "execute if score @s {objective} matches {encoding} unless score @s {previous} matches {encoding} run {action}"
            )),
            EntityTransition::Threshold {
                value,
                direction: ThresholdDirection::Rising,
                ..
            } => check_commands.push(format!(
                "execute if score @s {objective} matches {value}.. if score @s {previous} matches ..{} run {action}",
                value.saturating_sub(1)
            )),
            EntityTransition::Threshold {
                value,
                direction: ThresholdDirection::Falling,
                ..
            } => check_commands.push(format!(
                "execute if score @s {objective} matches ..{value} if score @s {previous} matches {}.. run {action}",
                value.saturating_add(1)
            )),
            EntityTransition::TimerElapsed(_) | EntityTransition::CooldownReady(_) => {
                check_commands.push(format!(
                    "execute if score @s {objective} matches 0 if score @s {previous} matches 1.. run {action}"
                ));
            }
            EntityTransition::HealthPercentage { .. } => {
                return Err(EntityDiagnostic::InvalidRawExtension {
                    archetype: definition.id.to_string(),
                    extension: format!("transition[{index}]"),
                    detail: "health-percentage transition reached scalar lowering".into(),
                });
            }
        }
        check_commands.push(format!(
            "scoreboard players operation @s {previous} = @s {objective}"
        ));
    }
    let check_function = if definition.transitions.is_empty() {
        None
    } else {
        let path = format!("{root}/transitions");
        functions.push(path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &path,
            check_commands,
        ));
        Some(format!("{}:{path}", definition.id.namespace()))
    };
    Ok(TransitionCompilation {
        records,
        functions,
        objectives,
        initialize_commands,
        check_function,
    })
}

fn lower_action(
    definition: &ArchetypeDefinition,
    action: &EntityAction,
    index: usize,
) -> Result<Vec<String>, EntityDiagnostic> {
    match action {
        EntityAction::Run(function) => Ok(vec![format!("function {function}")]),
        EntityAction::Dispatch(event) => Ok(vec![format!("function {}", event.location())]),
        EntityAction::ApplyEffect(binding) => {
            binding.validate(&definition.id)?;
            Ok(vec![format!(
                "effect give @s {} {} {} true",
                binding.effect(),
                binding.duration().get().div_ceil(20),
                binding.amplifier_value()
            )])
        }
        EntityAction::RemoveEffect(effect) => Ok(vec![format!("effect clear @s {effect}")]),
        EntityAction::AddTag(tag) => Ok(vec![format!("tag @s add {}", tag.as_str())]),
        EntityAction::RemoveTag(tag) => Ok(vec![format!("tag @s remove {}", tag.as_str())]),
        EntityAction::Despawn => {
            if definition.kind_label == "player" {
                Err(EntityDiagnostic::UnsupportedCapability {
                    archetype: definition.id.to_string(),
                    entity_kind: definition.kind_label.into(),
                    property: format!("transition[{index}] despawn"),
                })
            } else {
                Ok(vec!["kill @s".into()])
            }
        }
    }
}

struct PropertyCompilation {
    records: Vec<crate::component::ComponentRecord>,
    functions: Vec<String>,
    objectives: Vec<String>,
    initialize_function: Option<String>,
    refresh_function: Option<String>,
    source_dirty: Vec<String>,
    output_dirty: String,
    periodic: Option<(String, String, u32)>,
}

struct DerivationCompilation {
    records: Vec<crate::component::ComponentRecord>,
    functions: Vec<String>,
    objectives: Vec<String>,
    refresh_function: Option<String>,
}

fn compile_derivations(
    definition: &ArchetypeDefinition,
    root: &str,
) -> Result<DerivationCompilation, EntityDiagnostic> {
    use crate::entity::curve::DependencyGraph;

    let id = definition.id.to_string();
    let mut graph = DependencyGraph::new();
    let mut targets = BTreeSet::new();
    for derivation in &definition.derivations {
        let target = derivation.target.objective();
        if dirty_for_objective(definition.schema, &target).is_none() {
            return Err(EntityDiagnostic::InvalidRawExtension {
                archetype: definition.id.to_string(),
                extension: derivation.name.clone(),
                detail: format!("derived target `{target}` is not a field in this entity schema"),
            });
        }
        if !targets.insert(target.clone()) {
            return Err(EntityDiagnostic::DuplicateStateField {
                schema: id,
                field: target,
                detail: "multiple derivations write the same score".into(),
            });
        }
        graph.add_node(target.clone());
        for input in derivation.curve.inputs() {
            graph.add_dependency(input, target.clone());
        }
    }
    let order = graph.topological_order(&definition.id.to_string())?;
    let order_index = order
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut derivations: Vec<_> = definition.derivations.iter().collect();
    derivations.sort_by_key(|derivation| {
        (
            order_index
                .get(&derivation.target.objective())
                .copied()
                .unwrap_or(usize::MAX),
            derivation.name.as_str(),
        )
    });

    let mut records = Vec::new();
    let mut functions = Vec::new();
    let mut objectives = BTreeSet::new();
    let mut refresh_commands = Vec::new();
    for (index, derivation) in derivations.into_iter().enumerate() {
        let target = derivation.target.objective();
        let target_dirty = derivation.target.dirty_objective();
        let derivation_dirty =
            sand_commands::ObjectiveName::logical(format!("{id}.derive.{index}.dirty"))
                .as_str()
                .to_string();
        objectives.insert(derivation_dirty.clone());
        for input in derivation.curve.inputs() {
            let source_dirty = dirty_for_objective(definition.schema, &input).ok_or_else(|| {
                EntityDiagnostic::InvalidRawExtension {
                    archetype: definition.id.to_string(),
                    extension: derivation.name.clone(),
                    detail: format!("curve input `{input}` is not a field in this entity schema"),
                }
            })?;
            refresh_commands.push(format!(
                "execute if score @s {source_dirty} matches 1 run scoreboard players set @s {derivation_dirty} 1"
            ));
        }
        let lowered = derivation.curve.lower_scoreboard(
            &target,
            &format!("{id}.derive.{index}"),
            derivation.fixed,
        )?;
        objectives.extend(lowered.scratch_objectives().iter().cloned());
        let path = format!("{root}/derive/{index}");
        let rendered = render_lowered_curve(definition, &path, &lowered)?;
        objectives.extend(rendered.objectives);
        functions.push(path.clone());
        functions.extend(rendered.functions);
        records.extend(rendered.records);
        let mut commands = rendered.commands;
        if derivation.output == DerivedScoreEncoding::Whole && derivation.fixed.scale() != 1 {
            let mut conversion_objectives = BTreeSet::new();
            append_scaled_division(
                definition,
                &mut conversion_objectives,
                &mut commands,
                &target,
                derivation.fixed.scale(),
                derivation.fixed.rounding(),
                index,
            )?;
            objectives.extend(conversion_objectives);
        }
        commands.push(format!("scoreboard players set @s {target_dirty} 1"));
        records.push(function_record(definition.id.namespace(), &path, commands));
        refresh_commands.push(format!(
            "execute if score @s {derivation_dirty} matches 1 run function {}:{path}",
            definition.id.namespace()
        ));
        refresh_commands.push(format!("scoreboard players set @s {derivation_dirty} 0"));
    }

    let refresh_function = if definition.derivations.is_empty() {
        None
    } else {
        let path = format!("{root}/derive_refresh");
        functions.push(path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &path,
            refresh_commands,
        ));
        Some(format!("{}:{path}", definition.id.namespace()))
    };
    Ok(DerivationCompilation {
        records,
        functions,
        objectives: objectives.into_iter().collect(),
        refresh_function,
    })
}

fn dirty_for_objective(schema: StateSchema, objective: &str) -> Option<String> {
    schema.fields.iter().find_map(|field| {
        (objective_name(schema.namespace, schema.name, field.name) == objective)
            .then(|| dirty_name(schema.namespace, schema.name, field.name))
    })
}

struct RenderedCurve {
    commands: Vec<String>,
    records: Vec<crate::component::ComponentRecord>,
    functions: Vec<String>,
    objectives: Vec<String>,
}

fn render_lowered_curve(
    definition: &ArchetypeDefinition,
    path: &str,
    lowered: &LoweredCurve,
) -> Result<RenderedCurve, EntityDiagnostic> {
    let mut commands = Vec::new();
    let mut records = Vec::new();
    let mut functions = Vec::new();
    let mut objectives = BTreeSet::new();
    for (index, operation) in lowered.operations().iter().enumerate() {
        match operation {
            LoweredCurveOperation::SetConstant { destination, value } => {
                commands.push(format!(
                    "scoreboard players set @s {destination} {}",
                    scoreboard_value(definition, lowered.target_objective(), value.units())?
                ));
            }
            LoweredCurveOperation::Copy {
                destination,
                source,
            } => commands.push(format!(
                "scoreboard players operation @s {destination} = @s {source}"
            )),
            LoweredCurveOperation::ScoreToFixed {
                destination,
                source,
                scale,
                overflow,
            } => {
                require_scoreboard_overflow(definition, lowered, *overflow)?;
                let scale_objective =
                    constant_objective(definition, &format!("curve_input_scale_{index}"), *scale)?;
                objectives.insert(scale_objective.clone());
                commands.push(format!(
                    "scoreboard players operation @s {destination} = @s {source}"
                ));
                commands.push(format!(
                    "scoreboard players set #value {scale_objective} {scale}"
                ));
                commands.push(format!(
                    "scoreboard players operation @s {destination} *= #value {scale_objective}"
                ));
            }
            LoweredCurveOperation::Add {
                destination,
                source,
                overflow,
            } => {
                require_scoreboard_overflow(definition, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} += @s {source}"
                ));
            }
            LoweredCurveOperation::MultiplyFixed {
                destination,
                factor,
                scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(definition, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} *= @s {factor}"
                ));
                append_scaled_division(
                    definition,
                    &mut objectives,
                    &mut commands,
                    destination,
                    *scale,
                    *rounding,
                    index,
                )?;
            }
            LoweredCurveOperation::RatioFixed {
                destination,
                numerator,
                denominator,
                scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(definition, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} = @s {numerator}"
                ));
                let constant = constant_objective(definition, "curve_scale", *scale)?;
                objectives.insert(constant.clone());
                commands.push(format!("scoreboard players set #value {constant} {scale}"));
                commands.push(format!(
                    "scoreboard players operation @s {destination} *= #value {constant}"
                ));
                append_score_division(
                    definition,
                    &mut objectives,
                    &mut commands,
                    destination,
                    denominator,
                    *rounding,
                    index,
                )?;
            }
            LoweredCurveOperation::Clamp {
                destination,
                minimum,
                maximum,
            } => {
                let minimum =
                    scoreboard_value(definition, lowered.target_objective(), minimum.units())?;
                let maximum =
                    scoreboard_value(definition, lowered.target_objective(), maximum.units())?;
                commands.push(format!(
                    "execute if score @s {destination} matches ..{minimum} run scoreboard players set @s {destination} {minimum}"
                ));
                commands.push(format!(
                    "execute if score @s {destination} matches {maximum}.. run scoreboard players set @s {destination} {maximum}"
                ));
            }
            LoweredCurveOperation::SelectStepped {
                destination,
                input,
                bands,
                below,
            } => {
                let mut boundaries = Vec::new();
                let mut values = vec![scoreboard_value(
                    definition,
                    lowered.target_objective(),
                    below.units(),
                )?];
                for (minimum, value) in bands {
                    boundaries.push(scoreboard_value(
                        definition,
                        lowered.target_objective(),
                        minimum.units(),
                    )?);
                    values.push(scoreboard_value(
                        definition,
                        lowered.target_objective(),
                        value.units(),
                    )?);
                }
                let base = format!("{path}/select_{index}");
                build_threshold_tree(
                    definition,
                    &base,
                    input,
                    destination,
                    &boundaries,
                    &values,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", definition.id.namespace()));
            }
            LoweredCurveOperation::SelectPiecewise {
                destination,
                input,
                branches,
                fallback,
            } => {
                let mut boundaries = Vec::new();
                let mut values = Vec::new();
                for (maximum, source) in branches {
                    boundaries.push(scoreboard_value(
                        definition,
                        lowered.target_objective(),
                        maximum.units(),
                    )?);
                    values.push(source.clone());
                }
                values.push(fallback.clone());
                let base = format!("{path}/piecewise_{index}");
                build_piecewise_tree(
                    definition,
                    &base,
                    input,
                    destination,
                    &boundaries,
                    &values,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", definition.id.namespace()));
            }
            LoweredCurveOperation::LookupTable {
                destination,
                input,
                entries,
                fallback,
            } => {
                let mut encoded = Vec::new();
                for (key, value) in entries {
                    encoded.push((
                        scoreboard_value(definition, lowered.target_objective(), *key)?,
                        scoreboard_value(definition, lowered.target_objective(), value.units())?,
                    ));
                }
                let fallback =
                    scoreboard_value(definition, lowered.target_objective(), fallback.units())?;
                let base = format!("{path}/lookup_{index}");
                build_exact_tree(
                    definition,
                    &base,
                    input,
                    destination,
                    &encoded,
                    fallback,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", definition.id.namespace()));
            }
            LoweredCurveOperation::SelectEnum {
                destination,
                input,
                entries,
                fallback,
            } => {
                let mut encoded = Vec::new();
                for (encoding, value) in entries {
                    encoded.push((
                        *encoding,
                        scoreboard_value(definition, lowered.target_objective(), value.units())?,
                    ));
                }
                let fallback =
                    scoreboard_value(definition, lowered.target_objective(), fallback.units())?;
                let base = format!("{path}/enum_{index}");
                build_exact_tree(
                    definition,
                    &base,
                    input,
                    destination,
                    &encoded,
                    fallback,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", definition.id.namespace()));
            }
            LoweredCurveOperation::SelectFlag {
                destination,
                input,
                disabled,
                enabled,
            } => {
                commands.push(format!(
                    "execute if score @s {input} matches 0 run scoreboard players set @s {destination} {}",
                    scoreboard_value(definition, lowered.target_objective(), disabled.units())?
                ));
                commands.push(format!(
                    "execute unless score @s {input} matches 0 run scoreboard players set @s {destination} {}",
                    scoreboard_value(definition, lowered.target_objective(), enabled.units())?
                ));
            }
            LoweredCurveOperation::Custom {
                destination,
                callback,
                inputs: _,
            } => {
                let callback = callback.parse::<FunctionId>().map_err(|error| {
                    EntityDiagnostic::InvalidRawExtension {
                        archetype: definition.id.to_string(),
                        extension: lowered.target_objective().into(),
                        detail: format!(
                            "custom curve callback must be a canonical function ID: {error}"
                        ),
                    }
                })?;
                commands.push(format!(
                    "execute store result score @s {destination} run function {callback}"
                ));
            }
        }
    }
    Ok(RenderedCurve {
        commands,
        records,
        functions,
        objectives: objectives.into_iter().collect(),
    })
}

#[allow(clippy::too_many_arguments)]
fn build_exact_tree(
    definition: &ArchetypeDefinition,
    path: &str,
    input: &str,
    destination: &str,
    entries: &[(i32, i32)],
    fallback: i32,
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    functions.push(path.to_owned());
    let commands = if entries.len() <= 1 {
        let mut commands = vec![format!(
            "scoreboard players set @s {destination} {fallback}"
        )];
        if let Some((key, value)) = entries.first() {
            commands.push(format!(
                "execute if score @s {input} matches {key} run scoreboard players set @s {destination} {value}"
            ));
        }
        commands
    } else {
        let middle = entries.len() / 2;
        let (key, value) = entries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_exact_tree(
            definition,
            &left,
            input,
            destination,
            &entries[..middle],
            fallback,
            records,
            functions,
        );
        build_exact_tree(
            definition,
            &right,
            input,
            destination,
            &entries[middle + 1..],
            fallback,
            records,
            functions,
        );
        let mut commands = Vec::new();
        if key > i32::MIN {
            commands.push(format!(
                "execute if score @s {input} matches ..{} run function {}:{left}",
                key - 1,
                definition.id.namespace()
            ));
        }
        commands.push(format!(
            "execute if score @s {input} matches {key} run scoreboard players set @s {destination} {value}"
        ));
        if key < i32::MAX {
            commands.push(format!(
                "execute if score @s {input} matches {}.. run function {}:{right}",
                key + 1,
                definition.id.namespace()
            ));
        }
        commands
    };
    records.push(function_record(definition.id.namespace(), path, commands));
}

#[allow(clippy::too_many_arguments)]
fn build_threshold_tree(
    definition: &ArchetypeDefinition,
    path: &str,
    input: &str,
    destination: &str,
    boundaries: &[i32],
    values: &[i32],
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    functions.push(path.to_owned());
    let commands = if boundaries.is_empty() {
        vec![format!(
            "scoreboard players set @s {destination} {}",
            values[0]
        )]
    } else {
        let middle = boundaries.len() / 2;
        let boundary = boundaries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_threshold_tree(
            definition,
            &left,
            input,
            destination,
            &boundaries[..middle],
            &values[..middle + 1],
            records,
            functions,
        );
        build_threshold_tree(
            definition,
            &right,
            input,
            destination,
            &boundaries[middle + 1..],
            &values[middle + 1..],
            records,
            functions,
        );
        let mut commands = Vec::new();
        if boundary > i32::MIN {
            commands.push(format!(
                "execute if score @s {input} matches ..{} run function {}:{left}",
                boundary - 1,
                definition.id.namespace()
            ));
        }
        commands.push(format!(
            "execute if score @s {input} matches {boundary}.. run function {}:{right}",
            definition.id.namespace()
        ));
        commands
    };
    records.push(function_record(definition.id.namespace(), path, commands));
}

#[allow(clippy::too_many_arguments)]
fn build_piecewise_tree(
    definition: &ArchetypeDefinition,
    path: &str,
    input: &str,
    destination: &str,
    boundaries: &[i32],
    values: &[String],
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    functions.push(path.to_owned());
    let commands = if boundaries.is_empty() {
        vec![format!(
            "scoreboard players operation @s {destination} = @s {}",
            values[0]
        )]
    } else {
        let middle = boundaries.len() / 2;
        let boundary = boundaries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_piecewise_tree(
            definition,
            &left,
            input,
            destination,
            &boundaries[..middle],
            &values[..middle + 1],
            records,
            functions,
        );
        build_piecewise_tree(
            definition,
            &right,
            input,
            destination,
            &boundaries[middle + 1..],
            &values[middle + 1..],
            records,
            functions,
        );
        let mut commands = vec![format!(
            "execute if score @s {input} matches ..{boundary} run function {}:{left}",
            definition.id.namespace()
        )];
        if boundary < i32::MAX {
            commands.push(format!(
                "execute if score @s {input} matches {}.. run function {}:{right}",
                boundary + 1,
                definition.id.namespace()
            ));
        }
        commands
    };
    records.push(function_record(definition.id.namespace(), path, commands));
}

fn scoreboard_value(
    definition: &ArchetypeDefinition,
    derivation: &str,
    value: i64,
) -> Result<i32, EntityDiagnostic> {
    i32::try_from(value).map_err(|_| EntityDiagnostic::FixedPointOverflow {
        archetype: definition.id.to_string(),
        derivation: derivation.into(),
        detail: format!("fixed-point unit `{value}` does not fit a Minecraft score"),
    })
}

fn require_scoreboard_overflow(
    definition: &ArchetypeDefinition,
    lowered: &LoweredCurve,
    overflow: OverflowPolicy,
) -> Result<(), EntityDiagnostic> {
    if overflow == OverflowPolicy::Error {
        Ok(())
    } else {
        Err(EntityDiagnostic::UnsupportedProfile {
            archetype: definition.id.to_string(),
            property: lowered.target_objective().into(),
            profile: "vanilla-scoreboard".into(),
            reason: "runtime saturating arithmetic is not reliably representable".into(),
        })
    }
}

fn append_scaled_division(
    definition: &ArchetypeDefinition,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    divisor: i64,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    let objective = constant_objective(definition, &format!("curve_divisor_{index}"), divisor)?;
    objectives.insert(objective.clone());
    commands.push(format!(
        "scoreboard players set #value {objective} {divisor}"
    ));
    append_score_division(
        definition,
        objectives,
        commands,
        destination,
        &format!("#value {objective}"),
        rounding,
        index,
    )
}

fn append_score_division(
    definition: &ArchetypeDefinition,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    divisor: &str,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    let divisor = if divisor.starts_with("#value ") {
        divisor.to_owned()
    } else {
        format!("@s {divisor}")
    };
    if divisor.starts_with("@s ") {
        commands.push(format!(
            "execute unless score {divisor} matches 1.. run return fail"
        ));
    }
    let scratch = |role: &str| {
        sand_commands::ObjectiveName::logical(format!(
            "{}.division.{index}.{destination}.{role}",
            definition.id
        ))
        .as_str()
        .to_string()
    };
    let original = scratch("original");
    let product = scratch("product");
    let remainder = scratch("remainder");
    objectives.extend([original.clone(), product.clone(), remainder.clone()]);
    commands.push(format!(
        "scoreboard players operation @s {original} = @s {destination}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {destination} /= {divisor}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {product} = @s {destination}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {product} *= {divisor}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {remainder} = @s {original}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {remainder} -= @s {product}"
    ));
    match rounding {
        RoundingPolicy::Floor => {}
        RoundingPolicy::TowardZero => commands.push(format!(
            "execute if score @s {original} matches ..-1 if score @s {remainder} matches 1.. run scoreboard players add @s {destination} 1"
        )),
        RoundingPolicy::Ceiling => commands.push(format!(
            "execute if score @s {remainder} matches 1.. run scoreboard players add @s {destination} 1"
        )),
        RoundingPolicy::NearestTiesAwayFromZero | RoundingPolicy::NearestTiesToEven => {
            let twice = scratch("twice_remainder");
            let two = constant_objective(definition, "curve_two", 2)?;
            objectives.extend([twice.clone(), two.clone()]);
            commands.push(format!("scoreboard players set #value {two} 2"));
            commands.push(format!(
                "scoreboard players operation @s {twice} = @s {remainder}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {twice} *= #value {two}"
            ));
            commands.push(format!(
                "execute if score @s {twice} > {divisor} run scoreboard players add @s {destination} 1"
            ));
            if rounding == RoundingPolicy::NearestTiesAwayFromZero {
                commands.push(format!(
                    "execute if score @s {original} matches 0.. if score @s {twice} = {divisor} run scoreboard players add @s {destination} 1"
                ));
            } else {
                let parity = scratch("parity");
                objectives.insert(parity.clone());
                commands.push(format!(
                    "scoreboard players operation @s {parity} = @s {destination}"
                ));
                commands.push(format!(
                    "scoreboard players operation @s {parity} %= #value {two}"
                ));
                commands.push(format!(
                    "execute if score @s {twice} = {divisor} unless score @s {parity} matches 0 run scoreboard players add @s {destination} 1"
                ));
            }
        }
    }
    Ok(())
}

fn constant_objective(
    definition: &ArchetypeDefinition,
    role: &str,
    value: i64,
) -> Result<String, EntityDiagnostic> {
    scoreboard_value(definition, role, value)?;
    Ok(
        sand_commands::ObjectiveName::logical(format!("{}.{}.{}", definition.id, role, value))
            .as_str()
            .to_string(),
    )
}

fn compile_property(
    definition: &ArchetypeDefinition,
    property: &ArchetypeProperty,
    index: usize,
    root: &str,
    profile: &crate::version::VersionProfile,
) -> Result<PropertyCompilation, EntityDiagnostic> {
    let id = definition.id.to_string();
    let path = format!("{root}/property/{index}");
    let function = format!("{}:{path}", definition.id.namespace());
    let output_dirty =
        sand_commands::ObjectiveName::logical(format!("{id}.property.{index}.dirty"))
            .as_str()
            .to_string();
    let mut records = Vec::new();
    let mut functions = vec![path.clone()];
    let mut objectives = vec![output_dirty.clone()];
    let mut sources = Vec::new();
    let (commands, ownership, refresh) = match property {
        ArchetypeProperty::Health(binding) => {
            binding.validate(&id)?;
            sources.push(binding.max_health_field().dirty_objective());
            if let Some(current) = binding.current_health_field() {
                sources.push(current.dirty_objective());
            }
            let lowered = lower_health(definition, binding, index, root, profile)?;
            objectives.extend(lowered.objectives);
            functions.extend(lowered.functions);
            records.extend(lowered.records);
            (
                lowered.commands,
                binding.ownership_policy(),
                binding.refresh_policy(),
            )
        }
        ArchetypeProperty::Attribute(binding) => {
            binding.validate(&id)?;
            if let NumericPropertySource::StateScore {
                dirty_objective, ..
            } = binding.source()
            {
                sources.push(dirty_objective.clone());
            }
            let lowered = lower_attribute(definition, binding, index, root, profile)?;
            functions.extend(lowered.functions);
            records.extend(lowered.records);
            (
                lowered.commands,
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::AttributeModifier(binding) => {
            binding.validate(&id)?;
            if let NumericPropertySource::StateScore {
                dirty_objective, ..
            } = binding.source()
            {
                sources.push(dirty_objective.clone());
            }
            let lowered = lower_attribute_modifier(definition, binding, index, root, profile)?;
            functions.extend(lowered.functions);
            records.extend(lowered.records);
            (
                lowered.commands,
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::Effect(binding) => {
            binding.validate(&id)?;
            let seconds = binding.duration().get().div_ceil(20);
            (
                vec![format!(
                    "effect give @s {} {seconds} {} true",
                    binding.effect(),
                    binding.amplifier_value()
                )],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::ConditionalEffect { flag, binding } => {
            binding.validate(&id)?;
            sources.push(flag.dirty_objective());
            let seconds = binding.duration().get().div_ceil(20);
            (
                vec![
                    format!(
                        "execute if score @s {} matches 1 run effect give @s {} {seconds} {} true",
                        flag.objective(),
                        binding.effect(),
                        binding.amplifier_value()
                    ),
                    format!(
                        "execute unless score @s {} matches 1 run effect clear @s {}",
                        flag.objective(),
                        binding.effect()
                    ),
                ],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::Equipment(binding) => {
            binding.validate(&id)?;
            binding
                .stack()
                .validate()
                .map_err(|error| EntityDiagnostic::InvalidRawExtension {
                    archetype: id.clone(),
                    extension: binding.property_key().to_string(),
                    detail: error.to_string(),
                })?;
            let slot = equipment_slot(binding.slot());
            let command = format!(
                "item replace entity @s {slot} with {} {}",
                binding.stack(),
                binding.stack().count_value()
            );
            (
                vec![
                    if binding.ownership_policy() == OwnershipPolicy::InitializeMissing {
                        format!("execute unless items entity @s {slot} * run {command}")
                    } else {
                        command
                    },
                ],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::ConditionalEquipment { flag, binding } => {
            binding.validate(&id)?;
            binding
                .stack()
                .validate()
                .map_err(|error| EntityDiagnostic::InvalidRawExtension {
                    archetype: id.clone(),
                    extension: binding.property_key().to_string(),
                    detail: error.to_string(),
                })?;
            sources.push(flag.dirty_objective());
            let slot = equipment_slot(binding.slot());
            (
                vec![
                    format!(
                        "execute if score @s {} matches 1 run item replace entity @s {slot} with {} {}",
                        flag.objective(),
                        binding.stack(),
                        binding.stack().count_value()
                    ),
                    format!(
                        "execute unless score @s {} matches 1 run item replace entity @s {slot} with minecraft:air",
                        flag.objective()
                    ),
                ],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::Name(binding) => {
            binding.validate(&id)?;
            for segment in binding.text().segments() {
                match segment {
                    EntityTextSegment::Literal { .. } => {}
                    EntityTextSegment::Numeric {
                        dirty_objective, ..
                    }
                    | EntityTextSegment::Enum {
                        dirty_objective, ..
                    }
                    | EntityTextSegment::Flag {
                        dirty_objective, ..
                    } => sources.push(dirty_objective.clone()),
                }
            }
            let lowered = lower_name(definition, binding, index, root, profile)?;
            functions.extend(lowered.functions);
            records.extend(lowered.records);
            (
                lowered.commands,
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::Tag(binding) => {
            binding.validate(&id)?;
            (
                vec![format!("tag @s add {}", binding.tag().as_str())],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::ConditionalTag { flag, binding } => {
            binding.validate(&id)?;
            sources.push(flag.dirty_objective());
            (
                vec![
                    format!(
                        "execute if score @s {} matches 1 run tag @s add {}",
                        flag.objective(),
                        binding.tag().as_str()
                    ),
                    format!(
                        "execute unless score @s {} matches 1 run tag @s remove {}",
                        flag.objective(),
                        binding.tag().as_str()
                    ),
                ],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::Team(binding) => {
            binding.validate(&id)?;
            let command = format!("team join {} @s", binding.team().as_str());
            (
                vec![
                    if binding.ownership_policy() == OwnershipPolicy::InitializeMissing {
                        format!("execute if entity @s[team=] run {command}")
                    } else {
                        command
                    },
                ],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::ConditionalTeam { flag, binding } => {
            binding.validate(&id)?;
            sources.push(flag.dirty_objective());
            (
                vec![
                    format!(
                        "execute if score @s {} matches 1 run team join {} @s",
                        flag.objective(),
                        binding.team().as_str()
                    ),
                    format!(
                        "execute unless score @s {} matches 1 run team leave @s",
                        flag.objective()
                    ),
                ],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
        ArchetypeProperty::Nbt(binding) => {
            binding.validate_for::<crate::entity::ZombieKind>(&id)?;
            (
                vec![format!(
                    "data modify entity @s {} set value {}",
                    binding.property().path(),
                    render_nbt_value(binding.value())
                )],
                binding.ownership_policy(),
                binding.refresh_policy().clone(),
            )
        }
    };

    records.push(function_record(definition.id.namespace(), &path, commands));
    let writable = ownership.claims_write_ownership();
    let initialize_function = writable.then(|| function.clone());
    let observation_interval = match property {
        ArchetypeProperty::Health(binding) => binding.observation_interval(),
        _ => None,
    };
    let automatic_refresh = writable
        && (matches!(
            refresh,
            RefreshPolicy::WhenSourceChanges | RefreshPolicy::Every(_)
        ) || observation_interval.is_some());
    let periodic = match (refresh, observation_interval) {
        (RefreshPolicy::Every(ticks), _) if writable => {
            let clock =
                sand_commands::ObjectiveName::logical(format!("{id}.property.{index}.clock"))
                    .as_str()
                    .to_string();
            Some((clock, output_dirty.clone(), ticks.get()))
        }
        (_, Some(ticks)) if writable => {
            let clock =
                sand_commands::ObjectiveName::logical(format!("{id}.property.{index}.clock"))
                    .as_str()
                    .to_string();
            Some((clock, output_dirty.clone(), ticks.get()))
        }
        _ => None,
    };
    Ok(PropertyCompilation {
        records,
        functions,
        objectives,
        initialize_function,
        refresh_function: automatic_refresh.then_some(function),
        source_dirty: if automatic_refresh {
            sources
        } else {
            Vec::new()
        },
        output_dirty,
        periodic,
    })
}

struct NativeLowering {
    commands: Vec<String>,
    records: Vec<crate::component::ComponentRecord>,
    functions: Vec<String>,
    objectives: Vec<String>,
}

fn lower_attribute(
    definition: &ArchetypeDefinition,
    binding: &AttributeBinding,
    index: usize,
    root: &str,
    profile: &crate::version::VersionProfile,
) -> Result<NativeLowering, EntityDiagnostic> {
    match binding.source() {
        NumericPropertySource::Fixed { units, scale } => Ok(NativeLowering {
            commands: vec![format!(
                "attribute @s {} base set {}",
                binding.attribute().as_str(),
                *units as f64 / f64::from(*scale)
            )],
            records: Vec::new(),
            functions: Vec::new(),
            objectives: Vec::new(),
        }),
        NumericPropertySource::StateScore { objective, .. } => {
            require_macros(
                definition,
                profile,
                &format!("{root}/property/{index}/macro"),
            )?;
            let helper = format!("{root}/property/{index}/macro");
            let storage = format!("{}:__sand_entity", definition.id.namespace());
            let args = format!(
                "args.p{:012x}.{index}",
                stable_hash(&definition.id.to_string()) & 0xff_ffff_ffff_ffff
            );
            Ok(NativeLowering {
                commands: vec![
                    format!(
                        "execute store result storage {storage} {args}.value int 1 run scoreboard players get @s {objective}"
                    ),
                    format!(
                        "function {}:{helper} with storage {storage} {args}",
                        definition.id.namespace()
                    ),
                    format!("data remove storage {storage} {args}"),
                    format!("data remove storage {storage} args"),
                ],
                records: vec![function_record(
                    definition.id.namespace(),
                    &helper,
                    vec![format!(
                        "$attribute @s {} base set $(value)",
                        binding.attribute().as_str()
                    )],
                )],
                functions: vec![helper],
                objectives: Vec::new(),
            })
        }
    }
}

fn lower_attribute_modifier(
    definition: &ArchetypeDefinition,
    binding: &AttributeModifierBinding,
    index: usize,
    root: &str,
    profile: &crate::version::VersionProfile,
) -> Result<NativeLowering, EntityDiagnostic> {
    let remove = format!(
        "attribute @s {} modifier remove {}",
        binding.attribute().as_str(),
        binding.id()
    );
    match binding.source() {
        NumericPropertySource::Fixed { units, scale } => Ok(NativeLowering {
            commands: vec![
                remove,
                format!(
                    "attribute @s {} modifier add {} {} {}",
                    binding.attribute().as_str(),
                    binding.id(),
                    *units as f64 / f64::from(*scale),
                    binding.operation().as_str()
                ),
            ],
            records: Vec::new(),
            functions: Vec::new(),
            objectives: Vec::new(),
        }),
        NumericPropertySource::StateScore { objective, .. } => {
            let helper = format!("{root}/property/{index}/modifier_macro");
            require_macros(definition, profile, &helper)?;
            let storage = format!("{}:__sand_entity", definition.id.namespace());
            let args = format!(
                "args.m{:012x}.{index}",
                stable_hash(&definition.id.to_string()) & 0xff_ffff_ffff_ffff
            );
            Ok(NativeLowering {
                commands: vec![
                    remove,
                    format!(
                        "execute store result storage {storage} {args}.value int 1 run scoreboard players get @s {objective}"
                    ),
                    format!(
                        "function {}:{helper} with storage {storage} {args}",
                        definition.id.namespace()
                    ),
                    format!("data remove storage {storage} {args}"),
                    format!("data remove storage {storage} args"),
                ],
                records: vec![function_record(
                    definition.id.namespace(),
                    &helper,
                    vec![format!(
                        "$attribute @s {} modifier add {} $(value) {}",
                        binding.attribute().as_str(),
                        binding.id(),
                        binding.operation().as_str()
                    )],
                )],
                functions: vec![helper],
                objectives: Vec::new(),
            })
        }
    }
}

fn lower_health(
    definition: &ArchetypeDefinition,
    binding: &HealthBinding,
    index: usize,
    root: &str,
    profile: &crate::version::VersionProfile,
) -> Result<NativeLowering, EntityDiagnostic> {
    let helper = format!("{root}/property/{index}/max_health_macro");
    require_macros(definition, profile, &helper)?;
    let id = definition.id.to_string();
    let old_max = sand_commands::ObjectiveName::logical(format!("{id}.health.old_max"))
        .as_str()
        .to_string();
    let old_current = sand_commands::ObjectiveName::logical(format!("{id}.health.old_current"))
        .as_str()
        .to_string();
    let new_current = sand_commands::ObjectiveName::logical(format!("{id}.health.new_current"))
        .as_str()
        .to_string();
    let max = binding.max_health_field().objective();
    let storage = format!("{}:__sand_entity", definition.id.namespace());
    let args = format!(
        "args.h{:012x}.{index}",
        stable_hash(&id) & 0xff_ffff_ffff_ffff
    );
    let mut commands = vec![
        format!(
            "execute store result score @s {old_max} run attribute @s minecraft:max_health get 1"
        ),
        format!("execute store result score @s {old_current} run data get entity @s Health 1"),
        format!(
            "execute store result storage {storage} {args}.value int 1 run scoreboard players get @s {max}"
        ),
        format!(
            "function {}:{helper} with storage {storage} {args}",
            definition.id.namespace()
        ),
        format!("data remove storage {storage} {args}"),
        format!("data remove storage {storage} args"),
        format!("scoreboard players operation @s {new_current} = @s {old_current}"),
    ];
    match binding.resize_policy() {
        HealthResizePolicy::PreserveAbsolute => {
            commands.push(format!(
                "scoreboard players operation @s {new_current} < @s {max}"
            ));
        }
        HealthResizePolicy::PreserveRatio => {
            commands.push(format!(
                "scoreboard players operation @s {new_current} *= @s {max}"
            ));
            commands.push(format!(
                "execute if score @s {old_max} matches 1.. run scoreboard players operation @s {new_current} /= @s {old_max}"
            ));
        }
        HealthResizePolicy::Refill => {
            commands.push(format!(
                "scoreboard players operation @s {new_current} = @s {max}"
            ));
        }
    }
    if let Some(current) = binding.current_health_field() {
        match binding.current_health_sync() {
            CurrentHealthSync::ApplyState => commands.push(format!(
                "scoreboard players operation @s {new_current} = @s {}",
                current.objective()
            )),
            CurrentHealthSync::ObserveNative => {
                commands.push(format!(
                    "scoreboard players operation @s {} = @s {new_current}",
                    current.objective()
                ));
                commands.push(format!(
                    "scoreboard players set @s {} 1",
                    current.dirty_objective()
                ));
            }
            CurrentHealthSync::Bidirectional => {
                commands.push(format!(
                    "execute if score @s {} matches 1 run scoreboard players operation @s {new_current} = @s {}",
                    current.dirty_objective(),
                    current.objective()
                ));
                commands.push(format!(
                    "execute unless score @s {} matches 1 run scoreboard players operation @s {} = @s {new_current}",
                    current.dirty_objective(),
                    current.objective()
                ));
                commands.push(format!(
                    "execute unless score @s {} matches 1 run scoreboard players set @s {} 1",
                    current.dirty_objective(),
                    current.dirty_objective()
                ));
            }
            CurrentHealthSync::None => {}
        }
    }
    commands.push(format!(
        "execute store result entity @s Health float 1 run scoreboard players get @s {new_current}"
    ));
    Ok(NativeLowering {
        commands,
        records: vec![function_record(
            definition.id.namespace(),
            &helper,
            vec!["$attribute @s minecraft:max_health base set $(value)".into()],
        )],
        functions: vec![helper],
        objectives: vec![old_max, old_current, new_current],
    })
}

fn lower_name(
    definition: &ArchetypeDefinition,
    binding: &NameBinding,
    index: usize,
    root: &str,
    profile: &crate::version::VersionProfile,
) -> Result<NativeLowering, EntityDiagnostic> {
    let helper = format!("{root}/property/{index}/name_macro");
    let storage = format!("{}:__sand_entity", definition.id.namespace());
    let args = format!(
        "args.n{:012x}.{index}",
        stable_hash(&definition.id.to_string()) & 0xff_ffff_ffff_ffff
    );
    let mut setup = Vec::new();
    let mut rendered = Vec::new();
    let mut dynamic = false;
    for (segment_index, segment) in binding.text().segments().iter().enumerate() {
        let color = match segment {
            EntityTextSegment::Literal { color, .. }
            | EntityTextSegment::Numeric { color, .. }
            | EntityTextSegment::Enum { color, .. }
            | EntityTextSegment::Flag { color, .. } => color
                .as_ref()
                .map(|color| format!(",color:\"{color}\""))
                .unwrap_or_default(),
        };
        match segment {
            EntityTextSegment::Literal { text, .. } => rendered.push(format!(
                "{{text:{}{color}}}",
                serde_json::to_string(text).expect("serializing a Rust string cannot fail")
            )),
            EntityTextSegment::Numeric { objective, .. } => {
                dynamic = true;
                setup.push(format!(
                    "execute store result storage {storage} {args}.v{segment_index} int 1 run scoreboard players get @s {objective}"
                ));
                rendered.push(format!("{{text:\"$(v{segment_index})\"{color}}}"));
            }
            EntityTextSegment::Enum {
                objective,
                variants,
                ..
            } => {
                dynamic = true;
                for (score, value) in variants {
                    setup.push(format!(
                        "execute if score @s {objective} matches {score} run data modify storage {storage} {args}.v{segment_index} set value {}",
                        serde_json::to_string(value)
                            .expect("serializing a Rust string cannot fail")
                    ));
                }
                rendered.push(format!("{{text:$(v{segment_index}){color}}}"));
            }
            EntityTextSegment::Flag {
                objective,
                disabled,
                enabled,
                ..
            } => {
                dynamic = true;
                setup.push(format!(
                    "execute if score @s {objective} matches 0 run data modify storage {storage} {args}.v{segment_index} set value {}",
                    serde_json::to_string(disabled)
                        .expect("serializing a Rust string cannot fail")
                ));
                setup.push(format!(
                    "execute if score @s {objective} matches 1 run data modify storage {storage} {args}.v{segment_index} set value {}",
                    serde_json::to_string(enabled)
                        .expect("serializing a Rust string cannot fail")
                ));
                rendered.push(format!("{{text:$(v{segment_index}){color}}}"));
            }
        }
    }
    let value = format!("{{text:\"\",extra:[{}]}}", rendered.join(","));
    let visible = i32::from(binding.is_visible());
    if dynamic {
        require_macros(definition, profile, &helper)?;
        setup.push(format!(
            "function {}:{helper} with storage {storage} {args}",
            definition.id.namespace()
        ));
        setup.push(format!("data remove storage {storage} {args}"));
        setup.push(format!("data remove storage {storage} args"));
        setup.push(format!(
            "data modify entity @s CustomNameVisible set value {visible}b"
        ));
        Ok(NativeLowering {
            commands: setup,
            records: vec![function_record(
                definition.id.namespace(),
                &helper,
                vec![format!(
                    "$data modify entity @s CustomName set value {value}"
                )],
            )],
            functions: vec![helper],
            objectives: Vec::new(),
        })
    } else {
        Ok(NativeLowering {
            commands: vec![
                format!("data modify entity @s CustomName set value {value}"),
                format!("data modify entity @s CustomNameVisible set value {visible}b"),
            ],
            records: Vec::new(),
            functions: Vec::new(),
            objectives: Vec::new(),
        })
    }
}

fn require_macros(
    definition: &ArchetypeDefinition,
    profile: &crate::version::VersionProfile,
    resource: &str,
) -> Result<(), EntityDiagnostic> {
    if profile.supports(crate::version::VersionFeature::FunctionMacros) {
        Ok(())
    } else {
        Err(EntityDiagnostic::UnsupportedFunctionMacro {
            archetype: definition.id.to_string(),
            resource: format!("{}:{resource}", definition.id.namespace()),
            profile: profile.resolved_name().to_owned(),
        })
    }
}

fn equipment_slot(slot: sand_components::EquipmentSlot) -> &'static str {
    match slot {
        sand_components::EquipmentSlot::Head => "armor.head",
        sand_components::EquipmentSlot::Chest => "armor.chest",
        sand_components::EquipmentSlot::Legs => "armor.legs",
        sand_components::EquipmentSlot::Feet => "armor.feet",
        sand_components::EquipmentSlot::Body => "armor.body",
        sand_components::EquipmentSlot::Mainhand => "weapon.mainhand",
        sand_components::EquipmentSlot::Offhand => "weapon.offhand",
    }
}

fn render_nbt_value(value: &EntityNbtValue) -> String {
    match value {
        EntityNbtValue::Boolean(value) => format!("{}b", i32::from(*value)),
        EntityNbtValue::Integer(value) => value.to_string(),
        EntityNbtValue::Fixed { units, scale } => {
            format!("{}f", *units as f64 / f64::from(*scale))
        }
    }
}

fn property_cleanup_commands(property: &ArchetypeProperty) -> Vec<String> {
    let owned = |policy: OwnershipPolicy| {
        matches!(
            policy,
            OwnershipPolicy::Exact | OwnershipPolicy::ReconcileWhenDirty
        )
    };
    match property {
        ArchetypeProperty::Health(_) | ArchetypeProperty::Attribute(_) => Vec::new(),
        ArchetypeProperty::AttributeModifier(binding) if owned(binding.ownership_policy()) => {
            vec![format!(
                "attribute @s {} modifier remove {}",
                binding.attribute().as_str(),
                binding.id()
            )]
        }
        ArchetypeProperty::Effect(binding)
        | ArchetypeProperty::ConditionalEffect { binding, .. }
            if owned(binding.ownership_policy()) =>
        {
            vec![format!("effect clear @s {}", binding.effect())]
        }
        ArchetypeProperty::Equipment(binding)
        | ArchetypeProperty::ConditionalEquipment { binding, .. }
            if owned(binding.ownership_policy()) =>
        {
            vec![format!(
                "item replace entity @s {} with minecraft:air",
                equipment_slot(binding.slot())
            )]
        }
        ArchetypeProperty::Name(binding) if owned(binding.ownership_policy()) => vec![
            "data remove entity @s CustomName".into(),
            "data remove entity @s CustomNameVisible".into(),
        ],
        ArchetypeProperty::Tag(binding) | ArchetypeProperty::ConditionalTag { binding, .. }
            if owned(binding.ownership_policy()) =>
        {
            vec![format!("tag @s remove {}", binding.tag().as_str())]
        }
        ArchetypeProperty::Team(binding) | ArchetypeProperty::ConditionalTeam { binding, .. }
            if owned(binding.ownership_policy()) =>
        {
            vec!["team leave @s".into()]
        }
        ArchetypeProperty::Nbt(binding) if owned(binding.ownership_policy()) => {
            vec![format!(
                "data remove entity @s {}",
                binding.property().path()
            )]
        }
        _ => Vec::new(),
    }
}

fn validate_definition(definition: &ArchetypeDefinition) -> Result<(), EntityDiagnostic> {
    let id = definition.id.to_string();
    if definition.version == 0 || definition.schema.version == 0 {
        return Err(EntityDiagnostic::InvalidRawExtension {
            archetype: id,
            extension: "version".into(),
            detail: "version zero is reserved for uninitialized entities".into(),
        });
    }
    if let Some(adoption) = &definition.adoption {
        if adoption.every.get() == 0 {
            return Err(EntityDiagnostic::InvalidRefreshInterval {
                archetype: id,
                property: "adoption".into(),
            });
        }
        if adoption
            .max_distance
            .is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(EntityDiagnostic::UnconstrainedAdoption {
                archetype: id,
                detail: "adoption radius must be finite and greater than zero".into(),
            });
        }
    }
    match definition.reconcile {
        ReconcilePolicy::Every(ticks) if ticks.get() == 0 => {
            return Err(EntityDiagnostic::InvalidRefreshInterval {
                archetype: id,
                property: "reconciliation".into(),
            });
        }
        _ => {}
    }
    let ownership: Vec<(NativePropertyKey, OwnershipPolicy, String)> = definition
        .properties
        .iter()
        .enumerate()
        .map(|(index, property)| {
            let (key, policy) = match property {
                ArchetypeProperty::Health(binding) => {
                    (NativePropertyKey::Health, binding.ownership_policy())
                }
                ArchetypeProperty::Attribute(binding) => {
                    (binding.property_key(), binding.ownership_policy())
                }
                ArchetypeProperty::AttributeModifier(binding) => {
                    (binding.property_key(), binding.ownership_policy())
                }
                ArchetypeProperty::Effect(binding)
                | ArchetypeProperty::ConditionalEffect { binding, .. } => {
                    (binding.property_key(), binding.ownership_policy())
                }
                ArchetypeProperty::Equipment(binding)
                | ArchetypeProperty::ConditionalEquipment { binding, .. } => {
                    (binding.property_key(), binding.ownership_policy())
                }
                ArchetypeProperty::Name(binding) => {
                    (NativePropertyKey::Name, binding.ownership_policy())
                }
                ArchetypeProperty::Tag(binding)
                | ArchetypeProperty::ConditionalTag { binding, .. } => {
                    (binding.property_key(), binding.ownership_policy())
                }
                ArchetypeProperty::Team(binding)
                | ArchetypeProperty::ConditionalTeam { binding, .. } => {
                    (binding.property_key(), binding.ownership_policy())
                }
                ArchetypeProperty::Nbt(binding) => {
                    (binding.property_key(), binding.ownership_policy())
                }
            };
            (key, policy, format!("property[{index}]"))
        })
        .collect();
    validate_native_ownership(
        &id,
        ownership
            .iter()
            .map(|(key, policy, owner)| (key, *policy, owner.as_str())),
    )?;

    for property in &definition.properties {
        let requires_mutable_living = matches!(
            property,
            ArchetypeProperty::Health(_)
                | ArchetypeProperty::Attribute(_)
                | ArchetypeProperty::AttributeModifier(_)
                | ArchetypeProperty::Effect(_)
                | ArchetypeProperty::ConditionalEffect { .. }
                | ArchetypeProperty::Equipment(_)
                | ArchetypeProperty::ConditionalEquipment { .. }
        );
        if requires_mutable_living && !definition.mutable_living {
            return Err(EntityDiagnostic::UnsupportedCapability {
                archetype: id.clone(),
                entity_kind: definition.kind_label.into(),
                property: "mutable living property".into(),
            });
        }
        if matches!(property, ArchetypeProperty::Nbt(_)) && definition.kind_label == "player" {
            return Err(EntityDiagnostic::UnsafePlayerMutation {
                archetype: id.clone(),
                property: "entity NBT".into(),
            });
        }
    }
    if definition.migrations.is_empty() {
        return Ok(());
    }
    let mut migrations = definition.migrations.clone();
    migrations.sort_by_key(|migration| (migration.from, migration.to));
    let mut current = migrations[0].from;
    let mut seen_from = BTreeSet::new();
    for migration in migrations {
        if !seen_from.insert(migration.from)
            || migration.from != current
            || migration.to <= migration.from
        {
            return Err(EntityDiagnostic::MissingMigrationPath {
                archetype: id,
                from: current,
                to: definition.version,
            });
        }
        current = migration.to;
    }
    if current != definition.version {
        return Err(EntityDiagnostic::MissingMigrationPath {
            archetype: id,
            from: current,
            to: definition.version,
        });
    }
    Ok(())
}

fn function_record(
    namespace: &str,
    path: &str,
    commands: Vec<String>,
) -> crate::component::ComponentRecord {
    crate::component::ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".into(),
        path: path.to_string(),
        ext: "mcfunction".into(),
        content_type: "text".into(),
        content: commands.join("\n"),
    }
}

fn initialized_tag(id: &str) -> String {
    format!("__sand.a.{:012x}", stable_hash(id) & 0xff_ffff_ffff_ffff)
}

fn external_tag(id: &str) -> String {
    format!(
        "__sand.external.{:012x}",
        stable_hash(id) & 0xff_ffff_ffff_ffff
    )
}

fn generated_root(id: &str) -> String {
    format!(
        "__sand_entity/{:012x}",
        stable_hash(id) & 0xff_ffff_ffff_ffff
    )
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ZombieKind;
    use crate::entity::state::{EntityFlag, EntityScore, StateFieldDescriptor, StateFieldKind};

    struct MobState;
    static FIELDS: &[StateFieldDescriptor] = &[
        StateFieldDescriptor::new("level", StateFieldKind::Score, 1, Some((1, 100))),
        StateFieldDescriptor::new("health", StateFieldKind::Score, 20, Some((1, 2_000))),
        StateFieldDescriptor::new("sick", StateFieldKind::Flag, 0, Some((0, 1))),
    ];
    impl EntityState for MobState {
        fn schema() -> StateSchema {
            StateSchema {
                namespace: "rpg",
                name: "mob",
                version: 2,
                fields: FIELDS,
            }
        }
    }
    const LEVEL: EntityScore<i32> = EntityScore::new("rpg", "mob", "level", 1, Some((1, 100)));
    const HEALTH: EntityScore<i32> = EntityScore::new("rpg", "mob", "health", 20, Some((1, 2_000)));
    const SICK: EntityFlag = EntityFlag::new("rpg", "mob", "sick", false);

    fn profile() -> crate::version::VersionProfile {
        crate::version::VersionProfile::resolve(
            &crate::version::MinecraftVersion::parse("26.2").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn compile_is_repeat_deterministic_and_initialization_marks_last() {
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "plagued_zombie").unwrap(),
        )
        .adopt(Adoption::natural_and_external());
        let first = compile_definition(&archetype.definition(), &profile()).unwrap();
        let second = compile_definition(&archetype.definition(), &profile()).unwrap();
        assert_eq!(first.report, second.report);
        assert_eq!(first.records, second.records);
        for field in [LEVEL.objective(), HEALTH.objective(), SICK.objective()] {
            assert!(first.report.objectives.contains(&field));
        }
        for dirty in [
            LEVEL.dirty_objective(),
            HEALTH.dirty_objective(),
            SICK.dirty_objective(),
        ] {
            assert!(first.report.objectives.contains(&dirty));
        }
        let init = first
            .records
            .iter()
            .find(|record| record.path.ends_with("/initialize"))
            .unwrap();
        assert!(
            init.content
                .lines()
                .last()
                .unwrap()
                .starts_with("tag @s add ")
        );
    }

    #[test]
    fn no_subscriptions_has_no_tick_runtime() {
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "manual").unwrap(),
        )
        .reconcile(ReconcilePolicy::Manual);
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        assert!(compiled.tick_functions.is_empty());
        assert_eq!(compiled.report.outer_scans_per_cycle, 0);
    }

    #[test]
    fn migration_gap_is_structured_error() {
        let function = "rpg:migrate".parse::<FunctionId>().unwrap();
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "bad_migration").unwrap(),
        )
        .migration(Migration::new(1, 2, function.clone()))
        .migration(Migration::new(3, 4, function));
        let error = compile_definition(&archetype.definition(), &profile()).unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-MIGRATION-GAP");
    }

    #[test]
    fn summon_uses_direct_typed_execute_summon_without_scratch_identity() {
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "summoned").unwrap(),
        );
        let commands = archetype.summon();
        assert_eq!(
            commands,
            vec![format!(
                "execute summon minecraft:zombie run {}",
                archetype.attach()
            )]
        );
    }

    #[test]
    fn derivation_property_and_transition_form_one_dirty_pipeline() {
        let fixed = FixedPoint::new(1, RoundingPolicy::TowardZero, OverflowPolicy::Error).unwrap();
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "scaled").unwrap(),
        )
        .derive(
            EntityDerivation::new(
                "level_health",
                HEALTH,
                StatCurve::linear(StatCurve::state(LEVEL), 2.0, 18.0),
            )
            .fixed_point(fixed),
        )
        .health(
            HealthBinding::new(HEALTH)
                .resize(HealthResizePolicy::PreserveRatio)
                .refresh(RefreshPolicy::WhenSourceChanges),
        )
        .on(
            EntityTransition::threshold(LEVEL, 10, ThresholdDirection::Rising),
            EntityAction::Run("rpg:on_level_ten".parse::<FunctionId>().unwrap()),
        );
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        let reconcile = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        let derive_line = reconcile
            .content
            .lines()
            .position(|line| line.contains("/derive_refresh"))
            .unwrap();
        let property_line = reconcile
            .content
            .lines()
            .position(|line| line.contains("/refresh"))
            .unwrap();
        let transition_line = reconcile
            .content
            .lines()
            .position(|line| line.contains("/transitions"))
            .unwrap();
        assert!(derive_line < property_line);
        assert!(property_line < transition_line);
        assert!(compiled.report.objectives.contains(&HEALTH.objective()));
        assert_eq!(compiled.report.outer_scans_per_cycle, 1);
    }

    #[test]
    fn duplicate_native_ownership_is_rejected() {
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "conflict").unwrap(),
        )
        .health(HealthBinding::new(HEALTH))
        .health(HealthBinding::new(HEALTH));
        let error = compile_definition(&archetype.definition(), &profile()).unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-OWNERSHIP");
    }

    #[test]
    fn many_properties_share_one_reconciliation_scan() {
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "dedup").unwrap(),
        )
        .health(HealthBinding::new(HEALTH))
        .attribute(AttributeBinding::new(
            sand_components::AttributeType::AttackDamage,
            NumericPropertySource::state(LEVEL),
        ))
        .name(NameBinding::new(
            crate::entity::property::EntityText::new().score(LEVEL),
        ));
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        assert_eq!(
            compiled
                .records
                .iter()
                .filter(|record| record.path.ends_with("/reconcile_scan"))
                .count(),
            1
        );
        assert_eq!(compiled.report.outer_scans_per_cycle, 1);
    }

    #[test]
    fn invalid_health_percentage_is_a_structured_range_error() {
        let archetype = EntityArchetype::<ZombieKind, MobState>::new(
            ResourceLocation::new("rpg", "percentage").unwrap(),
        )
        .on(
            EntityTransition::health_percentage(
                HEALTH,
                HEALTH,
                10_001,
                ThresholdDirection::Falling,
            ),
            EntityAction::Run("rpg:low_health".parse::<FunctionId>().unwrap()),
        );
        let error = compile_definition(&archetype.definition(), &profile()).unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-RANGE");
    }
}
