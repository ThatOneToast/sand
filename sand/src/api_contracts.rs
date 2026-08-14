//! Generated/forwarded contracts for facade APIs whose definitions live in
//! implementation or proc-macro crates.
//!
//! Stable procedural macros cannot attach an attribute to a `pub use`, so the
//! facade owns these canonical identities and aliases in one auditable table.
//! Signatures are written once here and checked by focused facade tests.

use sand_api_contract::{ApiKind, ApiRegistration};

macro_rules! register {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        module: $module:literal,
        kind: $kind:ident,
        signature: $signature:literal,
        summary: $summary:literal,
        context: $context:literal,
        minecraft: $minecraft:literal,
        use_when: [$($use_when:literal),+ $(,)?],
        avoid_when: [$($avoid_when:literal),+ $(,)?],
        params: [$($param:literal => $param_doc:literal),* $(,)?],
        returns: $returns:expr,
        example: $example:literal $(,
        availability: [$($availability:literal),* $(,)?])?
    ) => {
        sand_api_contract::inventory::submit! {
            ApiRegistration {
                canonical_path: $path,
                aliases: &[$($alias),*],
                canonical_module: $module,
                kind: ApiKind::$kind,
                signature: $signature,
                summary: $summary,
                context: $context,
                minecraft: $minecraft,
                use_when: &[$($use_when),+],
                avoid_when: &[$($avoid_when),+],
                parameters: &[$(StaticApiParameter { name: $param, description: $param_doc }),*],
                returns: $returns,
                example: $example,
                availability: &[$($($availability),*)?],
            }
        }
    };
}

// Built-in event markers are a semantic family: each one is a stateless type
// consumed by `#[on_event]`, while its trigger/lifecycle meaning is declared
// here once for both the installed catalog and build-time enforcement. Keeping
// this provider beside the facade avoids duplicating an `#[api]` block beside
// every marker implementation in sand-core.
macro_rules! register_event_marker {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        summary: $summary:literal,
        minecraft: $minecraft:literal,
        example: $example:literal
    ) => {
        register! {
            path: $path,
            aliases: [$($alias),*],
            module: "sand::events",
            kind: Struct,
            signature: "pub struct event marker",
            summary: $summary,
            context: "This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
            minecraft: $minecraft,
            use_when: ["Registering a handler for this specific Minecraft or Sand runtime occurrence"],
            avoid_when: ["Representing mutable event data; read typed handler context or declared participants instead"],
            params: [],
            returns: None,
            example: $example
        }
    };
}

register! {
    path: "sand::prelude",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod prelude",
    summary: "Collects Sand's ordinary datapack-authoring vocabulary in one import.",
    context: "The prelude aliases canonical APIs from focused topic modules and is the stable starting point for author code.",
    minecraft: "Importing the prelude has no runtime effect; the imported builders and macros generate commands and datapack resources when used.",
    use_when: ["Starting a Sand datapack module", "Following Sand examples and book code"],
    avoid_when: ["A narrow import is needed to prevent a Rust name collision"],
    params: [],
    returns: None,
    example: "use sand::prelude::*;"
}

register! {
    path: "sand::vfx",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod vfx",
    summary: "Builds reusable particle, sound, and explicit raw-command effects.",
    context: "VFX keeps a small sequence of presentation commands reusable across functions instead of repeating command strings at every call site.",
    minecraft: "Renders particle and playsound commands, optionally wrapped at a selector or position.",
    use_when: ["Reusing a named presentation sequence", "Keeping particle and sound ordering deterministic"],
    avoid_when: ["Modeling game rules, state changes, or a datapack resource"],
    params: [],
    returns: None,
    example: "let effect = sand::vfx::Vfx::new(\"level_up\");"
}

register! {
    path: "sand::advanced",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod advanced",
    summary: "Provides the narrow custom-export hook outside ordinary datapack authoring.",
    context: "The advanced module deliberately exposes one version-aware JSON export operation without turning compiler wiring into a supported API.",
    minecraft: "The hook validates resources against the configured Minecraft release before emitting their datapack JSON.",
    use_when: ["Embedding Sand export in a custom build integration"],
    avoid_when: ["Using the normal sand build workflow or typed datapack builders"],
    params: [],
    returns: None,
    example: "let json = sand::advanced::try_export_components_json(\"example\", \"26.2\")?;"
}

register! {
    path: "sand::ResourceLocation",
    aliases: ["sand::prelude::ResourceLocation"],
    module: "sand",
    kind: Struct,
    signature: "pub struct ResourceLocation",
    summary: "Stores a validated Minecraft namespace:path identifier.",
    context: "Resource locations are the common identity representation behind Sand's typed resource IDs and command targets, so validation happens before an identifier reaches generated output.",
    minecraft: "Serializes as Minecraft's namespace:path resource-location syntax.",
    use_when: ["Naming a datapack resource", "Passing a validated identifier to an API that accepts multiple resource kinds"],
    avoid_when: ["A resource-kind-specific ID such as FunctionId or ItemId is available", "Keeping an unchecked user-provided identifier"],
    params: [],
    returns: None,
    example: "let id = sand::ResourceLocation::new(\"demo\", \"functions/start\")?;"
}

register! {
    path: "sand::command",
    aliases: ["sand::cmd", "sand::prelude::cmd"],
    module: "sand",
    kind: Module,
    signature: "pub mod command",
    summary: "Provides Sand's typed Minecraft command builders.",
    context: "The command module exposes builders for selectors, execute chains, scores, data, particles, sounds, and other vanilla commands while keeping rendering and validation consistent.",
    minecraft: "Renders the vanilla command text collected into generated .mcfunction files.",
    use_when: ["Building a Minecraft command from typed inputs", "Using a command family not imported directly by the prelude"],
    avoid_when: ["Defining a reusable datapack resource builder such as a predicate or recipe"],
    params: [],
    returns: None,
    example: "let command = sand::command::say(\"Ready\");"
}

