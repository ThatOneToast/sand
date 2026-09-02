//! Typed participant roles (#230 Phase 8).

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::EntityParticipantRole",
    aliases = ["sand::prelude::EntityParticipantRole"],
    module = "sand::participant",
    summary = "The role an entity/player participant plays in an event, independent of its [`ParticipantReliability`](super::reliability::ParticipantReliability).",
    context = "The role an entity/player participant plays in an event, independent of its [`ParticipantReliability`](super::reliability::ParticipantReliability). This is a stable, deliberately small vocabulary. Every variant has either an existing vanilla mechanism it maps to (documented per-variant) or a clear planned Phase 9 use — roles without credible evidence are not included. `Actor`/`Subject` are distinguished because not every event's primary participant is a player (`Actor` covers the general case; `Subject` is specifically the player-subject role every `Event<T>` context already exposes via `.player()`/`.subject()`).",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::EntityParticipantRole;",
    variants(Actor = "The primary non-player-specific actor of the event, when the event is not inherently player-scoped.", Attacker = "The entity credited with causing damage/an effect, which vanilla's own damage-source model may itself attribute indirectly (e.g. a thrown potion's thrower) — see [`DirectAttacker`](Self::DirectAttacker) for the immediate-cause distinction vanilla's damage source also draws.", DirectAttacker = "The entity that directly caused damage (e.g. the arrow itself, rather than the player who shot it) — distinct from [`Attacker`](Self::Attacker) the same way vanilla's damage source distinguishes a direct entity from a causing entity.", InteractedEntity = "The entity a player directly interacted with, as in `minecraft:player_interacted_with_entity`.", Killer = "The entity that landed a killing blow, as tracked by `minecraft:player_killed_entity`/`minecraft:entity_killed_player` criteria (see `sand-core/src/event/trigger.rs`).", Projectile = "A projectile entity (arrow, thrown item, fireball, ...).", ProjectileOwner = "The entity that fired/threw a [`Projectile`](Self::Projectile).", Subject = "The event's primary player subject — what `Event<T>::player()`/ `.subject()` already expose today, rendered as `@s`.", Target = "A generic targeted entity, for events whose vanilla criterion names its second entity \"target\" rather than attacker/victim.", Victim = "The entity that received damage/an effect."),
)]
/// The role an entity/player participant plays in an event, independent of
/// its [`ParticipantReliability`](super::reliability::ParticipantReliability).
///
/// This is a stable, deliberately small vocabulary. Every variant has
/// either an existing vanilla mechanism it maps to (documented per-variant)
/// or a clear planned Phase 9 use — roles without credible evidence are not
/// included. `Actor`/`Subject` are distinguished because not every event's
/// primary participant is a player (`Actor` covers the general case;
/// `Subject` is specifically the player-subject role every `Event<T>`
/// context already exposes via `.player()`/`.subject()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityParticipantRole {
    /// The event's primary player subject — what `Event<T>::player()`/
    /// `.subject()` already expose today, rendered as `@s`.
    Subject,
    /// The primary non-player-specific actor of the event, when the event
    /// is not inherently player-scoped.
    Actor,
    /// The entity credited with causing damage/an effect, which vanilla's
    /// own damage-source model may itself attribute indirectly (e.g. a
    /// thrown potion's thrower) — see [`DirectAttacker`](Self::DirectAttacker)
    /// for the immediate-cause distinction vanilla's damage source also
    /// draws.
    Attacker,
    /// The entity that directly caused damage (e.g. the arrow itself,
    /// rather than the player who shot it) — distinct from
    /// [`Attacker`](Self::Attacker) the same way vanilla's damage source
    /// distinguishes a direct entity from a causing entity.
    DirectAttacker,
    /// The entity that received damage/an effect.
    Victim,
    /// The entity that landed a killing blow, as tracked by
    /// `minecraft:player_killed_entity`/`minecraft:entity_killed_player`
    /// criteria (see `sand-core/src/event/trigger.rs`).
    Killer,
    /// A generic targeted entity, for events whose vanilla criterion names
    /// its second entity "target" rather than attacker/victim.
    Target,
    /// The entity a player directly interacted with, as in
    /// `minecraft:player_interacted_with_entity`.
    InteractedEntity,
    /// A projectile entity (arrow, thrown item, fireball, ...).
    Projectile,
    /// The entity that fired/threw a [`Projectile`](Self::Projectile).
    ProjectileOwner,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::LocationParticipantRole",
    module = "sand::participant",
    summary = "The role a captured location plays in an event. Deliberately minimal: only the one location role with existing evidence in Sand's advancement-trigger surface (`placed_block`, `item_used_on_block`, both of which already carry a `BlockPos`) is included. Additional roles (e.g. a distinct \"origin\" vs. \"destination\" block) are left for Phase 9 to add once a concrete second use exists.",
    context = "The role a captured location plays in an event. Deliberately minimal: only the one location role with existing evidence in Sand's advancement-trigger surface (`placed_block`, `item_used_on_block`, both of which already carry a `BlockPos`) is included. Additional roles (e.g. a distinct \"origin\" vs. \"destination\" block) are left for Phase 9 to add once a concrete second use exists. Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
    minecraft = "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::LocationParticipantRole;",
    variants(EventBlock = "The block position the event concerns (e.g. a placed or interacted-with block)."),
)]
/// The role a captured location plays in an event.
///
/// Deliberately minimal: only the one location role with existing evidence
/// in Sand's advancement-trigger surface (`placed_block`,
/// `item_used_on_block`, both of which already carry a `BlockPos`) is
/// included. Additional roles (e.g. a distinct "origin" vs. "destination"
/// block) are left for Phase 9 to add once a concrete second use exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LocationParticipantRole {
    /// The block position the event concerns (e.g. a placed or
    /// interacted-with block).
    EventBlock,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::participant::ParticipantHand",
    aliases = ["sand::prelude::ParticipantHand"],
    module = "sand::participant",
    summary = "Which hand a held-item participant is captured from.",
    context = "Which hand a held-item participant is captured from. Unlike inferring \"the item used to trigger this event\" (which vanilla does not always disambiguate by hand — see `EventParticipantPlan::observe_held_item`'s doc), addressing a specific hand slot is always exact and unambiguous: `PlayerMainHand`/ `PlayerOffHand` are directly queryable NBT paths on `@s`, not a correlated guess.",
    minecraft = "Unlike inferring \"the item used to trigger this event\" (which vanilla does not always disambiguate by hand — see `EventParticipantPlan::observe_held_item`'s doc), addressing a specific hand slot is always exact and unambiguous: `PlayerMainHand`/ `PlayerOffHand` are directly queryable NBT paths on `@s`, not a correlated guess.",
    use_when = ["Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan"],
    avoid_when = ["Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime"],
    example = "use sand::participant::ParticipantHand;",
    variants(MainHand = "Selects the main hand participant semantic.", OffHand = "Selects the off hand participant semantic."),
)]
/// Which hand a held-item participant is captured from.
///
/// Unlike inferring "the item used to trigger this event" (which vanilla
/// does not always disambiguate by hand — see
/// `EventParticipantPlan::observe_held_item`'s doc), addressing a specific
/// hand slot is always exact and unambiguous: `PlayerMainHand`/
/// `PlayerOffHand` are directly queryable NBT paths on `@s`, not a
/// correlated guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParticipantHand {
    #[doc = "Selects the main hand participant semantic."]
    MainHand,
    #[doc = "Selects the off hand participant semantic."]
    OffHand,
}

/// The role an item participant plays in an event.
///
/// Phase 7 (#229) already defined [`crate::item::ItemRole`] for exactly
/// this purpose and it cleanly covers the item side (`UsedItem`, `Weapon`,
/// `Tool`, `Ammunition`, `DroppedItem`, plus `ProjectileItem`/`EquippedItem`
/// which this phase's spec list did not separately call out but which are
/// no less credible). Reusing it here avoids two competing item-role
/// enums — see the re-export below.
pub use crate::item::ItemRole as ItemParticipantRole;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_roles_are_totally_ordered_and_distinct() {
        let mut roles = vec![
            EntityParticipantRole::ProjectileOwner,
            EntityParticipantRole::Subject,
            EntityParticipantRole::Attacker,
            EntityParticipantRole::Victim,
        ];
        roles.sort();
        roles.dedup();
        assert_eq!(roles.len(), 4);
    }

    #[test]
    fn item_participant_role_reuses_phase_7_item_role() {
        let role: ItemParticipantRole = crate::item::ItemRole::Weapon;
        assert_eq!(role, crate::item::ItemRole::Weapon);
    }
}
