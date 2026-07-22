//! Declarative event participant plans (#230 Phase 10).
//!
//! [`EventParticipantPlan`] lets an event definition *declare* which
//! participant observations it needs, instead of an author manually
//! sequencing [`observe_correlated_attacker`](super::observation::observe_correlated_attacker)'s
//! reset/mark/cleanup commands into [`crate::events::EventSetup`] by hand
//! (the Phase 9 pattern, still available and unchanged). A plan is applied
//! with one call:
//!
//! ```rust,ignore
//! impl SandEvent for HurtEvent {
//!     fn participants() -> EventParticipantPlan {
//!         EventParticipantPlan::new().observe_correlated_attacker()
//!     }
//!
//!     fn setup() -> EventSetup {
//!         EventSetup::none()
//!             .with_participants::<Self>(Self::participants(), &profile)
//!             .expect("target version supports the declared participants")
//!     }
//! }
//! ```
//!
//! For **advancement-backed** events (`AdvancementEvent::participants`,
//! #230), the export pipeline applies the plan automatically — no
//! `setup()`/`with_participants` call needed at all; see
//! `sand-core/src/event/mod.rs`'s `AdvancementEvent::participants` doc. The
//! `SandEvent::participants()`/`with_participants` path above remains for
//! tick-dispatch events, where `setup()` is still the author-defined
//! integration point (`#[event]`/the tick coordinator do not inspect
//! `participants()` automatically for this dispatch kind). Either way,
//! `participants()` is a genuine, additive default trait method — existing
//! implementations are unaffected, since the default returns
//! [`EventParticipantPlan::none`].
//!
//! # Lifecycle ordering
//!
//! For a tick-backed event, one generated check function runs, in order:
//! objectives are load-time only and don't participate here; then
//! `pre_observation` (existing setup commands, then the plan's
//! reset+mark/bind commands, via `.extend()` — appended, not prepended,
//! so any existing prerequisite commands run first); then the condition
//! test; then the handler dispatch and its synchronous descendants (all
//! still inside the same generated function, so the plan's temporary tag
//! is still present); then `post_observation` (existing commands, then the
//! plan's cleanup commands, again appended at the end).
//!
//! Cleanup runs **after** existing `post_observation` commands, not
//! before: `post_observation` always runs regardless of whether the
//! condition matched this tick (see [`crate::events::TickEventDispatch`]),
//! so it is the correct place for unconditional cleanup, and placing it
//! last means any legitimate user `post_observation` logic still has
//! access to the observed participant's tag before it's removed. Because
//! Sand generates straight-line command sequences (no exception unwinding,
//! no early return), cleanup is structurally unavoidable — it runs whether
//! the participant was present or absent, whether the condition matched or
//! not, and whether the handler ran or not, exactly as
//! [`crate::item::ItemSnapshot`]'s own cleanup contract already documents
//! for the same reason.

use std::collections::BTreeSet;

use crate::events::{EventSetup, SandEvent};
use crate::item::location::ItemLocation;
use crate::item::snapshot::{ItemSnapshot, SnapshotError, SnapshotReliability, SnapshotSchema};
use crate::participant::availability::{ParticipantAvailability, ParticipantUnavailableReason};
use crate::participant::lifetime::ParticipantLifetime;
use crate::participant::observation::{self, ObservationError, ObservationSchema};
use crate::participant::reference::EntityParticipant;
use crate::participant::role::{EntityParticipantRole, ItemParticipantRole, ParticipantHand};
use crate::version::VersionProfile;
use sand_commands::selector::SingleEntity;

/// The fixed, Sand-owned storage location every [`EventParticipantPlan`]
/// observation uses, regardless of the exporting pack's own namespace —
/// the same convention `__sand_local:` predicates use for other
/// Sand-generated resources. Using one constant location (rather than a
/// caller-supplied namespace) is what lets [`EventParticipantPlan::resolve`]
/// reconstruct the exact selector a plan's setup commands generated, given
/// only the event type — a Rust-level accessor has no access to the
/// exporting pack's namespace at the point a handler body calls it.
pub(crate) const PARTICIPANT_STORAGE: &str = "sand:__participants";

/// One declared entity observation within a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlanEntry {
    role: EntityParticipantRole,
    source: PlanSource,
}

/// The observation mechanism backing an entity plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlanSource {
    /// Directly captured by this event's own setup — see
    /// `sand-core/src/participant/observation.rs`'s module doc for why no
    /// other direct-capture mechanism exists yet.
    CorrelatedAttacker,
    /// Borrowed, unchanged, from `source_event_label`'s own same-cycle
    /// capture of the same role (#264) — see
    /// [`EventParticipantPlan::inherit_entity`]. Contributes zero setup or
    /// cleanup commands: the source event fully owns both, and this event
    /// only ever runs within the source's synchronous descendant call
    /// tree, so the source's temporary tag is already present the entire
    /// time this event's own handlers run.
    Inherited { source_event_label: &'static str },
}

