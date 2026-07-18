//! Typed item locations and immutable event-time item snapshots (#229 Phase 7).
//!
//! This module completes the shared item-abstraction layer needed by future
//! participant-rich event contexts (#230) without implementing them:
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
//! See `docs/items.md` for the full capture-ordering, reliability, absence,
//! lifetime, and concurrency contract, and `ai/known-limitations.md`
//! (`LIM-ITEM-001`..) for what remains unverified or explicitly deferred to
//! #230.

pub mod location;
pub mod snapshot;

pub use location::{ContainerIndex, HotbarIndex, InventoryIndex, ItemLocation, ItemLocationError};
pub use snapshot::{
    EventItem, ItemRole, ItemSnapshot, SnapshotAbsence, SnapshotError, SnapshotReliability,
    SnapshotSchema,
};
