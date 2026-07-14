//! Entity-kind markers distinguishing players from generic entities in the
//! typed query/context API (issue #227).

use std::fmt;

/// Marker for an "any entity" (`@e`-rooted) query/context kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnyEntity;

/// Marker for a player-only (`@a`-rooted) query/context kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerKind;

/// A query/context entity kind.
///
/// Sealed: only [`AnyEntity`] and [`PlayerKind`] implement it today.
/// Capability-specific kinds (living entities, individual mob types) are
/// follow-up work — see issue #228, which builds entity operations and
/// blueprints on top of this foundation.
pub trait EntityKind: sealed::Sealed + fmt::Debug + Clone + Copy + Default + 'static {
    /// Short label used in generated function paths and diagnostics.
    const LABEL: &'static str;
}

impl EntityKind for AnyEntity {
    const LABEL: &'static str = "entity";
}

impl EntityKind for PlayerKind {
    const LABEL: &'static str = "player";
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::AnyEntity {}
    impl Sealed for super::PlayerKind {}
}
