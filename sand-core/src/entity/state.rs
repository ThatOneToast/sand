//! Typed scoreboard-backed state for entity, living-entity, player, and global
//! schemas.
//!
//! Entity and living-entity state normally binds to the entity currently
//! executing as `@s`. Its accessors may mark deterministic hidden source-dirty
//! objectives so an archetype can reconcile only outputs that depend on the
//! changed field. State survives chunk unloading with the entity, but generated
//! timers and observers do not run while the entity is unloaded.
//!
//! Player state binds to the current player without dirty tracking. Global
//! state binds to a deterministic fake-player score holder, also without dirty
//! tracking. Accessors for every scope emit Minecraft commands; they do not
//! mutate Rust memory.
//!
//! Scoreboard persistence is deliberate: arbitrary custom top-level entity NBT
//! is not a reliable persistence mechanism in Minecraft 26.2.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

use sand_commands::selector::ScoreRange as SelectorScoreRange;

use crate::condition::{Condition, ScoreRange};
use crate::entity::diagnostic::EntityDiagnostic;

#[doc = "**API Contract:** Run `sand api show sand::entity::EnumEncoding` for the canonical contract."]
/// One enum variant's stable scoreboard encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumEncoding {
    #[doc = "**API Contract:** Run `sand api show sand::entity::EnumEncoding::name` for the canonical contract."]
    /// Rust-facing variant name used by diagnostics.
    pub name: &'static str,
    #[doc = "**API Contract:** Run `sand api show sand::entity::EnumEncoding::score` for the canonical contract."]
    /// Integer stored in the scoreboard.
    pub score: i32,
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumValue` for the canonical contract."]
/// A finite typed value stored by [`EntityEnum`].
///
/// `#[derive(EntityStateEnum)]` is the normal implementation path. Manual
/// implementations remain supported for established wire formats.
pub trait EntityEnumValue: Copy + Eq + fmt::Debug + 'static {
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumValue::ENCODINGS` for the canonical contract."]
    /// Complete stable encoding table.
    const ENCODINGS: &'static [EnumEncoding];

    /// Convert a value to its declared score.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumValue::encode` for the canonical contract."]
    fn encode(self) -> i32;
}

#[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind` for the canonical contract."]
/// Persistence and runtime behavior of a state field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StateFieldKind {
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Score` for the canonical contract."]
    /// Signed integer or fixed-point score.
    Score,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Fixed` for the canonical contract."]
    /// Fixed-point score encoded with the carried positive scale.
    Fixed(
        #[doc = "Number of scoreboard units representing one whole value."]
        #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Fixed::0` for the canonical contract."]
        i32,
    ),
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Flag` for the canonical contract."]
    /// Boolean encoded as zero or one.
    Flag,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Enum` for the canonical contract."]
    /// Finite enum encoded without string matching.
    Enum(
        #[doc = "The `Enum` variant carries the value described by its variant semantics: Finite enum encoded without string matching."]
        #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Enum::0` for the canonical contract."]
        &'static [EnumEncoding],
    ),
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Timer` for the canonical contract."]
    /// Elapsed/countdown timer.
    Timer,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Cooldown` for the canonical contract."]
    /// Reusable countdown that is ready at zero.
    Cooldown,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Version` for the canonical contract."]
    /// Schema or archetype version.
    Version,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldKind::Dirty` for the canonical contract."]
    /// Generated source/output dirty bit.
    Dirty,
}

#[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldDescriptor` for the canonical contract."]
/// Static metadata for one schema field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateFieldDescriptor {
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldDescriptor::name` for the canonical contract."]
    /// Rust-facing field name.
    pub name: &'static str,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldDescriptor::kind` for the canonical contract."]
    /// Storage/behavior family.
    pub kind: StateFieldKind,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldDescriptor::default` for the canonical contract."]
    /// Initial score assigned only when the value is missing.
    pub default: i32,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldDescriptor::bounds` for the canonical contract."]
    /// Optional inclusive bounds.
    pub bounds: Option<(i32, i32)>,
}

impl StateFieldDescriptor {
    /// Construct field metadata.
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateFieldDescriptor::new` for the canonical contract."]
    #[must_use]
    pub const fn new(
        name: &'static str,
        kind: StateFieldKind,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self {
            name,
            kind,
            default,
            bounds,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema` for the canonical contract."]
/// Complete metadata for a typed state schema.
#[derive(Debug, Clone, Copy)]
pub struct StateSchema {
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema::namespace` for the canonical contract."]
    /// Namespace used in generated logical names.
    pub namespace: &'static str,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema::name` for the canonical contract."]
    /// Schema name within the namespace.
    pub name: &'static str,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema::version` for the canonical contract."]
    /// Current version; zero is reserved for an uninitialized entity.
    pub version: u32,
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema::fields` for the canonical contract."]
    /// Fields in source declaration order.
    pub fields: &'static [StateFieldDescriptor],
}

impl StateSchema {
    /// Logical `namespace:name` identifier.
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema::id` for the canonical contract."]
    #[must_use]
    pub fn id(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }

    /// Validate field names, bounds, resolved objective names, and enum encodings.
    #[doc = "**API Contract:** Run `sand api show sand::entity::StateSchema::validate` for the canonical contract."]
    pub fn validate(&self) -> Result<(), EntityDiagnostic> {
        let schema = self.id();
        let mut names = BTreeSet::new();
        let mut objectives = BTreeMap::<String, &'static str>::new();
        for field in self.fields {
            if !names.insert(field.name) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema,
                    field: field.name.to_string(),
                    detail: "field name is declared more than once".into(),
                });
            }
            if let Some((min, max)) = field.bounds
                && min > max
            {
                return Err(EntityDiagnostic::InvalidRange {
                    schema,
                    field: field.name.into(),
                    range: format!("{min}..={max}"),
                });
            }
            if let StateFieldKind::Enum(encodings) = field.kind {
                validate_enum_encodings(&schema, field.name, encodings)?;
            }
            let objective = objective_name(self.namespace, self.name, field.name);
            if let Some(previous) = objectives.insert(objective.clone(), field.name) {
                return Err(EntityDiagnostic::DuplicateStateField {
                    schema,
                    field: field.name.into(),
                    detail: format!(
                        "objective `{objective}` collides with `{previous}` after Minecraft's 16-character limit"
                    ),
                });
            }
        }
        Ok(())
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityState` for the canonical contract."]
/// A type-level State schema.
///
/// The derive macro generates this implementation and associated typed field
/// constants. Manual implementations must return stable immutable metadata.
pub trait EntityState: 'static {
    /// Return this schema's metadata.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityState::schema` for the canonical contract."]
    fn schema() -> StateSchema;

    /// Storage-backed fields owned by this component.
    ///
    /// **API Contract:** Run `sand api show sand::entity::EntityState::data_fields`.
    fn data_fields() -> &'static [StateDataFieldDescriptor] {
        &[]
    }
}

