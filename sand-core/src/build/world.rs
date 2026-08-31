//! 🌍 World (datapack) — the primary world's seed, spawn, border, gamerules,
//! time, and weather.
//!
//! Everything built through [`World`] lowers into a generated
//! `data/<namespace>/function/__sand_world_init.mcfunction` invoked on
//! datapack load (`minecraft:load`), plus dimension resources under
//! `data/<namespace>/dimension/`. All of it travels with the exported
//! datapack and works identically in singleplayer, LAN, realms, or any
//! vanilla-compatible dedicated server — no mod or server config required.
//!
//! The one exception is [`WorldPreset`]: Minecraft only accepts a world
//! generation preset (superflat vs. default vs. large-biomes, …) *at world
//! creation*, which is a server/client bootstrap step with no datapack
//! representation. It is exposed here for API discoverability alongside the
//! rest of world configuration, but is 🖥️ **Server (host)**-only: `sand run`
//! reads it to decide how to create/reset its local dev world
//! (`server.properties` `level-type`/`generator-settings`); it has no effect
//! on the exported datapack and is not reproduced for anyone who drops the
//! datapack into their own existing world.

use sand_macros::api;
use std::collections::BTreeMap;

use sand_components::resource_location::ResourceLocation;

use super::dimension::Dimensions;

/// A fixed or random world seed.
///
/// 🌍 World (datapack) via [`sand run`]'s world bootstrap when creating a
/// fresh world for local testing; has no separate datapack representation
/// (a datapack cannot retroactively change a world's seed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::Seed",
    module = "sand::build",
    summary = "Seed selects a fixed or random world seed for sand run's local world bootstrap.",
    context = "Minecraft only accepts a seed at world creation; a datapack cannot retroactively change an existing world's seed.",
    minecraft = "Consumed by sand run when creating a fresh local world; has no separate datapack representation.",
    use_when = ["Pinning sand run's local dev world to a reproducible seed"],
    avoid_when = ["Changing an already-created world's seed; not possible via datapack"],
    variants(Fixed = "A specific, reproducible seed value.", Random = "A freshly randomized seed, vanilla's default."),
    variant_fields(Fixed = ["The fixed seed value."]),
    example = "Seed::Fixed(12345);"
)]
pub enum Seed {
    Fixed(i64),
    Random,
}

/// A world generation preset, as offered in vanilla's "Create New World"
/// flow.
///
/// 🖥️ **Server (host) only.** Minecraft applies a preset at world creation
/// time; there is no datapack mechanism to set it. `sand run` uses this only
/// to decide `level-type`/`generator-settings` when it creates or resets its
/// local dev world directory. It is **not** part of the exported datapack
/// and has no effect for anyone who drops the datapack into an existing
/// world of their own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::WorldPreset",
    module = "sand::build",
    summary = "WorldPreset names a world generation preset as offered in vanilla's Create New World flow.",
    context = "Server (host) only: Minecraft applies a preset at world creation time; there is no datapack mechanism to set it.",
    minecraft = "Read by sand run to decide level-type/generator-settings when creating or resetting its local dev world.",
    use_when = ["Choosing sand run's local world creation preset"],
    avoid_when = ["Changing an already-created world's generation; a preset only applies at creation"],
    variants(Normal = "Vanilla default world generation. The default.", Flat = "Vanilla superflat preset.", LargeBiomes = "Vanilla large biomes preset.", Amplified = "Vanilla amplified terrain preset.", SingleBiomeSurface = "Vanilla single-biome preset."),
    example = "World::new().preset(WorldPreset::Flat);"
)]
pub enum WorldPreset {
    #[default]
    Normal,
    Flat,
    LargeBiomes,
    Amplified,
    SingleBiomeSurface,
}

/// Player spawn configuration.
///
/// 🌍 World (datapack) — lowers to `setworldspawn` (and, when
/// [`Spawn::platform`] is set, a small platform placed via `fill`) in the
/// generated load function.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::Spawn",
    module = "sand::build",
    summary = "Spawn configures the player spawn point and optional spawn platform.",
    context = "Set on a World via World::spawn; commonly paired with a platform for void/flat dev worlds.",
    minecraft = "Lowers to a setworldspawn command (and an optional fill command for the platform) in the generated load function.",
    use_when = ["Setting where players spawn in the exported datapack's world"],
    avoid_when = ["Configuring a dedicated server's own spawn protection; that is a server.properties concern"],
    example = "Spawn::at(0, 64, 0).platform(ResourceLocation::new(\"minecraft\", \"stone\").unwrap(), 3);"
)]
pub struct Spawn {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
    pub(crate) yaw: f32,
    pub(crate) platform: Option<SpawnPlatform>,
}

