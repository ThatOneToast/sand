//! Reusable entity archetypes and lifecycle compilation.
//!
//! An archetype combines one statically known entity kind, a stable archetype
//! identity, a flattened composition of independent State components, and
//! native Minecraft behavior. Runtime identity is a Sand-owned tag plus
//! version score, not a durable Rust entity reference. Every generated
//! function rescans currently loaded entities and binds the match to `@s`.
//!
//! Initialization is idempotent and ordered: missing components are attached
//! through their canonical lifecycle first, native bindings are applied by the
//! property compiler, the optional typed callback runs, and the archetype
//! version/initialized marker is written last. Unloaded entities are not
//! scanned; their scoreboard state remains attached and reconciliation resumes
//! when they are observed again.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use sand_components::ResourceLocation;

use crate::entity::curve::{
    FixedPoint, LoweredCurve, LoweredCurveOperation, OverflowPolicy, RoundingPolicy, StatCurve,
};
use crate::entity::diagnostic::EntityDiagnostic;
use crate::entity::kind::{KnownEntityKind, MutableLivingEntityKind, SafeEntityDataWriteKind};
use crate::entity::property::{
    AttributeBinding, AttributeModifierBinding, CurrentHealthSync, EffectBinding, EntityName,
    EntityNbtBinding, EntityNbtValue, EntityTextSegment, EquipmentBinding, HealthBinding,
    HealthResizePolicy, NativePropertyKey, NumericPropertySource, OwnershipPolicy, RefreshPolicy,
    TagBinding, TeamBinding, validate_native_ownership,
};
use crate::entity::state::{
    EntityFlag, EntityStateField, NumericStateField, StateComposition, StateFieldReference,
    StateSchema, component_dirty_name, dirty_name, objective_name,
};
use crate::resource_ref::FunctionId;
use crate::state::Ticks;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::AdoptionSource",
    aliases = ["sand::prelude::AdoptionSource"],
    module = "sand::entity",
    summary = "Which externally existing entities an adoption scan may initialize.",
    context = "Which externally existing entities an adoption scan may initialize. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::AdoptionSource;",
    variants(External = "Entities carrying the archetype's Sand-owned external provenance tag.", Natural = "Unmarked entities not carrying this archetype's external provenance tag.", NaturalAndExternal = "Both natural and externally created entities."),
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::SpecialEntityPolicy",
    aliases = ["sand::prelude::SpecialEntityPolicy"],
    module = "sand::entity",
    summary = "Treatment of named, tamed, owned, or otherwise special entities.",
    context = "Treatment of named, tamed, owned, or otherwise special entities. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::SpecialEntityPolicy;",
    variants(Exclude = "Exclude special entities from automatic adoption.", Include = "Include them and allow explicitly owned properties to reconcile.", Preserve = "Adopt them while preserving unrelated name/owner/taming state."),
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::Adoption",
    aliases = ["sand::prelude::Adoption"],
    module = "sand::entity",
    summary = "A typed adoption scan. The entity type comes from the archetype's `K`; callers cannot create an unconstrained `@e` scan. Scans see loaded chunks only. `every` bounds work by running one type-constrained scan every N server ticks.",
    context = "A typed adoption scan. The entity type comes from the archetype's `K`; callers cannot create an unconstrained `@e` scan. Scans see loaded chunks only. `every` bounds work by running one type-constrained scan every N server ticks. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::Adoption;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::natural_and_external",
        aliases = ["sand::prelude::Adoption::natural_and_external"],
        module = "sand::entity",
        kind = "method",
        summary = "Adopt natural and external entities once per tick.",
        context = "Adopt natural and external entities once per tick. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "An `Adoption` that adopts natural and external entities once per tick.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let adoption = sand::entity::Adoption::natural_and_external();\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::natural",
        aliases = ["sand::prelude::Adoption::natural"],
        module = "sand::entity",
        kind = "method",
        summary = "Restrict adoption to the natural-spawn policy.",
        context = "Restrict adoption to the natural-spawn policy. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "An `Adoption` that restricts adoption to the natural-spawn policy.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let adoption = sand::entity::Adoption::natural();\n}",
    )]
    #[must_use]
    pub fn natural() -> Self {
        Self {
            source: AdoptionSource::Natural,
            ..Self::natural_and_external()
        }
    }

    /// Restrict adoption to externally summoned entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::external",
        aliases = ["sand::prelude::Adoption::external"],
        module = "sand::entity",
        kind = "method",
        summary = "Restrict adoption to externally summoned entities.",
        context = "Restrict adoption to externally summoned entities. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "An `Adoption` that restricts adoption to externally summoned entities.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let adoption = sand::entity::Adoption::external();\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::every",
        aliases = ["sand::prelude::Adoption::every"],
        module = "sand::entity",
        kind = "method",
        summary = "Set the scan cadence. A zero interval is rejected during archetype compilation.",
        context = "Set the scan cadence. A zero interval is rejected during archetype compilation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ticks = "`ticks` provides the Minecraft tick duration used to set the scan cadence. A zero interval is rejected during archetype compilation."),
        returns = "The `Adoption` value with the documented change applied to set the scan cadence. A zero interval is rejected during archetype compilation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(adoption_value: sand::entity::Adoption, ticks: sand::state::Ticks)  {\n    let updated_adoption = adoption_value.every(ticks);\n}",
    )]
    #[must_use]
    pub fn every(mut self, ticks: Ticks) -> Self {
        self.every = ticks;
        self
    }

    /// Limit adoption to a radius around each scan executor.
    ///
    /// Global type-constrained scans remain supported; this option is useful
    /// for packs that deliberately run the coordinator at player positions.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::within_blocks",
        aliases = ["sand::prelude::Adoption::within_blocks"],
        module = "sand::entity",
        kind = "method",
        summary = "Limit adoption to a radius around each scan executor.",
        context = "Limit adoption to a radius around each scan executor. Global type-constrained scans remain supported; this option is useful for packs that deliberately run the coordinator at player positions.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(blocks = "`blocks` is used to limit adoption to a radius around each scan executor."),
        returns = "The `Adoption` value with the documented change applied to limit adoption to a radius around each scan executor.",
        example = "use sand::prelude::*;\n\nfn demonstrate(adoption_value: sand::entity::Adoption, blocks: f64)  {\n    let updated_adoption = adoption_value.within_blocks(blocks);\n}",
    )]
    #[must_use]
    pub fn within_blocks(mut self, blocks: f64) -> Self {
        self.max_distance = Some(blocks);
        self
    }

    /// Choose how named/tamed/owned entities are treated.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::special_entities",
        aliases = ["sand::prelude::Adoption::special_entities"],
        module = "sand::entity",
        kind = "method",
        summary = "Choose how named/tamed/owned entities are treated.",
        context = "Choose how named/tamed/owned entities are treated. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Choose how named/tamed/owned entities are treated."],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` is used to choose how named/tamed/owned entities are treated."),
        returns = "The `Adoption` value with the documented change applied to choose how named/tamed/owned entities are treated.",
        example = "use sand::prelude::*;\n\nfn demonstrate(adoption_value: sand::entity::Adoption, policy: sand::entity::SpecialEntityPolicy)  {\n    let updated_adoption = adoption_value.special_entities(policy);\n}",
    )]
    #[must_use]
    pub fn special_entities(mut self, policy: SpecialEntityPolicy) -> Self {
        self.special = policy;
        self
    }

    /// Require a validated entity tag in addition to the typed entity kind.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::requiring_tag",
        aliases = ["sand::prelude::Adoption::requiring_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Require a validated entity tag in addition to the typed entity kind.",
        context = "Require a validated entity tag in addition to the typed entity kind. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` is used to require a validated entity tag in addition to the typed entity kind."),
        returns = "The `Adoption` value with the documented change applied to require a validated entity tag in addition to the typed entity kind.",
        example = "use sand::prelude::*;\n\nfn demonstrate(adoption_value: sand::entity::Adoption, tag: sand::entity::EntityTag)  {\n    let updated_adoption = adoption_value.requiring_tag(tag);\n}",
    )]
    #[must_use]
    pub fn requiring_tag(mut self, tag: crate::entity::property::EntityTag) -> Self {
        self.required_tags.push(tag);
        self
    }

    /// Exclude a validated entity tag from adoption.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::excluding_tag",
        aliases = ["sand::prelude::Adoption::excluding_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Exclude a validated entity tag from adoption.",
        context = "Exclude a validated entity tag from adoption. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` is used to exclude a validated entity tag from adoption."),
        returns = "The `Adoption` value with the documented change applied to exclude a validated entity tag from adoption.",
        example = "use sand::prelude::*;\n\nfn demonstrate(adoption_value: sand::entity::Adoption, tag: sand::entity::EntityTag)  {\n    let updated_adoption = adoption_value.excluding_tag(tag);\n}",
    )]
    #[must_use]
    pub fn excluding_tag(mut self, tag: crate::entity::property::EntityTag) -> Self {
        self.excluded_tags.push(tag);
        self
    }

    /// Restrict adoption with a typed state predicate.
    ///
    /// Multiple fields are merged into one selector `scores` map, avoiding
    /// handwritten score syntax and additional outer scans.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Adoption::where_state",
        aliases = ["sand::prelude::Adoption::where_state"],
        module = "sand::entity",
        kind = "method",
        summary = "Restrict adoption with a typed state predicate. Multiple fields are merged into one selector `scores` map, avoiding handwritten score syntax and additional outer scans.",
        context = "Restrict adoption with a typed state predicate. Multiple fields are merged into one selector `scores` map, avoiding handwritten score syntax and additional outer scans. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(predicate = "`predicate` provides the predicate that must match used to restrict adoption with a typed state predicate. Multiple fields are merged into one selector `scores` map, avoiding handwritten score syntax and additional outer scans."),
        returns = "The `Adoption` value with the documented change applied to restrict adoption with a typed state predicate. Multiple fields are merged into one selector `scores` map, avoiding handwritten score syntax and additional outer scans.",
        example = "use sand::prelude::*;\n\nfn demonstrate(adoption_value: sand::entity::Adoption, predicate: sand::entity::StatePredicate)  {\n    let updated_adoption = adoption_value.where_state(predicate);\n}",
    )]
    #[must_use]
    pub fn where_state(mut self, predicate: crate::entity::state::StatePredicate) -> Self {
        self.state_predicates.push(predicate);
        self
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::ReconcilePolicy",
    aliases = ["sand::prelude::ReconcilePolicy"],
    module = "sand::entity",
    summary = "When an initialized entity is checked against its archetype.",
    context = "When an initialized entity is checked against its archetype. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::ReconcilePolicy;",
    variants(Every = "Reconcile at an explicit interval.", InitializeOnly = "Initialize once and never automatically reapply owned properties.", Manual = "Reconciliation occurs only through a generated manual function.", WhenDirty = "Reconcile only when a state dependency is dirty.", WhenSchemaChanges = "Reconcile after a schema/archetype version changes."),
    variant_fields(Every = ["Reconcile at an explicit interval."]),
)]
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
    Every(#[doc = "Reconcile at an explicit interval."] Ticks),
    /// Reconciliation occurs only through a generated manual function.
    Manual,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::Migration",
    aliases = ["sand::prelude::Migration"],
    module = "sand::entity",
    summary = "One contiguous ordered archetype migration.",
    context = "One contiguous ordered archetype migration. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::Migration;",
    fields(action = "Canonical typed migration function.", from = "Version this step accepts.", to = "Version written only after the callback completes."),
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Migration::new",
        aliases = ["sand::prelude::Migration::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a migration step.",
        context = "Construct a migration step. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(from = "`from` is used when constructing a migration step.", to = "`to` is used when constructing a migration step.", action = "`action` provides the typed Minecraft resource identifier used to construct a migration step."),
        returns = "A `Migration` representing a migration step.",
        example = "use sand::prelude::*;\n\nfn demonstrate(from: u32, to: u32, action: sand::resource_ref::FunctionId)  {\n    let migration = sand::entity::Migration::new(from, to, action);\n}",
    )]
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

/// Component-first lifecycle definition for entity kind `K`.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityArchetype",
    aliases = ["sand::prelude::EntityArchetype"],
    module = "sand::entity",
    summary = "Component composition and native Minecraft behavior for entity kind `K`.",
    context = "An archetype gives one entity kind a stable identity, composes reusable State components, and binds their fields to native Minecraft behavior. No component is privileged as a root schema.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityArchetype;",
)]
#[derive(Debug, Clone)]
pub struct EntityArchetype<K> {
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
    components: Vec<ArchetypeComponent>,
    _kind: PhantomData<fn() -> K>,
}

