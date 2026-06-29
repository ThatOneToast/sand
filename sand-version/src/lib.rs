#![forbid(unsafe_code)]

//! Shared Minecraft version anchors used by Sand crates that cannot depend on
//! `sand-core` without creating build-time dependency cycles.

/// The latest Minecraft version Sand's bundled version table was verified against.
pub const LATEST_KNOWN: &str = "26.2";