register! {
    path: "sand::event",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod event",
    summary: "Defines advancement-backed event triggers and typed handler context.",
    context: "This module supplies the event model used by #[on_event] handlers, including custom advancement triggers and the player context passed to an event function.",
    minecraft: "Generates advancement trigger resources and the functions that dispatch their handlers.",
    use_when: ["Declaring or handling a typed Minecraft event", "Building a custom advancement-backed trigger"],
    avoid_when: ["Composing tick and dependency dispatch graphs directly; use sand::events"],
    params: [],
    returns: None,
    example: "use sand::event::Event;"
}

// Built-in advancement-backed event markers. Their aliases in `event::vanilla`
// are short discovery names; `events::*Event` remains the canonical catalog.
register_event_marker! { path: "sand::events::OnJoinEvent", aliases: ["sand::event::vanilla::OnJoin"], summary: "Marks Sand's load-or-new-player join observation.", minecraft: "Sand uses persistent per-player state, so it fires after load for present players and for newly observed players; reconnecting within the same persisted session is not a fresh login.", example: "#[sand::on_event]\nfn joined(event: sand::event::Event<sand::events::OnJoinEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::FirstJoinEvent", aliases: ["sand::event::vanilla::FirstJoin"], summary: "Marks the first observed join for a player.", minecraft: "Uses Sand's persisted first-join state and does not re-arm for that player.", example: "#[sand::on_event]\nfn first_join(event: sand::event::Event<sand::events::FirstJoinEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::OnDeathEvent", aliases: ["sand::event::vanilla::OnDeath"], summary: "Marks a player's death.", minecraft: "Backed by the vanilla death advancement trigger and dispatched as that player's reward function.", example: "#[sand::on_event]\nfn died(event: sand::event::Event<sand::events::OnDeathEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::OnRespawnEvent", aliases: ["sand::event::vanilla::OnRespawn"], summary: "Marks the first Sand observation after a player respawns.", minecraft: "Sand detects post-death player activity rather than receiving a standalone vanilla respawn trigger.", example: "#[sand::on_event]\nfn respawned(event: sand::event::Event<sand::events::OnRespawnEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::EntityKillEvent", aliases: ["sand::event::vanilla::EntityKill"], summary: "Marks a player killing an entity.", minecraft: "Uses minecraft:player_killed_entity; Sand can capture the player's main-hand weapon at trigger time.", example: "#[sand::on_event]\nfn kill(event: sand::event::Event<sand::events::EntityKillEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerKillEvent", aliases: ["sand::event::vanilla::PlayerKill"], summary: "Marks an entity killing a player.", minecraft: "Uses minecraft:entity_killed_player and exposes only evidence-backed correlated participants.", example: "#[sand::on_event]\nfn killed(event: sand::event::Event<sand::events::PlayerKillEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerDamageEntityEvent", aliases: ["sand::event::vanilla::PlayerDamagesEntity"], summary: "Marks a player damaging another entity.", minecraft: "Uses minecraft:player_hurt_entity and can capture the triggering player's weapon.", example: "#[sand::on_event]\nfn hit(event: sand::event::DamageEvent<sand::events::PlayerDamageEntityEvent>) { event.reflect_damage(); }" }
register_event_marker! { path: "sand::events::EntityDamagePlayerEvent", aliases: ["sand::event::vanilla::EntityDamagesPlayer"], summary: "Marks an entity damaging a player.", minecraft: "Uses minecraft:entity_hurt_player with a bounded correlated-attacker observation when available.", example: "#[sand::on_event]\nfn hurt(event: sand::event::DamageEvent<sand::events::EntityDamagePlayerEvent>) { event.reflect_damage(); }" }
register_event_marker! { path: "sand::events::ShotCrossbowEvent", aliases: ["sand::event::vanilla::CrossbowShot"], summary: "Marks a player shooting a crossbow.", minecraft: "Uses minecraft:shot_crossbow and re-arms its generated advancement after dispatch.", example: "#[sand::on_event]\nfn shot(event: sand::event::Event<sand::events::ShotCrossbowEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ChanneledLightningEvent", aliases: [], summary: "Marks a player channeling lightning through a trident action.", minecraft: "Uses minecraft:channeled_lightning advancement criteria.", example: "#[sand::on_event]\nfn lightning(event: sand::event::Event<sand::events::ChanneledLightningEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemConsumeEvent", aliases: ["sand::event::vanilla::AnyItemConsumed"], summary: "Marks a player consuming an item.", minecraft: "Uses minecraft:consume_item and fires for the player that consumed the stack.", example: "#[sand::on_event]\nfn consumed(event: sand::event::Event<sand::events::ItemConsumeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemCraftEvent", aliases: ["sand::event::vanilla::AnyItemCrafted"], summary: "Marks a player crafting an item.", minecraft: "Uses minecraft:recipe_crafted with Sand's broad built-in criterion.", example: "#[sand::on_event]\nfn crafted(event: sand::event::Event<sand::events::ItemCraftEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemEnchantEvent", aliases: ["sand::event::vanilla::AnyItemEnchanted"], summary: "Marks a player enchanting an item.", minecraft: "Uses minecraft:enchanted_item with no extra built-in item filter.", example: "#[sand::on_event]\nfn enchanted(event: sand::event::Event<sand::events::ItemEnchantEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BucketFillEvent", aliases: [], summary: "Marks a player filling a bucket.", minecraft: "Uses minecraft:filled_bucket advancement criteria.", example: "#[sand::on_event]\nfn filled(event: sand::event::Event<sand::events::BucketFillEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BucketEmptyEvent", aliases: [], summary: "Marks a player emptying a bucket.", minecraft: "Uses minecraft:filled_bucket's complementary empty-bucket criterion as emitted by Sand's trigger model.", example: "#[sand::on_event]\nfn emptied(event: sand::event::Event<sand::events::BucketEmptyEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::FishingEvent", aliases: [], summary: "Marks a player hooking something with a fishing rod.", minecraft: "Uses minecraft:fishing_rod_hooked, including all vanilla hooked-item/entity outcomes.", example: "#[sand::on_event]\nfn fish(event: sand::event::Event<sand::events::FishingEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemPickedUpEvent", aliases: [], summary: "Marks a player picking up an item.", minecraft: "Uses minecraft:inventory_changed detection for the built-in item-pickup criterion.", example: "#[sand::on_event]\nfn pickup(event: sand::event::Event<sand::events::ItemPickedUpEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ItemDurabilityChangeEvent", aliases: [], summary: "Marks a durability change on a player's item.", minecraft: "Uses minecraft:item_durability_changed advancement criteria.", example: "#[sand::on_event]\nfn durability(event: sand::event::Event<sand::events::ItemDurabilityChangeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BrewPotionEvent", aliases: ["sand::event::vanilla::PotionBrewed"], summary: "Marks a player brewing a potion.", minecraft: "Uses minecraft:brewed_potion without narrowing to one potion recipe.", example: "#[sand::on_event]\nfn brewed(event: sand::event::Event<sand::events::BrewPotionEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::TotemActivateEvent", aliases: [], summary: "Marks a player activating a totem of undying.", minecraft: "Uses minecraft:used_totem; advancement reset permits later totem activations.", example: "#[sand::on_event]\nfn saved(event: sand::event::Event<sand::events::TotemActivateEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::RecipeUnlockEvent", aliases: [], summary: "Marks a player unlocking a recipe.", minecraft: "Uses Sand's recipe-unlock advancement trigger registration.", example: "#[sand::on_event]\nfn recipe(event: sand::event::Event<sand::events::RecipeUnlockEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BlockPlaceEvent", aliases: ["sand::event::vanilla::AnyBlockPlaced"], summary: "Marks a player placing a block.", minecraft: "Uses minecraft:placed_block with no built-in block or item filter.", example: "#[sand::on_event]\nfn placed(event: sand::event::Event<sand::events::BlockPlaceEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::EnterBlockEvent", aliases: [], summary: "Marks a player entering a matching block volume.", minecraft: "Uses minecraft:enter_block without a built-in block-state filter.", example: "#[sand::on_event]\nfn entered(event: sand::event::Event<sand::events::EnterBlockEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::SlideDownBlockEvent", aliases: [], summary: "Marks a player sliding down a block.", minecraft: "Uses minecraft:slide_down_block advancement criteria.", example: "#[sand::on_event]\nfn slid(event: sand::event::Event<sand::events::SlideDownBlockEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::TargetHitEvent", aliases: [], summary: "Marks a player hitting a target block.", minecraft: "Uses minecraft:target_hit advancement criteria.", example: "#[sand::on_event]\nfn target(event: sand::event::Event<sand::events::TargetHitEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BeeNestDestroyedEvent", aliases: [], summary: "Marks a player destroying a bee nest or hive.", minecraft: "Uses minecraft:bee_nest_destroyed criteria.", example: "#[sand::on_event]\nfn bees(event: sand::event::Event<sand::events::BeeNestDestroyedEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ChangeDimensionEvent", aliases: ["sand::event::vanilla::DimensionChanged"], summary: "Marks a player changing dimension.", minecraft: "Uses minecraft:changed_dimension with Sand's broad built-in criterion.", example: "#[sand::on_event]\nfn dimension(event: sand::event::Event<sand::events::ChangeDimensionEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerSleepEvent", aliases: [], summary: "Marks a player sleeping in a bed.", minecraft: "Uses minecraft:slept_in_bed advancement criteria.", example: "#[sand::on_event]\nfn slept(event: sand::event::Event<sand::events::PlayerSleepEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::FallFromHeightEvent", aliases: [], summary: "Marks a player satisfying the vanilla fall-from-height trigger.", minecraft: "Uses minecraft:fall_from_height with Sand's unfiltered built-in threshold.", example: "#[sand::on_event]\nfn fell(event: sand::event::Event<sand::events::FallFromHeightEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerLevelUpEvent", aliases: ["sand::event::vanilla::PlayerLevelsUp"], summary: "Marks an increase in a player's experience level.", minecraft: "Minecraft has no level-up advancement; Sand compares generated scoreboard snapshots in its tick lifecycle.", example: "#[sand::on_event]\nfn leveled(event: sand::event::Event<sand::events::PlayerLevelUpEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::EffectsChangedEvent", aliases: [], summary: "Marks a change to a player's active effects.", minecraft: "Uses minecraft:effects_changed with Sand's broad built-in predicate.", example: "#[sand::on_event]\nfn effects(event: sand::event::Event<sand::events::EffectsChangedEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::StartRidingEvent", aliases: [], summary: "Marks a player beginning to ride an entity.", minecraft: "Uses minecraft:started_riding advancement criteria.", example: "#[sand::on_event]\nfn ride(event: sand::event::Event<sand::events::StartRidingEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::UseEnderEyeEvent", aliases: [], summary: "Marks a player using an eye of ender.", minecraft: "Uses minecraft:used_ender_eye advancement criteria.", example: "#[sand::on_event]\nfn eye(event: sand::event::Event<sand::events::UseEnderEyeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::HeroOfTheVillageEvent", aliases: [], summary: "Marks a player gaining Hero of the Village.", minecraft: "Uses minecraft:hero_of_the_village advancement criteria.", example: "#[sand::on_event]\nfn hero(event: sand::event::Event<sand::events::HeroOfTheVillageEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::LightningStrikeEvent", aliases: [], summary: "Marks a player observing a lightning strike.", minecraft: "Uses minecraft:lightning_strike advancement criteria.", example: "#[sand::on_event]\nfn strike(event: sand::event::Event<sand::events::LightningStrikeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::TameAnimalEvent", aliases: ["sand::event::vanilla::AnimalTamed"], summary: "Marks a player taming an animal.", minecraft: "Uses minecraft:tame_animal advancement criteria.", example: "#[sand::on_event]\nfn tame(event: sand::event::Event<sand::events::TameAnimalEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::BreedAnimalsEvent", aliases: ["sand::event::vanilla::AnimalsBreed"], summary: "Marks a player breeding animals.", minecraft: "Uses minecraft:bred_animals advancement criteria.", example: "#[sand::on_event]\nfn breed(event: sand::event::Event<sand::events::BreedAnimalsEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::SummonEntityEvent", aliases: ["sand::event::vanilla::EntitySummoned"], summary: "Marks a player summoning an entity.", minecraft: "Uses minecraft:summoned_entity advancement criteria.", example: "#[sand::on_event]\nfn summon(event: sand::event::Event<sand::events::SummonEntityEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::InteractWithEntityEvent", aliases: [], summary: "Marks a player interacting with an entity.", minecraft: "Uses minecraft:player_interacted_with_entity criteria.", example: "#[sand::on_event]\nfn interact(event: sand::event::Event<sand::events::InteractWithEntityEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::VillagerTradeEvent", aliases: [], summary: "Marks a player completing a villager trade.", minecraft: "Uses minecraft:villager_trade advancement criteria.", example: "#[sand::on_event]\nfn trade(event: sand::event::Event<sand::events::VillagerTradeEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ConstructBeaconEvent", aliases: [], summary: "Marks a player constructing a beacon.", minecraft: "Uses minecraft:construct_beacon advancement criteria.", example: "#[sand::on_event]\nfn beacon(event: sand::event::Event<sand::events::ConstructBeaconEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::CureZombieVillagerEvent", aliases: [], summary: "Marks a player curing a zombie villager.", minecraft: "Uses minecraft:cured_zombie_villager advancement criteria.", example: "#[sand::on_event]\nfn cure(event: sand::event::Event<sand::events::CureZombieVillagerEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::LootContainerOpenEvent", aliases: [], summary: "Marks a player opening a loot container.", minecraft: "Uses minecraft:player_generates_container_loot criteria.", example: "#[sand::on_event]\nfn loot(event: sand::event::Event<sand::events::LootContainerOpenEvent>) { sand::command::say(event.player()); }" }

