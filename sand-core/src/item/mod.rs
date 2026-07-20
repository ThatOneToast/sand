//! Typed item locations and immutable event-time item snapshots (#229).
//!
//! This module provides the shared item-abstraction layer participant-rich
//! event contexts (#230) build on:
//!
//! - [`location::ItemLocation`] — *where* an item lives (a hand, an
//!   equipment slot, an inventory index, a container slot, an item entity),
//!   typed and validated, never a raw slot string.
//! - [`snapshot::ItemSnapshot`] — *what was captured* from a location at a
//!   specific generated-command boundary: immutable, storage-backed data
//!   with an explicit [`snapshot::SnapshotReliability`] and lifetime
//!   contract, never a live re-read of the (possibly already mutated)
//!   source.
//!
//! See each module's own doc comment for the full capture-ordering,
//! reliability, absence, lifetime, and concurrency contract.
//! [`crate::participant::EventParticipantPlan::observe_weapon`]/
//! `observe_held_item` build on [`snapshot::ItemSnapshot::capture`] to wire
//! held-item roles into the #230 participant-context model automatically.

pub mod location;
pub mod snapshot;

pub use location::{ContainerIndex, HotbarIndex, InventoryIndex, ItemLocation, ItemLocationError};
pub use snapshot::{
    EventItem, ItemRole, ItemSnapshot, SnapshotAbsence, SnapshotError, SnapshotReliability,
    SnapshotSchema,
};
