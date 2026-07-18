//! Correlated entity participant observation (#230 Phase 9).
//!
//! This is Sand's first real participant-*recovery* backend, built directly
//! on top of Phase 8's typed reliability model
//! ([`super::reliability::ParticipantReliability`]) and the existing
//! vanilla-relation traversal machinery
//! ([`crate::entity::relation::Relation`]/[`crate::entity::context::EntityScope`]).
//! It deliberately implements exactly one observation mechanism —
//! [`observe_correlated_attacker`], backed by vanilla's `execute on attacker`
//! relation — rather than a general-purpose proximity/heuristic observer.
//!
//! # Why `execute on attacker` and nothing else
//!
//! `execute on attacker` (Minecraft 1.20.2+) is a direct vanilla relation
//! query: "the entity that last damaged the current entity." It is
//! single-valued by construction — [`crate::entity::relation::RelationQuery<One>`]
//! resolves to at most one entity, never a set — so there is no "multiple
//! credible candidates" ambiguity case to police here the way there would be
//! for a proximity-based guess. That is *why* this phase implements only
//! this mechanism: it is evidence Sand did not have to invent a selection
//! policy for. A future Phase 10 observer built on `@e[distance=..]`-style
//! proximity queries would need real ambiguity handling
//! (`AmbiguousCandidates` from Phase 8) and would be `Inferred`, not
//! `Correlated` — this module does not attempt that.
//!
//! # Why `Correlated`, not `Exact`
//!
//! `execute on attacker` is a genuine, direct vanilla relation query, not a
//! heuristic — but this module still reports
//! [`ParticipantReliability::Correlated`](crate::participant::reliability::ParticipantReliability::Correlated), never `Exact`, because:
//!
//! - There is no verified guarantee that vanilla's internal "last attacker"
//!   memory is updated synchronously with, and scoped exactly to, the
//!   specific damage event that fired the advancement criterion this
//!   observation is embedded in (as opposed to reflecting an earlier hit in
//!   the same tick, or lagging by one damage event in some edge case). No
//!   real-server evidence proves otherwise (see
//!   `ai/known-limitations.md` `LIM-VAL-010`).
//! - `ParticipantReliability::Exact` in Sand's model is reserved for
//!   references the *triggering mechanism itself* directly hands over (the
//!   advancement reward function's own `@s`). The attacker here is reached
//!   through an additional relation traversal Sand performs itself, one
//!   step removed from that guarantee.
//!
//! # Observation source, key, and lifetime
//!
//! [`observe_correlated_attacker`] generates:
//!
//! 1. An unconditional reset of a presence flag in command storage, keyed
//!    deterministically by an `event_label` the caller supplies (same
//!    FNV-1a scheme [`crate::item::ItemSnapshot`] uses for its own
//!    storage keys — see [`ObservationSchema::new`]).
//! 2. Two single-command `execute on attacker run <command>` lines — mark
//!    present, then tag the attacker — emitted directly rather than
//!    through [`crate::entity::context::EntityContext::attacker`]'s
//!    `if_present` multi-command wrapper (see the "Why not
//!    `if_present`" note below). Silently a no-op if the relation is
//!    absent — vanilla's own `execute on` semantics, unrelated to the
//!    version gating performed separately before any commands are built.
//!
//! ## Why not `if_present`
//!
//! An earlier version of this function routed the mark+tag step through
//! [`crate::entity::context::EntityContext::attacker`]`().if_present(...)`,
//! which wraps a multi-command body into a separately-registered dynamic
//! function (`register_dyn_fn_dedup`). That produced a real, reproduced
//! nondeterminism bug: two `try_export_components_json` calls in the same
//! process for identical input returned different JSON, because that
//! registration's timing relative to the exporter's `drain_dyn_fns()` call
//! was not consistent across repeated exports when triggered from inside
//! `SandEvent::setup()` (see `ai/known-limitations.md` `LIM-EXP-006` — the
//! root cause was not investigated further). Since the mark+tag step is
//! only two single commands, each fits directly on its own `execute on
//! attacker run <command>` line with no wrapping needed at all, which
//! sidesteps the bug entirely rather than working around it.
//! 3. The caller's `body`, receiving a [`CorrelatedEntityObservation`]
//!    whose `.participant()` selector addresses the tagged entity by a
//!    unique temporary tag (`__sand_observed_<key>`) — never a bare
//!    `@e[type=...]` query that could match the wrong entity.
//! 4. An unconditional cleanup removing that tag from whatever (zero or
//!    one) entity currently holds it.
//!
//! The observation is valid for [`super::lifetime::ParticipantLifetime::SynchronousDescendants`]
//! — the tag is added and removed within one straight-line generated
//! command sequence, so it remains addressable through the caller's `body`
//! (including same-cycle synchronous children reached from within it) and
//! is gone by the time that sequence returns. It is never persisted across
//! ticks; there is no window to refresh or expire, because this is not a
//! bounded multi-tick correlation like Phase 5's `.within(...)` — it is an
//! instantaneous relation query evaluated once, synchronously, at the point
//! `observe_correlated_attacker` is embedded. Conflating "this event fired
//! recently" with "this entity was observed recently" is exactly what this
//! module avoids by not reusing `.within(...)`'s tick-window infrastructure
//! here at all.
//!
//! # Multiplayer safety
//!
//! The presence flag's storage path is one deterministic path per
//! `event_label` (not per-player), safe under the same argument
//! [`crate::item::ItemSnapshot`]'s module doc gives in full: `execute as @a`
//! runs one player's entire synchronous call tree to completion before the
//! next, so no other player's observation can land between this reset and
//! this cleanup. The identity tag itself is scoped to one invocation by a
//! process-global counter-free but call-site-unique name
//! (`__sand_observed_<key>`, `key` derived from `event_label`) — two
//! *concurrent* observations for the *same* event type would collide if
//! nested inside one another before the first's cleanup runs (see
//! `ai/known-limitations.md` `LIM-CTX-005`), which is why nesting two calls
//! to `observe_correlated_attacker` for the same `event_label` inside one
//! synchronous call tree is not supported and is documented as such rather
//! than silently allowed.

