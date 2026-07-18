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
//!             .with_participants::<Self>(Self::participants(), "mypack:observations", &profile)
//!             .expect("target version supports the declared participants")
//!     }
//! }
//! ```
//!
//! This is **not** fully macro-transparent: `setup()` still calls
//! `.with_participants(...)` itself — `#[event]`/the tick coordinator do
//! not inspect `participants()` automatically. What it removes is the need
//! to hand-assemble the observation's reset/mark/cleanup command sequence
//! and get the pre/post-observation split right; `SandEvent::participants()`
//! is a genuine, additive default trait method (existing `SandEvent` impls
//! are unaffected — the default returns [`EventParticipantPlan::none`]).
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
use crate::participant::capabilities::{CapabilityOccurrence, EntityParticipantCapability};
use crate::participant::lifetime::ParticipantLifetime;
use crate::participant::observation::{
    self, CorrelationEvidence, ObservationError, ObservationSchema,
};
use crate::participant::reliability::ParticipantReliability;
use crate::participant::role::EntityParticipantRole;
use crate::version::VersionProfile;

/// One declared observation within a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlanEntry {
    role: EntityParticipantRole,
    source: PlanSource,
}

/// The observation mechanism backing a plan entry. Currently only
/// correlated attacker observation — see
/// `sand-core/src/participant/observation.rs`'s module doc for why no
/// other mechanism exists yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlanSource {
    CorrelatedAttacker,
}

/// A duplicate role was declared within one plan.
///
/// Declaring the same [`EntityParticipantRole`] twice in one
/// [`EventParticipantPlan`] is always a bug — a role either has one
/// observation or none, never two competing ones — so it is rejected at
/// plan-validation time rather than silently keeping the first or last
/// declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateParticipantRole {
    pub role: EntityParticipantRole,
}

impl std::fmt::Display for DuplicateParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "participant role {:?} is declared more than once in one EventParticipantPlan",
            self.role
        )
    }
}

impl std::error::Error for DuplicateParticipantRole {}

/// Either half of applying a plan failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventParticipantPlanError {
    DuplicateRole(DuplicateParticipantRole),
    Observation(ObservationError),
}

impl std::fmt::Display for EventParticipantPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRole(err) => err.fmt(f),
            Self::Observation(err) => err.fmt(f),
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