/// Compiler metadata for one component-owned typed storage path.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateDataFieldDescriptor {
    pub storage: &'static str,
    pub path: &'static str,
    pub default_snbt: &'static str,
    pub keyed: bool,
}

impl StateDataFieldDescriptor {
    #[doc(hidden)]
    pub const fn new(
        storage: &'static str,
        path: &'static str,
        default_snbt: &'static str,
    ) -> Self {
        Self {
            storage,
            path,
            default_snbt,
            keyed: false,
        }
    }

    #[doc(hidden)]
    pub const fn keyed(
        storage: &'static str,
        path: &'static str,
        default_snbt: &'static str,
    ) -> Self {
        Self {
            storage,
            path,
            default_snbt,
            keyed: true,
        }
    }
}

/// Compiler-facing composition contract implemented by generated State
/// components and State bundles.
#[doc(hidden)]
pub trait StateBundleMember: 'static {
    /// Concrete holder-bound view generated for this member.
    type Bound;
    /// Hidden owner-scope proof used to reject incompatible bundles.
    type Scope: StateScopeMarker;

    /// Bind every nested component to one execution-scoped holder.
    fn bind_member(holder: &'static str) -> Self::Bound;

    /// Attach all unique nested components in declaration order.
    fn attach_member(holder: &'static str) -> Vec<String>;

    /// Detach all unique nested components in reverse declaration order.
    fn detach_member(holder: &'static str) -> Vec<String>;

    /// Resolved presence objectives and accepted component versions for query filtering.
    fn presence_requirements() -> Vec<(String, u32)>;
}

/// A State component or nested bundle that can participate in archetype composition.
///
/// Implementations are generated by `State` and `StateBundle`; authors use
/// this capability through [`crate::entity::EntityArchetype::components`].
///
/// **API Contract:** Run `sand api show sand::entity::StateComposition`.
pub trait StateComposition: 'static {
    /// Returns flattened component identities for conflict detection and deduplication.
    ///
    /// **API Contract:** Run `sand api show sand::entity::StateComposition::composition_identities`.
    fn composition_identities() -> Vec<(String, u32)>;
    /// Lowers idempotent canonical attachment for this component composition.
    ///
    /// **API Contract:** Run `sand api show sand::entity::StateComposition::composition_attach`.
    fn composition_attach(holder: &'static str) -> Vec<String>;
    /// Lowers ownership-safe canonical detachment in reverse composition order.
    ///
    /// **API Contract:** Run `sand api show sand::entity::StateComposition::composition_detach`.
    fn composition_detach(holder: &'static str) -> Vec<String>;
}

impl<T> StateComposition for T
where
    T: StateBundleMember,
    T::Scope: ArchetypeStateScope,
{
    fn composition_identities() -> Vec<(String, u32)> {
        T::presence_requirements()
    }

    fn composition_attach(holder: &'static str) -> Vec<String> {
        T::attach_member(holder)
    }

    fn composition_detach(holder: &'static str) -> Vec<String> {
        T::detach_member(holder)
    }
}

/// Hidden type-level owner-scope marker implemented by generated State components.
#[doc(hidden)]
pub trait StateScopeMarker: 'static {}

/// Scope proof for components that may be composed into an entity archetype.
#[doc(hidden)]
pub trait ArchetypeStateScope: StateScopeMarker {}

/// Hidden proof that two generated component scopes are identical.
#[doc(hidden)]
pub trait SameStateScope<Rhs: StateScopeMarker>: StateScopeMarker {}

impl<T: StateScopeMarker> SameStateScope<T> for T {}

#[doc(hidden)]
pub struct PlayerStateScope;
#[doc(hidden)]
pub struct EntityStateScope;
#[doc(hidden)]
pub struct LivingStateScope;
#[doc(hidden)]
pub struct GlobalStateScope;

impl StateScopeMarker for PlayerStateScope {}
impl StateScopeMarker for EntityStateScope {}
impl StateScopeMarker for LivingStateScope {}
impl StateScopeMarker for GlobalStateScope {}
impl ArchetypeStateScope for EntityStateScope {}
impl ArchetypeStateScope for LivingStateScope {}

/// Construct the typed exact-version predicate used by generated State queries.
#[doc(hidden)]
pub fn state_presence_predicate(objective: String, version: u32) -> StatePredicate {
    exact_predicate(objective, version as i32)
}

/// Build the canonical idempotent attachment sequence for a component.
///
/// This is compiler wiring used by `#[derive(State)]`. Dependencies are
/// provisioned by the generated load function; attachment initializes only
/// missing values and publishes the component version last.
#[doc(hidden)]
pub fn state_attach_commands<S: EntityState>(
    holder: &'static str,
    presence_logical: &'static str,
    suppression_logical: &'static str,
    clear_suppression: bool,
) -> Vec<String> {
    let schema = S::schema();
    let presence = sand_commands::ObjectiveName::logical(presence_logical)
        .as_str()
        .to_owned();
    let suppression = sand_commands::ObjectiveName::logical(suppression_logical)
        .as_str()
        .to_owned();
    let mut commands = Vec::new();
    if clear_suppression {
        commands.push(format!("scoreboard players reset {holder} {suppression}"));
    }
    for field in schema.fields {
        let objective = objective_name(schema.namespace, schema.name, field.name);
        commands.push(format!(
            "execute unless score {holder} {objective} matches -2147483648.. run scoreboard players set {holder} {objective} {}",
            field.default
        ));
    }
    for field in S::data_fields() {
        commands.extend(state_data_initialize_commands(*field));
    }
    for command in crate::state::registry::initialize_hook_commands::<S>(holder) {
        commands.push(format!(
            "execute unless score {holder} {presence} matches 1.. run {command}"
        ));
    }
    for (from, to) in crate::state::registry::migration_steps::<S>() {
        for command in crate::state::registry::migrate_hook_commands::<S>(holder, from, to) {
            commands.push(format!(
                "execute if score {holder} {presence} matches {from} run {command}"
            ));
        }
        commands.push(format!(
            "execute if score {holder} {presence} matches {from} run scoreboard players set {holder} {presence} {to}"
        ));
    }
    commands.push(format!(
        "execute unless score {holder} {presence} matches 1.. run scoreboard players set {holder} {presence} {}",
        schema.version
    ));
    commands
}

/// Build ownership-safe component detachment commands.
#[doc(hidden)]
pub fn state_detach_commands<S: EntityState>(
    holder: &'static str,
    presence_logical: &'static str,
    suppression_logical: &'static str,
    track_dirty: bool,
    suppress_observation: bool,
) -> Vec<String> {
    let schema = S::schema();
    let presence = sand_commands::ObjectiveName::logical(presence_logical)
        .as_str()
        .to_owned();
    let suppression = sand_commands::ObjectiveName::logical(suppression_logical)
        .as_str()
        .to_owned();
    let mut commands = Vec::new();
    for command in crate::state::registry::cleanup_hook_commands::<S>(holder) {
        commands.push(format!(
            "execute if score {holder} {presence} matches 1.. run {command}"
        ));
    }
    for field in schema.fields {
        let objective = objective_name(schema.namespace, schema.name, field.name);
        commands.push(format!("scoreboard players reset {holder} {objective}"));
        if track_dirty {
            commands.push(format!(
                "scoreboard players reset {holder} {}",
                dirty_name(schema.namespace, schema.name, field.name)
            ));
        }
    }
    for field in S::data_fields() {
        commands.extend(state_data_remove_commands(*field));
    }
    commands.push(format!("scoreboard players reset {holder} {presence}"));
    if suppress_observation {
        commands.push(format!("scoreboard players set {holder} {suppression} 1"));
    }
    commands
}

/// Test whether a component is attached at its current version.
#[doc(hidden)]
pub fn state_attached_condition<S: EntityState>(
    holder: &'static str,
    presence_logical: &'static str,
) -> Condition {
    let presence = sand_commands::ObjectiveName::logical(presence_logical)
        .as_str()
        .to_owned();
    Condition::score(
        holder.to_owned(),
        presence,
        ScoreRange::Eq(S::schema().version as i32),
    )
}

/// Resolve a logical State objective through the canonical objective rules.
#[doc(hidden)]
pub fn resolve_state_objective(logical: &str) -> String {
    sand_commands::ObjectiveName::logical(logical)
        .as_str()
        .to_owned()
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField` for the canonical contract."]
/// Shared behavior implemented by all typed state field handles.
pub trait EntityStateField: Copy + 'static {
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField::Accessor` for the canonical contract."]
    /// Accessor returned by [`crate::entity::EntityContext::state`].
    type Accessor;

    /// Static field metadata.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField::descriptor` for the canonical contract."]
    fn descriptor(self) -> StateFieldDescriptor;

    /// Resolved scoreboard objective, at most 16 characters.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField::objective` for the canonical contract."]
    fn objective(self) -> String;

    /// Hidden source-dirty objective.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField::dirty_objective` for the canonical contract."]
    fn dirty_objective(self) -> String;

    /// Bind this field to the current executor.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField::bind` for the canonical contract."]
    fn bind(self) -> Self::Accessor {
        self.bind_to("@s", true)
    }

    /// Bind this field to an explicit typed score holder.
    ///
    /// `track_dirty` is enabled for entity/living state whose archetype
    /// reconciliation consumes the auxiliary objective. Player/global state
    /// passes `false` because its lifecycle owns only the primary objective.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityStateField::bind_to` for the canonical contract."]
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor;
}

#[doc = "**API Contract:** Run `sand api show sand::entity::StatePredicate` for the canonical contract."]
/// Typed score predicate consumable by [`crate::entity::EntityQuery`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePredicate {
    pub(crate) objective: String,
    pub(crate) selector_range: SelectorScoreRange,
    condition_range: ScoreRange,
}

impl StatePredicate {
    /// Generated scoreboard objective.
    #[doc = "**API Contract:** Run `sand api show sand::entity::StatePredicate::objective` for the canonical contract."]
    #[must_use]
    pub fn objective(&self) -> &str {
        &self.objective
    }

    /// Vanilla selector range used when this predicate constrains adoption or
    /// an [`crate::entity::EntityQuery`].
    #[doc = "**API Contract:** Run `sand api show sand::entity::StatePredicate::selector_range` for the canonical contract."]
    #[must_use]
    pub fn selector_range(&self) -> String {
        self.selector_range.to_string()
    }

    /// Turn this predicate into an `@s` condition.
    #[doc = "**API Contract:** Run `sand api show sand::entity::StatePredicate::condition` for the canonical contract."]
    #[must_use]
    pub fn condition(&self) -> Condition {
        Condition::score(
            "@s".into(),
            self.objective.clone(),
            self.condition_range.clone(),
        )
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityScore` for the canonical contract."]
/// Typed signed entity score.
#[derive(Debug, PartialEq, Eq)]
pub struct EntityScore<T = i32> {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    _marker: PhantomData<fn() -> T>,
}

#[doc = "**API Contract:** Run `sand api show sand::entity::Score` for the canonical contract."]
/// Canonical integer field marker for `#[derive(State)]` declarations.
pub type Score = EntityScore<i32>;

#[doc = "**API Contract:** Run `sand api show sand::entity::FixedScore` for the canonical contract."]
/// Canonical decimal field marker for `#[derive(State)]` declarations.
///
/// Values are stored as signed 32-bit scoreboard units. The generated schema
/// records a positive scale (1,000 by default), rounds decimal inputs to the
/// nearest unit with exact halves away from zero, then saturates at the field
/// bounds and the scoreboard range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedScore {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    scale: i32,
}

impl FixedScore {
    /// Construct a fixed-point field; normally generated by `State`.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn __new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        scale: i32,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Fixed(scale),
                default,
                bounds,
            ),
            scale,
        }
    }

    /// Number of scoreboard units representing one whole value.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScore::scale` for the canonical contract."]
    #[must_use]
    pub const fn scale(self) -> i32 {
        self.scale
    }

    /// Match a decimal range after applying this field's encoding.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScore::matches` for the canonical contract."]
    pub fn matches(self, range: impl RangeBounds<f64>) -> Result<StatePredicate, EntityDiagnostic> {
        let start = match range.start_bound() {
            Bound::Included(value) => Bound::Included(self.encode(*value)),
            Bound::Excluded(value) => Bound::Excluded(self.encode(*value)),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(value) => Bound::Included(self.encode(*value)),
            Bound::Excluded(value) => Bound::Excluded(self.encode(*value)),
            Bound::Unbounded => Bound::Unbounded,
        };
        predicate_for_range(
            self.objective(),
            format!("{}:{}", self.namespace, self.schema),
            self.descriptor.name,
            (start, end),
        )
    }

    fn encode(self, value: f64) -> i32 {
        encode_fixed(value, self.scale, self.descriptor.bounds)
    }
}

