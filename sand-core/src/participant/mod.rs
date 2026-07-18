//! Typed participant reliability, availability, and context-capability
//! model (#230 Phase 8).
//!
//! This module answers, in typed APIs, which participants an event *can*
//! expose and how strong the evidence for each one is. Phase 8 established
//! the vocabulary and validation surface; Phase 9
//! ([`observation::observe_correlated_attacker`]) adds the first real
//! participant-recovery backend on top of it — a narrow one, covering only
//! vanilla's `execute on attacker` relation. There is still no
//! interacted-entity correlation, no projectile-owner recovery, and no
//! proximity/heuristic (`Inferred`) observer anywhere in this module; those
//! remain future work. What exists here:
//!
//! - [`reliability::ParticipantReliability`] — how strong the evidence is.
//! - [`availability::ParticipantAvailability`] /
//!   [`availability::ParticipantUnavailableReason`] — whether a participant
//!   could be supplied at all, and why not.
//! - [`lifetime::ParticipantLifetime`] — how long a reference stays
//!   meaningful in generated-command terms.
//! - [`role::EntityParticipantRole`] / [`role::LocationParticipantRole`] /
//!   [`role::ItemParticipantRole`] (a re-export of Phase 7's
//!   [`crate::item::ItemRole`]) — what a participant *is* in the event.
//! - [`reference::PlayerParticipant`] / [`reference::EntityParticipant`] —
//!   typed, command-building references carrying their own reliability and
//!   lifetime, with `require_exact`/`require` gates.
//! - [`capabilities::EventContextCapabilities`] — the deterministic,
//!   `'static` descriptor of what an event type promises, plus the
//!   propagation/merge rules for `after`/`after_any`/`after_all`/`while`/
//!   `within`/advancement bridges.
//! - [`observation::observe_correlated_attacker`] — embeds a correlated
//!   `Attacker`/`Killer`-role [`EntityParticipant`] into a generated
//!   command sequence, backed by vanilla's `execute on attacker` relation.
//! - [`plan::EventParticipantPlan`] (#230 Phase 10) — a declarative way for
//!   an event definition to state which participant observations it needs
//!   (`SandEvent::participants()`), applied to an `EventSetup` with one
//!   call (`EventSetup::with_participants`) instead of manually sequencing
//!   `observe_correlated_attacker`'s commands.
//!
//! See `docs/event-context.md` for the full contract and
//! `ai/known-limitations.md` (`LIM-CTX-001`.., `LIM-VAL-010`) for what
//! remains architecture-only.

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
pub use role::{EntityParticipantRole, ItemParticipantRole, LocationParticipantRole};