use crate::condition::Condition;
use crate::entity::context::EntityContext;
use crate::entity::kind::EntityKind;
use crate::entity::relation::Relation;
use crate::events::graph::tick_event_resource_key;
use crate::participant::lifetime::ParticipantLifetime;
use crate::participant::reference::EntityParticipant;
use crate::participant::role::EntityParticipantRole;
use crate::version::VersionProfile;
use sand_commands::selector::SingleEntity;

/// The vanilla mechanism an observation's evidence comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CorrelationSource {
    /// `execute on attacker` — vanilla's own "last entity that damaged me"
    /// relation. The only source this phase implements.
    AttackerRelation,
}

/// The evidence backing a correlated participant, exposed alongside the
/// participant itself so callers (and diagnostics) can see *why* something
/// is `Correlated` rather than treating the label as unexplained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorrelationEvidence {
    pub source: CorrelationSource,
    /// `(major, minor, patch)` — the vanilla version this evidence source
    /// requires. `execute on attacker` requires 1.20.2+.
    pub min_version: (u32, u32, u32),
}

impl CorrelationEvidence {
    pub const ATTACKER_RELATION: CorrelationEvidence = CorrelationEvidence {
        source: CorrelationSource::AttackerRelation,
        min_version: (1, 20, 2),
    };
}

/// Deterministic identity for one observation's generated storage path and
/// temporary tag, derived from a caller-supplied event label the same way
/// [`crate::item::snapshot::SnapshotSchema`] derives its own key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationSchema {
    storage: String,
    key: String,
}

impl ObservationSchema {
    pub fn new(storage: impl Into<String>, event_label: &str) -> Self {
        Self {
            storage: storage.into(),
            key: tick_event_resource_key(event_label),
        }
    }

