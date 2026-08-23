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

use sand_commands::ChatColor;
use sand_components::{
    AttributeOperation, AttributeType, EquipmentSlot, ItemStack, StatusEffectId, Ticks,
};

use crate::entity::diagnostic::EntityDiagnostic;
use crate::entity::kind::{EntityKind, PlayerKind};
use crate::entity::state::{
    EntityEnum, EntityEnumValue, EntityFlag, EntityScore, EntityStateField,
};
use crate::resource_ref::FunctionId;

/// A typed identifier for an event that requests a property refresh.
///
/// Event identifiers are resource locations so independently authored packs
/// cannot accidentally share an unqualified event name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityEventId(crate::ResourceLocation);

impl EntityEventId {
    /// Construct a namespaced event identifier.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEventId::new` for the canonical contract."]
    pub fn new(location: crate::ResourceLocation) -> Self {
        Self(location)
    }

    /// Return the underlying resource location.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityEventId::location` for the canonical contract."]
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
    Every(Ticks),
    /// Refresh when the canonical datapack function is dispatched.
    OnFunction(FunctionId),
    /// Refresh when Sand dispatches the named typed event.
    OnEvent(EntityEventId),
    /// Generate no automatic scheduling; user code explicitly requests work.
    Manual,
}

impl RefreshPolicy {
    /// Validate scheduling invariants with archetype/property context.
    #[doc = "**API Contract:** Run `sand api show sand::entity::RefreshPolicy::validate` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::OwnershipPolicy::claims_write_ownership` for the canonical contract."]
    #[must_use]
    pub const fn claims_write_ownership(self) -> bool {
        matches!(
            self,
            Self::InitializeMissing | Self::Exact | Self::ReconcileWhenDirty
        )
    }

    /// Whether this policy reads native runtime state.
    #[doc = "**API Contract:** Run `sand api show sand::entity::OwnershipPolicy::observes_native_state` for the canonical contract."]
    #[must_use]
    pub const fn observes_native_state(self) -> bool {
        matches!(self, Self::Observe)
    }
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::new` for the canonical contract."]
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

    /// Add a state score representing native current health.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::current_health` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::observe_native_every` for the canonical contract."]
    #[must_use]
    pub fn observe_native_every(mut self, interval: Ticks) -> Self {
        self.observation_interval = Some(interval);
        self
    }

    /// Select behavior when max health changes.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::resize` for the canonical contract."]
    #[must_use]
    pub fn resize(mut self, policy: HealthResizePolicy) -> Self {
        self.resize = policy;
        self
    }

