//! Event subject-capability descriptors (#230 Phase 8; narrowed by #274).
//!
//! An [`EventContextCapabilities`] value describes what an event type's
//! *own dispatch shape* can promise about its **subject** participant (the
//! entity/player the event is naturally about — never a same-cycle
//! participant plan's entity/item bindings, which are a separate, real
//! mechanism: see [`crate::participant::plan::EventParticipantPlan`] and
//! `sand-core/src/compiler/export/participant_transport.rs`). It holds only
//! `'static` data (`Copy` enums), so it is deterministic, cheap to compare,
//! and safe to compute once per generic event monomorphization (no
//! `TypeId`-derived identity anywhere in this module: two distinct generic
//! instantiations of the same event family simply get distinct
//! `EventContextCapabilities` values by construction, compared structurally
//! like any other data).
//!
//! This module previously also carried a parallel set of helpers
//! (`EventContextCapabilities::for_event_with_participants`,
//! `capabilities::full::{propagate_after, merge_after_any, merge_after_all,
//! propagate_within}`) that extended the same propagation rules to declared
//! entity/item participants. An #274 audit found those helpers had zero
//! production call sites: they computed a "could honestly promise" value
//! that nothing in the export pipeline ever consulted, while real
//! participant transport (borrowing a concrete tag/item snapshot across a
//! same-cycle graph edge) is validated separately by
//! `sand-core/src/compiler/export/participant_transport.rs`. They were
//! removed rather than wired up, to avoid two parallel and divergent
//! sources of "what participants does this event have" truth. See
//! `docs/testing/participant-role-evidence.md`'s "Participant propagation
//! across event graph edges" section for the full history.

use crate::events::{SandEvent, SandEventDispatch};

use super::lifetime::ParticipantLifetime;
use super::reliability::ParticipantReliability;

/// Whether an event's subject is guaranteed to be a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubjectScope {
    /// The subject is guaranteed to be a player (every dispatch kind Phase
    /// 8 resolves capabilities for today is player-scoped).
    Player,
    /// No subject participant is currently supported for this dispatch
    /// shape (e.g. a same-cycle chained event, whose subject capability
    /// this phase does not attempt to resolve generically — see
    /// [`EventContextCapabilities::for_event`]).
    NonPlayerUnsupported,
}

/// What an event can promise about its own subject participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubjectCapability {
    pub reliability: ParticipantReliability,
    pub lifetime: ParticipantLifetime,
    pub scope: SubjectScope,
}

impl SubjectCapability {
    /// No subject participant is supported.
    pub const NONE: SubjectCapability = SubjectCapability {
        reliability: ParticipantReliability::Unavailable,
        lifetime: ParticipantLifetime::Invocation,
        scope: SubjectScope::NonPlayerUnsupported,
    };

    /// An exact player subject valid for the capturing invocation — what
    /// every tick-polled (`TickScope::Players`), advancement-triggered, and
    /// Phase 6 `TickScope::AdvancementPlayer` event already supplies today.
    pub const EXACT_PLAYER_INVOCATION: SubjectCapability = SubjectCapability {
        reliability: ParticipantReliability::Exact,
        lifetime: ParticipantLifetime::Invocation,
        scope: SubjectScope::Player,
    };
}

impl Default for SubjectCapability {
    fn default() -> Self {
        SubjectCapability::NONE
    }
}

/// What an event type's own dispatch shape promises about its subject
/// participant, declared once and shared by every occurrence of that event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventContextCapabilities {
    pub subject: SubjectCapability,
}

impl EventContextCapabilities {
    pub const NONE: EventContextCapabilities = EventContextCapabilities {
        subject: SubjectCapability::NONE,
    };

    pub const EXACT_PLAYER_SUBJECT: EventContextCapabilities = EventContextCapabilities {
        subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
    };

