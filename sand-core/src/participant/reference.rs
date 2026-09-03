//! Typed generated-command participant references (#230 Phase 8).

use sand_commands::selector::{
    AnyTarget, One, PlayersOnly, SingleEntity, SinglePlayer, SingleTargetArgument, Target,
};

use super::lifetime::ParticipantLifetime;
use super::reliability::ParticipantReliability;
use super::role::EntityParticipantRole;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::ParticipantReliabilityError",
    module = "sand::participant",
    summary = "A requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope.",
    context = "A requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope. This is the diagnostic behind [`PlayerParticipant::require_exact`]/[`EntityParticipant::require_exact`] (and the general [`require`](PlayerParticipant::require) form). It names the role, what was requested, and what was actually supplied so a caller can see exactly why a `require_exact()` call was rejected.",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::ParticipantReliabilityError;",
    fields(requested = "`requested` provides the requested when a requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope.", role = "`role` provides the role when a requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope.", supplied = "`supplied` provides the supplied when a requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope."),
)]
/// A requested reliability floor was not met by a supplied participant, or
/// a participant reference was used outside its declared execution scope.
///
/// This is the diagnostic behind
/// [`PlayerParticipant::require_exact`]/[`EntityParticipant::require_exact`]
/// (and the general [`require`](PlayerParticipant::require) form). It names
/// the role, what was requested, and what was actually supplied so a
/// caller can see exactly why a `require_exact()` call was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantReliabilityError {
    /// `role` provides the role when a requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope.
    pub role: EntityParticipantRole,
    /// `requested` provides the requested when a requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope.
    pub requested: ParticipantReliability,
    /// `supplied` provides the supplied when a requested reliability floor was not met by a supplied participant, or a participant reference was used outside its declared execution scope.
    pub supplied: ParticipantReliability,
}

impl std::fmt::Display for ParticipantReliabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "participant role {:?} requires {:?} reliability, but only {:?} was supplied",
            self.role, self.requested, self.supplied
        )
    }
}

impl std::error::Error for ParticipantReliabilityError {}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::PlayerParticipant",
    module = "sand::participant",
    summary = "A typed player participant reference: a command-building handle, not live runtime player data.",
    context = "A typed player participant reference: a command-building handle, not live runtime player data. Every field is private, so a caller cannot construct a `PlayerParticipant` claiming a reliability stronger than what [`PlayerParticipant::subject`] actually provides: The only constructor Phase 8 provides is [`PlayerParticipant::subject`] — the event's own triggering/polled player, which is the one case Sand can honestly mark [`ParticipantReliability::Exact`] today (Phase 6's `TickScope::has_player_subject`). Constructing an *other* player participant (e.g. a correlated nearby player) is Phase 9 observation work and deliberately not provided here — see [`EntityParticipant::correlated`]/[`EntityParticipant::inferred`] for the general non-exact constructors this phase does provide, which are intentionally the only way to build a non-subject reference so callers cannot self-report a stronger reliability than they actually have.",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::PlayerParticipant;",
)]
/// A typed player participant reference: a command-building handle, not
/// live runtime player data.
///
/// ```
/// use sand_core::participant::{ParticipantReliability, PlayerParticipant};
///
/// let subject = PlayerParticipant::subject();
/// assert!(subject.require_exact().is_ok());
/// assert_eq!(subject.reliability(), ParticipantReliability::Exact);
/// ```
///
/// Every field is private, so a caller cannot construct a `PlayerParticipant`
/// claiming a reliability stronger than what [`PlayerParticipant::subject`]
/// actually provides:
///
/// ```compile_fail
/// use sand_core::participant::{PlayerParticipant, EntityParticipantRole, ParticipantReliability, ParticipantLifetime};
///
/// // Fields are private — this does not compile.
/// let _ = PlayerParticipant {
///     selector: sand_commands::selector::Target::current_player(),
///     role: EntityParticipantRole::Attacker,
///     reliability: ParticipantReliability::Exact,
///     lifetime: ParticipantLifetime::Invocation,
/// };
/// ```
///
/// The only constructor Phase 8 provides is [`PlayerParticipant::subject`]
/// — the event's own triggering/polled player, which is the one case Sand
/// can honestly mark [`ParticipantReliability::Exact`] today (Phase 6's
/// `TickScope::has_player_subject`). Constructing an *other* player
/// participant (e.g. a correlated nearby player) is Phase 9 observation
/// work and deliberately not provided here — see
/// [`EntityParticipant::correlated`]/[`EntityParticipant::inferred`] for
/// the general non-exact constructors this phase does provide, which are
/// intentionally the only way to build a non-subject reference so callers
/// cannot self-report a stronger reliability than they actually have.
#[derive(Debug, Clone)]
pub struct PlayerParticipant {
    selector: SinglePlayer,
    role: EntityParticipantRole,
    reliability: ParticipantReliability,
    lifetime: ParticipantLifetime,
}

