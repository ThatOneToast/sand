//! Typed declarations for synchronizing entity state with native properties.
//!
//! This module describes ownership and refresh behavior without emitting
//! commands. An archetype/exporter validates these declarations, then performs
//! them while the affected entity is the current executor (`@s`). Declarations
//! are therefore reusable configuration, not durable entity references.
//!
//! Normal APIs use Sand's registry IDs, state fields, item stacks, durations,
//! colors, and function references. [`RawEntityProperty`] and
//! [`RawEntityStateField`] are deliberately explicit escape hatches for
//! unsupported or modded data.

use std::fmt;

use sand_commands::{ChatColor, TextComponent};
use sand_components::{
    AttributeOperation, AttributeType, EquipmentSlot, ItemStack, StatusEffectId, Ticks,
};

use crate::entity::diagnostic::EntityDiagnostic;
use crate::entity::kind::{EntityKind, PlayerKind};
use crate::entity::state::{
    EntityEnum, EntityEnumValue, EntityFlag, EntityScore, EntityStateField, StateFieldReference,
};
use crate::resource_ref::FunctionId;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityEventId",
    aliases = ["sand::prelude::EntityEventId"],
    module = "sand::entity",
    summary = "A typed identifier for an event that requests a property refresh.",
    context = "A typed identifier for an event that requests a property refresh. Event identifiers are resource locations so independently authored packs cannot accidentally share an unqualified event name.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityEventId;",
)]
/// A typed identifier for an event that requests a property refresh.
///
/// Event identifiers are resource locations so independently authored packs
/// cannot accidentally share an unqualified event name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityEventId(crate::ResourceLocation);

impl EntityEventId {
    /// Construct a namespaced event identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEventId::new",
        aliases = ["sand::prelude::EntityEventId::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a namespaced event identifier.",
        context = "Construct a namespaced event identifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(location = "`location` provides the typed resource identifier or location used to construct a namespaced event identifier."),
        returns = "An `EntityEventId` representing a namespaced event identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let entity_event_id = sand::entity::EntityEventId::new(location);\n}",
    )]
    pub fn new(location: crate::ResourceLocation) -> Self {
        Self(location)
    }

    /// Return the underlying resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityEventId::location",
        aliases = ["sand::prelude::EntityEventId::location"],
        module = "sand::entity",
        kind = "method",
        summary = "Return the underlying resource location.",
        context = "Return the underlying resource location. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Return the underlying resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_event_id_value: &sand::entity::EntityEventId)  {\n    let location = entity_event_id_value.location();\n}",
    )]
    #[must_use]
    pub fn location(&self) -> &crate::ResourceLocation {
        &self.0
    }
}

impl fmt::Display for EntityEventId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RefreshPolicy",
    aliases = ["sand::prelude::RefreshPolicy"],
    module = "sand::entity",
    summary = "When an archetype observes or materializes a native property.",
    context = "When an archetype observes or materializes a native property. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::RefreshPolicy;",
    variants(Every = "Refresh loaded matching entities at this interval.", Initialize = "Run once, after state has been provisioned and before initialization is marked complete.", Manual = "Generate no automatic scheduling; user code explicitly requests work.", OnEvent = "Refresh when Sand dispatches the named typed event.", OnFunction = "Refresh when the canonical datapack function is dispatched.", WhenSourceChanges = "Refresh only after one of the declaration's source fields changes."),
    variant_fields(Every = ["Refresh loaded matching entities at this interval."], OnEvent = ["Refresh when Sand dispatches the named typed event."], OnFunction = ["Refresh when the canonical datapack function is dispatched."]),
)]
/// When an archetype observes or materializes a native property.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RefreshPolicy {
    /// Run once, after state has been provisioned and before initialization is
    /// marked complete.
    Initialize,
    /// Refresh only after one of the declaration's source fields changes.
    WhenSourceChanges,
    /// Refresh loaded matching entities at this interval.
    ///
    /// Intervals are measured in game ticks. [`Self::validate`] rejects zero;
    /// periodic work does not run while an entity's chunk is unloaded.
    Every(#[doc = "Refresh loaded matching entities at this interval."] Ticks),
    /// Refresh when the canonical datapack function is dispatched.
    OnFunction(#[doc = "Refresh when the canonical datapack function is dispatched."] FunctionId),
    /// Refresh when Sand dispatches the named typed event.
    OnEvent(#[doc = "Refresh when Sand dispatches the named typed event."] EntityEventId),
    /// Generate no automatic scheduling; user code explicitly requests work.
    Manual,
}

impl RefreshPolicy {
    /// Validate scheduling invariants with archetype/property context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RefreshPolicy::validate",
        aliases = ["sand::prelude::RefreshPolicy::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate scheduling invariants with archetype/property context.",
        context = "Validate scheduling invariants with archetype/property context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate scheduling invariants with archetype/property context.", property = "`property` is the property checked when validating scheduling invariants with archetype/property context."),
        returns = "On success, the value produced to validate scheduling invariants with archetype/property context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(refresh_policy_value: &sand::entity::RefreshPolicy, archetype: impl fmt::Display, property: impl fmt::Display)  {\n    let validate = refresh_policy_value.validate(archetype, property);\n}",
    )]
    pub fn validate(
        &self,
        archetype: impl fmt::Display,
        property: impl fmt::Display,
    ) -> Result<(), EntityDiagnostic> {
        if matches!(self, Self::Every(ticks) if ticks.get() == 0) {
            return Err(EntityDiagnostic::InvalidRefreshInterval {
                archetype: archetype.to_string(),
                property: property.to_string(),
            });
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::OwnershipPolicy",
    aliases = ["sand::prelude::OwnershipPolicy"],
    module = "sand::entity",
    summary = "How reconciliation treats a property declared by an archetype.",
    context = "How reconciliation treats a property declared by an archetype. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::OwnershipPolicy;",
    variants(Exact = "Keep the declared value exact whenever the binding refreshes.", InitializeMissing = "Write a missing value during first initialization, then leave it alone.", Observe = "Read the native value into state without writing the native property.", Preserve = "Preserve the runtime/external value and never claim write ownership.", ReconcileWhenDirty = "Reconcile only after a dependency marks this output dirty."),
)]
/// How reconciliation treats a property declared by an archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OwnershipPolicy {
    /// Write a missing value during first initialization, then leave it alone.
    InitializeMissing,
    /// Preserve the runtime/external value and never claim write ownership.
    Preserve,
    /// Keep the declared value exact whenever the binding refreshes.
    Exact,
    /// Read the native value into state without writing the native property.
    Observe,
    /// Reconcile only after a dependency marks this output dirty.
    ReconcileWhenDirty,
}

impl OwnershipPolicy {
    /// Whether this policy claims write ownership for conflict detection.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::OwnershipPolicy::claims_write_ownership",
        aliases = ["sand::prelude::OwnershipPolicy::claims_write_ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Whether this policy claims write ownership for conflict detection.",
        context = "Whether this policy claims write ownership for conflict detection. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "`true` when the documented condition holds to determine whether this policy claims write ownership for conflict detection; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ownership_policy_value: sand::entity::OwnershipPolicy)  {\n    let is_claims_write_ownership = ownership_policy_value.claims_write_ownership();\n}",
    )]
    #[must_use]
    pub const fn claims_write_ownership(self) -> bool {
        matches!(
            self,
            Self::InitializeMissing | Self::Exact | Self::ReconcileWhenDirty
        )
    }

    /// Whether this policy reads native runtime state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::OwnershipPolicy::observes_native_state",
        aliases = ["sand::prelude::OwnershipPolicy::observes_native_state"],
        module = "sand::entity",
        kind = "method",
        summary = "Whether this policy reads native runtime state.",
        context = "Whether this policy reads native runtime state. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "`true` when the documented condition holds to determine whether this policy reads native runtime state; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ownership_policy_value: sand::entity::OwnershipPolicy)  {\n    let is_observes_native_state = ownership_policy_value.observes_native_state();\n}",
    )]
    #[must_use]
    pub const fn observes_native_state(self) -> bool {
        matches!(self, Self::Observe)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::HealthResizePolicy",
    aliases = ["sand::prelude::HealthResizePolicy"],
    module = "sand::entity",
    summary = "Current-health behavior when maximum health changes.",
    context = "Current-health behavior when maximum health changes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::HealthResizePolicy;",
    variants(PreserveAbsolute = "Keep the current absolute health, clamping only when the new maximum is lower.", PreserveRatio = "Preserve `current / old_max` using the binding's fixed-point ratio scratch score, then clamp to the new maximum.", Refill = "Set current health to the new maximum."),
)]
/// Current-health behavior when maximum health changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HealthResizePolicy {
    /// Keep the current absolute health, clamping only when the new maximum is
    /// lower.
    PreserveAbsolute,
    /// Preserve `current / old_max` using the binding's fixed-point ratio
    /// scratch score, then clamp to the new maximum.
    PreserveRatio,
    /// Set current health to the new maximum.
    Refill,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::CurrentHealthSync",
    aliases = ["sand::prelude::CurrentHealthSync"],
    module = "sand::entity",
    summary = "Direction in which current health is synchronized.",
    context = "Direction in which current health is synchronized. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::CurrentHealthSync;",
    variants(ApplyState = "Materialize the configured state score into native `Health`.", Bidirectional = "Observe native changes and apply explicit dirty state changes.", None = "Do not synchronize native current health.", ObserveNative = "Read native `Health` into the configured state score."),
)]
/// Direction in which current health is synchronized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CurrentHealthSync {
    /// Do not synchronize native current health.
    None,
    /// Read native `Health` into the configured state score.
    ObserveNative,
    /// Materialize the configured state score into native `Health`.
    ApplyState,
    /// Observe native changes and apply explicit dirty state changes.
    Bidirectional,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::HealthBinding",
    aliases = ["sand::prelude::HealthBinding"],
    module = "sand::entity",
    summary = "A max/current-health binding for a mutable living entity.",
    context = "A max/current-health binding for a mutable living entity. The maximum score is lowered to `minecraft:max_health`. Current health is optional when only max-health scaling is required. Reconciliation must not heal unless [`HealthResizePolicy::Refill`] was selected.",
    minecraft = "The maximum score is lowered to `minecraft:max_health`. Current health is optional when only max-health scaling is required. Reconciliation must not heal unless [`HealthResizePolicy::Refill`] was selected.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::HealthBinding;",
)]
/// A max/current-health binding for a mutable living entity.
///
/// The maximum score is lowered to `minecraft:max_health`. Current health is
/// optional when only max-health scaling is required. Reconciliation must not
/// heal unless [`HealthResizePolicy::Refill`] was selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthBinding {
    max_health: EntityScore<i32>,
    current_health: Option<EntityScore<i32>>,
    resize: HealthResizePolicy,
    current_sync: CurrentHealthSync,
    observation_interval: Option<Ticks>,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicyRef,
}

