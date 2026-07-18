//! Typed participant reliability, availability, and context-capability
//! model (#230 Phase 8).
//!
//! This module answers, in typed APIs, which participants an event *can*
//! expose and how strong the evidence for each one is — it does not
//! implement participant recovery itself. There is no attacker/victim
//! observation backend, no interacted-entity correlation, no
//! projectile-owner recovery, and no bounded observation tracker anywhere
//! in this module; those are Phase 9's work (#230). What exists here is
//! the vocabulary and validation surface Phase 9 providers will populate:
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
//!
//! See `docs/event-context.md` for the full contract and
//! `ai/known-limitations.md` (`LIM-CTX-001`..) for what remains
//! architecture-only.

pub mod availability;
pub mod capabilities;
pub mod lifetime;
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
pub use reference::{EntityParticipant, ParticipantReliabilityError, PlayerParticipant};
pub use reliability::{ItemEvidenceQualifier, ParticipantReliability};
pub use role::{EntityParticipantRole, ItemParticipantRole, LocationParticipantRole};