impl<K> EntityArchetype<K>
where
    K: KnownEntityKind,
{
    /// Create an archetype.
    ///
    /// The identifier namespaces every generated objective, marker, storage
    /// path, and helper function. The default archetype version is `1` and
    /// reconciliation is version-driven.
    ///
    /// `id` is the stable resource location used to derive those generated
    /// names. Renaming it after release creates a distinct archetype identity.
    ///
    /// Returns a new archetype builder using version `1` and schema-change
    /// reconciliation policy.
    ///
    /// Use this constructor when declaring the canonical lifecycle policy for
    /// one entity kind and a composition of reusable State components.
    ///
    /// Avoid creating multiple archetypes with the same identifier; their
    /// generated Minecraft objectives and helper functions would collide.
    ///
    /// Minecraft receives the resulting objectives, marker tags, storage
    /// paths, migration functions, and reconciliation commands at export.
    ///
    /// ```rust,ignore
    /// let archetype = EntityArchetype::<ZombieKind>::new(
    ///     "demo:managed_zombie".parse()?,
    /// );
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::new",
        aliases = ["sand::prelude::EntityArchetype::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Create an empty component-first archetype for entity kind `K`.",
        context = "The identifier namespaces archetype markers and helper functions. Add independent State components or bundles with components; the default archetype version is 1 and reconciliation is version-driven.",
        minecraft = "Minecraft receives the resulting objectives, marker tags, storage paths, migration functions, and reconciliation commands at export.",
        use_when = ["Declaring the component composition and native behavior for one entity kind"],
        avoid_when = ["Avoid creating multiple archetypes with the same identifier; their generated Minecraft objectives and helper functions would collide."],
        params(id = "`id` is the stable resource location used to derive those generated names. Renaming it after release creates a distinct archetype identity."),
        returns = "A component-first archetype builder at version 1.",
        example = "use sand::prelude::*;\nlet archetype = EntityArchetype::<ZombieKind>::new(\"demo:managed_zombie\".parse().unwrap());",
    )]
    #[must_use]
    pub fn new(id: ResourceLocation) -> Self {
        Self {
            id,
            version: 1,
            adoption: None,
            reconcile: ReconcilePolicy::WhenSchemaChanges,
            initialize: None,
            cleanup: None,
            migrations: Vec::new(),
            derivations: Vec::new(),
            transitions: Vec::new(),
            properties: Vec::new(),
            components: Vec::new(),
            _kind: PhantomData,
        }
    }

    /// Override the archetype version independently of component versions.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::version",
        aliases = ["sand::prelude::EntityArchetype::version"],
        module = "sand::entity",
        kind = "method",
        summary = "Override the archetype version independently of component versions.",
        context = "The archetype version tracks changes to composition or native behavior. Every composed State retains and migrates its own independent component version.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(version = "The positive version for this archetype composition and native behavior."),
        returns = "This archetype with its independent version updated.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>) { let _ = archetype.version(2); }",
    )]
    #[must_use]
    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Discover and initialize existing loaded entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::adopt",
        aliases = ["sand::prelude::EntityArchetype::adopt"],
        module = "sand::entity",
        kind = "method",
        summary = "Discover and initialize existing loaded entities.",
        context = "Discover and initialize existing loaded entities. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(adoption = "`adoption` is used to discover and initialize existing loaded entities."),
        returns = "The `EntityArchetype` value with the documented change applied to discover and initialize existing loaded entities.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>) { let _ = archetype.adopt(Adoption::natural()); }",
    )]
    #[must_use]
    pub fn adopt(mut self, adoption: Adoption) -> Self {
        self.adoption = Some(adoption);
        self
    }

    /// Choose automatic reconciliation behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::reconcile",
        aliases = ["sand::prelude::EntityArchetype::reconcile"],
        module = "sand::entity",
        kind = "method",
        summary = "Choose automatic reconciliation behavior.",
        context = "Choose automatic reconciliation behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Choose automatic reconciliation behavior."],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` is used to choose automatic reconciliation behavior."),
        returns = "The `EntityArchetype` value with the documented change applied to choose automatic reconciliation behavior.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>) { let _ = archetype.reconcile(ReconcilePolicy::WhenDirty); }",
    )]
    #[must_use]
    pub fn reconcile(mut self, policy: ReconcilePolicy) -> Self {
        self.reconcile = policy;
        self
    }

    /// Run a typed function after state/native setup and before completion is marked.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::initialize_with",
        aliases = ["sand::prelude::EntityArchetype::initialize_with"],
        module = "sand::entity",
        kind = "method",
        summary = "Run a typed function after state/native setup and before completion is marked.",
        context = "Run a typed function after state/native setup and before completion is marked. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(function = "`function` provides the callback invoked by this operation used to run a typed function after state/native setup and before completion is marked."),
        returns = "The `EntityArchetype` value with the documented change applied to run a typed function after state/native setup and before completion is marked.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, function: FunctionId) { let _ = archetype.initialize_with(function); }",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::cleanup_with",
        aliases = ["sand::prelude::EntityArchetype::cleanup_with"],
        module = "sand::entity",
        kind = "method",
        summary = "Run a best-effort typed cleanup callback before Sand-owned state is cleared.",
        context = "Run a best-effort typed cleanup callback before Sand-owned state is cleared. Vanilla provides no reliable callback for every external removal or unloaded entity. This callback is guaranteed only when the generated cleanup function is explicitly invoked while the entity is loaded.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(function = "`function` provides the callback invoked by this operation used to run a best-effort typed cleanup callback before Sand-owned state is cleared."),
        returns = "The `EntityArchetype` value with the documented change applied to run a best-effort typed cleanup callback before Sand-owned state is cleared.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, function: FunctionId) { let _ = archetype.cleanup_with(function); }",
    )]
    #[must_use]
    pub fn cleanup_with(mut self, function: FunctionId) -> Self {
        self.cleanup = Some(function);
        self
    }

    /// Add one ordered migration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::migration",
        aliases = ["sand::prelude::EntityArchetype::migration"],
        module = "sand::entity",
        kind = "method",
        summary = "Add one ordered migration.",
        context = "Add one ordered migration. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(migration = "`migration` provides the migration added when building one ordered migration."),
        returns = "The `EntityArchetype` value with the documented change applied to add one ordered migration.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, migration: Migration) { let _ = archetype.migration(migration); }",
    )]
    #[must_use]
    pub fn migration(mut self, migration: Migration) -> Self {
        self.migrations.push(migration);
        self
    }

    /// Compose an independent State component or nested bundle into this archetype.
    ///
    /// The generated initialize path attaches every unique component through
    /// its canonical lifecycle, so existing values and per-component versions
    /// are preserved. Cleanup detaches the same composition in reverse order.
    /// Repeated composition declarations are deduplicated by their flattened
    /// presence identities and never allocate a second copy of State.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::components",
        aliases = ["sand::prelude::EntityArchetype::components"],
        module = "sand::entity",
        kind = "method",
        summary = "Compose an independent State component or nested bundle into this archetype.",
        context = "Compose an independent State component or nested bundle into this archetype. The generated initialize path attaches every unique component through its canonical lifecycle, so existing values and per-component versions are preserved. Cleanup detaches the same composition in reverse order. Repeated composition declarations are deduplicated by their flattened presence identities and never allocate a second copy of State.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityArchetype` value with the documented change applied to compose an independent State component or nested bundle into this archetype.",
        example = "use sand::prelude::*; fn compose<K: KnownEntityKind, B: StateComposition>(archetype: EntityArchetype<K>) { let _ = archetype.components::<B>(); }",
    )]
    #[must_use]
    pub fn components<B>(mut self) -> Self
    where
        B: StateComposition,
    {
        for lifecycle in B::composition_lifecycles() {
            let mut identities = lifecycle.identities;
            identities.sort();
            identities.dedup();
            let candidate = ArchetypeComponent {
                identities,
                schemas: lifecycle.schemas,
                auto_tick_objectives: lifecycle.auto_tick_objectives,
                attach: lifecycle.attach,
                detach: lifecycle.detach,
            };
            if self.components.iter().any(|component| {
                component.identities == candidate.identities
                    && component.has_same_metadata(&candidate)
            }) {
                continue;
            }
            self.components.push(candidate);
        }
        self
    }

    /// Add a cached derived score and its typed dependency declaration.
    ///
    /// The exporter lowers the curve to entity-scoped scoreboard arithmetic
    /// and marks the result dirty only when one of its source scores changes.
    /// Cycles among derived targets are rejected before resources are written.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::derive",
        aliases = ["sand::prelude::EntityArchetype::derive"],
        module = "sand::entity",
        kind = "method",
        summary = "Add a cached derived score and its typed dependency declaration.",
        context = "Add a cached derived score and its typed dependency declaration. The exporter lowers the curve to entity-scoped scoreboard arithmetic and marks the result dirty only when one of its source scores changes. Cycles among derived targets are rejected before resources are written.",
        minecraft = "The exporter lowers the curve to entity-scoped scoreboard arithmetic and marks the result dirty only when one of its source scores changes. Cycles among derived targets are rejected before resources are written.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(target = "The composed State score that receives the derived value.", curve = "The typed numeric expression used to compute the target."),
        returns = "The `EntityArchetype` value with the documented change applied to add a cached derived score and its typed dependency declaration.",
        example = "use sand::prelude::*; fn derive<K: KnownEntityKind>(archetype: EntityArchetype<K>, target: Score, curve: StatCurve) { let _ = archetype.derive(target, curve); }",
    )]
    #[must_use]
    pub fn derive<F: NumericStateField>(mut self, target: F, curve: StatCurve) -> Self {
        self.derivations
            .push(EntityDerivation::for_target(target, curve));
        self
    }

    /// Add a derivation with advanced fixed-point or identity overrides.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::derive_with",
        aliases = ["sand::prelude::EntityArchetype::derive_with"],
        module = "sand::entity",
        kind = "method",
        summary = "Adds a derivation with an explicit advanced lowering configuration.",
        context = "Use derive for the normal typed target-and-curve path; derive_with accepts EntityDerivation when fixed-point output or helper identity must be overridden.",
        minecraft = "The same flattened component membership, dependency, dirty propagation, and cycle validation applies as for derive.",
        use_when = ["A derivation needs a non-default fixed-point or output encoding"],
        avoid_when = ["The inferred target identity and ordinary numeric encoding are sufficient"],
        params(derivation = "The explicitly configured derivation."),
        returns = "This archetype with the advanced derivation added.",
        example = "use sand::prelude::*; fn add<K: KnownEntityKind>(archetype: EntityArchetype<K>, derivation: EntityDerivation) { let _ = archetype.derive_with(derivation); }",
    )]
    #[must_use]
    pub fn derive_with(mut self, derivation: EntityDerivation) -> Self {
        self.derivations.push(derivation);
        self
    }

    /// Run a typed action when entity-bound state crosses a declared boundary.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::on",
        aliases = ["sand::prelude::EntityArchetype::on"],
        module = "sand::entity",
        kind = "method",
        summary = "Run a typed action when entity-bound state crosses a declared boundary.",
        context = "Run a typed action when entity-bound state crosses a declared boundary. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(transition = "`transition` provides the transition used when running a typed action when entity-bound state crosses a declared boundary.", action = "`action` provides the action used when running a typed action when entity-bound state crosses a declared boundary."),
        returns = "The `EntityArchetype` value with the documented change applied to run a typed action when entity-bound state crosses a declared boundary.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, transition: EntityTransition, action: EntityAction) { let _ = archetype.on(transition, action); }",
    )]
    #[must_use]
    pub fn on(mut self, transition: EntityTransition, action: EntityAction) -> Self {
        self.transitions
            .push(EntityTransitionRule { transition, action });
        self
    }

    /// Add an archetype-owned tag while preserving unrelated tags.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::tag",
        aliases = ["sand::prelude::EntityArchetype::tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Add an archetype-owned tag while preserving unrelated tags.",
        context = "Add an archetype-owned tag while preserving unrelated tags. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding added when building an archetype-owned tag while preserving unrelated tags."),
        returns = "The `EntityArchetype` value with the documented change applied to add an archetype-owned tag while preserving unrelated tags.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, binding: TagBinding) { let _ = archetype.tag(binding); }",
    )]
    #[must_use]
    pub fn tag(mut self, binding: TagBinding) -> Self {
        self.properties.push(ArchetypeProperty::Tag(binding));
        self
    }

    /// Add/remove an owned tag when a typed flag changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::tag_when",
        aliases = ["sand::prelude::EntityArchetype::tag_when"],
        module = "sand::entity",
        kind = "method",
        summary = "Add/remove an owned tag when a typed flag changes.",
        context = "Add/remove an owned tag when a typed flag changes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(flag = "`flag` is used to add/remove an owned tag when a typed flag changes.", binding = "`binding` is used to add/remove an owned tag when a typed flag changes."),
        returns = "The `EntityArchetype` value with the documented change applied to add/remove an owned tag when a typed flag changes.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, flag: EntityFlag, binding: TagBinding) { let _ = archetype.tag_when(flag, binding); }",
    )]
    #[must_use]
    pub fn tag_when(mut self, flag: EntityFlag, binding: TagBinding) -> Self {
        let binding = binding.refresh(RefreshPolicy::WhenSourceChanges);
        self.properties
            .push(ArchetypeProperty::ConditionalTag { flag, binding });
        self
    }

    /// Add typed team membership while leaving team configuration external.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::team",
        aliases = ["sand::prelude::EntityArchetype::team"],
        module = "sand::entity",
        kind = "method",
        summary = "Add typed team membership while leaving team configuration external.",
        context = "Add typed team membership while leaving team configuration external. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding added when building typed team membership while leaving team configuration external."),
        returns = "The `EntityArchetype` value with the documented change applied to add typed team membership while leaving team configuration external.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, binding: TeamBinding) { let _ = archetype.team(binding); }",
    )]
    #[must_use]
    pub fn team(mut self, binding: TeamBinding) -> Self {
        self.properties.push(ArchetypeProperty::Team(binding));
        self
    }

    /// Join/leave an owned team when a typed flag changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::team_when",
        aliases = ["sand::prelude::EntityArchetype::team_when"],
        module = "sand::entity",
        kind = "method",
        summary = "Join/leave an owned team when a typed flag changes.",
        context = "Join/leave an owned team when a typed flag changes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(flag = "`flag` is used to join/leave an owned team when a typed flag changes.", binding = "`binding` is used to join/leave an owned team when a typed flag changes."),
        returns = "The `EntityArchetype` value with the documented change applied to join/leave an owned team when a typed flag changes.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind>(archetype: EntityArchetype<K>, flag: EntityFlag, binding: TeamBinding) { let _ = archetype.team_when(flag, binding); }",
    )]
    #[must_use]
    pub fn team_when(mut self, flag: EntityFlag, binding: TeamBinding) -> Self {
        let binding = binding.refresh(RefreshPolicy::WhenSourceChanges);
        self.properties
            .push(ArchetypeProperty::ConditionalTeam { flag, binding });
        self
    }

    /// Archetype identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::id",
        aliases = ["sand::prelude::EntityArchetype::id"],
        module = "sand::entity",
        kind = "method",
        summary = "Archetype identifier.",
        context = "Archetype identifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& ResourceLocation` value produced to archetype identifier.",
        example = "use sand::prelude::*; fn inspect<K: KnownEntityKind>(archetype: &EntityArchetype<K>) { let _ = archetype.id(); }",
    )]
    #[must_use]
    pub fn id(&self) -> &ResourceLocation {
        &self.id
    }

    /// Sand-owned initialized marker, deterministic across exports.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::initialized_tag",
        aliases = ["sand::prelude::EntityArchetype::initialized_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Sand-owned initialized marker, deterministic across exports.",
        context = "Sand-owned initialized marker, deterministic across exports. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to sand-owned initialized marker, deterministic across exports.",
        example = "use sand::prelude::*; fn inspect<K: KnownEntityKind>(archetype: &EntityArchetype<K>) { let _ = archetype.initialized_tag(); }",
    )]
    #[must_use]
    pub fn initialized_tag(&self) -> String {
        initialized_tag(&self.id.to_string())
    }

    /// Sand-owned tag used to opt an externally summoned entity into an
    /// [`Adoption::external`] scan.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::external_adoption_tag",
        aliases = ["sand::prelude::EntityArchetype::external_adoption_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Sand-owned tag used to opt an externally summoned entity into an [`Adoption::external`] scan.",
        context = "Sand-owned tag used to opt an externally summoned entity into an [`Adoption::external`] scan. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `sand :: entity :: EntityTag` value produced to sand-owned tag used to opt an externally summoned entity into an [`Adoption::external`] scan.",
        example = "use sand::prelude::*; fn inspect<K: KnownEntityKind>(archetype: &EntityArchetype<K>) { let _ = archetype.external_adoption_tag(); }",
    )]
    #[must_use]
    pub fn external_adoption_tag(&self) -> crate::entity::property::EntityTag {
        crate::entity::property::EntityTag::generated(external_tag(&self.id.to_string()))
    }

    /// Call the generated attach/initialize function for the current `@s`.
    ///
    /// This command is execution-scoped. It does not create or return a
    /// persistent entity reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::attach",
        aliases = ["sand::prelude::EntityArchetype::attach"],
        module = "sand::entity",
        kind = "method",
        summary = "Call the generated attach/initialize function for the current `@s`.",
        context = "Call the generated attach/initialize function for the current `@s`. This command is execution-scoped. It does not create or return a persistent entity reference.",
        minecraft = "This command is execution-scoped. It does not create or return a persistent entity reference.",
        use_when = ["Call the generated attach/initialize function for the current `@s`."],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to call the generated attach/initialize function for the current `@s`.",
        example = "use sand::prelude::*; fn commands<K: KnownEntityKind>(archetype: &EntityArchetype<K>) { let _ = archetype.attach(); }",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::summon",
        aliases = ["sand::prelude::EntityArchetype::summon"],
        module = "sand::entity",
        kind = "method",
        summary = "Summon this archetype and initialize the newly created entity.",
        context = "Summon this archetype and initialize the newly created entity. Vanilla's `execute summon` binds the new entity directly to `@s`, so no selector, temporary tag, or global scratch identity is required.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The ordered values produced to summon this archetype and initialize the newly created entity.",
        example = "use sand::prelude::*; fn commands<K: KnownEntityKind>(archetype: &EntityArchetype<K>) { let _ = archetype.summon(); }",
    )]
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
            adoption: self.adoption.clone(),
            reconcile: self.reconcile,
            initialize: self.initialize.clone(),
            cleanup: self.cleanup.clone(),
            migrations: self.migrations.clone(),
            derivations: self.derivations.clone(),
            transitions: self.transitions.clone(),
            properties: self.properties.clone(),
            components: self.components.clone(),
        }
    }
}

