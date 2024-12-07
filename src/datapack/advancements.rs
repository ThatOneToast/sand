#[derive(Debug, Clone)]
pub enum Advancements {
    Minecraft,
    StoneAge,
    GettingAnUpgrade,
    AcquireHardware,
    SuitUp,
    HotStuff,
    IsntItIronPick,
    NotTodayThankYou,
    IceBucketChallenge,
    Diamonds,
    WeNeedToGoDeeper,
    CoverMeWithDiamonds,
    Enchanter,
    ZombieDoctor,
    EyeSpy,
    TheEnd,
    
    Nether,
    ReturnToSender,
    ThoseWereTheDays,
    HiddenInTheDepths,
    SubspaceBubble,
    ATerribleFortress,
    WhoIsCuttingOnions,
    OhShiny,
    ThisBoatHasLegs,
    UneasyAlliance,
    WarPigs,
    CountryLodeTakeMeHome,
    CoverMeInDebris,
    SpookyScarySkeleton,
    IntoFire,
    NotQuiteNineLives,
    FeelsLikeHome,
    HotTouristDestinations,
    WitheringHeights,
    LocalBrewary,
    BringHomeTheBeacon,
    AFuriousCocktail,
    Beaconator,
    HowDidWeGetHere,
    
    TheEndOrTheBeginning,
    FreeTheEnd,
    TheNextGeneration,
    RemoteGateway,
    TheEndAgain,
    YouNeedAMint,
    TheCityAtTheEndOfTheGame,
    SkysTheLimit,
    GreatViewFromUpHere,
    
    Adventure,
    VoluntaryExile,
    IsItABird,
    MonsterHunter,
    ThePowerOfBooks,
    WhatADeal,
    CraftingANewLook,
    StickySituation,
    OlBetsy,
    SurgeProtector,
    CabvesAndCliffs,
    RespectingTheRemmants,
    Sneak100,
    SweetDreams,
    HeroOfTheVillage,
    IsItABalloon,
    AThrowawayJoke,
    ItSpreads,
    TakeAim,
    MonstersHunted,
    Postmortal,
    HiredHelp,
    StarTrader,
    SmithingWithStyle,
    TwoBirdsOneArrow,
    WhosThePillagerNow,
    Arbalistic,
    CarefulRestoration,
    AdventuringTime,
    SoundOfMusic,
    LightAsARabbit,
    IsItAPlane,
    VeryVeryFrightening,
    SniperDuel,
    Bullseye,
    
    Husbandry,
    BeeOurGuest,
    TheParrotsAndTheBats,
    YouveGotAFriendInMe,
    WhateverFloatsYourBoat,
    BestFriendsForever,
    GlowAndBehold,
    FishyBusisness,
    TotalBeeLocation,
    BukkitBukkit,
    SmellsInteresting,
    ASeedyPlace,
    WaxOn,
    TwoByTow,
    BirthdaySong,
    ACompleteCatalogue,
    TacticalFishing,
    WhenTheSquadHopsIntoTown,
    LittleSniffs,
    ABalancedDiet,
    SeriousDedication,
    WaxOff,
    TheCutestPredator,
    WithOurPowersCombined,
    PlantingThePast,
    TheHealingPowerOfFriendship
}