impl HealthBinding {
    /// Bind a max-health state score with safe absolute-health preservation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::new",
        aliases = ["sand::prelude::HealthBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind a max-health state score with safe absolute-health preservation.",
        context = "Bind a max-health state score with safe absolute-health preservation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(max_health = "`max_health` provides the max health used when binding a max-health state score with safe absolute-health preservation."),
        returns = "A `HealthBinding` binding a max-health state score with safe absolute-health preservation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(max_health: sand::entity::EntityScore < i32 >)  {\n    let health_binding = sand::entity::HealthBinding::new(max_health);\n}",
    )]
    #[must_use]
    pub fn new(max_health: EntityScore<i32>) -> Self {
        Self {
            max_health,
            current_health: None,
            resize: HealthResizePolicy::PreserveAbsolute,
            current_sync: CurrentHealthSync::None,
            observation_interval: None,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicyRef::WhenSourceChanges,
        }
    }

    /// Bind a State score to the entity's native current health.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::current_health",
        aliases = ["sand::prelude::HealthBinding::current_health"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind a State score to the entity's native current health.",
        context = "The definition-owned score remains the typed gameplay-state handle; this binding tells an EntityArchetype how that score and the living entity's native Health NBT value relate.",
        minecraft = "ObserveNative reads native Health into the score, ApplyState writes dirty score changes to native Health, and Bidirectional does both. Source-dirty reconciliation happens when State changes; observe_native_every adds bounded periodic observation for native changes that Sand did not initiate.",
        use_when = ["Gameplay logic needs typed reads or writes of a living entity's current health", "Native damage or healing must be reconciled with a State field"],
        avoid_when = ["Only max-health scaling is required", "Another system is the authoritative owner of native Health synchronization"],
        params(field = "The definition-owned i32 State field that stores current health in health points.", sync = "The direction in which native Health and the State field are synchronized."),
        returns = "This health binding configured with the current-health field and synchronization policy.",
        example = "use sand::prelude::*;\n\nlet binding = HealthBinding::new(DamageableEntity::max_health)\n    .current_health(DamageableEntity::health, CurrentHealthSync::Bidirectional)\n    .observe_native_every(Ticks::new(1));",
    )]
    #[must_use]
    pub fn current_health(mut self, field: EntityScore<i32>, sync: CurrentHealthSync) -> Self {
        self.current_health = Some(field);
        self.current_sync = sync;
        self
    }

    /// Observe native current health at a bounded cadence in addition to
    /// source-dirty refreshes.
    ///
    /// Observation pauses while the entity is unloaded. Zero is rejected
    /// during archetype validation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::observe_native_every",
        aliases = ["sand::prelude::HealthBinding::observe_native_every"],
        module = "sand::entity",
        kind = "method",
        summary = "Observe native current health at a bounded cadence in addition to source-dirty refreshes.",
        context = "Observe native current health at a bounded cadence in addition to source-dirty refreshes. Observation pauses while the entity is unloaded. Zero is rejected during archetype validation.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(interval = "`interval` provides the interval observed when tracking native current health at a bounded cadence in addition to source-dirty refreshes."),
        returns = "The `HealthBinding` value with the documented change applied to observe native current health at a bounded cadence in addition to source-dirty refreshes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: sand::entity::HealthBinding, interval: sand::state::Ticks)  {\n    let updated_health_binding = health_binding_value.observe_native_every(interval);\n}",
    )]
    #[must_use]
    pub fn observe_native_every(mut self, interval: Ticks) -> Self {
        self.observation_interval = Some(interval);
        self
    }

    /// Select behavior when max health changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::resize",
        aliases = ["sand::prelude::HealthBinding::resize"],
        module = "sand::entity",
        kind = "method",
        summary = "Select behavior when max health changes.",
        context = "Select behavior when max health changes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy used when selecting behavior when max health changes."),
        returns = "The `HealthBinding` value with the documented change applied to select behavior when max health changes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: sand::entity::HealthBinding, policy: sand::entity::HealthResizePolicy)  {\n    let updated_health_binding = health_binding_value.resize(policy);\n}",
    )]
    #[must_use]
    pub fn resize(mut self, policy: HealthResizePolicy) -> Self {
        self.resize = policy;
        self
    }

    /// Select ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::ownership",
        aliases = ["sand::prelude::HealthBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Select ownership behavior.",
        context = "Select ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy used when selecting ownership behavior."),
        returns = "The `HealthBinding` value with the documented change applied to select ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: sand::entity::HealthBinding, policy: sand::entity::OwnershipPolicy)  {\n    let updated_health_binding = health_binding_value.ownership(policy);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, policy: OwnershipPolicy) -> Self {
        self.ownership = policy;
        self
    }

    /// Select automatic refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::refresh",
        aliases = ["sand::prelude::HealthBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Select automatic refresh scheduling.",
        context = "Select automatic refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy used when selecting automatic refresh scheduling."),
        returns = "The `HealthBinding` value with the documented change applied to select automatic refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: sand::entity::HealthBinding, policy: sand::entity::RefreshPolicy)  {\n    let updated_health_binding = health_binding_value.refresh(policy);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.refresh = policy.into();
        self
    }

    /// Max-health state field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::max_health_field",
        aliases = ["sand::prelude::HealthBinding::max_health_field"],
        module = "sand::entity",
        kind = "method",
        summary = "Max-health state field.",
        context = "Max-health state field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityScore < i32 >` value produced to max-health state field.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let max_health_field = health_binding_value.max_health_field();\n}",
    )]
    #[must_use]
    pub const fn max_health_field(&self) -> EntityScore<i32> {
        self.max_health
    }

    /// Optional current-health state field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::current_health_field",
        aliases = ["sand::prelude::HealthBinding::current_health_field"],
        module = "sand::entity",
        kind = "method",
        summary = "Optional current-health state field.",
        context = "Optional current-health state field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The matching value used to use optional current-health state field, or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let current_health_field = health_binding_value.current_health_field();\n}",
    )]
    #[must_use]
    pub const fn current_health_field(&self) -> Option<EntityScore<i32>> {
        self.current_health
    }

    /// Selected resize behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::resize_policy",
        aliases = ["sand::prelude::HealthBinding::resize_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected resize behavior.",
        context = "Selected resize behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `HealthResizePolicy` value produced to use selected resize behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let resize_policy = health_binding_value.resize_policy();\n}",
    )]
    #[must_use]
    pub const fn resize_policy(&self) -> HealthResizePolicy {
        self.resize
    }

    /// Selected current-health direction.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::current_health_sync",
        aliases = ["sand::prelude::HealthBinding::current_health_sync"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected current-health direction.",
        context = "Selected current-health direction. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `CurrentHealthSync` value produced to use selected current-health direction.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let current_health_sync = health_binding_value.current_health_sync();\n}",
    )]
    #[must_use]
    pub const fn current_health_sync(&self) -> CurrentHealthSync {
        self.current_sync
    }

    /// Optional native-health observation cadence.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::observation_interval",
        aliases = ["sand::prelude::HealthBinding::observation_interval"],
        module = "sand::entity",
        kind = "method",
        summary = "Optional native-health observation cadence.",
        context = "Optional native-health observation cadence. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The matching value used to use optional native-health observation cadence, or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let observation_interval = health_binding_value.observation_interval();\n}",
    )]
    #[must_use]
    pub const fn observation_interval(&self) -> Option<Ticks> {
        self.observation_interval
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::ownership_policy",
        aliases = ["sand::prelude::HealthBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let ownership_policy = health_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::refresh_policy",
        aliases = ["sand::prelude::HealthBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh behavior.",
        context = "Selected refresh behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RefreshPolicy` value produced to use selected refresh behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding)  {\n    let refresh_policy = health_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> RefreshPolicy {
        self.refresh.clone().into()
    }

    /// Validate combinations that cannot preserve their documented semantics.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::HealthBinding::validate",
        aliases = ["sand::prelude::HealthBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate combinations that cannot preserve their documented semantics.",
        context = "Validate combinations that cannot preserve their documented semantics. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate combinations that cannot preserve their documented semantics."),
        returns = "On success, the value produced to validate combinations that cannot preserve their documented semantics; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(health_binding_value: &sand::entity::HealthBinding, archetype: impl fmt::Display)  {\n    let validate = health_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        let archetype = archetype.to_string();
        self.refresh_policy()
            .validate(&archetype, NativePropertyKey::Health)?;
        if self.current_sync != CurrentHealthSync::None && self.current_health.is_none() {
            return Err(EntityDiagnostic::InvalidHealthResizePolicy {
                archetype,
                policy: format!("{:?}", self.resize),
                detail: "current-health synchronization requires a current-health state field"
                    .into(),
            });
        }
        if self
            .observation_interval
            .is_some_and(|interval| interval.get() == 0)
        {
            return Err(EntityDiagnostic::InvalidRefreshInterval {
                archetype,
                property: "health observation".into(),
            });
        }
        if self.resize == HealthResizePolicy::Refill
            && (self.observation_interval.is_some()
                || matches!(self.refresh_policy(), RefreshPolicy::Every(_)))
        {
            return Err(EntityDiagnostic::InvalidHealthResizePolicy {
                archetype,
                policy: "Refill".into(),
                detail: "periodic observation would heal the entity on every interval; use a source-change or initialization refresh".into(),
            });
        }
        Ok(())
    }
}

// Keeps bindings Eq even though FunctionId-backed policies are owned values.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RefreshPolicyRef {
    Initialize,
    WhenSourceChanges,
    Every(Ticks),
    OnFunction(FunctionId),
    OnEvent(EntityEventId),
    Manual,
}

impl From<RefreshPolicy> for RefreshPolicyRef {
    fn from(value: RefreshPolicy) -> Self {
        match value {
            RefreshPolicy::Initialize => Self::Initialize,
            RefreshPolicy::WhenSourceChanges => Self::WhenSourceChanges,
            RefreshPolicy::Every(v) => Self::Every(v),
            RefreshPolicy::OnFunction(v) => Self::OnFunction(v),
            RefreshPolicy::OnEvent(v) => Self::OnEvent(v),
            RefreshPolicy::Manual => Self::Manual,
        }
    }
}

impl From<RefreshPolicyRef> for RefreshPolicy {
    fn from(value: RefreshPolicyRef) -> Self {
        match value {
            RefreshPolicyRef::Initialize => Self::Initialize,
            RefreshPolicyRef::WhenSourceChanges => Self::WhenSourceChanges,
            RefreshPolicyRef::Every(v) => Self::Every(v),
            RefreshPolicyRef::OnFunction(v) => Self::OnFunction(v),
            RefreshPolicyRef::OnEvent(v) => Self::OnEvent(v),
            RefreshPolicyRef::Manual => Self::Manual,
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::NumericPropertySource",
    aliases = ["sand::prelude::NumericPropertySource"],
    module = "sand::entity",
    summary = "A typed numeric source for an attribute or effect parameter.",
    context = "A typed numeric source for an attribute or effect parameter. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::NumericPropertySource;",
    variants(Fixed = "A constant fixed-point value: `units / scale`.", StateScore = "The value of a typed entity score."),
    variant_fields(Fixed(scale = "`scale` provides the particle scale when a constant fixed-point value: `units / scale`.", units = "`units` provides the units when a constant fixed-point value: `units / scale`."), StateScore(dirty_objective = "Hidden source-dirty objective marked by typed mutations.", field = "Owning State component and field metadata retained for archetype membership validation.", objective = "Generated score objective.", scale = "Number of stored scoreboard units per logical value.")),
)]
/// A typed numeric source for an attribute or effect parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NumericPropertySource {
    /// A constant fixed-point value: `units / scale`.
    Fixed {
        #[doc = "`units` provides the units when a constant fixed-point value: `units / scale`."]
        units: i64,
        #[doc = "`scale` provides the particle scale when a constant fixed-point value: `units / scale`."]
        scale: u32,
    },
    /// The value of a typed entity score.
    StateScore {
        /// Generated score objective.
        objective: String,
        /// Hidden source-dirty objective marked by typed mutations.
        dirty_objective: String,
        /// Owning component and field metadata used by archetype validation.
        field: StateFieldReference,
        /// Number of stored scoreboard units per logical value.
        scale: i32,
    },
}