/// Erases an archetype's marker types for proc-macro registration.
///
/// This is public only so generated code can cross the crate boundary through
/// `sand::__private`; it is not part of the author-facing entity API.
#[doc(hidden)]
pub fn registered_definition<K>(archetype: &EntityArchetype<K>) -> ArchetypeDefinition
where
    K: KnownEntityKind,
{
    archetype.definition()
}

impl<K> EntityArchetype<K>
where
    K: KnownEntityKind + SafeEntityDataWriteKind,
{
    /// Bind a state-aware custom name and visibility.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::name",
        aliases = ["sand::prelude::EntityArchetype::name"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind a state-aware custom name and visibility.",
        context = "Bind a state-aware custom name and visibility. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding used when binding a state-aware custom name and visibility."),
        returns = "The `EntityArchetype` value with the documented change applied to bind a state-aware custom name and visibility.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + SafeEntityDataWriteKind>(archetype: EntityArchetype<K>, name: EntityName) { let _ = archetype.name(name); }",
    )]
    #[must_use]
    pub fn name(mut self, binding: EntityName) -> Self {
        self.properties.push(ArchetypeProperty::Name(binding));
        self
    }

    /// Bind a stable typed native-NBT field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::native_data",
        aliases = ["sand::prelude::EntityArchetype::native_data"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind a stable typed native-NBT field.",
        context = "Bind a stable typed native-NBT field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding used when binding a stable typed native-NBT field."),
        returns = "The `EntityArchetype` value with the documented change applied to bind a stable typed native-NBT field.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + SafeEntityDataWriteKind>(archetype: EntityArchetype<K>, binding: EntityNbtBinding) { let _ = archetype.native_data(binding); }",
    )]
    #[must_use]
    pub fn native_data(mut self, binding: EntityNbtBinding) -> Self {
        self.properties.push(ArchetypeProperty::Nbt(binding));
        self
    }
}

impl<K> EntityArchetype<K>
where
    K: KnownEntityKind + MutableLivingEntityKind,
{
    /// Synchronize current/max health according to an explicit resize policy.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::health",
        aliases = ["sand::prelude::EntityArchetype::health"],
        module = "sand::entity",
        kind = "method",
        summary = "Synchronize current/max health according to an explicit resize policy.",
        context = "Synchronize current/max health according to an explicit resize policy. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` is used to synchronize current/max health according to an explicit resize policy."),
        returns = "The `EntityArchetype` value with the documented change applied to synchronize current/max health according to an explicit resize policy.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, binding: HealthBinding) { let _ = archetype.health(binding); }",
    )]
    #[must_use]
    pub fn health(mut self, binding: HealthBinding) -> Self {
        self.properties.push(ArchetypeProperty::Health(binding));
        self
    }

    /// Bind an attribute base value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::attribute",
        aliases = ["sand::prelude::EntityArchetype::attribute"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind an attribute base value.",
        context = "Bind an attribute base value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding used when binding an attribute base value."),
        returns = "The `EntityArchetype` value with the documented change applied to bind an attribute base value.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, binding: AttributeBinding) { let _ = archetype.attribute(binding); }",
    )]
    #[must_use]
    pub fn attribute(mut self, binding: AttributeBinding) -> Self {
        self.properties.push(ArchetypeProperty::Attribute(binding));
        self
    }

    /// Bind one idempotent namespaced attribute modifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::attribute_modifier",
        aliases = ["sand::prelude::EntityArchetype::attribute_modifier"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind one idempotent namespaced attribute modifier.",
        context = "Bind one idempotent namespaced attribute modifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding used when binding one idempotent namespaced attribute modifier."),
        returns = "The `EntityArchetype` value with the documented change applied to bind one idempotent namespaced attribute modifier.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, binding: AttributeModifierBinding) { let _ = archetype.attribute_modifier(binding); }",
    )]
    #[must_use]
    pub fn attribute_modifier(mut self, binding: AttributeModifierBinding) -> Self {
        self.properties
            .push(ArchetypeProperty::AttributeModifier(binding));
        self
    }

    /// Apply an archetype-owned status effect on refresh.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::effect",
        aliases = ["sand::prelude::EntityArchetype::effect"],
        module = "sand::entity",
        kind = "method",
        summary = "Apply an archetype-owned status effect on refresh.",
        context = "Apply an archetype-owned status effect on refresh. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` provides the binding applied when an archetype-owned status effect on refresh."),
        returns = "The `EntityArchetype` value with the documented change applied to apply an archetype-owned status effect on refresh.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, binding: EffectBinding) { let _ = archetype.effect(binding); }",
    )]
    #[must_use]
    pub fn effect(mut self, binding: EffectBinding) -> Self {
        self.properties.push(ArchetypeProperty::Effect(binding));
        self
    }

    /// Apply/remove an effect only when `flag` is enabled/disabled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::effect_when",
        aliases = ["sand::prelude::EntityArchetype::effect_when"],
        module = "sand::entity",
        kind = "method",
        summary = "Apply/remove an effect only when `flag` is enabled/disabled.",
        context = "Apply/remove an effect only when `flag` is enabled/disabled. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(flag = "Apply/remove an effect only when `flag` is enabled/disabled.", binding = "`binding` is used to apply/remove an effect only when `flag` is enabled/disabled."),
        returns = "The `EntityArchetype` value with the documented change applied to apply/remove an effect only when `flag` is enabled/disabled.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, flag: EntityFlag, binding: EffectBinding) { let _ = archetype.effect_when(flag, binding); }",
    )]
    #[must_use]
    pub fn effect_when(mut self, flag: EntityFlag, binding: EffectBinding) -> Self {
        self.properties
            .push(ArchetypeProperty::ConditionalEffect { flag, binding });
        self
    }

    /// Own one typed equipment slot using Sand's canonical item stack model.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::equipment",
        aliases = ["sand::prelude::EntityArchetype::equipment"],
        module = "sand::entity",
        kind = "method",
        summary = "Own one typed equipment slot using Sand's canonical item stack model.",
        context = "Own one typed equipment slot using Sand's canonical item stack model. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(binding = "`binding` is used to own one typed equipment slot using Sand's canonical item stack model."),
        returns = "The `EntityArchetype` value with the documented change applied to own one typed equipment slot using Sand's canonical item stack model.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, binding: EquipmentBinding) { let _ = archetype.equipment(binding); }",
    )]
    #[must_use]
    pub fn equipment(mut self, binding: EquipmentBinding) -> Self {
        self.properties
            .push(ArchetypeProperty::Equipment(Box::new(binding)));
        self
    }

    /// Equip/clear one owned slot when a typed flag changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityArchetype::equipment_when",
        aliases = ["sand::prelude::EntityArchetype::equipment_when"],
        module = "sand::entity",
        kind = "method",
        summary = "Equip/clear one owned slot when a typed flag changes.",
        context = "Equip/clear one owned slot when a typed flag changes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(flag = "`flag` is used to equip/clear one owned slot when a typed flag changes.", binding = "`binding` is used to equip/clear one owned slot when a typed flag changes."),
        returns = "The `EntityArchetype` value with the documented change applied to equip/clear one owned slot when a typed flag changes.",
        example = "use sand::prelude::*; fn update<K: KnownEntityKind + MutableLivingEntityKind>(archetype: EntityArchetype<K>, flag: EntityFlag, binding: EquipmentBinding) { let _ = archetype.equipment_when(flag, binding); }",
    )]
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
    Name(EntityName),
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
    /// Independent components and bundles composed into this archetype.
    pub components: Vec<ArchetypeComponent>,
}

