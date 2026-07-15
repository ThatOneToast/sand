//! Cardinality-aware entity/player queries and their lowering into typed
//! execution-scoped contexts.

use sand_commands::Selector;
use sand_commands::selector::{
    EntityTarget, EntityTargets, Many, One, PlayerTarget, PlayerTargets, SingleEntity,
    SinglePlayer, SortOrder,
};

use crate::entity::context::EntityContext;
use crate::entity::kind::{AnyEntity, PlayerKind};
use crate::function::register_dyn_fn_dedup;

/// A cardinality-aware query over entities, built on top of
/// [`sand_commands::selector::EntityTarget`].
///
/// `A` is [`One`] once the query has been narrowed (e.g. via
/// [`EntityQuery::limit`]/[`EntityQuery::nearest`]) to select at most one
/// entity, or [`Many`] while it may still select any number.
#[derive(Debug, Clone)]
pub struct EntityQuery<A> {
    target: EntityTarget<A>,
}

/// An [`EntityQuery`] narrowed to select exactly one entity.
pub type SingleEntityQuery = EntityQuery<One>;

/// An [`EntityQuery`] that may select any number of entities.
pub type EntityQueries = EntityQuery<Many>;

/// A cardinality-aware query over players, built on top of
/// [`sand_commands::selector::PlayerTarget`].
#[derive(Debug, Clone)]
pub struct PlayerQuery<A> {
    target: PlayerTarget<A>,
}

/// A [`PlayerQuery`] narrowed to select exactly one player.
pub type SinglePlayerQuery = PlayerQuery<One>;

/// A [`PlayerQuery`] that may select any number of players.
pub type PlayerQueries = PlayerQuery<Many>;

// ── EntityQuery<Many> ──────────────────────────────────────────────────────────

impl EntityQuery<Many> {
    /// `@e` — all entities.
    pub fn entities() -> Self {
        Self {
            target: EntityTargets::all(),
        }
    }

    /// `@e[distance=..<radius>]` — all entities within `radius` blocks of the executor.
    pub fn nearby(radius: f64) -> Self {
        Self {
            target: EntityTargets::nearby(radius),
        }
    }

    /// `type=<ty>` — restrict to entities of the given type.
    pub fn entity_type(mut self, ty: impl Into<String>) -> Self {
        self.target = self.target.entity_type(ty);
        self
    }

    /// `type=!<ty>` — exclude entities of the given type.
    pub fn not_entity_type(mut self, ty: impl Into<String>) -> Self {
        self.target = self.target.not_type(ty);
        self
    }

    /// `type=!minecraft:player` — exclude players from the result set.
    pub fn excluding_players(mut self) -> Self {
        self.target = self.target.excluding_players();
        self
    }

    /// `tag=<tag>` — restrict to entities with the given tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.tag(tag);
        self
    }

    /// `tag=!<tag>` — exclude entities with the given tag.
    pub fn without_tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.not_tag(tag);
        self
    }

    /// `distance=..<max>` — restrict to entities within `max` blocks.
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.target = self.target.within_blocks(max);
        self
    }

    /// `distance=<min>..<max>` — restrict to entities between `min` and `max` blocks.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.target = self.target.distance_range(min, max);
        self
    }

    /// `distance=0.1..` — exclude the current executor.
    pub fn excluding_self(mut self) -> Self {
        self.target = self.target.excluding_self();
        self
    }

    /// Sort results (`sort=nearest|furthest|random|arbitrary`). Only affects
    /// order — does not by itself narrow cardinality; pair with
    /// [`EntityQuery::limit`] to guarantee at most one result.
    pub fn sort(self, order: SortOrder) -> Self {
        let selector = self.target.into_selector().sort(order);
        Self {
            target: EntityTargets::try_from(selector)
                .expect("sorting a validated entity target preserves its category"),
        }
    }

    /// `limit=<n>` — narrow to at most `n` entities, and to [`One`] cardinality.
    pub fn limit(self, n: i32) -> sand_commands::CommandResult<EntityQuery<One>> {
        Ok(EntityQuery {
            target: self.target.limit(n)?,
        })
    }

    /// Sort by nearest and narrow to the single nearest entity.
    pub fn nearest(self) -> EntityQuery<One> {
        EntityQuery {
            target: self.target.nearest(),
        }
    }

    /// Lower this query into a generated function invoked once per matching
    /// entity, with `@s` bound to that entity inside `body`.
    ///
    /// Produces `execute as <selector> at @s run function <generated>`. The
    /// generated function is deduplicated by body content (see
    /// [`crate::function::register_dyn_fn_dedup`]), so structurally identical
    /// `each` bodies across call sites share one generated function.
    pub fn each(self, body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>) -> Vec<String> {
        lower_each(self.target.into_selector(), body)
    }
}

// ── EntityQuery<One> ───────────────────────────────────────────────────────────

impl EntityQuery<One> {
    /// Access the underlying single-arity selector.
    pub fn selector(&self) -> &Selector {
        self.target.selector()
    }

    /// Run `body` with `@s` bound to the single matching entity (a no-op if
    /// there is none). See [`EntityQuery::<Many>::each`] for lowering details.
    pub fn each(self, body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>) -> Vec<String> {
        lower_each(self.target.into_selector(), body)
    }