impl PlayerParticipant {
    /// The event's own player subject, rendered as `@s`. Always
    /// [`ParticipantReliability::Exact`] with
    /// [`ParticipantLifetime::Invocation`] — a caller needing a wider
    /// lifetime must justify it via graph propagation (see
    /// `super::capabilities`), not by constructing this directly with a
    /// different lifetime.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::subject",
        module = "sand::participant",
        kind = "method",
        summary = "The event's own player subject, rendered as `@s`. Always [`ParticipantReliability::Exact`] with [`ParticipantLifetime::Invocation`] — a caller needing a wider lifetime must justify it via graph propagation (see `super::capabilities`), not by constructing this directly with a different lifetime.",
        context = "The event's own player subject, rendered as `@s`. Always [`ParticipantReliability::Exact`] with [`ParticipantLifetime::Invocation`] — a caller needing a wider lifetime must justify it via graph propagation (see `super::capabilities`), not by constructing this directly with a different lifetime. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "A `PlayerParticipant` configured for the event's own player subject, rendered as `@s`. Always [`ParticipantReliability::Exact`] with [`ParticipantLifetime::Invocation`] — a caller needing a wider lifetime must justify it via graph propagation (see `super::capabilities`), not by constructing this directly with a different lifetime.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_participant = sand::participant::PlayerParticipant::subject();\n}",
    )]
    pub fn subject() -> Self {
        Self {
            selector: SinglePlayer::self_(),
            role: EntityParticipantRole::Subject,
            reliability: ParticipantReliability::Exact,
            lifetime: ParticipantLifetime::Invocation,
        }
    }

    /// Returns the player's semantic role in the current event.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::role",
        module = "sand::participant",
        kind = "method",
        summary = "Returns the player's semantic role in the current event.",
        context = "Returns the player's semantic role in the current event. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns the player's semantic role in the current event.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_participant_value: &sand::participant::PlayerParticipant)  {\n    let role = player_participant_value.role();\n}",
    )]
    pub fn role(&self) -> EntityParticipantRole {
        self.role
    }

    /// Returns the evidence strength that established this player participant.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::reliability",
        module = "sand::participant",
        kind = "method",
        summary = "Returns the evidence strength that established this player participant.",
        context = "Returns the evidence strength that established this player participant. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns the evidence strength that established this player participant.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_participant_value: &sand::participant::PlayerParticipant)  {\n    let reliability = player_participant_value.reliability();\n}",
    )]
    pub fn reliability(&self) -> ParticipantReliability {
        self.reliability
    }

    /// Returns how long the player reference remains valid.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::lifetime",
        module = "sand::participant",
        kind = "method",
        summary = "Returns how long the player reference remains valid.",
        context = "Returns how long the player reference remains valid. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns how long the player reference remains valid.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_participant_value: &sand::participant::PlayerParticipant)  {\n    let lifetime = player_participant_value.lifetime();\n}",
    )]
    pub fn lifetime(&self) -> ParticipantLifetime {
        self.lifetime
    }

    /// The typed selector for building commands against this participant.
    /// Never exposes a raw/unrestricted selector string — the caller gets
    /// [`SinglePlayer`]'s own safe builder surface.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::selector",
        module = "sand::participant",
        kind = "method",
        summary = "The typed selector for building commands against this participant. Never exposes a raw/unrestricted selector string — the caller gets [`SinglePlayer`]'s own safe builder surface.",
        context = "The typed selector for building commands against this participant. Never exposes a raw/unrestricted selector string — the caller gets [`SinglePlayer`]'s own safe builder surface. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "The `& SinglePlayer` value produced to use the typed selector for building commands against this participant. Never exposes a raw/unrestricted selector string — the caller gets [`SinglePlayer`]'s own safe builder surface.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_participant_value: &sand::participant::PlayerParticipant)  {\n    let selector = player_participant_value.selector();\n}",
    )]
    pub fn selector(&self) -> Target<PlayersOnly, One> {
        self.selector.clone().into()
    }

    /// Require at least `required` reliability, or a
    /// [`ParticipantReliabilityError`] naming exactly what was requested
    /// vs. supplied.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::require",
        module = "sand::participant",
        kind = "method",
        summary = "Require at least `required` reliability, or a [`ParticipantReliabilityError`] naming exactly what was requested vs. supplied.",
        context = "Require at least `required` reliability, or a [`ParticipantReliabilityError`] naming exactly what was requested vs. supplied. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        params(required = "Require at least `required` reliability, or a [`ParticipantReliabilityError`] naming exactly what was requested vs. supplied."),
        returns = "On success, the value produced to require at least `required` reliability, or a [`ParticipantReliabilityError`] naming exactly what was requested vs. supplied; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_participant_value: &sand::participant::PlayerParticipant, required: sand::participant::ParticipantReliability)  {\n    let require = player_participant_value.require(required);\n}",
    )]
    pub fn require(
        &self,
        required: ParticipantReliability,
    ) -> Result<&Self, ParticipantReliabilityError> {
        if self.reliability.meets(required) {
            Ok(self)
        } else {
            Err(ParticipantReliabilityError {
                role: self.role,
                requested: required,
                supplied: self.reliability,
            })
        }
    }

    /// Require [`ParticipantReliability::Exact`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::PlayerParticipant::require_exact",
        module = "sand::participant",
        kind = "method",
        summary = "Require [`ParticipantReliability::Exact`].",
        context = "Require [`ParticipantReliability::Exact`]. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "On success, the value produced to require [`ParticipantReliability::Exact`]; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_participant_value: &sand::participant::PlayerParticipant)  {\n    let require_exact = player_participant_value.require_exact();\n}",
    )]
    pub fn require_exact(&self) -> Result<&Self, ParticipantReliabilityError> {
        self.require(ParticipantReliability::Exact)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::EntityParticipant",
    module = "sand::participant",
    summary = "A typed non-player-specific entity participant reference: a command-building handle, not live runtime entity data.",
    context = "A typed non-player-specific entity participant reference: a command-building handle, not live runtime entity data. A correlated/inferred reference — the only kinds this phase's constructors beyond `subject()` can produce, since no exact non-subject-entity capture backend exists yet — never satisfies an exact-reliability requirement:",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::EntityParticipant;",
)]
/// A typed non-player-specific entity participant reference: a
/// command-building handle, not live runtime entity data.
///
/// A correlated/inferred reference — the only kinds this phase's
/// constructors beyond `subject()` can produce, since no exact
/// non-subject-entity capture backend exists yet — never satisfies an
/// exact-reliability requirement:
///
/// ```
/// use sand_core::participant::{
///     EntityParticipant, EntityParticipantRole, ParticipantLifetime,
/// };
/// use sand_commands::Target;
///
/// let attacker = EntityParticipant::correlated(
///     Target::raw_single("@e[tag=candidate,limit=1]"),
///     EntityParticipantRole::Attacker,
///     ParticipantLifetime::Invocation,
/// );
/// assert!(attacker.require_exact().is_err());
/// ```
#[derive(Debug, Clone)]
pub struct EntityParticipant {
    selector: SingleEntity,
    role: EntityParticipantRole,
    reliability: ParticipantReliability,
    lifetime: ParticipantLifetime,
}