/// Type-erased canonical component composition retained by an archetype.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct ArchetypeComponent {
    identities: Vec<(String, u32)>,
    schemas: Vec<StateSchema>,
    auto_tick_objectives: Vec<String>,
    attach: fn(&'static str) -> Vec<String>,
    detach: fn(&'static str) -> Vec<String>,
}

impl ArchetypeComponent {
    fn has_same_metadata(&self, other: &Self) -> bool {
        self.schemas == other.schemas
            && self.auto_tick_objectives == other.auto_tick_objectives
            && std::ptr::fn_addr_eq(self.attach, other.attach)
            && std::ptr::fn_addr_eq(self.detach, other.detach)
    }
}

fn dedup_commands(commands: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    commands.retain(|command| seen.insert(command.clone()));
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityDerivation",
    aliases = ["sand::prelude::EntityDerivation"],
    module = "sand::entity",
    summary = "Advanced lowering configuration for a typed State-field derivation.",
    context = "The target field supplies the stable derivation identity and stored representation. Curves use fixed-point working arithmetic internally; this type is needed only when overriding its working precision, policies, or helper identity.",
    minecraft = "Sand evaluates the curve with scoreboard integer arithmetic, converts once to the destination field's declared scale, applies its bounds, and caches the result.",
    use_when = ["A derivation needs advanced fixed-point or output configuration"],
    avoid_when = ["The normal EntityArchetype::derive target-and-curve API is sufficient"],
    example = "use sand::entity::EntityDerivation;",
)]
/// Advanced lowering configuration for a typed State-field derivation.
///
/// [`Score`](crate::entity::state::Score) destinations store whole logical
/// values. [`FixedScore`](crate::entity::state::FixedScore) destinations store
/// logical decimals using their declared scale. Curve working precision is an
/// implementation detail unless explicitly customized with [`Self::fixed_point`].
#[derive(Debug, Clone)]
pub struct EntityDerivation {
    name: String,
    target: crate::entity::state::EntityScore<i32>,
    target_scale: i64,
    curve: StatCurve,
    fixed: FixedPoint,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::DerivedScoreEncoding",
    aliases = ["sand::prelude::DerivedScoreEncoding"],
    module = "sand::entity",
    summary = "Compatibility view of the representation inferred from a derivation's destination field.",
    context = "This value reports whether a derivation targets whole or scaled storage. Callers do not select it independently: Score implies Whole and FixedScore implies FixedPoint.",
    minecraft = "Sand infers the representation from the destination State field and performs the required scale conversion while lowering the derivation.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::DerivedScoreEncoding;",
    variants(FixedPoint = "The destination is a FixedScore with a schema-declared scale.", Whole = "The destination is a whole-number Score."),
)]
/// Compatibility view of the representation inferred from a derivation's
/// destination State field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DerivedScoreEncoding {
    /// The destination is a whole-number [`Score`](crate::entity::state::Score).
    Whole,
    /// The destination is a [`FixedScore`](crate::entity::state::FixedScore)
    /// with a schema-declared scale.
    FixedPoint,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::ThresholdDirection",
    aliases = ["sand::prelude::ThresholdDirection"],
    module = "sand::entity",
    summary = "Direction used by threshold-crossing transitions.",
    context = "Direction used by threshold-crossing transitions. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::ThresholdDirection;",
    variants(Falling = "Fire when the score moves from above to at most the threshold.", Rising = "Fire when the score moves from below to at least the threshold."),
)]
/// Direction used by threshold-crossing transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThresholdDirection {
    /// Fire when the score moves from below to at least the threshold.
    Rising,
    /// Fire when the score moves from above to at most the threshold.
    Falling,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTransition",
    aliases = ["sand::prelude::EntityTransition"],
    module = "sand::entity",
    summary = "A state change observed for one loaded archetyped entity.",
    context = "A state change observed for one loaded archetyped entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTransition;",
    variants(Changed = "Any numeric, enum, or flag value change.", CooldownReady = "A cooldown reached its ready state.", EnumChangedTo = "An enum changed to one stable encoding.", FlagDisabled = "A flag changed from enabled to disabled.", FlagEnabled = "A flag changed from disabled to enabled.", HealthPercentage = "A current/max-health percentage boundary was crossed.", Threshold = "A whole-score threshold was crossed.", TimerElapsed = "A timer reached zero from a positive value."),
    variant_fields(Changed = ["Any numeric, enum, or flag value change."], CooldownReady = ["A cooldown reached its ready state."], EnumChangedTo(encoding = "Stable enum encoding.", field = "Enum score field."), FlagDisabled = ["A flag changed from enabled to disabled."], FlagEnabled = ["A flag changed from disabled to enabled."], HealthPercentage(basis_points = "Inclusive percentage in basis points (`10_000 == 100%`).", current = "Current-health score.", direction = "Crossing direction.", maximum = "Maximum-health score."), Threshold(direction = "Crossing direction.", field = "Source score.", value = "Inclusive boundary."), TimerElapsed = ["A timer reached zero from a positive value."]),
)]
/// A state change observed for one loaded archetyped entity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityTransition {
    /// Any numeric, enum, or flag value change.
    Changed(#[doc = "Any numeric, enum, or flag value change."] EntityTransitionField),
    /// A flag changed from disabled to enabled.
    FlagEnabled(#[doc = "A flag changed from disabled to enabled."] EntityTransitionField),
    /// A flag changed from enabled to disabled.
    FlagDisabled(#[doc = "A flag changed from enabled to disabled."] EntityTransitionField),
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
    TimerElapsed(#[doc = "A timer reached zero from a positive value."] EntityTransitionField),
    /// A cooldown reached its ready state.
    CooldownReady(#[doc = "A cooldown reached its ready state."] EntityTransitionField),
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTransitionField",
    module = "sand::entity",
    summary = "Type-erased identity of a typed state field used by a transition plan.",
    context = "Type-erased identity of a typed state field used by a transition plan. Construct this through [`EntityTransition`] helpers; the stored objective is generated from schema metadata, never accepted as a raw string.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTransitionField;",
)]
/// Type-erased identity of a typed state field used by a transition plan.
///
/// Construct this through [`EntityTransition`] helpers; the stored objective
/// is generated from schema metadata, never accepted as a raw string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTransitionField {
    reference: StateFieldReference,
}

impl EntityTransitionField {
    fn typed<F: EntityStateField>(field: F) -> Self {
        Self {
            reference: field.field_reference(),
        }
    }

    fn objective(&self) -> &str {
        &self.reference.objective
    }
}

impl EntityTransition {
    /// Observe any change to a typed field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::changed",
        aliases = ["sand::prelude::EntityTransition::changed"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe any change to a typed field.",
        context = "Observe any change to a typed field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking any change to a typed field."),
        returns = "An `EntityTransition` observing any change to a typed field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<F : sand::entity::EntityStateField + 'static>(field: F)  {\n    let entity_transition = sand::entity::EntityTransition::changed::<F>(field);\n}",
    )]
    #[must_use]
    pub fn changed<F: EntityStateField>(field: F) -> Self {
        Self::Changed(EntityTransitionField::typed(field))
    }

    /// Observe a flag becoming enabled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::flag_enabled",
        aliases = ["sand::prelude::EntityTransition::flag_enabled"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe a flag becoming enabled.",
        context = "Observe a flag becoming enabled. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking a flag becoming enabled."),
        returns = "An `EntityTransition` observing a flag becoming enabled.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: sand::entity::EntityFlag)  {\n    let entity_transition = sand::entity::EntityTransition::flag_enabled(field);\n}",
    )]
    #[must_use]
    pub fn flag_enabled(field: EntityFlag) -> Self {
        Self::FlagEnabled(EntityTransitionField::typed(field))
    }

    /// Observe a flag becoming disabled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::flag_disabled",
        aliases = ["sand::prelude::EntityTransition::flag_disabled"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe a flag becoming disabled.",
        context = "Observe a flag becoming disabled. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking a flag becoming disabled."),
        returns = "An `EntityTransition` observing a flag becoming disabled.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: sand::entity::EntityFlag)  {\n    let entity_transition = sand::entity::EntityTransition::flag_disabled(field);\n}",
    )]
    #[must_use]
    pub fn flag_disabled(field: EntityFlag) -> Self {
        Self::FlagDisabled(EntityTransitionField::typed(field))
    }

    /// Observe a typed enum becoming one variant.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::enum_changed_to",
        aliases = ["sand::prelude::EntityTransition::enum_changed_to"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe a typed enum becoming one variant.",
        context = "Observe a typed enum becoming one variant. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking a typed enum becoming one variant.", value = "`value` provides the value being applied or compared used to observe a typed enum becoming one variant."),
        returns = "An `EntityTransition` observing a typed enum becoming one variant.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(field: sand::entity::EntityEnum < T >, value: T)  {\n    let entity_transition = sand::entity::EntityTransition::enum_changed_to::<T>(field, value);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::threshold",
        aliases = ["sand::prelude::EntityTransition::threshold"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe an inclusive threshold crossing.",
        context = "Observe an inclusive threshold crossing. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking an inclusive threshold crossing.", value = "`value` provides the value being applied or compared used to observe an inclusive threshold crossing.", direction = "`direction` provides the direction observed when tracking an inclusive threshold crossing."),
        returns = "An `EntityTransition` observing an inclusive threshold crossing.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: impl sand::entity::EntityStateField, value: i32, direction: sand::entity::ThresholdDirection)  {\n    let entity_transition = sand::entity::EntityTransition::threshold(field, value, direction);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::health_percentage",
        aliases = ["sand::prelude::EntityTransition::health_percentage"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe a health-ratio crossing in basis points.",
        context = "Observe a health-ratio crossing in basis points. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(current = "`current` provides the current observed when tracking a health-ratio crossing in basis points.", maximum = "`maximum` provides the maximum observed when tracking a health-ratio crossing in basis points.", basis_points = "`basis_points` provides the basis points observed when tracking a health-ratio crossing in basis points.", direction = "`direction` provides the direction observed when tracking a health-ratio crossing in basis points."),
        returns = "An `EntityTransition` observing a health-ratio crossing in basis points.",
        example = "use sand::prelude::*;\n\nfn demonstrate(current: impl sand::entity::EntityStateField, maximum: impl sand::entity::EntityStateField, basis_points: u16, direction: sand::entity::ThresholdDirection)  {\n    let entity_transition = sand::entity::EntityTransition::health_percentage(current, maximum, basis_points, direction);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::timer_elapsed",
        aliases = ["sand::prelude::EntityTransition::timer_elapsed"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe a timer reaching zero.",
        context = "Observe a timer reaching zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking a timer reaching zero."),
        returns = "An `EntityTransition` observing a timer reaching zero.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: sand::entity::EntityTimer)  {\n    let entity_transition = sand::entity::EntityTransition::timer_elapsed(field);\n}",
    )]
    #[must_use]
    pub fn timer_elapsed(field: crate::entity::state::EntityTimer) -> Self {
        Self::TimerElapsed(EntityTransitionField::typed(field))
    }

    /// Observe a cooldown becoming ready.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTransition::cooldown_ready",
        aliases = ["sand::prelude::EntityTransition::cooldown_ready"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe a cooldown becoming ready.",
        context = "Observe a cooldown becoming ready. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field observed when tracking a cooldown becoming ready."),
        returns = "An `EntityTransition` observing a cooldown becoming ready.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: sand::entity::EntityCooldown)  {\n    let entity_transition = sand::entity::EntityTransition::cooldown_ready(field);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityAction",
    aliases = ["sand::prelude::EntityAction"],
    module = "sand::entity",
    summary = "Typed work dispatched by an entity transition. [`Self::Run`] composes with the existing event/function infrastructure: the registered function runs with the transitioning entity bound to `@s` and can mutate state, update properties, summon, dispatch VFX, or transform the entity using normal Sand authoring APIs.",
    context = "Typed work dispatched by an entity transition. [`Self::Run`] composes with the existing event/function infrastructure: the registered function runs with the transitioning entity bound to `@s` and can mutate state, update properties, summon, dispatch VFX, or transform the entity using normal Sand authoring APIs. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityAction;",
    variants(AddTag = "Add a validated entity tag.", ApplyEffect = "Add or refresh a typed status effect.", Despawn = "Remove the current non-player entity.", Dispatch = "Dispatch a typed event function.", RemoveEffect = "Remove a typed status effect.", RemoveTag = "Remove a validated entity tag.", Run = "Call a canonical registered datapack function."),
    variant_fields(AddTag = ["Add a validated entity tag."], ApplyEffect = ["Add or refresh a typed status effect."], Dispatch = ["Dispatch a typed event function."], RemoveEffect = ["Remove a typed status effect."], RemoveTag = ["Remove a validated entity tag."], Run = ["Call a canonical registered datapack function."]),
)]
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
    Run(#[doc = "Call a canonical registered datapack function."] FunctionId),
    /// Dispatch a typed event function.
    Dispatch(#[doc = "Dispatch a typed event function."] crate::entity::property::EntityEventId),
    /// Add or refresh a typed status effect.
    ApplyEffect(#[doc = "Add or refresh a typed status effect."] EffectBinding),
    /// Remove a typed status effect.
    RemoveEffect(#[doc = "Remove a typed status effect."] sand_components::StatusEffectId),
    /// Add a validated entity tag.
    AddTag(#[doc = "Add a validated entity tag."] crate::entity::property::EntityTag),
    /// Remove a validated entity tag.
    RemoveTag(#[doc = "Remove a validated entity tag."] crate::entity::property::EntityTag),
    /// Remove the current non-player entity.
    Despawn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityTransitionRule {
    transition: EntityTransition,
    action: EntityAction,
}

impl EntityDerivation {
    /// Create a derivation whose stored representation is inferred from its
    /// destination State field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::for_target",
        aliases = ["sand::prelude::EntityDerivation::for_target"],
        module = "sand::entity",
        kind = "method",
        summary = "Create an advanced derivation while inferring identity and stored representation from its target State field.",
        context = "The ordinary path is EntityArchetype::derive; construct this value directly only to override working precision, policies, or the inferred identity. Score stores whole logical values and FixedScore stores values using its declared scale.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(target = "The composed State score that receives the derived value.", curve = "The typed numeric expression used to compute the target."),
        returns = "An `EntityDerivation` using the destination field's canonical storage representation.",
        example = "use sand::prelude::*; fn advanced(target: impl NumericStateField, curve: StatCurve) { let _ = EntityDerivation::for_target(target, curve); }",
    )]
    #[must_use]
    pub fn for_target<F: NumericStateField>(target: F, curve: StatCurve) -> Self {
        let name = format!("{}::{}", target.component_id(), target.descriptor().name);
        let target_scale = i64::from(target.numeric_scale());
        let fixed = if target_scale == 1 {
            FixedPoint::default()
        } else {
            FixedPoint::new(
                target_scale,
                RoundingPolicy::NearestTiesAwayFromZero,
                OverflowPolicy::Error,
            )
            .expect("NumericStateField scales are positive")
        };
        Self {
            name,
            target: target.erase_numeric(),
            target_scale,
            curve,
            fixed,
        }
    }

    /// Override the inferred stable identity used in diagnostics and helpers.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::named",
        aliases = ["sand::prelude::EntityDerivation::named"],
        module = "sand::entity",
        kind = "method",
        summary = "Overrides the stable identity inferred from the target State field.",
        context = "The identity appears in diagnostics and deterministic helper naming; most authors should keep the inferred component-and-field identity.",
        minecraft = "Changing this identity can rename generated derivation helpers without changing State storage.",
        use_when = ["Maintaining an intentional legacy helper identity"],
        avoid_when = ["The target field identity is the desired stable name"],
        params(name = "The explicit stable derivation identity."),
        returns = "This derivation with an explicit identity.",
        example = "use sand::prelude::*; fn rename(derivation: EntityDerivation) { let _ = derivation.named(\"legacy_health\"); }",
    )]
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Override the curve's internal working precision and policies.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::fixed_point",
        aliases = ["sand::prelude::EntityDerivation::fixed_point"],
        module = "sand::entity",
        kind = "method",
        summary = "Override a curve's internal fixed-point working precision and policies.",
        context = "This advanced control changes intermediate curve arithmetic only. It never changes the destination representation: Score remains whole and FixedScore retains its declared scale.",
        minecraft = "Sand evaluates intermediate curve operations at this scale, then converts once to the destination field's inferred scale.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(fixed = "`fixed` provides the fixed-value inputs used to select fixed-point scale, rounding, and overflow semantics."),
        returns = "The `EntityDerivation` with customized internal curve arithmetic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: sand::entity::EntityDerivation, fixed: sand::entity::FixedPoint)  {\n    let updated_entity_derivation = entity_derivation_value.fixed_point(fixed);\n}",
    )]
    #[must_use]
    pub fn fixed_point(mut self, fixed: FixedPoint) -> Self {
        self.fixed = fixed;
        self
    }

    /// Deprecated compatibility no-op.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::store_fixed_point",
        aliases = ["sand::prelude::EntityDerivation::store_fixed_point"],
        module = "sand::entity",
        kind = "method",
        summary = "Deprecated compatibility no-op; the destination field now determines storage.",
        context = "Use a FixedScore destination with the desired State scale. This method remains source-compatible but cannot override Score or FixedScore representation.",
        minecraft = "This method emits no additional behavior; lowering follows the destination field's declared representation.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The unchanged `EntityDerivation`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: sand::entity::EntityDerivation)  {\n    let updated_entity_derivation = entity_derivation_value.store_fixed_point();\n}",
    )]
    #[must_use]
    #[deprecated(
        since = "0.1.0",
        note = "the destination Score or FixedScore now determines storage; choose FixedScore with #[state(scale = ...)]"
    )]
    pub fn store_fixed_point(self) -> Self {
        self
    }

    /// Stable diagnostic/resource name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::name",
        aliases = ["sand::prelude::EntityDerivation::name"],
        module = "sand::entity",
        kind = "method",
        summary = "Stable diagnostic/resource name.",
        context = "Stable diagnostic/resource name. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to use stable diagnostic/resource name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: &sand::entity::EntityDerivation)  {\n    let name = entity_derivation_value.name();\n}",
    )]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Typed score receiving the cached value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::target",
        aliases = ["sand::prelude::EntityDerivation::target"],
        module = "sand::entity",
        kind = "method",
        summary = "Typed score receiving the cached value.",
        context = "Typed score receiving the cached value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `sand :: entity :: EntityScore < i32 >` value produced to typed score receiving the cached value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: &sand::entity::EntityDerivation)  {\n    let target = entity_derivation_value.target();\n}",
    )]
    #[must_use]
    pub const fn target(&self) -> crate::entity::state::EntityScore<i32> {
        self.target
    }

    /// Curve expression.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::curve",
        aliases = ["sand::prelude::EntityDerivation::curve"],
        module = "sand::entity",
        kind = "method",
        summary = "Curve expression.",
        context = "Curve expression. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& StatCurve` value produced to curve expression.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: &sand::entity::EntityDerivation)  {\n    let curve = entity_derivation_value.curve();\n}",
    )]
    #[must_use]
    pub fn curve(&self) -> &StatCurve {
        &self.curve
    }

    /// Fixed-point working representation used for curve evaluation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::fixed",
        aliases = ["sand::prelude::EntityDerivation::fixed"],
        module = "sand::entity",
        kind = "method",
        summary = "Fixed-point working representation used for curve evaluation.",
        context = "This describes intermediate arithmetic, not cached storage. The destination Score or FixedScore independently determines the stored representation.",
        minecraft = "Sand evaluates intermediate curve operations with this configuration before converting to the destination field's scale.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `FixedPoint` configuration used for intermediate curve arithmetic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: &sand::entity::EntityDerivation)  {\n    let fixed = entity_derivation_value.fixed();\n}",
    )]
    #[must_use]
    pub const fn fixed(&self) -> FixedPoint {
        self.fixed
    }

    /// Representation inferred from the destination State field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityDerivation::output_encoding",
        aliases = ["sand::prelude::EntityDerivation::output_encoding"],
        module = "sand::entity",
        kind = "method",
        summary = "Representation inferred from the destination State field.",
        context = "Score destinations report Whole; FixedScore destinations report FixedPoint. This is an introspection and compatibility API, not a separate storage choice.",
        minecraft = "Sand derives this value from the destination scale used when lowering the final scoreboard assignment.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `DerivedScoreEncoding` value produced to target score representation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_derivation_value: &sand::entity::EntityDerivation)  {\n    let output_encoding = entity_derivation_value.output_encoding();\n}",
    )]
    #[must_use]
    pub const fn output_encoding(&self) -> DerivedScoreEncoding {
        match self.target.retained_kind() {
            crate::entity::state::StateFieldKind::Fixed(_) => DerivedScoreEncoding::FixedPoint,
            _ => DerivedScoreEncoding::Whole,
        }
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
    let claims = component_claims(&definitions);
    definitions
        .iter()
        .map(|definition| compile_definition_with_claims(definition, profile, &claims))
        .collect()
}

