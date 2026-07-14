//! Execution-scoped entity context and relationship-preserving scoped bindings.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use sand_commands::Selector;
use sand_commands::selector::{Many, One};

use crate::entity::kind::{EntityKind, PlayerKind};
use crate::entity::relation::{Relation, RelationQuery};

/// The current executor (`@s`) at a known point in a generated command chain,
/// typed by entity kind.
///
/// `EntityContext` is **execution-scoped**: it is a handle for building
/// commands that refer to whichever entity is bound to `@s` at the point the
/// context is used, not a persistent reference to a specific entity. Once the
/// generated command chain that produced a context has finished running,
/// the context itself has no further meaning — it cannot be stored and
/// replayed against a different entity later. To keep a working reference to
/// a specific entity across a relationship traversal (which changes `@s`),
/// use [`EntityScope::bind`].
#[derive(Debug, Clone, Copy)]
pub struct EntityContext<K> {
    _kind: PhantomData<K>,
}

/// Execution-scoped context for the current player (`@s`, known to be a player).
pub type PlayerContext = EntityContext<PlayerKind>;

impl<K: EntityKind> Default for EntityContext<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: EntityKind> EntityContext<K> {
    pub(crate) fn new() -> Self {
        Self { _kind: PhantomData }
    }

    /// `tag @s add <tag>`.
    pub fn add_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_add(Selector::self_(), tag)
    }

    /// `tag @s remove <tag>`.
    pub fn remove_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_remove(Selector::self_(), tag)
    }

    /// The entity that owns this entity (e.g. a tamed wolf's owner).
    pub fn owner(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Owner)
    }

    /// The entity leashing this entity.
    pub fn leasher(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Leasher)
    }

    /// This entity's current attack/follow target.
    pub fn target(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Target)
    }

    /// The vehicle this entity is riding.
    pub fn vehicle(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Vehicle)
    }

    /// The entity steering this entity's vehicle.
    pub fn controller(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Controller)
    }

    /// The entity that last damaged this entity.
    pub fn attacker(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Attacker)
    }

    /// The entity that fired/summoned this entity (e.g. a projectile's shooter).
    pub fn origin(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Origin)
    }

    /// The entities riding this entity.
    pub fn passengers(&self) -> RelationQuery<Many> {
        RelationQuery::new(Relation::Passengers)
    }
}

// ── Scoped bindings ────────────────────────────────────────────────────────────

static SCOPE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A stable reference to a specific entity, preserved across relationship
/// traversal (which reassigns `@s`).
///
/// Backed by a uniquely namespaced temporary tag added to the bound entity
/// for the lifetime of the [`EntityScope::bind`] call and removed again at
/// the end of the generated command list. The tag name is allocated from a
/// process-global counter, so distinct `bind` call sites never collide; the
/// add/remove pair is emitted as an unconditional straight-line prefix/suffix
/// around the caller's body (Sand's command DSL has no early-return
/// branching), so cleanup always executes exactly once, synchronously,
/// before control returns to whatever iterated to this entity.
///
/// This is honest about scope: a `ScopedEntityRef` is only valid for the
/// duration of the single generated command chain it was created in. It is
/// not a persistent, storable, cross-tick entity reference.
pub struct ScopedEntityRef<K> {
    tag: String,
    _kind: PhantomData<K>,
}

impl<K: EntityKind> ScopedEntityRef<K> {
    fn selector(&self) -> Selector {
        Selector::all_entities().tag(&self.tag).limit(1)
    }

    /// `tag @e[tag=<scope>,limit=1] add <tag>` — tag the bound entity, not `@s`.
    pub fn add_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_add(self.selector(), tag)
    }

    /// `tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity.
    pub fn remove_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_remove(self.selector(), tag)
    }

    /// The bound entity's owner relationship, evaluated relative to `@s`
    /// (valid because the current executor is still the bound entity at the
    /// point relation methods are called from within the `bind` body).
    pub fn owner(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Owner)
    }

    /// The bound entity's leasher relationship.
    pub fn leasher(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Leasher)
    }

    /// The bound entity's target relationship.
    pub fn target(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Target)
    }

    /// The bound entity's vehicle relationship.
    pub fn vehicle(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Vehicle)
    }

    /// The bound entity's controller relationship.
    pub fn controller(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Controller)
    }

    /// The bound entity's attacker relationship.
    pub fn attacker(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Attacker)
    }

    /// The bound entity's origin relationship.
    pub fn origin(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Origin)
    }

    /// The bound entity's passengers.
    pub fn passengers(&self) -> RelationQuery<Many> {
        RelationQuery::new(Relation::Passengers)
    }
}

