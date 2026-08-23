//! Execution-scoped entity context and relationship-preserving scoped bindings.

use std::marker::PhantomData;

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

    /// Bind a typed entity-state field to the current executor (`@s`).
    ///
    /// The returned accessor emits commands against `@s`; it is not a
    /// storable entity reference and must remain inside the generated
    /// execution chain that supplied this context.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::state` for the canonical contract."]
    pub fn state<F: crate::entity::state::EntityStateField>(&self, field: F) -> F::Accessor {
        field.bind()
    }

    /// `tag @s add <tag>`.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::add_tag` for the canonical contract."]
    pub fn add_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_add(Selector::self_(), tag)
    }

    /// `tag @s remove <tag>`.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::remove_tag` for the canonical contract."]
    pub fn remove_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_remove(Selector::self_(), tag)
    }

    /// The entity that owns this entity (e.g. a tamed wolf's owner).
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::owner` for the canonical contract."]
    pub fn owner(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Owner)
    }

    /// The entity leashing this entity.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::leasher` for the canonical contract."]
    pub fn leasher(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Leasher)
    }

    /// This entity's current attack/follow target.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::target` for the canonical contract."]
    pub fn target(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Target)
    }

    /// The vehicle this entity is riding.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::vehicle` for the canonical contract."]
    pub fn vehicle(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Vehicle)
    }

    /// The entity steering this entity's vehicle.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::controller` for the canonical contract."]
    pub fn controller(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Controller)
    }

    /// The entity that last damaged this entity.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::attacker` for the canonical contract."]
    pub fn attacker(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Attacker)
    }

    /// The entity that fired/summoned this entity (e.g. a projectile's shooter).
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::origin` for the canonical contract."]
    pub fn origin(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Origin)
    }

    /// The entities riding this entity.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityContext::passengers` for the canonical contract."]
    pub fn passengers(&self) -> RelationQuery<Many> {
        RelationQuery::new(Relation::Passengers)
    }
}

// ── Scoped bindings ────────────────────────────────────────────────────────────

/// A stable reference to a specific entity, preserved across relationship
/// traversal (which reassigns `@s`).
///
/// Backed by a uniquely namespaced temporary tag added to the bound entity
/// for the lifetime of the [`EntityScope::bind`] call and removed again at
/// the end of the generated command list. The tag name is derived from the
/// Rust call site's file, line, and column, so distinct call sites do not
/// collide and repeated/concurrent exports produce identical output; the
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::add_tag` for the canonical contract."]
    pub fn add_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_add(self.selector(), tag)
    }

    /// `tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::remove_tag` for the canonical contract."]
    pub fn remove_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_remove(self.selector(), tag)
    }

    /// The bound entity's owner relationship, evaluated relative to `@s`
    /// (valid because the current executor is still the bound entity at the
    /// point relation methods are called from within the `bind` body).
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::owner` for the canonical contract."]
    pub fn owner(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Owner)
    }

    /// The bound entity's leasher relationship.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::leasher` for the canonical contract."]
    pub fn leasher(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Leasher)
    }

    /// The bound entity's target relationship.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::target` for the canonical contract."]
    pub fn target(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Target)
    }

    /// The bound entity's vehicle relationship.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::vehicle` for the canonical contract."]
    pub fn vehicle(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Vehicle)
    }

    /// The bound entity's controller relationship.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::controller` for the canonical contract."]
    pub fn controller(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Controller)
    }

    /// The bound entity's attacker relationship.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::attacker` for the canonical contract."]
    pub fn attacker(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Attacker)
    }

    /// The bound entity's origin relationship.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::origin` for the canonical contract."]
    pub fn origin(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Origin)
    }

    /// The bound entity's passengers.
    #[doc = "**API Contract:** Run `sand api show sand::entity::ScopedEntityRef::passengers` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityScope::bind` for the canonical contract."]
    #[track_caller]
    pub fn bind<K: EntityKind>(
        _ctx: &EntityContext<K>,
        body: impl FnOnce(&ScopedEntityRef<K>) -> Vec<String>,
    ) -> Vec<String> {
        let location = std::panic::Location::caller();
        let logical = format!(
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        let tag = format!(
            "__sand_scope_{:012x}",
            stable_hash(&logical) & 0xff_ffff_ffff_ffff
        );
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

fn stable_hash(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
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

    #[test]
    fn same_call_site_is_repeat_export_deterministic() {
        fn build(ctx: &EntityContext<AnyEntity>) -> Vec<String> {
            EntityScope::bind(ctx, |scoped| vec![scoped.add_tag("a")])
        }
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        assert_eq!(build(&ctx), build(&ctx));
    }
}