type ComponentClaims = std::collections::BTreeMap<String, Vec<String>>;

fn component_claims(definitions: &[ArchetypeDefinition]) -> ComponentClaims {
    let mut claims = ComponentClaims::new();
    for definition in definitions {
        let marker = initialized_tag(&definition.id.to_string());
        for component in &definition.components {
            for (identity, _) in &component.identities {
                claims
                    .entry(identity.clone())
                    .or_default()
                    .push(marker.clone());
            }
            for schema in &component.schemas {
                claims.entry(schema.id()).or_default().push(marker.clone());
            }
        }
    }
    for markers in claims.values_mut() {
        markers.sort();
        markers.dedup();
    }
    claims
}

fn dirty_pending_name(dirty_objective: &str, marker: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{dirty_objective}.{marker}.pending"))
        .as_str()
        .to_string()
}

fn dirty_distribution_commands(
    fields: &ArchetypeFields,
    claims: &ComponentClaims,
    objectives: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut commands = Vec::new();
    for field in fields.values() {
        let Some(claim_markers) = claims.get(&field.component) else {
            continue;
        };
        for claim_marker in claim_markers {
            let pending = dirty_pending_name(&field.dirty_objective, claim_marker);
            objectives.insert(pending.clone());
            commands.push(format!(
                "execute if score @s {} matches 1 if entity @s[tag={claim_marker}] run scoreboard players set @s {pending} 1",
                field.dirty_objective
            ));
        }
        commands.push(format!(
            "execute if score @s {} matches 1 run scoreboard players set @s {} 0",
            field.dirty_objective, field.dirty_objective
        ));
    }
    commands
}

fn dirty_acknowledgement_commands(fields: &ArchetypeFields, marker: &str) -> Vec<String> {
    fields
        .values()
        .map(|field| {
            format!(
                "scoreboard players set @s {} 0",
                dirty_pending_name(&field.dirty_objective, marker)
            )
        })
        .collect()
}

#[derive(Debug, Clone)]
struct ResolvedArchetypeField {
    component: String,
    field: String,
    objective: String,
    dirty_objective: String,
    component_dirty_objective: String,
    descriptor: crate::entity::state::StateFieldDescriptor,
}

struct ArchetypeFields {
    by_objective: std::collections::BTreeMap<String, ResolvedArchetypeField>,
    components: BTreeSet<String>,
}