impl NumericPropertySource {
    /// Construct a finite fixed-point constant.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NumericPropertySource::fixed",
        aliases = ["sand::prelude::NumericPropertySource::fixed"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a finite fixed-point constant.",
        context = "Construct a finite fixed-point constant. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(units = "`units` is used when constructing a finite fixed-point constant.", scale = "`scale` is used when constructing a finite fixed-point constant."),
        returns = "On success, the value produced to construct a finite fixed-point constant; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(units: i64, scale: u32)  {\n    let fixed = sand::entity::NumericPropertySource::fixed(units, scale);\n}",
    )]
    pub fn fixed(units: i64, scale: u32) -> Result<Self, EntityDiagnostic> {
        if scale == 0 {
            return Err(EntityDiagnostic::FixedPointOverflow {
                archetype: "<property-source>".into(),
                derivation: "<constant>".into(),
                detail: "fixed-point scale must be greater than zero".into(),
            });
        }
        Ok(Self::Fixed { units, scale })
    }

    /// Use a typed state score as the source.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NumericPropertySource::state",
        aliases = ["sand::prelude::NumericPropertySource::state"],
        module = "sand::entity",
        kind = "method",
        summary = "Use a typed state score as the source.",
        context = "Use a typed state score as the source. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Use a typed state score as the source."],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` sets the field for a typed state score as the source."),
        returns = "A `NumericPropertySource` configured for a typed state score as the source.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: impl sand::entity::NumericStateField)  {\n    let numeric_property_source = sand::entity::NumericPropertySource::state(field);\n}",
    )]
    #[must_use]
    pub fn state(field: impl super::NumericStateField) -> Self {
        Self::StateScore {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
            field: field.field_reference(),
            scale: field.numeric_scale(),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::AttributeBinding",
    aliases = ["sand::prelude::AttributeBinding"],
    module = "sand::entity",
    summary = "A typed native attribute binding. Attribute values are applied to the entity bound to `@s`; exporter capability checks must restrict this to entity kinds supporting attributes.",
    context = "A typed native attribute binding. Attribute values are applied to the entity bound to `@s`; exporter capability checks must restrict this to entity kinds supporting attributes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Attribute values are applied to the entity bound to `@s`; exporter capability checks must restrict this to entity kinds supporting attributes.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::AttributeBinding;",
)]
/// A typed native attribute binding.
///
/// Attribute values are applied to the entity bound to `@s`; exporter
/// capability checks must restrict this to entity kinds supporting attributes.
#[derive(Debug, Clone)]
pub struct AttributeBinding {
    attribute: AttributeType,
    source: NumericPropertySource,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::AttributeModifierBinding",
    aliases = ["sand::prelude::AttributeModifierBinding"],
    module = "sand::entity",
    summary = "An idempotent, namespaced native attribute modifier.",
    context = "An idempotent, namespaced native attribute modifier. Refresh removes the modifier ID before adding its current value, so the same entity never accumulates duplicates. Cleanup removes only this ID.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::AttributeModifierBinding;",
)]
/// An idempotent, namespaced native attribute modifier.
///
/// Refresh removes the modifier ID before adding its current value, so the
/// same entity never accumulates duplicates. Cleanup removes only this ID.
#[derive(Debug, Clone)]
pub struct AttributeModifierBinding {
    attribute: AttributeType,
    id: crate::ResourceLocation,
    source: NumericPropertySource,
    operation: AttributeOperation,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl AttributeModifierBinding {
    /// Create an exact, dirty-refreshed modifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::new",
        aliases = ["sand::prelude::AttributeModifierBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Create an exact, dirty-refreshed modifier.",
        context = "Create an exact, dirty-refreshed modifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(attribute = "`attribute` is used when creating an exact, dirty-refreshed modifier.", id = "`id` provides the typed resource identifier or location used to create an exact, dirty-refreshed modifier.", source = "`source` is used when creating an exact, dirty-refreshed modifier.", operation = "`operation` is used when creating an exact, dirty-refreshed modifier."),
        returns = "An `AttributeModifierBinding` representing an exact, dirty-refreshed modifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute: sand::component::AttributeType, id: sand::ResourceLocation, source: sand::entity::NumericPropertySource, operation: sand::component::AttributeOperation)  {\n    let attribute_modifier_binding = sand::entity::AttributeModifierBinding::new(attribute, id, source, operation);\n}",
    )]
    #[must_use]
    pub fn new(
        attribute: AttributeType,
        id: crate::ResourceLocation,
        source: NumericPropertySource,
        operation: AttributeOperation,
    ) -> Self {
        Self {
            attribute,
            id,
            source,
            operation,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicy::WhenSourceChanges,
        }
    }

    /// Select ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::ownership",
        aliases = ["sand::prelude::AttributeModifierBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Select ownership behavior.",
        context = "Select ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy used when selecting ownership behavior."),
        returns = "The `AttributeModifierBinding` value with the documented change applied to select ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: sand::entity::AttributeModifierBinding, policy: sand::entity::OwnershipPolicy)  {\n    let updated_attribute_modifier_binding = attribute_modifier_binding_value.ownership(policy);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, policy: OwnershipPolicy) -> Self {
        self.ownership = policy;
        self
    }

    /// Select refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::refresh",
        aliases = ["sand::prelude::AttributeModifierBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Select refresh scheduling.",
        context = "Select refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy used when selecting refresh scheduling."),
        returns = "The `AttributeModifierBinding` value with the documented change applied to select refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: sand::entity::AttributeModifierBinding, policy: sand::entity::RefreshPolicy)  {\n    let updated_attribute_modifier_binding = attribute_modifier_binding_value.refresh(policy);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.refresh = policy;
        self
    }

    /// Target attribute.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::attribute",
        aliases = ["sand::prelude::AttributeModifierBinding::attribute"],
        module = "sand::entity",
        kind = "method",
        summary = "Target attribute.",
        context = "Target attribute. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& AttributeType` value produced to target attribute.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding)  {\n    let attribute = attribute_modifier_binding_value.attribute();\n}",
    )]
    #[must_use]
    pub fn attribute(&self) -> &AttributeType {
        &self.attribute
    }

    /// Stable modifier resource ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::id",
        aliases = ["sand::prelude::AttributeModifierBinding::id"],
        module = "sand::entity",
        kind = "method",
        summary = "Stable modifier resource ID.",
        context = "Stable modifier resource ID. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& sand :: ResourceLocation` value produced to use stable modifier resource ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding)  {\n    let id = attribute_modifier_binding_value.id();\n}",
    )]
    #[must_use]
    pub fn id(&self) -> &crate::ResourceLocation {
        &self.id
    }

    /// Numeric modifier amount.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::source",
        aliases = ["sand::prelude::AttributeModifierBinding::source"],
        module = "sand::entity",
        kind = "method",
        summary = "Numeric modifier amount.",
        context = "Numeric modifier amount. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& NumericPropertySource` value produced to numeric modifier amount.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding)  {\n    let source = attribute_modifier_binding_value.source();\n}",
    )]
    #[must_use]
    pub fn source(&self) -> &NumericPropertySource {
        &self.source
    }

    /// Vanilla modifier operation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::operation",
        aliases = ["sand::prelude::AttributeModifierBinding::operation"],
        module = "sand::entity",
        kind = "method",
        summary = "Vanilla modifier operation.",
        context = "Vanilla modifier operation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `AttributeOperation` value produced to vanilla modifier operation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding)  {\n    let operation = attribute_modifier_binding_value.operation();\n}",
    )]
    #[must_use]
    pub const fn operation(&self) -> AttributeOperation {
        self.operation
    }

    /// Ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::ownership_policy",
        aliases = ["sand::prelude::AttributeModifierBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Ownership behavior.",
        context = "Ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding)  {\n    let ownership_policy = attribute_modifier_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::refresh_policy",
        aliases = ["sand::prelude::AttributeModifierBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Refresh scheduling.",
        context = "Refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding)  {\n    let refresh_policy = attribute_modifier_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Stable ownership key.
    #[must_use]
    pub(crate) fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::AttributeModifier {
            attribute: NativeAttributeKey::new(self.attribute.clone()),
            id: self.id.to_string(),
        }
    }

    /// Validate scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeModifierBinding::validate",
        aliases = ["sand::prelude::AttributeModifierBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate scheduling.",
        context = "Validate scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate scheduling."),
        returns = "On success, the value produced to validate scheduling; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(attribute_modifier_binding_value: &sand::entity::AttributeModifierBinding, archetype: impl fmt::Display)  {\n    let validate = attribute_modifier_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

impl AttributeBinding {
    /// Bind an attribute base value to a typed numeric source.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::new",
        aliases = ["sand::prelude::AttributeBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind an attribute base value to a typed numeric source.",
        context = "Bind an attribute base value to a typed numeric source. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(attribute = "`attribute` provides the attribute used when binding an attribute base value to a typed numeric source.", source = "`source` provides the source used when binding an attribute base value to a typed numeric source."),
        returns = "An `AttributeBinding` binding an attribute base value to a typed numeric source.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute: sand::component::AttributeType, source: sand::entity::NumericPropertySource)  {\n    let attribute_binding = sand::entity::AttributeBinding::new(attribute, source);\n}",
    )]
    #[must_use]
    pub fn new(attribute: AttributeType, source: NumericPropertySource) -> Self {
        Self {
            attribute,
            source,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicy::WhenSourceChanges,
        }
    }

    /// Set ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::ownership",
        aliases = ["sand::prelude::AttributeBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Set ownership behavior.",
        context = "Set ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy applied when setting ownership behavior."),
        returns = "The `AttributeBinding` value with the documented change applied to set ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: sand::entity::AttributeBinding, policy: sand::entity::OwnershipPolicy)  {\n    let updated_attribute_binding = attribute_binding_value.ownership(policy);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, policy: OwnershipPolicy) -> Self {
        self.ownership = policy;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::refresh",
        aliases = ["sand::prelude::AttributeBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(policy = "`policy` provides the policy applied when setting refresh scheduling."),
        returns = "The `AttributeBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: sand::entity::AttributeBinding, policy: sand::entity::RefreshPolicy)  {\n    let updated_attribute_binding = attribute_binding_value.refresh(policy);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.refresh = policy;
        self
    }

    /// Native attribute identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::attribute",
        aliases = ["sand::prelude::AttributeBinding::attribute"],
        module = "sand::entity",
        kind = "method",
        summary = "Native attribute identifier.",
        context = "Native attribute identifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& AttributeType` value produced to native attribute identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: &sand::entity::AttributeBinding)  {\n    let attribute = attribute_binding_value.attribute();\n}",
    )]
    #[must_use]
    pub fn attribute(&self) -> &AttributeType {
        &self.attribute
    }

    /// Numeric source.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::source",
        aliases = ["sand::prelude::AttributeBinding::source"],
        module = "sand::entity",
        kind = "method",
        summary = "Numeric source.",
        context = "Numeric source. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& NumericPropertySource` value produced to numeric source.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: &sand::entity::AttributeBinding)  {\n    let source = attribute_binding_value.source();\n}",
    )]
    #[must_use]
    pub fn source(&self) -> &NumericPropertySource {
        &self.source
    }

    /// Ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::ownership_policy",
        aliases = ["sand::prelude::AttributeBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Ownership behavior.",
        context = "Ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: &sand::entity::AttributeBinding)  {\n    let ownership_policy = attribute_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::refresh_policy",
        aliases = ["sand::prelude::AttributeBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Refresh scheduling.",
        context = "Refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: &sand::entity::AttributeBinding)  {\n    let refresh_policy = attribute_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Stable key used for ownership-conflict detection.
    #[must_use]
    pub(crate) fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::Attribute(NativeAttributeKey::new(self.attribute.clone()))
    }

    /// Validate refresh scheduling with archetype context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::AttributeBinding::validate",
        aliases = ["sand::prelude::AttributeBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate refresh scheduling with archetype context.",
        context = "Validate refresh scheduling with archetype context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate refresh scheduling with archetype context."),
        returns = "On success, the value produced to validate refresh scheduling with archetype context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(attribute_binding_value: &sand::entity::AttributeBinding, archetype: impl fmt::Display)  {\n    let validate = attribute_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EffectBinding",
    aliases = ["sand::prelude::EffectBinding"],
    module = "sand::entity",
    summary = "A typed status-effect binding. Reconciliation compares the effect ID, amplifier, and duration policy so a stable effect is not needlessly re-applied every tick.",
    context = "A typed status-effect binding. Reconciliation compares the effect ID, amplifier, and duration policy so a stable effect is not needlessly re-applied every tick. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EffectBinding;",
)]
/// A typed status-effect binding.
///
/// Reconciliation compares the effect ID, amplifier, and duration policy so a
/// stable effect is not needlessly re-applied every tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBinding {
    effect: StatusEffectId,
    duration: Ticks,
    amplifier: u8,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl EffectBinding {
    /// Create an effect binding with amplifier zero.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::new",
        aliases = ["sand::prelude::EffectBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Create an effect binding with amplifier zero.",
        context = "Create an effect binding with amplifier zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(effect = "`effect` provides the typed Minecraft resource identifier used to create an effect binding with amplifier zero.", duration = "`duration` provides the Minecraft tick duration used to create an effect binding with amplifier zero."),
        returns = "An `EffectBinding` representing an effect binding with amplifier zero.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect: sand::registry::StatusEffectId, duration: sand::state::Ticks)  {\n    let effect_binding = sand::entity::EffectBinding::new(effect, duration);\n}",
    )]
    #[must_use]
    pub fn new(effect: StatusEffectId, duration: Ticks) -> Self {
        Self {
            effect,
            duration,
            amplifier: 0,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicy::WhenSourceChanges,
        }
    }

    /// Set the zero-based Minecraft effect amplifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::amplifier",
        aliases = ["sand::prelude::EffectBinding::amplifier"],
        module = "sand::entity",
        kind = "method",
        summary = "Set the zero-based Minecraft effect amplifier.",
        context = "Set the zero-based Minecraft effect amplifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(amplifier = "`amplifier` provides the amplifier applied when setting the zero-based Minecraft effect amplifier."),
        returns = "The `EffectBinding` value with the documented change applied to set the zero-based Minecraft effect amplifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: sand::entity::EffectBinding, amplifier: u8)  {\n    let updated_effect_binding = effect_binding_value.amplifier(amplifier);\n}",
    )]
    #[must_use]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    /// Set ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::ownership",
        aliases = ["sand::prelude::EffectBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Set ownership behavior.",
        context = "Set ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ownership = "`ownership` provides the ownership applied when setting ownership behavior."),
        returns = "The `EffectBinding` value with the documented change applied to set ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: sand::entity::EffectBinding, ownership: sand::entity::OwnershipPolicy)  {\n    let updated_effect_binding = effect_binding_value.ownership(ownership);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::refresh",
        aliases = ["sand::prelude::EffectBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(refresh = "`refresh` provides the refresh applied when setting refresh scheduling."),
        returns = "The `EffectBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: sand::entity::EffectBinding, refresh: sand::entity::RefreshPolicy)  {\n    let updated_effect_binding = effect_binding_value.refresh(refresh);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Effect registry identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::effect",
        aliases = ["sand::prelude::EffectBinding::effect"],
        module = "sand::entity",
        kind = "method",
        summary = "Effect registry identifier.",
        context = "Effect registry identifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& StatusEffectId` value produced to effect registry identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: &sand::entity::EffectBinding)  {\n    let effect = effect_binding_value.effect();\n}",
    )]
    #[must_use]
    pub fn effect(&self) -> &StatusEffectId {
        &self.effect
    }

    /// Requested effect duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::duration",
        aliases = ["sand::prelude::EffectBinding::duration"],
        module = "sand::entity",
        kind = "method",
        summary = "Requested effect duration.",
        context = "Requested effect duration. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Ticks` value produced to requested effect duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: &sand::entity::EffectBinding)  {\n    let duration = effect_binding_value.duration();\n}",
    )]
    #[must_use]
    pub const fn duration(&self) -> Ticks {
        self.duration
    }

    /// Zero-based effect amplifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::amplifier_value",
        aliases = ["sand::prelude::EffectBinding::amplifier_value"],
        module = "sand::entity",
        kind = "method",
        summary = "Zero-based effect amplifier.",
        context = "Zero-based effect amplifier. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `u8` value produced to zero-based effect amplifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: &sand::entity::EffectBinding)  {\n    let amplifier_value = effect_binding_value.amplifier_value();\n}",
    )]
    #[must_use]
    pub const fn amplifier_value(&self) -> u8 {
        self.amplifier
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::ownership_policy",
        aliases = ["sand::prelude::EffectBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: &sand::entity::EffectBinding)  {\n    let ownership_policy = effect_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::refresh_policy",
        aliases = ["sand::prelude::EffectBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh scheduling.",
        context = "Selected refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to use selected refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_binding_value: &sand::entity::EffectBinding)  {\n    let refresh_policy = effect_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Stable ownership key.
    #[must_use]
    pub(crate) fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::Effect(NativeEffectKey::new(self.effect.clone()))
    }

    /// Validate refresh scheduling with archetype context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EffectBinding::validate",
        aliases = ["sand::prelude::EffectBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate refresh scheduling with archetype context.",
        context = "Validate refresh scheduling with archetype context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate refresh scheduling with archetype context."),
        returns = "On success, the value produced to validate refresh scheduling with archetype context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(effect_binding_value: &sand::entity::EffectBinding, archetype: impl fmt::Display)  {\n    let validate = effect_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EquipmentBinding",
    aliases = ["sand::prelude::EquipmentBinding"],
    module = "sand::entity",
    summary = "A typed equipment-slot binding reusing Sand's canonical [`ItemStack`].",
    context = "A typed equipment-slot binding reusing Sand's canonical [`ItemStack`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EquipmentBinding;",
)]
/// A typed equipment-slot binding reusing Sand's canonical [`ItemStack`].
#[derive(Debug, Clone)]
pub struct EquipmentBinding {
    slot: EquipmentSlot,
    stack: ItemStack,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl EquipmentBinding {
    /// Equip `stack` in `slot` according to preserve-by-default initialization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::new",
        aliases = ["sand::prelude::EquipmentBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Equip `stack` in `slot` according to preserve-by-default initialization.",
        context = "Equip `stack` in `slot` according to preserve-by-default initialization. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(slot = "Equip `stack` in `slot` according to preserve-by-default initialization.", stack = "Equip `stack` in `slot` according to preserve-by-default initialization."),
        returns = "An `EquipmentBinding` equipping `stack` in `slot` according to preserve-by-default initialization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(slot: sand::component::EquipmentSlot, stack: sand::component::ItemStack)  {\n    let equipment_binding = sand::entity::EquipmentBinding::new(slot, stack);\n}",
    )]
    #[must_use]
    pub fn new(slot: EquipmentSlot, stack: ItemStack) -> Self {
        Self {
            slot,
            stack,
            ownership: OwnershipPolicy::InitializeMissing,
            refresh: RefreshPolicy::Initialize,
        }
    }

