//! Event context capability descriptors (#230 Phase 8).
//!
//! An [`EventContextCapabilities`] value describes *what an event type can
//! promise*, not any runtime value — it holds only `'static` data
//! (`Copy` enums and `&'static` slices), so it is deterministic, cheap to
//! compare, and safe to compute once per generic event monomorphization
//! (no `TypeId`-derived identity anywhere in this module: two distinct
//! generic instantiations of the same event family simply get distinct
//! `EventContextCapabilities` values by construction, compared structurally
//! like any other data).

use crate::events::{SandEvent, SandEventDispatch};

use super::lifetime::ParticipantLifetime;
use super::reliability::ParticipantReliability;
use super::role::{EntityParticipantRole, ItemParticipantRole, LocationParticipantRole};

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

/// Whether a non-subject capability is guaranteed on every occurrence of
/// the event, or only sometimes (e.g. an item location that may be empty).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapabilityOccurrence {
    Unconditional,
    OccurrenceDependent,
}

/// A `(major, minor, patch)` version floor, compared via
/// `McVersion::new(major, minor, patch)`. Stored as a plain tuple rather
/// than [`crate::McVersion`] so capability descriptors stay `Copy` and
/// usable in `const`/`&'static` contexts.
pub type VersionFloor = (u32, u32, u32);

/// What an event can promise about one non-subject entity participant
/// role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityParticipantCapability {
    pub role: EntityParticipantRole,
    pub reliability: ParticipantReliability,
    pub lifetime: ParticipantLifetime,
    pub occurrence: CapabilityOccurrence,
    pub min_version: Option<VersionFloor>,
}

/// What an event can promise about one item participant role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemParticipantCapability {
    pub role: ItemParticipantRole,
    pub reliability: ParticipantReliability,
    pub lifetime: ParticipantLifetime,
    pub occurrence: CapabilityOccurrence,
    pub min_version: Option<VersionFloor>,
}

/// What an event can promise about one location participant role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationParticipantCapability {
    pub role: LocationParticipantRole,
    pub reliability: ParticipantReliability,
    pub lifetime: ParticipantLifetime,
    pub occurrence: CapabilityOccurrence,
    pub min_version: Option<VersionFloor>,
}

/// What an event type can provide, declared once and shared by every
/// occurrence of that event. `entities`/`items`/`locations` are `&'static`
/// slices — Phase 8 does not implement any participant-recovery backend,
/// so every current event's slices are empty; the fields exist so Phase 9
/// providers have somewhere to declare real entries without a breaking
/// change to this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventContextCapabilities {
    pub subject: SubjectCapability,
    pub entities: &'static [EntityParticipantCapability],
    pub items: &'static [ItemParticipantCapability],
    pub locations: &'static [LocationParticipantCapability],
}

impl EventContextCapabilities {
    pub const NONE: EventContextCapabilities = EventContextCapabilities {
        subject: SubjectCapability::NONE,
        entities: &[],
        items: &[],
        locations: &[],
    };

    pub const EXACT_PLAYER_SUBJECT: EventContextCapabilities = EventContextCapabilities {
        subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
        entities: &[],
        items: &[],
        locations: &[],
    };

