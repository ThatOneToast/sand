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
    summary = "A sealed marker describing what Sand knows statically about an execution-scoped entity.",
    context = "AnyEntity and PlayerKind represent query roots; concrete known kinds such as ZombieKind and MarkerKind enable only the focused EntityContext capabilities legal for that kind.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::EntityKind;",
)]
/// A query/context entity kind.
///
/// The trait is sealed because each implementation participates in Sand's
/// verified capability model. [`AnyEntity`] deliberately exposes fewer
/// operations than a known kind: an `@e` query may select entities with very
/// different vanilla behavior.
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
    path = "sand::entity::EquipmentEntityKind",
    aliases = ["sand::prelude::EquipmentEntityKind"],
    module = "sand::entity",
    summary = "An entity kind with vanilla equipment slots addressable through Sand's typed item-location API.",
    context = "This sealed capability is implemented for players and known living mobs with supported hand and armor slots. It is absent from MarkerKind and AnyEntity, where equipment legality is not statically known.",
    minecraft = "Equipment access delegates to vanilla item locations: player Inventory entries for PlayerKind and ArmorItems/HandItems for supported non-player living kinds.",
    use_when = ["Writing generic behavior that needs supported equipment slots"],
    avoid_when = ["Assuming an AnyEntity query contains only equipment-capable entities"],
    example = "use sand::entity::EquipmentEntityKind;",
)]
/// An entity kind with equipment slots supported by Sand's canonical item API.
///
/// The trait is sealed because the slot layout is part of Sand's verified
/// vanilla model. [`AnyEntity`] does not implement it: an `@e` query can also
/// contain markers and other entities without the modeled slots.
pub trait EquipmentEntityKind: KnownEntityKind + sealed::Equipment {}

impl EquipmentEntityKind for PlayerKind {}
impl EquipmentEntityKind for ZombieKind {}

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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::MountVehicleKind",
    aliases = ["sand::prelude::MountVehicleKind"],
    module = "sand::entity",
    summary = "An entity kind not statically known to be illegal as a ride vehicle.",
    context = "Known players and marker entities are excluded because vanilla rejects them as vehicles; AnyEntity remains available when the selected runtime type is not statically known.",
    minecraft = "Gates the vehicle side of the ride mount command without inventing a persistent entity reference.",
    use_when = ["Writing generic behavior that mounts a rider onto an entity"],
    avoid_when = ["The kind is a player or marker, which vanilla cannot use as a vehicle"],
    example = "use sand::entity::MountVehicleKind;",
)]
/// An entity kind not statically known to be illegal as a ride vehicle.
///
/// [`AnyEntity`] is included because its runtime type is unknown; callers are
/// responsible for selecting a legal vehicle. [`PlayerKind`] and
/// [`MarkerKind`] are excluded because their illegality is statically known.
pub trait MountVehicleKind: EntityKind + sealed::MountVehicle {}

impl MountVehicleKind for AnyEntity {}
impl MountVehicleKind for ZombieKind {}

pub(crate) mod sealed {
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

    pub trait Equipment {
        fn equipment_location(
            selector: sand_commands::Selector,
            slot: sand_components::EquipmentSlot,
        ) -> Result<crate::item::ItemLocation, crate::item::ItemLocationError>;
    }

    impl Equipment for super::PlayerKind {
        fn equipment_location(
            selector: sand_commands::Selector,
            slot: sand_components::EquipmentSlot,
        ) -> Result<crate::item::ItemLocation, crate::item::ItemLocationError> {
            use sand_components::EquipmentSlot;

            let inventory = crate::item::ItemLocation::entity(selector);
            Ok(match slot {
                EquipmentSlot::Mainhand => inventory.mainhand(),
                EquipmentSlot::Offhand => inventory.offhand(),
                EquipmentSlot::Head => inventory.helmet(),
                EquipmentSlot::Chest => inventory.chestplate(),
                EquipmentSlot::Legs => inventory.leggings(),
                EquipmentSlot::Feet => inventory.boots(),
                EquipmentSlot::Body => {
                    return Err(crate::item::ItemLocationError::UnsupportedLocation {
                        location: "PlayerEquipment(Body)".to_owned(),
                        reason: "the Body equipment slot does not apply to players",
                    });
                }
            })
        }
    }

    impl Equipment for super::ZombieKind {
        fn equipment_location(
            selector: sand_commands::Selector,
            slot: sand_components::EquipmentSlot,
        ) -> Result<crate::item::ItemLocation, crate::item::ItemLocationError> {
            crate::item::ItemLocation::entity_equipment(selector, slot)
        }
    }

    pub trait SafeDataWrite {}
    impl SafeDataWrite for super::ZombieKind {}
    impl SafeDataWrite for super::MarkerKind {}

    pub trait MountVehicle {}
    impl MountVehicle for super::AnyEntity {}
    impl MountVehicle for super::ZombieKind {}
}
