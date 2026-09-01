//! Optional custom hooks for generated State component lifecycle work.

use crate::entity::{AnyEntity, EntityContext, EntityState, PlayerContext};

/// Context supplied while a component is initialized.
///
/// **API Contract:** Run `sand api show sand::state::StateInit`.
#[derive(Debug, Clone, Copy)]
pub struct StateInit {
    holder: &'static str,
}
/// Context supplied for one eligible component tick.
///
/// **API Contract:** Run `sand api show sand::state::StateTick`.
#[derive(Debug, Clone, Copy)]
pub struct StateTick {
    holder: &'static str,
}
/// Context supplied while owned runtime state is reconciled.
///
/// **API Contract:** Run `sand api show sand::state::StateReconcile`.
#[derive(Debug, Clone, Copy)]
pub struct StateReconcile {
    holder: &'static str,
}
/// Context supplied before component-owned values are removed.
///
/// **API Contract:** Run `sand api show sand::state::StateCleanup`.
#[derive(Debug, Clone, Copy)]
pub struct StateCleanup {
    holder: &'static str,
}

impl StateInit {
    /// Constructs generated initialization context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateInit::new`.
    pub const fn new(holder: &'static str) -> Self {
        Self { holder }
    }
    /// Score holder used by initialization.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateInit::holder`.
    #[must_use]
    pub const fn holder(self) -> &'static str {
        self.holder
    }
    /// Typed current-entity context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateInit::entity`.
    #[must_use]
    pub fn entity(self) -> EntityContext<AnyEntity> {
        EntityContext::default()
    }
    /// Typed current-player context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateInit::player`.
    #[must_use]
    pub fn player(self) -> PlayerContext {
        PlayerContext::default()
    }
}
impl StateTick {
    /// Constructs generated tick context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateTick::new`.
    pub const fn new(holder: &'static str) -> Self {
        Self { holder }
    }
    /// Score holder used by ticking.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateTick::holder`.
    #[must_use]
    pub const fn holder(self) -> &'static str {
        self.holder
    }
    /// Typed current-entity context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateTick::entity`.
    #[must_use]
    pub fn entity(self) -> EntityContext<AnyEntity> {
        EntityContext::default()
    }
    /// Typed current-player context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateTick::player`.
    #[must_use]
    pub fn player(self) -> PlayerContext {
        PlayerContext::default()
    }
}
impl StateReconcile {
    /// Constructs generated reconciliation context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateReconcile::new`.
    pub const fn new(holder: &'static str) -> Self {
        Self { holder }
    }
    /// Score holder used by reconciliation.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateReconcile::holder`.
    #[must_use]
    pub const fn holder(self) -> &'static str {
        self.holder
    }
    /// Typed current-entity context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateReconcile::entity`.
    #[must_use]
    pub fn entity(self) -> EntityContext<AnyEntity> {
        EntityContext::default()
    }
    /// Typed current-player context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateReconcile::player`.
    #[must_use]
    pub fn player(self) -> PlayerContext {
        PlayerContext::default()
    }
}
impl StateCleanup {
    /// Constructs generated cleanup context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateCleanup::new`.
    pub const fn new(holder: &'static str) -> Self {
        Self { holder }
    }
    /// Score holder used by cleanup.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateCleanup::holder`.
    #[must_use]
    pub const fn holder(self) -> &'static str {
        self.holder
    }
    /// Typed current-entity context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateCleanup::entity`.
    #[must_use]
    pub fn entity(self) -> EntityContext<AnyEntity> {
        EntityContext::default()
    }
    /// Typed current-player context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateCleanup::player`.
    #[must_use]
    pub fn player(self) -> PlayerContext {
        PlayerContext::default()
    }
}

/// Context supplied while shared component dependencies are provisioned.
///
/// **API Contract:** Run `sand api show sand::state::StateProvision`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StateProvision;

/// Context supplied for one declared component migration transition.
///
/// **API Contract:** Run `sand api show sand::state::StateMigrate`.
#[derive(Debug, Clone, Copy)]
pub struct StateMigrate {
    holder: &'static str,
    from: u32,
    to: u32,
}

impl StateMigrate {
    /// Constructs generated migration context.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateMigrate::new`.
    pub const fn new(holder: &'static str, from: u32, to: u32) -> Self {
        Self { holder, from, to }
    }

    /// Current score holder.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateMigrate::holder`.
    #[must_use]
    pub const fn holder(self) -> &'static str {
        self.holder
    }

    /// Source version for this transition.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateMigrate::from`.
    #[must_use]
    pub const fn from(self) -> u32 {
        self.from
    }

    /// Destination version for this transition.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateMigrate::to`.
    #[must_use]
    pub const fn to(self) -> u32 {
        self.to
    }
}

/// Optional custom behavior around an automatically generated State lifecycle.
///
/// Every hook defaults to no commands. Implementations are registered with
/// `#[state_lifecycle]`; State's generated provisioning, initialization,
/// version publication, and ownership-safe cleanup remain authoritative.
///
/// **API Contract:** Run `sand api show sand::state::StateLifecycle`.
pub trait StateLifecycle: EntityState {
    /// Add shared load-time dependencies beyond inferred field backends.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateLifecycle::provision`.
    fn provision(_ctx: StateProvision) -> Vec<String> {
        Vec::new()
    }

    /// Run after missing owned values are initialized and before presence is published.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateLifecycle::initialize`.
    fn initialize(_ctx: StateInit) -> Vec<String> {
        Vec::new()
    }

    /// Run once for each eligible loaded owner at the planned cadence.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateLifecycle::tick`.
    fn tick(_ctx: StateTick) -> Vec<String> {
        Vec::new()
    }

    /// Reconcile component-owned native properties.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateLifecycle::reconcile`.
    fn reconcile(_ctx: StateReconcile) -> Vec<String> {
        Vec::new()
    }

    /// Run one explicitly registered version transition.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateLifecycle::migrate`.
    fn migrate(_ctx: StateMigrate) -> Vec<String> {
        Vec::new()
    }

    /// Run before component-owned values and bookkeeping are removed.
    ///
    /// **API Contract:** Run `sand api show sand::state::StateLifecycle::cleanup`.
    fn cleanup(_ctx: StateCleanup) -> Vec<String> {
        Vec::new()
    }
}
