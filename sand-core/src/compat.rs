//! Backwards-compatible exports kept for older Sand code.
//!
//! New code should prefer [`crate::prelude`] for normal authoring and
//! [`crate::advanced`] for lower-level integrations. This module exists as the
//! named home for APIs that remain available for source compatibility while the
//! public API tiers are documented more explicitly.

#[allow(deprecated)]
pub use crate::event::Event as TypedEvent;
