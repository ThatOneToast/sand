//! Entity-kind markers distinguishing players from generic entities in the
//! typed query/context API (issue #227).

use std::fmt;

/// Marker for an "any entity" (`@e`-rooted) query/context kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnyEntity;

/// Marker for a player-only (`@a`-rooted) query/context kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerKind;

/// Marker for a typed vanilla Zombie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZombieKind;

/// Marker for a typed vanilla marker entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkerKind;

/// A query/context entity kind.
///
/// Sealed: only [`AnyEntity`] and [`PlayerKind`] implement it today.
/// Capability-specific kinds (living entities, individual mob types) are
/// follow-up work — see issue #228, which builds entity operations and
/// blueprints on top of this foundation.
pub trait EntityKind: sealed::Sealed + fmt::Debug + Clone + Copy + Default + 'static {
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

/// An entity kind with one statically known vanilla/custom entity type.
///
/// Archetypes require this stronger bound because their adoption selector and
/// summon command must have a concrete typed entity type.
pub trait KnownEntityKind: EntityKind {
    /// Validated entity type used by summon/adoption lowering.
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

/// Capability implemented by living entities, including players.
///
/// This enables typed health observation, effects, damage, and safe
/// player-specific operations. Direct arbitrary entity-NBT writes require the
/// narrower [`MutableLivingEntityKind`] capability.
pub trait LivingEntityKind: KnownEntityKind + sealed::Living {}

impl LivingEntityKind for PlayerKind {}
impl LivingEntityKind for ZombieKind {}

/// A non-player living entity whose native data and attributes may be mutated.
///
/// `PlayerKind` intentionally does not implement this trait, structurally
/// preventing archetype health/attribute/equipment lowering from emitting
/// unsafe player entity-NBT commands.
pub trait MutableLivingEntityKind: LivingEntityKind + sealed::MutableLiving {}

impl MutableLivingEntityKind for ZombieKind {}

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
