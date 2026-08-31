//! 🖥️ Server (host) — local dev server integration.
//!
//! Everything in this module configures **Sand's own local dev server**
//! (`sand run`) or documents a `server.properties`-equivalent value you must
//! set yourself on a real dedicated server. **None of it is packaged into
//! the exported datapack.** A player who drops the generated datapack into
//! their own singleplayer world, a friend's server, or a host they don't
//! control has no way to receive these settings from the datapack — Sand
//! cannot embed them there because Minecraft itself has no datapack
//! mechanism for view distance, simulation distance, a difficulty
//! *default*, online-mode, or "wipe the world on start". If you deploy this
//! datapack to your own dedicated server, reproduce the equivalent settings
//! by hand in that server's `server.properties`.


use sand_macros::api;
/// Server-reported difficulty default.
///
/// 🖥️ **Server (host) only.** This is `server.properties`'
/// `difficulty` — the default applied when a world has none set yet. It is
/// distinct from a datapack-set difficulty *gamerule* override
/// (`gamerule` values travel with the world/datapack); this value does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::Difficulty",
    module = "sand::build",
    summary = "Difficulty is the server.properties difficulty default ServerConfig applies to sand run's local dev server.",
    context = "Distinct from a datapack-set difficulty gamerule override, which travels with the world; this value does not.",
    minecraft = "Maps directly to server.properties' difficulty key; has no datapack representation.",
    use_when = ["Setting sand run's default difficulty"],
    avoid_when = ["Setting a difficulty gamerule that should travel with the datapack; use World::gamerule"],
    variants(Peaceful = "No hostile mobs, no hunger loss.", Easy = "Reduced hostile mob damage and hunger effects.", Normal = "Standard vanilla difficulty (the default).", Hard = "Increased hostile mob damage and hunger effects."),
    example = "ServerConfig::new().difficulty(Difficulty::Hard);"
)]
pub enum Difficulty {
    Peaceful,
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Difficulty::as_str",
        module = "sand::build",
        summary = "Returns the server.properties value text for this difficulty.",
        context = "Used when sand run writes or updates server.properties from a ServerConfig.",
        minecraft = "Matches server.properties' difficulty key's exact accepted values (peaceful/easy/normal/hard).",
        use_when = ["Serializing a Difficulty into server.properties text"],
        avoid_when = ["Reading a datapack difficulty gamerule value"],
        returns = "The lowercase server.properties value.",
        example = "assert_eq!(Difficulty::Hard.as_str(), \"hard\");"
    )]
    pub fn as_str(self) -> &'static str {
        match self {
            Difficulty::Peaceful => "peaceful",
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
        }
    }
}

/// Whether `sand run` wipes and regenerates its local world directory
/// before starting the server each time.
///
/// 🖥️ **Server (host) only.** This governs Sand's own dev-server bootstrap
/// (`dist/server/world/`); it has no datapack representation and nothing to
/// do with how a real dedicated server operator manages their world
/// directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::WorldResetPolicy",
    module = "sand::build",
    summary = "WorldResetPolicy controls whether sand run wipes and regenerates its local world directory before starting the server.",
    context = "Governs only Sand's own dev-server bootstrap under dist/server/world/.",
    minecraft = "Has no datapack representation; a real dedicated server operator manages their own world directory independently.",
    use_when = ["Choosing whether dev/test profiles get a clean world every sand run"],
    avoid_when = ["Configuring anything that travels with the datapack"],
    variants(Keep = "Keep the existing local world directory across sand run invocations. The default.", AlwaysReset = "Delete and regenerate the local world directory before every sand run."),
    example = "ServerConfig::new().world_reset_policy(WorldResetPolicy::AlwaysReset);"
)]
pub enum WorldResetPolicy {
    /// Keep the existing local world directory across `sand run` invocations
    /// (vanilla-server-like persistence). The default.
    #[default]
    Keep,
    /// Delete and regenerate the local world directory before every `sand
    /// run`. Useful for `dev`/`test` profiles that want a clean world every
    /// time.
    AlwaysReset,
}