/// A small generated platform placed under the spawn point — useful for
/// void/flat dev worlds so players don't fall through on join.
///
/// 🌍 World (datapack).
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::SpawnPlatform",
    module = "sand::build",
    summary = "SpawnPlatform describes a small generated platform placed under a Spawn point.",
    context = "Attached to a Spawn via Spawn::platform; not constructed directly.",
    minecraft = "Lowers to a fill command in the generated load function.",
    use_when = ["Describing what block and radius a spawn platform uses"],
    avoid_when = ["Building general terrain; use a Generator instead"],
    example = "Spawn::at(0, 64, 0).platform(ResourceLocation::new(\"minecraft\", \"stone\").unwrap(), 3);"
)]
pub struct SpawnPlatform {
    pub(crate) block: ResourceLocation,
    pub(crate) radius: u32,
}

impl Spawn {
    /// Spawn at the given block coordinates, facing north (`yaw = 0`).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Spawn::at",
        module = "sand::build",
        summary = "Creates a spawn point at the given block coordinates, facing north.",
        context = "Starting point before optionally chaining .facing(...) or .platform(...).",
        minecraft = "Lowers to the generated load function's setworldspawn command.",
        use_when = ["Setting the exact spawn coordinates"],
        avoid_when = ["Adding a platform under spawn; chain .platform(...)"],
        params(x = "Spawn X coordinate.", y = "Spawn Y coordinate.", z = "Spawn Z coordinate."),
        returns = "A new Spawn.",
        example = "Spawn::at(0, 64, 0);"
    )]
    pub fn at(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z,
            yaw: 0.0,
            platform: None,
        }
    }

    /// Sets the initial facing (yaw, in degrees; vanilla convention).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Spawn::facing",
        module = "sand::build",
        summary = "Sets the initial facing (yaw, in degrees).",
        context = "Vanilla yaw convention: 0 is south by Minecraft's own axis, matching setworldspawn's angle argument.",
        minecraft = "Populates the setworldspawn command's angle argument.",
        use_when = ["Controlling which direction a freshly spawned player faces"],
        avoid_when = ["Setting spawn position; use Spawn::at"],
        params(yaw = "The facing angle in degrees."),
        returns = "This spawn with facing set.",
        example = "Spawn::at(0, 64, 0).facing(90.0);"
    )]
    pub fn facing(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self
    }

    /// Adds a square platform of `block`, `radius` blocks in each direction
    /// from the spawn column, one block below spawn Y.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Spawn::platform",
        module = "sand::build",
        summary = "Adds a small generated platform under the spawn point.",
        context = "Useful for void/flat dev worlds so players don't fall through on join.",
        minecraft = "Lowers to a fill command placing the given block in a square one block below spawn Y.",
        use_when = ["Preventing fall damage or void death on spawn in a void/flat world"],
        avoid_when = ["Building full terrain under spawn; use a Generator instead"],
        params(block = "The block the platform is made of.", radius = "The platform's radius in blocks from the spawn column."),
        returns = "This spawn with a platform configured.",
        example = "Spawn::at(0, 64, 0).platform(ResourceLocation::new(\"minecraft\", \"stone\").unwrap(), 3);"
    )]
    pub fn platform(mut self, block: ResourceLocation, radius: u32) -> Self {
        self.platform = Some(SpawnPlatform { block, radius });
        self
    }
}