/// A deterministic, statically-inspectable declaration of which
/// participant observations an event needs, separate from any runtime
/// participant value.
///
/// See the module doc for the full lifecycle and application contract.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventParticipantPlan {
    entries: Vec<PlanEntry>,
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
        self.entries.is_empty()
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

    fn observe_correlated_attacker_as(mut self, role: EntityParticipantRole) -> Self {
        self.entries.push(PlanEntry {
            role,
            source: PlanSource::CorrelatedAttacker,
        });
        self
    }

    /// Reject a plan that declares the same role more than once.
    pub fn validate(&self) -> Result<(), DuplicateParticipantRole> {
        let mut seen = BTreeSet::new();
        for entry in &self.entries {
            if !seen.insert(entry.role) {
                return Err(DuplicateParticipantRole { role: entry.role });
            }
        }
        Ok(())
    }

    /// The [`EntityParticipantCapability`] entries this plan contributes to
    /// an [`crate::participant::capabilities::EventContextCapabilities`]
    /// descriptor — see
    /// `EventContextCapabilities::for_event_with_participants`.
    pub fn capabilities(&self) -> Vec<EntityParticipantCapability> {
        self.entries
            .iter()
            .map(|entry| match entry.source {
                PlanSource::CorrelatedAttacker => EntityParticipantCapability {
                    role: entry.role,
                    reliability: ParticipantReliability::Correlated,
                    lifetime: ParticipantLifetime::SynchronousDescendants,
                    occurrence: CapabilityOccurrence::OccurrenceDependent,
                    min_version: Some(CorrelationEvidence::ATTACKER_RELATION.min_version),
                },
            })
            .collect()
    }

    /// Generate this plan's setup (reset + mark/bind) and cleanup command
    /// sequences for the given event type and target profile.
    fn build(
        &self,
        event_label: &str,
        storage: &str,
        profile: &VersionProfile,
    ) -> Result<(Vec<String>, Vec<String>), EventParticipantPlanError> {
        self.validate()?;
        let mut setup_commands = Vec::new();
        let mut cleanup_commands = Vec::new();
        for entry in &self.entries {
            match entry.source {
                PlanSource::CorrelatedAttacker => {
                    let schema = ObservationSchema::new(storage.to_string(), event_label);
                    let (commands, observation) =
                        observation::attacker_observation_setup(profile, schema, entry.role)?;
                    setup_commands.extend(commands);
                    cleanup_commands.extend(observation.cleanup_commands());
                }
            }
        }
        Ok((setup_commands, cleanup_commands))
    }
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
    /// A no-op (returns `self` unchanged, `Ok`) when `plan.is_empty()`.
    pub fn with_participants<E: SandEvent + 'static>(
        mut self,
        plan: EventParticipantPlan,
        storage: impl Into<String>,
        profile: &VersionProfile,
    ) -> Result<Self, EventParticipantPlanError> {
        if plan.is_empty() {
            return Ok(self);
        }
        let (setup_commands, cleanup_commands) =
            plan.build(std::any::type_name::<E>(), &storage.into(), profile)?;
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
        assert!(plan.capabilities().is_empty());
        assert_eq!(plan.validate(), Ok(()));
    }

    #[test]
    fn single_attacker_declaration_is_valid() {
        let plan = EventParticipantPlan::new().observe_correlated_attacker();
        assert!(!plan.is_empty());
        assert_eq!(plan.validate(), Ok(()));
        let caps = plan.capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].role, EntityParticipantRole::Attacker);
        assert_eq!(caps[0].reliability, ParticipantReliability::Correlated);
        assert_eq!(
            caps[0].lifetime,
            ParticipantLifetime::SynchronousDescendants
        );
        assert_eq!(
            caps[0].occurrence,
            CapabilityOccurrence::OccurrenceDependent
        );
        assert_eq!(caps[0].min_version, Some((1, 20, 2)));
    }

    #[test]
    fn duplicate_role_declaration_is_rejected() {
        let plan = EventParticipantPlan::new()
            .observe_correlated_attacker()
            .observe_correlated_attacker();
        assert_eq!(
            plan.validate(),
            Err(DuplicateParticipantRole {
                role: EntityParticipantRole::Attacker
            })
        );
    }

    #[test]
    fn distinct_roles_from_the_same_mechanism_are_allowed() {
        let plan = EventParticipantPlan::new()
            .observe_correlated_attacker()
            .observe_correlated_killer();
        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(plan.capabilities().len(), 2);
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
            .with_participants::<TestEvent>(plan, "mypack:observations", &profile("1.21.4"))
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
            .with_participants::<TestEvent>(
                EventParticipantPlan::none(),
                "mypack:observations",
                &profile("1.21.4"),
            )
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
        let result = EventSetup::none().with_participants::<TestEvent>(
            plan,
            "mypack:observations",
            &profile("1.19.4"),
        );
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
        let result = EventSetup::none().with_participants::<TestEvent>(
            plan,
            "mypack:observations",
            &profile("1.21.4"),
        );
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
        // what resolves the same-schema-reentrancy caveat (LIM-CTX-005)
        // for the plan API specifically (the manual
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
            .with_participants::<FirstEvent>(plan(), "sharedpack:observations", &profile("1.21.4"))
            .unwrap();
        let second = EventSetup::none()
            .with_participants::<SecondEvent>(plan(), "sharedpack:observations", &profile("1.21.4"))
            .unwrap();

        assert_ne!(
            first.pre_observation, second.pre_observation,
            "distinct event types must generate distinct schema keys even with identical storage and role"
        );
    }
}
