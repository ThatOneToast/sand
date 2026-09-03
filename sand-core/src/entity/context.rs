//! Execution-scoped entity context and relationship-preserving scoped bindings.

use std::marker::PhantomData;

use sand_commands::Selector;
use sand_commands::selector::{Many, One};

use crate::entity::kind::{EntityKind, PlayerKind};
use crate::entity::relation::{Relation, RelationQuery};

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityContext",
    aliases = ["sand::prelude::EntityContext"],
    module = "sand::entity",
    summary = "The current executor (`@s`) at a known point in a generated command chain, typed by entity kind.",
    context = "The current executor (`@s`) at a known point in a generated command chain, typed by entity kind. `EntityContext` is execution-scoped: it is a handle for building commands that refer to whichever entity is bound to `@s` at the point the context is used, not a persistent reference to a specific entity. Once the generated command chain that produced a context has finished running, the context itself has no further meaning — it cannot be stored and replayed against a different entity later. To keep a working reference to a specific entity across a relationship traversal (which changes `@s`), use [`EntityScope::bind`].",
    minecraft = "`EntityContext` is execution-scoped: it is a handle for building commands that refer to whichever entity is bound to `@s` at the point the context is used, not a persistent reference to a specific entity. Once the generated command chain that produced a context has finished running, the context itself has no further meaning — it cannot be stored and replayed against a different entity later. To keep a working reference to a specific entity across a relationship traversal (which changes `@s`), use [`EntityScope::bind`].",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityContext;",
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::PlayerContext",
    aliases = ["sand::prelude::PlayerContext"],
    module = "sand::entity",
    summary = "Execution-scoped context for the current player (`@s`, known to be a player).",
    context = "Execution-scoped context for the current player (`@s`, known to be a player). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::PlayerContext;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::state",
        aliases = ["sand::entity::PlayerContext::state", "sand::prelude::EntityContext::state", "sand::prelude::PlayerContext::state"],
        module = "sand::entity",
        kind = "method",
        summary = "Bind a typed entity-state field to the current executor (`@s`).",
        context = "Bind a typed entity-state field to the current executor (`@s`). The returned accessor emits commands against `@s`; it is not a storable entity reference and must remain inside the generated execution chain that supplied this context.",
        minecraft = "The returned accessor emits commands against `@s`; it is not a storable entity reference and must remain inside the generated execution chain that supplied this context.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` provides the field used when binding a typed entity-state field to the current executor (`@s`)."),
        returns = "The `F :: Accessor` value produced to bind a typed entity-state field to the current executor (`@s`).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static, F : sand::entity::EntityStateField + 'static>(entity_context_value: &sand::entity::EntityContext < K >, field: F)  {\n    let state = entity_context_value.state::<F>(field);\n}",
    )]
    pub fn state<F: crate::entity::state::EntityStateField>(&self, field: F) -> F::Accessor {
        field.bind()
    }

    /// `tag @s add <tag>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::add_tag",
        aliases = ["sand::entity::PlayerContext::add_tag", "sand::prelude::EntityContext::add_tag", "sand::prelude::PlayerContext::add_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag @s add <tag>`.",
        context = "`tag @s add <tag>`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag @s add <tag>` form."),
        returns = "The string value produced to emit the documented `tag @s add <tag>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >, tag: impl Into < String >)  {\n    let add_tag = entity_context_value.add_tag(tag);\n}",
    )]
    pub fn add_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_add(Selector::self_(), tag)
    }

    /// `tag @s remove <tag>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::remove_tag",
        aliases = ["sand::entity::PlayerContext::remove_tag", "sand::prelude::EntityContext::remove_tag", "sand::prelude::PlayerContext::remove_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag @s remove <tag>`.",
        context = "`tag @s remove <tag>`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag @s remove <tag>` form."),
        returns = "The string value produced to emit the documented `tag @s remove <tag>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >, tag: impl Into < String >)  {\n    let remove_tag = entity_context_value.remove_tag(tag);\n}",
    )]
    pub fn remove_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_remove(Selector::self_(), tag)
    }

    /// The entity that owns this entity (e.g. a tamed wolf's owner).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::owner",
        aliases = ["sand::entity::PlayerContext::owner", "sand::prelude::EntityContext::owner", "sand::prelude::PlayerContext::owner"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity that owns this entity (e.g. a tamed wolf's owner).",
        context = "The entity that owns this entity (e.g. a tamed wolf's owner). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the entity that owns this entity (e.g. a tamed wolf's owner).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let owner = entity_context_value.owner();\n}",
    )]
    pub fn owner(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Owner)
    }

    /// The entity leashing this entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::leasher",
        aliases = ["sand::entity::PlayerContext::leasher", "sand::prelude::EntityContext::leasher", "sand::prelude::PlayerContext::leasher"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity leashing this entity.",
        context = "The entity leashing this entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the entity leashing this entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let leasher = entity_context_value.leasher();\n}",
    )]
    pub fn leasher(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Leasher)
    }

    /// This entity's current attack/follow target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::target",
        aliases = ["sand::entity::PlayerContext::target", "sand::prelude::EntityContext::target", "sand::prelude::PlayerContext::target"],
        module = "sand::entity",
        kind = "method",
        summary = "This entity's current attack/follow target.",
        context = "This entity's current attack/follow target. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to thi entity's current attack/follow target.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let target = entity_context_value.target();\n}",
    )]
    pub fn target(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Target)
    }

    /// The vehicle this entity is riding.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::vehicle",
        aliases = ["sand::entity::PlayerContext::vehicle", "sand::prelude::EntityContext::vehicle", "sand::prelude::PlayerContext::vehicle"],
        module = "sand::entity",
        kind = "method",
        summary = "The vehicle this entity is riding.",
        context = "The vehicle this entity is riding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the vehicle this entity is riding.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let vehicle = entity_context_value.vehicle();\n}",
    )]
    pub fn vehicle(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Vehicle)
    }

    /// The entity steering this entity's vehicle.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::controller",
        aliases = ["sand::entity::PlayerContext::controller", "sand::prelude::EntityContext::controller", "sand::prelude::PlayerContext::controller"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity steering this entity's vehicle.",
        context = "The entity steering this entity's vehicle. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the entity steering this entity's vehicle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let controller = entity_context_value.controller();\n}",
    )]
    pub fn controller(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Controller)
    }

    /// The entity that last damaged this entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::attacker",
        aliases = ["sand::entity::PlayerContext::attacker", "sand::prelude::EntityContext::attacker", "sand::prelude::PlayerContext::attacker"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity that last damaged this entity.",
        context = "The entity that last damaged this entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the entity that last damaged this entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let attacker = entity_context_value.attacker();\n}",
    )]
    pub fn attacker(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Attacker)
    }

    /// The entity that fired/summoned this entity (e.g. a projectile's shooter).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::origin",
        aliases = ["sand::entity::PlayerContext::origin", "sand::prelude::EntityContext::origin", "sand::prelude::PlayerContext::origin"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity that fired/summoned this entity (e.g. a projectile's shooter).",
        context = "The entity that fired/summoned this entity (e.g. a projectile's shooter). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the entity that fired/summoned this entity (e.g. a projectile's shooter).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let origin = entity_context_value.origin();\n}",
    )]
    pub fn origin(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Origin)
    }

    /// The entities riding this entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::passengers",
        aliases = ["sand::entity::PlayerContext::passengers", "sand::prelude::EntityContext::passengers", "sand::prelude::PlayerContext::passengers"],
        module = "sand::entity",
        kind = "method",
        summary = "The entities riding this entity.",
        context = "The entities riding this entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < Many >` value produced to use the entities riding this entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let passengers = entity_context_value.passengers();\n}",
    )]
    pub fn passengers(&self) -> RelationQuery<Many> {
        RelationQuery::new(Relation::Passengers)
    }
}

