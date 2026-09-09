//! Execution-scoped entity context and relationship-preserving scoped bindings.

use std::marker::PhantomData;

use sand_commands::Selector;
use sand_commands::selector::{Many, One};

use crate::entity::capability::{
    EntityDataRoot, EntityEquipmentHandle, EntityIdentity, EntityMounts, EntityTransform,
    LivingEntity,
};
use crate::entity::kind::EntityKind;
use crate::entity::kind::{EquipmentEntityKind, LivingEntityKind, PlayerKind};
use crate::entity::property::EntityTag;
use crate::entity::relation::{Relation, RelationTraversal};
use crate::item::{EntityInventory, ItemLocation};

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

impl<K: EntityKind> Default for EntityContext<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: EntityKind> EntityContext<K> {
    pub(crate) fn new() -> Self {
        Self { _kind: PhantomData }
    }

    /// Identity and lifecycle operations for the current executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::identity",
        aliases = ["sand::prelude::EntityContext::identity"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns identity and lifecycle operations for this entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn identity(&self) -> EntityIdentity<K> {
        EntityIdentity::new(Selector::self_())
    }

    /// Position, teleportation, rotation, and facing operations for the current executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::transform",
        aliases = ["sand::prelude::EntityContext::transform"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns teleportation, rotation, and facing operations for this entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn transform(&self) -> EntityTransform<K> {
        EntityTransform::new(Selector::self_())
    }

    /// Ride and dismount mutations for the current executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::mounts",
        aliases = ["sand::prelude::EntityContext::mounts"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns ride and dismount operations for this entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn mounts(&self) -> EntityMounts<K> {
        EntityMounts::new(Selector::self_())
    }

    /// Read-only typed entity-data access. Writes are present only for safe kinds.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::data",
        aliases = ["sand::prelude::EntityContext::data"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns capability-gated typed entity-data access.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn data(&self) -> EntityDataRoot<K> {
        EntityDataRoot::new(Selector::self_())
    }

    /// Bind a typed entity-state field to the current executor (`@s`).
    ///
    /// The returned accessor emits commands against `@s`; it is not a
    /// storable entity reference and must remain inside the generated
    /// execution chain that supplied this context.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::state",
        aliases = ["sand::prelude::EntityContext::state"],
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

    /// Adds a validated tag to `@s`.
    ///
    /// This fundamental convenience delegates to [`Self::identity`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::add_tag",
        aliases = ["sand::prelude::EntityContext::add_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag @s add <tag>`.",
        context = "`tag @s add <tag>`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag @s add <tag>` form."),
        returns = "The string value produced to emit the documented `tag @s add <tag>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >, tag: &EntityTag)  {\n    let add_tag = entity_context_value.add_tag(tag);\n}",
    )]
    pub fn add_tag(&self, tag: &EntityTag) -> String {
        self.identity().add_tag(tag)
    }

    /// Removes a validated tag from `@s`.
    ///
    /// This fundamental convenience delegates to [`Self::identity`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::remove_tag",
        aliases = ["sand::prelude::EntityContext::remove_tag"],
        module = "sand::entity",
        kind = "method",
        summary = "`tag @s remove <tag>`.",
        context = "`tag @s remove <tag>`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(tag = "`tag` supplies the documented `tag @s remove <tag>` form."),
        returns = "The string value produced to emit the documented `tag @s remove <tag>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >, tag: &EntityTag)  {\n    let remove_tag = entity_context_value.remove_tag(tag);\n}",
    )]
    pub fn remove_tag(&self, tag: &EntityTag) -> String {
        self.identity().remove_tag(tag)
    }