    /// Set ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::ownership",
        aliases = ["sand::prelude::EquipmentBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Set ownership behavior.",
        context = "Set ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ownership = "`ownership` provides the ownership applied when setting ownership behavior."),
        returns = "The `EquipmentBinding` value with the documented change applied to set ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: sand::entity::EquipmentBinding, ownership: sand::entity::OwnershipPolicy)  {\n    let updated_equipment_binding = equipment_binding_value.ownership(ownership);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::refresh",
        aliases = ["sand::prelude::EquipmentBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(refresh = "`refresh` provides the refresh applied when setting refresh scheduling."),
        returns = "The `EquipmentBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: sand::entity::EquipmentBinding, refresh: sand::entity::RefreshPolicy)  {\n    let updated_equipment_binding = equipment_binding_value.refresh(refresh);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Equipment slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::slot",
        aliases = ["sand::prelude::EquipmentBinding::slot"],
        module = "sand::entity",
        kind = "method",
        summary = "Equipment slot.",
        context = "Equipment slot. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EquipmentSlot` value produced to equipment slot.",
        example = "use sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: &sand::entity::EquipmentBinding)  {\n    let slot = equipment_binding_value.slot();\n}",
    )]
    #[must_use]
    pub const fn slot(&self) -> EquipmentSlot {
        self.slot
    }

    /// Concrete component-bearing item stack.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::stack",
        aliases = ["sand::prelude::EquipmentBinding::stack"],
        module = "sand::entity",
        kind = "method",
        summary = "Concrete component-bearing item stack.",
        context = "Concrete component-bearing item stack. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& ItemStack` value produced to concrete component-bearing item stack.",
        example = "use sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: &sand::entity::EquipmentBinding)  {\n    let stack = equipment_binding_value.stack();\n}",
    )]
    #[must_use]
    pub fn stack(&self) -> &ItemStack {
        &self.stack
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::ownership_policy",
        aliases = ["sand::prelude::EquipmentBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: &sand::entity::EquipmentBinding)  {\n    let ownership_policy = equipment_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::refresh_policy",
        aliases = ["sand::prelude::EquipmentBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh scheduling.",
        context = "Selected refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to use selected refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: &sand::entity::EquipmentBinding)  {\n    let refresh_policy = equipment_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Stable ownership key.
    #[must_use]
    pub(crate) fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::Equipment(NativeEquipmentKey::new(self.slot))
    }

    /// Validate refresh scheduling with archetype context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EquipmentBinding::validate",
        aliases = ["sand::prelude::EquipmentBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate refresh scheduling with archetype context.",
        context = "Validate refresh scheduling with archetype context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate refresh scheduling with archetype context."),
        returns = "On success, the value produced to validate refresh scheduling with archetype context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(equipment_binding_value: &sand::entity::EquipmentBinding, archetype: impl fmt::Display)  {\n    let validate = equipment_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTextSegment",
    module = "sand::entity",
    summary = "One state-aware custom-name segment.",
    context = "One state-aware custom-name segment. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTextSegment;",
    variants(Canonical = "A canonical styled Sand TextComponent.", Enum = "A finite enum rendered through its stable encoding table.", Flag = "A boolean field rendered using caller-provided labels.", Literal = "Literal user-visible text.", Numeric = "A numeric scoreboard field rendered for `@s`."),
    variant_fields(Canonical(component = "Styled canonical text component."), Enum(color = "Optional named Minecraft color.", dirty_objective = "Hidden dirty objective for the field.", field = "Owning State component and field metadata retained for archetype membership validation.", objective = "Generated objective holding the encoding.", variants = "`(score, display name)` mappings in schema order."), Flag(color = "Optional named Minecraft color.", dirty_objective = "Hidden dirty objective for the field.", disabled = "Text displayed for zero.", enabled = "Text displayed for one.", field = "Owning State component and field metadata retained for archetype membership validation.", objective = "Generated zero/one objective."), Literal(color = "Optional named Minecraft color.", text = "Segment contents."), Numeric(color = "Optional named Minecraft color.", dirty_objective = "Hidden dirty objective for the field.", field = "Owning State component and field metadata retained for archetype membership validation.", objective = "Generated objective holding the field.")),
)]
/// One state-aware custom-name segment.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum EntityTextSegment {
    /// A canonical Sand text component retained without a parallel styling model.
    Canonical {
        /// Styled canonical text component.
        component: TextComponent,
    },
    /// Literal user-visible text.
    Literal {
        /// Segment contents.
        text: String,
        /// Optional named Minecraft color.
        color: Option<ChatColor>,
    },
    /// A numeric scoreboard field rendered for `@s`.
    Numeric {
        /// Generated objective holding the field.
        objective: String,
        /// Hidden dirty objective for the field.
        dirty_objective: String,
        /// Owning component and field identity.
        field: StateFieldReference,
        /// Optional named Minecraft color.
        color: Option<ChatColor>,
    },
    /// A finite enum rendered through its stable encoding table.
    Enum {
        /// Generated objective holding the encoding.
        objective: String,
        /// Hidden dirty objective for the field.
        dirty_objective: String,
        /// Owning component and field identity.
        field: StateFieldReference,
        /// `(score, display name)` mappings in schema order.
        variants: Vec<(i32, String)>,
        /// Optional named Minecraft color.
        color: Option<ChatColor>,
    },
    /// A boolean field rendered using caller-provided labels.
    Flag {
        /// Generated zero/one objective.
        objective: String,
        /// Hidden dirty objective for the field.
        dirty_objective: String,
        /// Owning component and field identity.
        field: StateFieldReference,
        /// Text displayed for zero.
        disabled: String,
        /// Text displayed for one.
        enabled: String,
        /// Optional named Minecraft color.
        color: Option<ChatColor>,
    },
}

impl EntityTextSegment {
    /// Apply a named Minecraft color to this segment.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTextSegment::color",
        module = "sand::entity",
        kind = "method",
        summary = "Apply a named Minecraft color to this segment.",
        context = "Apply a named Minecraft color to this segment. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to apply a named Minecraft color to this segment."),
        returns = "The `EntityTextSegment` value with the documented change applied to apply a named Minecraft color to this segment.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_text_segment_value: sand::entity::EntityTextSegment, value: sand::text::ChatColor)  {\n    let updated_entity_text_segment = entity_text_segment_value.color(value);\n}",
    )]
    #[must_use]
    pub fn color(mut self, value: ChatColor) -> Self {
        match &mut self {
            Self::Canonical { .. } => {}
            Self::Literal { color, .. }
            | Self::Numeric { color, .. }
            | Self::Enum { color, .. }
            | Self::Flag { color, .. } => *color = Some(value),
        }
        self
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityText",
    module = "sand::entity",
    summary = "A custom-name template materialized for the current entity.",
    context = "A custom-name template materialized for the current entity. Dynamic segments use `@s` scoreboard components. Exporters may lower enum and flag mappings through deterministic generated helper functions when direct text components cannot express the mapping.",
    minecraft = "Dynamic segments use `@s` scoreboard components. Exporters may lower enum and flag mappings through deterministic generated helper functions when direct text components cannot express the mapping.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityText;",
)]
/// A custom-name template materialized for the current entity.
///
/// Dynamic segments use `@s` scoreboard components. Exporters may lower enum
/// and flag mappings through deterministic generated helper functions when
/// direct text components cannot express the mapping.
#[derive(Debug, Clone, Default)]
pub struct EntityText {
    segments: Vec<EntityTextSegment>,
}