/// One declared held-item snapshot within a plan. Always exact — see
/// [`ParticipantHand`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ItemPlanEntry {
    role: ItemParticipantRole,
    source: ItemPlanSource,
}

/// The capture mechanism backing an item plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemPlanSource {
    /// Directly captured by this event's own setup, from `hand`.
    Hand(ParticipantHand),
    /// Borrowed, unchanged, from `source_event_label`'s own same-cycle
    /// capture of the same role/hand (#264) — see
    /// [`EventParticipantPlan::inherit_item`]. Contributes zero setup
    /// commands: the source event's capture already ran.
    Inherited {
        source_event_label: &'static str,
        hand: ParticipantHand,
    },
}

impl ItemPlanEntry {
    fn hand(self) -> ParticipantHand {
        match self.source {
            ItemPlanSource::Hand(hand) => hand,
            ItemPlanSource::Inherited { hand, .. } => hand,
        }
    }

    fn location(self) -> ItemLocation {
        match self.hand() {
            ParticipantHand::MainHand => ItemLocation::PlayerMainHand,
            ParticipantHand::OffHand => ItemLocation::PlayerOffHand,
        }
    }
}

/// A duplicate role was declared within one plan.
///
/// Declaring the same [`EntityParticipantRole`] (or, for item entries, the
/// same [`ItemParticipantRole`]) twice in one [`EventParticipantPlan`] is
/// always a bug — a role either has one observation or none, never two
/// competing ones — so it is rejected at plan-validation time rather than
/// silently keeping the first or last declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateParticipantRole {
    Entity(EntityParticipantRole),
    Item(ItemParticipantRole),
}

impl std::fmt::Display for DuplicateParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entity(role) => write!(
                f,
                "entity participant role {role:?} is declared more than once in one EventParticipantPlan"
            ),
            Self::Item(role) => write!(
                f,
                "item participant role {role:?} is declared more than once in one EventParticipantPlan"
            ),
        }
    }
}

impl std::error::Error for DuplicateParticipantRole {}

/// Any part of building or applying a plan failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventParticipantPlanError {
    DuplicateRole(DuplicateParticipantRole),
    Observation(ObservationError),
    Snapshot(String),
}

impl std::fmt::Display for EventParticipantPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRole(err) => err.fmt(f),
            Self::Observation(err) => err.fmt(f),
            Self::Snapshot(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for EventParticipantPlanError {}

impl From<DuplicateParticipantRole> for EventParticipantPlanError {
    fn from(err: DuplicateParticipantRole) -> Self {
        Self::DuplicateRole(err)
    }
}

impl From<ObservationError> for EventParticipantPlanError {
    fn from(err: ObservationError) -> Self {
        Self::Observation(err)
    }
}

impl From<SnapshotError> for EventParticipantPlanError {
    fn from(err: SnapshotError) -> Self {
        Self::Snapshot(err.to_string())
    }
}

/// A deterministic, statically-inspectable declaration of which
/// participant observations an event needs, separate from any runtime
/// participant value.
///
/// See the module doc for the full lifecycle and application contract.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventParticipantPlan {
    entries: Vec<PlanEntry>,
    item_entries: Vec<ItemPlanEntry>,
}

impl EventParticipantPlan {
    /// An empty plan — the default every `SandEvent` gets unless it
    /// overrides [`crate::events::SandEvent::participants`]. See
    /// [`Self::none`] for the exact same value under an explicit name.
    pub fn new() -> Self {
        Self::default()
    }