impl EntityStateField for FixedScore {
    type Accessor = FixedScoreAccessor;

    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }

    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }

    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }

    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        FixedScoreAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

/// A fixed-point field bound to one generated scoreboard holder.
#[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreAccessor` for the canonical contract."]
#[derive(Debug, Clone, Copy)]
pub struct FixedScoreAccessor {
    field: FixedScore,
    holder: &'static str,
    track_dirty: bool,
}

impl FixedScoreAccessor {
    /// Return the raw scoreboard view together with its declared scale.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreAccessor::get` for the canonical contract."]
    #[must_use]
    pub fn get(self) -> FixedScoreValue {
        FixedScoreValue {
            score: EntityScoreValue {
                objective: self.field.objective(),
                holder: self.holder,
            },
            scale: self.field.scale,
        }
    }

    /// Assign a decimal value using the field's deterministic encoding.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreAccessor::set` for the canonical contract."]
    #[must_use]
    pub fn set(self, value: f64) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            self.field.encode(value),
        )
    }

    /// Add a decimal value using the field's deterministic encoding.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreAccessor::add` for the canonical contract."]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, value: f64) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "add",
            self.field.encode(value),
        )
    }

    /// Subtract a decimal value using the field's deterministic encoding.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreAccessor::subtract` for the canonical contract."]
    #[must_use]
    pub fn subtract(self, value: f64) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "remove",
            self.field.encode(value),
        )
    }

    /// Build a condition over a decimal range.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreAccessor::matches` for the canonical contract."]
    pub fn matches(self, range: impl RangeBounds<f64>) -> Result<Condition, EntityDiagnostic> {
        let predicate = self.field.matches(range)?;
        Ok(Condition::score(
            self.holder.to_string(),
            predicate.objective,
            predicate.condition_range,
        ))
    }
}