    /// Select ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, policy: OwnershipPolicy) -> Self {
        self.ownership = policy;
        self
    }

    /// Select automatic refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.refresh = policy.into();
        self
    }

    /// Max-health state field.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::max_health_field` for the canonical contract."]
    #[must_use]
    pub const fn max_health_field(&self) -> EntityScore<i32> {
        self.max_health
    }

    /// Optional current-health state field.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::current_health_field` for the canonical contract."]
    #[must_use]
    pub const fn current_health_field(&self) -> Option<EntityScore<i32>> {
        self.current_health
    }

    /// Selected resize behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::resize_policy` for the canonical contract."]
    #[must_use]
    pub const fn resize_policy(&self) -> HealthResizePolicy {
        self.resize
    }

    /// Selected current-health direction.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::current_health_sync` for the canonical contract."]
    #[must_use]
    pub const fn current_health_sync(&self) -> CurrentHealthSync {
        self.current_sync
    }

    /// Optional native-health observation cadence.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::observation_interval` for the canonical contract."]
    #[must_use]
    pub const fn observation_interval(&self) -> Option<Ticks> {
        self.observation_interval
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::refresh_policy` for the canonical contract."]
    #[must_use]
    pub fn refresh_policy(&self) -> RefreshPolicy {
        self.refresh.clone().into()
    }

    /// Validate combinations that cannot preserve their documented semantics.
    #[doc = "**API Contract:** Run `sand api show sand::entity::HealthBinding::validate` for the canonical contract."]
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

/// A typed numeric source for an attribute or effect parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NumericPropertySource {
    /// A constant fixed-point value: `units / scale`.
    Fixed { units: i64, scale: u32 },
    /// The value of a typed entity score.
    StateScore {
        /// Generated score objective.
        objective: String,
        /// Hidden source-dirty objective marked by typed mutations.
        dirty_objective: String,
    },
}

impl NumericPropertySource {
    /// Construct a finite fixed-point constant.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NumericPropertySource::fixed` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::NumericPropertySource::state` for the canonical contract."]
    #[must_use]
    pub fn state<T: 'static>(field: EntityScore<T>) -> Self {
        Self::StateScore {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
        }
    }
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, policy: OwnershipPolicy) -> Self {
        self.ownership = policy;
        self
    }

    /// Select refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.refresh = policy;
        self
    }

    /// Target attribute.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::attribute` for the canonical contract."]
    #[must_use]
    pub fn attribute(&self) -> &AttributeType {
        &self.attribute
    }

    /// Stable modifier resource ID.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::id` for the canonical contract."]
    #[must_use]
    pub fn id(&self) -> &crate::ResourceLocation {
        &self.id
    }

    /// Numeric modifier amount.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::source` for the canonical contract."]
    #[must_use]
    pub fn source(&self) -> &NumericPropertySource {
        &self.source
    }

    /// Vanilla modifier operation.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::operation` for the canonical contract."]
    #[must_use]
    pub const fn operation(&self) -> AttributeOperation {
        self.operation
    }

    /// Ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeModifierBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

impl AttributeBinding {
    /// Bind an attribute base value to a typed numeric source.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, policy: OwnershipPolicy) -> Self {
        self.ownership = policy;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.refresh = policy;
        self
    }

    /// Native attribute identifier.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::attribute` for the canonical contract."]
    #[must_use]
    pub fn attribute(&self) -> &AttributeType {
        &self.attribute
    }

    /// Numeric source.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::source` for the canonical contract."]
    #[must_use]
    pub fn source(&self) -> &NumericPropertySource {
        &self.source
    }

    /// Ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::AttributeBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::amplifier` for the canonical contract."]
    #[must_use]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    /// Set ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Effect registry identifier.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::effect` for the canonical contract."]
    #[must_use]
    pub fn effect(&self) -> &StatusEffectId {
        &self.effect
    }

    /// Requested effect duration.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::duration` for the canonical contract."]
    #[must_use]
    pub const fn duration(&self) -> Ticks {
        self.duration
    }

    /// Zero-based effect amplifier.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::amplifier_value` for the canonical contract."]
    #[must_use]
    pub const fn amplifier_value(&self) -> u8 {
        self.amplifier
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EffectBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Equipment slot.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::slot` for the canonical contract."]
    #[must_use]
    pub const fn slot(&self) -> EquipmentSlot {
        self.slot
    }

    /// Concrete component-bearing item stack.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::stack` for the canonical contract."]
    #[must_use]
    pub fn stack(&self) -> &ItemStack {
        &self.stack
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EquipmentBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

/// One state-aware custom-name segment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityTextSegment {
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
        /// Optional named Minecraft color.
        color: Option<ChatColor>,
    },
    /// A finite enum rendered through its stable encoding table.
    Enum {
        /// Generated objective holding the encoding.
        objective: String,
        /// Hidden dirty objective for the field.
        dirty_objective: String,
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTextSegment::color` for the canonical contract."]
    #[must_use]
    pub fn color(mut self, value: ChatColor) -> Self {
        match &mut self {
            Self::Literal { color, .. }
            | Self::Numeric { color, .. }
            | Self::Enum { color, .. }
            | Self::Flag { color, .. } => *color = Some(value),
        }
        self
    }
}

/// A custom-name template materialized for the current entity.
///
/// Dynamic segments use `@s` scoreboard components. Exporters may lower enum
/// and flag mappings through deterministic generated helper functions when
/// direct text components cannot express the mapping.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityText {
    segments: Vec<EntityTextSegment>,
}

impl EntityText {
    /// Start an empty template.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::new` for the canonical contract."]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Append literal text.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::literal` for the canonical contract."]
    #[must_use]
    pub fn literal(mut self, text: impl Into<String>) -> Self {
        self.segments.push(EntityTextSegment::Literal {
            text: text.into(),
            color: None,
        });
        self
    }

    /// Append a typed numeric state field.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::score` for the canonical contract."]
    #[must_use]
    pub fn score<T: 'static>(mut self, field: EntityScore<T>) -> Self {
        self.segments.push(EntityTextSegment::Numeric {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
            color: None,
        });
        self
    }

    /// Append a typed enum using schema variant names as display strings.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::enum_value` for the canonical contract."]
    #[must_use]
    pub fn enum_value<T: EntityEnumValue>(mut self, field: EntityEnum<T>) -> Self {
        self.segments.push(EntityTextSegment::Enum {
            objective: field.objective(),
            dirty_objective: field.dirty_objective(),
            variants: T::ENCODINGS
                .iter()
                .map(|encoding| (encoding.score, encoding.name.to_owned()))
                .collect(),
            color: None,
        });
        self
    }

    /// Append a typed flag with explicit display strings.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::flag` for the canonical contract."]
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
            disabled: disabled.into(),
            enabled: enabled.into(),
            color: None,
        });
        self
    }

    /// Color the most recently appended segment.
    ///
    /// Calling this on an empty template is a harmless no-op.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::color_last` for the canonical contract."]
    #[must_use]
    pub fn color_last(mut self, color: ChatColor) -> Self {
        if let Some(segment) = self.segments.pop() {
            self.segments.push(segment.color(color));
        }
        self
    }

    /// Ordered segments used by text lowering.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityText::segments` for the canonical contract."]
    #[must_use]
    pub fn segments(&self) -> &[EntityTextSegment] {
        &self.segments
    }
}