// Tick-polled and tracked built-ins. These are bare `SandEvent` markers: the
// handler receives the marker itself, and Sand owns the per-player detector.
register_event_marker! { path: "sand::events::PlayerStartSneakingEvent", aliases: ["sand::event::vanilla::PlayerStartsSneaking"], summary: "Marks the transition into sneaking.", minecraft: "Sand compares a tracked player-sneaking predicate across ticks and fires only on false-to-true transitions.", example: "#[sand::on_event]\nfn start_sneak(_: sand::events::PlayerStartSneakingEvent) { sand::command::say(\"Sneak\"); }" }
register_event_marker! { path: "sand::events::PlayerStopSneakingEvent", aliases: ["sand::event::vanilla::PlayerStopsSneaking"], summary: "Marks the transition out of sneaking.", minecraft: "Sand's tracked sneaking predicate fires only on true-to-false transitions.", example: "#[sand::on_event]\nfn stop_sneak(_: sand::events::PlayerStopSneakingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerSneakEvent", aliases: ["sand::event::vanilla::PlayerSneaking"], summary: "Marks each tick a player is sneaking.", minecraft: "Sand evaluates its player-sneaking predicate for each online player every tick.", example: "#[sand::on_event]\nfn sneaking(_: sand::events::PlayerSneakEvent) { sand::command::say(\"Sneaking\"); }" }
register_event_marker! { path: "sand::events::PlayerSprintEvent", aliases: ["sand::event::vanilla::PlayerSprinting"], summary: "Marks each tick a player is sprinting.", minecraft: "Sand evaluates its tracked sprinting predicate for each online player every tick.", example: "#[sand::on_event]\nfn sprinting(_: sand::events::PlayerSprintEvent) { sand::command::say(\"Sprint\"); }" }
register_event_marker! { path: "sand::events::PlayerStartSprintingEvent", aliases: [], summary: "Marks the transition into sprinting.", minecraft: "Sand emits a false-to-true transition from its per-player sprinting tracker.", example: "#[sand::on_event]\nfn start_sprint(_: sand::events::PlayerStartSprintingEvent) { sand::command::say(\"Sprint\"); }" }
register_event_marker! { path: "sand::events::PlayerStopSprintingEvent", aliases: [], summary: "Marks the transition out of sprinting.", minecraft: "Sand emits a true-to-false transition from its per-player sprinting tracker.", example: "#[sand::on_event]\nfn stop_sprint(_: sand::events::PlayerStopSprintingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerSwimmingEvent", aliases: ["sand::event::vanilla::PlayerSwimming"], summary: "Marks each tick a player is swimming.", minecraft: "Sand evaluates the swimming entity predicate as each online player.", example: "#[sand::on_event]\nfn swimming(_: sand::events::PlayerSwimmingEvent) { sand::command::say(\"Swim\"); }" }
register_event_marker! { path: "sand::events::PlayerStartSwimmingEvent", aliases: [], summary: "Marks the transition into swimming.", minecraft: "Sand's tracked swimming predicate fires only on false-to-true transitions.", example: "#[sand::on_event]\nfn start_swim(_: sand::events::PlayerStartSwimmingEvent) { sand::command::say(\"Swim\"); }" }
register_event_marker! { path: "sand::events::PlayerStopSwimmingEvent", aliases: [], summary: "Marks the transition out of swimming.", minecraft: "Sand's tracked swimming predicate fires only on true-to-false transitions.", example: "#[sand::on_event]\nfn stop_swim(_: sand::events::PlayerStopSwimmingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerFlyingEvent", aliases: [], summary: "Marks each tick a player has flying enabled.", minecraft: "Sand evaluates the player's abilities.flying NBT predicate each tick; it is a player-only detector.", example: "#[sand::on_event]\nfn flying(_: sand::events::PlayerFlyingEvent) { sand::command::say(\"Fly\"); }" }
register_event_marker! { path: "sand::events::PlayerStartFlyingEvent", aliases: [], summary: "Marks the transition into flying.", minecraft: "Sand tracks the player flying predicate and fires on false-to-true transitions.", example: "#[sand::on_event]\nfn start_flying(_: sand::events::PlayerStartFlyingEvent) { sand::command::say(\"Fly\"); }" }
register_event_marker! { path: "sand::events::PlayerStopFlyingEvent", aliases: [], summary: "Marks the transition out of flying.", minecraft: "Sand tracks the player flying predicate and fires on true-to-false transitions.", example: "#[sand::on_event]\nfn stop_flying(_: sand::events::PlayerStopFlyingEvent) { sand::command::say(\"Stop\"); }" }
register_event_marker! { path: "sand::events::PlayerOnFireEvent", aliases: ["sand::event::vanilla::PlayerOnFire"], summary: "Marks each tick a player is on fire.", minecraft: "Sand evaluates its player-on-fire predicate for each online player.", example: "#[sand::on_event]\nfn burning(_: sand::events::PlayerOnFireEvent) { sand::command::say(\"Fire\"); }" }
register_event_marker! { path: "sand::events::PlayerCaughtFireEvent", aliases: [], summary: "Marks the transition into being on fire.", minecraft: "Sand's tracked fire predicate fires only on false-to-true transitions.", example: "#[sand::on_event]\nfn caught_fire(_: sand::events::PlayerCaughtFireEvent) { sand::command::say(\"Fire\"); }" }
register_event_marker! { path: "sand::events::PlayerExtinguishedEvent", aliases: [], summary: "Marks the transition out of being on fire.", minecraft: "Sand's tracked fire predicate fires only on true-to-false transitions.", example: "#[sand::on_event]\nfn extinguished(_: sand::events::PlayerExtinguishedEvent) { sand::command::say(\"Safe\"); }" }
register_event_marker! { path: "sand::events::PlayerInCreativeEvent", aliases: [], summary: "Marks each tick a player is in creative mode.", minecraft: "Sand evaluates an entity gamemode=creative condition per online player.", example: "#[sand::on_event]\nfn creative(_: sand::events::PlayerInCreativeEvent) { sand::command::say(\"Creative\"); }" }
register_event_marker! { path: "sand::events::PlayerInAdventureEvent", aliases: [], summary: "Marks each tick a player is in adventure mode.", minecraft: "Sand evaluates an entity gamemode=adventure condition per online player.", example: "#[sand::on_event]\nfn adventure(_: sand::events::PlayerInAdventureEvent) { sand::command::say(\"Adventure\"); }" }
register_event_marker! { path: "sand::events::PlayerInSpectatorEvent", aliases: [], summary: "Marks each tick a player is in spectator mode.", minecraft: "Sand evaluates an entity gamemode=spectator condition per online player.", example: "#[sand::on_event]\nfn spectator(_: sand::events::PlayerInSpectatorEvent) { sand::command::say(\"Spectator\"); }" }
register_event_marker! { path: "sand::events::PlayerHealthChangedEvent", aliases: [], summary: "Marks a change in a player's health score.", minecraft: "Sand tracks health through a generated scoreboard baseline and fires on any change.", example: "#[sand::on_event]\nfn health(_: sand::events::PlayerHealthChangedEvent) { sand::command::say(\"Health\"); }" }
register_event_marker! { path: "sand::events::PlayerHealthLostEvent", aliases: [], summary: "Marks a decrease in a player's health score.", minecraft: "Sand compares generated current and previous health score baselines.", example: "#[sand::on_event]\nfn lost(_: sand::events::PlayerHealthLostEvent) { sand::command::say(\"Hurt\"); }" }
register_event_marker! { path: "sand::events::PlayerHealthGainedEvent", aliases: [], summary: "Marks an increase in a player's health score.", minecraft: "Sand compares generated current and previous health score baselines.", example: "#[sand::on_event]\nfn gained(_: sand::events::PlayerHealthGainedEvent) { sand::command::say(\"Healed\"); }" }
register_event_marker! { path: "sand::events::ArmorEquipEvent", aliases: [], summary: "Marks a player equipping an item in a selected equipment slot.", minecraft: "Sand tracks equipment-slot state each tick; #[on_event] requires a slot filter and may further filter item/custom data.", example: "#[sand::on_event(slot = Feet)]\nfn equip(event: sand::event::Event<sand::events::ArmorEquipEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::ArmorUnequipEvent", aliases: [], summary: "Marks a player removing an item from a selected equipment slot.", minecraft: "Sand tracks equipment-slot state each tick; #[on_event] requires a slot filter and may further filter item/custom data.", example: "#[sand::on_event(slot = Feet)]\nfn unequip(event: sand::event::Event<sand::events::ArmorUnequipEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::HoldingItemEvent", aliases: [], summary: "Marks each tick a player holds a selected item.", minecraft: "Sand renders execute-if-items checks against the requested mainhand or offhand slot.", example: "#[sand::on_event(item = \"minecraft:shield\", slot = Offhand)]\nfn shield(event: sand::event::Event<sand::events::HoldingItemEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::CurrentlyWearingEvent", aliases: [], summary: "Marks each tick a player wears a selected armor item.", minecraft: "Sand renders execute-if-items checks against the requested armor slot and item filter.", example: "#[sand::on_event(slot = Head, item = \"minecraft:diamond_helmet\")]\nfn helmet(event: sand::event::Event<sand::events::CurrentlyWearingEvent>) { sand::command::say(event.player()); }" }
register_event_marker! { path: "sand::events::PlayerLowHealthEvent", aliases: [], summary: "Marks crossing down to a configured player health threshold.", minecraft: "Sand tracks vanilla health through a shared scoreboard baseline; one threshold value is allowed per exported pack.", example: "#[sand::on_event]\nfn low(_: sand::events::PlayerLowHealthEvent<6>) { sand::command::say(\"Low health\"); }" }
register_event_marker! { path: "sand::events::PlayerRecoveredHealthEvent", aliases: [], summary: "Marks crossing back above a configured low-health threshold.", minecraft: "Shares PlayerLowHealthEvent's generated health tracker, so both types must use the same half-heart threshold.", example: "#[sand::on_event]\nfn recovered(_: sand::events::PlayerRecoveredHealthEvent<6>) { sand::command::say(\"Recovered\"); }" }
register_event_marker! { path: "sand::events::EffectStarted", aliases: [], summary: "Marks a player gaining a selected generated status-effect marker.", minecraft: "Sand emits an entity-properties effect predicate and a false-to-true transition tracker for the selected effect.", example: "#[sand::on_event]\nfn speed(_: sand::events::EffectStarted<sand::events::Speed>) { sand::command::say(\"Speed\"); }" }
register_event_marker! { path: "sand::events::EffectStopped", aliases: [], summary: "Marks a player losing a selected generated status-effect marker.", minecraft: "Sand emits a true-to-false transition from the same generated effect tracker used by EffectStarted.", example: "#[sand::on_event]\nfn speed_end(_: sand::events::EffectStopped<sand::events::Speed>) { sand::command::say(\"Speed ended\"); }" }