    /// Equivalent to [`new`](Self::new) — an explicit name for the "no
    /// participants declared" plan, matching [`EventSetup::none`]'s naming.
    pub fn none() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.item_entries.is_empty()
    }

    /// Declare a correlated attacker observation for
    /// [`EntityParticipantRole::Attacker`].
    pub fn observe_correlated_attacker(self) -> Self {
        self.observe_correlated_attacker_as(EntityParticipantRole::Attacker)
    }

    /// Declare a correlated attacker observation under
    /// [`EntityParticipantRole::Killer`] instead — the identical mechanism,
    /// used for events whose semantics call the observed entity a killer
    /// rather than an attacker (e.g. a player-death event).
    pub fn observe_correlated_killer(self) -> Self {
        self.observe_correlated_attacker_as(EntityParticipantRole::Killer)
    }

    /// `pub(crate)` general form of [`Self::observe_correlated_attacker`]/
    /// [`Self::observe_correlated_killer`] — reached by [`crate::participant::builder::ParticipantBuilder::observe_entity`]
    /// so the builder can declare a direct correlated observation under any
    /// role, not just the two named shorthands.
    pub(crate) fn observe_correlated_attacker_as(mut self, role: EntityParticipantRole) -> Self {
        self.entries.push(PlanEntry {
            role,
            source: PlanSource::CorrelatedAttacker,
        });
        self
    }

    /// Declare that this event borrows `role` from `Source`'s own
    /// same-cycle capture, instead of capturing it independently (#264).
    ///
    /// Valid only when `Source` is a real ancestor of this event reachable
    /// through an unbroken chain of plain, single-parent, unbounded
    /// `.after(...)`/`chain::<...>()` edges (no `after_any`/`after_all`
    /// fan-in, no `.within(...)`, no advancement-bridge hop along the way),
    /// and `Source`'s own plan must declare `role` via direct capture, not
    /// itself via `inherit_entity` — transitive inheritance is not
    /// supported; every link in a multi-hop chain must name the actual
    /// capturing ancestor directly. Both conditions are enforced by the
    /// export pipeline, which has the full event graph available (this
    /// plan-building API does not) — see
    /// `sand-core/src/compiler/export/participant_transport.rs`. An
    /// unsatisfiable declaration fails export with an actionable
    /// diagnostic; it can never silently generate a dangling reference.
    ///
    /// Zero setup/cleanup commands are generated for this entry — `Source`
    /// fully owns both, and this event only ever runs inside `Source`'s own
    /// synchronous descendant call tree, so the borrowed reference is valid
    /// for this event's entire execution.
    ///
    /// ```rust,ignore
    /// impl SandEvent for ChildAfterDamage {
    ///     fn dispatch() -> impl Into<SandEventDispatch> {
    ///         SandEventDispatch::chain::<EntityDamagePlayerEvent>()
    ///     }
    ///     fn participants() -> EventParticipantPlan {
    ///         EventParticipantPlan::new()
    ///             .inherit_entity::<EntityDamagePlayerEvent>(EntityParticipantRole::Attacker)
    ///     }
    /// }
    /// ```
    pub fn inherit_entity<Source: crate::events::SandEvent + 'static>(
        mut self,
        role: EntityParticipantRole,
    ) -> Self {
        self.entries.push(PlanEntry {
            role,
            source: PlanSource::Inherited {
                source_event_label: std::any::type_name::<Source>(),
            },
        });
        self
    }

    /// Declare an [`ItemParticipantRole::Weapon`] snapshot captured from
    /// [`ParticipantHand::MainHand`] — the conventional assumption for
    /// melee combat (whatever the attacker is holding at the moment of the
    /// hit). Shorthand for `observe_held_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)`.
    pub fn observe_weapon(self) -> Self {
        self.observe_held_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
    }

    /// Declare a held-item snapshot for `role`, captured from `hand`.
    ///
    /// Always [`crate::participant::ParticipantReliability::ExactSnapshot`] — addressing a
    /// specific hand slot on `@s` is a directly queryable NBT path, never a
    /// correlated guess (see [`ParticipantHand`]). Which *role* label to
    /// apply (`Weapon` vs `UsedItem` vs any other [`ItemParticipantRole`])
    /// is the caller's own event-semantic judgment — this plan does not
    /// infer intent from vanilla behavior, it only captures the exact item
    /// present in the named hand.
    pub fn observe_held_item(mut self, role: ItemParticipantRole, hand: ParticipantHand) -> Self {
        self.item_entries.push(ItemPlanEntry {
            role,
            source: ItemPlanSource::Hand(hand),
        });
        self
    }

    /// The item-snapshot counterpart to [`Self::inherit_entity`] — borrows
    /// `role` (captured from `hand`) from `Source`'s own same-cycle
    /// capture instead of capturing it independently. `hand` must match
    /// the hand `Source`'s own plan declared that role from — this API
    /// does not look `Source`'s declaration up for you, so a mismatched
    /// hand resolves to a snapshot `Source` never actually captured (a
    /// distinct, always-absent snapshot handle, not a compile or export
    /// error — see the module doc's determinism note). Same export-time
    /// ancestor-chain and direct-capture validation as
    /// [`Self::inherit_entity`].
    pub fn inherit_item<Source: crate::events::SandEvent + 'static>(
        mut self,
        role: ItemParticipantRole,
        hand: ParticipantHand,
    ) -> Self {
        self.item_entries.push(ItemPlanEntry {
            role,
            source: ItemPlanSource::Inherited {
                source_event_label: std::any::type_name::<Source>(),
                hand,
            },
        });
        self
    }

    /// Reject a plan that declares the same entity or item role more than
    /// once.
    pub fn validate(&self) -> Result<(), DuplicateParticipantRole> {
        let mut seen = BTreeSet::new();
        for entry in &self.entries {
            if !seen.insert(entry.role) {
                return Err(DuplicateParticipantRole::Entity(entry.role));
            }
        }
        let mut seen = BTreeSet::new();
        for entry in &self.item_entries {
            if !seen.insert(entry.role) {
                return Err(DuplicateParticipantRole::Item(entry.role));
            }
        }
        Ok(())
    }

    /// Entity roles this plan declares via `Source`'s same-cycle capture,
    /// with the declared source event label — `pub(crate)`, consumed by
    /// the export pipeline's ancestor-chain validation
    /// (`sand-core/src/compiler/export/participant_transport.rs`), not
    /// part of the public plan-building API.
    pub(crate) fn inherited_entity_roles(&self) -> Vec<(EntityParticipantRole, &'static str)> {
        self.entries
            .iter()
            .filter_map(|entry| match entry.source {
                PlanSource::Inherited { source_event_label } => {
                    Some((entry.role, source_event_label))
                }
                _ => None,
            })
            .collect()
    }

    /// Entity roles this plan captures directly (not inherited) —
    /// `pub(crate)`, the counterpart validation needs to confirm a
    /// requested inheritance source actually owns a direct capture of the
    /// role, not itself another inherited borrow.
    pub(crate) fn direct_entity_roles(&self) -> Vec<EntityParticipantRole> {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.source, PlanSource::CorrelatedAttacker))
            .map(|entry| entry.role)
            .collect()
    }

    /// The item-role counterpart to [`Self::inherited_entity_roles`].
    pub(crate) fn inherited_item_roles(&self) -> Vec<(ItemParticipantRole, &'static str)> {
        self.item_entries
            .iter()
            .filter_map(|entry| match entry.source {
                ItemPlanSource::Inherited {
                    source_event_label, ..
                } => Some((entry.role, source_event_label)),
                _ => None,
            })
            .collect()
    }

    /// The item-role counterpart to [`Self::direct_entity_roles`].
    pub(crate) fn direct_item_roles(&self) -> Vec<ItemParticipantRole> {
        self.item_entries
            .iter()
            .filter(|entry| matches!(entry.source, ItemPlanSource::Hand(_)))
            .map(|entry| entry.role)
            .collect()
    }

    /// Generate this plan's setup (reset + mark/bind) and cleanup command
    /// sequences for the given event type and target profile, at the fixed
    /// [`PARTICIPANT_STORAGE`] location every plan uses.
    ///
    /// `pub(crate)` — reached both by [`EventSetup::with_participants`] (the
    /// tick-dispatch integration path) and by the export pipeline's
    /// automatic advancement-dispatch integration
    /// (`sand-core/src/compiler/export/pipeline.rs`).
    pub(crate) fn build(
        &self,
        event_label: &str,
        profile: &VersionProfile,
    ) -> Result<(Vec<String>, Vec<String>), EventParticipantPlanError> {
        self.validate()?;
        let mut setup_commands = Vec::new();
        let mut cleanup_commands = Vec::new();
        for entry in &self.entries {
            match entry.source {
                PlanSource::CorrelatedAttacker => {
                    let schema = ObservationSchema::new(PARTICIPANT_STORAGE, event_label);
                    let (commands, observation) =
                        observation::attacker_observation_setup(profile, schema, entry.role)?;
                    setup_commands.extend(commands);
                    cleanup_commands.extend(observation.cleanup_commands());
                }
                // Zero commands — the source event's own plan already
                // captured and owns cleanup of this binding; see
                // `inherit_entity`'s doc.
                PlanSource::Inherited { .. } => {}
            }
        }
        for entry in &self.item_entries {
            match entry.source {
                ItemPlanSource::Hand(_) => {
                    let schema = SnapshotSchema::new(
                        PARTICIPANT_STORAGE,
                        &item_entry_label(event_label, *entry),
                    );
                    let (_snapshot, commands) = ItemSnapshot::capture(
                        &entry.location(),
                        schema,
                        SnapshotReliability::ExactPostTrigger,
                    )?;
                    // Item snapshots have no cleanup step — the storage is
                    // unconditionally reset (not removed) at the start of
                    // every capture, so a stale value can never leak
                    // through to the next invocation (see
                    // `ItemSnapshot::capture`'s doc).
                    setup_commands.extend(commands);
                }
                // Zero commands — the source event's own capture already ran.
                ItemPlanSource::Inherited { .. } => {}
            }
        }
        Ok((setup_commands, cleanup_commands))
    }

    /// Reconstruct the typed participant reference this plan's setup
    /// commands bind `role` to, given the same `event_label` — without
    /// re-generating any commands.
    ///
    /// This is what [`crate::event::Event::entity`] calls so a handler body
    /// can address a declared participant without threading generated
    /// tag/storage names through user code. Returns
    /// [`ParticipantUnavailableReason::NotApplicable`] if `role` was not
    /// declared in this plan — a caller cannot self-report a stronger
    /// availability than the plan actually declared.
    ///
    /// `pub(crate)` — public handler code reaches this through the infallible
    /// [`Self::require_entity`] wrapper (or the `Event<E>`/`SandEvent`
    /// accessor sugar built on it) instead, per #273: a role that a
    /// statically-known event definition did not declare is a build-time
    /// authoring bug, not a value for ordinary handler code to branch on.
    pub(crate) fn resolve(
        &self,
        event_label: &str,
        role: EntityParticipantRole,
    ) -> ParticipantAvailability<EntityParticipant> {
        let Some(entry) = self.entries.iter().find(|entry| entry.role == role) else {
            return ParticipantAvailability::Unavailable(
                ParticipantUnavailableReason::NotApplicable,
            );
        };
        let schema_event_label = match entry.source {
            PlanSource::CorrelatedAttacker => event_label,
            // Resolve against the *source's* own key, not this event's —
            // nothing was ever captured under this event's own label for
            // an inherited entry (`build` emits zero commands for it), so
            // the only valid reconstruction reads the source's tag.
            PlanSource::Inherited {
                source_event_label, ..
            } => source_event_label,
        };
        let schema = ObservationSchema::new(PARTICIPANT_STORAGE, schema_event_label);
        ParticipantAvailability::Available(EntityParticipant::correlated(
            SingleEntity::raw(format!("@e[tag={},limit=1]", schema.tag())),
            role,
            ParticipantLifetime::SynchronousDescendants,
        ))
    }

    /// Reconstruct the typed item snapshot handle this plan's setup
    /// commands captured `role` into, given the same `event_label` —
    /// without generating any commands. See [`Self::resolve`] for the
    /// entity-role equivalent this mirrors. Returns
    /// [`ParticipantUnavailableReason::NotApplicable`] if `role` was not
    /// declared in this plan.
    ///
    /// `pub(crate)` — see [`Self::resolve`]'s doc for why public code reaches
    /// this through [`Self::require_item`] instead.
    pub(crate) fn resolve_item(
        &self,
        event_label: &str,
        role: ItemParticipantRole,
    ) -> ParticipantAvailability<ItemSnapshot> {
        let Some(entry) = self.item_entries.iter().find(|entry| entry.role == role) else {
            return ParticipantAvailability::Unavailable(
                ParticipantUnavailableReason::NotApplicable,
            );
        };
        let schema_event_label = match entry.source {
            ItemPlanSource::Hand(_) => event_label,
            // See `resolve`'s equivalent comment — an inherited entry
            // resolves against the source's own key, since that is where
            // the actual capture happened.
            ItemPlanSource::Inherited {
                source_event_label, ..
            } => source_event_label,
        };
        let schema = SnapshotSchema::new(
            PARTICIPANT_STORAGE,
            &item_entry_label(schema_event_label, *entry),
        );
        ParticipantAvailability::Available(ItemSnapshot::reconstruct(
            schema,
            entry.location().kind(),
            SnapshotReliability::ExactPostTrigger,
        ))
    }

    /// Infallible entity-participant access (#273): the public accessor
    /// surface (`Event<E>::entity`/`.attacker`/etc., and the equivalent
    /// bare-`SandEvent` accessors) resolves through this instead of
    /// [`Self::resolve`] directly, so ordinary handler code never sees a
    /// [`ParticipantAvailability`] wrapper.
    ///
    /// # Panics
    ///
    /// Panics with an actionable message if `role` was not declared on this
    /// plan. This is a fully local, immediately-knowable authoring mistake —
    /// a concrete event's `participants()` is a `fn() -> EventParticipantPlan`
    /// resolved once per event type, so whether a role is declared is a
    /// static fact about that one function, not something that depends on
    /// game state. `sand build`'s mandatory graph validation is expected to
    /// catch this earlier, before any output is written (missing/ambiguous
    /// participant access is reported there with the event, handler, and
    /// requested role); this panic is the internal safety net for the rare
    /// case a mistake reaches codegen without going through that path.
    #[track_caller]
    pub(crate) fn require_entity(
        &self,
        event_label: &str,
        role: EntityParticipantRole,
    ) -> EntityParticipant {
        match self.resolve(event_label, role) {
            ParticipantAvailability::Available(participant) => participant,
            ParticipantAvailability::Unavailable(reason) => panic!(
                "`{event_label}` does not provide the `{role:?}` entity participant ({}).\n\n\
                 This accessor requires EntityParticipantRole::{role:?}.\n\
                 Declare it directly with `ParticipantBuilder::new().observe_entity(EntityParticipantRole::{role:?})`, \
                 inherit it from an ancestor with `.inherit_entity::<Source>(EntityParticipantRole::{role:?})`, \
                 or use an event that provides it.",
                reason.description()
            ),
        }
    }

    /// The item-participant counterpart to [`Self::require_entity`].
    #[track_caller]
    pub(crate) fn require_item(
        &self,
        event_label: &str,
        role: ItemParticipantRole,
    ) -> ItemSnapshot {
        match self.resolve_item(event_label, role) {
            ParticipantAvailability::Available(snapshot) => snapshot,
            ParticipantAvailability::Unavailable(reason) => panic!(
                "`{event_label}` does not provide the `{role:?}` item participant ({}).\n\n\
                 This accessor requires ItemParticipantRole::{role:?}.\n\
                 Declare it directly with `ParticipantBuilder::new().observe_item(ItemParticipantRole::{role:?}, hand)`, \
                 inherit it from an ancestor with `.inherit_item::<Source>(ItemParticipantRole::{role:?}, hand)`, \
                 or use an event that provides it.",
                reason.description()
            ),
        }
    }
}