impl EntityParticipant {
    /// The event's own subject, treated as a generic entity rather than
    /// specifically a player (for events whose subject need not be a
    /// player). Always [`ParticipantReliability::Exact`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::subject",
        module = "sand::participant",
        kind = "method",
        summary = "The event's own subject, treated as a generic entity rather than specifically a player (for events whose subject need not be a player). Always [`ParticipantReliability::Exact`].",
        context = "The event's own subject, treated as a generic entity rather than specifically a player (for events whose subject need not be a player). Always [`ParticipantReliability::Exact`]. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "An `EntityParticipant` configured for the event's own subject, treated as a generic entity rather than specifically a player (for events whose subject need not be a player). Always [`ParticipantReliability::Exact`].",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let entity_participant = sand::participant::EntityParticipant::subject();\n}",
    )]
    pub fn subject() -> Self {
        Self {
            selector: SingleEntity::self_(),
            role: EntityParticipantRole::Subject,
            reliability: ParticipantReliability::Exact,
            lifetime: ParticipantLifetime::Invocation,
        }
    }

    /// Construct a correlated entity participant reference.
    ///
    /// There is no exact-entity constructor beyond
    /// [`EntityParticipant::subject`]/[`PlayerParticipant::subject`] in
    /// Phase 8: "exact non-subject entity" requires a stable generated
    /// binding mechanism (e.g. the tag-then-target pattern
    /// `EntityScope::bind` already uses for live traversal) applied at an
    /// authoritative event boundary, which is Phase 9 observation-backend
    /// work, not a type-system concern. Correlated/inferred references
    /// remain honestly weaker than `Exact` by construction — there is no
    /// API path to mark a `selector` exact without going through
    /// [`subject`](Self::subject).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::correlated",
        module = "sand::participant",
        kind = "method",
        summary = "Construct a correlated entity participant reference.",
        context = "Construct a correlated entity participant reference. There is no exact-entity constructor beyond [`EntityParticipant::subject`]/[`PlayerParticipant::subject`] in Phase 8: \"exact non-subject entity\" requires a stable generated binding mechanism (e.g. the tag-then-target pattern `EntityScope::bind` already uses for live traversal) applied at an authoritative event boundary, which is Phase 9 observation-backend work, not a type-system concern. Correlated/inferred references remain honestly weaker than `Exact` by construction — there is no API path to mark a `selector` exact without going through [`subject`](Self::subject).",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        params(selector = "There is no exact-entity constructor beyond [`EntityParticipant::subject`]/[`PlayerParticipant::subject`] in Phase 8: \"exact non-subject entity\" requires a stable generated binding mechanism (e.g. the tag-then-target pattern `EntityScope::bind` already uses for live traversal) applied at an authoritative event boundary, which is Phase 9 observation-backend work, not a type-system concern. Correlated/inferred references remain honestly weaker than `Exact` by construction — there is no API path to mark a `selector` exact without going through [`subject`](Self::subject).", role = "`role` is used when constructing a correlated entity participant reference.", lifetime = "`lifetime` is used when constructing a correlated entity participant reference."),
        returns = "An `EntityParticipant` representing a correlated entity participant reference.",
        example = "use sand::prelude::*;\nlet entity_participant = EntityParticipant::correlated(Target::self_(), EntityParticipantRole::Attacker, ParticipantLifetime::Invocation);",
    )]
    pub fn correlated(
        selector: impl SingleTargetArgument,
        role: EntityParticipantRole,
        lifetime: ParticipantLifetime,
    ) -> Self {
        Self {
            selector: selector.into(),
            role,
            reliability: ParticipantReliability::Correlated,
            lifetime,
        }
    }

    /// Construct an inferred entity participant reference (a heuristic
    /// query result that may be ambiguous). See [`correlated`](Self::correlated).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::inferred",
        module = "sand::participant",
        kind = "method",
        summary = "Construct an inferred entity participant reference (a heuristic query result that may be ambiguous). See [`correlated`](Self::correlated).",
        context = "Construct an inferred entity participant reference (a heuristic query result that may be ambiguous). See [`correlated`](Self::correlated). Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        params(selector = "`selector` provides the Minecraft target selection used to construct an inferred entity participant reference (a heuristic query result that may be ambiguous). See [`correlated`](Self::correlated).", role = "`role` is used when constructing an inferred entity participant reference (a heuristic query result that may be ambiguous). See [`correlated`](Self::correlated).", lifetime = "`lifetime` is used when constructing an inferred entity participant reference (a heuristic query result that may be ambiguous). See [`correlated`](Self::correlated)."),
        returns = "An `EntityParticipant` representing an inferred entity participant reference (a heuristic query result that may be ambiguous). See [`correlated`](Self::correlated).",
        example = "use sand::prelude::*;\nlet entity_participant = EntityParticipant::inferred(Target::self_(), EntityParticipantRole::Attacker, ParticipantLifetime::Invocation);",
    )]
    pub fn inferred(
        selector: impl SingleTargetArgument,
        role: EntityParticipantRole,
        lifetime: ParticipantLifetime,
    ) -> Self {
        Self {
            selector: selector.into(),
            role,
            reliability: ParticipantReliability::Inferred,
            lifetime,
        }
    }

    /// Returns the entity's semantic role in the current event.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::role",
        module = "sand::participant",
        kind = "method",
        summary = "Returns the entity's semantic role in the current event.",
        context = "Returns the entity's semantic role in the current event. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns the entity's semantic role in the current event.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_participant_value: &sand::participant::EntityParticipant)  {\n    let role = entity_participant_value.role();\n}",
    )]
    pub fn role(&self) -> EntityParticipantRole {
        self.role
    }

    /// Returns the evidence strength that established this entity participant.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::reliability",
        module = "sand::participant",
        kind = "method",
        summary = "Returns the evidence strength that established this entity participant.",
        context = "Returns the evidence strength that established this entity participant. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns the evidence strength that established this entity participant.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_participant_value: &sand::participant::EntityParticipant)  {\n    let reliability = entity_participant_value.reliability();\n}",
    )]
    pub fn reliability(&self) -> ParticipantReliability {
        self.reliability
    }

    /// Returns how long the entity reference remains valid.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::lifetime",
        module = "sand::participant",
        kind = "method",
        summary = "Returns how long the entity reference remains valid.",
        context = "Returns how long the entity reference remains valid. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns how long the entity reference remains valid.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_participant_value: &sand::participant::EntityParticipant)  {\n    let lifetime = entity_participant_value.lifetime();\n}",
    )]
    pub fn lifetime(&self) -> ParticipantLifetime {
        self.lifetime
    }

    /// Returns the single-entity selector bound to this participant.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::selector",
        module = "sand::participant",
        kind = "method",
        summary = "Returns the single-entity selector bound to this participant.",
        context = "Returns the single-entity selector bound to this participant. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "Returns the single-entity selector bound to this participant.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_participant_value: &sand::participant::EntityParticipant)  {\n    let selector = entity_participant_value.selector();\n}",
    )]
    pub fn selector(&self) -> Target<AnyTarget, One> {
        self.selector.clone().into()
    }

    /// Checks that the participant evidence meets the requested reliability.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::require",
        module = "sand::participant",
        kind = "method",
        summary = "Checks that the participant evidence meets the requested reliability.",
        context = "Checks that the participant evidence meets the requested reliability. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        params(required = "`required` is the required checked to determine that the participant evidence meets the requested reliability."),
        returns = "On success, the value produced to check that the participant evidence meets the requested reliability; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_participant_value: &sand::participant::EntityParticipant, required: sand::participant::ParticipantReliability)  {\n    let require = entity_participant_value.require(required);\n}",
    )]
    pub fn require(
        &self,
        required: ParticipantReliability,
    ) -> Result<&Self, ParticipantReliabilityError> {
        if self.reliability.meets(required) {
            Ok(self)
        } else {
            Err(ParticipantReliabilityError {
                role: self.role,
                requested: required,
                supplied: self.reliability,
            })
        }
    }

    /// Rejects participant evidence that is not exact for this handler operation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::require_exact",
        module = "sand::participant",
        kind = "method",
        summary = "Rejects participant evidence that is not exact for this handler operation.",
        context = "Rejects participant evidence that is not exact for this handler operation. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
        minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        returns = "On success, the value produced to reject participant evidence that is not exact for this handler operation; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_participant_value: &sand::participant::EntityParticipant)  {\n    let require_exact = entity_participant_value.require_exact();\n}",
    )]
    pub fn require_exact(&self) -> Result<&Self, ParticipantReliabilityError> {
        self.require(ParticipantReliability::Exact)
    }

    /// `execute as <this participant's selector> at @s run <cmd>` — run a
    /// typed command with this participant as *both* the executing entity
    /// and the execution position, without ever stringifying the selector
    /// yourself.
    ///
    /// This is the normal way to consume a resolved [`EntityParticipant`]:
    /// build `cmd` with any other typed command builder (targeting `@s`,
    /// which — via the leading `execute as <participant>` — resolves to
    /// *this participant*, not the caller).
    ///
    /// # A real, live-fire-confirmed bug this method used to have
    ///
    /// Before this fix, `execute_at` generated a bare `execute at <selector>
    /// run <cmd>`. Vanilla's `execute at` only moves the execution
    /// *position* — it never rebinds the executing entity (`@s`). Every
    /// caller of this method builds `cmd` with [`sand_commands::Target::self_`]
    /// specifically to reference "this participant" — so with the old
    /// `execute at`-only form, `@s` inside `cmd` silently kept resolving to
    /// whatever `@s` already was in the *caller's* context (typically the
    /// event's subject, e.g. the victim of an attack), never to the
    /// participant this method exists to address. Every structural/export
    /// test asserted the exact (wrong) generated string, so nothing caught
    /// it — real Minecraft 26.2 runtime validation for #265 did: a summoned
    /// "attacker" zombie's real combat relation was captured via
    /// `execute on attacker` correctly, but reading its UUID back out
    /// through `execute_at` + `@s` produced the *victim's* UUID every time.
    /// See `docs/testing/participant-role-evidence.md` for the exact
    /// before/after evidence. Leading `execute as` fixes this: `as`
    /// rebinds `@s`, and the trailing `at @s` (now referring to the new,
    /// correct `@s`) preserves this method's original position-moving
    /// behavior for any `cmd` that also needs it (e.g. relative
    /// coordinates, particle/sound effects).
    ///
    /// ```
    /// use sand_core::participant::{EntityParticipant, EntityParticipantRole};
    /// use sand_core::participant::lifetime::ParticipantLifetime;
    /// use sand_core::state::StorageSchema;
    /// use sand_commands::Target;
    ///
    /// let attacker = EntityParticipant::correlated(
    ///     Target::raw_single("@e[tag=x,limit=1]"),
    ///     EntityParticipantRole::Attacker,
    ///     ParticipantLifetime::SynchronousDescendants,
    /// );
    /// static AUDIT: StorageSchema<()> = StorageSchema::new("pack:audit", "audit");
    /// let cmd = attacker.execute_at(AUDIT.field::<String>("attacker_uuid").copy_from_entity(Target::self_(), "UUID"));
    /// assert_eq!(
    ///     cmd,
    ///     "execute as @e[tag=x,limit=1] at @s run data modify storage pack:audit audit.attacker_uuid set from entity @s UUID"
    /// );
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::participant::EntityParticipant::execute_at",
        module = "sand::participant",
        kind = "method",
        summary = "`execute as <this participant's selector> at @s run <cmd>` — run a typed command with this participant as *both* the executing entity and the execution position, without ever stringifying the selector yourself.",
        context = "`execute as <this participant's selector> at @s run <cmd>` — run a typed command with this participant as *both* the executing entity and the execution position, without ever stringifying the selector yourself. This is the normal way to consume a resolved [`EntityParticipant`]: build `cmd` with any other typed command builder (targeting `@s`, which — via the leading `execute as <participant>` — resolves to *this participant*, not the caller). Before this fix, `execute_at` generated a bare `execute at <selector> run <cmd>`. Vanilla's `execute at` only moves the execution *position* — it never rebinds the executing entity (`@s`). Every caller of this method builds `cmd` with [`sand::command::Target::self_`] specifically to reference \"this participant\" — so with the old `execute at`-only form, `@s` inside `cmd` silently kept resolving to whatever `@s` already was in the *caller's* context (typically the event's subject, e.g. the victim of an attack), never to the participant this method exists to address. Every structural/export test asserted the exact (wrong) generated string, so nothing caught it — real Minecraft 26.2 runtime validation for #265 did: a summoned \"attacker\" zombi...",
        minecraft = "This is the normal way to consume a resolved [`EntityParticipant`]: build `cmd` with any other typed command builder (targeting `@s`, which — via the leading `execute as <participant>` — resolves to *this participant*, not the caller).",
        use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
        avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
        params(cmd = "This is the normal way to consume a resolved [`EntityParticipant`]: build `cmd` with any other typed command builder (targeting `@s`, which — via the leading `execute as <participant>` — resolves to *this participant*, not the caller)."),
        returns = "The rendered Minecraft command text produced to emit the documented `execute as <this participant's selector> at @s run <cmd>` — run a typed command with this participant as *both* the executing entity and the execution position, without ever stringifying the selector yourself form.",
        example = "use {sand::participant::EntityParticipant, sand::participant::EntityParticipantRole};\nuse sand::participant::ParticipantLifetime;\nuse sand::data::StorageSchema;\nuse sand::command::Target;\nlet attacker = EntityParticipant::correlated(\nTarget::raw_single(\"@e[tag=x,limit=1]\"),\nEntityParticipantRole::Attacker,\nParticipantLifetime::SynchronousDescendants,\n);\nstatic AUDIT: StorageSchema<()> = StorageSchema::new(\"pack:audit\", \"audit\");\nlet cmd = attacker.execute_at(AUDIT.field::<String>(\"attacker_uuid\").copy_from_entity(Target::self_(), \"UUID\"));\nassert_eq!(\ncmd,\n\"execute as @e[tag=x,limit=1] at @s run data modify storage pack:audit audit.attacker_uuid set from entity @s UUID\"\n);",
    )]
    pub fn execute_at(&self, cmd: impl Into<String>) -> String {
        format!("execute as {} at @s run {}", self.selector, cmd.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_subject_is_exact_and_invocation_scoped() {
        let subject = PlayerParticipant::subject();
        assert_eq!(subject.role(), EntityParticipantRole::Subject);
        assert_eq!(subject.reliability(), ParticipantReliability::Exact);
        assert_eq!(subject.lifetime(), ParticipantLifetime::Invocation);
        assert!(subject.require_exact().is_ok());
    }

    #[test]
    fn player_subject_selector_renders_as_self() {
        assert_eq!(PlayerParticipant::subject().selector().to_string(), "@s");
    }

    #[test]
    fn entity_subject_is_exact() {
        let subject = EntityParticipant::subject();
        assert_eq!(subject.reliability(), ParticipantReliability::Exact);
        assert!(subject.require_exact().is_ok());
    }

    #[test]
    fn correlated_entity_does_not_satisfy_exact_requirement() {
        let attacker = EntityParticipant::correlated(
            SingleEntity::raw("@e[tag=candidate,limit=1]"),
            EntityParticipantRole::Attacker,
            ParticipantLifetime::Invocation,
        );
        let err = attacker.require_exact().unwrap_err();
        assert_eq!(err.role, EntityParticipantRole::Attacker);
        assert_eq!(err.requested, ParticipantReliability::Exact);
        assert_eq!(err.supplied, ParticipantReliability::Correlated);
    }

    #[test]
    fn inferred_entity_does_not_satisfy_correlated_requirement() {
        let target = EntityParticipant::inferred(
            SingleEntity::raw("@e[type=zombie,limit=1,sort=nearest]"),
            EntityParticipantRole::Target,
            ParticipantLifetime::Invocation,
        );
        assert!(target.require(ParticipantReliability::Correlated).is_err());
    }

    #[test]
    fn reliability_error_message_names_role_and_levels() {
        let target = EntityParticipant::inferred(
            SingleEntity::raw("@e[limit=1]"),
            EntityParticipantRole::Victim,
            ParticipantLifetime::Invocation,
        );
        let err = target.require_exact().unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Victim"));
        assert!(message.contains("Exact"));
        assert!(message.contains("Inferred"));
    }
}
