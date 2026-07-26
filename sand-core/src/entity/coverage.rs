//! Audited coverage of entity properties and specialized entity families.
//!
//! This module is a machine-readable statement of the entity runtime surface.
//! It is intentionally descriptive: an entry does not itself emit commands and
//! [`EntityCapabilityStatus::SpecializedFollowUp`] is not a promise that a raw
//! NBT mutation is safe. Exporters and diagnostics can use the matrix to explain
//! why an operation is unavailable for an entity kind or Minecraft profile.
//!
//! The matrix is static and ordered by [`EntityRuntimeOperation`]. It therefore
//! has no process-global registration state, cannot leak between exports, and
//! produces stable iteration order under Rust's parallel test harness.

/// Broad part of the vanilla entity model to which an operation belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityPropertyFamily {
    /// Properties shared by most entities, such as tags and custom names.
    Universal,
    /// Position, orientation, motion, counters, and entity relationships.
    Spatial,
    /// Properties available only to living entities.
    Living,
    /// Common capability extensions such as ageable or tameable entities.
    Extension,
    /// Families whose semantics require a dedicated API and validation model.
    Specialized,
}

/// The primary level of typed support intended for an entity operation.
///
/// Profile restrictions and player safety are represented separately by
/// [`EntityProfileSupport`] and [`EntityPlayerSafety`]. Consumers must inspect
/// all three fields rather than treating this value as the complete verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityCapabilityStatus {
    /// Available through a typed handle shared by general entity kinds.
    TypedGeneral,
    /// Available only after proving the entity has the required capability.
    TypedCapabilitySpecific,
    /// Intentionally deferred to a dedicated, specialized entity API.
    SpecializedFollowUp,
    /// Reliably observable, but not safely or generally mutable.
    ReadOnly,
    /// Available only through an explicitly named raw escape hatch.
    RawOnly,
    /// Unsupported because the operation cannot be represented reliably.
    NotReliablyRepresentable,
}

/// Minecraft-profile constraints on an entity operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EntityProfileSupport {
    /// The operation is available in every Minecraft profile supported by Sand.
    AllSupported,
    /// The exact command, NBT, component, or registry form is profile-dependent.
    VersionGated {
        /// Human-readable boundary used in capability diagnostics.
        requirement: &'static str,
    },
    /// Vanilla datapacks do not expose enough stable behavior to support it.
    NotReliablyRepresentable,
}

/// Whether an entity operation may be applied when `@s` is a player.
///
/// This classification is about command safety, not selector cardinality.
/// In particular, [`EntityPlayerSafety::UnsafeEntityNbtMutation`] prevents an
/// exporter from treating player data as ordinary mutable entity NBT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityPlayerSafety {
    /// The typed lowering is safe for players.
    Safe,
    /// A player-specific command path or validation rule is required.
    RequiresPlayerSpecificLowering,
    /// The normal entity implementation would mutate player NBT and is unsafe.
    UnsafeEntityNbtMutation,
    /// The capability does not apply to players.
    NotApplicable,
}

/// Issue that owns work deliberately excluded from the general entity runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EntityCoverageOwner {
    /// General entity operations and specialized runtime follow-ups in issue #228.
    EntityRuntime228,
}

