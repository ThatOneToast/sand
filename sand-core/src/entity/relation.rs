//! Typed vanilla entity relationship traversal (`execute on <relation>`).

use std::marker::PhantomData;

use sand_commands::selector::{Many, One};

use crate::entity::context::EntityContext;
use crate::entity::kind::{AnyEntity, EntityKind, PlayerKind};
use crate::error::Result;
use crate::function::register_dyn_fn_dedup;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::Relation",
    aliases = ["sand::prelude::Relation"],
    module = "sand::entity",
    summary = "A vanilla entity relationship reachable via `execute on <relation>`.",
    context = "A vanilla entity relationship reachable via `execute on <relation>`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::Relation;",
    variants(Attacker = "The entity that last damaged this entity.", Controller = "The entity steering this entity's vehicle (e.g. a boat's rower).", Leasher = "The entity leashing this entity.", Origin = "The entity that fired/summoned this entity (e.g. a projectile's shooter).", Owner = "The entity that owns this entity (e.g. a tamed wolf's owner).", Passengers = "The entities riding this entity. Many-cardinality.", Target = "This entity's current attack/follow target (mobs only).", Vehicle = "The vehicle this entity is riding."),
)]
/// A vanilla entity relationship reachable via `execute on <relation>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// The entity that owns this entity (e.g. a tamed wolf's owner).
    Owner,
    /// The entity leashing this entity.
    Leasher,
    /// This entity's current attack/follow target (mobs only).
    Target,
    /// The vehicle this entity is riding.
    Vehicle,
    /// The entity steering this entity's vehicle (e.g. a boat's rower).
    Controller,
    /// The entity that last damaged this entity.
    Attacker,
    /// The entity that fired/summoned this entity (e.g. a projectile's shooter).
    Origin,
    /// The entities riding this entity. Many-cardinality.
    Passengers,
}

impl Relation {
    /// The `execute on <keyword>` relation keyword.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Relation::keyword",
        aliases = ["sand::prelude::Relation::keyword"],
        module = "sand::entity",
        kind = "method",
        summary = "The `execute on <keyword>` relation keyword.",
        context = "The `execute on <keyword>` relation keyword. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The string value produced to use the `execute on <keyword>` relation keyword.",
        example = "use sand::prelude::*;\n\nfn demonstrate(relation_value: sand::entity::Relation)  {\n    let keyword = relation_value.keyword();\n}",
    )]
    pub const fn keyword(self) -> &'static str {
        match self {
            Relation::Owner => "owner",
            Relation::Leasher => "leasher",
            Relation::Target => "target",
            Relation::Vehicle => "vehicle",
            Relation::Controller => "controller",
            Relation::Attacker => "attacker",
            Relation::Origin => "origin",
            Relation::Passengers => "passengers",
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RelationTraversal",
    aliases = ["sand::prelude::RelationTraversal"],
    module = "sand::entity",
    summary = "A pending `execute on` relationship traversal from an [`EntityContext`].",
    context = "A pending traversal through a vanilla `execute on <relation>` relationship. This is an execution capability returned by entity contexts, not a selector or a renamed target type; callers normally use it through inference by chaining a relation method such as `owner().if_present(...)`.",
    minecraft = "Sand lowers this capability to the `execute on <relation>` syntax available throughout Minecraft 26.x+.",
    use_when = ["Traversing an owner, vehicle, passenger, or other vanilla entity relationship"],
    avoid_when = ["Selecting entities by filters; use `Target` for selector-compatible targeting"],
    example = "use sand::prelude::*;\n\nfn commands(ctx: &EntityContext<AnyEntity>) {\n    let _commands = ctx.owner().if_present(|owner| vec![owner.add_tag(\"has_owner\")]).unwrap();\n}",
)]
/// A pending traversal of a single [`Relation`] from an [`EntityContext`].
///
/// `A` encodes cardinality: [`One`] for relations that resolve to at most one
/// entity, [`Many`] for [`Relation::Passengers`].
///
/// Cardinality is enforced at the type level: [`RelationTraversal::<One>::if_present`]
/// and [`RelationTraversal::<One>::if_player`] are only defined for single-cardinality
/// relations, and [`RelationTraversal::<Many>::each`] is only defined for
/// many-cardinality ones. Calling the single-relation API on a many-cardinality
/// relation is a compile error, not a runtime one:
///
/// ```compile_fail
/// use sand_core::entity::{EntityContext, kind::AnyEntity};
/// let ctx: EntityContext<AnyEntity> = EntityContext::default();
///
/// // `passengers()` is many-cardinality — `if_present` does not exist for it.
/// ctx.passengers().if_present(|p| vec![p.add_tag("x")]);
/// ```
pub struct RelationTraversal<A> {
    relation: Relation,
    _arity: PhantomData<A>,
}

impl<A> RelationTraversal<A> {
    pub(crate) fn new(relation: Relation) -> Self {
        Self {
            relation,
            _arity: PhantomData,
        }
    }