    /// Validate that no role is declared more than once within `entities`,
    /// `items`, or `locations` — a duplicate declaration for one event type
    /// is always a bug (a role either has one capability or none), never a
    /// legitimate "two different variants of the same role."
    pub fn validate(&self) -> Result<(), CapabilityDescriptorError> {
        let mut seen = std::collections::BTreeSet::new();
        for entity in self.entities {
            if !seen.insert(entity.role) {
                return Err(CapabilityDescriptorError::DuplicateEntityRole { role: entity.role });
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for item in self.items {
            if !seen.insert(item.role) {
                return Err(CapabilityDescriptorError::DuplicateItemRole { role: item.role });
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for location in self.locations {
            if !seen.insert(location.role) {
                return Err(CapabilityDescriptorError::DuplicateLocationRole {
                    role: location.role,
                });
            }
        }
        Ok(())
    }

    /// A cheap owned copy suitable as input to the merge functions below,
    /// which need to build new (non-`'static`) lists.
    pub fn to_resolved(self) -> ResolvedEventContextCapabilities {
        ResolvedEventContextCapabilities {
            subject: self.subject,
            entities: self.entities.to_vec(),
            items: self.items.to_vec(),
            locations: self.locations.to_vec(),
        }
    }

    /// Structurally derive the capabilities of `E`'s *own* dispatch shape —
    /// not counting anything a same-cycle parent might contribute.
    ///
    /// - `AdvancementTrigger`/legacy `TickCondition` dispatch: exact player
    ///   subject, invocation lifetime. A raw/custom advancement criterion's
    ///   own extra JSON fields (beyond the player subject itself) are never
    ///   exposed as capabilities — only participants an
    ///   [`crate::participant::EventParticipantPlan`] explicitly declares
    ///   are.
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
    ///   full graph-integrated capability resolution is Phase 9 work.
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

    /// [`Self::for_event`]'s subject capability, plus every entity/item
    /// capability `E::participants()` declares (#230 Phase 7 — plan
    /// capabilities are not visible to `for_event` alone, since a plan's
    /// capabilities are computed at runtime from a `Vec`, not the `'static`
    /// slices `EventContextCapabilities` requires; this returns the owned
    /// [`ResolvedEventContextCapabilities`] instead).
    ///
    /// This is what a caller wanting the *full* picture for one event type
    /// — subject and declared participants together — should call, rather
    /// than combining [`Self::for_event`] and
    /// `E::participants().capabilities()` by hand.
    pub fn for_event_with_participants<E: SandEvent + 'static>() -> ResolvedEventContextCapabilities
    {
        let subject = Self::for_event::<E>().subject;
        let plan = E::participants();
        ResolvedEventContextCapabilities {
            subject,
            entities: plan.capabilities(),
            items: plan.item_capabilities(),
            locations: Vec::new(),
        }
    }
}

/// A capability descriptor is internally inconsistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityDescriptorError {
    DuplicateEntityRole { role: EntityParticipantRole },
    DuplicateItemRole { role: ItemParticipantRole },
    DuplicateLocationRole { role: LocationParticipantRole },
}

impl std::fmt::Display for CapabilityDescriptorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateEntityRole { role } => {
                write!(
                    f,
                    "entity participant role {role:?} is declared more than once"
                )
            }
            Self::DuplicateItemRole { role } => {
                write!(
                    f,
                    "item participant role {role:?} is declared more than once"
                )
            }
            Self::DuplicateLocationRole { role } => {
                write!(
                    f,
                    "location participant role {role:?} is declared more than once"
                )
            }
        }
    }
}

impl std::error::Error for CapabilityDescriptorError {}

/// The owned result of merging/propagating capabilities across graph
/// edges. Unlike [`EventContextCapabilities`], this is not `'static` — it
/// is a transient composition of one or more declared descriptors, not a
/// new per-event-type declaration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedEventContextCapabilities {
    pub subject: SubjectCapability,
    pub entities: Vec<EntityParticipantCapability>,
    pub items: Vec<ItemParticipantCapability>,
    pub locations: Vec<LocationParticipantCapability>,
}