register! {
    path: "sand::item",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod item",
    summary: "Builds custom items and typed item-component behavior.",
    context: "The item module groups the custom-item builder, item predicates, and location helpers used by #[custom_item] functions and live inventory checks.",
    minecraft: "Generates item component JSON and renders vanilla item commands or item predicates where used.",
    use_when: ["Defining or matching a custom item", "Working with an item stack outside a resource component"],
    avoid_when: ["Addressing only a live slot; use sand::inventory for location construction"],
    params: [],
    returns: None,
    example: "use sand::item::CustomItem;"
}

register! {
    path: "sand::item::location",
    aliases: [],
    module: "sand::item",
    kind: Module,
    signature: "pub mod location",
    summary: "Provides validated live item and inventory addressing.",
    context: "Location types describe where an item currently resides without confusing that mutable location with captured event-time data.",
    minecraft: "Builds the entity and block inventory targets used by item, data, and execute-if-items commands.",
    use_when: ["Constructing a live equipment, inventory, container, or item-entity location"],
    avoid_when: ["Keeping event-time item evidence after a handler changes the source"],
    params: [],
    returns: None,
    example: "let slot = sand::item::ItemLocation::PlayerMainHand;"
}

register! {
    path: "sand::item::snapshot",
    aliases: [],
    module: "sand::item",
    kind: Module,
    signature: "pub mod snapshot",
    summary: "Provides immutable, short-lived event-time item capture handles.",
    context: "Snapshots preserve the source item before handler logic mutates inventory and communicate their capture reliability explicitly.",
    minecraft: "Uses guarded data copies into deterministic command-storage paths during one synchronous event dispatch.",
    use_when: ["A handler needs evidence of the item present at trigger time"],
    avoid_when: ["Representing a live inventory slot or durable player inventory state"],
    params: [],
    returns: None,
    example: "use sand::item::snapshot::ItemSnapshot;"
}