/// A state-aware native custom-name declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameBinding {
    text: EntityText,
    visible: bool,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl NameBinding {
    /// Create a visible name refreshed only when a source changes.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::visible` for the canonical contract."]
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Name template.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::text` for the canonical contract."]
    #[must_use]
    pub fn text(&self) -> &EntityText {
        &self.text
    }

    /// Whether Minecraft should render the name without targeting the entity.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::is_visible` for the canonical contract."]
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::refresh_policy` for the canonical contract."]
    #[must_use]
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh
    }

    /// Validate refresh scheduling with archetype context.
    #[doc = "**API Contract:** Run `sand api show sand::entity::NameBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, NativePropertyKey::Name)
    }
}

/// A validated entity tag.
///
/// This is intentionally not an `Into<String>` API: whitespace, command
/// delimiters, empty names, and values beyond vanilla's 1024-byte limit are
/// rejected before command generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityTag(String);

impl EntityTag {
    /// Validate and construct an entity tag.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTag::new` for the canonical contract."]
    pub fn new(value: &str) -> Result<Self, PropertyNameError> {
        validate_token(value, 1024, "entity tag")?;
        Ok(Self(value.to_owned()))
    }

    pub(crate) fn generated(value: String) -> Self {
        debug_assert!(validate_token(&value, 1024, "entity tag").is_ok());
        Self(value)
    }

    /// Validated command token.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTag::as_str` for the canonical contract."]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated scoreboard team name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityTeam(String);

impl EntityTeam {
    /// Validate and construct a team name.
    ///
    /// Vanilla team names are limited to 16 bytes.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTeam::new` for the canonical contract."]
    pub fn new(value: &str) -> Result<Self, PropertyNameError> {
        validate_token(value, 16, "team")?;
        Ok(Self(value.to_owned()))
    }

    /// Validated command token.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityTeam::as_str` for the canonical contract."]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A typed archetype-owned entity-tag declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagBinding {
    tag: EntityTag,
    ownership: OwnershipPolicy,
    refresh: RefreshPolicy,
}

impl TagBinding {
    /// Add a validated tag during initialization.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::new` for the canonical contract."]
    #[must_use]
    pub fn new(tag: EntityTag) -> Self {
        Self {
            tag,
            ownership: OwnershipPolicy::InitializeMissing,
            refresh: RefreshPolicy::Initialize,
        }
    }