/// Read-only fixed-point score view. Minecraft still stores integer units;
/// `scale()` tells display or data-copying code how to interpret them.
#[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreValue` for the canonical contract."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedScoreValue {
    score: EntityScoreValue,
    scale: i32,
}

impl FixedScoreValue {
    /// Emit the underlying scoreboard read command.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreValue::command` for the canonical contract."]
    #[must_use]
    pub fn command(&self) -> String {
        self.score.command()
    }

    /// Number of stored units per whole value.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreValue::scale` for the canonical contract."]
    #[must_use]
    pub const fn scale(&self) -> i32 {
        self.scale
    }

    /// Resolved objective name.
    #[doc = "**API Contract:** Run `sand api show sand::entity::FixedScoreValue::objective` for the canonical contract."]
    #[must_use]
    pub fn objective(&self) -> &str {
        self.score.objective()
    }
}

impl<T> Copy for EntityScore<T> {}
impl<T> Clone for EntityScore<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> EntityScore<T> {
    /// Construct a score field; normally generated by `State`.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScore::new` for the canonical contract."]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self::__new(
            namespace,
            schema,
            name,
            StateFieldKind::Score,
            default,
            bounds,
        )
    }

    /// Compiler-facing constructor for version and dirty score fields.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn __new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        kind: StateFieldKind,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(name, kind, default, bounds),
            _marker: PhantomData,
        }
    }

    /// Match an inclusive/open Rust range without handwritten selector maps.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScore::matches` for the canonical contract."]
    pub fn matches(self, range: impl RangeBounds<i32>) -> Result<StatePredicate, EntityDiagnostic> {
        predicate_for_range(
            self.objective(),
            format!("{}:{}", self.namespace, self.schema),
            self.descriptor.name,
            range,
        )
    }
}

impl<T: 'static> EntityStateField for EntityScore<T> {
    type Accessor = EntityScoreAccessor<T>;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityScoreAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityScoreAccessor` for the canonical contract."]
/// An [`EntityScore`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityScoreAccessor<T = i32> {
    field: EntityScore<T>,
    holder: &'static str,
    track_dirty: bool,
}

impl<T: 'static> EntityScoreAccessor<T> {
    /// Return a readable typed score view.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScoreAccessor::get` for the canonical contract."]
    #[must_use]
    pub fn get(self) -> EntityScoreValue {
        EntityScoreValue {
            objective: self.field.objective(),
            holder: self.holder,
        }
    }

    /// Assign a value, marking the source dirty for archetype-bound state.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScoreAccessor::set` for the canonical contract."]
    #[must_use]
    pub fn set(self, value: i32) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "set", value)
    }

    /// Add a value, marking the source dirty for archetype-bound state.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScoreAccessor::add` for the canonical contract."]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, value: i32) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "add", value)
    }

    /// Subtract a value, marking the source dirty for archetype-bound state.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScoreAccessor::subtract` for the canonical contract."]
    #[must_use]
    pub fn subtract(self, value: i32) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "remove", value)
    }

    /// Build a typed condition over this score.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScoreAccessor::matches` for the canonical contract."]
    pub fn matches(self, range: impl RangeBounds<i32>) -> Result<Condition, EntityDiagnostic> {
        let predicate = predicate_for_range(
            self.field.objective(),
            format!("{}:{}", self.field.namespace, self.field.schema),
            self.field.descriptor.name,
            range,
        )?;
        Ok(Condition::score(
            self.holder.to_string(),
            predicate.objective,
            predicate.condition_range,
        ))
    }
}