impl EntityText {
    /// Start an empty template.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::new",
        module = "sand::entity",
        kind = "method",
        summary = "Start an empty template.",
        context = "Start an empty template. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "An `EntityText` initialized to an empty template.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let entity_text = sand::entity::EntityText::new();\n}",
    )]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Append literal text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::literal",
        module = "sand::entity",
        kind = "method",
        summary = "Append literal text.",
        context = "Append literal text. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(text = "`text` provides the author-visible text appended when building literal text."),
        returns = "The `EntityText` value with the documented change applied to append literal text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_text_value: sand::entity::EntityText, text: impl Into < String >)  {\n    let updated_entity_text = entity_text_value.literal(text);\n}",
    )]
    #[must_use]
    pub fn literal(mut self, text: impl Into<String>) -> Self {
        self.segments.push(EntityTextSegment::Literal {
            text: text.into(),
            color: None,
        });
        self
    }

    /// Append a typed numeric state field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::score",
        module = "sand::entity",
        kind = "method",
        summary = "Append a typed numeric state field.",
        context = "Append a typed numeric state field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field appended when building a typed numeric state field."),
        returns = "The `EntityText` value with the documented change applied to append a typed numeric state field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : 'static>(entity_text_value: sand::entity::EntityText, field: sand::entity::EntityScore < T >)  {\n    let updated_entity_text = entity_text_value.score::<T>(field);\n}",
    )]
    #[must_use]
    pub fn score<T: 'static>(mut self, field: EntityScore<T>) -> Self {
        self.segments.push(EntityTextSegment::Numeric {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
            field: field.field_reference(),
            color: None,
        });
        self
    }

    /// Append a typed enum using schema variant names as display strings.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::enum_value",
        module = "sand::entity",
        kind = "method",
        summary = "Append a typed enum using schema variant names as display strings.",
        context = "Append a typed enum using schema variant names as display strings. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field appended when building a typed enum using schema variant names as display strings."),
        returns = "The `EntityText` value with the documented change applied to append a typed enum using schema variant names as display strings.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(entity_text_value: sand::entity::EntityText, field: sand::entity::EntityEnum < T >)  {\n    let updated_entity_text = entity_text_value.enum_value::<T>(field);\n}",
    )]
    #[must_use]
    pub fn enum_value<T: EntityEnumValue>(mut self, field: EntityEnum<T>) -> Self {
        self.segments.push(EntityTextSegment::Enum {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
            field: field.field_reference(),
            variants: T::ENCODINGS
                .iter()
                .map(|encoding| (encoding.score, encoding.name.to_owned()))
                .collect(),
            color: None,
        });
        self
    }

    /// Append a typed flag with explicit display strings.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::flag",
        module = "sand::entity",
        kind = "method",
        summary = "Append a typed flag with explicit display strings.",
        context = "Append a typed flag with explicit display strings. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field appended when building a typed flag with explicit display strings.", disabled = "`disabled` provides the disabled appended when building a typed flag with explicit display strings.", enabled = "`enabled` provides the enabled appended when building a typed flag with explicit display strings."),
        returns = "The `EntityText` value with the documented change applied to append a typed flag with explicit display strings.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_text_value: sand::entity::EntityText, field: sand::entity::EntityFlag, disabled: impl Into < String >, enabled: impl Into < String >)  {\n    let updated_entity_text = entity_text_value.flag(field, disabled, enabled);\n}",
    )]
    #[must_use]
    pub fn flag(
        mut self,
        field: EntityFlag,
        disabled: impl Into<String>,
        enabled: impl Into<String>,
    ) -> Self {
        self.segments.push(EntityTextSegment::Flag {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
            field: field.field_reference(),
            disabled: disabled.into(),
            enabled: enabled.into(),
            color: None,
        });
        self
    }

    /// Color the most recently appended segment.
    ///
    /// Calling this on an empty template is a harmless no-op.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::color_last",
        module = "sand::entity",
        kind = "method",
        summary = "Color the most recently appended segment. Calling this on an empty template is a harmless no-op.",
        context = "Color the most recently appended segment. Calling this on an empty template is a harmless no-op. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(color = "`color` is used to color the most recently appended segment. Calling this on an empty template is a harmless no-op."),
        returns = "The `EntityText` value with the documented change applied to color the most recently appended segment. Calling this on an empty template is a harmless no-op.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_text_value: sand::entity::EntityText, color: sand::text::ChatColor)  {\n    let updated_entity_text = entity_text_value.color_last(color);\n}",
    )]
    #[must_use]
    pub fn color_last(mut self, color: ChatColor) -> Self {
        if let Some(segment) = self.segments.pop() {
            self.segments.push(segment.color(color));
        }
        self
    }

    /// Ordered segments used by text lowering.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityText::segments",
        module = "sand::entity",
        kind = "method",
        summary = "Ordered segments used by text lowering.",
        context = "Ordered segments used by text lowering. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& [EntityTextSegment]` value produced to ordered segments used by text lowering.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_text_value: &sand::entity::EntityText)  {\n    let segments = entity_text_value.segments();\n}",
    )]
    #[must_use]
    pub fn segments(&self) -> &[EntityTextSegment] {
        &self.segments
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityName",
    aliases = ["sand::prelude::EntityName"],
    module = "sand::entity",
    summary = "A component-aware custom entity name built from canonical Sand text and typed State fields.",
    context = "Literal segments use TextComponent directly, while typed State segments retain their owning component identity for archetype membership validation.",
    minecraft = "Sand lowers static TextComponent segments directly and materializes dynamic State values through deterministic archetype helper functions.",
    use_when = ["Building a styled native entity name from static text and composed State fields"],
    avoid_when = ["Building ordinary chat text that does not need archetype State"],
    example = "use sand::entity::EntityName;",
)]
/// Component-aware native entity name using Sand's canonical text styling.
#[derive(Debug, Clone)]
pub struct EntityName {
    text: EntityText,
    visible: bool,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl EntityName {
    /// Start an empty visible name refreshed when a referenced State field changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::new",
        aliases = ["sand::prelude::EntityName::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Starts an empty visible entity name refreshed when referenced State changes.",
        context = "Append canonical TextComponent values with text and typed component fields with state.",
        minecraft = "The completed name is written to CustomName and CustomNameVisible by the archetype compiler.",
        use_when = ["Defining an archetype's native custom name"],
        avoid_when = ["Creating chat, title, or item text"],
        returns = "An empty EntityName builder.",
        example = "let name = sand::entity::EntityName::new();",
    )]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            text: EntityText::new(),
            visible: true,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicy::WhenSourceChanges,
        }
    }

    /// Append one already-styled canonical text component.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::text",
        aliases = ["sand::prelude::EntityName::text"],
        module = "sand::entity",
        kind = "method",
        summary = "Appends one already-styled canonical TextComponent segment.",
        context = "Styling belongs to this segment through the normal Text or TextComponent builder instead of mutating the previous segment.",
        minecraft = "The canonical JSON text component is embedded in CustomName without introducing a second styling DSL.",
        use_when = ["Appending static styled text to an entity name"],
        avoid_when = ["Rendering a State value; use state instead"],
        params(component = "The canonical styled text component to append."),
        returns = "This EntityName with the component appended.",
        example = "use sand::prelude::*; let name = EntityName::new().text(Text::new(\"Boss \" ).gold());",
    )]
    #[must_use]
    pub fn text(mut self, component: TextComponent) -> Self {
        self.text
            .segments
            .push(EntityTextSegment::Canonical { component });
        self
    }

    /// Append a typed numeric State field with styling applied to that segment.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::state",
        aliases = ["sand::prelude::EntityName::state"],
        module = "sand::entity",
        kind = "method",
        summary = "Appends a typed numeric State field with its segment color.",
        context = "The field retains its component and field identity so export rejects references outside the archetype composition.",
        minecraft = "The score is materialized for the current entity and inserted into the generated CustomName text.",
        use_when = ["Showing a composed State score in an entity name"],
        avoid_when = ["Showing an unrelated raw scoreboard objective"],
        params(field = "The typed State score to render.", color = "The Minecraft color applied only to this State segment."),
        returns = "This EntityName with the State segment appended.",
        example = "use sand::prelude::*; fn add(name: EntityName, field: Score) { let _ = name.state(field, ChatColor::Yellow); }",
    )]
    #[must_use]
    pub fn state<T: 'static>(mut self, field: EntityScore<T>, color: ChatColor) -> Self {
        self.text = self.text.score(field).color_last(color);
        self
    }

    /// Append a typed enum State field with styling applied to that segment.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::enum_state",
        aliases = ["sand::prelude::EntityName::enum_state"],
        module = "sand::entity",
        kind = "method",
        summary = "Appends a typed enum State field with its segment color.",
        context = "The enum's stable encoding table is retained while the field's owning component is validated against the archetype composition.",
        minecraft = "Sand selects the encoded display label and inserts it into the generated CustomName text.",
        use_when = ["Showing a composed State enum in an entity name"],
        avoid_when = ["Showing a numeric State value; use state instead"],
        params(field = "The typed State enum to render.", color = "The color applied only to this enum segment."),
        returns = "This EntityName with the enum segment appended.",
        example = "use sand::prelude::*; fn add<T: EntityEnumValue>(name: EntityName, field: EntityEnum<T>) { let _ = name.enum_state(field, ChatColor::Gold); }",
    )]
    #[must_use]
    pub fn enum_state<T: EntityEnumValue>(
        mut self,
        field: EntityEnum<T>,
        color: ChatColor,
    ) -> Self {
        self.text = self.text.enum_value(field).color_last(color);
        self
    }

    /// Append a typed flag State field with labels and styling for that segment.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::flag_state",
        aliases = ["sand::prelude::EntityName::flag_state"],
        module = "sand::entity",
        kind = "method",
        summary = "Appends a typed flag State field with explicit labels and segment color.",
        context = "The flag retains its owning component identity and renders exactly one caller-provided label without a separate styling pass.",
        minecraft = "Sand selects the disabled or enabled label and inserts it into the generated CustomName text.",
        use_when = ["Showing a composed State flag in an entity name"],
        avoid_when = ["Using a flag only as a condition"],
        params(field = "The typed State flag to render.", disabled = "Text rendered for zero.", enabled = "Text rendered for one.", color = "The color applied only to this flag segment."),
        returns = "This EntityName with the flag segment appended.",
        example = "use sand::prelude::*; fn add(name: EntityName, field: EntityFlag) { let _ = name.flag_state(field, \"off\", \"on\", ChatColor::Red); }",
    )]
    #[must_use]
    pub fn flag_state(
        mut self,
        field: EntityFlag,
        disabled: impl Into<String>,
        enabled: impl Into<String>,
        color: ChatColor,
    ) -> Self {
        self.text = self.text.flag(field, disabled, enabled).color_last(color);
        self
    }

    /// Set a deterministic periodic refresh cadence.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::refresh_every",
        aliases = ["sand::prelude::EntityName::refresh_every"],
        module = "sand::entity",
        kind = "method",
        summary = "Refreshes the name at a deterministic tick cadence.",
        context = "Use this when native or external changes require periodic materialization in addition to typed dirty tracking.",
        minecraft = "The archetype shares its reconciliation scan and advances a per-entity refresh clock.",
        use_when = ["A dynamic name must be refreshed periodically"],
        avoid_when = ["Typed State writes are the only source of changes"],
        params(ticks = "The positive refresh interval; zero is rejected during export."),
        returns = "This EntityName with periodic refresh configured.",
        example = "use sand::prelude::*; let name = EntityName::new().refresh_every(Ticks::new(5));",
    )]
    #[must_use]
    pub fn refresh_every(mut self, ticks: Ticks) -> Self {
        self.refresh = RefreshPolicy::Every(ticks);
        self
    }

    /// Control whether Minecraft renders the name without targeting the entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::visible",
        aliases = ["sand::prelude::EntityName::visible"],
        module = "sand::entity",
        kind = "method",
        summary = "Controls whether Minecraft renders the custom name above the entity.",
        context = "Visibility is part of the same native name declaration as its static and State-backed segments.",
        minecraft = "Sand writes CustomNameVisible together with the compiled CustomName.",
        use_when = ["The archetype should own custom-name visibility"],
        avoid_when = ["Another system owns CustomNameVisible"],
        params(visible = "Whether the native custom name is visible."),
        returns = "This EntityName with visibility configured.",
        example = "let name = sand::entity::EntityName::new().visible(false);",
    )]
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set ownership behavior for the native CustomName fields.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityName::ownership",
        aliases = ["sand::prelude::EntityName::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Sets how the archetype reconciles its native custom name.",
        context = "The ownership policy controls whether Sand reapplies or preserves external changes to CustomName and CustomNameVisible.",
        minecraft = "The archetype compiler applies the selected ownership policy during reconciliation.",
        use_when = ["Choosing how the archetype cooperates with external custom-name changes"],
        avoid_when = ["The default dirty-driven ownership is sufficient"],
        params(ownership = "The native property ownership policy."),
        returns = "This EntityName with ownership configured.",
        example = "use sand::prelude::*; let name = EntityName::new().ownership(OwnershipPolicy::PreserveExternal);",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    pub(crate) fn text_value(&self) -> &EntityText {
        &self.text
    }

    pub(crate) const fn is_visible(&self) -> bool {
        self.visible
    }

    pub(crate) const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    pub(crate) fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    pub(crate) fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, NativePropertyKey::Name)
    }
}

impl Default for EntityName {
    fn default() -> Self {
        Self::new()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::NameBinding",
    module = "sand::entity",
    summary = "A state-aware native custom-name declaration.",
    context = "A state-aware native custom-name declaration. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::NameBinding;",
)]
/// A state-aware native custom-name declaration.
#[derive(Debug, Clone)]
pub struct NameBinding {
    text: EntityText,
    visible: bool,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl NameBinding {
    /// Create a visible name refreshed only when a source changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::new",
        module = "sand::entity",
        kind = "method",
        summary = "Create a visible name refreshed only when a source changes.",
        context = "Create a visible name refreshed only when a source changes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(text = "`text` is used when creating a visible name refreshed only when a source changes."),
        returns = "A `NameBinding` representing a visible name refreshed only when a source changes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text: sand::entity::EntityText)  {\n    let name_binding = sand::entity::NameBinding::new(text);\n}",
    )]
    #[must_use]
    pub fn new(text: EntityText) -> Self {
        Self {
            text,
            visible: true,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicy::WhenSourceChanges,
        }
    }

    /// Control `CustomNameVisible`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::visible",
        module = "sand::entity",
        kind = "method",
        summary = "Control `CustomNameVisible`.",
        context = "Control `CustomNameVisible`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(visible = "`visible` provides the switch that enables or disables the behavior used to control `CustomNameVisible`."),
        returns = "The `NameBinding` value with the documented change applied to control `CustomNameVisible`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: sand::entity::NameBinding, visible: bool)  {\n    let updated_name_binding = name_binding_value.visible(visible);\n}",
    )]
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::ownership",
        module = "sand::entity",
        kind = "method",
        summary = "Set ownership behavior.",
        context = "Set ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ownership = "`ownership` provides the ownership applied when setting ownership behavior."),
        returns = "The `NameBinding` value with the documented change applied to set ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: sand::entity::NameBinding, ownership: sand::entity::OwnershipPolicy)  {\n    let updated_name_binding = name_binding_value.ownership(ownership);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::refresh",
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(refresh = "`refresh` provides the refresh applied when setting refresh scheduling."),
        returns = "The `NameBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: sand::entity::NameBinding, refresh: sand::entity::RefreshPolicy)  {\n    let updated_name_binding = name_binding_value.refresh(refresh);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Name template.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::text",
        module = "sand::entity",
        kind = "method",
        summary = "Name template.",
        context = "Name template. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& EntityText` value produced to name template.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: &sand::entity::NameBinding)  {\n    let text = name_binding_value.text();\n}",
    )]
    #[must_use]
    pub fn text(&self) -> &EntityText {
        &self.text
    }

    /// Whether Minecraft should render the name without targeting the entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::is_visible",
        module = "sand::entity",
        kind = "method",
        summary = "Whether Minecraft should render the name without targeting the entity.",
        context = "Whether Minecraft should render the name without targeting the entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "`true` when the documented condition holds to determine whether Minecraft should render the name without targeting the entity; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: &sand::entity::NameBinding)  {\n    let is_is_visible = name_binding_value.is_visible();\n}",
    )]
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::ownership_policy",
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: &sand::entity::NameBinding)  {\n    let ownership_policy = name_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::refresh_policy",
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh scheduling.",
        context = "Selected refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to use selected refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name_binding_value: &sand::entity::NameBinding)  {\n    let refresh_policy = name_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Validate refresh scheduling with archetype context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::NameBinding::validate",
        module = "sand::entity",
        kind = "method",
        summary = "Validate refresh scheduling with archetype context.",
        context = "Validate refresh scheduling with archetype context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate refresh scheduling with archetype context."),
        returns = "On success, the value produced to validate refresh scheduling with archetype context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(name_binding_value: &sand::entity::NameBinding, archetype: impl fmt::Display)  {\n    let validate = name_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, NativePropertyKey::Name)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTag",
    aliases = ["sand::prelude::EntityTag"],
    module = "sand::entity",
    summary = "A validated entity tag. This is intentionally not an `Into<String>` API: whitespace, command delimiters, empty names, and values beyond vanilla's 1024-byte limit are rejected before command generation.",
    context = "A validated entity tag. This is intentionally not an `Into<String>` API: whitespace, command delimiters, empty names, and values beyond vanilla's 1024-byte limit are rejected before command generation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "This is intentionally not an `Into<String>` API: whitespace, command delimiters, empty names, and values beyond vanilla's 1024-byte limit are rejected before command generation.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTag;",
)]
/// A validated entity tag.
///
/// This is intentionally not an `Into<String>` API: whitespace, command
/// delimiters, empty names, and values beyond vanilla's 1024-byte limit are
/// rejected before command generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityTag(String);

impl EntityTag {
    /// Validate and construct an entity tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTag::new",
        aliases = ["sand::prelude::EntityTag::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate and construct an entity tag.",
        context = "Validate and construct an entity tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to validate and construct an entity tag."),
        returns = "On success, the value produced to validate and construct an entity tag; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: & str)  {\n    let entity_tag_result = sand::entity::EntityTag::new(value);\n}",
    )]
    pub fn new(value: &str) -> Result<Self, PropertyNameError> {
        validate_token(value, 1024, "entity tag")?;
        Ok(Self(value.to_owned()))
    }

    pub(crate) fn generated(value: String) -> Self {
        debug_assert!(validate_token(&value, 1024, "entity tag").is_ok());
        Self(value)
    }

    /// Validated command token.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTag::as_str",
        aliases = ["sand::prelude::EntityTag::as_str"],
        module = "sand::entity",
        kind = "method",
        summary = "Validated command token.",
        context = "Validated command token. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The rendered Minecraft command text produced to use validated command token.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_tag_value: &sand::entity::EntityTag)  {\n    let as_str = entity_tag_value.as_str();\n}",
    )]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityTeam",
    aliases = ["sand::prelude::EntityTeam"],
    module = "sand::entity",
    summary = "A validated scoreboard team name.",
    context = "A validated scoreboard team name. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityTeam;",
)]
/// A validated scoreboard team name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityTeam(String);

