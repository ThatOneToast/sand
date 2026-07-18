//! Typed participant reliability (#230 Phase 8).

use crate::item::SnapshotReliability;

/// How strong the evidence is for a captured/referenced event participant.
///
/// Variants are declared weakest-first so the derived [`Ord`] doubles as a
/// strength ordering: `a.meets(b)` is exactly `a >= b`. This is the single
/// reliability vocabulary for every kind of participant (entities, players,
/// items, locations) — see [`SnapshotReliability::as_participant_reliability`]
/// for how Phase 7's item-specific enum maps into it.
///
/// - [`Unavailable`](Self::Unavailable) — nothing could be supplied.
/// - [`Inferred`](Self::Inferred) — selected through a heuristic/query that
///   may be ambiguous (e.g. a nearest-entity guess). Phase 8 defines this
///   level; nothing in the current codebase produces it yet.
/// - [`Correlated`](Self::Correlated) — associated through a bounded
///   observation mechanism with stated constraints (a plausible match, not
///   a value the triggering mechanism directly handed to Sand). Nothing in
///   the current codebase produces this for entities yet; Phase 7's
///   `SnapshotReliability::Correlated` maps here for items.
/// - [`ExactSnapshot`](Self::ExactSnapshot) — data was copied at (or as
///   close as Sand can get to) the authoritative event boundary and is
///   immutable afterward. This is what every `ItemSnapshot` capture
///   produces (items are never referenced live — they are always copied
///   into storage) — see [`ItemEvidenceQualifier`] for the finer-grained
///   distinction Phase 7 draws between the two item capture points.
/// - [`Exact`](Self::Exact) — a live, authoritative reference supplied
///   directly by the triggering vanilla execution context (e.g. the
///   advancement reward function's `@s`, or a tick-polled player's `@s`).
///   Reserved for references that remain usable for further live command
///   building, not frozen copies.
///
/// `Exact` ranks above `ExactSnapshot` in this ordering: a live reference
/// can still be traversed/re-queried (e.g. `execute on attacker run ...`),
/// while a snapshot is deliberately frozen data. Both are strong evidence;
/// they answer different questions ("what can I still address" vs. "what
/// was true at capture time").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParticipantReliability {
    Unavailable,
    Inferred,
    Correlated,
    ExactSnapshot,
    Exact,
}

impl ParticipantReliability {
    /// Whether this reliability level satisfies a `required` floor.
    ///
    /// `Correlated.meets(Exact)` is `false`; `Exact.meets(Correlated)` is
    /// `true`. This is the check behind
    /// [`PlayerParticipant::require_exact`](super::reference::PlayerParticipant::require_exact)
    /// and its siblings.
    pub fn meets(self, required: ParticipantReliability) -> bool {
        self >= required
    }
}

/// Finer-grained evidence for an item participant's [`ExactSnapshot`](ParticipantReliability::ExactSnapshot)
/// reliability, preserving the distinction Phase 7's `SnapshotReliability`
/// draws between its two "exact" levels rather than flattening them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemEvidenceQualifier {
    /// Copied before any vanilla-side mutation window Sand knows of —
    /// [`SnapshotReliability::Exact`].
    CapturedBeforeVanillaMutation,
    /// Copied at the first point Sand's own generated code ran, but vanilla
    /// may already have mutated the source before the triggering criterion
    /// fired at all — [`SnapshotReliability::ExactPostTrigger`].
    CapturedAtFirstSandControl,
}

impl SnapshotReliability {
    /// Map this Phase 7 item-specific reliability level into the shared
    /// Phase 8 [`ParticipantReliability`] vocabulary.
    ///
    /// Both [`SnapshotReliability::Exact`] and
    /// [`SnapshotReliability::ExactPostTrigger`] map to
    /// [`ParticipantReliability::ExactSnapshot`] — items are always copied
    /// into storage, never referenced live, so neither ever qualifies for
    /// [`ParticipantReliability::Exact`]. The distinction between them is
    /// not lost: see [`SnapshotReliability::item_evidence_qualifier`].
    pub fn as_participant_reliability(self) -> ParticipantReliability {
        match self {
            SnapshotReliability::Exact | SnapshotReliability::ExactPostTrigger => {
                ParticipantReliability::ExactSnapshot
            }
            SnapshotReliability::Correlated => ParticipantReliability::Correlated,
            SnapshotReliability::Unavailable => ParticipantReliability::Unavailable,
        }
    }