    /// Set whether reconciliation preserves, observes, or enforces this tag.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Validated tag.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::tag` for the canonical contract."]
    #[must_use]
    pub fn tag(&self) -> &EntityTag {
        &self.tag
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::TagBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::new` for the canonical contract."]
    #[must_use]
    pub fn new(team: EntityTeam) -> Self {
        Self {
            team,
            ownership: OwnershipPolicy::InitializeMissing,
            refresh: RefreshPolicy::Initialize,
        }
    }

    /// Set membership ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Target team.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::team` for the canonical contract."]
    #[must_use]
    pub fn team(&self) -> &EntityTeam {
        &self.team
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::TeamBinding::validate` for the canonical contract."]
    pub fn validate(&self, archetype: impl fmt::Display) -> Result<(), EntityDiagnostic> {
        self.refresh.validate(archetype, self.property_key())
    }
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::PropertyNameError::kind` for the canonical contract."]
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// Original rejected value.
    #[doc = "**API Contract:** Run `sand api show sand::entity::PropertyNameError::value` for the canonical contract."]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Validation failure.
    #[doc = "**API Contract:** Run `sand api show sand::entity::PropertyNameError::reason` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtProperty::path` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtProperty::wire_type` for the canonical contract."]
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

/// Typed value accepted by a stable native-NBT binding.
///
/// Decimal values use explicit fixed-point units to keep authoring and export
/// deterministic and to exclude NaN/infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityNbtValue {
    /// Boolean byte.
    Boolean(bool),
    /// Signed integer value.
    Integer(i32),
    /// Decimal represented as `units / scale`.
    Fixed { units: i64, scale: u32 },
}

impl EntityNbtValue {
    /// Construct a fixed-point decimal, rejecting a zero scale.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtValue::fixed` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::ownership` for the canonical contract."]
    #[must_use]
    pub fn ownership(mut self, ownership: OwnershipPolicy) -> Self {
        self.ownership = ownership;
        self
    }

    /// Set refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::refresh` for the canonical contract."]
    #[must_use]
    pub fn refresh(mut self, refresh: RefreshPolicy) -> Self {
        self.refresh = refresh;
        self
    }

    /// Stable NBT property.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::property` for the canonical contract."]
    #[must_use]
    pub const fn property(&self) -> EntityNbtProperty {
        self.property
    }

    /// Typed NBT value.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::value` for the canonical contract."]
    #[must_use]
    pub fn value(&self) -> &EntityNbtValue {
        &self.value
    }

    /// Selected ownership behavior.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::ownership_policy` for the canonical contract."]
    #[must_use]
    pub const fn ownership_policy(&self) -> OwnershipPolicy {
        self.ownership
    }

    /// Selected refresh scheduling.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::refresh_policy` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityNbtBinding::validate_for` for the canonical contract."]
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

/// Whether a raw property declaration only reads or may mutate NBT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawPropertyAccess {
    /// Read-only observation.
    ReadOnly,
    /// Native entity-NBT mutation.
    Mutable,
}

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
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityProperty::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityProperty::path` for the canonical contract."]
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Declared wire type.
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityProperty::wire_type` for the canonical contract."]
    #[must_use]
    pub const fn wire_type(&self) -> EntityNbtType {
        self.wire_type
    }

    /// Declared access mode.
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityProperty::access` for the canonical contract."]
    #[must_use]
    pub const fn access(&self) -> RawPropertyAccess {
        self.access
    }

    /// Validate player safety for a statically known entity kind.
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityProperty::validate_for` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityStateField::new` for the canonical contract."]
    pub fn new(name: &str, backend: RawStateBackend) -> Result<Self, PropertyNameError> {
        validate_token(name, 64, "raw entity state field")?;
        Ok(Self {
            name: name.to_owned(),
            backend,
        })
    }

    /// Caller-owned logical field name.
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityStateField::name` for the canonical contract."]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Explicit persistence backend.
    #[doc = "**API Contract:** Run `sand api show sand::entity::RawEntityStateField::backend` for the canonical contract."]
    #[must_use]
    pub const fn backend(&self) -> RawStateBackend {
        self.backend
    }
}

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
