//! Typed generated-command participant references (#230 Phase 8).

use sand_commands::selector::{SingleEntity, SinglePlayer};

use super::lifetime::ParticipantLifetime;
use super::reliability::ParticipantReliability;
use super::role::EntityParticipantRole;

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
    pub role: EntityParticipantRole,
    pub requested: ParticipantReliability,
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
///     selector: sand_commands::selector::SinglePlayer::self_(),
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
    pub fn subject() -> Self {
        Self {
            selector: SinglePlayer::self_(),
            role: EntityParticipantRole::Subject,
            reliability: ParticipantReliability::Exact,
            lifetime: ParticipantLifetime::Invocation,
        }
    }

    pub fn role(&self) -> EntityParticipantRole {
        self.role
    }

    pub fn reliability(&self) -> ParticipantReliability {
        self.reliability
    }

    pub fn lifetime(&self) -> ParticipantLifetime {
        self.lifetime
    }

    /// The typed selector for building commands against this participant.
    /// Never exposes a raw/unrestricted selector string — the caller gets
    /// [`SinglePlayer`]'s own safe builder surface.
    pub fn selector(&self) -> &SinglePlayer {
        &self.selector
    }

    /// Require at least `required` reliability, or a
    /// [`ParticipantReliabilityError`] naming exactly what was requested
    /// vs. supplied.
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
    pub fn require_exact(&self) -> Result<&Self, ParticipantReliabilityError> {
        self.require(ParticipantReliability::Exact)
    }
}

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
/// use sand_commands::selector::SingleEntity;
///
/// let attacker = EntityParticipant::correlated(
///     SingleEntity::raw("@e[tag=candidate,limit=1]"),
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
    pub fn correlated(
        selector: SingleEntity,
        role: EntityParticipantRole,
        lifetime: ParticipantLifetime,
    ) -> Self {
        Self {
            selector,
            role,
            reliability: ParticipantReliability::Correlated,
            lifetime,
        }
    }

    /// Construct an inferred entity participant reference (a heuristic
    /// query result that may be ambiguous). See [`correlated`](Self::correlated).
    pub fn inferred(
        selector: SingleEntity,
        role: EntityParticipantRole,
        lifetime: ParticipantLifetime,
    ) -> Self {
        Self {
            selector,
            role,
            reliability: ParticipantReliability::Inferred,
            lifetime,
        }
    }

    pub fn role(&self) -> EntityParticipantRole {
        self.role
    }

    pub fn reliability(&self) -> ParticipantReliability {
        self.reliability
    }

    pub fn lifetime(&self) -> ParticipantLifetime {
        self.lifetime
    }

    pub fn selector(&self) -> &SingleEntity {
        &self.selector
    }

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

    pub fn require_exact(&self) -> Result<&Self, ParticipantReliabilityError> {
        self.require(ParticipantReliability::Exact)
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
        assert_eq!(
            PlayerParticipant::subject()
                .selector()
                .selector()
                .to_string(),
            "@s"
        );
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