/// Local dev-server integration settings, consumed only by `sand run`.
///
/// 🖥️ **Server (host) only — never serialized into the exported datapack.**
/// See the module docs for why: view distance, simulation distance, a
/// difficulty *default*, online-mode, and world-reset policy all live in
/// `server.properties` or Sand's own bootstrap logic, not in any datapack
/// file. Deploying this datapack to a different server reproduces none of
/// these automatically — set the equivalent values by hand in that server's
/// `server.properties`.
///
/// ```
/// use sand_core::build::{Difficulty, ServerConfig, WorldResetPolicy};
///
/// let server = ServerConfig::new()
///     .view_distance(12)
///     .simulation_distance(8)
///     .difficulty(Difficulty::Hard)
///     .online_mode(false)
///     .world_reset_policy(WorldResetPolicy::AlwaysReset);
///
/// assert_eq!(server.get_view_distance(), 12);
/// ```
#[derive(Debug, Clone, Copy)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::ServerConfig",
    module = "sand::build",
    summary = "ServerConfig holds local dev-server integration settings consumed only by sand run.",
    context = "Structurally separate from World: view distance, simulation distance, a difficulty default, online-mode, and world-reset policy have no datapack representation at all.",
    minecraft = "Never serialized into the exported datapack; anyone deploying the datapack to their own server must reproduce equivalent server.properties values by hand.",
    use_when = ["Configuring sand run's local Minecraft server process"],
    avoid_when = ["Configuring anything that should travel with the exported datapack; use World"],
    example = "ServerConfig::new().view_distance(12).difficulty(Difficulty::Hard);"
)]
pub struct ServerConfig {
    view_distance: u8,
    simulation_distance: u8,
    difficulty: Difficulty,
    online_mode: bool,
    world_reset_policy: WorldResetPolicy,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            view_distance: 10,
            simulation_distance: 10,
            difficulty: Difficulty::Normal,
            online_mode: true,
            world_reset_policy: WorldResetPolicy::Keep,
        }
    }
}