/// Read-only view of the current entity's score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityScoreValue {
    objective: String,
    holder: &'static str,
}

impl EntityScoreValue {
    /// Emit `scoreboard players get <holder> <objective>`.
    #[must_use]
    pub fn command(&self) -> String {
        format!("scoreboard players get {} {}", self.holder, self.objective)
    }

    /// Resolved objective name.
    #[must_use]
    pub fn objective(&self) -> &str {
        &self.objective
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlag` for the canonical contract."]
/// Typed boolean entity field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityFlag {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
}

impl EntityFlag {
    /// Construct a flag field; normally generated by `State`.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlag::new` for the canonical contract."]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        default: bool,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Flag,
                default as i32,
                Some((0, 1)),
            ),
        }
    }

    /// Predicate for an enabled flag.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlag::is_enabled` for the canonical contract."]
    #[must_use]
    pub fn is_enabled(self) -> StatePredicate {
        exact_predicate(self.objective(), 1)
    }

    /// Predicate for a disabled flag.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlag::is_disabled` for the canonical contract."]
    #[must_use]
    pub fn is_disabled(self) -> StatePredicate {
        exact_predicate(self.objective(), 0)
    }
}

impl EntityStateField for EntityFlag {
    type Accessor = EntityFlagAccessor;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityFlagAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlagAccessor` for the canonical contract."]
/// An [`EntityFlag`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityFlagAccessor {
    field: EntityFlag,
    holder: &'static str,
    track_dirty: bool,
}

impl EntityFlagAccessor {
    /// Set the flag to one.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlagAccessor::enable` for the canonical contract."]
    #[must_use]
    pub fn enable(self) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "set", 1)
    }
    /// Set the flag to zero.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlagAccessor::disable` for the canonical contract."]
    #[must_use]
    pub fn disable(self) -> Vec<String> {
        mutation(self.field, self.holder, self.track_dirty, "set", 0)
    }
    /// Condition: enabled.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlagAccessor::is_enabled` for the canonical contract."]
    #[must_use]
    pub fn is_enabled(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(1))
    }
    /// Condition: disabled.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityFlagAccessor::is_disabled` for the canonical contract."]
    #[must_use]
    pub fn is_disabled(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(0))
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnum` for the canonical contract."]
/// Typed finite entity enum.
#[derive(Debug, PartialEq, Eq)]
pub struct EntityEnum<T: EntityEnumValue> {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    _marker: PhantomData<fn() -> T>,
}

#[doc = "**API Contract:** Run `sand api show sand::entity::Data` for the canonical contract."]
/// Canonical typed-data field marker.
///
/// Data fields are lowered through Sand's typed storage backend by the State
/// derive. The marker carries no runtime Rust value.
#[derive(Debug, PartialEq, Eq)]
pub struct Data<T> {
    storage: &'static str,
    path: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Data<T> {}
impl<T> Clone for Data<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Data<T> {
    /// Constructs a generated component-owned typed storage handle.
    ///
    /// **API Contract:** Run `sand api show sand::entity::Data::new`.
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// Build a typed query for this component-owned storage value.
    ///
    /// **API Contract:** Run `sand api show sand::entity::Data::get`.
    pub fn get(self) -> String {
        crate::state::Nbt::storage(self.storage)
            .typed_path::<T>(self.path)
            .get()
            .to_string()
    }

    /// Replace this component-owned storage value.
    ///
    /// **API Contract:** Run `sand api show sand::entity::Data::set`.
    pub fn set(self, value: impl Into<sand_commands::NbtValue>) -> Vec<String> {
        vec![
            crate::state::Nbt::storage(self.storage)
                .typed_path::<T>(self.path)
                .set(value)
                .to_string(),
        ]
    }

    /// Test whether this component-owned storage path currently exists.
    ///
    /// **API Contract:** Run `sand api show sand::entity::Data::exists`.
    pub fn exists(self) -> Condition {
        Condition::nbt_exists(
            sand_commands::DataTarget::storage(self.storage),
            sand_commands::NbtPath::new(self.path),
        )
    }

    /// Remove this component-owned storage value.
    ///
    /// **API Contract:** Run `sand api show sand::entity::Data::remove`.
    pub fn remove(self) -> Vec<String> {
        vec![format!(
            "data remove storage {} {}",
            self.storage, self.path
        )]
    }
}

/// A typed command-storage field keyed by the current entity or player's UUID.
///
/// The UUID lives in a shared owner record while the value remains under its
/// component-owned path. Every operation copies the four UUID integers into a
/// short-lived macro argument compound and then runs a generated helper. This
/// keeps values stable across unload/reload without writing custom top-level
/// entity NBT.
#[doc = "**API Contract:** Run `sand api show sand::entity::KeyedData` for the canonical contract."]
#[derive(Debug, PartialEq, Eq)]
pub struct KeyedData<T> {
    storage: &'static str,
    path: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for KeyedData<T> {}
impl<T> Clone for KeyedData<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> KeyedData<T> {
    /// Construct a generated UUID-keyed storage handle.
    #[doc = "**API Contract:** Run `sand api show sand::entity::KeyedData::new` for the canonical contract."]
    #[must_use]
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// Read the current owner's value through a generated UUID-keyed helper.
    #[doc = "**API Contract:** Run `sand api show sand::entity::KeyedData::get` for the canonical contract."]
    #[must_use]
    pub fn get(self) -> Vec<String> {
        keyed_data_call(
            self.storage,
            format!(
                "$data get storage {} {}",
                self.storage,
                keyed_path(self.path)
            ),
        )
    }

    /// Replace the current owner's component-owned value.
    #[doc = "**API Contract:** Run `sand api show sand::entity::KeyedData::set` for the canonical contract."]
    #[must_use]
    pub fn set(self, value: impl Into<sand_commands::NbtValue>) -> Vec<String> {
        keyed_data_call(
            self.storage,
            format!(
                "$data modify storage {} {} set value {}",
                self.storage,
                keyed_path(self.path),
                value.into()
            ),
        )
    }

    /// Remove only the current owner's value for this component field.
    #[doc = "**API Contract:** Run `sand api show sand::entity::KeyedData::remove` for the canonical contract."]
    #[must_use]
    pub fn remove(self) -> Vec<String> {
        keyed_data_call(
            self.storage,
            format!(
                "$data remove storage {} {}",
                self.storage,
                keyed_path(self.path)
            ),
        )
    }

    /// Run commands only when the current owner's value actually exists.
    ///
    /// This callback is evaluated by Minecraft at runtime. It deliberately is
    /// not a Rust `Option` or ordinary [`Condition`], because command-storage
    /// membership depends on the executing entity's UUID.
    #[doc = "**API Contract:** Run `sand api show sand::entity::KeyedData::if_present` for the canonical contract."]
    #[must_use]
    pub fn if_present(self, body: impl FnOnce() -> Vec<String>) -> Vec<String> {
        let body = body();
        if body.is_empty() {
            return Vec::new();
        }
        let callback = crate::function::register_dyn_fn_dedup("sand/state_data/body", body);
        keyed_data_call(
            self.storage,
            format!(
                "$execute if data storage {} {} run function __sand_local:{}",
                self.storage,
                keyed_path(self.path),
                callback
            ),
        )
    }
}

pub(crate) fn state_data_initialize_commands(field: StateDataFieldDescriptor) -> Vec<String> {
    if !field.keyed {
        return vec![format!(
            "execute unless data storage {} {} run data modify storage {} {} set value {}",
            field.storage, field.path, field.storage, field.path, field.default_snbt
        )];
    }
    let owner_path = keyed_owner_path();
    let value_path = keyed_path(field.path);
    keyed_data_call(
        field.storage,
        format!(
            "$execute unless data storage {0} {owner_path} run data modify storage {0} owners append value {{uuid:{1},components:{{}}}}\n$execute unless data storage {0} {value_path} run data modify storage {0} {value_path} set value {2}",
            field.storage,
            keyed_uuid(),
            field.default_snbt,
        ),
    )
}

fn state_data_remove_commands(field: StateDataFieldDescriptor) -> Vec<String> {
    if field.keyed {
        keyed_data_call(
            field.storage,
            format!(
                "$data remove storage {} {}",
                field.storage,
                keyed_path(field.path)
            ),
        )
    } else {
        vec![format!(
            "data remove storage {} {}",
            field.storage, field.path
        )]
    }
}

fn keyed_uuid() -> &'static str {
    "[I;$(u0),$(u1),$(u2),$(u3)]"
}

fn keyed_owner_path() -> String {
    format!("owners[{{uuid:{}}}]", keyed_uuid())
}

fn keyed_path(path: &str) -> String {
    format!("{}.{}", keyed_owner_path(), path)
}

fn keyed_data_call(storage: &str, macro_body: String) -> Vec<String> {
    let path = crate::function::register_dyn_fn_dedup(
        "sand/state_data/keyed",
        macro_body.lines().map(str::to_owned).collect(),
    );
    let args = "__sand_owner";
    let mut commands = (0..4)
        .map(|index| {
            format!(
                "data modify storage {storage} {args}.u{index} set from entity @s UUID[{index}]"
            )
        })
        .collect::<Vec<_>>();
    commands.push(format!(
        "function __sand_local:{path} with storage {storage} {args}"
    ));
    commands
}

impl<T: EntityEnumValue> Copy for EntityEnum<T> {}
impl<T: EntityEnumValue> Clone for EntityEnum<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: EntityEnumValue> EntityEnum<T> {
    /// Construct an enum field from its default encoded score.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnum::new` for the canonical contract."]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        default_score: i32,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Enum(T::ENCODINGS),
                default_score,
                None,
            ),
            _marker: PhantomData,
        }
    }

    /// Predicate for exactly one variant.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnum::is` for the canonical contract."]
    #[must_use]
    pub fn is(self, value: T) -> StatePredicate {
        exact_predicate(self.objective(), value.encode())
    }
}