/// World border configuration.
///
/// 🌍 World (datapack) — lowers to `worldborder` commands in the generated
/// load function (`center`, `set`, `damage amount`, `warning distance`,
/// `warning time`).
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::WorldBorder",
    module = "sand::build",
    summary = "WorldBorder configures the playable area's center, diameter, damage, and warning settings.",
    context = "Set on a World via World::border.",
    minecraft = "Lowers to worldborder center/set/damage/warning commands in the generated load function.",
    use_when = ["Limiting how far players can travel from spawn"],
    avoid_when = ["Limiting sand run's own render/simulation distance; use ServerConfig"],
    example = "WorldBorder::diameter(6000.0).center(0.0, 0.0);"
)]
pub struct WorldBorder {
    pub(crate) center_x: f64,
    pub(crate) center_z: f64,
    pub(crate) diameter: f64,
    pub(crate) damage_per_block: f64,
    pub(crate) warning_distance: u32,
    pub(crate) warning_time: u32,
}

impl WorldBorder {
    /// A border of the given diameter (blocks), centered on `(0, 0)`, with
    /// vanilla-default damage (0.2/block/second), warning distance (5), and
    /// warning time (15s).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::WorldBorder::diameter",
        module = "sand::build",
        summary = "Creates a border of the given diameter, centered on (0, 0), with vanilla-default damage and warnings.",
        context = "Starting point before optionally overriding center, damage, or warning settings.",
        minecraft = "Lowers to a worldborder set command with vanilla-default damage (0.2/block/s), warning distance (5), and warning time (15s).",
        use_when = ["Setting the playable area's overall size"],
        avoid_when = ["Changing the center; chain .center(...)"],
        params(diameter = "The border's diameter in blocks."),
        returns = "A new WorldBorder.",
        example = "WorldBorder::diameter(6000.0);"
    )]
    pub fn diameter(diameter: f64) -> Self {
        Self {
            center_x: 0.0,
            center_z: 0.0,
            diameter,
            damage_per_block: 0.2,
            warning_distance: 5,
            warning_time: 15,
        }
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::WorldBorder::center",
        module = "sand::build",
        summary = "Sets the border's center point.",
        context = "Overrides the default center of (0, 0).",
        minecraft = "Lowers to a worldborder center command.",
        use_when = ["Centering the border away from the world origin"],
        avoid_when = ["Setting the border's size; use WorldBorder::diameter"],
        params(x = "Center X coordinate.", z = "Center Z coordinate."),
        returns = "This border with its center set.",
        example = "WorldBorder::diameter(6000.0).center(100.0, -50.0);"
    )]
    pub fn center(mut self, x: f64, z: f64) -> Self {
        self.center_x = x;
        self.center_z = z;
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::WorldBorder::damage_per_block",
        module = "sand::build",
        summary = "Sets border damage dealt per block a player is outside it, per second.",
        context = "Overrides the vanilla default of 0.2.",
        minecraft = "Lowers to a worldborder damage amount command.",
        use_when = ["Tuning how punishing crossing the border is"],
        avoid_when = ["Tuning the warning distance/time; use the warning_* methods"],
        params(damage = "Damage per block outside the border, per second."),
        returns = "This border with damage set.",
        example = "WorldBorder::diameter(6000.0).damage_per_block(1.0);"
    )]
    pub fn damage_per_block(mut self, damage: f64) -> Self {
        self.damage_per_block = damage;
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::WorldBorder::warning_distance",
        module = "sand::build",
        summary = "Sets how many blocks from the border the warning effect begins.",
        context = "Overrides the vanilla default of 5 blocks.",
        minecraft = "Lowers to a worldborder warning distance command.",
        use_when = ["Giving players more or less visual warning before the border"],
        avoid_when = ["Setting the damage rate; use WorldBorder::damage_per_block"],
        params(blocks = "Warning distance in blocks."),
        returns = "This border with warning distance set.",
        example = "WorldBorder::diameter(6000.0).warning_distance(10);"
    )]
    pub fn warning_distance(mut self, blocks: u32) -> Self {
        self.warning_distance = blocks;
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::WorldBorder::warning_time",
        module = "sand::build",
        summary = "Sets how many seconds before a shrinking border reaches a player that the warning effect begins.",
        context = "Overrides the vanilla default of 15 seconds.",
        minecraft = "Lowers to a worldborder warning time command.",
        use_when = ["Giving players more or less time-based warning before a shrinking border"],
        avoid_when = ["Setting the warning distance; use WorldBorder::warning_distance"],
        params(seconds = "Warning time in seconds."),
        returns = "This border with warning time set.",
        example = "WorldBorder::diameter(6000.0).warning_time(30);"
    )]
    pub fn warning_time(mut self, seconds: u32) -> Self {
        self.warning_time = seconds;
        self
    }
}