/// The per-item-entry storage/key label — incorporates the role and hand so
/// multiple item entries within one plan (or an item entry alongside an
/// entity entry) never derive the same [`SnapshotSchema`]/[`ObservationSchema`]
/// key from one shared `event_label`.
fn item_entry_label(event_label: &str, entry: ItemPlanEntry) -> String {
    format!("{event_label}::item::{:?}::{:?}", entry.role, entry.hand())
}

impl EventSetup {
    /// Apply `plan`'s generated commands to this setup: the plan's
    /// reset+mark/bind commands are appended to `pre_observation`, and its
    /// cleanup commands are appended to `post_observation` — see the
    /// [module doc](self) for the exact ordering contract. `E` supplies the
    /// deterministic `event_label` (via `std::any::type_name::<E>()`, the
    /// same scheme [`crate::item::ItemSnapshot`] uses) so callers never
    /// need to invent one.
    ///
    /// This is the tick-dispatch integration path — for advancement-backed
    /// `SandEvent`s (`AdvancementEvent::participants`), the export pipeline
    /// applies the plan automatically; see [`crate::event::AdvancementEvent::participants`].
    ///
    /// A no-op (returns `self` unchanged, `Ok`) when `plan.is_empty()`.
    pub fn with_participants<E: SandEvent + 'static>(
        mut self,
        plan: EventParticipantPlan,
        profile: &VersionProfile,
    ) -> Result<Self, EventParticipantPlanError> {
        if plan.is_empty() {
            return Ok(self);
        }
        let (setup_commands, cleanup_commands) = plan.build(std::any::type_name::<E>(), profile)?;
        self.pre_observation.extend(setup_commands);
        self.post_observation.extend(cleanup_commands);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::MinecraftVersion;

    fn profile(version: &str) -> VersionProfile {
        VersionProfile::resolve(&MinecraftVersion::parse(version).unwrap()).unwrap()
    }

    #[test]
    fn empty_plan_is_a_no_op() {
        let plan = EventParticipantPlan::none();
        assert!(plan.is_empty());
        assert_eq!(plan.validate(), Ok(()));
    }

    #[test]
    fn single_attacker_declaration_is_valid() {
        let plan = EventParticipantPlan::new().observe_correlated_attacker();
        assert!(!plan.is_empty());
        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(
            plan.direct_entity_roles(),
            vec![EntityParticipantRole::Attacker]
        );
        assert!(plan.inherited_entity_roles().is_empty());
    }

    #[test]
    fn duplicate_role_declaration_is_rejected() {
        let plan = EventParticipantPlan::new()
            .observe_correlated_attacker()
            .observe_correlated_attacker();
        assert_eq!(
            plan.validate(),
            Err(DuplicateParticipantRole::Entity(
                EntityParticipantRole::Attacker
            ))
        );
    }

    #[test]
    fn distinct_roles_from_the_same_mechanism_are_allowed() {
        let plan = EventParticipantPlan::new()
            .observe_correlated_attacker()
            .observe_correlated_killer();
        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(
            plan.direct_entity_roles(),
            vec![
                EntityParticipantRole::Attacker,
                EntityParticipantRole::Killer
            ]
        );
    }

    #[test]
    fn with_participants_appends_setup_and_cleanup_around_existing_commands() {
        let plan = EventParticipantPlan::new().observe_correlated_attacker();
        let setup = EventSetup {
            objectives: vec!["scoreboard objectives add p10_trigger dummy".into()],
            pre_observation: vec!["say existing pre".into()],
            post_observation: vec!["say existing post".into()],
        };
        struct TestEvent;
        impl SandEvent for TestEvent {
            fn dispatch() -> impl Into<crate::events::SandEventDispatch> {
                crate::events::SandEventDispatch::tick().as_players()
            }
        }
        let applied = setup
            .with_participants::<TestEvent>(plan, &profile("1.21.4"))
            .unwrap();

        assert_eq!(applied.pre_observation[0], "say existing pre");
        assert!(
            applied.pre_observation.len() > 1,
            "plan setup commands must be appended"
        );
        assert_eq!(applied.post_observation[0], "say existing post");
        assert!(
            applied.post_observation.len() > 1,
            "plan cleanup commands must be appended"
        );
        assert!(
            applied
                .post_observation
                .last()
                .unwrap()
                .starts_with("tag @e[tag=__sand_observed_")
        );
    }

    #[test]
    fn with_participants_is_a_no_op_for_an_empty_plan() {
        let setup = EventSetup {
            objectives: vec![],
            pre_observation: vec!["say only".into()],
            post_observation: vec![],
        };
        struct TestEvent;
        impl SandEvent for TestEvent {
            fn dispatch() -> impl Into<crate::events::SandEventDispatch> {
                crate::events::SandEventDispatch::tick().as_players()
            }
        }
        let applied = setup
            .clone()
            .with_participants::<TestEvent>(EventParticipantPlan::none(), &profile("1.21.4"))
            .unwrap();
        assert_eq!(applied, setup);
    }

    #[test]
    fn with_participants_rejects_unsupported_target_version() {
        let plan = EventParticipantPlan::new().observe_correlated_attacker();
        struct TestEvent;
        impl SandEvent for TestEvent {
            fn dispatch() -> impl Into<crate::events::SandEventDispatch> {
                crate::events::SandEventDispatch::tick().as_players()
            }
        }
        let result = EventSetup::none().with_participants::<TestEvent>(plan, &profile("1.19.4"));
        assert!(matches!(
            result,
            Err(EventParticipantPlanError::Observation(
                ObservationError::UnsupportedVersion { .. }
            ))
        ));
    }

    #[test]
    fn with_participants_rejects_duplicate_role_before_generating_commands() {
        let plan = EventParticipantPlan::new()
            .observe_correlated_attacker()
            .observe_correlated_attacker();
        struct TestEvent;
        impl SandEvent for TestEvent {
            fn dispatch() -> impl Into<crate::events::SandEventDispatch> {
                crate::events::SandEventDispatch::tick().as_players()
            }
        }
        let result = EventSetup::none().with_participants::<TestEvent>(plan, &profile("1.21.4"));
        assert!(matches!(
            result,
            Err(EventParticipantPlanError::DuplicateRole(_))
        ));
    }

    #[test]
    fn distinct_event_types_never_collide_even_with_the_same_role_and_storage() {
        // The plan API's schema key is always derived from
        // `std::any::type_name::<E>()`, never a caller-supplied string, so
        // two distinct SandEvent types applying the same plan against the
        // same storage namespace will not collide in practice — this is
        // what resolves the same-schema-reentrancy caveat documented in
        // `observation.rs`'s module doc for the plan API specifically (the manual
        // `observe_correlated_attacker` API still accepts a caller-chosen
        // event_label and retains the caveat). This relies on an unguarded
        // 32-bit FNV-1a hash, the same scheme `tick_event_resource_key`
        // uses elsewhere — a collision is astronomically unlikely, not
        // structurally impossible; there is no export-time collision
        // registry for this keyspace the way `component.rs`'s
        // `key_registry` guards the event graph's own keyspace.
        struct FirstEvent;
        impl SandEvent for FirstEvent {
            fn dispatch() -> impl Into<crate::events::SandEventDispatch> {
                crate::events::SandEventDispatch::tick().as_players()
            }
        }
        struct SecondEvent;
        impl SandEvent for SecondEvent {
            fn dispatch() -> impl Into<crate::events::SandEventDispatch> {
                crate::events::SandEventDispatch::tick().as_players()
            }
        }

        let plan = || EventParticipantPlan::new().observe_correlated_attacker();
        let first = EventSetup::none()
            .with_participants::<FirstEvent>(plan(), &profile("1.21.4"))
            .unwrap();
        let second = EventSetup::none()
            .with_participants::<SecondEvent>(plan(), &profile("1.21.4"))
            .unwrap();

        assert_ne!(
            first.pre_observation, second.pre_observation,
            "distinct event types must generate distinct schema keys even with identical storage and role"
        );
    }

    // ── Item plan entries ─────────────────────────────────────────────────

    #[test]
    fn observe_weapon_declares_mainhand_exact_snapshot() {
        let plan = EventParticipantPlan::new().observe_weapon();
        assert!(!plan.is_empty());
        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(plan.direct_item_roles(), vec![ItemParticipantRole::Weapon]);
        let (setup, _) = plan
            .build("WeaponCapabilityTestEvent", &profile("1.21.4"))
            .unwrap();
        assert!(
            setup.iter().any(|cmd| cmd.contains("SelectedItem")),
            "expected an exact mainhand snapshot capture: {setup:?}"
        );
    }

    #[test]
    fn duplicate_item_role_declaration_is_rejected() {
        let plan = EventParticipantPlan::new()
            .observe_held_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .observe_held_item(ItemParticipantRole::Weapon, ParticipantHand::OffHand);
        assert_eq!(
            plan.validate(),
            Err(DuplicateParticipantRole::Item(ItemParticipantRole::Weapon))
        );
    }

    #[test]
    fn distinct_hands_are_distinct_roles_when_labeled_differently() {
        let plan = EventParticipantPlan::new()
            .observe_held_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .observe_held_item(ItemParticipantRole::UsedItem, ParticipantHand::OffHand);
        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(
            plan.direct_item_roles(),
            vec![ItemParticipantRole::Weapon, ItemParticipantRole::UsedItem]
        );
    }

    #[test]
    fn item_plan_build_generates_capture_commands_and_no_cleanup() {
        let plan = EventParticipantPlan::new().observe_weapon();
        let (setup, cleanup) = plan.build("TestWeaponEvent", &profile("1.21.4")).unwrap();
        assert!(!setup.is_empty());
        assert!(
            setup
                .iter()
                .any(|cmd| cmd.contains("SelectedItem") || cmd.contains("data modify")),
            "expected item capture commands: {setup:?}"
        );
        assert!(
            cleanup.is_empty(),
            "item snapshots reset on every capture, they have no separate cleanup step"
        );
    }

    #[test]
    fn resolve_item_reconstructs_the_same_schema_build_used() {
        let plan = EventParticipantPlan::new().observe_weapon();
        let (setup, _) = plan
            .build("TestResolveWeaponEvent", &profile("1.21.4"))
            .unwrap();
        let resolved = plan.resolve_item("TestResolveWeaponEvent", ItemParticipantRole::Weapon);
        let ParticipantAvailability::Available(snapshot) = resolved else {
            panic!("expected the declared weapon role to resolve as available");
        };
        assert!(
            setup.iter().any(|cmd| cmd.contains(snapshot.storage())),
            "resolve_item's storage must match what build() actually captured into: {setup:?}"
        );
    }

    #[test]
    fn resolve_item_is_unavailable_for_an_undeclared_role() {
        let plan = EventParticipantPlan::new().observe_weapon();
        let resolved = plan.resolve_item("AnyEvent", ItemParticipantRole::Ammunition);
        assert_eq!(
            resolved,
            ParticipantAvailability::Unavailable(ParticipantUnavailableReason::NotApplicable)
        );
    }

    #[test]
    fn item_and_entity_entries_in_one_plan_never_collide() {
        let plan = EventParticipantPlan::new()
            .observe_correlated_attacker()
            .observe_weapon();
        let (setup, _) = plan.build("TestCombinedEvent", &profile("1.21.4")).unwrap();
        // Distinct generated identities: the attacker's tag name and the
        // weapon snapshot's storage path must not accidentally share a key.
        let attacker_marker = setup
            .iter()
            .any(|cmd| cmd.contains("__sand_observed_") || cmd.contains("execute on attacker"));
        let weapon_marker = setup
            .iter()
            .any(|cmd| cmd.contains("SelectedItem") || cmd.contains("snap."));
        assert!(
            attacker_marker,
            "expected attacker observation commands: {setup:?}"
        );
        assert!(
            weapon_marker,
            "expected weapon snapshot commands: {setup:?}"
        );
    }
}