impl ServerConfig {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::new",
        module = "sand::build",
        summary = "Creates a ServerConfig with vanilla server.properties defaults.",
        context = "Starting point before overriding individual settings for sand run.",
        minecraft = "Matches vanilla's default view-distance/simulation-distance (10), difficulty (normal), online-mode (true), and keeps the local world across runs.",
        use_when = ["Starting local dev-server configuration"],
        avoid_when = ["Configuring datapack content; use World"],
        returns = "A new ServerConfig with vanilla defaults.",
        example = "ServerConfig::new();"
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// 🖥️ Server (host) only — `server.properties` `view-distance`.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::view_distance",
        module = "sand::build",
        summary = "Sets server.properties' view-distance for sand run's local server.",
        context = "Server (host) only — has no datapack representation.",
        minecraft = "Written directly into server.properties' view-distance key.",
        use_when = ["Tuning sand run's server-side render distance"],
        avoid_when = ["Anything datapack-visible; view distance has no datapack equivalent"],
        params(chunks = "The view distance in chunks."),
        returns = "This config with view_distance set.",
        example = "ServerConfig::new().view_distance(12);"
    )]
    pub fn view_distance(mut self, chunks: u8) -> Self {
        self.view_distance = chunks;
        self
    }

    /// 🖥️ Server (host) only — `server.properties` `simulation-distance`.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::simulation_distance",
        module = "sand::build",
        summary = "Sets server.properties' simulation-distance for sand run's local server.",
        context = "Server (host) only — has no datapack representation.",
        minecraft = "Written directly into server.properties' simulation-distance key.",
        use_when = ["Tuning sand run's server-side simulation range"],
        avoid_when = ["Anything datapack-visible; simulation distance has no datapack equivalent"],
        params(chunks = "The simulation distance in chunks."),
        returns = "This config with simulation_distance set.",
        example = "ServerConfig::new().simulation_distance(8);"
    )]
    pub fn simulation_distance(mut self, chunks: u8) -> Self {
        self.simulation_distance = chunks;
        self
    }

    /// 🖥️ Server (host) only — `server.properties` `difficulty` default.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::difficulty",
        module = "sand::build",
        summary = "Sets server.properties' difficulty default for sand run's local server.",
        context = "Server (host) only — distinct from a datapack-set difficulty gamerule.",
        minecraft = "Written directly into server.properties' difficulty key.",
        use_when = ["Choosing sand run's default difficulty"],
        avoid_when = ["Setting a difficulty gamerule that should travel with the datapack; use World::gamerule"],
        params(difficulty = "The difficulty default to apply."),
        returns = "This config with difficulty set.",
        example = "ServerConfig::new().difficulty(Difficulty::Hard);"
    )]
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// 🖥️ Server (host) only — `server.properties` `online-mode`.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::online_mode",
        module = "sand::build",
        summary = "Sets server.properties' online-mode for sand run's local server.",
        context = "Server (host) only — has no datapack representation.",
        minecraft = "Written directly into server.properties' online-mode key.",
        use_when = ["Testing offline/cracked-client connections against sand run"],
        avoid_when = ["Anything datapack-visible; online-mode has no datapack equivalent"],
        params(enabled = "Whether Mojang session authentication is required."),
        returns = "This config with online_mode set.",
        example = "ServerConfig::new().online_mode(false);"
    )]
    pub fn online_mode(mut self, enabled: bool) -> Self {
        self.online_mode = enabled;
        self
    }

    /// 🖥️ Server (host) only — controls whether `sand run` wipes its local
    /// world directory between runs.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::world_reset_policy",
        module = "sand::build",
        summary = "Sets whether sand run wipes its local world directory between runs.",
        context = "Server (host) only — governs only Sand's own dev-server bootstrap.",
        minecraft = "Has no datapack representation; affects only dist/server/world/ on the machine running sand run.",
        use_when = ["Getting a clean world every sand run for dev/test profiles"],
        avoid_when = ["Configuring datapack content; use World"],
        params(policy = "Whether to keep or always reset the local world."),
        returns = "This config with world_reset_policy set.",
        example = "ServerConfig::new().world_reset_policy(WorldResetPolicy::AlwaysReset);"
    )]
    pub fn world_reset_policy(mut self, policy: WorldResetPolicy) -> Self {
        self.world_reset_policy = policy;
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::get_view_distance",
        module = "sand::build",
        summary = "Returns the configured view distance.",
        context = "Read by sand run when applying a ServerConfig to server.properties.",
        minecraft = "Matches the value server.properties' view-distance key receives.",
        use_when = ["Inspecting a ServerConfig's view distance"],
        avoid_when = ["Setting it; use .view_distance(...)"],
        returns = "The configured view distance in chunks.",
        example = "assert_eq!(ServerConfig::new().get_view_distance(), 10);"
    )]
    pub fn get_view_distance(&self) -> u8 {
        self.view_distance
    }
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::get_simulation_distance",
        module = "sand::build",
        summary = "Returns the configured simulation distance.",
        context = "Read by sand run when applying a ServerConfig to server.properties.",
        minecraft = "Matches the value server.properties' simulation-distance key receives.",
        use_when = ["Inspecting a ServerConfig's simulation distance"],
        avoid_when = ["Setting it; use .simulation_distance(...)"],
        returns = "The configured simulation distance in chunks.",
        example = "assert_eq!(ServerConfig::new().get_simulation_distance(), 10);"
    )]
    pub fn get_simulation_distance(&self) -> u8 {
        self.simulation_distance
    }
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::get_difficulty",
        module = "sand::build",
        summary = "Returns the configured difficulty default.",
        context = "Read by sand run when applying a ServerConfig to server.properties.",
        minecraft = "Matches the value server.properties' difficulty key receives.",
        use_when = ["Inspecting a ServerConfig's difficulty default"],
        avoid_when = ["Setting it; use .difficulty(...)"],
        returns = "The configured Difficulty.",
        example = "assert_eq!(ServerConfig::new().get_difficulty(), Difficulty::Normal);"
    )]
    pub fn get_difficulty(&self) -> Difficulty {
        self.difficulty
    }
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::get_online_mode",
        module = "sand::build",
        summary = "Returns the configured online-mode flag.",
        context = "Read by sand run when applying a ServerConfig to server.properties.",
        minecraft = "Matches the value server.properties' online-mode key receives.",
        use_when = ["Inspecting a ServerConfig's online-mode setting"],
        avoid_when = ["Setting it; use .online_mode(...)"],
        returns = "The configured online-mode flag.",
        example = "assert!(ServerConfig::new().get_online_mode());"
    )]
    pub fn get_online_mode(&self) -> bool {
        self.online_mode
    }
    #[api(
        registry = sand_api_contract,
        path = "sand::build::ServerConfig::get_world_reset_policy",
        module = "sand::build",
        summary = "Returns the configured world-reset policy.",
        context = "Read by sand run to decide whether to wipe its local world directory before starting.",
        minecraft = "Has no server.properties equivalent; it governs sand run's own bootstrap logic.",
        use_when = ["Inspecting a ServerConfig's world-reset policy"],
        avoid_when = ["Setting it; use .world_reset_policy(...)"],
        returns = "The configured WorldResetPolicy.",
        example = "assert_eq!(ServerConfig::new().get_world_reset_policy(), WorldResetPolicy::Keep);"
    )]
    pub fn get_world_reset_policy(&self) -> WorldResetPolicy {
        self.world_reset_policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_vanilla_server_properties_defaults() {
        let cfg = ServerConfig::new();
        assert_eq!(cfg.get_view_distance(), 10);
        assert_eq!(cfg.get_simulation_distance(), 10);
        assert_eq!(cfg.get_difficulty(), Difficulty::Normal);
        assert!(cfg.get_online_mode());
        assert_eq!(cfg.get_world_reset_policy(), WorldResetPolicy::Keep);
    }

    #[test]
    fn builder_overrides_every_field() {
        let cfg = ServerConfig::new()
            .view_distance(4)
            .simulation_distance(4)
            .difficulty(Difficulty::Peaceful)
            .online_mode(false)
            .world_reset_policy(WorldResetPolicy::AlwaysReset);
        assert_eq!(cfg.get_view_distance(), 4);
        assert_eq!(cfg.get_difficulty(), Difficulty::Peaceful);
        assert!(!cfg.get_online_mode());
        assert_eq!(cfg.get_world_reset_policy(), WorldResetPolicy::AlwaysReset);
    }
}