    /// The finer-grained item capture evidence behind an `ExactSnapshot`
    /// mapping, or `None` for levels that don't carry this distinction.
    pub fn item_evidence_qualifier(self) -> Option<ItemEvidenceQualifier> {
        match self {
            SnapshotReliability::Exact => {
                Some(ItemEvidenceQualifier::CapturedBeforeVanillaMutation)
            }
            SnapshotReliability::ExactPostTrigger => {
                Some(ItemEvidenceQualifier::CapturedAtFirstSandControl)
            }
            SnapshotReliability::Correlated | SnapshotReliability::Unavailable => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_satisfies_exact_requirement() {
        assert!(ParticipantReliability::Exact.meets(ParticipantReliability::Exact));
    }

    #[test]
    fn exact_satisfies_weaker_requirement() {
        assert!(ParticipantReliability::Exact.meets(ParticipantReliability::Correlated));
        assert!(ParticipantReliability::ExactSnapshot.meets(ParticipantReliability::Inferred));
    }

    #[test]
    fn correlated_does_not_satisfy_exact() {
        assert!(!ParticipantReliability::Correlated.meets(ParticipantReliability::Exact));
        assert!(!ParticipantReliability::Correlated.meets(ParticipantReliability::ExactSnapshot));
    }

    #[test]
    fn inferred_does_not_satisfy_correlated_or_exact() {
        assert!(!ParticipantReliability::Inferred.meets(ParticipantReliability::Correlated));
        assert!(!ParticipantReliability::Inferred.meets(ParticipantReliability::Exact));
    }

    #[test]
    fn unavailable_is_the_weakest_level() {
        assert!(ParticipantReliability::Unavailable <= ParticipantReliability::Inferred);
        assert!(!ParticipantReliability::Unavailable.meets(ParticipantReliability::Inferred));
    }

    #[test]
    fn deterministic_reliability_ordering() {
        let mut levels = [
            ParticipantReliability::Exact,
            ParticipantReliability::Unavailable,
            ParticipantReliability::Correlated,
            ParticipantReliability::ExactSnapshot,
            ParticipantReliability::Inferred,
        ];
        levels.sort();
        assert_eq!(
            levels,
            [
                ParticipantReliability::Unavailable,
                ParticipantReliability::Inferred,
                ParticipantReliability::Correlated,
                ParticipantReliability::ExactSnapshot,
                ParticipantReliability::Exact,
            ]
        );
    }

    #[test]
    fn item_snapshot_reliability_mapping_preserves_exact_post_trigger() {
        assert_eq!(
            SnapshotReliability::Exact.as_participant_reliability(),
            ParticipantReliability::ExactSnapshot
        );
        assert_eq!(
            SnapshotReliability::ExactPostTrigger.as_participant_reliability(),
            ParticipantReliability::ExactSnapshot
        );
        // Same umbrella level, but the qualifier keeps the two distinct.
        assert_ne!(
            SnapshotReliability::Exact.item_evidence_qualifier(),
            SnapshotReliability::ExactPostTrigger.item_evidence_qualifier()
        );
        assert_eq!(
            SnapshotReliability::Exact.item_evidence_qualifier(),
            Some(ItemEvidenceQualifier::CapturedBeforeVanillaMutation)
        );
        assert_eq!(
            SnapshotReliability::ExactPostTrigger.item_evidence_qualifier(),
            Some(ItemEvidenceQualifier::CapturedAtFirstSandControl)
        );
    }

    #[test]
    fn item_snapshot_correlated_and_unavailable_map_directly() {
        assert_eq!(
            SnapshotReliability::Correlated.as_participant_reliability(),
            ParticipantReliability::Correlated
        );
        assert_eq!(
            SnapshotReliability::Unavailable.as_participant_reliability(),
            ParticipantReliability::Unavailable
        );
        assert_eq!(
            SnapshotReliability::Correlated.item_evidence_qualifier(),
            None
        );
        assert_eq!(
            SnapshotReliability::Unavailable.item_evidence_qualifier(),
            None
        );
    }
}