/// A stable operation or specialized family in the entity coverage audit.
///
/// Variants are declaration-ordered so [`ENTITY_RUNTIME_COVERAGE`] can be
/// mechanically checked for deterministic ordering and complete one-to-one
/// coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum EntityRuntimeOperation {
    // Universal
    /// Add, remove, and test entity tags.
    Tags,
    /// Join, leave, and query scoreboard teams.
    Teams,
    /// Read or write a typed custom name.
    CustomName,
    /// Control whether a custom name is visible.
    CustomNameVisibility,
    /// Control the glowing flag.
    Glowing,
    /// Control whether an entity emits sounds.
    Silent,
    /// Control ordinary damage immunity.
    Invulnerable,
    /// Control gravity where the entity family honors it.
    Gravity,
    /// Keep eligible mobs from naturally despawning.
    Persistence,
    /// Remove the entity from the world.
    Removal,
    /// Access stable, typed entity data adapters.
    TypedData,
    /// Access unsupported entity data through an explicit raw escape hatch.
    RawData,
    /// Store an arbitrary entity identity for use in a later execution chain.
    PersistentEntityReference,
    // Spatial
    /// Observe the current position.
    Position,
    /// Teleport an entity.
    Teleport,
    /// Observe or change yaw and pitch.
    Rotation,
    /// Face a position or another entity.
    Facing,
    /// Observe or change motion where vanilla permits it.
    Motion,
    /// Read or change the remaining fire counter.
    FireCounter,
    /// Read or change fall-distance state.
    FallDistance,
    /// Read or change the remaining air counter.
    AirCounter,
    /// Read or change the frozen-ticks counter.
    FrozenTicks,
    /// Traverse the bound entity's passengers.
    Passengers,
    /// Traverse the vehicle ridden by the bound entity.
    Vehicle,
    // Living
    /// Observe or synchronize current and maximum health.
    Health,
    /// Observe or synchronize absorption.
    Absorption,
    /// Read and change attributes through typed registry identifiers.
    Attributes,
    /// Add, remove, and observe typed status effects.
    Effects,
    /// Apply damage or healing.
    DamageAndHealing,
    /// Read and change typed equipment slots and item stacks.
    Equipment,
    /// Control equipment-drop behavior.
    DropBehavior,
    /// Control supported living-entity AI flags.
    AiFlags,
    /// Apply living-specific persistence behavior.
    LivingPersistence,
    // Extensions
    /// Age, breeding cooldown, and baby/adult behavior.
    Ageable,
    /// Taming state and tameable-family behavior.
    Tameable,
    /// Owner identity and ownership relationships.
    Owner,
    /// Sitting state for supported tameable families.
    Sitting,
    /// Anger targets and anger duration.
    Anger,
    /// Family-specific variants such as cat or frog variants.
    Variants,
    /// Entity-owned inventories outside ordinary equipment.
    Inventories,
    // Specialized families
    /// Villagers and wandering traders as runtime entities.
    VillagersAndWanderingTraders,
    /// Text, item, and block display entities.
    DisplayEntities,
    /// Interaction entities.
    InteractionEntities,
    /// Boats, minecarts, mounts, and other vehicles.
    VehiclesAndMounts,
    /// Arrows, fireballs, thrown items, and other projectiles.
    Projectiles,
    /// Dropped item entities.
    ItemEntities,
    /// Experience orb entities.
    ExperienceOrbs,
    /// Area-effect cloud entities.
    AreaEffectClouds,
    /// Paintings and item frames.
    PaintingsAndItemFrames,
    /// Inventory-bearing mounts and vehicle containers.
    InventoryBearingMounts,
}

impl EntityRuntimeOperation {
    /// Stable diagnostic identifier for this operation.
    ///
    /// These identifiers are documentation and diagnostic keys, not generated
    /// Minecraft resource locations.
    pub const fn id(self) -> &'static str {
        use EntityRuntimeOperation::*;
        match self {
            Tags => "universal.tags",
            Teams => "universal.teams",
            CustomName => "universal.custom_name",
            CustomNameVisibility => "universal.custom_name_visibility",
            Glowing => "universal.glowing",
            Silent => "universal.silent",
            Invulnerable => "universal.invulnerable",
            Gravity => "universal.gravity",
            Persistence => "universal.persistence",
            Removal => "universal.removal",
            TypedData => "universal.typed_data",
            RawData => "universal.raw_data",
            PersistentEntityReference => "universal.persistent_entity_reference",
            Position => "spatial.position",
            Teleport => "spatial.teleport",
            Rotation => "spatial.rotation",
            Facing => "spatial.facing",
            Motion => "spatial.motion",
            FireCounter => "spatial.fire_counter",
            FallDistance => "spatial.fall_distance",
            AirCounter => "spatial.air_counter",
            FrozenTicks => "spatial.frozen_ticks",
            Passengers => "spatial.passengers",
            Vehicle => "spatial.vehicle",
            Health => "living.health",
            Absorption => "living.absorption",
            Attributes => "living.attributes",
            Effects => "living.effects",
            DamageAndHealing => "living.damage_and_healing",
            Equipment => "living.equipment",
            DropBehavior => "living.drop_behavior",
            AiFlags => "living.ai_flags",
            LivingPersistence => "living.persistence",
            Ageable => "extension.ageable",
            Tameable => "extension.tameable",
            Owner => "extension.owner",
            Sitting => "extension.sitting",
            Anger => "extension.anger",
            Variants => "extension.variants",
            Inventories => "extension.inventories",
            VillagersAndWanderingTraders => "specialized.villagers_and_wandering_traders",
            DisplayEntities => "specialized.display_entities",
            InteractionEntities => "specialized.interaction_entities",
            VehiclesAndMounts => "specialized.vehicles_and_mounts",
            Projectiles => "specialized.projectiles",
            ItemEntities => "specialized.item_entities",
            ExperienceOrbs => "specialized.experience_orbs",
            AreaEffectClouds => "specialized.area_effect_clouds",
            PaintingsAndItemFrames => "specialized.paintings_and_item_frames",
            InventoryBearingMounts => "specialized.inventory_bearing_mounts",
        }
    }
}