register! {
    path: "sand::state",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod state",
    summary: "Provides low-level typed scoreboard, timer, and storage state primitives.",
    context: "State is the implementation vocabulary beneath derived schemas and gives authors explicit control over score-backed and storage-backed values when a reusable schema is not appropriate.",
    minecraft: "Renders scoreboard and data-storage commands and describes their persistent datapack state locations.",
    use_when: ["Building typed score, timer, flag, or storage operations", "Implementing a reusable state abstraction"],
    avoid_when: ["A #[derive(State)] schema expresses the application state directly"],
    params: [],
    returns: None,
    example: "use sand::state::ScoreVar;"
}

register! {
    path: "sand::component",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod component",
    summary: "Builds datapack JSON components such as recipes, loot tables, tags, dialogs, and advancements.",
    context: "Component builders model vanilla data resources as typed Rust values that #[datapack_component] can register for export.",
    minecraft: "Serializes the selected component into its corresponding datapack JSON resource.",
    use_when: ["Constructing a datapack JSON resource", "Returning a component from a #[datapack_component] function"],
    avoid_when: ["Emitting one command in a function body"],
    params: [],
    returns: None,
    example: "use sand::component::Advancement;"
}

register! {
    path: "sand::systems",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod systems",
    summary: "Contains optional higher-level gameplay systems built on Sand primitives.",
    context: "Systems bundle reusable patterns such as damage tracking, cooldowns, player data, movement, inventory, and entity behavior behind explicit Cargo features.",
    minecraft: "Each enabled system emits the commands, scores, storage, and lifecycle functions required by its gameplay pattern.",
    use_when: ["Opting into a supported higher-level gameplay subsystem", "Reusing its typed configuration and runtime helpers"],
    avoid_when: ["A small feature can be expressed more clearly with the underlying state and command APIs"],
    params: [],
    returns: None,
    example: "use sand::systems::damage;"
}

