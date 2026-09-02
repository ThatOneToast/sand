//! Entity-kind markers distinguishing players from generic entities in the
//! typed query/context API (issue #227).

use std::fmt;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::AnyEntity",
    aliases = ["sand::prelude::AnyEntity"],
    module = "sand::entity",
    summary = "Marker for an \"any entity\" (`@e`-rooted) query/context kind.",
    context = "Marker for an \"any entity\" (`@e`-rooted) query/context kind. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::AnyEntity;",
)]
/// Marker for an "any entity" (`@e`-rooted) query/context kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnyEntity;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::PlayerKind",
    aliases = ["sand::prelude::PlayerKind"],
    module = "sand::entity",
    summary = "Marker for a player-only (`@a`-rooted) query/context kind.",
    context = "Marker for a player-only (`@a`-rooted) query/context kind. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::PlayerKind;",
)]
/// Marker for a player-only (`@a`-rooted) query/context kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerKind;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::ZombieKind",
    aliases = ["sand::prelude::ZombieKind"],
    module = "sand::entity",
    summary = "Marker for a typed vanilla Zombie.",
    context = "Marker for a typed vanilla Zombie. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::ZombieKind;",
)]
/// Marker for a typed vanilla Zombie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZombieKind;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::MarkerKind",
    aliases = ["sand::prelude::MarkerKind"],
    module = "sand::entity",
    summary = "Marker for a typed vanilla marker entity.",
    context = "Marker for a typed vanilla marker entity. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::MarkerKind;",
)]
/// Marker for a typed vanilla marker entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkerKind;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::EntityKind",
    aliases = ["sand::prelude::EntityKind"],
    module = "sand::entity",
    summary = "A query/context entity kind. Sealed: only [`AnyEntity`] and [`PlayerKind`] implement it today. Capability-specific kinds (living entities, individual mob types) are follow-up work — see issue #228, which builds entity operations and blueprints on top of this foundation.",
    context = "A query/context entity kind. Sealed: only [`AnyEntity`] and [`PlayerKind`] implement it today. Capability-specific kinds (living entities, individual mob types) are follow-up work — see issue #228, which builds entity operations and blueprints on top of this foundation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityKind;",
)]
/// A query/context entity kind.
///
/// Sealed: only [`AnyEntity`] and [`PlayerKind`] implement it today.
/// Capability-specific kinds (living entities, individual mob types) are
/// follow-up work — see issue #228, which builds entity operations and
/// blueprints on top of this foundation.
pub trait EntityKind: sealed::Sealed + fmt::Debug + Clone + Copy + Default + 'static {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::EntityKind::LABEL",
        aliases = ["sand::prelude::EntityKind::LABEL"],
        module = "sand::entity",
        kind = "associated_const",
        summary = "Short label used in generated function paths and diagnostics.",
        context = "Short label used in generated function paths and diagnostics. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        example = "use sand::entity::EntityKind;",
    )]
    /// Short label used in generated function paths and diagnostics.
    const LABEL: &'static str;
}

impl EntityKind for AnyEntity {
    const LABEL: &'static str = "entity";
}

impl EntityKind for PlayerKind {
    const LABEL: &'static str = "player";
}

impl EntityKind for ZombieKind {
    const LABEL: &'static str = "zombie";
}

impl EntityKind for MarkerKind {
    const LABEL: &'static str = "marker";
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::KnownEntityKind",
    aliases = ["sand::prelude::KnownEntityKind"],
    module = "sand::entity",
    summary = "An entity kind with one statically known vanilla/custom entity type.",
    context = "An entity kind with one statically known vanilla/custom entity type. Archetypes require this stronger bound because their adoption selector and summon command must have a concrete typed entity type.",
    minecraft = "Archetypes require this stronger bound because their adoption selector and summon command must have a concrete typed entity type.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::KnownEntityKind;",
)]
/// An entity kind with one statically known vanilla/custom entity type.
///
/// Archetypes require this stronger bound because their adoption selector and
/// summon command must have a concrete typed entity type.
pub trait KnownEntityKind: EntityKind {
    /// Validated entity type used by summon/adoption lowering.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::KnownEntityKind::entity_type",
        aliases = ["sand::prelude::KnownEntityKind::entity_type"],
        module = "sand::entity",
        summary = "Validated entity type used by summon/adoption lowering.",
        context = "Validated entity type used by summon/adoption lowering. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `sand :: registry :: EntityTypeId` value produced to use validated entity type used by summon/adoption lowering.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::entity::KnownEntityKind>()  {\n    let entity_type = <T as sand::entity::KnownEntityKind>::entity_type();\n}",
    )]
    fn entity_type() -> sand_components::EntityTypeId;
}