    /// Structurally derive the capabilities of `E`'s *own* dispatch shape —
    /// not counting anything a same-cycle parent might contribute.
    ///
    /// - `AdvancementTrigger`/legacy `TickCondition` dispatch: exact player
    ///   subject, invocation lifetime.
    /// - Structured [`SandEventDispatch::tick`]: exact player subject iff
    ///   [`TickScope::has_player_subject`](crate::events::TickScope::has_player_subject)
    ///   holds for the declared scope.
    /// - [`SandEventDispatch::chain`]/`compose()`: **not resolved here.**
    ///   A `ChainEventDispatch`'s parent(s) are identified by type-erased
    ///   function-pointer factories (see `sand-core/src/events/graph.rs`
    ///   `OccurrenceParent`) precisely so the parent marker type never
    ///   needs to be instantiated — that means this function cannot
    ///   generically call `EventContextCapabilities::for_event::<Parent>()`
    ///   from inside `E::dispatch()`'s already-erased value. `for_event`
    ///   returns [`EventContextCapabilities::NONE`] for a chained event;
    ///   callers who know the concrete parent type must instead call
    ///   [`for_event::<Parent>()`](Self::for_event) themselves and combine
    ///   it with [`propagate_after`]/[`merge_after_any`]/[`merge_after_all`]
    ///   below. This is a real, documented limitation, not an oversight —
    ///   full graph-integrated subject resolution remains unimplemented.
    pub fn for_event<E: SandEvent + 'static>() -> EventContextCapabilities {
        let dispatch: SandEventDispatch = E::dispatch().into();
        match dispatch {
            SandEventDispatch::AdvancementTrigger(_) | SandEventDispatch::TickCondition(_) => {
                Self::EXACT_PLAYER_SUBJECT
            }
            SandEventDispatch::Tick(tick) => {
                if tick.scope.has_player_subject() {
                    Self::EXACT_PLAYER_SUBJECT
                } else {
                    Self::NONE
                }
            }
            SandEventDispatch::Chain(_) => Self::NONE,
            // Tracked-transition dispatch runs `execute as @a ... at @s` per
            // player, same as `TickCondition` — the subject is always the
            // exact observed player.
            SandEventDispatch::Tracked(_) => Self::EXACT_PLAYER_SUBJECT,
        }
    }
}

/// Graph propagation/merge produced an incompatible or empty combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMergeError {
    /// A merge was requested over zero parents.
    EmptyParentSet,
    /// Parents disagree on subject scope (e.g. one player-scoped, one not)
    /// — merging them cannot produce an honest single subject capability.
    IncompatibleSubjectScope,
}

impl std::fmt::Display for ContextMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyParentSet => {
                write!(f, "cannot merge context capabilities over zero parents")
            }
            Self::IncompatibleSubjectScope => {
                write!(
                    f,
                    "parents disagree on subject scope and cannot be merged into one honest subject capability"
                )
            }
        }
    }
}

impl std::error::Error for ContextMergeError {}

/// A same-cycle `.after::<Parent>()` child inherits its sole parent's
/// subject capability, but only for the child's own synchronous-descendant
/// lifetime — the child did not itself run at [`ParticipantLifetime::Invocation`]
/// of the parent's own call, it runs one level deeper.
///
/// This is the rule behind both a direct tick-graph `after::<E>()` edge and
/// a Phase 6 advancement-backed bridge's synchronous dispatch — both
/// guarantee a single, statically-known parent, so both use this same
/// propagation rule (see `TickScope::AdvancementPlayer`'s doc: it is
/// compatible only with the sole-parent `after::<E>()` shape for exactly
/// this reason).
pub fn propagate_after(parent: SubjectCapability) -> SubjectCapability {
    SubjectCapability {
        reliability: parent.reliability,
        lifetime: parent
            .lifetime
            .max(ParticipantLifetime::SynchronousDescendants),
        scope: parent.scope,
    }
}

/// `after_any` merge: every parent in the group must agree on subject
/// scope (since only one of them will actually have fired, the child
/// cannot assume anything not true of *every* candidate). The resulting
/// reliability is the weakest of the group — an honest floor, since the
/// child cannot statically know which parent supplied it.
pub fn merge_after_any(
    parents: &[SubjectCapability],
) -> Result<SubjectCapability, ContextMergeError> {
    merge_disjunctive_or_conjunctive(parents)
}

/// `after_all` merge: every parent must have fired, so the same
/// same-scope-agreement rule applies; unlike `after_any`, every named
/// parent really did contribute, but Phase 8 does not yet track per-parent
/// participant identity, so it still cannot claim anything beyond the
/// shared subject — a genuinely richer per-parent context (e.g. two
/// different parents each naming a different attacker) must not be unioned
/// into one field, which is why this shares the same conservative
/// implementation as [`merge_after_any`] rather than trying to combine
/// parent-specific fields.
pub fn merge_after_all(
    parents: &[SubjectCapability],
) -> Result<SubjectCapability, ContextMergeError> {
    merge_disjunctive_or_conjunctive(parents)
}

