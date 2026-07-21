//! Tracks overall `sand run` health, kept explicitly separate from the
//! Minecraft server *process* becoming ready. A server can be process-ready
//! (accepting commands, players can join) while its datapack failed to
//! load — `sand run` must not report that as an unqualified success.

use serde::Serialize;

use super::diagnostic::{Diagnostic, Fatality};

/// Overall health of a `sand run` invocation, distinct from whether the
/// Minecraft server *process* is up. Monotonic within a run: once
/// `Degraded` or `Failed`, later healthy-looking output (including the
/// `Done (...)!` ready banner) never resets it back to `Healthy` — a
/// failure earlier in the run doesn't un-happen because the server kept
/// going.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunHealth {
    /// The server process is (or became) ready, and no diagnostic fatal to
    /// startup or datapack health has been observed.
    Healthy,
    /// The server process is (or became) ready, but the datapack (or a
    /// `/reload`) failed to load fully.
    Degraded,
    /// The server process failed to start, or exited before becoming ready.
    Failed,
}

impl RunHealth {
    pub fn is_healthy(self) -> bool {
        matches!(self, RunHealth::Healthy)
    }
}

/// Accumulates [`RunHealth`] from the diagnostics observed during a run.
#[derive(Debug, Clone, Copy)]
pub struct HealthTracker {
    health: RunHealth,
}

impl HealthTracker {
    pub fn new() -> Self {
        Self {
            health: RunHealth::Healthy,
        }
    }

    pub fn current(self) -> RunHealth {
        self.health
    }

    /// Fold one diagnostic's fatality into the running health. `Failed`
    /// always wins; `Degraded` sticks unless already `Failed`; a nonfatal
    /// diagnostic never changes the current health, including a nonfatal
    /// one observed after a fatal one.
    pub fn observe(&mut self, diagnostic: &Diagnostic) {
        match diagnostic.fatality() {
            Fatality::FatalToStartup => self.health = RunHealth::Failed,
            Fatality::FatalToDatapackHealth => {
                if self.health != RunHealth::Failed {
                    self.health = RunHealth::Degraded;
                }
            }
            Fatality::Nonfatal => {}
        }
    }
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::diagnostic::{DiagnosticCode, Severity};
    use crate::console::phase::RunPhase;

    fn diag(code: DiagnosticCode, phase: RunPhase) -> Diagnostic {
        Diagnostic {
            phase,
            severity: Severity::Error,
            code,
            resource: None,
            subsystem: None,
            file: None,
            position: None,
            line: None,
            cursor: None,
            context: None,
            reason: "test".to_string(),
            hint: None,
            related: vec![],
            raw_lines: vec![],
            missing_tag: None,
        }
    }

    #[test]
    fn starts_healthy() {
        assert_eq!(HealthTracker::new().current(), RunHealth::Healthy);
    }

    #[test]
    fn datapack_load_failure_degrades() {
        let mut t = HealthTracker::new();
        t.observe(&diag(
            DiagnosticCode::CommandParseError,
            RunPhase::DatapackDiscovery,
        ));
        assert_eq!(t.current(), RunHealth::Degraded);
    }

    #[test]
    fn startup_failure_fails() {
        let mut t = HealthTracker::new();
        t.observe(&diag(
            DiagnosticCode::StartupFailure,
            RunPhase::ServerStartup,
        ));
        assert_eq!(t.current(), RunHealth::Failed);
    }

    #[test]
    fn failed_is_sticky_even_after_a_degraded_observation() {
        let mut t = HealthTracker::new();
        t.observe(&diag(
            DiagnosticCode::StartupFailure,
            RunPhase::ServerStartup,
        ));
        t.observe(&diag(
            DiagnosticCode::CommandParseError,
            RunPhase::DatapackDiscovery,
        ));
        assert_eq!(t.current(), RunHealth::Failed);
    }

    #[test]
    fn nonfatal_runtime_command_error_after_readiness_does_not_degrade() {
        let mut t = HealthTracker::new();
        t.observe(&diag(
            DiagnosticCode::RuntimeCommandError,
            RunPhase::Runtime,
        ));
        assert_eq!(t.current(), RunHealth::Healthy);
    }

    #[test]
    fn a_reload_failure_degrades_even_after_a_prior_healthy_ready_state() {
        let mut t = HealthTracker::new();
        // Server was healthy at readiness...
        assert_eq!(t.current(), RunHealth::Healthy);
        // ...but a later `/reload` fails.
        t.observe(&diag(DiagnosticCode::ReloadFailure, RunPhase::Reload));
        assert_eq!(t.current(), RunHealth::Degraded);
    }
}