/// The daylight cycle's initial time and whether it advances.
///
/// 🌍 World (datapack) — lowers to `time set` / `gamerule doDaylightCycle`
/// in the generated load function.
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::TimeConfig",
    module = "sand::build",
    summary = "TimeConfig sets the daylight cycle's initial time and whether it advances.",
    context = "Set on a World via World::time; useful for deterministic test/bench profiles.",
    minecraft = "Lowers to a time set command (and optionally gamerule doDaylightCycle false) in the generated load function.",
    use_when = ["Fixing the initial or frozen time of day for a world"],
    avoid_when = ["Setting initial weather; use WeatherConfig"],
    example = "TimeConfig::set(6000).frozen();"
)]
pub struct TimeConfig {
    pub(crate) ticks: i64,
    pub(crate) freeze: bool,
}

impl TimeConfig {
    /// Sets the time of day in ticks (0 = sunrise, 6000 = noon, 13000 =
    /// dusk, 18000 = midnight; vanilla range 0..24000).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::TimeConfig::set",
        module = "sand::build",
        summary = "Sets the time of day in ticks.",
        context = "0 is sunrise, 6000 is noon, 13000 is dusk, 18000 is midnight; vanilla range is 0..24000.",
        minecraft = "Lowers to a time set command in the generated load function.",
        use_when = ["Choosing the initial time of day"],
        avoid_when = ["Stopping the daylight cycle; chain .frozen()"],
        params(ticks = "Time of day in ticks."),
        returns = "A new TimeConfig.",
        example = "TimeConfig::set(6000);"
    )]
    pub fn set(ticks: i64) -> Self {
        Self {
            ticks,
            freeze: false,
        }
    }

    /// Freezes the daylight cycle at this time (`gamerule doDaylightCycle
    /// false`) — useful for deterministic `test`/`bench` profiles.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::TimeConfig::frozen",
        module = "sand::build",
        summary = "Freezes the daylight cycle at this time.",
        context = "Useful for deterministic test/bench profiles that need a fixed lighting condition.",
        minecraft = "Lowers to an additional gamerule doDaylightCycle false command.",
        use_when = ["Keeping time fixed for deterministic tests or screenshots"],
        avoid_when = ["Allowing the day/night cycle to progress normally; the default"],
        returns = "This TimeConfig with the cycle frozen.",
        example = "TimeConfig::set(18000).frozen();"
    )]
    pub fn frozen(mut self) -> Self {
        self.freeze = true;
        self
    }
}

/// Initial weather state.
///
/// 🌍 World (datapack) — lowers to `weather` commands in the generated load
/// function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::WeatherConfig",
    module = "sand::build",
    summary = "WeatherConfig sets the initial weather state for a World.",
    context = "Set on a World via World::weather.",
    minecraft = "Lowers to a weather command in the generated load function.",
    use_when = ["Starting a world in a specific weather state"],
    avoid_when = ["Setting the time of day; use TimeConfig"],
    variants(Clear = "Clear skies.", Rain = "Raining.", Thunder = "Thunderstorm."),
    example = "World::new().weather(WeatherConfig::Rain);"
)]
pub enum WeatherConfig {
    Clear,
    Rain,
    Thunder,
}