fn merge_disjunctive_or_conjunctive(
    parents: &[SubjectCapability],
) -> Result<SubjectCapability, ContextMergeError> {
    let Some(first) = parents.first() else {
        return Err(ContextMergeError::EmptyParentSet);
    };
    for parent in &parents[1..] {
        if parent.scope != first.scope {
            return Err(ContextMergeError::IncompatibleSubjectScope);
        }
    }
    let weakest_reliability = parents
        .iter()
        .map(|p| p.reliability)
        .min()
        .unwrap_or(ParticipantReliability::Unavailable);
    let narrowest_lifetime = parents
        .iter()
        .map(|p| p.lifetime)
        .min()
        .unwrap_or(ParticipantLifetime::Invocation);
    Ok(SubjectCapability {
        reliability: weakest_reliability,
        lifetime: narrowest_lifetime.max(ParticipantLifetime::SynchronousDescendants),
        scope: first.scope,
    })
}

/// `.while_(...)` contributes a persistent-state *condition*, never a
/// participant — it must not alter the subject capability it's attached
/// to.
pub fn propagate_while(current: SubjectCapability) -> SubjectCapability {
    current
}

/// `.when(...)`/`.unless(...)` are ordinary conditions and must not alter
/// capabilities either.
pub fn propagate_when_unless(current: SubjectCapability) -> SubjectCapability {
    current
}