impl ArchetypeFields {
    fn new(definition: &ArchetypeDefinition) -> Result<Self, EntityDiagnostic> {
        let mut schemas = std::collections::BTreeMap::<String, StateSchema>::new();
        for (index, component) in definition.components.iter().enumerate() {
            if definition.components[..index].iter().any(|existing| {
                existing.identities == component.identities
                    && !existing.has_same_metadata(component)
            }) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema: definition.id.to_string(),
                    field: component
                        .identities
                        .iter()
                        .map(|(identity, version)| format!("{identity}@{version}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    detail: "two composed components claim the same lifecycle identity with conflicting schemas, auto-tick metadata, or lifecycle hooks".into(),
                });
            }
            for schema in &component.schemas {
                schema.validate()?;
                let id = schema.id();
                if let Some(existing) = schemas.get(&id) {
                    if existing != schema {
                        return Err(EntityDiagnostic::DuplicateStateField {
                            schema: id,
                            field: "<component>".into(),
                            detail: "the flattened composition contains conflicting metadata for one component identity".into(),
                        });
                    }
                } else {
                    schemas.insert(id, *schema);
                }
            }
        }
        let components = schemas.keys().cloned().collect();
        let mut by_objective = std::collections::BTreeMap::new();
        for (component, schema) in schemas {
            for descriptor in schema.fields {
                let objective = objective_name(schema.namespace, schema.name, descriptor.name);
                let resolved = ResolvedArchetypeField {
                    component: component.clone(),
                    field: descriptor.name.to_owned(),
                    dirty_objective: dirty_name(schema.namespace, schema.name, descriptor.name),
                    component_dirty_objective: component_dirty_name(schema.namespace, schema.name),
                    objective: objective.clone(),
                    descriptor: *descriptor,
                };
                if let Some(previous) = by_objective.insert(objective.clone(), resolved) {
                    return Err(EntityDiagnostic::DuplicateStateField {
                        schema: definition.id.to_string(),
                        field: objective,
                        detail: format!(
                            "flattened fields `{}::{}` and `{}::{}` resolve to the same objective",
                            previous.component, previous.field, component, descriptor.name
                        ),
                    });
                }
            }
        }
        Ok(Self {
            by_objective,
            components,
        })
    }

    fn values(&self) -> impl Iterator<Item = &ResolvedArchetypeField> {
        self.by_objective.values()
    }

    fn field_for_dirty_objective(&self, dirty_objective: &str) -> Option<&ResolvedArchetypeField> {
        self.by_objective
            .values()
            .find(|field| field.dirty_objective == dirty_objective)
    }

    fn resolve_reference<'a>(
        &'a self,
        definition: &ArchetypeDefinition,
        property: impl Into<String>,
        reference: &StateFieldReference,
    ) -> Result<&'a ResolvedArchetypeField, EntityDiagnostic> {
        let property = property.into();
        if !self.components.contains(&reference.component) {
            return Err(EntityDiagnostic::MissingArchetypeComponent {
                archetype: definition.id.to_string(),
                property,
                component: reference.component.clone(),
                field: reference.field.clone(),
            });
        }
        self.by_objective
            .get(&reference.objective)
            .filter(|field| {
                field.component == reference.component
                    && field.field == reference.field
                    && field.descriptor == reference.descriptor
                    && field.dirty_objective == reference.dirty_objective
            })
            .ok_or_else(|| EntityDiagnostic::InvalidRawExtension {
                archetype: definition.id.to_string(),
                extension: property,
                detail: format!(
                    "typed field `{}::{}` does not match the composed component metadata",
                    reference.component, reference.field
                ),
            })
    }

    fn resolve_objective<'a>(
        &'a self,
        definition: &ArchetypeDefinition,
        property: impl Into<String>,
        objective: &str,
    ) -> Result<&'a ResolvedArchetypeField, EntityDiagnostic> {
        let property = property.into();
        self.by_objective
            .get(objective)
            .ok_or_else(|| EntityDiagnostic::InvalidRawExtension {
                archetype: definition.id.to_string(),
                extension: property,
                detail: format!(
                    "score objective `{objective}` is not a field in the flattened archetype composition"
                ),
            })
    }
}

#[cfg(test)]
fn compile_definition(
    definition: &ArchetypeDefinition,
    profile: &crate::version::VersionProfile,
) -> Result<CompiledArchetype, EntityDiagnostic> {
    let claims = component_claims(std::slice::from_ref(definition));
    compile_definition_with_claims(definition, profile, &claims)
}

