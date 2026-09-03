//! Cardinality-aware entity/player queries and their lowering into typed
//! execution-scoped contexts.

use sand_commands::Selector;
use sand_commands::selector::{
    EntityTarget, EntityTargets, IntoEntityType, Many, One, PlayerTarget, PlayerTargets,
    SingleEntity, SinglePlayer, SortOrder,
};

use crate::entity::context::EntityContext;
use crate::entity::kind::{AnyEntity, PlayerKind};
use crate::function::register_dyn_fn_dedup;

/// Compiler-facing contract implemented by `#[derive(StateQuery)]`.
#[doc(hidden)]
pub trait StateQuerySpec: 'static {
    type Item;

    fn each(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String>;

    /// Run a query body against the current Minecraft executor.
    ///
    /// Unlike [`StateQuerySpec::each`], this does not create an entity scan.
    /// Generated implementations guard every command with the query's
    /// required and forbidden component-presence predicates. This is the
    /// canonical bridge for event handlers, whose dispatcher has already
    /// selected and bound the event owner as `@s`.
    fn current(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String>;
}

/// Generated zero-sized query parameter used inside `#[system]` bodies.
#[doc(hidden)]
pub struct StateQueryHandle<Q>(std::marker::PhantomData<fn() -> Q>);

impl<Q> StateQueryHandle<Q> {
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<Q: StateQuerySpec> StateQueryHandle<Q> {
    #[doc(hidden)]
    pub fn each(self, body: impl FnOnce(Q::Item) -> Vec<String>) -> Vec<String> {
        Q::each(body)
    }

    /// Run once for the current executor when it satisfies the query.
    #[doc(hidden)]
    pub fn current(self, body: impl FnOnce(Q::Item) -> Vec<String>) -> Vec<String> {
        Q::current(body)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityQuery",
    aliases = ["sand::prelude::EntityQuery"],
    module = "sand::entity",
    summary = "A cardinality-aware query over entities, built on top of [`sand::command::EntityTarget`].",
    context = "A cardinality-aware query over entities, built on top of [`sand::command::EntityTarget`]. `A` is [`One`] once the query has been narrowed (e.g. via [`EntityQuery::limit`]/[`EntityQuery::nearest`]) to select at most one entity, or [`Many`] while it may still select any number.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityQuery;",
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::SingleEntityQuery",
    aliases = ["sand::prelude::SingleEntityQuery"],
    module = "sand::entity",
    summary = "An [`EntityQuery`] narrowed to select exactly one entity.",
    context = "An [`EntityQuery`] narrowed to select exactly one entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::SingleEntityQuery;",
)]
/// An [`EntityQuery`] narrowed to select exactly one entity.
pub type SingleEntityQuery = EntityQuery<One>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityQueries",
    aliases = ["sand::prelude::EntityQueries"],
    module = "sand::entity",
    summary = "An [`EntityQuery`] that may select any number of entities.",
    context = "An [`EntityQuery`] that may select any number of entities. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityQueries;",
)]
/// An [`EntityQuery`] that may select any number of entities.
pub type EntityQueries = EntityQuery<Many>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::PlayerQuery",
    aliases = ["sand::prelude::PlayerQuery"],
    module = "sand::entity",
    summary = "A cardinality-aware query over players, built on top of [`sand::command::PlayerTarget`].",
    context = "A cardinality-aware query over players, built on top of [`sand::command::PlayerTarget`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::PlayerQuery;",
)]
/// A cardinality-aware query over players, built on top of
/// [`sand_commands::selector::PlayerTarget`].
#[derive(Debug, Clone)]
pub struct PlayerQuery<A> {
    target: PlayerTarget<A>,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::SinglePlayerQuery",
    aliases = ["sand::prelude::SinglePlayerQuery"],
    module = "sand::entity",
    summary = "A [`PlayerQuery`] narrowed to select exactly one player.",
    context = "A [`PlayerQuery`] narrowed to select exactly one player. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::SinglePlayerQuery;",
)]
/// A [`PlayerQuery`] narrowed to select exactly one player.
pub type SinglePlayerQuery = PlayerQuery<One>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::PlayerQueries",
    aliases = ["sand::prelude::PlayerQueries"],
    module = "sand::entity",
    summary = "A [`PlayerQuery`] that may select any number of players.",
    context = "A [`PlayerQuery`] that may select any number of players. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::PlayerQueries;",
)]
/// A [`PlayerQuery`] that may select any number of players.
pub type PlayerQueries = PlayerQuery<Many>;