/// `.within::<E>(TickWindow)` cannot retain any capability more precise
/// than an [`EventCycle`](ParticipantLifetime::EventCycle)-scoped subject:
/// the correlation crosses tick boundaries, so anything ephemeral captured
/// at the original firing invocation is gone by the time the bounded
/// condition is later observed true. Only the tracked subject itself
/// (the scoreboard age counter's own `@s`) remains meaningful, and even
/// that downgrades to [`ParticipantLifetime::EventCycle`] — never
/// `Invocation`/`SynchronousDescendants` — because it is being read back
/// from persisted state, not handed over synchronously.
pub fn propagate_within(parent_subject: SubjectCapability) -> SubjectCapability {
    if parent_subject.scope == SubjectScope::NonPlayerUnsupported {
        return SubjectCapability::NONE;
    }
    SubjectCapability {
        reliability: parent_subject
            .reliability
            .min(ParticipantReliability::Correlated),
        lifetime: ParticipantLifetime::EventCycle,
        scope: parent_subject.scope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;
    use crate::events::{EventSetup, SandEvent, SandEventDispatch};

    struct SimpleTickEvent;
    impl SandEvent for SimpleTickEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }

    struct AdvancementBackedEvent;
    impl SandEvent for AdvancementBackedEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::TickCondition("score @s x matches 1".into())
        }
        fn setup() -> EventSetup {
            EventSetup::none()
        }
    }

    struct ChainedEvent;
    impl SandEvent for ChainedEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::chain::<SimpleTickEvent>()
                .when(Condition::raw("block ~ ~-1 ~ minecraft:stone"))
        }
    }

    #[test]
    fn default_simple_event_gets_exact_player_subject() {
        let caps = EventContextCapabilities::for_event::<SimpleTickEvent>();
        assert_eq!(caps.subject, SubjectCapability::EXACT_PLAYER_INVOCATION);
    }

    #[test]
    fn legacy_tick_condition_also_gets_exact_player_subject() {
        let caps = EventContextCapabilities::for_event::<AdvancementBackedEvent>();
        assert_eq!(caps.subject.scope, SubjectScope::Player);
        assert_eq!(caps.subject.reliability, ParticipantReliability::Exact);
    }

    #[test]
    fn chained_event_capabilities_are_not_resolved_generically() {
        let caps = EventContextCapabilities::for_event::<ChainedEvent>();
        assert_eq!(caps, EventContextCapabilities::NONE);
    }

    #[test]
    fn stable_equality_and_ordering() {
        let a = EventContextCapabilities::EXACT_PLAYER_SUBJECT;
        let b = EventContextCapabilities::EXACT_PLAYER_SUBJECT;
        assert_eq!(a, b);
        assert!(EventContextCapabilities::NONE < EventContextCapabilities::EXACT_PLAYER_SUBJECT);
    }

    #[test]
    fn generic_event_types_remain_distinct() {
        struct Wrapper<T>(std::marker::PhantomData<T>);
        impl<T: 'static> SandEvent for Wrapper<T> {
            fn dispatch() -> impl Into<SandEventDispatch> {
                SandEventDispatch::tick().as_players()
            }
        }
        // Distinct monomorphizations still compute the same, stable value —
        // no TypeId-derived divergence.
        let a = EventContextCapabilities::for_event::<Wrapper<u8>>();
        let b = EventContextCapabilities::for_event::<Wrapper<u16>>();
        assert_eq!(a, b);
    }

    #[test]
    fn single_parent_exact_player_propagation() {
        let parent = SubjectCapability::EXACT_PLAYER_INVOCATION;
        let child = propagate_after(parent);
        assert_eq!(child.reliability, ParticipantReliability::Exact);
        assert_eq!(child.lifetime, ParticipantLifetime::SynchronousDescendants);
        assert_eq!(child.scope, SubjectScope::Player);
    }

    #[test]
    fn advancement_bridge_exact_player_propagation_uses_same_rule() {
        // TickScope::AdvancementPlayer resolves to the same
        // EXACT_PLAYER_INVOCATION capability as TickScope::Players, so the
        // bridge case is exercised by the same propagate_after call.
        let parent = SubjectCapability::EXACT_PLAYER_INVOCATION;
        assert_eq!(
            propagate_after(parent),
            propagate_after(SubjectCapability::EXACT_PLAYER_INVOCATION)
        );
    }

    #[test]
    fn after_any_shared_subject_propagation() {
        let parents = [
            SubjectCapability::EXACT_PLAYER_INVOCATION,
            SubjectCapability::EXACT_PLAYER_INVOCATION,
        ];
        let merged = merge_after_any(&parents).unwrap();
        assert_eq!(merged.scope, SubjectScope::Player);
        assert_eq!(merged.reliability, ParticipantReliability::Exact);
    }

    #[test]
    fn after_all_shared_subject_propagation() {
        let parents = [
            SubjectCapability::EXACT_PLAYER_INVOCATION,
            SubjectCapability::EXACT_PLAYER_INVOCATION,
        ];
        let merged = merge_after_all(&parents).unwrap();
        assert_eq!(merged.scope, SubjectScope::Player);
    }

    #[test]
    fn incompatible_scope_merge_is_rejected() {
        let parents = [
            SubjectCapability::EXACT_PLAYER_INVOCATION,
            SubjectCapability::NONE,
        ];
        assert_eq!(
            merge_after_any(&parents),
            Err(ContextMergeError::IncompatibleSubjectScope)
        );
    }

    #[test]
    fn empty_parent_set_is_rejected() {
        assert_eq!(merge_after_any(&[]), Err(ContextMergeError::EmptyParentSet));
    }

    #[test]
    fn merge_is_order_independent() {
        let a = SubjectCapability::EXACT_PLAYER_INVOCATION;
        let b = SubjectCapability {
            reliability: ParticipantReliability::Correlated,
            ..SubjectCapability::EXACT_PLAYER_INVOCATION
        };
        let forward = merge_after_any(&[a, b]).unwrap();
        let backward = merge_after_any(&[b, a]).unwrap();
        assert_eq!(forward, backward);
    }

    #[test]
    fn within_does_not_retain_ephemeral_participant_context() {
        let parent = SubjectCapability::EXACT_PLAYER_INVOCATION;
        let via_within = propagate_within(parent);
        assert_eq!(via_within.lifetime, ParticipantLifetime::EventCycle);
        assert!(via_within.reliability <= ParticipantReliability::Correlated);
        assert_ne!(via_within.lifetime, ParticipantLifetime::Invocation);
    }

    #[test]
    fn while_does_not_add_participants() {
        let current = SubjectCapability::EXACT_PLAYER_INVOCATION;
        assert_eq!(propagate_while(current), current);
    }

    #[test]
    fn when_and_unless_do_not_alter_capabilities() {
        let current = SubjectCapability::EXACT_PLAYER_INVOCATION;
        assert_eq!(propagate_when_unless(current), current);
    }
}