    /// Alias for [`EntityQuery::<One>::each`] that reads naturally at
    /// single-cardinality call sites.
    pub fn get(self, body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>) -> Vec<String> {
        self.each(body)
    }
}

impl From<SingleEntity> for EntityQuery<One> {
    fn from(target: SingleEntity) -> Self {
        Self { target }
    }
}

impl From<EntityTargets> for EntityQuery<Many> {
    fn from(target: EntityTargets) -> Self {
        Self { target }
    }
}

// ── PlayerQuery<Many> ──────────────────────────────────────────────────────────

impl PlayerQuery<Many> {
    /// `@a` — all players.
    pub fn players() -> Self {
        Self {
            target: PlayerTargets::all(),
        }
    }

    /// `tag=<tag>` — restrict to players with the given tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.tag(tag);
        self
    }

    /// `tag=!<tag>` — exclude players with the given tag.
    pub fn without_tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.not_tag(tag);
        self
    }

    /// `distance=..<max>` — restrict to players within `max` blocks.
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.target = self.target.within_blocks(max);
        self
    }

    /// `distance=<min>..<max>` — restrict to players between `min` and `max` blocks.
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.target = self.target.distance_range(min, max);
        self
    }

    /// Sort results. See [`EntityQuery::sort`].
    pub fn sort(self, order: SortOrder) -> Self {
        let selector = self.target.into_selector().sort(order);
        Self {
            target: PlayerTargets::try_from(selector)
                .expect("sorting a validated player target preserves its category"),
        }
    }

    /// `limit=<n>` — narrow to at most `n` players, and to [`One`] cardinality.
    pub fn limit(self, n: i32) -> sand_commands::CommandResult<PlayerQuery<One>> {
        Ok(PlayerQuery {
            target: self.target.limit(n)?,
        })
    }

    /// Sort by nearest and narrow to the single nearest player.
    pub fn nearest(self) -> PlayerQuery<One> {
        PlayerQuery {
            target: self.target.nearest(),
        }
    }

    /// Run `body` with `@s` bound to each matching player in turn.
    pub fn each(self, body: impl FnOnce(&EntityContext<PlayerKind>) -> Vec<String>) -> Vec<String> {
        lower_each(self.target.into_selector(), body)
    }
}

// ── PlayerQuery<One> ───────────────────────────────────────────────────────────

impl PlayerQuery<One> {
    /// Access the underlying single-arity selector.
    pub fn selector(&self) -> &Selector {
        self.target.selector()
    }

    /// Run `body` with `@s` bound to the single matching player (a no-op if
    /// there is none).
    pub fn each(self, body: impl FnOnce(&EntityContext<PlayerKind>) -> Vec<String>) -> Vec<String> {
        lower_each(self.target.into_selector(), body)
    }

    /// Alias for [`PlayerQuery::<One>::each`].
    pub fn get(self, body: impl FnOnce(&EntityContext<PlayerKind>) -> Vec<String>) -> Vec<String> {
        self.each(body)
    }
}

impl From<SinglePlayer> for PlayerQuery<One> {
    fn from(target: SinglePlayer) -> Self {
        Self { target }
    }
}

impl From<PlayerTargets> for PlayerQuery<Many> {
    fn from(target: PlayerTargets) -> Self {
        Self { target }
    }
}

// ── Shared lowering ────────────────────────────────────────────────────────────

fn lower_each<K: crate::entity::kind::EntityKind>(
    selector: Selector,
    body: impl FnOnce(&EntityContext<K>) -> Vec<String>,
) -> Vec<String> {
    let inner = body(&EntityContext::new());
    if inner.is_empty() {
        return Vec::new();
    }
    let path = register_dyn_fn_dedup("sand/entity_query", inner);
    vec![format!(
        "execute as {selector} at @s run function __sand_local:{path}"
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_each_lowers_to_execute_as_at_run_function() {
        let cmds = EntityQuery::entities()
            .entity_type("minecraft:zombie")
            .tag("hostile")
            .within_blocks(15.0)
            .sort(SortOrder::Nearest)
            .limit(1)
            .unwrap()
            .each(|entity| vec![entity.add_tag("observed")]);

        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute as @e[type=minecraft:zombie,tag=hostile,distance=..15,sort=nearest,limit=1] at @s run function __sand_local:sand/entity_query/"
        ));
    }

    #[test]
    fn empty_each_body_emits_no_commands() {
        let cmds = EntityQuery::entities().each(|_| Vec::new());
        assert!(cmds.is_empty());
    }

    #[test]
    fn structurally_identical_bodies_dedup_to_the_same_function() {
        let a = EntityQuery::entities()
            .tag("a")
            .each(|e| vec![e.add_tag("x")]);
        let b = EntityQuery::entities()
            .tag("b")
            .each(|e| vec![e.add_tag("x")]);
        let fn_a = a[0].rsplit("function ").next().unwrap();
        let fn_b = b[0].rsplit("function ").next().unwrap();
        assert_eq!(fn_a, fn_b);
    }

    #[test]
    fn players_each_lowers_with_player_selector() {
        let cmds = PlayerQuery::players()
            .tag("ready")
            .nearest()
            .each(|p| vec![p.add_tag("chosen")]);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute as @a[tag=ready,sort=nearest,limit=1] at @s run function __sand_local:sand/entity_query/"
        ));
    }
}