register! {
    path: "sand::resourcepack",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod resourcepack",
    summary: "Provides optional typed resource-pack authoring APIs.",
    context: "The feature-gated module accompanies HUD and texture macros with types that register generated assets alongside a Sand datapack project.",
    minecraft: "Writes resource-pack assets such as GUI textures and HUD definitions rather than datapack data resources.",
    use_when: ["Authoring a resource pack together with a Sand datapack", "Using the resourcepack feature's HUD or texture facilities"],
    avoid_when: ["Building a datapack-only project without client asset output"],
    params: [],
    returns: None,
    example: "use sand::resourcepack;",
    availability: ["Cargo feature: resourcepack"]
}

register! {
    path: "sand::all",
    aliases: ["sand::prelude::all"],
    module: "sand",
    kind: Macro,
    signature: "all![condition, ...]",
    summary: "Combines typed conditions that must all hold.",
    context: "The macro is concise syntax for Condition::all and preserves Sand's typed condition model when several guards must be true together.",
    minecraft: "Lowers to the execute-condition clauses needed for every supplied condition.",
    use_when: ["Guarding a command or branch with multiple required conditions"],
    avoid_when: ["Any alternative may pass; use any! instead"],
    params: [],
    returns: None,
    example: "let ready = sand::all![has_key, is_near_door];"
}

register! {
    path: "sand::any",
    aliases: ["sand::prelude::any"],
    module: "sand",
    kind: Macro,
    signature: "any![condition, ...]",
    summary: "Combines typed conditions where any alternative may hold.",
    context: "The macro is concise syntax for Condition::any and makes alternative guard branches explicit in the typed condition model.",
    minecraft: "Lowers each alternative into the execute-condition plan required to preserve OR behavior.",
    use_when: ["A command or branch should run when one of several typed conditions is true"],
    avoid_when: ["Every condition is required; use all! instead"],
    params: [],
    returns: None,
    example: "let visible = sand::any![is_day, has_night_vision];"
}

register! {
    path: "sand::mcfunction",
    aliases: ["sand::prelude::mcfunction"],
    module: "sand",
    kind: Macro,
    signature: "mcfunction![command; ...]",
    summary: "Collects command expressions into one Minecraft function body.",
    context: "The macro accepts displayable command builders and literal command strings in source order, making a short function body explicit without manual Vec assembly.",
    minecraft: "Produces the ordered command lines exported into a .mcfunction file by a surrounding Sand function or registration.",
    use_when: ["Passing several commands to a Sand builder that expects a command list", "Writing a compact function body from typed command expressions"],
    avoid_when: ["A single typed command builder is sufficient", "Using raw strings where a typed command builder exists"],
    params: [],
    returns: None,
    example: "let commands = sand::mcfunction![sand::command::say(\"Ready\");];"
}

