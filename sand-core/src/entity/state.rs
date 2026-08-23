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

/// One enum variant's stable scoreboard encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumEncoding {
    /// Rust-facing variant name used by diagnostics.
    pub name: &'static str,
    /// Integer stored in the scoreboard.
    pub score: i32,
}

/// A finite typed value stored by [`EntityEnum`].
///
/// `#[derive(EntityStateEnum)]` is the normal implementation path. Manual
/// implementations remain supported for established wire formats.
pub trait EntityEnumValue: Copy + Eq + fmt::Debug + 'static {
    /// Complete stable encoding table.
    const ENCODINGS: &'static [EnumEncoding];

    /// Convert a value to its declared score.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEnumValue::encode` for the canonical contract."]
    fn encode(self) -> i32;
}

/// Persistence and runtime behavior of a state field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StateFieldKind {
    /// Signed integer or fixed-point score.
    Score,
    /// Boolean encoded as zero or one.
    Flag,
    /// Finite enum encoded without string matching.
    Enum(&'static [EnumEncoding]),
    /// Elapsed/countdown timer.
    Timer,
    /// Reusable countdown that is ready at zero.
    Cooldown,
    /// Schema or archetype version.
    Version,
    /// Generated source/output dirty bit.
    Dirty,
}

/// Static metadata for one schema field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateFieldDescriptor {
    /// Rust-facing field name.
    pub name: &'static str,
    /// Storage/behavior family.
    pub kind: StateFieldKind,
    /// Initial score assigned only when the value is missing.
    pub default: i32,
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

/// Complete metadata for a typed state schema.
#[derive(Debug, Clone, Copy)]
pub struct StateSchema {
    /// Namespace used in generated logical names.
    pub namespace: &'static str,
    /// Schema name within the namespace.
    pub name: &'static str,
    /// Current version; zero is reserved for an uninitialized entity.
    pub version: u32,
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

/// A type-level State schema.
///
/// The derive macro generates this implementation and associated typed field
/// constants. Manual implementations must return stable immutable metadata.
pub trait EntityState: 'static {
    /// Return this schema's metadata.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityState::schema` for the canonical contract."]
    fn schema() -> StateSchema;
}

/// Shared behavior implemented by all typed state field handles.
pub trait EntityStateField: Copy + 'static {
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

/// Typed signed entity score.
#[derive(Debug, PartialEq, Eq)]
pub struct EntityScore<T = i32> {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    _marker: PhantomData<fn() -> T>,
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

/// Typed finite entity enum.
#[derive(Debug, PartialEq, Eq)]
pub struct EntityEnum<T: EntityEnumValue> {
    namespace: &'static str,
    schema: &'static str,
    descriptor: StateFieldDescriptor,
    _marker: PhantomData<fn() -> T>,
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
