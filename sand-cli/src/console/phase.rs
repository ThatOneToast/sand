//! Explicit `sand run` phase tracking.
//!
//! Every diagnostic is associated with the phase that was active when it was
//! observed, so e.g. a command failure reported while the server is still
//! loading the world is never confused with the same failure reported after
//! a `/reload`. Phase transitions are driven by real process/command state
//! (see [`PhaseTracker`]), not inferred from log wording alone.

use serde::Serialize;

/// The major phases a `sand run` invocation moves through, in roughly the
/// order they occur. `sand run` may skip `CargoBuild`/`SandExport` when
/// invoked with `--no-build`, and may cycle back to `Reload` repeatedly
/// during an interactive session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunPhase {
    /// Running `cargo build` for the datapack crate.
    CargoBuild,
    /// Exporting compiled Sand data into the `dist/` datapack tree.
    SandExport,
    /// Downloading/verifying the server jar and syncing the datapack into
    /// `dist/server/world/datapacks/`.
    PackInstall,
    /// The Minecraft server process has been launched and is initializing,
    /// before it reports itself ready.
    ServerStartup,
    /// The server is discovering/loading datapacks, either during initial
    /// startup or as part of a `/reload`.
    DatapackDiscovery,
    /// A `/reload` has been issued and the server is re-evaluating data.
    Reload,
    /// The server is up and accepting commands/players.
    Runtime,
    /// The server has been asked to stop, or has exited.
    Shutdown,
}

impl RunPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            RunPhase::CargoBuild => "cargo_build",
            RunPhase::SandExport => "sand_export",
            RunPhase::PackInstall => "pack_install",
            RunPhase::ServerStartup => "server_startup",
            RunPhase::DatapackDiscovery => "datapack_discovery",
            RunPhase::Reload => "reload",
            RunPhase::Runtime => "runtime",
            RunPhase::Shutdown => "shutdown",
        }
    }
}

impl std::fmt::Display for RunPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tracks the current [`RunPhase`] for the server-process portion of a
/// `sand run` invocation (`ServerStartup` onward). The pre-process phases
/// (`CargoBuild`, `SandExport`, `PackInstall`) are set directly by
/// `run_cmd::run` before the server process is ever spawned; this tracker
/// only owns transitions that are driven by the server's own log output and
/// by commands issued to it.
#[derive(Debug, Clone, Copy)]
pub struct PhaseTracker {
    phase: RunPhase,
    seen_ready: bool,
}

impl PhaseTracker {
    pub fn new() -> Self {
        Self {
            phase: RunPhase::ServerStartup,
            seen_ready: false,
        }
    }

    pub fn current(&self) -> RunPhase {
        self.phase
    }

    /// Observe a raw (unclassified) server log message and update phase
    /// state. Only transitions on lines that unambiguously indicate a phase
    /// change; everything else leaves the current phase untouched.
    pub fn observe_log(&mut self, message: &str) {
        if !self.seen_ready
            && (message.contains("Preparing level")
                || message.contains("Reloading ResourceManager")
                || message.contains("Loading data packs")
                || message.contains("data pack"))
        {
            self.phase = RunPhase::DatapackDiscovery;
            return;
        }

        if message.contains("Done (") && message.contains(")!") {
            self.seen_ready = true;
            self.phase = RunPhase::Runtime;
            return;
        }

        if self.seen_ready
            && (message.contains("Reloading ResourceManager") || message.contains("data pack"))
        {
            // A reload's own discovery work is still phase `Reload`, not a
            // fresh `DatapackDiscovery` — that phase only applies to the
            // very first, pre-ready load.
            self.phase = RunPhase::Reload;
            return;
        }

        if message.contains("Stopping the server")
            || message.contains("Stopping server")
            || message.contains("Closing Server")
        {
            self.phase = RunPhase::Shutdown;
            return;
        }

        if self.seen_ready && self.phase == RunPhase::Reload {
            // Any further ordinary output after a reload's discovery lines
            // means the reload finished (successfully or not); fall back to
            // `Runtime` until another `/reload` is issued.
            if !message.contains("data pack") {
                self.phase = RunPhase::Runtime;
            }
        }
    }

    /// Observe a command sent to the server's stdin (operator/console
    /// input), so an explicit `/reload` is tracked precisely rather than
    /// inferred from log wording alone.
    pub fn observe_command(&mut self, command: &str) {
        let trimmed = command.trim().trim_start_matches('/');
        if trimmed == "reload" {
            self.phase = RunPhase::Reload;
        } else if trimmed == "stop" {
            self.phase = RunPhase::Shutdown;
        }
    }
}

impl Default for PhaseTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_server_startup() {
        assert_eq!(PhaseTracker::new().current(), RunPhase::ServerStartup);
    }

    #[test]
    fn moves_to_datapack_discovery_before_ready() {
        let mut t = PhaseTracker::new();
        t.observe_log("Preparing level \"world\"");
        assert_eq!(t.current(), RunPhase::DatapackDiscovery);
    }

    #[test]
    fn moves_to_runtime_on_ready_banner() {
        let mut t = PhaseTracker::new();
        t.observe_log("Preparing level \"world\"");
        t.observe_log("Done (3.421s)! For help, type \"help\"");
        assert_eq!(t.current(), RunPhase::Runtime);
    }

    #[test]
    fn explicit_reload_command_sets_reload_phase() {
        let mut t = PhaseTracker::new();
        t.observe_log("Done (3.421s)! For help, type \"help\"");
        assert_eq!(t.current(), RunPhase::Runtime);
        t.observe_command("reload");
        assert_eq!(t.current(), RunPhase::Reload);
    }

    #[test]
    fn reload_discovery_does_not_report_as_first_time_datapack_discovery() {
        let mut t = PhaseTracker::new();
        t.observe_log("Done (3.421s)! For help, type \"help\"");
        t.observe_command("reload");
        t.observe_log("Reloading ResourceManager: default, arcane");
        assert_eq!(t.current(), RunPhase::Reload);
        t.observe_log("Loaded 12 recipes");
        assert_eq!(t.current(), RunPhase::Runtime);
    }

    #[test]
    fn stop_command_and_log_both_set_shutdown() {
        let mut t = PhaseTracker::new();
        t.observe_log("Done (3.421s)! For help, type \"help\"");
        t.observe_command("stop");
        assert_eq!(t.current(), RunPhase::Shutdown);

        let mut t2 = PhaseTracker::new();
        t2.observe_log("Done (3.421s)! For help, type \"help\"");
        t2.observe_log("Stopping the server");
        assert_eq!(t2.current(), RunPhase::Shutdown);
    }
}
