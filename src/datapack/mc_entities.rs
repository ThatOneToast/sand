#[derive(Debug, Clone, PartialEq)]
pub enum MinecraftEntity {
    // Passive Mobs
    Allay,
    Axolotl,
    Bat,
    Cat,
    Chicken,
    Cod,
    Cow,
    Dolphin,
    Donkey,
    Fox,
    Frog,
    GlowSquid,
    Horse,
    Mooshroom,
    Mule,
    Ocelot,
    Parrot,
    Pig,
    Rabbit,
    Salmon,
    Sheep,
    SkeletonHorse,
    Sniffer,
    SnowGolem,
    Squid,
    Strider,
    TropicalFish,
    Turtle,
    Villager,
    WanderingTrader,

    // Neutral Mobs
    Bee,
    CaveSpider,
    Enderman,
    Goat,
    IronGolem,
    Llama,
    Panda,
    PolarBear,
    Spider,
    TraderLlama,
    Wolf,
    ZombifiedPiglin,

    // Hostile Mobs
    Blaze,
    Creeper,
    Drowned,
    ElderGuardian,
    Endermite,
    Evoker,
    Ghast,
    Guardian,
    Hoglin,
    Husk,
    MagmaCube,
    Phantom,
    Piglin,
    PiglinBrute,
    Pillager,
    Ravager,
    Shulker,
    Silverfish,
    Skeleton,
    Slime,
    Stray,
    Vex,
    Vindicator,
    Warden,
    Witch,
    Wither,
    WitherSkeleton,
    Zoglin,
    Zombie,
    ZombieVillager,

    // Bosses
    EnderDragon,
    WitherBoss,

    // Projectiles
    Arrow,
    DragonFireball,
    Egg,
    EnderPearl,
    ExperienceBottle,
    FireworkRocket,
    FishingBobber,
    LargeFireball,
    LlamaSpit,
    ShulkerBullet,
    SmallFireball,
    Snowball,
    SpectralArrow,
    ThrownExperienceBottle,
    ThrownPotion,
    ThrownTrident,
    WitherSkull,

    // Vehicles
    Boat,
    ChestBoat,
    Minecart,
    ChestMinecart,
    CommandBlockMinecart,
    FurnaceMinecart,
    HopperMinecart,
    SpawnerMinecart,
    TNTMinecart,

    // Other
    ArmorStand,
    EndCrystal,
    EvokerFangs,
    ExperienceOrb,
    FallingBlock,
    Item,
    LeashKnot,
    Lightning,
    Marker,
    Painting,
    PrimedTnt,

    // Display Entities (1.19.4+)
    BlockDisplay,
    ItemDisplay,
    TextDisplay,
    Interaction,

    // Technical
    Player,
}