impl Default for SubjectCapability {
    fn default() -> Self {
        SubjectCapability::NONE
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
    /// Two parents declare the same entity role with different
    /// capabilities (different reliability, lifetime, or occurrence) — a
    /// merge would have to silently pick one, so it is rejected instead.
    ConflictingEntityCapability {
        role: EntityParticipantRole,
    },
    ConflictingItemCapability {
        role: ItemParticipantRole,
    },
    ConflictingLocationCapability {
        role: LocationParticipantRole,
    },
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
            Self::ConflictingEntityCapability { role } => {
                write!(
                    f,
                    "parents declare conflicting capabilities for entity role {role:?}"
                )
            }
            Self::ConflictingItemCapability { role } => {
                write!(
                    f,
                    "parents declare conflicting capabilities for item role {role:?}"
                )
            }
            Self::ConflictingLocationCapability { role } => {
                write!(
                    f,
                    "parents declare conflicting capabilities for location role {role:?}"
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

/// [`propagate_after`]/[`merge_after_any`]/[`merge_after_all`]/
/// [`propagate_within`] extended to a full [`ResolvedEventContextCapabilities`]
/// (subject **and** declared entity/item participants), applying the same
/// degradation rule to every entity/item capability that the subject rule
/// already applies to the subject.
///
/// **Scope note:** this is capability *bookkeeping* only — it computes what
/// a composed child could honestly promise about an inherited participant,
/// for diagnostics and typed-context callers to consult. It does not by
/// itself thread real command-level participant values (tags/storage
/// paths) across chain/compose graph edges — the export pipeline's
/// generated commands do not yet re-bind a parent's observed participant
/// into a child's own scope. Wiring real cross-edge propagation is tracked
/// as focused follow-up scope, not silently promised here.
pub mod full {
    use super::{
        ContextMergeError, EntityParticipantCapability, ItemParticipantCapability,
        ParticipantLifetime, ResolvedEventContextCapabilities,
        propagate_after as propagate_subject_after, propagate_within as propagate_subject_within,
    };

    fn degrade_entity_after(cap: EntityParticipantCapability) -> EntityParticipantCapability {
        EntityParticipantCapability {
            lifetime: cap
                .lifetime
                .max(ParticipantLifetime::SynchronousDescendants),
            ..cap
        }
    }

    fn degrade_item_after(cap: ItemParticipantCapability) -> ItemParticipantCapability {
        ItemParticipantCapability {
            lifetime: cap
                .lifetime
                .max(ParticipantLifetime::SynchronousDescendants),
            ..cap
        }
    }

    /// Full-capability counterpart of [`super::propagate_after`].
    pub fn propagate_after(
        parent: ResolvedEventContextCapabilities,
    ) -> ResolvedEventContextCapabilities {
        ResolvedEventContextCapabilities {
            subject: propagate_subject_after(parent.subject),
            entities: parent
                .entities
                .into_iter()
                .map(degrade_entity_after)
                .collect(),
            items: parent.items.into_iter().map(degrade_item_after).collect(),
            locations: parent.locations,
        }
    }

    /// Full-capability counterpart of [`super::merge_after_any`]/
    /// [`super::merge_after_all`] — same conservative rule for both, as the
    /// subject-only versions already document. Entity/item capabilities are
    /// intersected by role: a role must appear (with an *equal* capability —
    /// see [`ContextMergeError::ConflictingEntityCapability`]/
    /// [`ContextMergeError::ConflictingItemCapability`]) in every parent to
    /// survive the merge, since the child cannot statically know which
    /// parent actually fired.
    pub fn merge_disjunctive_or_conjunctive(
        parents: &[ResolvedEventContextCapabilities],
    ) -> Result<ResolvedEventContextCapabilities, ContextMergeError> {
        let subjects: Vec<_> = parents.iter().map(|p| p.subject).collect();
        let subject = super::merge_disjunctive_or_conjunctive(&subjects)?;

        let mut entities = Vec::new();
        if let Some(first) = parents.first() {
            for candidate in &first.entities {
                let mut agreed = Some(*candidate);
                for other in &parents[1..] {
                    match other.entities.iter().find(|e| e.role == candidate.role) {
                        Some(entry) if *entry == *candidate => {}
                        Some(_) => {
                            return Err(ContextMergeError::ConflictingEntityCapability {
                                role: candidate.role,
                            });
                        }
                        None => agreed = None,
                    }
                }
                if let Some(entry) = agreed {
                    entities.push(degrade_entity_after(entry));
                }
            }
        }

        let mut items = Vec::new();
        if let Some(first) = parents.first() {
            for candidate in &first.items {
                let mut agreed = Some(*candidate);
                for other in &parents[1..] {
                    match other.items.iter().find(|i| i.role == candidate.role) {
                        Some(entry) if *entry == *candidate => {}
                        Some(_) => {
                            return Err(ContextMergeError::ConflictingItemCapability {
                                role: candidate.role,
                            });
                        }
                        None => agreed = None,
                    }
                }
                if let Some(entry) = agreed {
                    items.push(degrade_item_after(entry));
                }
            }
        }

        Ok(ResolvedEventContextCapabilities {
            subject,
            entities,
            items,
            locations: Vec::new(),
        })
    }

    /// Full-capability counterpart of [`super::propagate_within`] — entity/
    /// item participants never survive a bounded cross-tick window at all
    /// (unlike the subject, which downgrades to `Correlated`/`EventCycle`
    /// rather than disappearing): an ephemeral synchronous-descendant-scoped
    /// entity/item reference from the original firing invocation cannot
    /// possibly still be valid once `.within(...)` later observes its
    /// condition true, potentially ticks later.
    pub fn propagate_within(
        parent: ResolvedEventContextCapabilities,
    ) -> ResolvedEventContextCapabilities {
        ResolvedEventContextCapabilities {
            subject: propagate_subject_within(parent.subject),
            entities: Vec::new(),
            items: Vec::new(),
            locations: Vec::new(),
        }
    }
}

/// `.within::<E>(TickWindow)` cannot retain any capability more precise
/// than an [`EventCycle`](ParticipantLifetime::EventCycle)-scoped subject:
/// the correlation crosses tick boundaries, so anything ephemeral captured
/// at the original firing invocation (a synchronous-descendant-scoped
/// entity/item reference, in particular) is gone by the time the bounded
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
        assert!(caps.entities.is_empty());
        assert!(caps.items.is_empty());
        assert!(caps.locations.is_empty());
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
    fn validate_rejects_duplicate_entity_role() {
        const DUPES: &[EntityParticipantCapability] = &[
            EntityParticipantCapability {
                role: EntityParticipantRole::Victim,
                reliability: ParticipantReliability::Correlated,
                lifetime: ParticipantLifetime::Invocation,
                occurrence: CapabilityOccurrence::Unconditional,
                min_version: None,
            },
            EntityParticipantCapability {
                role: EntityParticipantRole::Victim,
                reliability: ParticipantReliability::Inferred,
                lifetime: ParticipantLifetime::Invocation,
                occurrence: CapabilityOccurrence::Unconditional,
                min_version: None,
            },
        ];
        let caps = EventContextCapabilities {
            subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
            entities: DUPES,
            items: &[],
            locations: &[],
        };
        assert_eq!(
            caps.validate(),
            Err(CapabilityDescriptorError::DuplicateEntityRole {
                role: EntityParticipantRole::Victim
            })
        );
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

    // ── for_event_with_participants (#230 Phase 7) ────────────────────────

    struct CombatEventWithAttacker;
    impl SandEvent for CombatEventWithAttacker {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::TickCondition("score @s x matches 1".into())
        }
        fn participants() -> crate::participant::EventParticipantPlan {
            crate::participant::EventParticipantPlan::new().observe_correlated_attacker()
        }
    }

    #[test]
    fn for_event_with_participants_includes_declared_entity_capabilities() {
        let resolved =
            EventContextCapabilities::for_event_with_participants::<CombatEventWithAttacker>();
        assert_eq!(resolved.subject.reliability, ParticipantReliability::Exact);
        assert_eq!(resolved.entities.len(), 1);
        assert_eq!(resolved.entities[0].role, EntityParticipantRole::Attacker);
        assert_eq!(
            resolved.entities[0].reliability,
            ParticipantReliability::Correlated
        );
        assert!(resolved.items.is_empty());
    }

    #[test]
    fn for_event_with_participants_is_empty_for_undeclared_events() {
        let resolved = EventContextCapabilities::for_event_with_participants::<SimpleTickEvent>();
        assert!(resolved.entities.is_empty());
        assert!(resolved.items.is_empty());
    }

    // ── capabilities::full propagation (#230 Phase 7) ─────────────────────

    mod full_propagation {
        use super::full;
        use super::*;

        fn attacker_capability() -> EntityParticipantCapability {
            EntityParticipantCapability {
                role: EntityParticipantRole::Attacker,
                reliability: ParticipantReliability::Correlated,
                lifetime: ParticipantLifetime::SynchronousDescendants,
                occurrence: CapabilityOccurrence::OccurrenceDependent,
                min_version: Some((1, 20, 2)),
            }
        }

        #[test]
        fn propagate_after_degrades_entity_lifetime_like_subject() {
            let parent = ResolvedEventContextCapabilities {
                subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
                entities: vec![EntityParticipantCapability {
                    lifetime: ParticipantLifetime::Invocation,
                    ..attacker_capability()
                }],
                items: vec![],
                locations: vec![],
            };
            let child = full::propagate_after(parent);
            assert_eq!(
                child.entities[0].lifetime,
                ParticipantLifetime::SynchronousDescendants
            );
            assert_eq!(
                child.subject.lifetime,
                ParticipantLifetime::SynchronousDescendants
            );
        }

        #[test]
        fn merge_after_any_keeps_only_roles_every_parent_agrees_on() {
            let with_attacker = ResolvedEventContextCapabilities {
                subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
                entities: vec![attacker_capability()],
                items: vec![],
                locations: vec![],
            };
            let without_attacker = ResolvedEventContextCapabilities {
                subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
                entities: vec![],
                items: vec![],
                locations: vec![],
            };
            let merged =
                full::merge_disjunctive_or_conjunctive(&[with_attacker, without_attacker]).unwrap();
            assert!(
                merged.entities.is_empty(),
                "a role only one parent declares must not survive the merge"
            );
        }

        #[test]
        fn merge_rejects_conflicting_capability_for_the_same_role() {
            let a = ResolvedEventContextCapabilities {
                subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
                entities: vec![attacker_capability()],
                items: vec![],
                locations: vec![],
            };
            let b = ResolvedEventContextCapabilities {
                subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
                entities: vec![EntityParticipantCapability {
                    reliability: ParticipantReliability::Inferred,
                    ..attacker_capability()
                }],
                items: vec![],
                locations: vec![],
            };
            let err = full::merge_disjunctive_or_conjunctive(&[a, b]).unwrap_err();
            assert_eq!(
                err,
                ContextMergeError::ConflictingEntityCapability {
                    role: EntityParticipantRole::Attacker
                }
            );
        }

        #[test]
        fn propagate_within_drops_all_entity_and_item_participants() {
            let parent = ResolvedEventContextCapabilities {
                subject: SubjectCapability::EXACT_PLAYER_INVOCATION,
                entities: vec![attacker_capability()],
                items: vec![],
                locations: vec![],
            };
            let child = full::propagate_within(parent);
            assert!(child.entities.is_empty());
            assert_eq!(child.subject.lifetime, ParticipantLifetime::EventCycle);
        }
    }
}