    /// The underlying relation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RelationTraversal::relation",
        aliases = ["sand::prelude::RelationTraversal::relation"],
        module = "sand::entity",
        kind = "method",
        summary = "The underlying relation.",
        context = "The underlying relation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `Relation` value produced to use the underlying relation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>) {\n    let relation = ctx.owner().relation();\n}",
    )]
    pub fn relation(&self) -> Relation {
        self.relation
    }

    fn lower<K: EntityKind>(
        &self,
        type_filter: Option<&str>,
        body: impl FnOnce(&EntityContext<K>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        let inner = body(&EntityContext::new());
        if inner.is_empty() {
            return Ok(Vec::new());
        }
        let prefix = format!("sand/entity_relation/{}", self.relation.keyword());
        let path = register_dyn_fn_dedup(&prefix, inner);
        let guard = match type_filter {
            Some(ty) => format!(" if entity @s[type={ty}]"),
            None => String::new(),
        };
        Ok(vec![format!(
            "execute on {}{guard} run function __sand_local:{path}",
            self.relation.keyword()
        )])
    }
}

impl RelationTraversal<One> {
    /// Run `body` if the relation resolves to an entity, as a generic
    /// [`AnyEntity`] context. No-op (empty command list) if the relation is
    /// absent at runtime — vanilla `execute on <relation>` fails silently
    /// when there is no such entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RelationTraversal::if_present",
        aliases = ["sand::prelude::RelationTraversal::if_present"],
        module = "sand::entity",
        kind = "method",
        summary = "Run `body` if the relation resolves to an entity, as a generic [`AnyEntity`] context. No-op (empty command list) if the relation is absent at runtime — vanilla `execute on <relation>` fails silently when there is no such entity.",
        context = "Run `body` if the relation resolves to an entity, as a generic [`AnyEntity`] context. No-op (empty command list) if the relation is absent at runtime — vanilla `execute on <relation>` fails silently when there is no such entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "Commands generated in the related entity context."),
        returns = "On success, the value produced to run `body` if the relation resolves to an entity, as a generic [`AnyEntity`] context. No-op (empty command list) if the relation is absent at runtime — vanilla `execute on <relation>` fails silently when there is no such entity; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>) {\n    let _commands = ctx.owner().if_present(|owner| vec![owner.add_tag(\"found\")]);\n}",
    )]
    pub fn if_present(
        &self,
        body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.lower(None, body)
    }

    /// Run `body` only if the relation resolves to a player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RelationTraversal::if_player",
        aliases = ["sand::prelude::RelationTraversal::if_player"],
        module = "sand::entity",
        kind = "method",
        summary = "Run `body` only if the relation resolves to a player.",
        context = "Run `body` only if the relation resolves to a player. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "Commands generated when the related entity is a player."),
        returns = "On success, the value produced to run `body` only if the relation resolves to a player; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>) {\n    let _commands = ctx.owner().if_player(|player| vec![player.add_tag(\"owner\")]);\n}",
    )]
    pub fn if_player(
        &self,
        body: impl FnOnce(&EntityContext<PlayerKind>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.lower(Some("minecraft:player"), body)
    }
}

impl RelationTraversal<Many> {
    /// Run `body` once for each passenger, as a generic [`AnyEntity`] context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::RelationTraversal::each",
        aliases = ["sand::prelude::RelationTraversal::each"],
        module = "sand::entity",
        kind = "method",
        summary = "Run `body` once for each passenger, as a generic [`AnyEntity`] context.",
        context = "Run `body` once for each passenger, as a generic [`AnyEntity`] context. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "Commands generated for each related passenger."),
        returns = "On success, the value produced to run `body` once for each passenger, as a generic [`AnyEntity`] context; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>) {\n    let _commands = ctx.passengers().each(|passenger| vec![passenger.add_tag(\"aboard\")]);\n}",
    )]
    pub fn each(
        &self,
        body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.lower(None, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_if_present_lowers_to_execute_on_owner() {
        let cmds = RelationTraversal::<One>::new(Relation::Owner)
            .if_present(|owner| vec![owner.add_tag("has_owner")])
            .unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].starts_with(
                "execute on owner run function __sand_local:sand/entity_relation/owner/"
            )
        );
    }

    #[test]
    fn owner_if_player_adds_player_type_guard() {
        let cmds = RelationTraversal::<One>::new(Relation::Owner)
            .if_player(|owner| vec![owner.add_tag("owner_is_player")])
            .unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute on owner if entity @s[type=minecraft:player] run function __sand_local:sand/entity_relation/owner/"
        ));
    }

    #[test]
    fn empty_relation_body_emits_no_commands() {
        let cmds = RelationTraversal::<One>::new(Relation::Owner)
            .if_present(|_| Vec::new())
            .unwrap();
        assert!(cmds.is_empty());
    }

    #[test]
    fn passengers_each_lowers_to_execute_on_passengers() {
        let cmds = RelationTraversal::<Many>::new(Relation::Passengers)
            .each(|passenger| vec![passenger.add_tag("carried")])
            .unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute on passengers run function __sand_local:sand/entity_relation/passengers/"
        ));
    }
}