impl EntityTeam {
    /// Validate and construct a team name.
    ///
    /// Vanilla team names are limited to 16 bytes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTeam::new",
        aliases = ["sand::prelude::EntityTeam::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate and construct a team name. Vanilla team names are limited to 16 bytes.",
        context = "Validate and construct a team name. Vanilla team names are limited to 16 bytes. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to validate and construct a team name. Vanilla team names are limited to 16 bytes."),
        returns = "On success, the value produced to validate and construct a team name. Vanilla team names are limited to 16 bytes; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: & str)  {\n    let entity_team_result = sand::entity::EntityTeam::new(value);\n}",
    )]
    pub fn new(value: &str) -> Result<Self, PropertyNameError> {
        validate_token(value, 16, "team")?;
        Ok(Self(value.to_owned()))
    }

    /// Validated command token.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityTeam::as_str",
        aliases = ["sand::prelude::EntityTeam::as_str"],
        module = "sand::entity",
        kind = "method",
        summary = "Validated command token.",
        context = "Validated command token. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The rendered Minecraft command text produced to use validated command token.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_team_value: &sand::entity::EntityTeam)  {\n    let as_str = entity_team_value.as_str();\n}",
    )]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::TagBinding",
    aliases = ["sand::prelude::TagBinding"],
    module = "sand::entity",
    summary = "A typed archetype-owned entity-tag declaration.",
    context = "A typed archetype-owned entity-tag declaration. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::TagBinding;",
)]
/// A typed archetype-owned entity-tag declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagBinding {
    tag: EntityTag,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl TagBinding {
    /// Add a validated tag during initialization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::new",
        aliases = ["sand::prelude::TagBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Add a validated tag during initialization.",
        context = "Add a validated tag during initialization. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` provides the tag added when building a validated tag during initialization."),
        returns = "A `TagBinding` with a validated tag during initialization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag: sand::entity::EntityTag)  {\n    let tag_binding = sand::entity::TagBinding::new(tag);\n}",
    )]
    #[must_use]
    pub fn new(tag: EntityTag) -> Self {
        Self {
            tag,
            ownership: OwnershipPolicy::InitializeMissing,
            refresh: RefreshPolicy::Initialize,
        }
    }

    /// Set whether reconciliation preserves, observes, or enforces this tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::ownership",
        aliases = ["sand::prelude::TagBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Set whether reconciliation preserves, observes, or enforces this tag.",
        context = "Set whether reconciliation preserves, observes, or enforces this tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ownership = "`ownership` provides the ownership applied when setting whether reconciliation preserves, observes, or enforces this tag."),
        returns = "The `TagBinding` value with the documented change applied to set whether reconciliation preserves, observes, or enforces this tag.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag_binding_value: sand::entity::TagBinding, ownership: sand::entity::OwnershipPolicy)  {\n    let updated_tag_binding = tag_binding_value.ownership(ownership);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::refresh",
        aliases = ["sand::prelude::TagBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(refresh = "`refresh` provides the refresh applied when setting refresh scheduling."),
        returns = "The `TagBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag_binding_value: sand::entity::TagBinding, refresh: sand::entity::RefreshPolicy)  {\n    let updated_tag_binding = tag_binding_value.refresh(refresh);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Validated tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::tag",
        aliases = ["sand::prelude::TagBinding::tag"],
        module = "sand::entity",
        kind = "method",
        summary = "Validated tag.",
        context = "Validated tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& EntityTag` value produced to use validated tag.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag_binding_value: &sand::entity::TagBinding)  {\n    let tag = tag_binding_value.tag();\n}",
    )]
    #[must_use]
    pub fn tag(&self) -> &EntityTag {
        &self.tag
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::ownership_policy",
        aliases = ["sand::prelude::TagBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag_binding_value: &sand::entity::TagBinding)  {\n    let ownership_policy = tag_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::refresh_policy",
        aliases = ["sand::prelude::TagBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh scheduling.",
        context = "Selected refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to use selected refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag_binding_value: &sand::entity::TagBinding)  {\n    let refresh_policy = tag_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Stable conflict key.
    #[must_use]
    pub(crate) fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::Tag(self.tag.clone())
    }

    /// Validate refresh scheduling with archetype context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TagBinding::validate",
        aliases = ["sand::prelude::TagBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate refresh scheduling with archetype context.",
        context = "Validate refresh scheduling with archetype context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate refresh scheduling with archetype context."),
        returns = "On success, the value produced to validate refresh scheduling with archetype context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(tag_binding_value: &sand::entity::TagBinding, archetype: impl fmt::Display)  {\n    let validate = tag_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::TeamBinding",
    aliases = ["sand::prelude::TeamBinding"],
    module = "sand::entity",
    summary = "A typed scoreboard-team membership declaration. Exact ownership changes only the current entity's membership; it does not create, delete, or reconfigure the shared team itself.",
    context = "A typed scoreboard-team membership declaration. Exact ownership changes only the current entity's membership; it does not create, delete, or reconfigure the shared team itself. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::TeamBinding;",
)]
/// A typed scoreboard-team membership declaration.
///
/// Exact ownership changes only the current entity's membership; it does not
/// create, delete, or reconfigure the shared team itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamBinding {
    team: EntityTeam,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl TeamBinding {
    /// Join a validated team during initialization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::new",
        aliases = ["sand::prelude::TeamBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Join a validated team during initialization.",
        context = "Join a validated team during initialization. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(team = "`team` is used to join a validated team during initialization."),
        returns = "A `TeamBinding` joining a validated team during initialization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team: sand::entity::EntityTeam)  {\n    let team_binding = sand::entity::TeamBinding::new(team);\n}",
    )]
    #[must_use]
    pub fn new(team: EntityTeam) -> Self {
        Self {
            team,
            ownership: OwnershipPolicy::InitializeMissing,
            refresh: RefreshPolicy::Initialize,
        }
    }

    /// Set membership ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::ownership",
        aliases = ["sand::prelude::TeamBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Set membership ownership behavior.",
        context = "Set membership ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ownership = "`ownership` provides the ownership applied when setting membership ownership behavior."),
        returns = "The `TeamBinding` value with the documented change applied to set membership ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team_binding_value: sand::entity::TeamBinding, ownership: sand::entity::OwnershipPolicy)  {\n    let updated_team_binding = team_binding_value.ownership(ownership);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::refresh",
        aliases = ["sand::prelude::TeamBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(refresh = "`refresh` provides the refresh applied when setting refresh scheduling."),
        returns = "The `TeamBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team_binding_value: sand::entity::TeamBinding, refresh: sand::entity::RefreshPolicy)  {\n    let updated_team_binding = team_binding_value.refresh(refresh);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Target team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::team",
        aliases = ["sand::prelude::TeamBinding::team"],
        module = "sand::entity",
        kind = "method",
        summary = "Target team.",
        context = "Target team. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& EntityTeam` value produced to target team.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team_binding_value: &sand::entity::TeamBinding)  {\n    let team = team_binding_value.team();\n}",
    )]
    #[must_use]
    pub fn team(&self) -> &EntityTeam {
        &self.team
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::ownership_policy",
        aliases = ["sand::prelude::TeamBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team_binding_value: &sand::entity::TeamBinding)  {\n    let ownership_policy = team_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::refresh_policy",
        aliases = ["sand::prelude::TeamBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh scheduling.",
        context = "Selected refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to use selected refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team_binding_value: &sand::entity::TeamBinding)  {\n    let refresh_policy = team_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Team membership has a single conflict domain per entity.
    #[must_use]
    pub(crate) const fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::Team
    }

    /// Validate refresh scheduling with archetype context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::TeamBinding::validate",
        aliases = ["sand::prelude::TeamBinding::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate refresh scheduling with archetype context.",
        context = "Validate refresh scheduling with archetype context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate refresh scheduling with archetype context."),
        returns = "On success, the value produced to validate refresh scheduling with archetype context; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(team_binding_value: &sand::entity::TeamBinding, archetype: impl fmt::Display)  {\n    let validate = team_binding_value.validate(archetype);\n}",
    )]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::PropertyNameError",
    module = "sand::entity",
    summary = "Invalid typed tag, team, NBT path, or raw extension name.",
    context = "Invalid typed tag, team, NBT path, or raw extension name. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::PropertyNameError;",
)]
/// Invalid typed tag, team, NBT path, or raw extension name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {kind} `{value}`: {reason}")]
pub struct PropertyNameError {
    kind: &'static str,
    value: String,
    reason: &'static str,
}

impl PropertyNameError {
    /// Category of rejected value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PropertyNameError::kind",
        module = "sand::entity",
        kind = "method",
        summary = "Category of rejected value.",
        context = "Category of rejected value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to category of rejected value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(property_name_error_value: &sand::entity::PropertyNameError)  {\n    let kind = property_name_error_value.kind();\n}",
    )]
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// Original rejected value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PropertyNameError::value",
        module = "sand::entity",
        kind = "method",
        summary = "Original rejected value.",
        context = "Original rejected value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to original rejected value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(property_name_error_value: &sand::entity::PropertyNameError)  {\n    let value = property_name_error_value.value();\n}",
    )]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Validation failure.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PropertyNameError::reason",
        module = "sand::entity",
        kind = "method",
        summary = "Validation failure.",
        context = "Validation failure. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to validation failure.",
        example = "use sand::prelude::*;\n\nfn demonstrate(property_name_error_value: &sand::entity::PropertyNameError)  {\n    let reason = property_name_error_value.reason();\n}",
    )]
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

fn validate_token(value: &str, max: usize, kind: &'static str) -> Result<(), PropertyNameError> {
    let reason = if value.is_empty() {
        Some("value is empty")
    } else if value.len() > max {
        Some("value exceeds Minecraft's byte limit")
    } else if value
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '"' | '\'' | '\\'))
    {
        Some("value contains whitespace, control, quote, or escape characters")
    } else {
        None
    };
    if let Some(reason) = reason {
        Err(PropertyNameError {
            kind,
            value: value.to_owned(),
            reason,
        })
    } else {
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityNbtProperty",
    aliases = ["sand::prelude::EntityNbtProperty"],
    module = "sand::entity",
    summary = "Stable, typed native entity-NBT properties supported by Sand.",
    context = "Stable, typed native entity-NBT properties supported by Sand. These variants describe property identity and wire type. Capability and profile validation still occurs before export. In particular, mutable entity-NBT writes are not safe for players.",
    minecraft = "These variants describe property identity and wire type. Capability and profile validation still occurs before export. In particular, mutable entity-NBT writes are not safe for players.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityNbtProperty;",
    variants(Absorption = "`AbsorptionAmount` float on living entities.", AirTicks = "`Air` signed short counter.", FallDistance = "`FallDistance` float.", FireTicks = "`Fire` signed short counter.", FrozenTicks = "`TicksFrozen` integer counter.", Glowing = "`Glowing` byte.", Invulnerable = "`Invulnerable` byte.", NoGravity = "`NoGravity` byte.", Persistent = "`PersistenceRequired` byte for supported mobs.", Silent = "`Silent` byte."),
)]
/// Stable, typed native entity-NBT properties supported by Sand.
///
/// These variants describe property identity and wire type. Capability and
/// profile validation still occurs before export. In particular, mutable
/// entity-NBT writes are not safe for players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityNbtProperty {
    /// `Silent` byte.
    Silent,
    /// `Invulnerable` byte.
    Invulnerable,
    /// `NoGravity` byte.
    NoGravity,
    /// `PersistenceRequired` byte for supported mobs.
    Persistent,
    /// `Glowing` byte.
    Glowing,
    /// `Fire` signed short counter.
    FireTicks,
    /// `FallDistance` float.
    FallDistance,
    /// `Air` signed short counter.
    AirTicks,
    /// `TicksFrozen` integer counter.
    FrozenTicks,
    /// `AbsorptionAmount` float on living entities.
    Absorption,
}