/// The primary world: seed, spawn, border, gamerules, time, weather, and the
/// [`Dimensions`] it contains.
///
/// 🌍 World (datapack), except [`World::preset`] — see the module docs.
///
/// ```
/// use sand_core::build::{Spawn, TimeConfig, WeatherConfig, World, WorldBorder};
///
/// let world = World::new()
///     .spawn(Spawn::at(0, 64, 0))
///     .border(WorldBorder::diameter(6000.0))
///     .gamerule("keepInventory", "true")
///     .time(TimeConfig::set(6000).frozen())
///     .weather(WeatherConfig::Clear);
///
/// assert_eq!(world.gamerules().get("keepInventory").map(String::as_str), Some("true"));
/// ```
#[derive(Debug, Clone, Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::World",
    module = "sand::build",
    summary = "World is the primary world's seed, spawn, border, gamerules, time, weather, and Dimensions.",
    context = "The top-level datapack-facing type a sand.build.rs script assembles and hands to SandBuild::world.",
    minecraft = "Everything reachable from World lowers into the exported datapack, except WorldPreset (server-only, see its own docs).",
    use_when = ["Configuring the world an exported datapack ships"],
    avoid_when = ["Configuring sand run's own local server process; use ServerConfig"],
    example = "World::new().spawn(Spawn::at(0, 64, 0)).gamerule(\"keepInventory\", \"true\");"
)]
pub struct World {
    pub(crate) seed: Option<Seed>,
    pub(crate) preset: WorldPreset,
    pub(crate) spawn: Option<Spawn>,
    pub(crate) border: Option<WorldBorder>,
    pub(crate) gamerules: BTreeMap<String, String>,
    pub(crate) time: Option<TimeConfig>,
    pub(crate) weather: Option<WeatherConfig>,
    pub(crate) dimensions: Dimensions,
}