impl<T: EntityEnumValue> EntityStateField for EntityEnum<T> {
    type Accessor = EntityEnumAccessor<T>;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityEnumAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumAccessor` for the canonical contract."]
/// An [`EntityEnum`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityEnumAccessor<T: EntityEnumValue> {
    field: EntityEnum<T>,
    holder: &'static str,
    track_dirty: bool,
}

impl<T: EntityEnumValue> EntityEnumAccessor<T> {
    /// Store a variant, marking the source dirty for archetype-bound state.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumAccessor::set` for the canonical contract."]
    #[must_use]
    pub fn set(self, value: T) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            value.encode(),
        )
    }
    /// Condition: current value equals `value`.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumAccessor::is` for the canonical contract."]
    #[must_use]
    pub fn is(self, value: T) -> Condition {
        holder_condition(
            self.holder,
            self.field.objective(),
            ScoreRange::Eq(value.encode()),
        )
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimer` for the canonical contract."]
/// Typed elapsed/countdown entity timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTimer {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
}

impl EntityTimer {
    /// Construct a non-negative timer field.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimer::new` for the canonical contract."]
    #[must_use]
    pub const fn new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        initial: i32,
    ) -> Self {
        Self {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Timer,
                initial,
                Some((0, i32::MAX)),
            ),
        }
    }

    /// Predicate: timer reached zero.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimer::elapsed` for the canonical contract."]
    #[must_use]
    pub fn elapsed(self) -> StatePredicate {
        exact_predicate(self.objective(), 0)
    }
}

