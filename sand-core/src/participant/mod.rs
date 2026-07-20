//! Typed participant reliability, availability, and context-capability
//! model (#230).
//!
//! This module answers, in typed APIs, which participants an event *can*
//! expose and how strong the evidence for each one is. What exists here:
//!
//! - [`reliability::ParticipantReliability`] — how strong the evidence is.
//! - [`availability::ParticipantAvailability`] /
//!   [`availability::ParticipantUnavailableReason`] — whether a participant
//!   could be supplied at all, and why not.
//! - [`lifetime::ParticipantLifetime`] — how long a reference stays
//!   meaningful in generated-command terms.
//! - [`role::EntityParticipantRole`] / [`role::LocationParticipantRole`] /
//!   [`role::ItemParticipantRole`] (a re-export of Phase 7's
//!   [`crate::item::ItemRole`]) / [`role::ParticipantHand`] — what a
//!   participant *is* in the event.
//! - [`reference::PlayerParticipant`] / [`reference::EntityParticipant`] —
//!   typed, command-building references carrying their own reliability and
//!   lifetime, with `require_exact`/`require` gates.
//! - [`capabilities::EventContextCapabilities`] — the deterministic,
//!   `'static` descriptor of what an event type's *own* dispatch shape
//!   promises, plus [`capabilities::EventContextCapabilities::for_event_with_participants`]
//!   (which additionally folds in a declared [`plan::EventParticipantPlan`]),
//!   and the propagation/merge rules for `after`/`after_any`/`after_all`/
//!   `while`/`within`/advancement bridges (subject-only in the base
//!   functions; [`capabilities::full`] extends the same rules to declared
//!   entity/item participants as capability bookkeeping — see that module's
//!   doc for the boundary between capability bookkeeping and actual
//!   command-level value propagation, which remains unimplemented).
//! - [`observation::observe_correlated_attacker`] — embeds a correlated
//!   `Attacker`/`Killer`-role [`EntityParticipant`] into a generated
//!   command sequence, backed by vanilla's `execute on attacker` relation.
//!   The only entity observation backend implemented — see
//!   `docs/testing/participant-role-evidence.md` for the full role-by-role
//!   evidence audit and why every other entity role (victim, direct
//!   attacker, interacted entity, projectile, projectile owner) is
//!   currently `Unavailable` rather than guessed.
//! - [`plan::EventParticipantPlan`] — a declarative way for an event
//!   definition to state which participant observations it needs
//!   (`SandEvent::participants()`/`AdvancementEvent::participants()`).
//!   Advancement-backed events (`AdvancementEvent::participants`) are
//!   applied automatically by the export pipeline; tick-dispatch events
//!   (`SandEvent::participants`) still apply it via
//!   `EventSetup::with_participants` from their own `setup()`. Also covers
//!   held-item snapshots (`observe_weapon`/`observe_held_item`), backed by
//!   [`crate::item::ItemSnapshot`].
//!
//! See `docs/testing/participant-role-evidence.md` for the full role
//! support matrix (backend, reliability, evidence, unavailable reasons) and
//! `book/src/09-events.md`/`book/src/reference/vanilla-limitations.md` for
//! the user-facing summary.

pub mod availability;
pub mod capabilities;
pub mod lifetime;
pub mod observation;
pub mod plan;
pub mod reference;
pub mod reliability;
pub mod role;

pub use availability::{ParticipantAvailability, ParticipantUnavailableReason};
pub use capabilities::{
    CapabilityDescriptorError, CapabilityOccurrence, ContextMergeError,
    EntityParticipantCapability, EventContextCapabilities, ItemParticipantCapability,
    LocationParticipantCapability, ResolvedEventContextCapabilities, SubjectCapability,
    SubjectScope, VersionFloor,
};
pub use lifetime::ParticipantLifetime;
pub use observation::{
    CorrelatedEntityObservation, CorrelationEvidence, CorrelationSource, ObservationError,
    ObservationSchema, observe_correlated_attacker,
};
pub use plan::{DuplicateParticipantRole, EventParticipantPlan, EventParticipantPlanError};
pub use reference::{EntityParticipant, ParticipantReliabilityError, PlayerParticipant};
pub use reliability::{ItemEvidenceQualifier, ParticipantReliability};
pub use role::{
    EntityParticipantRole, ItemParticipantRole, LocationParticipantRole, ParticipantHand,
};