/// One audited entity operation and its support boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityCapabilityCoverage {
    /// Operation or specialized family being classified.
    pub operation: EntityRuntimeOperation,
    /// Broad entity-property family.
    pub family: EntityPropertyFamily,
    /// Primary typed-support classification.
    pub status: EntityCapabilityStatus,
    /// Minecraft-profile restriction.
    pub profile: EntityProfileSupport,
    /// Safety of applying the operation to a player.
    pub player_safety: EntityPlayerSafety,
    /// Issue responsible for deliberately deferred implementation work.
    pub owner: Option<EntityCoverageOwner>,
    /// Concise rationale suitable for coverage reports and diagnostics.
    pub notes: &'static str,
}

const ALL: EntityProfileSupport = EntityProfileSupport::AllSupported;
const NBT_GATED: EntityProfileSupport = EntityProfileSupport::VersionGated {
    requirement: "validate the field, component, and command form against the export profile",
};
const VANILLA_LIMIT: EntityProfileSupport = EntityProfileSupport::NotReliablyRepresentable;
const SAFE: EntityPlayerSafety = EntityPlayerSafety::Safe;
const PLAYER_PATH: EntityPlayerSafety = EntityPlayerSafety::RequiresPlayerSpecificLowering;
const UNSAFE_NBT: EntityPlayerSafety = EntityPlayerSafety::UnsafeEntityNbtMutation;
const NOT_PLAYER: EntityPlayerSafety = EntityPlayerSafety::NotApplicable;
const ISSUE_228: Option<EntityCoverageOwner> = Some(EntityCoverageOwner::EntityRuntime228);

macro_rules! row {
    ($operation:ident, $family:ident, $status:ident, $profile:expr, $players:expr, $owner:expr, $notes:literal) => {
        EntityCapabilityCoverage {
            operation: EntityRuntimeOperation::$operation,
            family: EntityPropertyFamily::$family,
            status: EntityCapabilityStatus::$status,
            profile: $profile,
            player_safety: $players,
            owner: $owner,
            notes: $notes,
        }
    };
}