register! {
    path: "sand::api",
    aliases: ["sand::prelude::api"],
    module: "sand",
    kind: Macro,
    signature: "#[api(...)]",
    summary: "Attaches Sand's authoritative API contract to a supported declaration.",
    context: "The attribute provides the concise semantic description that drives Rustdoc, installed metadata, CLI discovery, and ordinary-build contract enforcement.",
    minecraft: "The attribute emits no Minecraft data by itself; it records how the annotated API participates in Minecraft authoring.",
    use_when: ["Contributing a new supported Sand API", "Documenting a deliberately public author-facing declaration"],
    avoid_when: ["Annotating private implementation details or using documentation as an item-level exemption"],
    params: [],
    returns: None,
    example: "#[sand::api(path = \"sand::feature\", summary = \"...\", context = \"...\", minecraft = \"...\", use_when = [\"...\"], avoid_when = [\"...\"], example = \"...\")]"
}

register! {
    path: "sand::function",
    aliases: ["sand::prelude::function"],
    module: "sand",
    kind: Macro,
    signature: "#[function]",
    summary: "Registers a Rust function as an exported Minecraft function.",
    context: "The attribute turns a command-producing Rust body into a named Sand function that other generated functions and Minecraft load/tag wiring can call.",
    minecraft: "Exports the collected command lines to data/<namespace>/function/<name>.mcfunction.",
    use_when: ["Writing reusable datapack command logic", "Defining a function invoked by another Sand API or a generated tag"],
    avoid_when: ["Returning a JSON component resource; use #[datapack_component]"],
    params: [],
    returns: None,
    example: "#[sand::function]\nfn welcome() { sand::command::say(\"Welcome\"); }"
}

register! {
    path: "sand::datapack_component",
    aliases: ["sand::prelude::datapack_component"],
    module: "sand",
    kind: Macro,
    signature: "#[datapack_component(...)]",
    summary: "Registers a component-producing function for datapack JSON export.",
    context: "The attribute connects a typed component value returned by a Rust function to Sand's component registry and optional lifecycle tags.",
    minecraft: "Exports the returned advancement, recipe, loot table, tag, dialog, or other component as its version-valid JSON resource.",
    use_when: ["Defining a reusable datapack JSON component", "Registering load, tick, or tag-oriented component output"],
    avoid_when: ["Writing a command body; use #[function]"],
    params: [],
    returns: None,
    example: "#[sand::datapack_component]\nfn reward() -> sand::component::Advancement { todo!() }"
}

register! {
    path: "sand::on_event",
    aliases: ["sand::prelude::on_event"],
    module: "sand",
    kind: Macro,
    signature: "#[on_event(...)]",
    summary: "Registers a Rust function as a typed Minecraft event handler.",
    context: "The attribute connects an advancement-backed or SandEvent marker to a command-producing handler and supplies its typed event context.",
    minecraft: "Generates the event detection resources and handler dispatch functions required by the selected event type.",
    use_when: ["Responding to a vanilla or custom Sand event", "Writing a handler that needs typed player or participant context"],
    avoid_when: ["Creating a JSON component without runtime handler logic; use #[datapack_component]"],
    params: [],
    returns: None,
    example: "#[sand::on_event]\nfn joined(event: sand::event::Event<OnJoinEvent>) { sand::command::say(event.player()); }"
}

register! {
    path: "sand::custom_item",
    aliases: ["sand::prelude::custom_item"],
    module: "sand",
    kind: Macro,
    signature: "#[custom_item(...)]",
    summary: "Derives a typed custom-item reference from a CustomItem factory function.",
    context: "The attribute inspects the declared custom-item definition and generates the stable helper type, matching predicate, and equipment checks used by author code.",
    minecraft: "Does not register a payload by itself; it emits the typed reference, item predicate, and equipment-check helpers that later render item commands or predicates.",
    use_when: ["Giving a custom item a reusable typed Rust handle", "Generating equipment checks tied to one item definition"],
    avoid_when: ["Constructing a one-off CustomItem value without generated helpers"],
    params: [],
    returns: None,
    example: "#[sand::custom_item]\nfn compass() -> sand::item::CustomItem { sand::item::CustomItem::new(\"minecraft:compass\").custom_data(\"demo_compass\") }"
}

register! {
    path: "sand::armor_event",
    aliases: ["sand::prelude::armor_event"],
    module: "sand",
    kind: Macro,
    signature: "#[armor_event(...)]",
    summary: "Registers an equip or unequip handler for a custom armor item.",
    context: "The attribute generates player inventory polling and transition bookkeeping needed to invoke a no-argument handler exactly when the selected armor or offhand item changes.",
    minecraft: "Emits a minecraft:tick function that checks @a players, records prior slot state, and invokes the handler on the requested transition.",
    use_when: ["Reacting when a player equips or removes an armor or offhand item"],
    avoid_when: ["Handling a general inventory change or a non-armor item event"],
    params: [],
    returns: None,
    example: "#[sand::armor_event(Equip, slot = Head, item = \"minecraft:diamond_helmet\")]\nfn helmet_equipped() { sand::command::say(\"Protected\"); }"
}

register! {
    path: "sand::schedule",
    aliases: ["sand::prelude::schedule"],
    module: "sand",
    kind: Macro,
    signature: "#[schedule(...)]",
    summary: "Registers a per-player countdown-driven scheduled function.",
    context: "The attribute creates a command-producing body plus generated start and stop entry points; Sand tracks each player's remaining ticks and phase with scoreboard state rather than wrapping Minecraft's /schedule command.",
    minecraft: "Emits tick-driven scoreboard countdown logic and <name>_start / <name>_stop functions that control each player's lifecycle.",
    use_when: ["Defining recurring or delayed datapack command logic"],
    avoid_when: ["Running commands immediately in the current function body"],
    params: [],
    returns: None,
    example: "#[sand::schedule(ticks = 60, every = 5)]\nfn refresh() { sand::command::say(\"Refreshing\"); }"
}