impl EntityNbtProperty {
    /// Canonical NBT path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtProperty::path",
        aliases = ["sand::prelude::EntityNbtProperty::path"],
        module = "sand::entity",
        kind = "method",
        summary = "Canonical NBT path.",
        context = "Canonical NBT path. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to canonical NBT path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_property_value: sand::entity::EntityNbtProperty)  {\n    let path = entity_nbt_property_value.path();\n}",
    )]
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Silent => "Silent",
            Self::Invulnerable => "Invulnerable",
            Self::NoGravity => "NoGravity",
            Self::Persistent => "PersistenceRequired",
            Self::Glowing => "Glowing",
            Self::FireTicks => "Fire",
            Self::FallDistance => "FallDistance",
            Self::AirTicks => "Air",
            Self::FrozenTicks => "TicksFrozen",
            Self::Absorption => "AbsorptionAmount",
        }
    }

    /// Stable wire type expected by this property.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtProperty::wire_type",
        aliases = ["sand::prelude::EntityNbtProperty::wire_type"],
        module = "sand::entity",
        kind = "method",
        summary = "Stable wire type expected by this property.",
        context = "Stable wire type expected by this property. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityNbtType` value produced to use stable wire type expected by this property.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_property_value: sand::entity::EntityNbtProperty)  {\n    let wire_type = entity_nbt_property_value.wire_type();\n}",
    )]
    #[must_use]
    pub const fn wire_type(self) -> EntityNbtType {
        match self {
            Self::Silent
            | Self::Invulnerable
            | Self::NoGravity
            | Self::Persistent
            | Self::Glowing => EntityNbtType::Boolean,
            Self::FireTicks | Self::AirTicks | Self::FrozenTicks => EntityNbtType::Integer,
            Self::FallDistance | Self::Absorption => EntityNbtType::Float,
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityNbtValue",
    aliases = ["sand::prelude::EntityNbtValue"],
    module = "sand::entity",
    summary = "Typed value accepted by a stable native-NBT binding.",
    context = "Typed value accepted by a stable native-NBT binding. Decimal values use explicit fixed-point units to keep authoring and export deterministic and to exclude NaN/infinity.",
    minecraft = "Decimal values use explicit fixed-point units to keep authoring and export deterministic and to exclude NaN/infinity.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityNbtValue;",
    variants(Boolean = "Boolean byte.", Fixed = "Decimal represented as `units / scale`.", Integer = "Signed integer value."),
    variant_fields(Boolean = ["Boolean byte."], Fixed(scale = "`scale` provides the particle scale when decimal represented as `units / scale`.", units = "`units` provides the units when decimal represented as `units / scale`."), Integer = ["Signed integer value."]),
)]
/// Typed value accepted by a stable native-NBT binding.
///
/// Decimal values use explicit fixed-point units to keep authoring and export
/// deterministic and to exclude NaN/infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityNbtValue {
    /// Boolean byte.
    Boolean(#[doc = "Boolean byte."] bool),
    /// Signed integer value.
    Integer(#[doc = "Signed integer value."] i32),
    /// Decimal represented as `units / scale`.
    Fixed {
        #[doc = "`units` provides the units when decimal represented as `units / scale`."]
        units: i64,
        #[doc = "`scale` provides the particle scale when decimal represented as `units / scale`."]
        scale: u32,
    },
}

impl EntityNbtValue {
    /// Construct a fixed-point decimal, rejecting a zero scale.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtValue::fixed",
        aliases = ["sand::prelude::EntityNbtValue::fixed"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a fixed-point decimal, rejecting a zero scale.",
        context = "Construct a fixed-point decimal, rejecting a zero scale. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(units = "`units` is used when constructing a fixed-point decimal, rejecting a zero scale.", scale = "`scale` is used when constructing a fixed-point decimal, rejecting a zero scale."),
        returns = "On success, the value produced to construct a fixed-point decimal, rejecting a zero scale; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(units: i64, scale: u32)  {\n    let fixed = sand::entity::EntityNbtValue::fixed(units, scale);\n}",
    )]
    pub fn fixed(units: i64, scale: u32) -> Result<Self, EntityDiagnostic> {
        if scale == 0 {
            return Err(EntityDiagnostic::FixedPointOverflow {
                archetype: "<native-property>".into(),
                derivation: "<constant>".into(),
                detail: "fixed-point scale must be greater than zero".into(),
            });
        }
        Ok(Self::Fixed { units, scale })
    }

    fn wire_type(&self) -> EntityNbtType {
        match self {
            Self::Boolean(_) => EntityNbtType::Boolean,
            Self::Integer(_) => EntityNbtType::Integer,
            Self::Fixed { .. } => EntityNbtType::Float,
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityNbtBinding",
    aliases = ["sand::prelude::EntityNbtBinding"],
    module = "sand::entity",
    summary = "A stable typed entity-NBT property declaration. This model prevents raw SNBT injection. Exporters must still profile-check the selected field and reject writes to players.",
    context = "A stable typed entity-NBT property declaration. This model prevents raw SNBT injection. Exporters must still profile-check the selected field and reject writes to players. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "This model prevents raw SNBT injection. Exporters must still profile-check the selected field and reject writes to players.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityNbtBinding;",
)]
/// A stable typed entity-NBT property declaration.
///
/// This model prevents raw SNBT injection. Exporters must still profile-check
/// the selected field and reject writes to players.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityNbtBinding {
    property: EntityNbtProperty,
    value: EntityNbtValue,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl EntityNbtBinding {
    /// Bind a stable property to a typed value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::new",
        aliases = ["sand::prelude::EntityNbtBinding::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind a stable property to a typed value.",
        context = "Bind a stable property to a typed value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(property = "`property` provides the property used when binding a stable property to a typed value.", value = "`value` provides the value being applied or compared used to bind a stable property to a typed value."),
        returns = "An `EntityNbtBinding` binding a stable property to a typed value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(property: sand::entity::EntityNbtProperty, value: sand::entity::EntityNbtValue)  {\n    let entity_nbt_binding = sand::entity::EntityNbtBinding::new(property, value);\n}",
    )]
    #[must_use]
    pub fn new(property: EntityNbtProperty, value: EntityNbtValue) -> Self {
        Self {
            property,
            value,
            ownership: OwnershipPolicy::ReconcileWhenDirty,
            refresh: RefreshPolicy::WhenSourceChanges,
        }
    }

    /// Set ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::ownership",
        aliases = ["sand::prelude::EntityNbtBinding::ownership"],
        module = "sand::entity",
        kind = "method",
        summary = "Set ownership behavior.",
        context = "Set ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ownership = "`ownership` provides the ownership applied when setting ownership behavior."),
        returns = "The `EntityNbtBinding` value with the documented change applied to set ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_binding_value: sand::entity::EntityNbtBinding, ownership: sand::entity::OwnershipPolicy)  {\n    let updated_entity_nbt_binding = entity_nbt_binding_value.ownership(ownership);\n}",
    )]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::refresh",
        aliases = ["sand::prelude::EntityNbtBinding::refresh"],
        module = "sand::entity",
        kind = "method",
        summary = "Set refresh scheduling.",
        context = "Set refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(refresh = "`refresh` provides the refresh applied when setting refresh scheduling."),
        returns = "The `EntityNbtBinding` value with the documented change applied to set refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_binding_value: sand::entity::EntityNbtBinding, refresh: sand::entity::RefreshPolicy)  {\n    let updated_entity_nbt_binding = entity_nbt_binding_value.refresh(refresh);\n}",
    )]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Stable NBT property.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::property",
        aliases = ["sand::prelude::EntityNbtBinding::property"],
        module = "sand::entity",
        kind = "method",
        summary = "Stable NBT property.",
        context = "Stable NBT property. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityNbtProperty` value produced to use stable NBT property.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_binding_value: &sand::entity::EntityNbtBinding)  {\n    let property = entity_nbt_binding_value.property();\n}",
    )]
    #[must_use]
    pub const fn property(&self) -> EntityNbtProperty {
        self.property
    }

    /// Typed NBT value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::value",
        aliases = ["sand::prelude::EntityNbtBinding::value"],
        module = "sand::entity",
        kind = "method",
        summary = "Typed NBT value.",
        context = "Typed NBT value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& EntityNbtValue` value produced to typed NBT value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_binding_value: &sand::entity::EntityNbtBinding)  {\n    let value = entity_nbt_binding_value.value();\n}",
    )]
    #[must_use]
    pub fn value(&self) -> &EntityNbtValue {
        &self.value
    }

    /// Selected ownership behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::ownership_policy",
        aliases = ["sand::prelude::EntityNbtBinding::ownership_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected ownership behavior.",
        context = "Selected ownership behavior. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `OwnershipPolicy` value produced to use selected ownership behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_binding_value: &sand::entity::EntityNbtBinding)  {\n    let ownership_policy = entity_nbt_binding_value.ownership_policy();\n}",
    )]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::refresh_policy",
        aliases = ["sand::prelude::EntityNbtBinding::refresh_policy"],
        module = "sand::entity",
        kind = "method",
        summary = "Selected refresh scheduling.",
        context = "Selected refresh scheduling. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& RefreshPolicy` value produced to use selected refresh scheduling.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_nbt_binding_value: &sand::entity::EntityNbtBinding)  {\n    let refresh_policy = entity_nbt_binding_value.refresh_policy();\n}",
    )]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Stable conflict key.
    #[must_use]
    pub(crate) const fn property_key(&self) -> NativePropertyKey {
        NativePropertyKey::Nbt(self.property)
    }

    /// Reject direct player entity-NBT mutation before lowering.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityNbtBinding::validate_for",
        aliases = ["sand::prelude::EntityNbtBinding::validate_for"],
        module = "sand::entity",
        kind = "method",
        summary = "Reject direct player entity-NBT mutation before lowering.",
        context = "Reject direct player entity-NBT mutation before lowering. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to reject direct player entity-NBT mutation before lowering."),
        returns = "On success, the value produced to reject direct player entity-NBT mutation before lowering; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_nbt_binding_value: &sand::entity::EntityNbtBinding, archetype: impl fmt::Display)  {\n    let validate_for = entity_nbt_binding_value.validate_for::<K>(archetype);\n}",
    )]
    pub fn validate_for<K: EntityKind>(
        &self,
        archetype: impl fmt::Display,
    ) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(&archetype, self.property_key())?;
        if self.property.wire_type() != self.value.wire_type() {
            return Err(EntityDiagnostic::InvalidRawExtension {
                archetype: archetype.to_string(),
                extension: self.property_key().to_string(),
                detail: format!(
                    "typed NBT property expects {:?}, received {:?}",
                    self.property.wire_type(),
                    self.value.wire_type()
                ),
            });
        }
        if std::any::TypeId::of::<K>() == std::any::TypeId::of::<PlayerKind>()
            && self.ownership.claims_write_ownership()
        {
            return Err(EntityDiagnostic::UnsafePlayerMutation {
                archetype: archetype.to_string(),
                property: self.property_key().to_string(),
            });
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityNbtType",
    aliases = ["sand::prelude::EntityNbtType"],
    module = "sand::entity",
    summary = "Wire type for an explicitly unsupported/modded NBT property.",
    context = "Wire type for an explicitly unsupported/modded NBT property. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityNbtType;",
    variants(Boolean = "Boolean byte (`0b`/`1b`).", Compound = "Arbitrary compound/list value whose SNBT is supplied by advanced code.", Float = "32-bit floating point value.", Integer = "Signed 32-bit integer.", String = "UTF-8 NBT string."),
)]
/// Wire type for an explicitly unsupported/modded NBT property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EntityNbtType {
    /// Boolean byte (`0b`/`1b`).
    Boolean,
    /// Signed 32-bit integer.
    Integer,
    /// 32-bit floating point value.
    Float,
    /// UTF-8 NBT string.
    String,
    /// Arbitrary compound/list value whose SNBT is supplied by advanced code.
    Compound,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RawPropertyAccess",
    module = "sand::entity",
    summary = "Whether a raw property declaration only reads or may mutate NBT.",
    context = "Whether a raw property declaration only reads or may mutate NBT. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::RawPropertyAccess;",
    variants(Mutable = "Native entity-NBT mutation.", ReadOnly = "Read-only observation."),
)]
/// Whether a raw property declaration only reads or may mutate NBT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawPropertyAccess {
    /// Read-only observation.
    ReadOnly,
    /// Native entity-NBT mutation.
    Mutable,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RawEntityProperty",
    aliases = ["sand::prelude::RawEntityProperty"],
    module = "sand::entity",
    summary = "Explicit escape hatch for unsupported or modded native properties.",
    context = "Explicit escape hatch for unsupported or modded native properties. Raw declarations are never inferred as player-safe. Mutable raw properties fail [`Self::validate_for`] for [`PlayerKind`].",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::RawEntityProperty;",
)]
/// Explicit escape hatch for unsupported or modded native properties.
///
/// Raw declarations are never inferred as player-safe. Mutable raw properties
/// fail [`Self::validate_for`] for [`PlayerKind`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntityProperty {
    path: String,
    wire_type: EntityNbtType,
    access: RawPropertyAccess,
}