    pub fn storage(&self) -> &str {
        &self.storage
    }

    fn present_path(&self) -> String {
        format!("obs.{}", self.key)
    }

    fn tag(&self) -> String {
        format!("__sand_observed_{}", self.key)
    }
}

/// An observation that failed to construct — never a runtime "no candidate
/// found" outcome (that is represented by
/// [`CorrelatedEntityObservation::is_absent`] at generated-command time,
/// since Sand cannot know at export time whether vanilla will find an
/// attacker). This is a build-time/version diagnostic only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationError {
    /// The active `VersionProfile` predates the evidence source's minimum
    /// version.
    UnsupportedVersion {
        role: EntityParticipantRole,
        evidence: CorrelationEvidence,
        target_version: String,
    },
}

impl std::fmt::Display for ObservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion {
                role,
                evidence,
                target_version,
            } => {
                let (major, minor, patch) = evidence.min_version;
                write!(
                    f,
                    "participant role {role:?} via {:?} requires Minecraft {major}.{minor}.{patch}+, but the target version is {target_version} — remove this observation or select a supported target version",
                    evidence.source
                )
            }
        }
    }
}

impl std::error::Error for ObservationError {}

/// A correlated entity observation embedded into a generated command
/// sequence: an immutable handle describing generated storage/tag identity,
/// not a live runtime value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelatedEntityObservation {
    schema: ObservationSchema,
    role: EntityParticipantRole,
    evidence: CorrelationEvidence,
}

impl CorrelatedEntityObservation {
    /// A `Condition` true exactly when the underlying relation resolved to
    /// an entity for this invocation.
    pub fn is_present(&self) -> Condition {
        Condition::StorageExists {
            location: self.schema.storage().to_string(),
            path: format!("{}{{present:1b}}", self.schema.present_path()),
        }
    }

    /// The negation of [`Self::is_present`].
    pub fn is_absent(&self) -> Condition {
        Condition::negate(self.is_present())
    }

    /// The correlated entity participant handle. Always
    /// [`ParticipantReliability::Correlated`](crate::participant::reliability::ParticipantReliability::Correlated),
    /// [`ParticipantLifetime::SynchronousDescendants`] — see the module
    /// doc. Only meaningful when [`Self::is_present`] holds; the caller is
    /// responsible for guarding its use accordingly (Sand cannot express
    /// "this Rust value only exists when true" for a runtime-only fact —
    /// the same honest limitation `ItemSnapshot` documents).
    pub fn participant(&self) -> EntityParticipant {
        EntityParticipant::correlated(
            SingleEntity::raw(format!("@e[tag={},limit=1]", self.schema.tag())),
            self.role,
            ParticipantLifetime::SynchronousDescendants,
        )
    }

    pub fn evidence(&self) -> CorrelationEvidence {
        self.evidence
    }

    pub fn role(&self) -> EntityParticipantRole {
        self.role
    }

    /// Commands that unconditionally remove the temporary observation tag,
    /// regardless of whether an attacker was actually found. Already
    /// appended automatically at the end of
    /// [`observe_correlated_attacker`]'s generated sequence — exposed
    /// separately only so a caller building a non-linear composition can
    /// place it explicitly.
    pub fn cleanup_commands(&self) -> Vec<String> {
        vec![sand_commands::builtins::tag_remove(
            sand_commands::selector::Selector::all_entities().tag(self.schema.tag()),
            self.schema.tag(),
        )]
    }
}