    /// The entity that owns this entity (e.g. a tamed wolf's owner).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::owner",
        aliases = ["sand::prelude::EntityContext::owner"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity that owns this entity (e.g. a tamed wolf's owner).",
        context = "The entity that owns this entity (e.g. a tamed wolf's owner). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use the entity that owns this entity (e.g. a tamed wolf's owner).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let owner = entity_context_value.owner();\n}",
    )]
    pub fn owner(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Owner)
    }

    /// The entity leashing this entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::leasher",
        aliases = ["sand::prelude::EntityContext::leasher"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity leashing this entity.",
        context = "The entity leashing this entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use the entity leashing this entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let leasher = entity_context_value.leasher();\n}",
    )]
    pub fn leasher(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Leasher)
    }

    /// This entity's current attack/follow target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::target",
        aliases = ["sand::prelude::EntityContext::target"],
        module = "sand::entity",
        kind = "method",
        summary = "This entity's current attack/follow target.",
        context = "This entity's current attack/follow target. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use this entity's current attack/follow target.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let target = entity_context_value.target();\n}",
    )]
    pub fn target(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Target)
    }

    /// The vehicle this entity is riding.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::vehicle",
        aliases = ["sand::prelude::EntityContext::vehicle"],
        module = "sand::entity",
        kind = "method",
        summary = "The vehicle this entity is riding.",
        context = "The vehicle this entity is riding. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use the vehicle this entity is riding.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let vehicle = entity_context_value.vehicle();\n}",
    )]
    pub fn vehicle(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Vehicle)
    }

    /// The entity steering this entity's vehicle.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::controller",
        aliases = ["sand::prelude::EntityContext::controller"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity steering this entity's vehicle.",
        context = "The entity steering this entity's vehicle. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use the entity steering this entity's vehicle.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let controller = entity_context_value.controller();\n}",
    )]
    pub fn controller(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Controller)
    }

    /// The entity that last damaged this entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::attacker",
        aliases = ["sand::prelude::EntityContext::attacker"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity that last damaged this entity.",
        context = "The entity that last damaged this entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use the entity that last damaged this entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let attacker = entity_context_value.attacker();\n}",
    )]
    pub fn attacker(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Attacker)
    }

    /// The entity that fired/summoned this entity (e.g. a projectile's shooter).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::origin",
        aliases = ["sand::prelude::EntityContext::origin"],
        module = "sand::entity",
        kind = "method",
        summary = "The entity that fired/summoned this entity (e.g. a projectile's shooter).",
        context = "The entity that fired/summoned this entity (e.g. a projectile's shooter). This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < One >` value produced to use the entity that fired/summoned this entity (e.g. a projectile's shooter).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let origin = entity_context_value.origin();\n}",
    )]
    pub fn origin(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Origin)
    }

    /// The entities riding this entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::passengers",
        aliases = ["sand::prelude::EntityContext::passengers"],
        module = "sand::entity",
        kind = "method",
        summary = "The entities riding this entity.",
        context = "The entities riding this entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `RelationTraversal < Many >` value produced to use the entities riding this entity.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(entity_context_value: &sand::entity::EntityContext < K >)  {\n    let passengers = entity_context_value.passengers();\n}",
    )]
    pub fn passengers(&self) -> RelationTraversal<Many> {
        RelationTraversal::new(Relation::Passengers)
    }
}