impl RawEntityProperty {
    /// Construct a validated raw NBT path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityProperty::new",
        aliases = ["sand::prelude::RawEntityProperty::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a validated raw NBT path.",
        context = "Construct a validated raw NBT path. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(path = "`path` provides the typed resource identifier or location used to construct a validated raw NBT path.", wire_type = "`wire_type` is used when constructing a validated raw NBT path.", access = "`access` is used when constructing a validated raw NBT path."),
        returns = "On success, the value produced to construct a validated raw NBT path; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: & str, wire_type: sand::entity::EntityNbtType, access: sand::entity::RawPropertyAccess)  {\n    let raw_entity_property_result = sand::entity::RawEntityProperty::new(path, wire_type, access);\n}",
    )]
    pub fn new(
        path: &str,
        wire_type: EntityNbtType,
        access: RawPropertyAccess,
    ) -> Result<Self, PropertyNameError> {
        validate_nbt_path(path)?;
        Ok(Self {
            path: path.to_owned(),
            wire_type,
            access,
        })
    }

    /// NBT path owned by advanced caller code.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityProperty::path",
        aliases = ["sand::prelude::RawEntityProperty::path"],
        module = "sand::entity",
        kind = "method",
        summary = "NBT path owned by advanced caller code.",
        context = "NBT path owned by advanced caller code. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to nbt path owned by advanced caller code.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_entity_property_value: &sand::entity::RawEntityProperty)  {\n    let path = raw_entity_property_value.path();\n}",
    )]
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Declared wire type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityProperty::wire_type",
        aliases = ["sand::prelude::RawEntityProperty::wire_type"],
        module = "sand::entity",
        kind = "method",
        summary = "Declared wire type.",
        context = "Declared wire type. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityNbtType` value produced to declared wire type.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_entity_property_value: &sand::entity::RawEntityProperty)  {\n    let wire_type = raw_entity_property_value.wire_type();\n}",
    )]
    #[must_use]
    pub const fn wire_type(&self) -> EntityNbtType {
        self.wire_type
    }

    /// Declared access mode.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityProperty::access",
        aliases = ["sand::prelude::RawEntityProperty::access"],
        module = "sand::entity",
        kind = "method",
        summary = "Declared access mode.",
        context = "Declared access mode. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RawPropertyAccess` value produced to declared access mode.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_entity_property_value: &sand::entity::RawEntityProperty)  {\n    let access = raw_entity_property_value.access();\n}",
    )]
    #[must_use]
    pub const fn access(&self) -> RawPropertyAccess {
        self.access
    }

    /// Validate player safety for a statically known entity kind.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityProperty::validate_for",
        aliases = ["sand::prelude::RawEntityProperty::validate_for"],
        module = "sand::entity",
        kind = "method",
        summary = "Validate player safety for a statically known entity kind.",
        context = "Validate player safety for a statically known entity kind. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(archetype = "`archetype` provides the entity archetype supplying the property used to validate player safety for a statically known entity kind."),
        returns = "On success, the value produced to validate player safety for a statically known entity kind; otherwise, the documented validation or export diagnostic.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(raw_entity_property_value: &sand::entity::RawEntityProperty, archetype: impl fmt::Display)  {\n    let validate_for = raw_entity_property_value.validate_for::<K>(archetype);\n}",
    )]
    pub fn validate_for<K: EntityKind>(
        &self,
        archetype: impl fmt::Display,
    ) -> Result<(), EntityDiagnostic> {
        if std::any::TypeId::of::<K>() == std::any::TypeId::of::<PlayerKind>()
            && self.access == RawPropertyAccess::Mutable
        {
            return Err(EntityDiagnostic::UnsafePlayerMutation {
                archetype: archetype.to_string(),
                property: self.path.clone(),
            });
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RawEntityStateField",
    aliases = ["sand::prelude::RawEntityStateField"],
    module = "sand::entity",
    summary = "Explicit escape hatch for a state field Sand does not model.",
    context = "Explicit escape hatch for a state field Sand does not model. The backend is named rather than silently mapped to a shared storage compound. Exporters must namespace the key per schema and per entity.",
    minecraft = "The backend is named rather than silently mapped to a shared storage compound. Exporters must namespace the key per schema and per entity.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::RawEntityStateField;",
)]
/// Explicit escape hatch for a state field Sand does not model.
///
/// The backend is named rather than silently mapped to a shared storage
/// compound. Exporters must namespace the key per schema and per entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntityStateField {
    name: String,
    backend: RawStateBackend,
}

impl RawEntityStateField {
    /// Construct a validated raw state field name and explicit backend.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityStateField::new",
        aliases = ["sand::prelude::RawEntityStateField::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Construct a validated raw state field name and explicit backend.",
        context = "Construct a validated raw state field name and explicit backend. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(name = "`name` is used when constructing a validated raw state field name and explicit backend.", backend = "`backend` is used when constructing a validated raw state field name and explicit backend."),
        returns = "On success, the value produced to construct a validated raw state field name and explicit backend; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: & str, backend: sand::entity::RawStateBackend)  {\n    let raw_entity_state_field_result = sand::entity::RawEntityStateField::new(name, backend);\n}",
    )]
    pub fn new(name: &str, backend: RawStateBackend) -> Result<Self, PropertyNameError> {
        validate_token(name, 64, "raw entity state field")?;
        Ok(Self {
            name: name.to_owned(),
            backend,
        })
    }

    /// Caller-owned logical field name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityStateField::name",
        aliases = ["sand::prelude::RawEntityStateField::name"],
        module = "sand::entity",
        kind = "method",
        summary = "Caller-owned logical field name.",
        context = "Caller-owned logical field name. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to caller-owned logical field name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_entity_state_field_value: &sand::entity::RawEntityStateField)  {\n    let name = raw_entity_state_field_value.name();\n}",
    )]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Explicit persistence backend.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RawEntityStateField::backend",
        aliases = ["sand::prelude::RawEntityStateField::backend"],
        module = "sand::entity",
        kind = "method",
        summary = "Explicit persistence backend.",
        context = "Explicit persistence backend. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RawStateBackend` value produced to use explicit persistence backend.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_entity_state_field_value: &sand::entity::RawEntityStateField)  {\n    let backend = raw_entity_state_field_value.backend();\n}",
    )]
    #[must_use]
    pub const fn backend(&self) -> RawStateBackend {
        self.backend
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RawStateBackend",
    module = "sand::entity",
    summary = "Persistence backend chosen by a raw state field.",
    context = "Persistence backend chosen by a raw state field. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::RawStateBackend;",
    variants(SandStorage = "Sand-owned per-entity storage with collision-safe identity choreography.", Scoreboard = "Entity scoreboard entry.", Tag = "Entity tag presence."),
)]
/// Persistence backend chosen by a raw state field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RawStateBackend {
    /// Entity scoreboard entry.
    Scoreboard,
    /// Entity tag presence.
    Tag,
    /// Sand-owned per-entity storage with collision-safe identity choreography.
    SandStorage,
}

fn validate_nbt_path(value: &str) -> Result<(), PropertyNameError> {
    if value.is_empty()
        || value.len() > 256
        || value.chars().any(|c| {
            c.is_whitespace() || c.is_control() || matches!(c, '"' | '\'' | '\\' | ';' | '{' | '}')
        })
    {
        return Err(PropertyNameError {
            kind: "raw entity NBT path",
            value: value.to_owned(),
            reason: "expected a non-empty, bounded simple NBT path without SNBT delimiters",
        });
    }
    Ok(())
}

/// Stable key used to detect conflicting native-property ownership.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NativeAttributeKey(String);

impl NativeAttributeKey {
    /// Construct a conflict key from Sand's typed attribute model.
    #[must_use]
    pub fn new(attribute: AttributeType) -> Self {
        Self(attribute.as_str().to_owned())
    }

    /// Canonical registry identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable effect ownership key constructed from a registry identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NativeEffectKey(String);

impl NativeEffectKey {
    /// Construct a conflict key from a validated status-effect ID.
    #[must_use]
    pub fn new(effect: StatusEffectId) -> Self {
        Self(effect.to_string())
    }

    /// Canonical registry identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable equipment ownership key constructed from a typed slot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NativeEquipmentKey(String);

impl NativeEquipmentKey {
    /// Construct a conflict key from a typed equipment slot.
    #[must_use]
    pub fn new(slot: EquipmentSlot) -> Self {
        Self(slot.as_str().to_owned())
    }

    /// Canonical slot name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable key used to detect conflicting native-property ownership.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub(crate) enum NativePropertyKey {
    /// Native current/max-health group.
    Health,
    /// Attribute base/modifier group by registry ID.
    Attribute(NativeAttributeKey),
    /// One namespaced modifier on one attribute.
    AttributeModifier {
        /// Attribute registry key.
        attribute: NativeAttributeKey,
        /// Canonical modifier resource ID.
        id: String,
    },
    /// Status effect by registry ID.
    Effect(NativeEffectKey),
    /// Equipment by canonical slot name. This is constructed from a typed
    /// [`EquipmentSlot`] by [`EquipmentBinding::property_key`].
    Equipment(NativeEquipmentKey),
    /// Custom name and visibility.
    Name,
    /// Entity tag by validated token.
    Tag(EntityTag),
    /// Team membership.
    Team,
    /// Stable typed NBT path.
    Nbt(EntityNbtProperty),
}

impl fmt::Display for NativePropertyKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Health => formatter.write_str("health"),
            Self::Attribute(id) => write!(formatter, "attribute:{}", id.as_str()),
            Self::AttributeModifier { attribute, id } => {
                write!(formatter, "attribute:{}:modifier:{id}", attribute.as_str())
            }
            Self::Effect(id) => write!(formatter, "effect:{}", id.as_str()),
            Self::Equipment(slot) => write!(formatter, "equipment:{}", slot.as_str()),
            Self::Name => formatter.write_str("name"),
            Self::Tag(tag) => write!(formatter, "tag:{}", tag.as_str()),
            Self::Team => formatter.write_str("team"),
            Self::Nbt(property) => write!(formatter, "nbt:{}", property.path()),
        }
    }
}

/// Validate unique write ownership and return a contextual diagnostic.
pub(crate) fn validate_native_ownership<'a>(
    archetype: impl fmt::Display,
    declarations: impl IntoIterator<Item = (&'a NativePropertyKey, OwnershipPolicy, &'a str)>,
) -> Result<(), EntityDiagnostic> {
    let mut owners = std::collections::BTreeMap::<&NativePropertyKey, &str>::new();
    for (key, policy, owner) in declarations {
        if !policy.claims_write_ownership() {
            continue;
        }
        if let Some(previous) = owners.insert(key, owner) {
            return Err(EntityDiagnostic::ConflictingOwnership {
                archetype: archetype.to_string(),
                property: key.to_string(),
                first: previous.to_owned(),
                second: owner.to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ZombieKind;

    #[test]
    fn ownership_only_conflicts_for_writers() {
        let health = NativePropertyKey::Health;
        assert!(
            validate_native_ownership(
                "rpg:zombie",
                [
                    (&health, OwnershipPolicy::Observe, "observer"),
                    (&health, OwnershipPolicy::Exact, "archetype"),
                ],
            )
            .is_ok()
        );
        let error = validate_native_ownership(
            "rpg:zombie",
            [
                (&health, OwnershipPolicy::InitializeMissing, "base"),
                (&health, OwnershipPolicy::ReconcileWhenDirty, "rarity"),
            ],
        )
        .unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-OWNERSHIP");
    }

    #[test]
    fn keys_distinguish_attribute_effect_and_slot() {
        let attribute = AttributeBinding::new(
            AttributeType::AttackDamage,
            NumericPropertySource::fixed(75, 10).unwrap(),
        );
        let effect = EffectBinding::new(
            StatusEffectId::minecraft("strength").unwrap(),
            Ticks::seconds(5),
        );
        assert_eq!(
            attribute.property_key().to_string(),
            "attribute:minecraft:attack_damage"
        );
        assert_eq!(
            effect.property_key().to_string(),
            "effect:minecraft:strength"
        );
        assert_ne!(
            NativePropertyKey::Equipment(NativeEquipmentKey::new(EquipmentSlot::Head)),
            NativePropertyKey::Equipment(NativeEquipmentKey::new(EquipmentSlot::Mainhand))
        );
    }

    #[test]
    fn zero_refresh_is_rejected() {
        let error = RefreshPolicy::Every(Ticks::new(0))
            .validate("rpg:zombie", NativePropertyKey::Health)
            .unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-REFRESH-INTERVAL");
    }

    #[test]
    fn typed_nbt_rejects_a_wire_type_mismatch() {
        let binding = EntityNbtBinding::new(EntityNbtProperty::Silent, EntityNbtValue::Integer(1));
        let error = binding
            .validate_for::<ZombieKind>("rpg:zombie")
            .unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-RAW");
    }

    #[test]
    fn periodic_refill_is_rejected_before_it_can_heal_every_tick() {
        let health = EntityScore::new("rpg", "mob", "max", 20, Some((1, 100)));
        let binding = HealthBinding::new(health)
            .resize(HealthResizePolicy::Refill)
            .observe_native_every(Ticks::new(20));
        let error = binding.validate("rpg:zombie").unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-HEALTH-RESIZE");
    }

    #[test]
    fn named_modifiers_have_independent_ownership_keys() {
        let first = AttributeModifierBinding::new(
            AttributeType::MovementSpeed,
            crate::ResourceLocation::new("rpg", "sickness").unwrap(),
            NumericPropertySource::fixed(-10, 100).unwrap(),
            AttributeOperation::AddMultipliedTotal,
        );
        let second = AttributeModifierBinding::new(
            AttributeType::MovementSpeed,
            crate::ResourceLocation::new("rpg", "phase").unwrap(),
            NumericPropertySource::fixed(10, 100).unwrap(),
            AttributeOperation::AddMultipliedTotal,
        );
        assert_ne!(first.property_key(), second.property_key());
    }

    #[test]
    fn typed_names_reject_command_fragments() {
        assert!(EntityTag::new("rpg.mob").is_ok());
        assert!(EntityTag::new("bad tag").is_err());
        assert!(EntityTeam::new("0123456789abcdef").is_ok());
        assert!(EntityTeam::new("0123456789abcdefg").is_err());
    }

    #[test]
    fn mutable_raw_nbt_is_never_player_safe() {
        let raw = RawEntityProperty::new(
            "ModdedPower",
            EntityNbtType::Integer,
            RawPropertyAccess::Mutable,
        )
        .unwrap();
        assert_eq!(
            raw.validate_for::<PlayerKind>("rpg:player")
                .unwrap_err()
                .code(),
            "SAND-ENTITY-PLAYER-NBT"
        );
        assert!(raw.validate_for::<ZombieKind>("rpg:zombie").is_ok());
    }

    #[test]
    fn raw_boundaries_reject_snbt_injection() {
        assert!(
            RawEntityProperty::new(
                "Health; run kill @a",
                EntityNbtType::Float,
                RawPropertyAccess::Mutable
            )
            .is_err()
        );
        assert!(RawEntityStateField::new("modded_power", RawStateBackend::SandStorage).is_ok());
        assert!(RawEntityStateField::new("bad field", RawStateBackend::Scoreboard).is_err());
    }
}