impl KnownEntityKind for PlayerKind {
    fn entity_type() -> sand_components::EntityTypeId {
        sand_components::EntityTypeId::minecraft("player")
            .expect("the built-in player entity id is valid")
    }
}

impl KnownEntityKind for ZombieKind {
    fn entity_type() -> sand_components::EntityTypeId {
        sand_components::EntityTypeId::minecraft("zombie")
            .expect("the built-in zombie entity id is valid")
    }
}

impl KnownEntityKind for MarkerKind {
    fn entity_type() -> sand_components::EntityTypeId {
        sand_components::EntityTypeId::minecraft("marker")
            .expect("the built-in marker entity id is valid")
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::LivingEntityKind",
    aliases = ["sand::prelude::LivingEntityKind"],
    module = "sand::entity",
    summary = "Capability implemented by living entities, including players.",
    context = "Capability implemented by living entities, including players. This enables typed health observation, effects, damage, and safe player-specific operations. Direct arbitrary entity-NBT writes require the narrower [`MutableLivingEntityKind`] capability.",
    minecraft = "This enables typed health observation, effects, damage, and safe player-specific operations. Direct arbitrary entity-NBT writes require the narrower [`MutableLivingEntityKind`] capability.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::LivingEntityKind;",
)]
/// Capability implemented by living entities, including players.
///
/// This enables typed health observation, effects, damage, and safe
/// player-specific operations. Direct arbitrary entity-NBT writes require the
/// narrower [`MutableLivingEntityKind`] capability.
pub trait LivingEntityKind: KnownEntityKind + sealed::Living {}

impl LivingEntityKind for PlayerKind {}
impl LivingEntityKind for ZombieKind {}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::MutableLivingEntityKind",
    aliases = ["sand::prelude::MutableLivingEntityKind"],
    module = "sand::entity",
    summary = "A non-player living entity whose native data and attributes may be mutated.",
    context = "A non-player living entity whose native data and attributes may be mutated. `PlayerKind` intentionally does not implement this trait, structurally preventing archetype health/attribute/equipment lowering from emitting unsafe player entity-NBT commands.",
    minecraft = "`PlayerKind` intentionally does not implement this trait, structurally preventing archetype health/attribute/equipment lowering from emitting unsafe player entity-NBT commands.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::MutableLivingEntityKind;",
)]
/// A non-player living entity whose native data and attributes may be mutated.
///
/// `PlayerKind` intentionally does not implement this trait, structurally
/// preventing archetype health/attribute/equipment lowering from emitting
/// unsafe player entity-NBT commands.
pub trait MutableLivingEntityKind: LivingEntityKind + sealed::MutableLiving {}

impl MutableLivingEntityKind for ZombieKind {}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::SafeEntityDataWriteKind",
    aliases = ["sand::prelude::SafeEntityDataWriteKind"],
    module = "sand::entity",
    summary = "A non-player entity kind that permits stable typed entity-NBT writes.",
    context = "A non-player entity kind that permits stable typed entity-NBT writes. This capability is intentionally absent from [`PlayerKind`].",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::SafeEntityDataWriteKind;",
)]
/// A non-player entity kind that permits stable typed entity-NBT writes.
///
/// This capability is intentionally absent from [`PlayerKind`].
pub trait SafeEntityDataWriteKind: KnownEntityKind + sealed::SafeDataWrite {}

impl SafeEntityDataWriteKind for ZombieKind {}
impl SafeEntityDataWriteKind for MarkerKind {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::AnyEntity {}
    impl Sealed for super::PlayerKind {}
    impl Sealed for super::ZombieKind {}
    impl Sealed for super::MarkerKind {}

    pub trait Living {}
    impl Living for super::PlayerKind {}
    impl Living for super::ZombieKind {}

    pub trait MutableLiving {}
    impl MutableLiving for super::ZombieKind {}

    pub trait SafeDataWrite {}
    impl SafeDataWrite for super::ZombieKind {}
    impl SafeDataWrite for super::MarkerKind {}
}