impl<K: LivingEntityKind> EntityContext<K> {
    /// Living-only health, damage, effects, and attribute operations.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::living",
        aliases = ["sand::prelude::EntityContext::living"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns living-only health, damage, effects, and attribute operations.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn living(&self) -> LivingEntity<K> {
        LivingEntity::new(Selector::self_())
    }
}

impl<K: EquipmentEntityKind> EntityContext<K> {
    /// Typed equipment locations for the current executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::equipment",
        aliases = ["sand::prelude::EntityContext::equipment"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns typed equipment locations for a statically capable entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn equipment(&self) -> EntityEquipmentHandle<K> {
        EntityEquipmentHandle::new(Selector::self_())
    }
}

impl EntityContext<PlayerKind> {
    /// Typed inventory locations for the current player executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityContext::inventory",
        aliases = ["sand::prelude::EntityContext::inventory"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns Sand's canonical typed inventory-location factory for this player.",
        context = "The factory explicitly retains @s while providing validated hotbar, main-inventory, ender-chest, hand, and armor locations.",
        minecraft = "Locations lower through Sand's existing item and NBT command paths; this does not mutate player entity NBT directly.",
        use_when = ["Reading, matching, or replacing a known player's inventory items"],
        avoid_when = ["Addressing an entity whose inventory layout is not statically known"],
        returns = "A selector-preserving player inventory factory.",
        example = "use sand::prelude::*; let player = EntityContext::<PlayerKind>::default(); let slot = player.inventory().hotbar(0);",
    )]
    pub fn inventory(&self) -> EntityInventory {
        ItemLocation::entity(Selector::self_())
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

    /// Identity and lifecycle operations targeting the bound entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::identity",
        aliases = ["sand::prelude::ScopedEntityRef::identity"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns identity and lifecycle operations for this entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn identity(&self) -> EntityIdentity<K> {
        EntityIdentity::new(self.selector())
    }

    /// Transform operations targeting the bound entity, even after `@s` changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::transform",
        aliases = ["sand::prelude::ScopedEntityRef::transform"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns teleportation, rotation, and facing operations for this entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn transform(&self) -> EntityTransform<K> {
        EntityTransform::new(self.selector())
    }

    /// Ride and dismount mutations targeting the bound entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::mounts",
        aliases = ["sand::prelude::ScopedEntityRef::mounts"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns ride and dismount operations for this entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn mounts(&self) -> EntityMounts<K> {
        EntityMounts::new(self.selector())
    }

    /// Typed data access targeting the bound entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::data",
        aliases = ["sand::prelude::ScopedEntityRef::data"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns capability-gated typed entity-data access.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn data(&self) -> EntityDataRoot<K> {
        EntityDataRoot::new(self.selector())
    }

    /// Adds a validated tag to the bound entity, not `@s`.
    ///
    /// This fundamental convenience delegates to [`Self::identity`].
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
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >, tag: &EntityTag)  {\n    let add_tag = scoped_entity_ref_value.add_tag(tag);\n}",
    )]
    pub fn add_tag(&self, tag: &EntityTag) -> String {
        self.identity().add_tag(tag)
    }

    /// Removes a validated tag from the bound entity.
    ///
    /// This fundamental convenience delegates to [`Self::identity`].
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
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >, tag: &EntityTag)  {\n    let remove_tag = scoped_entity_ref_value.remove_tag(tag);\n}",
    )]
    pub fn remove_tag(&self, tag: &EntityTag) -> String {
        self.identity().remove_tag(tag)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's owner relationship, evaluated relative to `@s` (valid because the current executor is still the bound entity at the point relation methods are called from within the `bind` body).",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let owner = scoped_entity_ref_value.owner();\n}",
    )]
    pub fn owner(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Owner)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's leasher relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let leasher = scoped_entity_ref_value.leasher();\n}",
    )]
    pub fn leasher(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Leasher)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's target relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let target = scoped_entity_ref_value.target();\n}",
    )]
    pub fn target(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Target)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's vehicle relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let vehicle = scoped_entity_ref_value.vehicle();\n}",
    )]
    pub fn vehicle(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Vehicle)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's controller relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let controller = scoped_entity_ref_value.controller();\n}",
    )]
    pub fn controller(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Controller)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's attacker relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let attacker = scoped_entity_ref_value.attacker();\n}",
    )]
    pub fn attacker(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Attacker)
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
        returns = "The `RelationTraversal < One >` value produced to use the bound entity's origin relationship.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let origin = scoped_entity_ref_value.origin();\n}",
    )]
    pub fn origin(&self) -> RelationTraversal<One> {
        RelationTraversal::new(Relation::Origin)
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
        returns = "The `RelationTraversal < Many >` value produced to use the bound entity's passengers.",
        example = "use sand::prelude::*;\n\nfn demonstrate<K : sand::entity::EntityKind + 'static>(scoped_entity_ref_value: &sand::entity::ScopedEntityRef < K >)  {\n    let passengers = scoped_entity_ref_value.passengers();\n}",
    )]
    pub fn passengers(&self) -> RelationTraversal<Many> {
        RelationTraversal::new(Relation::Passengers)
    }
}