// ── Scoped bindings ────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::ScopedEntityRef",
    aliases = ["sand::prelude::ScopedEntityRef"],
    module = "sand::entity",
    summary = "A stable reference to a specific entity, preserved across relationship traversal (which reassigns `@s`).",
    context = "A stable reference to a specific entity, preserved across relationship traversal (which reassigns `@s`). Backed by a uniquely namespaced temporary tag added to the bound entity for the lifetime of the [`EntityScope::bind`] call and removed again at the end of the generated command list. The tag name is derived from the Rust call site's file, line, and column, so distinct call sites do not collide and repeated/concurrent exports produce identical output; the add/remove pair is emitted as an unconditional straight-line prefix/suffix around the caller's body (Sand's command DSL has no early-return branching), so cleanup always executes exactly once, synchronously, before control returns to whatever iterated to this entity. This is honest about scope: a `ScopedEntityRef` is only valid for the duration of the single generated command chain it was created in. It is not a persistent, storable, cross-tick entity reference.",
    minecraft = "Backed by a uniquely namespaced temporary tag added to the bound entity for the lifetime of the [`EntityScope::bind`] call and removed again at the end of the generated command list. The tag name is derived from the Rust call site's file, line, and column, so distinct call sites do not collide and repeated/concurrent exports produce identical output; the add/remove pair is emitted as an unconditional straight-line prefix/suffix around the caller's body (Sand's command DSL has no early-return branching), so cleanup always executes exactly once, synchronously, before control returns to whatever iterated to this entity.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::ScopedEntityRef;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::add_tag",
        aliases = ["sand::prelude::ScopedEntityRef::add_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag @e[tag=<scope>,limit=1] add <tag>` — tag the bound entity, not `@s`.",
        context = "`tag @e[tag=<scope>,limit=1] add <tag>` — tag the bound entity, not `@s`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag @e[tag=<scope>,limit=1] add <tag>` — tag the bound entity, not `@s` form."),
        returns = "The string value produced to emit the documented `tag @e[tag=<scope>,limit=1] add <tag>` — tag the bound entity, not `@s` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >, tag: impl Into < String >)  {\n    let add_tag = scoped_entity_ref_value.add_tag(tag);\n}",
    )]
    pub fn add_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_add(self.selector(), tag)
    }

    /// `tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::remove_tag",
        aliases = ["sand::prelude::ScopedEntityRef::remove_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity.",
        context = "`tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity form."),
        returns = "The string value produced to emit the documented `tag @e[tag=<scope>,limit=1] remove <tag>` — untag the bound entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >, tag: impl Into < String >)  {\n    let remove_tag = scoped_entity_ref_value.remove_tag(tag);\n}",
    )]
    pub fn remove_tag(&self, tag: impl Into<String>) -> String {
        sand_commands::builtins::tag_remove(self.selector(), tag)
    }

    /// The bound entity's owner relationship, evaluated relative to `@s`
    /// (valid because the current executor is still the bound entity at the
    /// point relation methods are called from within the `bind` body).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::owner",
        aliases = ["sand::prelude::ScopedEntityRef::owner"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's owner relationship, evaluated relative to `@s` (valid because the current executor is still the bound entity at the point relation methods are called from within the `bind` body).",
        context = "The bound entity's owner relationship, evaluated relative to `@s` (valid because the current executor is still the bound entity at the point relation methods are called from within the `bind` body). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's owner relationship, evaluated relative to `@s` (valid because the current executor is still the bound entity at the point relation methods are called from within the `bind` body).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let owner = scoped_entity_ref_value.owner();\n}",
    )]
    pub fn owner(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Owner)
    }

    /// The bound entity's leasher relationship.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::leasher",
        aliases = ["sand::prelude::ScopedEntityRef::leasher"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's leasher relationship.",
        context = "The bound entity's leasher relationship. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's leasher relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let leasher = scoped_entity_ref_value.leasher();\n}",
    )]
    pub fn leasher(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Leasher)
    }

    /// The bound entity's target relationship.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::target",
        aliases = ["sand::prelude::ScopedEntityRef::target"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's target relationship.",
        context = "The bound entity's target relationship. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's target relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let target = scoped_entity_ref_value.target();\n}",
    )]
    pub fn target(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Target)
    }

    /// The bound entity's vehicle relationship.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::vehicle",
        aliases = ["sand::prelude::ScopedEntityRef::vehicle"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's vehicle relationship.",
        context = "The bound entity's vehicle relationship. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's vehicle relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let vehicle = scoped_entity_ref_value.vehicle();\n}",
    )]
    pub fn vehicle(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Vehicle)
    }

    /// The bound entity's controller relationship.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::controller",
        aliases = ["sand::prelude::ScopedEntityRef::controller"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's controller relationship.",
        context = "The bound entity's controller relationship. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's controller relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let controller = scoped_entity_ref_value.controller();\n}",
    )]
    pub fn controller(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Controller)
    }

    /// The bound entity's attacker relationship.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::attacker",
        aliases = ["sand::prelude::ScopedEntityRef::attacker"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's attacker relationship.",
        context = "The bound entity's attacker relationship. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's attacker relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let attacker = scoped_entity_ref_value.attacker();\n}",
    )]
    pub fn attacker(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Attacker)
    }

    /// The bound entity's origin relationship.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::origin",
        aliases = ["sand::prelude::ScopedEntityRef::origin"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's origin relationship.",
        context = "The bound entity's origin relationship. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < One >` value produced to use the bound entity's origin relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let origin = scoped_entity_ref_value.origin();\n}",
    )]
    pub fn origin(&self) -> RelationQuery<One> {
        RelationQuery::new(Relation::Origin)
    }

    /// The bound entity's passengers.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::passengers",
        aliases = ["sand::prelude::ScopedEntityRef::passengers"],
        module = "sand::entity",
        kind = "method",
        summary = "The bound entity's passengers.",
        context = "The bound entity's passengers. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationQuery < Many >` value produced to use the bound entity's passengers.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let passengers = scoped_entity_ref_value.passengers();\n}",
    )]
    pub fn passengers(&self) -> RelationQuery<Many> {
        RelationQuery::new(Relation::Passengers)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityScope",
    aliases = ["sand::prelude::EntityScope"],
    module = "sand::entity",
    summary = "Entry point for scoped, relationship-traversal-safe entity bindings.",
    context = "Entry point for scoped, relationship-traversal-safe entity bindings. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityScope;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityScope::bind",
        aliases = ["sand::prelude::EntityScope::bind"],
        module = "sand::entity",
        kind = "method",
        summary = "Tag the entity currently bound to `@s` with a unique, collision-safe temporary tag, run `body` with a [`ScopedEntityRef`] that can reach that entity again by tag (even after `@s` has changed via relation traversal inside `body`), then remove the tag.",
        context = "Tag the entity currently bound to `@s` with a unique, collision-safe temporary tag, run `body` with a [`ScopedEntityRef`] that can reach that entity again by tag (even after `@s` has changed via relation traversal inside `body`), then remove the tag. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(_ctx = "`ctx` is used to tag the entity currently bound to `@s` with a unique, collision-safe temporary tag, run `body` with a [`ScopedEntityRef`] that can reach that entity again by tag (even after `@s` has changed via relation traversal inside `body`), then remove the tag.", body = "Tag the entity currently bound to `@s` with a unique, collision-safe temporary tag, run `body` with a [`ScopedEntityRef`] that can reach that entity again by tag (even after `@s` has changed via relation traversal inside `body`), then remove the tag."),
        returns = "The ordered values produced to tag the entity currently bound to `@s` with a unique, collision-safe temporary tag, run `body` with a [`ScopedEntityRef`] that can reach that entity again by tag (even after `@s` has changed via relation traversal inside `body`), then remove the tag.",
        example = "use {sand::entity::EntityContext, sand::entity::EntityScope, sand::entity::AnyEntity};\nuse {sand::version::MinecraftVersion, sand::version::VersionProfile};\nlet profile = VersionProfile::resolve(&MinecraftVersion::parse(\"latest\").unwrap()).unwrap();\nlet ctx: EntityContext<AnyEntity> = EntityContext::default();\nlet cmds = EntityScope::bind(&ctx, |arrow_ref| {\narrow_ref\n.owner()\n.if_player(&profile, |owner| vec![owner.add_tag(\"shot_by_owner\")])\n.unwrap()\n});\nassert!(cmds[0].starts_with(\"tag @s add __sand_scope_\"));\nassert!(cmds.last().unwrap().starts_with(\"tag @e[tag=__sand_scope_\"));",
    )]
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
