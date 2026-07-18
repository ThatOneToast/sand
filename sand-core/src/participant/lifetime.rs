//! Typed participant lifetime (#230 Phase 8).
//!
//! Participant references are generated-command execution concepts, not
//! Rust-owned Minecraft entities — Sand is a compiler, not a runtime agent
//! on the server. Rust's borrow checker cannot enforce how long a `@s`
//! binding or a temporary scope tag stays meaningful across generated
//! `function` calls; [`ParticipantLifetime`] documents the contract
//! instead, the same way [`ItemSnapshot`](crate::item::ItemSnapshot)'s
//! module doc documents its own (execution-scoped) lifetime.

/// How long a participant reference remains valid, in terms of generated
/// command execution rather than Rust scoping.
///
/// Declared narrowest-first so the derived [`Ord`] doubles as a
/// "how long does this reference stay meaningful" ordering:
/// `captured.covers(needed)` is `captured >= needed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParticipantLifetime {
    /// Valid only within the generated function call that captured/bound
    /// it — e.g. `@s` inside the handler body that receives it directly.
    Invocation,
    /// Valid through the capturing invocation and any same-cycle
    /// synchronous descendant graph calls it makes (direct handlers, and
    /// same-cycle chained children reached from within that call tree).
    /// This is the lifetime [`ItemSnapshot::capture`](crate::item::ItemSnapshot::capture)
    /// documents for its own storage-backed captures.
    SynchronousDescendants,
    /// Valid for the current event-cycle coordinator pass (e.g. a bounded
    /// `.within(...)` window's tracked state) — not a Rust-owned value, and
    /// not the same as a snapshot the user has explicitly copied into their
    /// own durable storage.
    EventCycle,
}

impl ParticipantLifetime {
    /// Whether a reference captured with `self` lifetime remains valid for
    /// a use that needs `needed`.
    ///
    /// A reference captured at [`Invocation`](Self::Invocation) does not
    /// cover a use that needs
    /// [`SynchronousDescendants`](Self::SynchronousDescendants) — it never
    /// promised to survive a descendant call. A reference captured at
    /// [`EventCycle`](Self::EventCycle) covers any narrower use.
    pub fn covers(self, needed: ParticipantLifetime) -> bool {
        self >= needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_scoped_context_is_accepted_at_invocation_use() {
        assert!(ParticipantLifetime::Invocation.covers(ParticipantLifetime::Invocation));
    }

    #[test]
    fn invocation_scoped_context_does_not_cover_descendant_use() {
        assert!(
            !ParticipantLifetime::Invocation.covers(ParticipantLifetime::SynchronousDescendants)
        );
    }

    #[test]
    fn synchronous_descendants_covers_invocation_use() {
        assert!(
            ParticipantLifetime::SynchronousDescendants.covers(ParticipantLifetime::Invocation)
        );
    }

    #[test]
    fn event_cycle_covers_every_narrower_use() {
        assert!(ParticipantLifetime::EventCycle.covers(ParticipantLifetime::Invocation));
        assert!(
            ParticipantLifetime::EventCycle.covers(ParticipantLifetime::SynchronousDescendants)
        );
        assert!(ParticipantLifetime::EventCycle.covers(ParticipantLifetime::EventCycle));
    }

    #[test]
    fn deterministic_lifetime_ordering() {
        let mut lifetimes = [
            ParticipantLifetime::EventCycle,
            ParticipantLifetime::Invocation,
            ParticipantLifetime::SynchronousDescendants,
        ];
        lifetimes.sort();
        assert_eq!(
            lifetimes,
            [
                ParticipantLifetime::Invocation,
                ParticipantLifetime::SynchronousDescendants,
                ParticipantLifetime::EventCycle,
            ]
        );
    }
}