impl World {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::new",
        module = "sand::build",
        summary = "Creates an empty World with no dimensions, spawn, border, gamerules, time, or weather configured.",
        context = "Starting point before chaining the other World builder methods.",
        minecraft = "Produces no world-init function or dimension resources until something is configured.",
        use_when = ["Starting a build script's World configuration"],
        avoid_when = ["Configuring server-only settings; use ServerConfig::new"],
        returns = "An empty World.",
        example = "World::new();"
    )]
    pub fn new() -> Self {
        Self::default()
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::seed",
        module = "sand::build",
        summary = "Sets the world seed sand run's local bootstrap should use when creating a fresh world.",
        context = "A datapack cannot retroactively change an existing world's seed; see Seed's own docs.",
        minecraft = "Consumed by sand run's local world-creation bootstrap only.",
        use_when = ["Pinning sand run's local dev world to a reproducible seed"],
        avoid_when = ["Setting spawn coordinates; use World::spawn"],
        params(seed = "The seed to use."),
        returns = "This world with the seed set.",
        example = "World::new().seed(Seed::Fixed(42));"
    )]
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// 🖥️ Server (host) only — see module docs on [`WorldPreset`].
    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::preset",
        module = "sand::build",
        summary = "Sets the world generation preset sand run's local bootstrap should use.",
        context = "Server (host) only — see WorldPreset's own docs for why.",
        minecraft = "Consumed by sand run's local world-creation bootstrap only; not part of the exported datapack.",
        use_when = ["Choosing sand run's local world creation preset"],
        avoid_when = ["Choosing a Generator for a specific Dimension; use Dimension::generator"],
        params(preset = "The world generation preset."),
        returns = "This world with the preset set.",
        example = "World::new().preset(WorldPreset::Flat);"
    )]
    pub fn preset(mut self, preset: WorldPreset) -> Self {
        self.preset = preset;
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::spawn",
        module = "sand::build",
        summary = "Sets the player spawn configuration.",
        context = "See Spawn's own docs for coordinates, facing, and platform options.",
        minecraft = "Lowers to a setworldspawn command in the generated load function.",
        use_when = ["Setting where players spawn"],
        avoid_when = ["Setting the world border; use World::border"],
        params(spawn = "The spawn configuration."),
        returns = "This world with spawn set.",
        example = "World::new().spawn(Spawn::at(0, 64, 0));"
    )]
    pub fn spawn(mut self, spawn: Spawn) -> Self {
        self.spawn = Some(spawn);
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::border",
        module = "sand::build",
        summary = "Sets the world border configuration.",
        context = "See WorldBorder's own docs for diameter, center, damage, and warning options.",
        minecraft = "Lowers to worldborder commands in the generated load function.",
        use_when = ["Limiting how far players can travel from spawn"],
        avoid_when = ["Setting spawn; use World::spawn"],
        params(border = "The world border configuration."),
        returns = "This world with the border set.",
        example = "World::new().border(WorldBorder::diameter(6000.0));"
    )]
    pub fn border(mut self, border: WorldBorder) -> Self {
        self.border = Some(border);
        self
    }

    /// Sets a single vanilla or datapack-defined gamerule by name.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::gamerule",
        module = "sand::build",
        summary = "Sets a single vanilla or datapack-defined gamerule by name.",
        context = "Call once per gamerule; later calls with the same name overwrite the earlier value.",
        minecraft = "Lowers to a gamerule command in the generated load function.",
        use_when = ["Setting keepInventory, doMobSpawning, or any other gamerule"],
        avoid_when = ["Setting the difficulty default; use ServerConfig::difficulty"],
        params(name = "The gamerule's name.", value = "The gamerule's value as text."),
        returns = "This world with the gamerule set.",
        example = "World::new().gamerule(\"keepInventory\", \"true\");"
    )]
    pub fn gamerule(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.gamerules.insert(name.into(), value.into());
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::time",
        module = "sand::build",
        summary = "Sets the initial time-of-day configuration.",
        context = "See TimeConfig's own docs for tick values and freezing the daylight cycle.",
        minecraft = "Lowers to a time set command (and optionally a gamerule) in the generated load function.",
        use_when = ["Fixing the initial time of day"],
        avoid_when = ["Setting weather; use World::weather"],
        params(time = "The time configuration."),
        returns = "This world with time set.",
        example = "World::new().time(TimeConfig::set(6000));"
    )]
    pub fn time(mut self, time: TimeConfig) -> Self {
        self.time = Some(time);
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::weather",
        module = "sand::build",
        summary = "Sets the initial weather state.",
        context = "See WeatherConfig's own docs for the available states.",
        minecraft = "Lowers to a weather command in the generated load function.",
        use_when = ["Starting a world in a specific weather state"],
        avoid_when = ["Setting the time of day; use World::time"],
        params(weather = "The weather state."),
        returns = "This world with weather set.",
        example = "World::new().weather(WeatherConfig::Clear);"
    )]
    pub fn weather(mut self, weather: WeatherConfig) -> Self {
        self.weather = Some(weather);
        self
    }

    /// Configures the Overworld/Nether/End and any custom dimensions this
    /// world contains.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::dimensions",
        module = "sand::build",
        summary = "Sets the Overworld/Nether/End and any custom dimensions this world contains.",
        context = "See Dimensions' own docs; omitted vanilla dimensions keep default vanilla generation.",
        minecraft = "Each configured dimension lowers to one dimension resource.",
        use_when = ["Configuring which dimensions this world overrides and how"],
        avoid_when = ["Configuring one dimension's generator; use Dimension::generator"],
        params(dimensions = "The dimensions configuration."),
        returns = "This world with dimensions set.",
        example = "World::new().dimensions(Dimensions::new());"
    )]
    pub fn dimensions(mut self, dimensions: Dimensions) -> Self {
        self.dimensions = dimensions;
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::gamerules",
        module = "sand::build",
        summary = "Returns the configured gamerules as a name-to-value map.",
        context = "Used by lowering and by tests asserting on which gamerules a World configures.",
        minecraft = "Reflects exactly the gamerule commands the generated load function will contain.",
        use_when = ["Inspecting which gamerules a World configures"],
        avoid_when = ["Setting a gamerule; use World::gamerule"],
        returns = "A map of gamerule name to value.",
        example = "assert!(World::new().gamerules().is_empty());"
    )]
    pub fn gamerules(&self) -> &BTreeMap<String, String> {
        &self.gamerules
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::World::dimensions_ref",
        module = "sand::build",
        summary = "Returns the configured Dimensions collection.",
        context = "Used by lowering and validation to iterate every configured dimension.",
        minecraft = "Reflects exactly the dimension resources the world will lower to.",
        use_when = ["Inspecting a World's configured dimensions"],
        avoid_when = ["Configuring dimensions; use World::dimensions"],
        returns = "A reference to the configured Dimensions.",
        example = "assert!(World::new().dimensions_ref().entries().is_empty());"
    )]
    pub fn dimensions_ref(&self) -> &Dimensions {
        &self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_accumulates_gamerules() {
        let world = World::new()
            .gamerule("keepInventory", "true")
            .gamerule("doMobSpawning", "false");
        assert_eq!(world.gamerules().len(), 2);
        assert_eq!(
            world.gamerules().get("doMobSpawning").map(String::as_str),
            Some("false")
        );
    }

    #[test]
    fn default_preset_is_normal() {
        assert_eq!(World::new().preset, WorldPreset::Normal);
    }
}