// ── EntityQuery<Many> ──────────────────────────────────────────────────────────

impl EntityQuery<Many> {
    /// `@e` — all entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::entities",
        aliases = ["sand::entity::EntityQueries::entities", "sand::entity::SingleEntityQuery::entities", "sand::prelude::EntityQueries::entities", "sand::prelude::EntityQuery::entities", "sand::prelude::SingleEntityQuery::entities"],
        module = "sand::entity",
        kind = "method",
        summary = "`@e` — all entities.",
        context = "`@e` — all entities. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "An `EntityQuery` that emits the documented `@e` — all entities form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let entity_query = sand::entity::EntityQuery ::< sand::command::Many >::entities();\n}",
    )]
    pub fn entities() -> Self {
        Self {
            target: EntityTargets::all(),
        }
    }

    /// `@e[distance=..<radius>]` — all entities within `radius` blocks of the executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::nearby",
        aliases = ["sand::entity::EntityQueries::nearby", "sand::entity::SingleEntityQuery::nearby", "sand::prelude::EntityQueries::nearby", "sand::prelude::EntityQuery::nearby", "sand::prelude::SingleEntityQuery::nearby"],
        module = "sand::entity",
        kind = "method",
        summary = "`@e[distance=..<radius>]` — all entities within `radius` blocks of the executor.",
        context = "`@e[distance=..<radius>]` — all entities within `radius` blocks of the executor. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(radius = "`@e[distance=..<radius>]` — all entities within `radius` blocks of the executor."),
        returns = "An `EntityQuery` that emits the documented `@e[distance=..<radius>]` — all entities within `radius` blocks of the executor form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(radius: f64)  {\n    let entity_query = sand::entity::EntityQuery ::< sand::command::Many >::nearby(radius);\n}",
    )]
    pub fn nearby(radius: f64) -> Self {
        Self {
            target: EntityTargets::nearby(radius),
        }
    }

    /// `type=<ty>` — restrict to entities of the given type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::entity_type",
        aliases = ["sand::entity::EntityQueries::entity_type", "sand::entity::SingleEntityQuery::entity_type", "sand::prelude::EntityQueries::entity_type", "sand::prelude::EntityQuery::entity_type", "sand::prelude::SingleEntityQuery::entity_type"],
        module = "sand::entity",
        kind = "method",
        summary = "`type=<ty>` — restrict to entities of the given type.",
        context = "`type=<ty>` — restrict to entities of the given type. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ty = "`ty` supplies the documented `type=<ty>` — restrict to entities of the given type form."),
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `type=<ty>` — restrict to entities of the given type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, ty: impl sand::command::IntoEntityType)  {\n    let updated_entity_query = entity_query_value.entity_type(ty);\n}",
    )]
    pub fn entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.target = self.target.entity_type(ty);
        self
    }

    /// `type=!<ty>` — exclude entities of the given type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::not_entity_type",
        aliases = ["sand::entity::EntityQueries::not_entity_type", "sand::entity::SingleEntityQuery::not_entity_type", "sand::prelude::EntityQueries::not_entity_type", "sand::prelude::EntityQuery::not_entity_type", "sand::prelude::SingleEntityQuery::not_entity_type"],
        module = "sand::entity",
        kind = "method",
        summary = "`type=!<ty>` — exclude entities of the given type.",
        context = "`type=!<ty>` — exclude entities of the given type. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(ty = "`ty` supplies the documented `type=!<ty>` — exclude entities of the given type form."),
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `type=!<ty>` — exclude entities of the given type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, ty: impl sand::command::IntoEntityType)  {\n    let updated_entity_query = entity_query_value.not_entity_type(ty);\n}",
    )]
    pub fn not_entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.target = self.target.not_type(ty);
        self
    }

    /// `type=!minecraft:player` — exclude players from the result set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::excluding_players",
        aliases = ["sand::entity::EntityQueries::excluding_players", "sand::entity::SingleEntityQuery::excluding_players", "sand::prelude::EntityQueries::excluding_players", "sand::prelude::EntityQuery::excluding_players", "sand::prelude::SingleEntityQuery::excluding_players"],
        module = "sand::entity",
        kind = "method",
        summary = "`type=!minecraft:player` — exclude players from the result set.",
        context = "`type=!minecraft:player` — exclude players from the result set. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `type=!minecraft:player` — exclude players from the result set form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >)  {\n    let updated_entity_query = entity_query_value.excluding_players();\n}",
    )]
    pub fn excluding_players(mut self) -> Self {
        self.target = self.target.excluding_players();
        self
    }

    /// Restrict the query with a typed entity-state predicate.
    ///
    /// Repeated calls merge into one vanilla `scores={...}` selector map.
    /// Duplicate predicates for the same field return a structured command
    /// error rather than silently choosing one bound.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::state",
        aliases = ["sand::entity::EntityQueries::state", "sand::entity::SingleEntityQuery::state", "sand::prelude::EntityQueries::state", "sand::prelude::EntityQuery::state", "sand::prelude::SingleEntityQuery::state"],
        module = "sand::entity",
        kind = "method",
        summary = "Restrict the query with a typed entity-state predicate.",
        context = "Restrict the query with a typed entity-state predicate. Repeated calls merge into one vanilla `scores={...}` selector map. Duplicate predicates for the same field return a structured command error rather than silently choosing one bound.",
        minecraft = "Repeated calls merge into one vanilla `scores={...}` selector map. Duplicate predicates for the same field return a structured command error rather than silently choosing one bound.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(predicate = "`predicate` provides the predicate that must match used to restrict the query with a typed entity-state predicate."),
        returns = "The `sand :: command :: CommandResult < Self >` value produced to restrict the query with a typed entity-state predicate.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, predicate: sand::entity::StatePredicate)  {\n    let state = entity_query_value.state(predicate);\n}",
    )]
    pub fn state(
        mut self,
        predicate: crate::entity::state::StatePredicate,
    ) -> sand_commands::CommandResult<Self> {
        self.target = self.target.score(
            sand_commands::ObjectiveName::try_dynamic(predicate.objective)?,
            predicate.selector_range,
        )?;
        Ok(self)
    }

    /// `tag=<tag>` — restrict to entities with the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::tag",
        aliases = ["sand::entity::EntityQueries::tag", "sand::entity::SingleEntityQuery::tag", "sand::prelude::EntityQueries::tag", "sand::prelude::EntityQuery::tag", "sand::prelude::SingleEntityQuery::tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag=<tag>` — restrict to entities with the given tag.",
        context = "`tag=<tag>` — restrict to entities with the given tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — restrict to entities with the given tag form."),
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `tag=<tag>` — restrict to entities with the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, tag: impl Into < String >)  {\n    let updated_entity_query = entity_query_value.tag(tag);\n}",
    )]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.tag(tag);
        self
    }

    /// `tag=!<tag>` — exclude entities with the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::without_tag",
        aliases = ["sand::entity::EntityQueries::without_tag", "sand::entity::SingleEntityQuery::without_tag", "sand::prelude::EntityQueries::without_tag", "sand::prelude::EntityQuery::without_tag", "sand::prelude::SingleEntityQuery::without_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag=!<tag>` — exclude entities with the given tag.",
        context = "`tag=!<tag>` — exclude entities with the given tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag=!<tag>` — exclude entities with the given tag form."),
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `tag=!<tag>` — exclude entities with the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, tag: impl Into < String >)  {\n    let updated_entity_query = entity_query_value.without_tag(tag);\n}",
    )]
    pub fn without_tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.not_tag(tag);
        self
    }

    /// `distance=..<max>` — restrict to entities within `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::within_blocks",
        aliases = ["sand::entity::EntityQueries::within_blocks", "sand::entity::SingleEntityQuery::within_blocks", "sand::prelude::EntityQueries::within_blocks", "sand::prelude::EntityQuery::within_blocks", "sand::prelude::SingleEntityQuery::within_blocks"],
        module = "sand::entity",
        kind = "method",
        summary = "`distance=..<max>` — restrict to entities within `max` blocks.",
        context = "`distance=..<max>` — restrict to entities within `max` blocks. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(max = "`distance=..<max>` — restrict to entities within `max` blocks."),
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `distance=..<max>` — restrict to entities within `max` blocks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, max: f64)  {\n    let updated_entity_query = entity_query_value.within_blocks(max);\n}",
    )]
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.target = self.target.within_blocks(max);
        self
    }

    /// `distance=<min>..<max>` — restrict to entities between `min` and `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::distance_range",
        aliases = ["sand::entity::EntityQueries::distance_range", "sand::entity::SingleEntityQuery::distance_range", "sand::prelude::EntityQueries::distance_range", "sand::prelude::EntityQuery::distance_range", "sand::prelude::SingleEntityQuery::distance_range"],
        module = "sand::entity",
        kind = "method",
        summary = "`distance=<min>..<max>` — restrict to entities between `min` and `max` blocks.",
        context = "`distance=<min>..<max>` — restrict to entities between `min` and `max` blocks. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(min = "`distance=<min>..<max>` — restrict to entities between `min` and `max` blocks.", max = "`distance=<min>..<max>` — restrict to entities between `min` and `max` blocks."),
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `distance=<min>..<max>` — restrict to entities between `min` and `max` blocks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, min: f64, max: f64)  {\n    let updated_entity_query = entity_query_value.distance_range(min, max);\n}",
    )]
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.target = self.target.distance_range(min, max);
        self
    }

    /// `distance=0.1..` — exclude the current executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::excluding_self",
        aliases = ["sand::entity::EntityQueries::excluding_self", "sand::entity::SingleEntityQuery::excluding_self", "sand::prelude::EntityQueries::excluding_self", "sand::prelude::EntityQuery::excluding_self", "sand::prelude::SingleEntityQuery::excluding_self"],
        module = "sand::entity",
        kind = "method",
        summary = "`distance=0.1..` — exclude the current executor.",
        context = "`distance=0.1..` — exclude the current executor. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityQuery` value with the documented change applied to emit the documented `distance=0.1..` — exclude the current executor form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >)  {\n    let updated_entity_query = entity_query_value.excluding_self();\n}",
    )]
    pub fn excluding_self(mut self) -> Self {
        self.target = self.target.excluding_self();
        self
    }

    /// Sort results (`sort=nearest|furthest|random|arbitrary`). Only affects
    /// order — does not by itself narrow cardinality; pair with
    /// [`EntityQuery::limit`] to guarantee at most one result.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::sort",
        aliases = ["sand::entity::EntityQueries::sort", "sand::entity::SingleEntityQuery::sort", "sand::prelude::EntityQueries::sort", "sand::prelude::EntityQuery::sort", "sand::prelude::SingleEntityQuery::sort"],
        module = "sand::entity",
        kind = "method",
        summary = "Sort results (`sort=nearest|furthest|random|arbitrary`). Only affects order — does not by itself narrow cardinality; pair with [`EntityQuery::limit`] to guarantee at most one result.",
        context = "Sort results (`sort=nearest|furthest|random|arbitrary`). Only affects order — does not by itself narrow cardinality; pair with [`EntityQuery::limit`] to guarantee at most one result. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(order = "`order` is used to sort results (`sort=nearest|furthest|random|arbitrary`). Only affects order — does not by itself narrow cardinality; pair with [`EntityQuery::limit`] to guarantee at most one result."),
        returns = "The `EntityQuery` value with the documented change applied to sort results (`sort=nearest|furthest|random|arbitrary`). Only affects order — does not by itself narrow cardinality; pair with [`EntityQuery::limit`] to guarantee at most one result.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, order: sand::command::SortOrder)  {\n    let updated_entity_query = entity_query_value.sort(order);\n}",
    )]
    pub fn sort(self, order: SortOrder) -> Self {
        let selector = self.target.into_selector().sort(order);
        Self {
            target: EntityTargets::try_from(selector)
                .expect("sorting a validated entity target preserves its category"),
        }
    }

    /// `limit=<n>` — narrow to at most `n` entities, and to [`One`] cardinality.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::limit",
        aliases = ["sand::entity::EntityQueries::limit", "sand::entity::SingleEntityQuery::limit", "sand::prelude::EntityQueries::limit", "sand::prelude::EntityQuery::limit", "sand::prelude::SingleEntityQuery::limit"],
        module = "sand::entity",
        kind = "method",
        summary = "`limit=<n>` — narrow to at most `n` entities, and to [`One`] cardinality.",
        context = "`limit=<n>` — narrow to at most `n` entities, and to [`One`] cardinality. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(n = "`limit=<n>` — narrow to at most `n` entities, and to [`One`] cardinality."),
        returns = "The `sand :: command :: CommandResult < EntityQuery < One > >` value produced to emit the documented `limit=<n>` — narrow to at most `n` entities, and to [`One`] cardinality form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >, n: i32)  {\n    let limit = entity_query_value.limit(n);\n}",
    )]
    pub fn limit(self, n: i32) -> sand_commands::CommandResult<EntityQuery<One>> {
        Ok(EntityQuery {
            target: self.target.limit(n)?,
        })
    }

    /// Sort by nearest and narrow to the single nearest entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::nearest",
        aliases = ["sand::entity::EntityQueries::nearest", "sand::entity::SingleEntityQuery::nearest", "sand::prelude::EntityQueries::nearest", "sand::prelude::EntityQuery::nearest", "sand::prelude::SingleEntityQuery::nearest"],
        module = "sand::entity",
        kind = "method",
        summary = "Sort by nearest and narrow to the single nearest entity.",
        context = "Sort by nearest and narrow to the single nearest entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `EntityQuery < One >` value produced to sort by nearest and narrow to the single nearest entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::Many >)  {\n    let nearest = entity_query_value.nearest();\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::selector",
        aliases = ["sand::entity::EntityQueries::selector", "sand::entity::SingleEntityQuery::selector", "sand::prelude::EntityQueries::selector", "sand::prelude::EntityQuery::selector", "sand::prelude::SingleEntityQuery::selector"],
        module = "sand::entity",
        kind = "method",
        summary = "Access the underlying single-arity selector.",
        context = "Access the underlying single-arity selector. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& Selector` value produced to acces the underlying single-arity selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: &sand::entity::EntityQuery < sand::command::One >)  {\n    let selector = entity_query_value.selector();\n}",
    )]
    pub fn selector(&self) -> &Selector {
        self.target.selector()
    }

    /// Run `body` with `@s` bound to the single matching entity (a no-op if
    /// there is none). See [`EntityQuery::<Many>::each`] for lowering details.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::each",
        aliases = ["sand::entity::EntityQueries::each", "sand::entity::SingleEntityQuery::each", "sand::prelude::EntityQueries::each", "sand::prelude::EntityQuery::each", "sand::prelude::SingleEntityQuery::each"],
        module = "sand::entity",
        kind = "method",
        summary = "Run `body` with `@s` bound to the single matching entity (a no-op if there is none). See [`EntityQuery::<Many>::each`] for lowering details.",
        context = "Run `body` with `@s` bound to the single matching entity (a no-op if there is none). See [`EntityQuery::<Many>::each`] for lowering details. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "Run `body` with `@s` bound to the single matching entity (a no-op if there is none). See [`EntityQuery::<Many>::each`] for lowering details."),
        returns = "The ordered values produced to run `body` with `@s` bound to the single matching entity (a no-op if there is none). See [`EntityQuery::<Many>::each`] for lowering details.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::One >, body: impl FnOnce (& sand::entity::EntityContext < sand::entity::AnyEntity >) -> Vec < String >)  {\n    let values = entity_query_value.each(body);\n}",
    )]
    pub fn each(self, body: impl FnOnce(&EntityContext<AnyEntity>) -> Vec<String>) -> Vec<String> {
        lower_each(self.target.into_selector(), body)
    }

    /// Alias for [`EntityQuery::<One>::each`] that reads naturally at
    /// single-cardinality call sites.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityQuery::get",
        aliases = ["sand::entity::EntityQueries::get", "sand::entity::SingleEntityQuery::get", "sand::prelude::EntityQueries::get", "sand::prelude::EntityQuery::get", "sand::prelude::SingleEntityQuery::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Alias for [`EntityQuery::<One>::each`] that reads naturally at single-cardinality call sites.",
        context = "Alias for [`EntityQuery::<One>::each`] that reads naturally at single-cardinality call sites. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "`body` sets the player-visible text for alias for [`EntityQuery::<One>::each`] that reads naturally at single-cardinality call sites."),
        returns = "The ordered values produced to use alias for [`EntityQuery::<One>::each`] that reads naturally at single-cardinality call sites.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_query_value: sand::entity::EntityQuery < sand::command::One >, body: impl FnOnce (& sand::entity::EntityContext < sand::entity::AnyEntity >) -> Vec < String >)  {\n    let values = entity_query_value.get(body);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::players",
        aliases = ["sand::entity::PlayerQueries::players", "sand::entity::SinglePlayerQuery::players", "sand::prelude::PlayerQueries::players", "sand::prelude::PlayerQuery::players", "sand::prelude::SinglePlayerQuery::players"],
        module = "sand::entity",
        kind = "method",
        summary = "`@a` — all players.",
        context = "`@a` — all players. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "A `PlayerQuery` that emits the documented `@a` — all players form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_query = sand::entity::PlayerQuery ::< sand::command::Many >::players();\n}",
    )]
    pub fn players() -> Self {
        Self {
            target: PlayerTargets::all(),
        }
    }

    /// Restrict players with a typed State predicate.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::state",
        aliases = ["sand::entity::PlayerQueries::state", "sand::entity::SinglePlayerQuery::state", "sand::prelude::PlayerQueries::state", "sand::prelude::PlayerQuery::state", "sand::prelude::SinglePlayerQuery::state"],
        module = "sand::entity",
        kind = "method",
        summary = "Restrict players with a typed State predicate.",
        context = "Restrict players with a typed State predicate. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(predicate = "`predicate` provides the predicate that must match used to restrict players with a typed State predicate."),
        returns = "The `sand :: command :: CommandResult < Self >` value produced to restrict players with a typed State predicate.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, predicate: sand::entity::StatePredicate)  {\n    let state = player_query_value.state(predicate);\n}",
    )]
    pub fn state(
        mut self,
        predicate: crate::entity::state::StatePredicate,
    ) -> sand_commands::CommandResult<Self> {
        self.target = self.target.score(
            sand_commands::ObjectiveName::try_dynamic(predicate.objective)?,
            predicate.selector_range,
        )?;
        Ok(self)
    }

    /// `tag=<tag>` — restrict to players with the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::tag",
        aliases = ["sand::entity::PlayerQueries::tag", "sand::entity::SinglePlayerQuery::tag", "sand::prelude::PlayerQueries::tag", "sand::prelude::PlayerQuery::tag", "sand::prelude::SinglePlayerQuery::tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag=<tag>` — restrict to players with the given tag.",
        context = "`tag=<tag>` — restrict to players with the given tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — restrict to players with the given tag form."),
        returns = "The `PlayerQuery` value with the documented change applied to emit the documented `tag=<tag>` — restrict to players with the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, tag: impl Into < String >)  {\n    let updated_player_query = player_query_value.tag(tag);\n}",
    )]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.tag(tag);
        self
    }

    /// `tag=!<tag>` — exclude players with the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::without_tag",
        aliases = ["sand::entity::PlayerQueries::without_tag", "sand::entity::SinglePlayerQuery::without_tag", "sand::prelude::PlayerQueries::without_tag", "sand::prelude::PlayerQuery::without_tag", "sand::prelude::SinglePlayerQuery::without_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag=!<tag>` — exclude players with the given tag.",
        context = "`tag=!<tag>` — exclude players with the given tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag=!<tag>` — exclude players with the given tag form."),
        returns = "The `PlayerQuery` value with the documented change applied to emit the documented `tag=!<tag>` — exclude players with the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, tag: impl Into < String >)  {\n    let updated_player_query = player_query_value.without_tag(tag);\n}",
    )]
    pub fn without_tag(mut self, tag: impl Into<String>) -> Self {
        self.target = self.target.not_tag(tag);
        self
    }

    /// `distance=..<max>` — restrict to players within `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::within_blocks",
        aliases = ["sand::entity::PlayerQueries::within_blocks", "sand::entity::SinglePlayerQuery::within_blocks", "sand::prelude::PlayerQueries::within_blocks", "sand::prelude::PlayerQuery::within_blocks", "sand::prelude::SinglePlayerQuery::within_blocks"],
        module = "sand::entity",
        kind = "method",
        summary = "`distance=..<max>` — restrict to players within `max` blocks.",
        context = "`distance=..<max>` — restrict to players within `max` blocks. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(max = "`distance=..<max>` — restrict to players within `max` blocks."),
        returns = "The `PlayerQuery` value with the documented change applied to emit the documented `distance=..<max>` — restrict to players within `max` blocks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, max: f64)  {\n    let updated_player_query = player_query_value.within_blocks(max);\n}",
    )]
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.target = self.target.within_blocks(max);
        self
    }

    /// `distance=<min>..<max>` — restrict to players between `min` and `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::distance_range",
        aliases = ["sand::entity::PlayerQueries::distance_range", "sand::entity::SinglePlayerQuery::distance_range", "sand::prelude::PlayerQueries::distance_range", "sand::prelude::PlayerQuery::distance_range", "sand::prelude::SinglePlayerQuery::distance_range"],
        module = "sand::entity",
        kind = "method",
        summary = "`distance=<min>..<max>` — restrict to players between `min` and `max` blocks.",
        context = "`distance=<min>..<max>` — restrict to players between `min` and `max` blocks. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(min = "`distance=<min>..<max>` — restrict to players between `min` and `max` blocks.", max = "`distance=<min>..<max>` — restrict to players between `min` and `max` blocks."),
        returns = "The `PlayerQuery` value with the documented change applied to emit the documented `distance=<min>..<max>` — restrict to players between `min` and `max` blocks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, min: f64, max: f64)  {\n    let updated_player_query = player_query_value.distance_range(min, max);\n}",
    )]
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.target = self.target.distance_range(min, max);
        self
    }

    /// Sort results. See [`EntityQuery::sort`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::sort",
        aliases = ["sand::entity::PlayerQueries::sort", "sand::entity::SinglePlayerQuery::sort", "sand::prelude::PlayerQueries::sort", "sand::prelude::PlayerQuery::sort", "sand::prelude::SinglePlayerQuery::sort"],
        module = "sand::entity",
        kind = "method",
        summary = "Sort results. See [`EntityQuery::sort`].",
        context = "Sort results. See [`EntityQuery::sort`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(order = "`order` is used to sort results. See [`EntityQuery::sort`]."),
        returns = "The `PlayerQuery` value with the documented change applied to sort results. See [`EntityQuery::sort`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, order: sand::command::SortOrder)  {\n    let updated_player_query = player_query_value.sort(order);\n}",
    )]
    pub fn sort(self, order: SortOrder) -> Self {
        let selector = self.target.into_selector().sort(order);
        Self {
            target: PlayerTargets::try_from(selector)
                .expect("sorting a validated player target preserves its category"),
        }
    }

    /// `limit=<n>` — narrow to at most `n` players, and to [`One`] cardinality.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::limit",
        aliases = ["sand::entity::PlayerQueries::limit", "sand::entity::SinglePlayerQuery::limit", "sand::prelude::PlayerQueries::limit", "sand::prelude::PlayerQuery::limit", "sand::prelude::SinglePlayerQuery::limit"],
        module = "sand::entity",
        kind = "method",
        summary = "`limit=<n>` — narrow to at most `n` players, and to [`One`] cardinality.",
        context = "`limit=<n>` — narrow to at most `n` players, and to [`One`] cardinality. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(n = "`limit=<n>` — narrow to at most `n` players, and to [`One`] cardinality."),
        returns = "The `sand :: command :: CommandResult < PlayerQuery < One > >` value produced to emit the documented `limit=<n>` — narrow to at most `n` players, and to [`One`] cardinality form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >, n: i32)  {\n    let limit = player_query_value.limit(n);\n}",
    )]
    pub fn limit(self, n: i32) -> sand_commands::CommandResult<PlayerQuery<One>> {
        Ok(PlayerQuery {
            target: self.target.limit(n)?,
        })
    }

    /// Sort by nearest and narrow to the single nearest player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::nearest",
        aliases = ["sand::entity::PlayerQueries::nearest", "sand::entity::SinglePlayerQuery::nearest", "sand::prelude::PlayerQueries::nearest", "sand::prelude::PlayerQuery::nearest", "sand::prelude::SinglePlayerQuery::nearest"],
        module = "sand::entity",
        kind = "method",
        summary = "Sort by nearest and narrow to the single nearest player.",
        context = "Sort by nearest and narrow to the single nearest player. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `PlayerQuery < One >` value produced to sort by nearest and narrow to the single nearest player.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::Many >)  {\n    let nearest = player_query_value.nearest();\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::selector",
        aliases = ["sand::entity::PlayerQueries::selector", "sand::entity::SinglePlayerQuery::selector", "sand::prelude::PlayerQueries::selector", "sand::prelude::PlayerQuery::selector", "sand::prelude::SinglePlayerQuery::selector"],
        module = "sand::entity",
        kind = "method",
        summary = "Access the underlying single-arity selector.",
        context = "Access the underlying single-arity selector. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `& Selector` value produced to acces the underlying single-arity selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: &sand::entity::PlayerQuery < sand::command::One >)  {\n    let selector = player_query_value.selector();\n}",
    )]
    pub fn selector(&self) -> &Selector {
        self.target.selector()
    }

    /// Run `body` with `@s` bound to the single matching player (a no-op if
    /// there is none).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::each",
        aliases = ["sand::entity::PlayerQueries::each", "sand::entity::SinglePlayerQuery::each", "sand::prelude::PlayerQueries::each", "sand::prelude::PlayerQuery::each", "sand::prelude::SinglePlayerQuery::each"],
        module = "sand::entity",
        kind = "method",
        summary = "Run `body` with `@s` bound to the single matching player (a no-op if there is none).",
        context = "Run `body` with `@s` bound to the single matching player (a no-op if there is none). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "Run `body` with `@s` bound to the single matching player (a no-op if there is none)."),
        returns = "The ordered values produced to run `body` with `@s` bound to the single matching player (a no-op if there is none).",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::One >, body: impl FnOnce (& sand::entity::EntityContext < sand::entity::PlayerKind >) -> Vec < String >)  {\n    let values = player_query_value.each(body);\n}",
    )]
    pub fn each(self, body: impl FnOnce(&EntityContext<PlayerKind>) -> Vec<String>) -> Vec<String> {
        lower_each(self.target.into_selector(), body)
    }

    /// Alias for [`PlayerQuery::<One>::each`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::PlayerQuery::get",
        aliases = ["sand::entity::PlayerQueries::get", "sand::entity::SinglePlayerQuery::get", "sand::prelude::PlayerQueries::get", "sand::prelude::PlayerQuery::get", "sand::prelude::SinglePlayerQuery::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Alias for [`PlayerQuery::<One>::each`].",
        context = "Alias for [`PlayerQuery::<One>::each`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(body = "`body` sets the player-visible text for alias for [`PlayerQuery::<One>::each`]."),
        returns = "The ordered values produced to use alias for [`PlayerQuery::<One>::each`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_query_value: sand::entity::PlayerQuery < sand::command::One >, body: impl FnOnce (& sand::entity::EntityContext < sand::entity::PlayerKind >) -> Vec < String >)  {\n    let values = player_query_value.get(body);\n}",
    )]
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
    use crate::entity::state::{EntityFlag, EntityScore, EntityStateField};

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

    #[test]
    fn typed_state_predicates_merge_into_one_score_map() {
        let level = EntityScore::<i32>::new("rpg", "mob", "level", 1, None);
        let sick = EntityFlag::new("rpg", "mob", "sick", false);
        let commands = EntityQuery::entities()
            .entity_type("minecraft:zombie")
            .state(level.matches(10..=20).unwrap())
            .unwrap()
            .state(sick.is_enabled())
            .unwrap()
            .each(|entity| vec![entity.add_tag("matched")]);
        assert!(commands[0].contains("scores={"));
        assert!(commands[0].contains(&format!("{}=10..20", level.objective())));
        assert!(commands[0].contains(&format!("{}=1", sick.objective())));
        assert_eq!(commands[0].matches("scores={").count(), 1);
    }
}