register! {
    path: "sand::entity_archetype",
    aliases: ["sand::prelude::entity_archetype"],
    module: "sand",
    kind: Macro,
    signature: "#[entity_archetype]",
    summary: "Registers a typed entity-archetype factory.",
    context: "The attribute links an EntityArchetype definition to Sand's lifecycle registry so adoption, state initialization, and derived values are evaluated consistently.",
    minecraft: "Generates the functions and periodic checks required to maintain the declared archetype for loaded entities.",
    use_when: ["Declaring reusable behavior and state for a Minecraft entity kind"],
    avoid_when: ["Issuing a one-time selector command without archetype lifecycle behavior"],
    params: [],
    returns: None,
    example: "#[sand::entity_archetype]\nfn zombie() -> sand::entity::EntityArchetype<ZombieKind, Mob> { todo!() }"
}

register! {
    path: "sand::State",
    aliases: ["sand::prelude::State"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(State)]",
    summary: "Derives a typed, versioned Sand state schema from a Rust struct.",
    context: "The derive converts field-level state metadata into concrete typed accessors and lifecycle information so application state does not need hand-written score or storage plumbing.",
    minecraft: "Allocates and maintains the scoreboard or storage-backed data locations declared by the schema.",
    use_when: ["Modeling persistent gameplay state as a Rust struct", "Sharing one validated state schema across functions and events"],
    avoid_when: ["A single transient command-local value needs no schema"],
    params: [],
    returns: None,
    example: "use sand::prelude::*;\n\n#[derive(State)]\n#[state(namespace = \"demo\", scope = player)]\nstruct PlayerState {\n    #[state(default = 0)]\n    score: EntityScore<i32>,\n}"
}

register! {
    path: "sand::EntityStateEnum",
    aliases: ["sand::prelude::EntityStateEnum"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(EntityStateEnum)]",
    summary: "Maps a fieldless Rust enum to a stable integer entity-state encoding.",
    context: "The derive gives a finite state machine an explicit scoreboard representation while preserving enum names at the Rust call site.",
    minecraft: "Stores the selected variant as the derived integer value in Sand's entity-state machinery.",
    use_when: ["Representing a finite entity-state field with named Rust variants"],
    avoid_when: ["The state has data-carrying variants or needs an open-ended numeric range"],
    params: [],
    returns: None,
    example: "#[derive(sand::EntityStateEnum)]\nenum Phase { Idle, Alert }"
}

register! {
    path: "sand::SandStorage",
    aliases: ["sand::prelude::SandStorage"],
    module: "sand",
    kind: Macro,
    signature: "#[derive(SandStorage)]",
    summary: "Derives typed NBT-storage accessors from a Rust struct.",
    context: "The derive connects named Rust fields to one declared command-storage root so callers use validated paths instead of repeated string literals.",
    minecraft: "Addresses the configured data storage and its NBT fields through typed data-command locations.",
    use_when: ["Defining a fixed storage schema shared by several functions"],
    avoid_when: ["Reading an arbitrary dynamic NBT path"],
    params: [],
    returns: None,
    example: "#[derive(sand::SandStorage)]\n#[sand(storage = \"demo:state\", root = \"player\")]\nstruct PlayerStorage { score: i32 }"
}

register! {
    path: "sand::run_fn",
    aliases: ["sand::prelude::run_fn"],
    module: "sand",
    kind: Macro,
    signature: "run_fn!(...)",
    summary: "Creates a generated Sand function for an inline command sequence.",
    context: "The macro gives a local sequence of command expressions a stable generated function identity so it can be scheduled, called, or registered without manually declaring a separate item.",
    minecraft: "Exports the inline sequence as a generated .mcfunction and references it from the surrounding Sand output.",
    use_when: ["A command sequence needs a function reference but does not justify a named Rust function"],
    avoid_when: ["The sequence is part of the current function body or should be reusable by name"],
    params: [],
    returns: None,
    example: "let delayed = sand::run_fn! { sand::command::say(\"Later\") };"
}

register! {
    path: "sand::hud_bar",
    aliases: [],
    module: "sand",
    kind: Macro,
    signature: "hud_bar!(...)",
    summary: "Declares a custom resource-pack HUD bar.",
    context: "The macro packages HUD frame, fill, and layout metadata into Sand's resource-pack registry so gameplay code can address the resulting handle.",
    minecraft: "Writes the GUI textures and resource-pack metadata used to render the configured HUD bar on clients.",
    use_when: ["Adding a reusable custom status bar to a Sand resource pack"],
    avoid_when: ["Displaying a one-off chat or actionbar message"],
    params: [],
    returns: None,
    example: "let health = sand::hud_bar!(/* resource-pack HUD fields */);",
    availability: ["Cargo feature: resourcepack"]
}

register! {
    path: "sand::hud_element",
    aliases: [],
    module: "sand",
    kind: Macro,
    signature: "hud_element!(...)",
    summary: "Declares a custom resource-pack HUD element.",
    context: "The macro registers a positioned UI element and its asset metadata as a typed handle that Sand's resource-pack runtime can update.",
    minecraft: "Writes client resource-pack assets and metadata for the configured HUD element.",
    use_when: ["Showing a reusable texture-backed HUD indicator"],
    avoid_when: ["Rendering structured text through tellraw or an ordinary title"],
    params: [],
    returns: None,
    example: "let indicator = sand::hud_element!(/* resource-pack HUD fields */);",
    availability: ["Cargo feature: resourcepack"]
}

register! {
    path: "sand::texture",
    aliases: [],
    module: "sand",
    kind: Macro,
    signature: "texture!(...)",
    summary: "Registers a texture asset for Sand resource-pack output.",
    context: "The macro associates a source texture and its destination identity with the resource-pack registry so HUD and GUI declarations can refer to it consistently.",
    minecraft: "Copies or emits the declared texture into the matching assets/<namespace>/textures resource-pack path.",
    use_when: ["Bundling a named texture used by Sand's resource-pack APIs"],
    avoid_when: ["Referencing a vanilla texture that does not need to be packaged"],
    params: [],
    returns: None,
    example: "let icon = sand::texture!(/* texture source and destination */);",
    availability: ["Cargo feature: resourcepack"]
}
