//! Typed vanilla entity relationship traversal (`execute on <relation>`).

use std::marker::PhantomData;

use sand_commands::selector::{Many, One};

use crate::entity::context::EntityContext;
use crate::entity::kind::{AnyEntity, EntityKind, PlayerKind};
use crate::error::{Result, SandError};
use crate::function::register_dyn_fn_dedup;
use crate::version::VersionProfile;

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

    /// Returns `Err` with an actionable diagnostic if `profile` predates this
    /// relation's introduction in vanilla `execute on`.
    ///
    /// `owner`, `leasher`, `target`, `vehicle`, and `passengers` were added
    /// alongside the `execute on` command itself (Minecraft 1.16) and are
    /// available on every profile Sand supports today, so they are never
    /// gated. `attacker`, `controller`, and `origin` were added in later
    /// releases; the thresholds below should be re-verified against the
    /// vanilla changelog before relying on them for a profile close to the
    /// boundary.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::Relation::check_supported",
        aliases = ["sand::prelude::Relation::check_supported"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns `Err` with an actionable diagnostic if `profile` predates this relation's introduction in vanilla `execute on`.",
        context = "Returns `Err` with an actionable diagnostic if `profile` predates this relation's introduction in vanilla `execute on`. `owner`, `leasher`, `target`, `vehicle`, and `passengers` were added alongside the `execute on` command itself (Minecraft 1.16) and are available on every profile Sand supports today, so they are never gated. `attacker`, `controller`, and `origin` were added in later releases; the thresholds below should be re-verified against the vanilla changelog before relying on them for a profile close to the boundary.",
        minecraft = "`owner`, `leasher`, `target`, `vehicle`, and `passengers` were added alongside the `execute on` command itself (Minecraft 1.16) and are available on every profile Sand supports today, so they are never gated. `attacker`, `controller`, and `origin` were added in later releases; the thresholds below should be re-verified against the vanilla changelog before relying on them for a profile close to the boundary.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(profile = "Returns `Err` with an actionable diagnostic if `profile` predates this relation's introduction in vanilla `execute on`."),
        returns = "Returns `Err` with an actionable diagnostic if `profile` predates this relation's introduction in vanilla `execute on`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(relation_value: sand::entity::Relation, profile: & sand::version::VersionProfile)  {\n    let check_supported = relation_value.check_supported(profile);\n}",
    )]
    pub fn check_supported(self, profile: &VersionProfile) -> Result<()> {
        let min: Option<(u32, u32, u32)> = match self {
            Relation::Owner
            | Relation::Leasher
            | Relation::Target
            | Relation::Vehicle
            | Relation::Passengers => None,
            Relation::Attacker => Some((1, 20, 2)),
            Relation::Controller | Relation::Origin => Some((1, 21, 2)),
        };
        let Some((major, minor, patch)) = min else {
            return Ok(());
        };
        if profile
            .requested()
            .is_at_least_components(major, minor, patch)
        {
            Ok(())
        } else {
            Err(SandError::VersionGating {
                location: format!("execute on {}", self.keyword()),
                kind: "entity_relation".to_string(),
                requested_version: profile.resolved_name().to_owned(),
                is_fallback: profile.is_fallback(),
                feature_name: format!("entity_relation_{}", self.keyword()),
                fallback_note: format!(
                    " (`execute on {}` requires Minecraft {major}.{minor}.{patch}+ — \
                     select a supported target version or avoid this relation)",
                    self.keyword()
                ),
            })
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
    minecraft = "Sand lowers this capability to `execute on <relation>` and validates relation availability for the selected Minecraft version.",
    use_when = ["Traversing an owner, vehicle, passenger, or other vanilla entity relationship"],
    avoid_when = ["Selecting entities by filters; use `Target` for selector-compatible targeting"],
    example = "use sand::prelude::*;\n\nfn commands(ctx: &EntityContext<AnyEntity>, profile: &VersionProfile) {\n    let _commands = ctx.owner().if_present(profile, |owner| vec![owner.add_tag(\"has_owner\")]).unwrap();\n}",
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
/// use sand_core::version::{MinecraftVersion, VersionProfile};
///
/// let profile = VersionProfile::resolve(&MinecraftVersion::parse("latest").unwrap()).unwrap();
/// let ctx: EntityContext<AnyEntity> = EntityContext::default();
///
/// // `passengers()` is many-cardinality — `if_present` does not exist for it.
/// ctx.passengers().if_present(&profile, |p| vec![p.add_tag("x")]);
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
        profile: &VersionProfile,
        type_filter: Option<&str>,
        body: impl FnOnce(&EntityContext<K>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.relation.check_supported(profile)?;
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
        params(profile = "`profile` provides the profile used when running `body` if the relation resolves to an entity, as a generic [`AnyEntity`] context. No-op (empty command list) if the relation is absent at runtime — vanilla `execute on <relation>` fails silently when there is no such entity.", body = "Run `body` if the relation resolves to an entity, as a generic [`AnyEntity`] context. No-op (empty command list) if the relation is absent at runtime — vanilla `execute on <relation>` fails silently when there is no such entity."),
        returns = "On success, the value produced to run `body` if the relation resolves to an entity, as a generic [`AnyEntity`] context. No-op (empty command list) if the relation is absent at runtime — vanilla `execute on <relation>` fails silently when there is no such entity; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>, profile: &VersionProfile) {\n    let _commands = ctx.owner().if_present(profile, |owner| vec![owner.add_tag(\"found\")]);\n}",
    )]
    pub fn if_present(
        &self,
        profile: &VersionProfile,
        body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.lower(profile, None, body)
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
        params(profile = "`profile` provides the profile used when running `body` only if the relation resolves to a player.", body = "Run `body` only if the relation resolves to a player."),
        returns = "On success, the value produced to run `body` only if the relation resolves to a player; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>, profile: &VersionProfile) {\n    let _commands = ctx.owner().if_player(profile, |player| vec![player.add_tag(\"owner\")]);\n}",
    )]
    pub fn if_player(
        &self,
        profile: &VersionProfile,
        body: impl FnOnce(&EntityContext<PlayerKind>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.lower(profile, Some("minecraft:player"), body)
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
        params(profile = "`profile` provides the profile used when running `body` once for each passenger, as a generic [`AnyEntity`] context.", body = "Run `body` once for each passenger, as a generic [`AnyEntity`] context."),
        returns = "On success, the value produced to run `body` once for each passenger, as a generic [`AnyEntity`] context; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ctx: &EntityContext<AnyEntity>, profile: &VersionProfile) {\n    let _commands = ctx.passengers().each(profile, |passenger| vec![passenger.add_tag(\"aboard\")]);\n}",
    )]
    pub fn each(
        &self,
        profile: &VersionProfile,
        body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>,
    ) -> Result<Vec<String>> {
        self.lower(profile, None, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::MinecraftVersion;

    fn profile(v: &str) -> VersionProfile {
        VersionProfile::resolve(&MinecraftVersion::parse(v).unwrap()).unwrap()
    }

    #[test]
    fn always_available_relations_are_never_gated() {
        let old = profile("1.19.4");
        for r in [
            Relation::Owner,
            Relation::Leasher,
            Relation::Target,
            Relation::Vehicle,
            Relation::Passengers,
        ] {
            assert!(r.check_supported(&old).is_ok(), "{r:?} should be ungated");
        }
    }

    #[test]
    fn attacker_is_gated_before_1_20_2() {
        let old = profile("1.20.1");
        assert!(Relation::Attacker.check_supported(&old).is_err());
        let new = profile("1.20.2");
        assert!(Relation::Attacker.check_supported(&new).is_ok());
    }

    #[test]
    fn controller_and_origin_are_gated_before_1_21_2() {
        let old = profile("1.21.1");
        assert!(Relation::Controller.check_supported(&old).is_err());
        assert!(Relation::Origin.check_supported(&old).is_err());
        let new = profile("1.21.2");
        assert!(Relation::Controller.check_supported(&new).is_ok());
        assert!(Relation::Origin.check_supported(&new).is_ok());
    }

    #[test]
    fn later_1x_and_26_series_satisfy_all_gates() {
        let p = profile("26.1");
        assert!(Relation::Attacker.check_supported(&p).is_ok());
        assert!(Relation::Controller.check_supported(&p).is_ok());
        assert!(Relation::Origin.check_supported(&p).is_ok());
    }

    #[test]
    fn version_gating_error_names_relation_and_minimum() {
        let old = profile("1.19.4");
        let err = Relation::Attacker.check_supported(&old).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("entity_relation_attacker"));
        assert!(message.contains("1.20.2"));
    }

    #[test]
    fn owner_if_present_lowers_to_execute_on_owner() {
        let p = profile("latest");
        let cmds = RelationTraversal::<One>::new(Relation::Owner)
            .if_present(&p, |owner| vec![owner.add_tag("has_owner")])
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
        let p = profile("latest");
        let cmds = RelationTraversal::<One>::new(Relation::Owner)
            .if_player(&p, |owner| vec![owner.add_tag("owner_is_player")])
            .unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute on owner if entity @s[type=minecraft:player] run function __sand_local:sand/entity_relation/owner/"
        ));
    }

    #[test]
    fn gated_relation_returns_err_before_lowering() {
        let old = profile("1.19.4");
        let result = RelationTraversal::<One>::new(Relation::Attacker)
            .if_present(&old, |a| vec![a.add_tag("hit")]);
        assert!(result.is_err());
    }

    #[test]
    fn empty_relation_body_emits_no_commands() {
        let p = profile("latest");
        let cmds = RelationTraversal::<One>::new(Relation::Owner)
            .if_present(&p, |_| Vec::new())
            .unwrap();
        assert!(cmds.is_empty());
    }

    #[test]
    fn passengers_each_lowers_to_execute_on_passengers() {
        let p = profile("latest");
        let cmds = RelationTraversal::<Many>::new(Relation::Passengers)
            .each(&p, |passenger| vec![passenger.add_tag("carried")])
            .unwrap();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with(
            "execute on passengers run function __sand_local:sand/entity_relation/passengers/"
        ));
    }
}