impl ToString for Advancements {
    fn to_string(&self) -> String {
        match self {
        Advancements::Minecraft => "minecraft:story/root".to_string(),
        Advancements::StoneAge => "minecraft:story/stone_age".to_string(),
        Advancements::GettingAnUpgrade => "minecraft:story/upgrade_tools".to_string(),
        Advancements::AcquireHardware => "minecraft:story/smelt_iron".to_string(),
        Advancements::SuitUp => "minecraft:story/obtain_armor".to_string(),
        Advancements::HotStuff => "minecraft:story/lava_bucket".to_string(),
        Advancements::IsntItIronPick => "minecraft:story/iron_tools".to_string(),
        Advancements::NotTodayThankYou => "minecraft:story/deflect_arrow".to_string(),
        Advancements::IceBucketChallenge => "minecraft:story/form_obsidian".to_string(),
        Advancements::Diamonds => "minecraft:story/mine_diamond".to_string(),
        Advancements::WeNeedToGoDeeper => "minecraft:story/enter_the_nether".to_string(),
        Advancements::CoverMeWithDiamonds => "minecraft:story/shiny_gear".to_string(),
        Advancements::Enchanter => "minecraft:story/enchant_item".to_string(),
        Advancements::ZombieDoctor => "minecraft:story/cure_zombie_villager".to_string(),
        Advancements::EyeSpy => "minecraft:story/follow_ender_eye".to_string(),
        Advancements::TheEnd => "minecraft:story/enter_the_end".to_string(),
        
        Advancements::Nether => "minecraft:nether/root".to_string(),
        Advancements::ReturnToSender => "minecraft:nether/return_to_sender".to_string(),
        Advancements::ThoseWereTheDays => "minecraft:nether/find_bastion".to_string(),
        Advancements::HiddenInTheDepths => "minecraft:nether/obtain_ancient_debris".to_string(),
        Advancements::SubspaceBubble => "minecraft:nether/fast_travel".to_string(),
        Advancements::ATerribleFortress => "minecraft:nether/find_fortress".to_string(),
        Advancements::WhoIsCuttingOnions => "minecraft:nether/obtain_crying_obsidian".to_string(),
        Advancements::OhShiny => "minecraft:nether/distract_piglin".to_string(),
        Advancements::ThisBoatHasLegs => "minecraft:nether/ride_strider".to_string(),
        Advancements::UneasyAlliance => "minecraft:nether/uneasy_alliance".to_string(),
        Advancements::WarPigs => "minecraft:nether/loot_bastion".to_string(),
        Advancements::CountryLodeTakeMeHome => "minecraft:nether/use_lodestone".to_string(),
        Advancements::CoverMeInDebris => "minecraft:nether/netherite_armor".to_string(),
        Advancements::SpookyScarySkeleton => "minecraft:nether/get_wither_skull".to_string(),
        Advancements::IntoFire => "minecraft:nether/obtain_blaze_rod".to_string(),
        Advancements::NotQuiteNineLives => "minecraft:nether/charge_respawn_anchor".to_string(),
        Advancements::FeelsLikeHome => "minecraft:nether/ride_strider_in_overworl_lava".to_string(),
        Advancements::HotTouristDestinations => "minecraft:nether/explore_nether".to_string(),
        Advancements::WitheringHeights => "minecraft:nether/summon_wither".to_string(),
        Advancements::LocalBrewary => "minecraft:nether/brew_potion".to_string(),
        Advancements::BringHomeTheBeacon => "minecraft:nether/create_beacon".to_string(),
        Advancements::AFuriousCocktail => "minecraft:nether/all_potions".to_string(),
        Advancements::Beaconator => "minecraft:nether/create_full_beacon".to_string(),
        Advancements::HowDidWeGetHere => "minecraft:nether/all_effects".to_string(),
        
        Advancements::TheEndOrTheBeginning => "minecraft:end/root".to_string(),
        Advancements::FreeTheEnd => "minecraft:end/kill_dragon".to_string(),
        Advancements::TheNextGeneration => "minecraft:end/dragon_egg".to_string(),
        Advancements::RemoteGateway => "minecraft:end/enter_end_gateway".to_string(),
        Advancements::TheEndAgain => "minecraft:end/respawn_dragon".to_string(),
        Advancements::YouNeedAMint => "minecraft:end/dragon_breath".to_string(),
        Advancements::TheCityAtTheEndOfTheGame => "minecraft:end/find_end_city".to_string(),
        Advancements::SkysTheLimit => "minecraft:end/elytra".to_string(),
        Advancements::GreatViewFromUpHere => "minecraft:end/levitate".to_string(),
        
        Advancements::Adventure => "minecraft:adventure/root".to_string(),
        Advancements::VoluntaryExile => "minecraft:adventure/voluntary_exile".to_string(),
        Advancements::IsItABird => "minecraft:adventure/spyglass_at_parrot".to_string(),
        Advancements::MonsterHunter => "minecraft:adventure/kill_a_mob".to_string(),
        Advancements::ThePowerOfBooks => "minecraft:adventure/read_power_of_chiseled_bookshelf".to_string(),
        Advancements::WhatADeal => "minecraft:adventure/trade".to_string(),
        Advancements::CraftingANewLook => "minecraft:adventure/trim_with_any_armor_pattern".to_string(),
        Advancements::StickySituation => "minecraft:adventure/honey_block_slide".to_string(),
        Advancements::OlBetsy => "minecraft:adventure/ol_betsy".to_string(),
        Advancements::SurgeProtector => "minecraft:adventure/lightning_rod_with_villager_no_fire".to_string(),
        Advancements::CabvesAndCliffs => "minecraft:adventure/fall_from_world_height".to_string(),
        Advancements::RespectingTheRemmants => "minecraft:adventure/salvage_shred".to_string(),
        Advancements::Sneak100 => "minecraft:adventure/avoid_vibration".to_string(),
        Advancements::SweetDreams => "minecraft:adventure/sleep_in_bed".to_string(),
        Advancements::HeroOfTheVillage => "minecraft:adventure/hero_of_the_village".to_string(),
        Advancements::IsItABalloon => "minecraft:adventure/spyglass_at_ghast".to_string(),
        Advancements::AThrowawayJoke => "minecraft:adventure/throw_trident".to_string(),
        Advancements::ItSpreads => "minecraft:adventure/adventure/kill_mob_near_sculk_catalyst".to_string(),
        Advancements::TakeAim => "minecraft:adventure/shoot_arrow".to_string(),
        Advancements::MonstersHunted => "minecraft:adventure/kill_all_mobs".to_string(),
        Advancements::Postmortal => "minecraft:adventure/totem_of_undying".to_string(),
        Advancements::HiredHelp => "minecraft:adventure/summon_iron_golem".to_string(),
        Advancements::StarTrader => "minecraft:adventure/trade_at_world_height".to_string(),
        Advancements::SmithingWithStyle => "minecraft:adventure/trim_with_all_exclusive_armor_patterns".to_string(),
        Advancements::TwoBirdsOneArrow => "minecraft:adventure/two_birds_one_arrow".to_string(),
        Advancements::WhosThePillagerNow => "minecraft:adventure/whos_the_pillager_now".to_string(),
        Advancements::Arbalistic => "minecraft:adventure/arbalistic".to_string(),
        Advancements::CarefulRestoration => "minecraft:adventure/craft_decorated_pot_using_only_sherds".to_string(),
        Advancements::AdventuringTime => "minecraft:adventure/adventuring_time".to_string(),
        Advancements::SoundOfMusic => "minecraft:adventure/play_jukebox_in_meadows".to_string(),
        Advancements::LightAsARabbit => "minecraft:adventure/walk_on_powder_snow_with_leather_boots".to_string(),
        Advancements::IsItAPlane => "minecraft:adventure/spyglass_at_dragon".to_string(),
        Advancements::VeryVeryFrightening => "minecraft:adventure/very_very_frightening".to_string(),
        Advancements::SniperDuel => "minecraft:adventure/sniper_duel".to_string(),
        Advancements::Bullseye => "minecraft:adventure/bullseye".to_string(),
        
        Advancements::Husbandry => "minecraft:husbandry/root".to_string(),
        Advancements::BeeOurGuest => "minecraft:husbandry/safely_harvest_honey".to_string(),
        Advancements::TheParrotsAndTheBats => "minecraft:husbandry/breed_an_animal".to_string(),
        Advancements::YouveGotAFriendInMe => "minecraft:husbandry/allay_deliver_item_to_player".to_string(),
        Advancements::WhateverFloatsYourBoat => "minecraft:husbandry/ride_a_boat_with_a_goat".to_string(),
        Advancements::BestFriendsForever => "minecraft:husbandry/tame_an_animal".to_string(),
        Advancements::GlowAndBehold => "minecraft:husbandry/make_a_sign_glow".to_string(),
        Advancements::FishyBusisness => "minecraft:husbandry/fishy_business".to_string(),
        Advancements::TotalBeeLocation => "minecraft:husbandry/silk_touch_nest".to_string(),
        Advancements::BukkitBukkit => "minecraft:husbandry/tadpol_in_a_bucket".to_string(),
        Advancements::SmellsInteresting => "minecraft:husbandry/obtain_sniffer_egg".to_string(),
        Advancements::ASeedyPlace => "minecraft:husbandry/plant_seed".to_string(),
        Advancements::WaxOn => "minecraft:husbandry/wax_on".to_string(),
        Advancements::TwoByTow => "minecraft:husbandry/breed_all_animals".to_string(),
        Advancements::BirthdaySong => "minecraft:husbandry/allay_deliver_cake_to_note_block".to_string(),
        Advancements::ACompleteCatalogue => "minecraft:husbandry/complete_catalogue".to_string(),
        Advancements::TacticalFishing => "minecraft:husbandry/tactical_fishing".to_string(),
        Advancements::WhenTheSquadHopsIntoTown => "minecraft:husbandry/leash_all_frog_variants".to_string(),
        Advancements::LittleSniffs => "minecraft:husbandry/feed_snifflet".to_string(),
        Advancements::ABalancedDiet => "minecraft:husbandry/balanced_diet".to_string(),
        Advancements::SeriousDedication => "minecraft:husbandry/obtain_netherite_hoe".to_string(),
        Advancements::WaxOff => "minecraft:husbandry/wax_off".to_string(),
        Advancements::TheCutestPredator => "minecraft:husbandry/axolotl_in_a_bucket".to_string(),
        Advancements::WithOurPowersCombined => "minecraft:husbandry/froglights".to_string(),
        Advancements::PlantingThePast => "minecraft:husbandry/plant_any_sniffer_seed".to_string(),
        Advancements::TheHealingPowerOfFriendship => "minecraft:husbandry/kill_axolotl_target".to_string(),
        
        }
    }
}