impl ToString for MinecraftEntity {
    fn to_string(&self) -> String {
        match self {
            // Passive Mobs
            MinecraftEntity::Allay => "allay",
            MinecraftEntity::Axolotl => "axolotl",
            MinecraftEntity::Bat => "bat",
            MinecraftEntity::Cat => "cat",
            MinecraftEntity::Chicken => "chicken",
            MinecraftEntity::Cod => "cod",
            MinecraftEntity::Cow => "cow",
            MinecraftEntity::Dolphin => "dolphin",
            MinecraftEntity::Donkey => "donkey",
            MinecraftEntity::Fox => "fox",
            MinecraftEntity::Frog => "frog",
            MinecraftEntity::GlowSquid => "glow_squid",
            MinecraftEntity::Horse => "horse",
            MinecraftEntity::Mooshroom => "mooshroom",
            MinecraftEntity::Mule => "mule",
            MinecraftEntity::Ocelot => "ocelot",
            MinecraftEntity::Parrot => "parrot",
            MinecraftEntity::Pig => "pig",
            MinecraftEntity::Rabbit => "rabbit",
            MinecraftEntity::Salmon => "salmon",
            MinecraftEntity::Sheep => "sheep",
            MinecraftEntity::SkeletonHorse => "skeleton_horse",
            MinecraftEntity::Sniffer => "sniffer",
            MinecraftEntity::SnowGolem => "snow_golem",
            MinecraftEntity::Squid => "squid",
            MinecraftEntity::Strider => "strider",
            MinecraftEntity::TropicalFish => "tropical_fish",
            MinecraftEntity::Turtle => "turtle",
            MinecraftEntity::Villager => "villager",
            MinecraftEntity::WanderingTrader => "wandering_trader",

            // Neutral Mobs
            MinecraftEntity::Bee => "bee",
            MinecraftEntity::CaveSpider => "cave_spider",
            MinecraftEntity::Enderman => "enderman",
            MinecraftEntity::Goat => "goat",
            MinecraftEntity::IronGolem => "iron_golem",
            MinecraftEntity::Llama => "llama",
            MinecraftEntity::Panda => "panda",
            MinecraftEntity::PolarBear => "polar_bear",
            MinecraftEntity::Spider => "spider",
            MinecraftEntity::TraderLlama => "trader_llama",
            MinecraftEntity::Wolf => "wolf",
            MinecraftEntity::ZombifiedPiglin => "zombified_piglin",

            // Hostile Mobs
            MinecraftEntity::Blaze => "blaze",
            MinecraftEntity::Creeper => "creeper",
            MinecraftEntity::Drowned => "drowned",
            MinecraftEntity::ElderGuardian => "elder_guardian",
            MinecraftEntity::Endermite => "endermite",
            MinecraftEntity::Evoker => "evoker",
            MinecraftEntity::Ghast => "ghast",
            MinecraftEntity::Guardian => "guardian",
            MinecraftEntity::Hoglin => "hoglin",
            MinecraftEntity::Husk => "husk",
            MinecraftEntity::MagmaCube => "magma_cube",
            MinecraftEntity::Phantom => "phantom",
            MinecraftEntity::Piglin => "piglin",
            MinecraftEntity::PiglinBrute => "piglin_brute",
            MinecraftEntity::Pillager => "pillager",
            MinecraftEntity::Ravager => "ravager",
            MinecraftEntity::Shulker => "shulker",
            MinecraftEntity::Silverfish => "silverfish",
            MinecraftEntity::Skeleton => "skeleton",
            MinecraftEntity::Slime => "slime",
            MinecraftEntity::Stray => "stray",
            MinecraftEntity::Vex => "vex",
            MinecraftEntity::Vindicator => "vindicator",
            MinecraftEntity::Warden => "warden",
            MinecraftEntity::Witch => "witch",
            MinecraftEntity::Wither => "wither",
            MinecraftEntity::WitherSkeleton => "wither_skeleton",
            MinecraftEntity::Zoglin => "zoglin",
            MinecraftEntity::Zombie => "zombie",
            MinecraftEntity::ZombieVillager => "zombie_villager",

            // Bosses
            MinecraftEntity::EnderDragon => "ender_dragon",
            MinecraftEntity::WitherBoss => "wither",

            // Projectiles
            MinecraftEntity::Arrow => "arrow",
            MinecraftEntity::DragonFireball => "dragon_fireball",
            MinecraftEntity::Egg => "egg",
            MinecraftEntity::EnderPearl => "ender_pearl",
            MinecraftEntity::ExperienceBottle => "experience_bottle",
            MinecraftEntity::FireworkRocket => "firework_rocket",
            MinecraftEntity::FishingBobber => "fishing_bobber",
            MinecraftEntity::LargeFireball => "fireball",
            MinecraftEntity::LlamaSpit => "llama_spit",
            MinecraftEntity::ShulkerBullet => "shulker_bullet",
            MinecraftEntity::SmallFireball => "small_fireball",
            MinecraftEntity::Snowball => "snowball",
            MinecraftEntity::SpectralArrow => "spectral_arrow",
            MinecraftEntity::ThrownExperienceBottle => "experience_bottle",
            MinecraftEntity::ThrownPotion => "potion",
            MinecraftEntity::ThrownTrident => "trident",
            MinecraftEntity::WitherSkull => "wither_skull",

            // Vehicles
            MinecraftEntity::Boat => "boat",
            MinecraftEntity::ChestBoat => "chest_boat",
            MinecraftEntity::Minecart => "minecart",
            MinecraftEntity::ChestMinecart => "chest_minecart",
            MinecraftEntity::CommandBlockMinecart => "command_block_minecart",
            MinecraftEntity::FurnaceMinecart => "furnace_minecart",
            MinecraftEntity::HopperMinecart => "hopper_minecart",
            MinecraftEntity::SpawnerMinecart => "spawner_minecart",
            MinecraftEntity::TNTMinecart => "tnt_minecart",

            // Other
            MinecraftEntity::ArmorStand => "armor_stand",
            MinecraftEntity::EndCrystal => "end_crystal",
            MinecraftEntity::EvokerFangs => "evoker_fangs",
            MinecraftEntity::ExperienceOrb => "experience_orb",
            MinecraftEntity::FallingBlock => "falling_block",
            MinecraftEntity::Item => "item",
            MinecraftEntity::LeashKnot => "leash_knot",
            MinecraftEntity::Lightning => "lightning_bolt",
            MinecraftEntity::Marker => "marker",
            MinecraftEntity::Painting => "painting",
            MinecraftEntity::PrimedTnt => "tnt",

            // Display Entities
            MinecraftEntity::BlockDisplay => "block_display",
            MinecraftEntity::ItemDisplay => "item_display",
            MinecraftEntity::TextDisplay => "text_display",
            MinecraftEntity::Interaction => "interaction",

            // Technical
            MinecraftEntity::Player => "player",
        }
        .to_string()
    }
}