impl EntityStateField for EntityTimer {
    type Accessor = EntityTimerAccessor;
    fn descriptor(self) -> StateFieldDescriptor {
        self.descriptor
    }
    fn objective(self) -> String {
        objective_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn dirty_objective(self) -> String {
        dirty_name(self.namespace, self.schema, self.descriptor.name)
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityTimerAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimerAccessor` for the canonical contract."]
/// An [`EntityTimer`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityTimerAccessor {
    field: EntityTimer,
    holder: &'static str,
    track_dirty: bool,
}

impl EntityTimerAccessor {
    /// Start a countdown.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimerAccessor::start` for the canonical contract."]
    #[must_use]
    pub fn start(self, ticks: crate::state::Ticks) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            ticks.get().min(i32::MAX as u32) as i32,
        )
    }
    /// Decrement a positive timer for the bound score holder.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimerAccessor::tick` for the canonical contract."]
    #[must_use]
    pub fn tick(self) -> Vec<String> {
        let decrement = format!(
            "execute if score {1} {0} matches 1.. run scoreboard players remove {1} {0} 1",
            self.field.objective(),
            self.holder,
        );
        if self.track_dirty {
            vec![
                format!(
                    "execute if score {2} {0} matches 1.. run scoreboard players set {2} {1} 1",
                    self.field.objective(),
                    self.field.dirty_objective(),
                    self.holder,
                ),
                decrement,
            ]
        } else {
            vec![decrement]
        }
    }
    /// Condition: timer reached zero.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTimerAccessor::elapsed` for the canonical contract."]
    #[must_use]
    pub fn elapsed(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(0))
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityCooldown` for the canonical contract."]
/// Typed entity cooldown, ready at zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityCooldown(EntityTimer);

impl EntityCooldown {
    /// Construct a cooldown field.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityCooldown::new` for the canonical contract."]
    #[must_use]
    pub const fn new(namespace: &'static str, schema: &'static str, name: &'static str) -> Self {
        Self(EntityTimer {
            namespace,
            schema,
            descriptor: StateFieldDescriptor::new(
                name,
                StateFieldKind::Cooldown,
                0,
                Some((0, i32::MAX)),
            ),
        })
    }

    /// Predicate: cooldown is ready.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityCooldown::ready` for the canonical contract."]
    #[must_use]
    pub fn ready(self) -> StatePredicate {
        exact_predicate(self.objective(), 0)
    }
}