/// Entry point for scoped, relationship-traversal-safe entity bindings.
pub struct EntityScope;

impl EntityScope {
    /// Tag the entity currently bound to `@s` with a unique, collision-safe
    /// temporary tag, run `body` with a [`ScopedEntityRef`] that can reach
    /// that entity again by tag (even after `@s` has changed via relation
    /// traversal inside `body`), then remove the tag.
    ///
    /// # Example
    /// ```
    /// use sand_core::entity::{EntityContext, EntityScope, kind::AnyEntity};
    /// use sand_core::version::{MinecraftVersion, VersionProfile};
    ///
    /// let profile = VersionProfile::resolve(&MinecraftVersion::parse("latest").unwrap()).unwrap();
    /// let ctx: EntityContext<AnyEntity> = EntityContext::default();
    /// let cmds = EntityScope::bind(&ctx, |arrow_ref| {
    ///     arrow_ref
    ///         .owner()
    ///         .if_player(&profile, |owner| vec![owner.add_tag("shot_by_owner")])
    ///         .unwrap()
    /// });
    /// assert!(cmds[0].starts_with("tag @s add __sand_scope_"));
    /// assert!(cmds.last().unwrap().starts_with("tag @e[tag=__sand_scope_"));
    /// ```
    pub fn bind<K: EntityKind>(
        _ctx: &EntityContext<K>,
        body: impl FnOnce(&ScopedEntityRef<K>) -> Vec<String>,
    ) -> Vec<String> {
        let n = SCOPE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tag = format!("__sand_scope_{n}");
        let scoped = ScopedEntityRef {
            tag: tag.clone(),
            _kind: PhantomData,
        };

        let body_cmds = body(&scoped);
        if body_cmds.is_empty() {
            return Vec::new();
        }

        let mut cmds = Vec::with_capacity(body_cmds.len() + 2);
        cmds.push(sand_commands::builtins::tag_add(
            Selector::self_(),
            tag.clone(),
        ));
        cmds.extend(body_cmds);
        cmds.push(sand_commands::builtins::tag_remove(
            Selector::all_entities().tag(&tag),
            tag,
        ));
        cmds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::kind::AnyEntity;

    #[test]
    fn add_and_remove_tag_use_self() {
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        assert_eq!(ctx.add_tag("observed"), "tag @s add observed");
        assert_eq!(ctx.remove_tag("observed"), "tag @s remove observed");
    }

    #[test]
    fn scoped_ref_targets_by_tag_not_self() {
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        let cmds = EntityScope::bind(&ctx, |scoped| vec![scoped.add_tag("special")]);
        assert_eq!(cmds.len(), 3);
        assert!(cmds[0].starts_with("tag @s add __sand_scope_"));
        let scope_tag = cmds[0].strip_prefix("tag @s add ").unwrap();
        assert_eq!(
            cmds[1],
            format!("tag @e[tag={scope_tag},limit=1] add special")
        );
        assert_eq!(
            cmds[2],
            format!("tag @e[tag={scope_tag}] remove {scope_tag}")
        );
    }

    #[test]
    fn empty_scope_body_emits_no_commands() {
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        let cmds = EntityScope::bind(&ctx, |_scoped| Vec::new());
        assert!(cmds.is_empty());
    }

    #[test]
    fn distinct_bind_call_sites_get_distinct_tags() {
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        let a = EntityScope::bind(&ctx, |scoped| vec![scoped.add_tag("a")]);
        let b = EntityScope::bind(&ctx, |scoped| vec![scoped.add_tag("a")]);
        assert_ne!(a[0], b[0]);
    }
}