/// Complete, deterministic entity-runtime coverage matrix.
///
/// Entries are ordered by [`EntityRuntimeOperation`]. Specialized families are
/// deliberately assigned to issue #228 unless and until a narrower runtime
/// issue owns them. Issue #296 is **not** an owner here: it covers villager
/// trade resources, not villager or wandering-trader runtime entity APIs.
pub static ENTITY_RUNTIME_COVERAGE: &[EntityCapabilityCoverage] = &[
    row!(
        Tags,
        Universal,
        TypedGeneral,
        ALL,
        SAFE,
        None,
        "Typed tag mutation preserves unrelated tags unless exact ownership is requested."
    ),
    row!(
        Teams,
        Universal,
        TypedGeneral,
        ALL,
        SAFE,
        None,
        "Scoreboard teams provide the player-safe lowering for faction and presentation state."
    ),
    row!(
        CustomName,
        Universal,
        TypedGeneral,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Non-player names use profile-aware entity data; players require their distinct display-name facilities."
    ),
    row!(
        CustomNameVisibility,
        Universal,
        TypedGeneral,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "The underlying entity field is profile-validated and must not be written to player NBT."
    ),
    row!(
        Glowing,
        Universal,
        TypedGeneral,
        NBT_GATED,
        PLAYER_PATH,
        None,
        "Teams and effects may provide a safer player lowering than entity data."
    ),
    row!(
        Silent,
        Universal,
        TypedGeneral,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Player NBT mutation is unsafe even though the field is common on non-player entities."
    ),
    row!(
        Invulnerable,
        Universal,
        TypedGeneral,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Player invulnerability needs a player-specific supported command path."
    ),
    row!(
        Gravity,
        Universal,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Only expose mutation after validating that the concrete entity family honors gravity."
    ),
    row!(
        Persistence,
        Universal,
        TypedCapabilitySpecific,
        NBT_GATED,
        NOT_PLAYER,
        None,
        "Natural-despawn persistence applies to eligible non-player mob families."
    ),
    row!(
        Removal,
        Universal,
        TypedGeneral,
        ALL,
        PLAYER_PATH,
        None,
        "Players must use a player-safe kick or lifecycle path rather than entity removal."
    ),
    row!(
        TypedData,
        Universal,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Only stable fields receive typed adapters; each adapter validates entity kind and profile."
    ),
    row!(
        RawData,
        Universal,
        RawOnly,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Unsupported fields remain behind an explicitly named raw escape hatch."
    ),
    row!(
        PersistentEntityReference,
        Universal,
        NotReliablyRepresentable,
        VANILLA_LIMIT,
        SAFE,
        ISSUE_228,
        "EntityContext remains execution-scoped; selectors and temporary tags cannot create a durable arbitrary-entity reference."
    ),
    row!(
        Position,
        Spatial,
        ReadOnly,
        ALL,
        SAFE,
        None,
        "Position observation is execution-scoped to the entity currently bound to @s."
    ),
    row!(
        Teleport,
        Spatial,
        TypedGeneral,
        ALL,
        SAFE,
        None,
        "Teleport uses typed coordinates and preserves execution-scoped identity."
    ),
    row!(
        Rotation,
        Spatial,
        TypedGeneral,
        ALL,
        SAFE,
        None,
        "Yaw and pitch lowering is supported through teleport/rotation commands."
    ),
    row!(
        Facing,
        Spatial,
        TypedGeneral,
        ALL,
        SAFE,
        None,
        "Facing uses typed coordinates or the current execution chain's entity target."
    ),
    row!(
        Motion,
        Spatial,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Legal motion writes vary by entity kind and are unsafe for players."
    ),
    row!(
        FireCounter,
        Spatial,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Counter representation and writable behavior are profile-validated."
    ),
    row!(
        FallDistance,
        Spatial,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Direct player entity-data mutation is not a supported lowering."
    ),
    row!(
        AirCounter,
        Spatial,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Air semantics require an entity kind that can breathe."
    ),
    row!(
        FrozenTicks,
        Spatial,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "The field and legal mutation are checked against the export profile."
    ),
    row!(
        Passengers,
        Spatial,
        ReadOnly,
        NBT_GATED,
        SAFE,
        None,
        "Typed execute-on traversal observes loaded relationships but does not create a durable reference."
    ),
    row!(
        Vehicle,
        Spatial,
        ReadOnly,
        NBT_GATED,
        SAFE,
        None,
        "Typed execute-on traversal is execution-scoped and only sees loaded relationships."
    ),
    row!(
        Health,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        PLAYER_PATH,
        None,
        "Living handles synchronize current health and max-health attributes with explicit resize policy."
    ),
    row!(
        Absorption,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        PLAYER_PATH,
        None,
        "Players require supported attribute/effect/command lowering rather than entity NBT writes."
    ),
    row!(
        Attributes,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        SAFE,
        None,
        "Typed attribute registries and entity-kind validation gate base values and modifiers."
    ),
    row!(
        Effects,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        SAFE,
        None,
        "Typed effect registries prevent applying living-only operations to markers and displays."
    ),
    row!(
        DamageAndHealing,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        SAFE,
        None,
        "Damage and healing commands require a living-capability handle."
    ),
    row!(
        Equipment,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        PLAYER_PATH,
        None,
        "Typed slots and item stacks use player-safe inventory commands where applicable."
    ),
    row!(
        DropBehavior,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        UNSAFE_NBT,
        None,
        "Drop-chance fields are mob-specific and not safely mutable for players."
    ),
    row!(
        AiFlags,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        NOT_PLAYER,
        None,
        "AI controls apply only to validated mob families."
    ),
    row!(
        LivingPersistence,
        Living,
        TypedCapabilitySpecific,
        NBT_GATED,
        NOT_PLAYER,
        None,
        "Living persistence is separate from ownership of unrelated mob data."
    ),
    row!(
        Ageable,
        Extension,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Age and breeding semantics vary by ageable family and remain follow-up runtime work."
    ),
    row!(
        Tameable,
        Extension,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Taming requires family-specific ownership and lifecycle semantics."
    ),
    row!(
        Owner,
        Extension,
        ReadOnly,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Existing execute-on owner traversal is observable; persistent owner mutation remains follow-up work."
    ),
    row!(
        Sitting,
        Extension,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Sitting is not a universal tameable field and needs family validation."
    ),
    row!(
        Anger,
        Extension,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Anger targets and timers need family-specific typed adapters."
    ),
    row!(
        Variants,
        Extension,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Variant registries and encodings differ across entity families and profiles."
    ),
    row!(
        Inventories,
        Extension,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Non-equipment inventories require dedicated container semantics."
    ),
    row!(
        VillagersAndWanderingTraders,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Runtime professions, offers, gossip, and restocking are outside #295; #296 owns trade resources only and does not own this runtime family."
    ),
    row!(
        DisplayEntities,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Display interpolation, transformation, and payloads require a dedicated API."
    ),
    row!(
        InteractionEntities,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Interaction dimensions, response, and last-interaction evidence require dedicated semantics."
    ),
    row!(
        VehiclesAndMounts,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Mount controls, physics, and passenger mutation are not general entity properties."
    ),
    row!(
        Projectiles,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Projectile ownership, motion, payload, and impact lifecycle require dedicated support."
    ),
    row!(
        ItemEntities,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Dropped stacks, pickup delay, age, and ownership require item-entity support without duplicating the item model."
    ),
    row!(
        ExperienceOrbs,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Orb value, merge behavior, and targeting require specialized validation."
    ),
    row!(
        AreaEffectClouds,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Radius, duration, effects, and owner behavior form a specialized lifecycle."
    ),
    row!(
        PaintingsAndItemFrames,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Attached-position, facing, variant, and displayed-item behavior require specialized APIs."
    ),
    row!(
        InventoryBearingMounts,
        Specialized,
        SpecializedFollowUp,
        NBT_GATED,
        NOT_PLAYER,
        ISSUE_228,
        "Container slots and mount behavior must be modeled together rather than exposed as raw entity data."
    ),
];