impl EntityStateField for EntityCooldown {
    type Accessor = EntityCooldownAccessor;
    fn descriptor(self) -> StateFieldDescriptor {
        self.0.descriptor
    }
    fn objective(self) -> String {
        self.0.objective()
    }
    fn dirty_objective(self) -> String {
        self.0.dirty_objective()
    }
    fn bind_to(self, holder: &'static str, track_dirty: bool) -> Self::Accessor {
        EntityCooldownAccessor {
            field: self,
            holder,
            track_dirty,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityCooldownAccessor` for the canonical contract."]
/// An [`EntityCooldown`] bound to its schema-selected score holder.
///
/// Entity/living schemas normally use the current executor (`@s`), player
/// schemas use the current player (`@s`), and global schemas use their
/// deterministic fake-player holder.
#[derive(Debug, Clone, Copy)]
pub struct EntityCooldownAccessor {
    field: EntityCooldown,
    holder: &'static str,
    track_dirty: bool,
}

impl EntityCooldownAccessor {
    /// Start the cooldown.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityCooldownAccessor::start` for the canonical contract."]
    #[must_use]
    pub fn start(self, ticks: crate::state::Ticks) -> Vec<String> {
        mutation(
            self.field,
            self.holder,
            self.track_dirty,
            "set",
            ticks.get().min(i32::MAX as u32) as i32,
        )
    }
    /// Condition: ready.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityCooldownAccessor::ready` for the canonical contract."]
    #[must_use]
    pub fn ready(self) -> Condition {
        holder_condition(self.holder, self.field.objective(), ScoreRange::Eq(0))
    }
}

fn validate_enum_encodings(
    schema: &str,
    field: &str,
    encodings: &[EnumEncoding],
) -> Result<(), EntityDiagnostic> {
    let mut variants = BTreeSet::new();
    let mut scores = BTreeMap::new();
    for encoding in encodings {
        if !variants.insert(encoding.name) {
            return Err(EntityDiagnostic::InvalidEnumEncoding {
                schema: schema.into(),
                field: field.into(),
                detail: format!("variant `{}` is declared twice", encoding.name),
            });
        }
        if let Some(previous) = scores.insert(encoding.score, encoding.name) {
            return Err(EntityDiagnostic::InvalidEnumEncoding {
                schema: schema.into(),
                field: field.into(),
                detail: format!(
                    "`{previous}` and `{}` both encode as {}",
                    encoding.name, encoding.score
                ),
            });
        }
    }
    Ok(())
}

fn mutation<F: EntityStateField>(
    field: F,
    holder: &str,
    track_dirty: bool,
    operation: &str,
    value: i32,
) -> Vec<String> {
    let objective = field.objective();
    let mut commands = vec![format!(
        "scoreboard players {operation} {holder} {objective} {value}"
    )];
    if let Some((min, max)) = field.descriptor().bounds {
        if min != i32::MIN {
            commands.push(format!(
                "execute if score {holder} {objective} matches ..{} run scoreboard players set {holder} {objective} {min}",
                min - 1
            ));
        }
        if max != i32::MAX {
            commands.push(format!(
                "execute if score {holder} {objective} matches {}.. run scoreboard players set {holder} {objective} {max}",
                max + 1
            ));
        }
    }
    if track_dirty {
        commands.push(format!(
            "scoreboard players set {holder} {} 1",
            field.dirty_objective()
        ));
    }
    commands
}

fn encode_fixed(value: f64, scale: i32, bounds: Option<(i32, i32)>) -> i32 {
    let scaled = value * f64::from(scale);
    let encoded = if scaled.is_nan() {
        0
    } else if scaled >= f64::from(i32::MAX) {
        i32::MAX
    } else if scaled <= f64::from(i32::MIN) {
        i32::MIN
    } else {
        scaled.round() as i32
    };
    bounds.map_or(encoded, |(min, max)| encoded.clamp(min, max))
}

fn holder_condition(holder: &str, objective: String, range: ScoreRange) -> Condition {
    Condition::score(holder.to_string(), objective, range)
}

pub(crate) fn objective_name(namespace: &str, schema: &str, field: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{namespace}:{schema}.{field}"))
        .as_str()
        .to_string()
}

pub(crate) fn dirty_name(namespace: &str, schema: &str, field: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{namespace}:{schema}.{field}.dirty"))
        .as_str()
        .to_string()
}

fn exact_predicate(objective: String, value: i32) -> StatePredicate {
    StatePredicate {
        objective,
        selector_range: SelectorScoreRange::exact(value),
        condition_range: ScoreRange::Eq(value),
    }
}

fn predicate_for_range(
    objective: String,
    schema: String,
    field: &str,
    range: impl RangeBounds<i32>,
) -> Result<StatePredicate, EntityDiagnostic> {
    let min = match range.start_bound() {
        Bound::Included(value) => Some(*value),
        Bound::Excluded(value) => value.checked_add(1),
        Bound::Unbounded => None,
    };
    let max = match range.end_bound() {
        Bound::Included(value) => Some(*value),
        Bound::Excluded(value) => value.checked_sub(1),
        Bound::Unbounded => None,
    };
    if matches!(range.start_bound(), Bound::Excluded(&i32::MAX))
        || matches!(range.end_bound(), Bound::Excluded(&i32::MIN))
        || matches!((min, max), (Some(min), Some(max)) if min > max)
    {
        return Err(EntityDiagnostic::InvalidRange {
            schema,
            field: field.into(),
            range: format!("{min:?}..={max:?}"),
        });
    }
    let selector_range = match (min, max) {
        (Some(min), Some(max)) => SelectorScoreRange::between(min, max),
        (Some(min), None) => SelectorScoreRange::at_least(min),
        (None, Some(max)) => SelectorScoreRange::at_most(max),
        (None, None) => SelectorScoreRange::between(i32::MIN, i32::MAX),
    };
    let condition_range = match (min, max) {
        (Some(min), Some(max)) if min == max => ScoreRange::Eq(min),
        (min, max) => ScoreRange::Between(min, max),
    };
    Ok(StatePredicate {
        objective,
        selector_range,
        condition_range,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Phase {
        Calm,
        Enraged,
    }

    impl EntityEnumValue for Phase {
        const ENCODINGS: &'static [EnumEncoding] = &[
            EnumEncoding {
                name: "Calm",
                score: 0,
            },
            EnumEncoding {
                name: "Enraged",
                score: 2,
            },
        ];
        fn encode(self) -> i32 {
            match self {
                Self::Calm => 0,
                Self::Enraged => 2,
            }
        }
    }

    #[test]
    fn names_are_stable_bounded_and_separate_from_dirty() {
        let field = EntityScore::<i32>::new("rpg", "mob", "maximum_health", 20, None);
        assert_eq!(field.objective(), field.objective());
        assert!(field.objective().len() <= 16);
        assert_ne!(field.objective(), field.dirty_objective());
    }

    #[test]
    fn write_marks_only_the_source_dirty() {
        let field = EntityScore::<i32>::new("rpg", "mob", "level", 1, Some((1, 100)));
        let commands = field.bind().set(10);
        assert_eq!(
            commands,
            vec![
                format!("scoreboard players set @s {} 10", field.objective()),
                format!(
                    "execute if score @s {} matches ..0 run scoreboard players set @s {} 1",
                    field.objective(),
                    field.objective()
                ),
                format!(
                    "execute if score @s {} matches 101.. run scoreboard players set @s {} 100",
                    field.objective(),
                    field.objective()
                ),
                format!("scoreboard players set @s {} 1", field.dirty_objective()),
            ]
        );
    }

    #[test]
    fn fixed_scores_round_saturate_and_keep_their_scale() {
        let field = FixedScore::__new("rpg", "mob", "speed", 10, 13, Some((-20, 20)));
        let bound = field.bind();
        assert_eq!(field.scale(), 10);
        assert_eq!(bound.get().scale(), 10);
        assert!(bound.get().command().contains(&field.objective()));
        assert_eq!(
            bound.set(1.25)[0],
            format!("scoreboard players set @s {} 13", field.objective())
        );
        assert_eq!(
            bound.set(-1.25)[0],
            format!("scoreboard players set @s {} -13", field.objective())
        );
        assert_eq!(
            bound.set(f64::INFINITY)[0],
            format!("scoreboard players set @s {} 20", field.objective())
        );
    }

    #[test]
    fn keyed_data_uses_uuid_macro_storage_without_entity_nbt_writes() {
        let field = KeyedData::<i32>::new("rpg:state", "components.\"stats\".xp");
        let commands = field.set(7);
        assert_eq!(commands.len(), 5);
        for (index, command) in commands[..4].iter().enumerate() {
            assert_eq!(
                command,
                &format!(
                    "data modify storage rpg:state __sand_owner.u{index} set from entity @s UUID[{index}]"
                )
            );
        }
        assert!(commands[4].starts_with("function __sand_local:sand/state_data/keyed/"));
        assert!(commands[4].ends_with(" with storage rpg:state __sand_owner"));
        assert!(
            commands
                .iter()
                .all(|command| !command.contains("data modify entity"))
        );
        let helpers = crate::function::drain_dyn_fns();
        assert_eq!(helpers.len(), 1);
        assert!(helpers[0].1[0].contains("owners[{uuid:[I;$(u0),$(u1),$(u2),$(u3)]}]"));
    }

    #[test]
    fn enum_predicate_uses_declared_encoding() {
        let field = EntityEnum::new("rpg", "mob", "phase", Phase::Calm.encode());
        assert_eq!(field.is(Phase::Enraged).selector_range.to_string(), "2");
    }

    #[test]
    fn inverted_range_is_structured_error() {
        let field = EntityScore::<i32>::new("rpg", "mob", "level", 1, None);
        assert_eq!(
            field
                .matches(std::ops::RangeInclusive::new(20, 10))
                .unwrap_err()
                .code(),
            "SAND-ENTITY-RANGE"
        );
    }

    #[test]
    fn duplicate_enum_scores_are_rejected() {
        static BAD: &[EnumEncoding] = &[
            EnumEncoding {
                name: "A",
                score: 1,
            },
            EnumEncoding {
                name: "B",
                score: 1,
            },
        ];
        static FIELDS: &[StateFieldDescriptor] = &[StateFieldDescriptor::new(
            "phase",
            StateFieldKind::Enum(BAD),
            1,
            None,
        )];
        let error = StateSchema {
            namespace: "rpg",
            name: "bad",
            version: 1,
            fields: FIELDS,
        }
        .validate()
        .unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-ENUM");
    }
}