impl<K: LivingEntityKind> ScopedEntityRef<K> {
    /// Living-only operations targeting the bound entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::living",
        aliases = ["sand::prelude::ScopedEntityRef::living"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns living-only health, damage, effects, and attribute operations.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn living(&self) -> LivingEntity<K> {
        LivingEntity::new(self.selector())
    }
}

impl<K: EquipmentEntityKind> ScopedEntityRef<K> {
    /// Typed equipment locations targeting the bound entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::equipment",
        aliases = ["sand::prelude::ScopedEntityRef::equipment"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns typed equipment locations for a statically capable entity.",
        context = "Returns a focused capability for the execution-scoped entity represented by this handle; no persistent entity identity or lifecycle resource is created.",
        minecraft = "Operations emitted by the capability retain this handle's @s or scoped tag-backed target.",
        use_when = ["Discovering legal operations on this entity"],
        avoid_when = ["Keeping an entity identity across ticks"],
        example = "use sand::prelude::*;",
        returns = "The requested capability value or canonical Minecraft command.",
    )]
    pub fn equipment(&self) -> EntityEquipmentHandle<K> {
        EntityEquipmentHandle::new(self.selector())
    }
}

impl ScopedEntityRef<PlayerKind> {
    /// Typed inventory locations targeting the bound player after `@s` changes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::ScopedEntityRef::inventory",
        aliases = ["sand::prelude::ScopedEntityRef::inventory"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns Sand's canonical typed inventory-location factory for the bound player.",
        context = "The factory retains the scope's explicit tag selector, so nested relationship execution cannot redirect inventory operations to another @s.",
        minecraft = "Locations lower through Sand's existing item and NBT command paths; this does not mutate player entity NBT directly.",
        use_when = ["Reading, matching, or replacing a scoped player's inventory items"],
        avoid_when = ["Keeping the scope-backed reference beyond EntityScope::bind"],
        returns = "A selector-preserving player inventory factory.",
        example = "use sand::prelude::*;",
    )]
    pub fn inventory(&self) -> EntityInventory {
        ItemLocation::entity(self.selector())
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
    /// use sand_core::entity::{EntityContext, EntityScope, EntityTag, kind::AnyEntity};
    /// use sand_core::version::{MinecraftVersion, VersionProfile};
    ///
    /// let profile = VersionProfile::resolve(&MinecraftVersion::parse("latest").unwrap()).unwrap();
    /// let ctx: EntityContext<AnyEntity> = EntityContext::default();
    /// let cmds = EntityScope::bind(&ctx, |arrow_ref| {
    ///     arrow_ref
    ///         .owner()
    ///         .if_player(|owner| vec![owner.identity().add_tag(&EntityTag::new("shot_by_owner").unwrap())])
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
        example = "use sand::prelude::*;\nlet ctx: EntityContext<AnyEntity> = EntityContext::default();\nlet tag = EntityTag::new(\"shot_by_owner\").unwrap();\nlet cmds = EntityScope::bind(&ctx, |arrow_ref| {\narrow_ref.owner().if_player(|owner| vec![owner.identity().add_tag(&tag)]).unwrap()\n});\nassert!(cmds[0].starts_with(\"tag @s add __sand_scope_\"));",
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

    fn tag(value: &str) -> EntityTag {
        EntityTag::new(value).unwrap()
    }

    #[test]
    fn add_and_remove_tag_use_self() {
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        assert_eq!(ctx.add_tag(&tag("observed")), "tag @s add observed");
        assert_eq!(ctx.remove_tag(&tag("observed")), "tag @s remove observed");
    }

    #[test]
    fn scoped_ref_targets_by_tag_not_self() {
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        let cmds = EntityScope::bind(&ctx, |scoped| vec![scoped.add_tag(&tag("special"))]);
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
        let a = EntityScope::bind(&ctx, |scoped| vec![scoped.add_tag(&tag("a"))]);
        let b = EntityScope::bind(&ctx, |scoped| vec![scoped.add_tag(&tag("a"))]);
        assert_ne!(a[0], b[0]);
    }

    #[test]
    fn same_call_site_is_repeat_export_deterministic() {
        fn build(ctx: &EntityContext<AnyEntity>) -> Vec<String> {
            EntityScope::bind(ctx, |scoped| vec![scoped.add_tag(&tag("a"))])
        }
        let ctx: EntityContext<AnyEntity> = EntityContext::new();
        assert_eq!(build(&ctx), build(&ctx));
    }
}