fn compile_definition_with_claims(
    definition: &ArchetypeDefinition,
    _profile: &crate::version::VersionProfile,
    claims: &ComponentClaims,
) -> Result<CompiledArchetype, EntityDiagnostic> {
    let fields = ArchetypeFields::new(definition)?;
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
    for field in fields.values() {
        objectives.insert(field.objective.clone());
        objectives.insert(field.dirty_objective.clone());
        objectives.insert(field.component_dirty_objective.clone());
        if let Some(markers) = claims.get(&field.component) {
            for claim_marker in markers {
                objectives.insert(dirty_pending_name(&field.dirty_objective, claim_marker));
            }
        }
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

    let mut component_attach = Vec::new();
    for component in &definition.components {
        component_attach.extend((component.attach)("@s"));
    }
    dedup_commands(&mut component_attach);
    let provision_path = format!("{root}/provision");
    functions.insert(provision_path.clone());
    records.push(function_record(
        definition.id.namespace(),
        &provision_path,
        component_attach,
    ));
    let mut initialize_commands = vec![format!(
        "function {}:{provision_path}",
        definition.id.namespace()
    )];
    let repair_objective = sand_commands::ObjectiveName::logical(format!("{id}.component_repair"))
        .as_str()
        .to_string();
    objectives.insert(repair_objective.clone());
    let mut repair_refresh_commands = Vec::new();

    let derivations = compile_derivations(definition, &fields, &root, &marker)?;
    objectives.extend(derivations.objectives);
    functions.extend(derivations.functions);
    records.extend(derivations.records);
    initialize_commands.extend(derivations.initialize_commands.iter().cloned());
    repair_refresh_commands.extend(derivations.initialize_commands.iter().cloned());

    let mut refresh_sources: Vec<(String, String)> = Vec::new();
    let mut refresh_outputs: Vec<(String, String)> = Vec::new();
    let mut periodic_refreshes: Vec<(String, String, u32)> = Vec::new();
    for (index, property) in definition.properties.iter().enumerate() {
        let compiled = compile_property(definition, &fields, property, index, &root, _profile)?;
        objectives.extend(compiled.objectives);
        functions.extend(compiled.functions);
        records.extend(compiled.records);
        if let Some(function) = compiled.initialize_function {
            initialize_commands.push(format!("function {function}"));
            repair_refresh_commands.push(format!("function {function}"));
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
            let pending = fields
                .field_for_dirty_objective(source)
                .map(|field| dirty_pending_name(&field.dirty_objective, &marker));
            if let Some(pending) = pending {
                commands.push(format!(
                    "execute if score @s {pending} matches 1 run scoreboard players set @s {output} 1"
                ));
            }
            commands.push(format!(
                "execute if score @s {source} matches 1 run scoreboard players set @s {output} 1"
            ));
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
    let transitions = compile_transitions(definition, &fields, &root)?;
    objectives.extend(transitions.objectives);
    functions.extend(transitions.functions);
    records.extend(transitions.records);
    initialize_commands.extend(transitions.initialize_commands.iter().cloned());
    repair_refresh_commands.extend(transitions.initialize_commands);
    initialize_commands.extend(dirty_distribution_commands(
        &fields,
        claims,
        &mut objectives,
    ));
    initialize_commands.extend(dirty_acknowledgement_commands(&fields, &marker));
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
    let mut reconcile_commands = vec![format!("scoreboard players set @s {repair_objective} 0")];
    for component in &definition.components {
        for (identity, version) in &component.identities {
            reconcile_commands.push(format!(
                "execute unless score @s {identity} matches {version} run scoreboard players set @s {repair_objective} 1"
            ));
        }
    }
    reconcile_commands.push(format!(
        "function {}:{provision_path}",
        definition.id.namespace()
    ));
    if !repair_refresh_commands.is_empty() {
        let repair_path = format!("{root}/repair_refresh");
        functions.insert(repair_path.clone());
        records.push(function_record(
            definition.id.namespace(),
            &repair_path,
            repair_refresh_commands,
        ));
        reconcile_commands.push(format!(
            "execute if score @s {repair_objective} matches 1 if score @s {version_objective} matches {} run function {}:{repair_path}",
            definition.version,
            definition.id.namespace()
        ));
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
    let independently_ticked = definition
        .components
        .iter()
        .flat_map(|component| component.auto_tick_objectives.iter().cloned())
        .collect::<BTreeSet<_>>();
    for field in fields.values().filter(|field| {
        matches!(
            field.descriptor.kind,
            crate::entity::state::StateFieldKind::Timer
                | crate::entity::state::StateFieldKind::Cooldown
        ) && !independently_ticked.contains(&field.objective)
    }) {
        let tick_owner_guard = claims
            .get(&field.component)
            .into_iter()
            .flatten()
            .take_while(|claim_marker| claim_marker.as_str() < marker.as_str())
            .map(|claim_marker| format!("unless entity @s[tag={claim_marker}]"))
            .collect::<Vec<_>>()
            .join(" ");
        let tick_owner_guard = if tick_owner_guard.is_empty() {
            String::new()
        } else {
            format!("{tick_owner_guard} ")
        };
        reconcile_commands.push(format!(
            "execute {tick_owner_guard}if score @s {} matches 1.. run scoreboard players set @s {} 1",
            field.objective, field.dirty_objective,
        ));
        reconcile_commands.push(format!(
            "execute {tick_owner_guard}if score @s {} matches 1.. run scoreboard players set @s {} 1",
            field.objective, field.component_dirty_objective,
        ));
        reconcile_commands.push(format!(
            "execute {tick_owner_guard}if score @s {} matches 1.. run scoreboard players remove @s {} 1",
            field.objective, field.objective,
        ));
    }
    reconcile_commands.extend(dirty_distribution_commands(
        &fields,
        claims,
        &mut objectives,
    ));
    for acknowledgement in dirty_acknowledgement_commands(&fields, &marker) {
        reconcile_commands.push(format!(
            "execute if score @s {repair_objective} matches 1 run {acknowledgement}"
        ));
    }
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
    reconcile_commands.extend(dirty_distribution_commands(
        &fields,
        claims,
        &mut objectives,
    ));
    reconcile_commands.extend(dirty_acknowledgement_commands(&fields, &marker));
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
    let mut component_cleanup = Vec::new();
    for component in definition.components.iter().rev() {
        let mut retaining_markers = component
            .identities
            .iter()
            .flat_map(|(identity, _)| claims.get(identity).into_iter().flatten())
            .filter(|claim| *claim != &marker)
            .cloned()
            .collect::<Vec<_>>();
        retaining_markers.sort();
        retaining_markers.dedup();
        component_cleanup.extend(
            (component.detach)("@s")
                .into_iter()
                .map(|command| guard_component_cleanup(&command, &retaining_markers)),
        );
    }
    dedup_commands(&mut component_cleanup);
    cleanup_commands.extend(component_cleanup);
    for property in &definition.properties {
        cleanup_commands.extend(property_cleanup_commands(property));
    }
    let component_objectives = fields
        .values()
        .flat_map(|field| {
            [
                field.objective.clone(),
                field.dirty_objective.clone(),
                field.component_dirty_objective.clone(),
            ]
        })
        .collect::<BTreeSet<_>>();
    for objective in &objectives {
        if !component_objectives.contains(objective) {
            cleanup_commands.push(format!("scoreboard players reset @s {objective}"));
        }
    }
    dedup_commands(&mut cleanup_commands);
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
            if let Some(field) = &predicate.field {
                fields.resolve_reference(definition, "adoption predicate", field)?;
            } else {
                fields.resolve_objective(
                    definition,
                    "adoption predicate",
                    predicate.objective(),
                )?;
            }
            if !seen_predicates.insert(predicate.objective()) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema: definition.id.to_string(),
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

    let has_timers = fields.values().any(|field| {
        matches!(
            field.descriptor.kind,
            crate::entity::state::StateFieldKind::Timer
                | crate::entity::state::StateFieldKind::Cooldown
        ) && !independently_ticked.contains(&field.objective)
    });
    let property_schedules_scan = !refresh_outputs.is_empty() || has_timers;
    let component_schedules_scan = !definition.components.is_empty()
        && !matches!(
            definition.reconcile,
            ReconcilePolicy::InitializeOnly | ReconcilePolicy::Manual
        );
    let needs_reconcile_scan = (!matches!(
        definition.reconcile,
        ReconcilePolicy::InitializeOnly | ReconcilePolicy::Manual
    ) && (!definition.properties.is_empty()
        || !definition.derivations.is_empty()
        || !definition.transitions.is_empty()
        || !definition.migrations.is_empty()
        || definition.version > 1))
        || property_schedules_scan
        || component_schedules_scan;
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

fn guard_component_cleanup(command: &str, retaining_markers: &[String]) -> String {
    if retaining_markers.is_empty() {
        return command.to_owned();
    }
    let guards = retaining_markers
        .iter()
        .map(|marker| format!("unless entity @s[tag={marker}]"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("execute {guards} run {command}")
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
    fields: &ArchetypeFields,
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
                    schema: definition.id.to_string(),
                    field: format!("transition[{index}]"),
                    range: format!("{basis_points} basis points"),
                });
            }
            for field in [current, maximum] {
                fields.resolve_reference(
                    definition,
                    format!("transition[{index}]"),
                    &field.reference,
                )?;
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
        fields.resolve_reference(definition, format!("transition[{index}]"), &field.reference)?;
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
    initialize_commands: Vec<String>,
    refresh_function: Option<String>,
}

fn compile_derivations(
    definition: &ArchetypeDefinition,
    fields: &ArchetypeFields,
    root: &str,
    marker: &str,
) -> Result<DerivationCompilation, EntityDiagnostic> {
    use crate::entity::curve::DependencyGraph;

    let id = definition.id.to_string();
    let mut graph = DependencyGraph::new();
    let mut targets = BTreeSet::new();
    for derivation in &definition.derivations {
        let target = derivation.target.objective();
        fields.resolve_reference(
            definition,
            format!("derivation `{}` target", derivation.name),
            &derivation.target.field_reference(),
        )?;
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
    let mut initialize_commands = Vec::new();
    for (index, derivation) in derivations.into_iter().enumerate() {
        let target = derivation.target.objective();
        let target_dirty = derivation.target.dirty_objective();
        let derivation_dirty =
            sand_commands::ObjectiveName::logical(format!("{id}.derive.{index}.dirty"))
                .as_str()
                .to_string();
        objectives.insert(derivation_dirty.clone());
        let curve_references = derivation.curve.field_references();
        for input in derivation.curve.inputs() {
            let source = if let Some(reference) = curve_references.get(&input) {
                fields.resolve_reference(
                    definition,
                    format!("derivation `{}` input", derivation.name),
                    reference,
                )?
            } else {
                fields.resolve_objective(
                    definition,
                    format!("derivation `{}` input", derivation.name),
                    &input,
                )?
            };
            let source_dirty = &source.dirty_objective;
            let pending = dirty_pending_name(source_dirty, marker);
            refresh_commands.push(format!(
                "execute if score @s {pending} matches 1 run scoreboard players set @s {derivation_dirty} 1"
            ));
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
        if derivation.fixed.scale() != derivation.target_scale {
            let mut conversion_objectives = BTreeSet::new();
            append_scale_conversion(
                definition,
                &mut conversion_objectives,
                &mut commands,
                &target,
                derivation.fixed.scale(),
                derivation.target_scale,
                RoundingPolicy::NearestTiesAwayFromZero,
                index,
            )?;
            objectives.extend(conversion_objectives);
        }
        append_destination_bounds(
            &mut commands,
            &target,
            derivation.target.descriptor().bounds,
        );
        commands.push(format!("scoreboard players set @s {target_dirty} 1"));
        records.push(function_record(definition.id.namespace(), &path, commands));
        initialize_commands.push(format!("function {}:{path}", definition.id.namespace()));
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
        initialize_commands,
        refresh_function,
    })
}

fn append_destination_bounds(
    commands: &mut Vec<String>,
    destination: &str,
    bounds: Option<(i32, i32)>,
) {
    if let Some((minimum, maximum)) = bounds {
        if minimum != i32::MIN {
            commands.push(format!(
                "execute if score @s {destination} matches ..{} run scoreboard players set @s {destination} {minimum}",
                minimum - 1
            ));
        }
        if maximum != i32::MAX {
            commands.push(format!(
                "execute if score @s {destination} matches {}.. run scoreboard players set @s {destination} {maximum}",
                maximum + 1
            ));
        }
    }
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
                source_scale,
                target_scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(definition, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} = @s {source}"
                ));
                append_scale_conversion(
                    definition,
                    &mut objectives,
                    &mut commands,
                    destination,
                    *source_scale,
                    *target_scale,
                    *rounding,
                    index,
                )?;
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

#[allow(clippy::too_many_arguments)] // Lowering keeps each scoreboard operand explicit.
fn append_scale_conversion(
    definition: &ArchetypeDefinition,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    source_scale: i64,
    target_scale: i64,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    debug_assert!(source_scale > 0 && target_scale > 0);
    let common = gcd(source_scale, target_scale);
    let multiplier = target_scale / common;
    let divisor = source_scale / common;

    if multiplier != 1 {
        let objective = constant_objective(
            definition,
            &format!("curve_rescale_multiplier_{index}"),
            multiplier,
        )?;
        objectives.insert(objective.clone());
        commands.push(format!(
            "scoreboard players set #value {objective} {multiplier}"
        ));
        commands.push(format!(
            "scoreboard players operation @s {destination} *= #value {objective}"
        ));
    }
    if divisor != 1 {
        append_scaled_division(
            definition,
            objectives,
            commands,
            destination,
            divisor,
            rounding,
            index,
        )?;
    }
    Ok(())
}

fn gcd(mut left: i64, mut right: i64) -> i64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.abs()
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
            let half = scratch("half_divisor");
            let divisor_parity = scratch("divisor_parity");
            let two = constant_objective(definition, "curve_two", 2)?;
            objectives.extend([half.clone(), divisor_parity.clone(), two.clone()]);
            commands.push(format!("scoreboard players set #value {two} 2"));
            commands.push(format!(
                "scoreboard players operation @s {half} = {divisor}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {half} /= #value {two}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {divisor_parity} = {divisor}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {divisor_parity} %= #value {two}"
            ));
            commands.push(format!(
                "execute if score @s {remainder} > @s {half} run scoreboard players add @s {destination} 1"
            ));
            if rounding == RoundingPolicy::NearestTiesAwayFromZero {
                commands.push(format!(
                    "execute if score @s {original} matches 0.. if score @s {remainder} = @s {half} if score @s {divisor_parity} matches 0 run scoreboard players add @s {destination} 1"
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
                    "execute if score @s {remainder} = @s {half} if score @s {divisor_parity} matches 0 unless score @s {parity} matches 0 run scoreboard players add @s {destination} 1"
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
    fields: &ArchetypeFields,
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
            fields.resolve_reference(
                definition,
                format!("property[{index}] health maximum"),
                &binding.max_health_field().field_reference(),
            )?;
            sources.push(binding.max_health_field().dirty_objective());
            if let Some(current) = binding.current_health_field() {
                fields.resolve_reference(
                    definition,
                    format!("property[{index}] health current"),
                    &current.field_reference(),
                )?;
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
                dirty_objective,
                field,
                ..
            } = binding.source()
            {
                fields.resolve_reference(
                    definition,
                    format!("property[{index}] attribute"),
                    field,
                )?;
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
                dirty_objective,
                field,
                ..
            } = binding.source()
            {
                fields.resolve_reference(
                    definition,
                    format!("property[{index}] attribute modifier"),
                    field,
                )?;
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
            fields.resolve_reference(
                definition,
                format!("property[{index}] conditional effect"),
                &flag.field_reference(),
            )?;
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
            fields.resolve_reference(
                definition,
                format!("property[{index}] conditional equipment"),
                &flag.field_reference(),
            )?;
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
            for segment in binding.text_value().segments() {
                match segment {
                    EntityTextSegment::Canonical { .. } | EntityTextSegment::Literal { .. } => {}
                    EntityTextSegment::Numeric {
                        dirty_objective,
                        field,
                        ..
                    }
                    | EntityTextSegment::Enum {
                        dirty_objective,
                        field,
                        ..
                    }
                    | EntityTextSegment::Flag {
                        dirty_objective,
                        field,
                        ..
                    } => {
                        fields.resolve_reference(
                            definition,
                            format!("property[{index}] name"),
                            field,
                        )?;
                        sources.push(dirty_objective.clone());
                    }
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
            fields.resolve_reference(
                definition,
                format!("property[{index}] conditional tag"),
                &flag.field_reference(),
            )?;
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
            fields.resolve_reference(
                definition,
                format!("property[{index}] conditional team"),
                &flag.field_reference(),
            )?;
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
        NumericPropertySource::StateScore {
            objective, scale, ..
        } => {
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
                        "execute store result storage {storage} {args}.value double {} run scoreboard players get @s {objective}",
                        1.0 / f64::from(*scale)
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
        NumericPropertySource::StateScore {
            objective, scale, ..
        } => {
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
                        "execute store result storage {storage} {args}.value double {} run scoreboard players get @s {objective}",
                        1.0 / f64::from(*scale)
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
    binding: &EntityName,
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
    for (segment_index, segment) in binding.text_value().segments().iter().enumerate() {
        let color = match segment {
            EntityTextSegment::Canonical { .. } => String::new(),
            EntityTextSegment::Literal { color, .. }
            | EntityTextSegment::Numeric { color, .. }
            | EntityTextSegment::Enum { color, .. }
            | EntityTextSegment::Flag { color, .. } => color
                .as_ref()
                .map(|color| format!(",color:\"{color}\""))
                .unwrap_or_default(),
        };
        match segment {
            EntityTextSegment::Canonical { component } => rendered.push(component.to_string()),
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
    if definition.version == 0 {
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
    use crate::entity::state::{
        EntityFlag, EntityScore, EntityState, FixedScore, NumericStateField, StateFieldDescriptor,
        StateFieldKind,
    };

    struct MobState;
    static FIELDS: &[StateFieldDescriptor] = &[
        StateFieldDescriptor::new("level", StateFieldKind::Score, 1, Some((1, 100))),
        StateFieldDescriptor::new("health", StateFieldKind::Score, 20, Some((1, 2_000))),
        StateFieldDescriptor::new("speed", StateFieldKind::Fixed(100), 125, Some((0, 1_000))),
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
    impl StateComposition for MobState {
        fn composition_identities() -> Vec<(String, u32)> {
            vec![("rpg:mob.presence".into(), 2)]
        }

        fn composition_schemas() -> Vec<StateSchema> {
            vec![Self::schema()]
        }

        fn composition_attach(_: &'static str) -> Vec<String> {
            Vec::new()
        }

        fn composition_detach(_: &'static str) -> Vec<String> {
            Vec::new()
        }
    }
    struct ConflictingMobState;
    static CONFLICTING_FIELDS: &[StateFieldDescriptor] = &[StateFieldDescriptor::new(
        "other",
        StateFieldKind::Score,
        0,
        None,
    )];
    impl EntityState for ConflictingMobState {
        fn schema() -> StateSchema {
            StateSchema {
                namespace: "rpg",
                name: "mob",
                version: 2,
                fields: CONFLICTING_FIELDS,
            }
        }
    }
    impl StateComposition for ConflictingMobState {
        fn composition_identities() -> Vec<(String, u32)> {
            vec![("rpg:mob.presence".into(), 2)]
        }

        fn composition_schemas() -> Vec<StateSchema> {
            vec![Self::schema()]
        }

        fn composition_attach(_: &'static str) -> Vec<String> {
            Vec::new()
        }

        fn composition_detach(_: &'static str) -> Vec<String> {
            Vec::new()
        }
    }
    const LEVEL: EntityScore<i32> = EntityScore::new("rpg", "mob", "level", 1, Some((1, 100)));
    const HEALTH: EntityScore<i32> = EntityScore::new("rpg", "mob", "health", 20, Some((1, 2_000)));
    const SPEED: FixedScore = FixedScore::__new("rpg", "mob", "speed", 100, 125, Some((0, 1_000)));
    const SICK: EntityFlag = EntityFlag::new("rpg", "mob", "sick", false);
    struct TimedState;
    static TIMED_FIELDS: &[StateFieldDescriptor] = &[
        StateFieldDescriptor::new("timer", StateFieldKind::Timer, 0, Some((0, i32::MAX))),
        StateFieldDescriptor::new("cooldown", StateFieldKind::Cooldown, 0, Some((0, i32::MAX))),
    ];
    impl EntityState for TimedState {
        fn schema() -> StateSchema {
            StateSchema {
                namespace: "rpg",
                name: "timed",
                version: 1,
                fields: TIMED_FIELDS,
            }
        }
    }
    impl StateComposition for TimedState {
        fn composition_identities() -> Vec<(String, u32)> {
            vec![("rpg:timed.presence".into(), 1)]
        }

        fn composition_schemas() -> Vec<StateSchema> {
            vec![Self::schema()]
        }

        fn composition_attach(_: &'static str) -> Vec<String> {
            Vec::new()
        }

        fn composition_detach(_: &'static str) -> Vec<String> {
            Vec::new()
        }
    }
    const TIMER: crate::entity::state::EntityTimer =
        crate::entity::state::EntityTimer::new("rpg", "timed", "timer", 0);
    const COOLDOWN: crate::entity::state::EntityCooldown =
        crate::entity::state::EntityCooldown::new("rpg", "timed", "cooldown");

    fn profile() -> crate::version::VersionProfile {
        crate::version::VersionProfile::resolve(
            &crate::version::MinecraftVersion::parse("26.2").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn compile_is_repeat_deterministic_and_initialization_marks_last() {
        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "plagued_zombie").unwrap(),
        )
        .components::<MobState>()
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
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "manual").unwrap())
                .components::<MobState>()
                .reconcile(ReconcilePolicy::Manual);
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        assert!(compiled.tick_functions.is_empty());
        assert_eq!(compiled.report.outer_scans_per_cycle, 0);
    }

    #[test]
    fn component_versions_schedule_default_reconciliation() {
        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "component_upgrade").unwrap(),
        )
        .components::<MobState>();
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        assert!(
            compiled
                .records
                .iter()
                .any(|record| record.path.ends_with("/reconcile_scan"))
        );
        assert_eq!(compiled.report.outer_scans_per_cycle, 1);
    }

    #[test]
    fn shared_component_dirty_state_is_distributed_once_to_every_active_archetype() {
        let first = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "shared_first").unwrap(),
        )
        .components::<MobState>()
        .derive(HEALTH, StatCurve::state(LEVEL))
        .attribute(AttributeBinding::new(
            sand_components::AttributeType::AttackDamage,
            NumericPropertySource::state(LEVEL),
        ))
        .definition();
        let second = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "shared_second").unwrap(),
        )
        .components::<MobState>()
        .attribute(AttributeBinding::new(
            sand_components::AttributeType::AttackDamage,
            NumericPropertySource::state(LEVEL),
        ))
        .definition();
        let claims = component_claims(&[first.clone(), second.clone()]);
        let compiled = compile_definition_with_claims(&first, &profile(), &claims).unwrap();
        let reconcile = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        let second_marker = initialized_tag(&second.id.to_string());
        let first_pending = dirty_pending_name(
            &LEVEL.dirty_objective(),
            &initialized_tag(&first.id.to_string()),
        );
        let second_pending = dirty_pending_name(&LEVEL.dirty_objective(), &second_marker);
        assert!(reconcile.content.contains(&format!(
            "tag={second_marker}] run scoreboard players set @s {second_pending} 1"
        )));
        assert!(
            reconcile
                .content
                .contains(&format!("scoreboard players set @s {first_pending} 0"))
        );
        let derive_refresh = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/derive_refresh"))
            .unwrap();
        assert!(
            derive_refresh
                .content
                .contains(&format!("{first_pending} matches 1"))
        );
        let property_refresh = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/refresh"))
            .unwrap();
        assert!(
            property_refresh
                .content
                .contains(&format!("{first_pending} matches 1"))
        );
        assert!(!property_refresh.content.contains(&format!(
            "scoreboard players set @s {} 0",
            LEVEL.dirty_objective()
        )));
        assert!(!reconcile.content.lines().any(|line| {
            line == format!("scoreboard players set @s {} 0", LEVEL.dirty_objective())
        }));
    }

    #[test]
    fn repaired_components_refresh_derived_and_native_state() {
        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "repair_refresh").unwrap(),
        )
        .components::<MobState>()
        .derive(HEALTH, StatCurve::state(LEVEL))
        .health(HealthBinding::new(HEALTH));
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        let reconcile = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        let repair = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/repair_refresh"))
            .unwrap();
        let repair_objective = sand_commands::ObjectiveName::logical(format!(
            "{}.component_repair",
            archetype.definition().id
        ))
        .as_str()
        .to_string();
        assert!(reconcile.content.contains(&repair_objective));
        assert!(reconcile.content.contains("/repair_refresh"));
        assert!(repair.content.contains("/derive/0"));
        assert!(repair.content.contains("/property/0"));
    }

    #[test]
    fn composed_timers_and_cooldowns_tick_and_mark_dirty() {
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "timed").unwrap())
                .components::<TimedState>();
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        let reconcile = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        for field in [TIMER.objective(), COOLDOWN.objective()] {
            assert!(reconcile.content.contains(&format!(
                "execute if score @s {field} matches 1.. run scoreboard players remove @s {field} 1"
            )));
        }
        assert!(reconcile.content.contains(&TIMER.dirty_objective()));
        assert!(reconcile.content.contains(&COOLDOWN.dirty_objective()));

        let mut independently_ticked = archetype;
        independently_ticked.components[0].auto_tick_objectives =
            vec![TIMER.objective(), COOLDOWN.objective()];
        let compiled = compile_definition(&independently_ticked.definition(), &profile()).unwrap();
        let reconcile = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        assert!(!reconcile.content.contains(&format!(
            "scoreboard players remove @s {} 1",
            TIMER.objective()
        )));
        assert!(!reconcile.content.contains(&format!(
            "scoreboard players remove @s {} 1",
            COOLDOWN.objective()
        )));
    }

    #[test]
    fn shared_component_timers_tick_once_per_entity() {
        let first = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "timed_first").unwrap(),
        )
        .components::<TimedState>()
        .definition();
        let second = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "timed_second").unwrap(),
        )
        .components::<TimedState>()
        .definition();
        let claims = component_claims(&[first.clone(), second.clone()]);
        let first_compiled = compile_definition_with_claims(&first, &profile(), &claims).unwrap();
        let second_compiled = compile_definition_with_claims(&second, &profile(), &claims).unwrap();
        let first_reconcile = first_compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        let second_reconcile = second_compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/reconcile"))
            .unwrap();
        let first_marker = initialized_tag(&first.id.to_string());
        let second_marker = initialized_tag(&second.id.to_string());
        let decrement = format!("scoreboard players remove @s {} 1", TIMER.objective());
        assert!(first_reconcile.content.contains(&decrement));
        assert!(second_reconcile.content.contains(&decrement));
        let (owner, guarded, owner_marker) = if first_marker < second_marker {
            (first_reconcile, second_reconcile, first_marker)
        } else {
            (second_reconcile, first_reconcile, second_marker)
        };
        assert!(owner.content.contains(&format!(
            "execute if score @s {} matches 1.. run {decrement}",
            TIMER.objective()
        )));
        assert!(
            guarded
                .content
                .contains(&format!("unless entity @s[tag={owner_marker}]"))
        );
    }

    #[test]
    fn migration_gap_is_structured_error() {
        let function = "rpg:migrate".parse::<FunctionId>().unwrap();
        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "bad_migration").unwrap(),
        )
        .components::<MobState>()
        .migration(Migration::new(1, 2, function.clone()))
        .migration(Migration::new(3, 4, function));
        let error = compile_definition(&archetype.definition(), &profile()).unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-MIGRATION-GAP");
    }

    #[test]
    fn summon_uses_direct_typed_execute_summon_without_scratch_identity() {
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "summoned").unwrap())
                .components::<MobState>();
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
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "scaled").unwrap())
                .components::<MobState>()
                .derive(
                    HEALTH,
                    StatCurve::linear(StatCurve::state(LEVEL), 2.0, 18.0),
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
    fn derivations_infer_destination_and_input_scales() {
        let fixed_target = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "fixed_target").unwrap(),
        )
        .components::<MobState>()
        .derive(SPEED, StatCurve::state(HEALTH));
        let definition = fixed_target.definition();
        assert_eq!(definition.derivations[0].fixed().scale(), 100);
        assert_eq!(
            definition.derivations[0].output_encoding(),
            DerivedScoreEncoding::FixedPoint
        );
        let compiled = compile_definition(&definition, &profile()).unwrap();
        let derive = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/derive/0"))
            .unwrap();
        assert!(derive.content.contains(" 100"));
        assert!(derive.content.contains("*="));
        assert!(derive.content.contains("matches 1001.."));

        let whole_target = EntityDerivation::for_target(HEALTH, StatCurve::state(SPEED));
        assert_eq!(whole_target.fixed().scale(), 1_000);
        assert_eq!(whole_target.output_encoding(), DerivedScoreEncoding::Whole);

        let unit_fixed = FixedScore::__new("rpg", "mob", "unit_fixed", 1, 0, None);
        let unit_fixed_target = EntityDerivation::for_target(unit_fixed, StatCurve::constant(1.0));
        assert_eq!(
            unit_fixed_target.output_encoding(),
            DerivedScoreEncoding::FixedPoint
        );

        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "whole_target").unwrap(),
        )
        .components::<MobState>()
        .derive_with(whole_target);
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        let derive = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/derive/0"))
            .unwrap();
        // FixedScore(scale=100) -> working scale 1,000 -> Score(scale=1).
        assert!(derive.content.contains(" 10"));
        assert!(derive.content.contains(" 1000"));
    }

    #[test]
    fn destination_conversion_uses_canonical_rounding() {
        let working =
            FixedPoint::new(1_000, RoundingPolicy::TowardZero, OverflowPolicy::Error).unwrap();
        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "canonical_destination_rounding").unwrap(),
        )
        .components::<MobState>()
        .derive_with(
            EntityDerivation::for_target(SPEED, StatCurve::constant(1.239)).fixed_point(working),
        );
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        let derive = compiled
            .records
            .iter()
            .find(|record| record.path.ends_with("/derive/0"))
            .unwrap();
        assert!(derive.content.lines().any(|line| {
            line.contains("matches 0..") && line.contains("run scoreboard players add")
        }));
        assert!(!derive.content.lines().any(|line| {
            line.contains("matches ..-1") && line.contains("run scoreboard players add")
        }));
    }

    #[test]
    fn erased_fixed_destination_retains_its_scale_when_reused() {
        let derivation = EntityDerivation::for_target(SPEED, StatCurve::constant(1.25));
        let target = derivation.target();
        assert_eq!(target.numeric_scale(), 100);

        let source = NumericPropertySource::state(target);
        assert!(matches!(
            source,
            NumericPropertySource::StateScore { scale: 100, .. }
        ));

        let lowered = StatCurve::state(target)
            .lower_scoreboard("result", "rpg:mob.result", FixedPoint::default())
            .unwrap();
        assert!(lowered.operations().iter().any(|operation| matches!(
            operation,
            LoweredCurveOperation::ScoreToFixed {
                source_scale: 100,
                ..
            }
        )));

        let health = EntityScore::<i32>::new("rpg", "mob", "health", 20, None);
        let commands = health.bind().add(target.bind());
        assert!(
            commands
                .iter()
                .any(|command| command.contains("#divisor") && command.ends_with(" 100"))
        );

        let commands = target.bind().add(1);
        assert!(commands[0].starts_with("scoreboard players add @s "));
        assert!(commands[0].ends_with(" 100"));

        let commands = target.bind().subtract(health.bind());
        assert!(
            commands
                .iter()
                .any(|command| command.contains("#multiplier") && command.ends_with(" 100"))
        );
    }

    #[test]
    fn duplicate_native_ownership_is_rejected() {
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "conflict").unwrap())
                .components::<MobState>()
                .health(HealthBinding::new(HEALTH))
                .health(HealthBinding::new(HEALTH));
        let error = compile_definition(&archetype.definition(), &profile()).unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-OWNERSHIP");
    }

    #[test]
    fn identity_equal_components_with_conflicting_metadata_are_rejected() {
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "conflict").unwrap())
                .components::<MobState>()
                .components::<ConflictingMobState>();
        let error = compile_definition(&archetype.definition(), &profile()).unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-STATE-DUPLICATE");
        assert!(error.to_string().contains("rpg:mob.presence@2"));
        assert!(error.to_string().contains("lifecycle identity"));
    }

    #[test]
    fn many_properties_share_one_reconciliation_scan() {
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "dedup").unwrap())
                .components::<MobState>()
                .health(HealthBinding::new(HEALTH))
                .attribute(AttributeBinding::new(
                    sand_components::AttributeType::AttackDamage,
                    NumericPropertySource::state(LEVEL),
                ))
                .name(EntityName::new().state(LEVEL, sand_commands::ChatColor::Yellow));
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
    fn native_numeric_binding_consumes_fixed_score_in_logical_units() {
        let archetype = EntityArchetype::<ZombieKind>::new(
            ResourceLocation::new("rpg", "fixed_attribute").unwrap(),
        )
        .components::<MobState>()
        .attribute(AttributeBinding::new(
            sand_components::AttributeType::MovementSpeed,
            NumericPropertySource::state(SPEED),
        ));
        let compiled = compile_definition(&archetype.definition(), &profile()).unwrap();
        let refresh = compiled
            .records
            .iter()
            .find(|record| record.content.contains("double 0.01"))
            .unwrap();
        assert!(refresh.content.contains("double 0.01"));
        assert!(refresh.content.contains(&SPEED.objective()));
    }

    #[test]
    fn invalid_health_percentage_is_a_structured_range_error() {
        let archetype =
            EntityArchetype::<ZombieKind>::new(ResourceLocation::new("rpg", "percentage").unwrap())
                .components::<MobState>()
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
