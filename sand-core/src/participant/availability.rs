//! Typed participant availability (#230 Phase 8).
//!
//! `Option<T>` alone cannot distinguish "this event's semantics make this
//! participant optional" from "Sand/vanilla cannot supply this participant
//! at all." [`ParticipantAvailability<T>`] keeps those cases explicit; a
//! caller working with an already-[`Available`](ParticipantAvailability::Available)
//! value may still use `Option<T>` internally for its own event-semantic
//! optionality (e.g. "no offhand item this occurrence"), but the outer
//! unsupported/unavailable states are never collapsed into it.

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::ParticipantUnavailableReason",
    aliases = ["sand::prelude::ParticipantUnavailableReason"],
    module = "sand::participant",
    summary = "A small, stable, public vocabulary of reasons a participant could not be supplied. Exporter-internal errors are not exposed through this type — see the module doc.",
    context = "A small, stable, public vocabulary of reasons a participant could not be supplied. Exporter-internal errors are not exposed through this type — see the module doc. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::ParticipantUnavailableReason;",
    variants(AmbiguousCandidates = "More than one candidate matched and none could be chosen safely.", CorrelationExpired = "A bounded correlation window closed before this participant could be associated.", ItemSourceAlreadyMutated = "The item source had already been mutated/consumed by vanilla before Sand could capture it.", LifetimeExpired = "The participant reference was used outside the [`ParticipantLifetime`](super::lifetime::ParticipantLifetime) it was valid for.", NoMatchingObservation = "A correlation/observation query ran and matched nothing.", NotApplicable = "This role does not apply to this event at all (e.g. \"victim\" on a non-combat event).", NotSuppliedByTrigger = "The triggering mechanism (advancement criterion, tick condition) never supplies this participant at all.", UnsupportedBackend = "The event's dispatch backend (tick-polled vs. advancement-backed vs. graph-bridged) does not supply this participant.", UnsupportedVersion = "The active `VersionProfile`/target version does not support the mechanism this participant would come from."),
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::ParticipantUnavailableReason::description",
        aliases = ["sand::prelude::ParticipantUnavailableReason::description"],
        module = "sand::participant",
        kind = "method",
        summary = "A short, stable, human-readable description suitable for diagnostics.",
        context = "A short, stable, human-readable description suitable for diagnostics. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "The string value produced to use a short, stable, human-readable description suitable for diagnostics.",
        example = "use sand::prelude::*;\n\nfn demonstrate(participant_unavailable_reason_value: sand::participant::ParticipantUnavailableReason)  {\n    let description = participant_unavailable_reason_value.description();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::ParticipantAvailability",
    aliases = ["sand::prelude::ParticipantAvailability"],
    module = "sand::participant",
    summary = "Whether a typed participant could be supplied for a specific event occurrence, and why not if not.",
    context = "Whether a typed participant could be supplied for a specific event occurrence, and why not if not. Never collapse this into `Option<T>` — the whole point is to keep \"unsupported\"/\"ambiguous\"/\"expired\" distinguishable from a event-semantic `None`.",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::ParticipantAvailability;",
    variants(Available = "Selects the available participant semantic.", Unavailable = "Selects the unavailable participant semantic."),
    variant_fields(Available = ["Selects the available participant semantic."], Unavailable = ["Selects the unavailable participant semantic."]),
)]
/// Whether a typed participant could be supplied for a specific event
/// occurrence, and why not if not.
///
/// Never collapse this into `Option<T>` — the whole point is to keep
/// "unsupported"/"ambiguous"/"expired" distinguishable from a event-semantic
/// `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticipantAvailability<T> {
    #[doc = "Selects the available participant semantic."]
    Available(#[doc = "Selects the available participant semantic."] T),
    #[doc = "Selects the unavailable participant semantic."]
    Unavailable(
        #[doc = "Selects the unavailable participant semantic."] ParticipantUnavailableReason,
    ),
}

impl<T> ParticipantAvailability<T> {
    /// Reports whether the event plan made this participant available to the handler.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::ParticipantAvailability::is_available",
        aliases = ["sand::prelude::ParticipantAvailability::is_available"],
        module = "sand::participant",
        kind = "method",
        summary = "Reports whether the event plan made this participant available to the handler.",
        context = "Reports whether the event plan made this participant available to the handler. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "`true` when the documented condition holds to report whether the event plan made this participant available to the handler; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(participant_availability_value: &sand::participant::ParticipantAvailability < T >)  {\n    let is_is_available = participant_availability_value.is_available();\n}",
    )]
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available(_))
    }

    /// Returns why the participant is unavailable, or `None` when it is available.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::ParticipantAvailability::reason",
        aliases = ["sand::prelude::ParticipantAvailability::reason"],
        module = "sand::participant",
        kind = "method",
        summary = "Returns why the participant is unavailable, or `None` when it is available.",
        context = "Returns why the participant is unavailable, or `None` when it is available. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns why the participant is unavailable, or `None` when it is available.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(participant_availability_value: &sand::participant::ParticipantAvailability < T >)  {\n    let reason = participant_availability_value.reason();\n}",
    )]
    pub fn reason(&self) -> Option<ParticipantUnavailableReason> {
        match self {
            Self::Available(_) => None,
            Self::Unavailable(reason) => Some(*reason),
        }
    }

    /// Extracts the participant value when available.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::ParticipantAvailability::available",
        aliases = ["sand::prelude::ParticipantAvailability::available"],
        module = "sand::participant",
        kind = "method",
        summary = "Extracts the participant value when available.",
        context = "Extracts the participant value when available. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "The matching value used to extract the participant value when available, or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(participant_availability_value: sand::participant::ParticipantAvailability < T >)  {\n    let available = participant_availability_value.available();\n}",
    )]
    pub fn available(self) -> Option<T> {
        match self {
            Self::Available(value) => Some(value),
            Self::Unavailable(_) => None,
        }
    }

    /// Transforms an available participant while preserving its unavailable reason.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::ParticipantAvailability::map",
        aliases = ["sand::prelude::ParticipantAvailability::map"],
        module = "sand::participant",
        kind = "method",
        summary = "Transforms an available participant while preserving its unavailable reason.",
        context = "Transforms an available participant while preserving its unavailable reason. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        params(f = "`f` is used to transform an available participant while preserving its unavailable reason."),
        returns = "The `ParticipantAvailability < U >` value produced to transform an available participant while preserving its unavailable reason.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(participant_availability_value: sand::participant::ParticipantAvailability < T >, f: impl FnOnce (T) -> U)  {\n    let map = participant_availability_value.map::<U>(f);\n}",
    )]
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