/// Returns the complete entity-runtime coverage matrix in stable operation order.
pub const fn entity_runtime_coverage() -> &'static [EntityCapabilityCoverage] {
    ENTITY_RUNTIME_COVERAGE
}

/// Looks up one operation's audited coverage.
///
/// This performs a deterministic binary search over the static matrix and does
/// not allocate or consult mutable global state.
pub fn entity_runtime_capability(
    operation: EntityRuntimeOperation,
) -> &'static EntityCapabilityCoverage {
    ENTITY_RUNTIME_COVERAGE
        .binary_search_by_key(&operation, |entry| entry.operation)
        .map(|index| &ENTITY_RUNTIME_COVERAGE[index])
        .expect("the exhaustive static entity coverage matrix contains every operation")
}

/// Iterates entries in one family while preserving matrix order.
pub fn entity_runtime_family(
    family: EntityPropertyFamily,
) -> impl Iterator<Item = &'static EntityCapabilityCoverage> {
    ENTITY_RUNTIME_COVERAGE
        .iter()
        .filter(move |entry| entry.family == family)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn matrix_is_unique_ordered_and_exhaustive() {
        let operations: Vec<_> = ENTITY_RUNTIME_COVERAGE
            .iter()
            .map(|entry| entry.operation)
            .collect();
        let unique: BTreeSet<_> = operations.iter().copied().collect();

        assert_eq!(operations.len(), 50);
        assert_eq!(unique.len(), operations.len(), "duplicate coverage entry");
        assert!(
            operations.windows(2).all(|pair| pair[0] < pair[1]),
            "matrix order must follow EntityRuntimeOperation"
        );
        assert_eq!(
            operations.first(),
            Some(&EntityRuntimeOperation::Tags),
            "first enum variant is missing"
        );
        assert_eq!(
            operations.last(),
            Some(&EntityRuntimeOperation::InventoryBearingMounts),
            "last enum variant is missing"
        );
    }

    #[test]
    fn operation_ids_are_unique_and_stable() {
        let ids: BTreeSet<_> = ENTITY_RUNTIME_COVERAGE
            .iter()
            .map(|entry| entry.operation.id())
            .collect();
        assert_eq!(ids.len(), ENTITY_RUNTIME_COVERAGE.len());
        assert_eq!(
            EntityRuntimeOperation::VillagersAndWanderingTraders.id(),
            "specialized.villagers_and_wandering_traders"
        );
        assert_eq!(
            EntityRuntimeOperation::DamageAndHealing.id(),
            "living.damage_and_healing"
        );
    }

    #[test]
    fn family_iteration_preserves_classification_and_order() {
        let specialized: Vec<_> =
            entity_runtime_family(EntityPropertyFamily::Specialized).collect();
        assert_eq!(specialized.len(), 10);
        assert!(
            specialized
                .windows(2)
                .all(|pair| pair[0].operation < pair[1].operation)
        );
        assert!(specialized.iter().all(|entry| {
            entry.status == EntityCapabilityStatus::SpecializedFollowUp
                && entry.owner == Some(EntityCoverageOwner::EntityRuntime228)
                && entry.player_safety == EntityPlayerSafety::NotApplicable
        }));
    }

    #[test]
    fn representative_capabilities_are_honestly_classified() {
        assert_eq!(
            entity_runtime_capability(EntityRuntimeOperation::Tags).status,
            EntityCapabilityStatus::TypedGeneral
        );
        assert_eq!(
            entity_runtime_capability(EntityRuntimeOperation::Health).status,
            EntityCapabilityStatus::TypedCapabilitySpecific
        );
        assert_eq!(
            entity_runtime_capability(EntityRuntimeOperation::RawData).status,
            EntityCapabilityStatus::RawOnly
        );
        assert_eq!(
            entity_runtime_capability(EntityRuntimeOperation::Passengers).status,
            EntityCapabilityStatus::ReadOnly
        );
        assert_eq!(
            entity_runtime_capability(EntityRuntimeOperation::Motion).player_safety,
            EntityPlayerSafety::UnsafeEntityNbtMutation
        );
    }

    #[test]
    fn villager_runtime_is_not_assigned_to_trade_resource_issue_296() {
        let villagers =
            entity_runtime_capability(EntityRuntimeOperation::VillagersAndWanderingTraders);
        assert_eq!(
            villagers.owner,
            Some(EntityCoverageOwner::EntityRuntime228),
            "#296 covers trade resources, not villager runtime entities"
        );
        assert!(villagers.notes.contains("#296 owns trade resources only"));
    }

    #[test]
    fn repeated_iteration_has_identical_output() {
        let snapshot = || {
            entity_runtime_coverage()
                .iter()
                .map(|entry| {
                    (
                        entry.operation.id(),
                        entry.family,
                        entry.status,
                        entry.profile,
                        entry.player_safety,
                        entry.owner,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(snapshot(), snapshot());
    }

    #[test]
    fn persistent_references_are_explicitly_not_representable() {
        let persistent =
            entity_runtime_capability(EntityRuntimeOperation::PersistentEntityReference);
        assert_eq!(
            persistent.status,
            EntityCapabilityStatus::NotReliablyRepresentable
        );
        assert_eq!(
            persistent.profile,
            EntityProfileSupport::NotReliablyRepresentable
        );
    }
}