// Additional helper methods could be implemented
impl MinecraftEntity {
    /// Returns whether this entity is considered hostile
    pub fn is_hostile(&self) -> bool {
        matches!(
            self,
            MinecraftEntity::Blaze
                | MinecraftEntity::Creeper
                | MinecraftEntity::Drowned
                | MinecraftEntity::ElderGuardian
                | MinecraftEntity::Endermite
                | MinecraftEntity::Evoker
                | MinecraftEntity::Ghast
                | MinecraftEntity::Guardian
                | MinecraftEntity::Hoglin
                | MinecraftEntity::Husk
                | MinecraftEntity::MagmaCube
                | MinecraftEntity::Phantom
                | MinecraftEntity::Piglin
                | MinecraftEntity::PiglinBrute
                | MinecraftEntity::Pillager
                | MinecraftEntity::Ravager
                | MinecraftEntity::Shulker
                | MinecraftEntity::Silverfish
                | MinecraftEntity::Skeleton
                | MinecraftEntity::Slime
                | MinecraftEntity::Stray
                | MinecraftEntity::Vex
                | MinecraftEntity::Vindicator
                | MinecraftEntity::Warden
                | MinecraftEntity::Witch
                | MinecraftEntity::Wither
                | MinecraftEntity::WitherSkeleton
                | MinecraftEntity::Zoglin
                | MinecraftEntity::Zombie
                | MinecraftEntity::ZombieVillager
        )
    }

    /// Returns whether this entity is considered passive
    pub fn is_passive(&self) -> bool {
        matches!(
            self,
            MinecraftEntity::Allay
                | MinecraftEntity::Axolotl
                | MinecraftEntity::Bat
                | MinecraftEntity::Cat
                | MinecraftEntity::Chicken
                | MinecraftEntity::Cod
                | MinecraftEntity::Cow
                | MinecraftEntity::Donkey
                | MinecraftEntity::Fox
                | MinecraftEntity::Frog
                | MinecraftEntity::GlowSquid
                | MinecraftEntity::Horse
                | MinecraftEntity::Mooshroom
                | MinecraftEntity::Mule
                | MinecraftEntity::Ocelot
                | MinecraftEntity::Parrot
                | MinecraftEntity::Pig
                | MinecraftEntity::Rabbit
                | MinecraftEntity::Salmon
                | MinecraftEntity::Sheep
                | MinecraftEntity::SkeletonHorse
                | MinecraftEntity::Sniffer
                | MinecraftEntity::SnowGolem
                | MinecraftEntity::Squid
                | MinecraftEntity::Strider
                | MinecraftEntity::TropicalFish
                | MinecraftEntity::Turtle
                | MinecraftEntity::Villager
                | MinecraftEntity::WanderingTrader
        )
    }

    /// Returns whether this entity is a projectile
    pub fn is_projectile(&self) -> bool {
        matches!(
            self,
            MinecraftEntity::Arrow
                | MinecraftEntity::DragonFireball
                | MinecraftEntity::Egg
                | MinecraftEntity::EnderPearl
                | MinecraftEntity::ExperienceBottle
                | MinecraftEntity::FireworkRocket
                | MinecraftEntity::FishingBobber
                | MinecraftEntity::LargeFireball
                | MinecraftEntity::LlamaSpit
                | MinecraftEntity::ShulkerBullet
                | MinecraftEntity::SmallFireball
                | MinecraftEntity::Snowball
                | MinecraftEntity::SpectralArrow
                | MinecraftEntity::ThrownExperienceBottle
                | MinecraftEntity::ThrownPotion
                | MinecraftEntity::ThrownTrident
                | MinecraftEntity::WitherSkull
        )
    }
}