/// Observe the entity that last damaged (`execute on attacker`) the entity
/// currently bound to `@s`, and embed `body`'s commands so they run with
/// the observation still bound (see the module doc for exact ordering and
/// lifetime).
///
/// `ctx` is the [`EntityContext`] for whichever entity is the intended
/// *victim* — e.g. call this from a handler where `@s` is already the
/// exact player subject of an `EntityHurtPlayer`-backed event
/// ([`EntityDamagePlayerEvent`](crate::events::EntityDamagePlayerEvent)) or
/// an `EntityKilledPlayer`-backed event
/// ([`PlayerKillEvent`](crate::events::PlayerKillEvent), where the role
/// would be [`EntityParticipantRole::Killer`] instead of
/// [`EntityParticipantRole::Attacker`] — the underlying mechanism is
/// identical, only the role and the event's own semantics differ).
///
/// Returns [`ObservationError::UnsupportedVersion`] if `profile` predates
/// `execute on attacker` (Minecraft 1.20.2). Does not fail for "no
/// attacker found" — that is a runtime fact, checked via
/// [`CorrelatedEntityObservation::is_present`]/`is_absent`, not a build-time
/// error.
///
/// `_ctx` is not read — it exists purely to type-constrain callers to a
/// point where `@s` is a known [`EntityContext`], the same
/// documentation-only-parameter convention
/// [`EntityScope::bind`](crate::entity::EntityScope::bind) already uses.
pub fn observe_correlated_attacker<K: EntityKind>(
    _ctx: &EntityContext<K>,
    profile: &VersionProfile,
    schema: ObservationSchema,
    role: EntityParticipantRole,
    body: impl FnOnce(&CorrelatedEntityObservation) -> Vec<String>,
) -> std::result::Result<Vec<String>, ObservationError> {
    if Relation::Attacker.check_supported(profile).is_err() {
        return Err(ObservationError::UnsupportedVersion {
            role,
            evidence: CorrelationEvidence::ATTACKER_RELATION,
            target_version: profile.resolved_name.clone(),
        });
    }

    let present_path = schema.present_path();
    let storage = schema.storage().to_string();
    let tag = schema.tag();

    // Each `execute on attacker run <command>` line runs a single command
    // directly, inline — deliberately not routed through
    // `EntityContext::attacker().if_present(...)`'s multi-command
    // dynamic-function-wrapping (`RelationQuery::lower`), which registers a
    // separate generated function via `register_dyn_fn_dedup` from inside
    // `SandEvent::setup()`. That registration happens at a point in the
    // export pipeline relative to `drain_dyn_fns()` that is not guaranteed
    // deterministic across repeated exports in the same process (see the
    // regression this avoided: a first/second `try_export_components_json`
    // call could produce different output for the same input). Only two
    // single-command lines are needed here, so the wrapping was never
    // necessary in the first place.
    let mut commands = vec![format!(
        "data modify storage {storage} {present_path}.present set value 0b"
    )];
    commands.push(format!(
        "execute on attacker run data modify storage {storage} {present_path}.present set value 1b"
    ));
    commands.push(format!("execute on attacker run tag @s add {tag}"));

    let observation = CorrelatedEntityObservation {
        schema: schema.clone(),
        role,
        evidence: CorrelationEvidence::ATTACKER_RELATION,
    };
    commands.extend(body(&observation));
    commands.extend(observation.cleanup_commands());

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::kind::PlayerKind;
    use crate::participant::reliability::ParticipantReliability;
    use crate::version::MinecraftVersion;

    fn profile(version: &str) -> VersionProfile {
        VersionProfile::resolve(&MinecraftVersion::parse(version).unwrap()).unwrap()
    }

    #[test]
    fn unsupported_version_is_rejected_before_any_commands_are_generated() {
        let ctx: EntityContext<PlayerKind> = EntityContext::default();
        let schema = ObservationSchema::new("mypack:observations", "TestVictimEvent");
        let err = observe_correlated_attacker(
            &ctx,
            &profile("1.19.4"),
            schema,
            EntityParticipantRole::Attacker,
            |_| vec!["say should not run".to_string()],
        )
        .unwrap_err();
        assert!(matches!(err, ObservationError::UnsupportedVersion { .. }));
    }

    #[test]
    fn supported_version_generates_reset_mark_body_cleanup_in_order() {
        let ctx: EntityContext<PlayerKind> = EntityContext::default();
        let schema = ObservationSchema::new("mypack:observations", "TestVictimEvent");
        let commands = observe_correlated_attacker(
            &ctx,
            &profile("1.21.4"),
            schema,
            EntityParticipantRole::Attacker,
            |observation| {
                assert_eq!(observation.role(), EntityParticipantRole::Attacker);
                vec!["say handler ran".to_string()]
            },
        )
        .unwrap();

        let reset_index = commands
            .iter()
            .position(|c| c.contains("present set value 0b"))
            .expect("reset command present");
        let mark_index = commands
            .iter()
            .position(|c| c.starts_with("execute on attacker run"))
            .expect("mark/bind command present");
        let body_index = commands
            .iter()
            .position(|c| c == "say handler ran")
            .expect("body command present");
        let cleanup_index = commands
            .iter()
            .position(|c| c.starts_with("tag @e[tag=__sand_observed_"))
            .expect("cleanup command present");

        assert!(
            reset_index < mark_index,
            "reset must run before the mark/bind"
        );
        assert!(
            mark_index < body_index,
            "mark/bind must run before the body"
        );
        assert!(
            body_index < cleanup_index,
            "cleanup must run after the body"
        );
    }

    #[test]
    fn distinct_event_labels_produce_distinct_storage_paths_and_tags() {
        let ctx: EntityContext<PlayerKind> = EntityContext::default();
        let a = observe_correlated_attacker(
            &ctx,
            &profile("1.21.4"),
            ObservationSchema::new("mypack:observations", "EventA"),
            EntityParticipantRole::Attacker,
            |_| vec![],
        )
        .unwrap();
        let b = observe_correlated_attacker(
            &ctx,
            &profile("1.21.4"),
            ObservationSchema::new("mypack:observations", "EventB"),
            EntityParticipantRole::Attacker,
            |_| vec![],
        )
        .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn repeated_observation_for_the_same_schema_is_deterministic() {
        let ctx: EntityContext<PlayerKind> = EntityContext::default();
        let make = || {
            observe_correlated_attacker(
                &ctx,
                &profile("1.21.4"),
                ObservationSchema::new("mypack:observations", "SameEvent"),
                EntityParticipantRole::Attacker,
                |_| vec![],
            )
            .unwrap()
        };
        assert_eq!(make(), make());
    }

    #[test]
    fn is_present_and_is_absent_are_exact_negations() {
        let schema = ObservationSchema::new("mypack:observations", "PresenceEvent");
        let observation = CorrelatedEntityObservation {
            schema,
            role: EntityParticipantRole::Attacker,
            evidence: CorrelationEvidence::ATTACKER_RELATION,
        };
        match (observation.is_present(), observation.is_absent()) {
            (Condition::StorageExists { .. }, Condition::Not(inner)) => {
                assert!(matches!(*inner, Condition::StorageExists { .. }));
            }
            other => panic!("unexpected condition shape: {other:?}"),
        }
    }

    #[test]
    fn participant_reliability_is_always_correlated_never_exact() {
        let schema = ObservationSchema::new("mypack:observations", "ReliabilityEvent");
        let observation = CorrelatedEntityObservation {
            schema,
            role: EntityParticipantRole::Killer,
            evidence: CorrelationEvidence::ATTACKER_RELATION,
        };
        let participant = observation.participant();
        assert_eq!(
            participant.reliability(),
            ParticipantReliability::Correlated
        );
        assert!(participant.require_exact().is_err());
        assert!(
            participant
                .require(ParticipantReliability::Correlated)
                .is_ok()
        );
    }

    #[test]
    fn cleanup_commands_target_only_the_generated_observation_tag() {
        let schema = ObservationSchema::new("mypack:observations", "CleanupEvent");
        let observation = CorrelatedEntityObservation {
            schema: schema.clone(),
            role: EntityParticipantRole::Attacker,
            evidence: CorrelationEvidence::ATTACKER_RELATION,
        };
        let cleanup = observation.cleanup_commands();
        assert_eq!(cleanup.len(), 1);
        assert!(cleanup[0].contains(&schema.tag()));
    }
}
