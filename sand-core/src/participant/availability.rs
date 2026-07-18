//! Typed participant availability (#230 Phase 8).
//!
//! `Option<T>` alone cannot distinguish "this event's semantics make this
//! participant optional" from "Sand/vanilla cannot supply this participant
//! at all." [`ParticipantAvailability<T>`] keeps those cases explicit; a
//! caller working with an already-[`Available`](ParticipantAvailability::Available)
//! value may still use `Option<T>` internally for its own event-semantic
//! optionality (e.g. "no offhand item this occurrence"), but the outer
//! unsupported/unavailable states are never collapsed into it.

/// A small, stable, public vocabulary of reasons a participant could not be
/// supplied. Exporter-internal errors are not exposed through this type —
/// see the module doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParticipantUnavailableReason {
    /// The triggering mechanism (advancement criterion, tick condition)
    /// never supplies this participant at all.
    NotSuppliedByTrigger,
    /// The active `VersionProfile`/target version does not support the
    /// mechanism this participant would come from.
    UnsupportedVersion,
    /// The event's dispatch backend (tick-polled vs. advancement-backed vs.
    /// graph-bridged) does not supply this participant.
    UnsupportedBackend,
    /// More than one candidate matched and none could be chosen safely.
    AmbiguousCandidates,
    /// A bounded correlation window closed before this participant could be
    /// associated.
    CorrelationExpired,
    /// A correlation/observation query ran and matched nothing.
    NoMatchingObservation,
    /// This role does not apply to this event at all (e.g. "victim" on a
    /// non-combat event).
    NotApplicable,
    /// The item source had already been mutated/consumed by vanilla before
    /// Sand could capture it.
    ItemSourceAlreadyMutated,
    /// The participant reference was used outside the
    /// [`ParticipantLifetime`](super::lifetime::ParticipantLifetime) it was
    /// valid for.
    LifetimeExpired,
}

impl ParticipantUnavailableReason {
    /// A short, stable, human-readable description suitable for
    /// diagnostics.
    pub fn description(self) -> &'static str {
        match self {
            Self::NotSuppliedByTrigger => "not supplied by the triggering mechanism",
            Self::UnsupportedVersion => "unsupported by the target Minecraft version",
            Self::UnsupportedBackend => "unsupported by this event's dispatch backend",
            Self::AmbiguousCandidates => "ambiguous — more than one candidate matched",
            Self::CorrelationExpired => {
                "the correlation window closed before this could be associated"
            }
            Self::NoMatchingObservation => "no matching observation was found",
            Self::NotApplicable => "this role does not apply to this event",
            Self::ItemSourceAlreadyMutated => "the item source was already mutated before capture",
            Self::LifetimeExpired => "used outside the lifetime this reference was valid for",
        }
    }
}

/// Whether a typed participant could be supplied for a specific event
/// occurrence, and why not if not.
///
/// Never collapse this into `Option<T>` — the whole point is to keep
/// "unsupported"/"ambiguous"/"expired" distinguishable from a event-semantic
/// `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticipantAvailability<T> {
    Available(T),
    Unavailable(ParticipantUnavailableReason),
}

impl<T> ParticipantAvailability<T> {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available(_))
    }

    pub fn reason(&self) -> Option<ParticipantUnavailableReason> {
        match self {
            Self::Available(_) => None,
            Self::Unavailable(reason) => Some(*reason),
        }
    }

    pub fn available(self) -> Option<T> {
        match self {
            Self::Available(value) => Some(value),
            Self::Unavailable(_) => None,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ParticipantAvailability<U> {
        match self {
            Self::Available(value) => ParticipantAvailability::Available(f(value)),
            Self::Unavailable(reason) => ParticipantAvailability::Unavailable(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_produces_actionable_reason() {
        let value: ParticipantAvailability<u8> =
            ParticipantAvailability::Unavailable(ParticipantUnavailableReason::AmbiguousCandidates);
        assert!(!value.is_available());
        assert_eq!(
            value.reason(),
            Some(ParticipantUnavailableReason::AmbiguousCandidates)
        );
        assert!(!value.reason().unwrap().description().is_empty());
    }

    #[test]
    fn available_round_trips() {
        let value = ParticipantAvailability::Available(7u8);
        assert!(value.is_available());
        assert_eq!(value.reason(), None);
        assert_eq!(value.available(), Some(7));
    }

    #[test]
    fn map_preserves_unavailable_reason() {
        let value: ParticipantAvailability<u8> =
            ParticipantAvailability::Unavailable(ParticipantUnavailableReason::LifetimeExpired);
        let mapped = value.map(|n| n as u32);
        assert_eq!(
            mapped,
            ParticipantAvailability::Unavailable(ParticipantUnavailableReason::LifetimeExpired)
        );
    }

    #[test]
    fn every_reason_has_a_distinct_description() {
        let reasons = [
            ParticipantUnavailableReason::NotSuppliedByTrigger,
            ParticipantUnavailableReason::UnsupportedVersion,
            ParticipantUnavailableReason::UnsupportedBackend,
            ParticipantUnavailableReason::AmbiguousCandidates,
            ParticipantUnavailableReason::CorrelationExpired,
            ParticipantUnavailableReason::NoMatchingObservation,
            ParticipantUnavailableReason::NotApplicable,
            ParticipantUnavailableReason::ItemSourceAlreadyMutated,
            ParticipantUnavailableReason::LifetimeExpired,
        ];
        let mut descriptions: Vec<&str> = reasons.iter().map(|r| r.description()).collect();
        descriptions.sort_unstable();
        descriptions.dedup();
        assert_eq!(descriptions.len(), reasons.len());
    }
}
