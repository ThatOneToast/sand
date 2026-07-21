//! Turns classified/grouped log events into what actually gets printed.
//!
//! Four output modes are supported (see [`OutputMode`]): `classified`
//! (default filtered/formatted output), `verbose` (classified output plus
//! the raw lines behind each event), `raw` (unfiltered passthrough, for
//! debugging the classifier itself), and `json` (structured diagnostics
//! only, one JSON object per line, for machine consumption).
//!
//! Diagnostics pass through three stages before printing: [`Correlator`]
//! (link a consequence to its root failure), [`Deduplicator`] (fold
//! repeats), then rendering. [`HealthTracker`] observes every diagnostic
//! (pre-correlation/dedup, since merging or folding a diagnostic doesn't
//! change whether it happened) to track [`RunHealth`] across the whole run,
//! kept explicitly separate from the server *process* becoming ready.

use colored::Colorize;
use serde::Serialize;

use super::classify::Category;
use super::correlate::Correlator;
use super::dedup::{Deduplicator, Observation, RepeatSummary};
use super::diagnostic::{Diagnostic, GroupedEvent, RelatedDiagnostic, Severity, build_diagnostic};
use super::health::{HealthTracker, RunHealth};
use super::phase::RunPhase;

/// How `sand run` should present the Minecraft server's log output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum OutputMode {
    /// Sand's filtered, formatted console (default).
    Classified,
    /// Classified output, plus the raw log lines behind each event.
    Verbose,
    /// The server's raw, unfiltered log, with no classification at all.
    Raw,
    /// Structured diagnostics only, one JSON object per line on stdout.
    Json,
}

/// Prints [`GroupedEvent`]s according to the active [`OutputMode`].
pub struct Renderer {
    mode: OutputMode,
    mc_version: String,
    printed_ready_banner: bool,
    correlator: Correlator,
    dedup: Deduplicator,
    health: HealthTracker,
}

impl Renderer {
    pub fn new(mode: OutputMode, mc_version: String) -> Self {
        Self {
            mode,
            mc_version,
            printed_ready_banner: false,
            correlator: Correlator::new(),
            dedup: Deduplicator::new(),
            health: HealthTracker::new(),
        }
    }

    pub fn render(&mut self, event: &GroupedEvent, phase: RunPhase) {
        if self.mode == OutputMode::Raw {
            for line in &event.lines {
                println!("{}", line.raw);
            }
            return;
        }

        if matches!(
            event.category,
            Category::DatapackError | Category::FatalError
        ) {
            match build_diagnostic(event, phase) {
                Some(diag) => self.emit_diagnostic(diag),
                None => self.emit_plain_error(event),
            }
            return;
        }

        // JSON mode only ever emits the structured diagnostic model, never
        // terminal-formatted progress strings.
        if self.mode == OutputMode::Json {
            return;
        }

        self.render_non_diagnostic(event);
    }

    /// Current overall run health, for [`super::process::RunOutcome`].
    pub fn health(&self) -> RunHealth {
        self.health.current()
    }

    /// Flush anything still buffered (a correlator root awaiting a
    /// consequence that never came, a pending dedup repeat count) and print
    /// the final health status, so nothing is lost and the run's true
    /// outcome is never left implicit. Call once after the last `render`.
    pub fn finish(&mut self) {
        self.flush_pending_correlation();
        if let Some(summary) = self.dedup.flush() {
            self.print_repeat_summary(&summary);
        }
        self.print_final_health();
    }

    /// Flush a diagnostic the correlator is holding in case a consequence
    /// follows, if no consequence has shown up within a quiet period. Bounds
    /// the extra latency correlation adds to the same window the grouper
    /// already uses for its own continuation-line buffering.
    pub fn flush_pending_correlation(&mut self) {
        if let Some(root) = self.correlator.flush() {
            self.dedup_and_emit(root);
        }
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.health.observe(&diagnostic);
        for correlated in self.correlator.observe(diagnostic) {
            self.dedup_and_emit(correlated);
        }
    }

    fn dedup_and_emit(&mut self, diagnostic: Diagnostic) {
        match self.dedup.observe(diagnostic) {
            Observation::New {
                diagnostic,
                previous_repeat,
            } => {
                if let Some(summary) = previous_repeat {
                    self.print_repeat_summary(&summary);
                }
                self.print_diagnostic(&diagnostic);
            }
            Observation::Repeated => {}
        }
    }

    fn emit_plain_error(&self, event: &GroupedEvent) {
        match self.mode {
            OutputMode::Json => {}
            OutputMode::Classified | OutputMode::Verbose => {
                println!("{} {}", "error:".red().bold(), message(event));
                if self.mode == OutputMode::Verbose {
                    self.print_raw_lines(event);
                }
            }
            OutputMode::Raw => unreachable!("handled above"),
        }
    }

    fn render_non_diagnostic(&mut self, event: &GroupedEvent) {
        match event.category {
            Category::Suppress => {}
            Category::Ready => self.render_ready(),
            Category::Shutdown => println!("{}", message(event).yellow()),
            Category::PlayerEvent => println!("{}", message(event)),
            Category::CommandResponse => println!("{}", message(event)),
            Category::Warning => println!("{} {}", "warning:".yellow().bold(), message(event)),
            Category::Visible => println!("{}", message(event)),
            Category::DatapackError | Category::FatalError => unreachable!("handled by render()"),
        }
        if self.mode == OutputMode::Verbose && event.category != Category::Suppress {
            self.print_raw_lines(event);
        }
    }

    /// Marks the server *process* as ready. Deliberately does not claim
    /// overall success: if datapack diagnostics already degraded
    /// [`RunHealth`] by this point (the common case — a load-time failure
    /// is always logged before the ready banner), this prints a neutral
    /// status instead of the green checkmark, and [`Self::finish`] prints
    /// the definitive final status at the true end of the run.
    fn render_ready(&mut self) {
        if self.printed_ready_banner {
            return;
        }
        self.printed_ready_banner = true;
        println!();
        if self.health.current().is_healthy() {
            println!(
                "{} Minecraft {} ready",
                "✓".green().bold(),
                self.mc_version.yellow()
            );
        } else {
            println!(
                "{} Minecraft {} process is ready",
                "…".dimmed(),
                self.mc_version.yellow()
            );
        }
        println!("  Type server commands directly; use `stop` to exit.");
        println!();
    }

    /// Prints the definitive run-health status at the true end of the run.
    /// Silent when [`RunHealth::Healthy`], so the common/happy path isn't
    /// cluttered with a redundant confirmation beyond the ready banner
    /// already printed by [`Self::render_ready`].
    fn print_final_health(&self) {
        match self.mode {
            OutputMode::Json => {
                println!("{}", serde_json::json!({ "health": self.health.current() }));
            }
            OutputMode::Classified | OutputMode::Verbose => match self.health.current() {
                RunHealth::Healthy => {}
                RunHealth::Degraded => {
                    println!();
                    println!(
                        "{} Minecraft {} process started, but the datapack failed to load.",
                        "✗".red().bold(),
                        self.mc_version.yellow()
                    );
                }
                RunHealth::Failed => {
                    println!();
                    println!(
                        "{} Minecraft {} did not start successfully.",
                        "✗".red().bold(),
                        self.mc_version.yellow()
                    );
                }
            },
            OutputMode::Raw => {}
        }
    }

    fn print_diagnostic(&self, diag: &Diagnostic) {
        match self.mode {
            OutputMode::Json => {
                println!(
                    "{}",
                    serde_json::to_string(&JsonDiagnostic::from(diag)).unwrap()
                );
            }
            OutputMode::Classified | OutputMode::Verbose => {
                let label = match diag.severity {
                    Severity::Error => "datapack error".red().bold(),
                    Severity::Warning => "datapack warning".yellow().bold(),
                };
                let resource = diag.resource.as_deref().unwrap_or("<unknown resource>");
                println!(
                    "{label}[{}]: failed to load {resource}",
                    format!("{:?}", diag.code).to_ascii_lowercase()
                );
                println!("  {} {}", "phase:".dimmed(), diag.phase);

                if let Some(file) = &diag.file {
                    println!("  {} {}", "file:".dimmed(), file);
                }
                if let Some(line) = diag.line {
                    match diag.cursor {
                        Some(cursor) => {
                            println!("  {} {line}, position {cursor}", "line:".dimmed())
                        }
                        None => println!("  {} {line}", "line:".dimmed()),
                    }
                }
                match (&diag.position, &diag.context) {
                    (Some(pos), Some(ctx)) => println!("  {pos}: {ctx}"),
                    (None, Some(ctx)) => println!("  {} {}", "context:".dimmed(), ctx),
                    _ => {}
                }
                println!("  {} {}", "reason:".dimmed(), diag.reason);
                if let Some(hint) = &diag.hint {
                    println!("  {} {}", "hint:".dimmed(), hint);
                }
                for related in &diag.related {
                    print!(
                        "  {} {} could not resolve {}",
                        "related:".dimmed(),
                        related.resource,
                        related.missing
                    );
                    if let Some(source) = &related.source {
                        print!(" (source: {source})");
                    }
                    println!();
                }
                if self.mode == OutputMode::Verbose && !diag.raw_lines.is_empty() {
                    println!("  {}", "raw:".dimmed());
                    for line in &diag.raw_lines {
                        println!("    {line}");
                    }
                }
                println!();
            }
            OutputMode::Raw => unreachable!("handled above"),
        }
    }

    fn print_repeat_summary(&self, summary: &RepeatSummary) {
        match self.mode {
            OutputMode::Json => {
                println!("{}", serde_json::json!({ "repeated": summary.count }));
            }
            OutputMode::Classified | OutputMode::Verbose => {
                println!("  {} repeated {} times", "note:".dimmed(), summary.count);
                println!();
            }
            OutputMode::Raw => unreachable!("handled above"),
        }
    }

    fn print_raw_lines(&self, event: &GroupedEvent) {
        for line in &event.lines {
            println!("  {} {}", "raw:".dimmed(), line.raw);
        }
    }
}

/// The JSON-serialized shape of a [`Diagnostic`]. A thin wrapper (rather
/// than deriving `Serialize` straight onto the terminal-rendering path)
/// keeps the JSON schema stable even if internal `Diagnostic` fields shift.
#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    phase: RunPhase,
    severity: Severity,
    code: super::diagnostic::DiagnosticCode,
    /// Whether this diagnostic invalidates a successful run (see
    /// [`Diagnostic::fatality`]) — `false` for nonfatal runtime feedback.
    fatal: bool,
    resource: &'a Option<String>,
    subsystem: &'a Option<String>,
    file: &'a Option<String>,
    line: Option<u32>,
    position: &'a Option<String>,
    cursor: Option<u32>,
    context: &'a Option<String>,
    reason: &'a str,
    hint: &'a Option<String>,
    related: &'a [RelatedDiagnostic],
    raw: &'a [String],
}

impl<'a> From<&'a Diagnostic> for JsonDiagnostic<'a> {
    fn from(diag: &'a Diagnostic) -> Self {
        use super::diagnostic::Fatality;
        Self {
            phase: diag.phase,
            severity: diag.severity,
            code: diag.code,
            fatal: !matches!(diag.fatality(), Fatality::Nonfatal),
            resource: &diag.resource,
            subsystem: &diag.subsystem,
            file: &diag.file,
            line: diag.line,
            position: &diag.position,
            cursor: diag.cursor,
            context: &diag.context,
            reason: &diag.reason,
            hint: &diag.hint,
            related: &diag.related,
            raw: &diag.raw_lines,
        }
    }
}

fn message(event: &GroupedEvent) -> &str {
    event.headline().message.trim()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::classify::classify;
    use crate::console::diagnostic::Grouper;
    use crate::console::log_record::{Stream, parse_line};

    fn single_event(line: &str) -> GroupedEvent {
        let mut grouper = Grouper::new();
        let record = parse_line(line);
        let category = classify(Stream::Stdout, &record);
        let mut events = grouper.feed(Stream::Stdout, record, category);
        if events.is_empty() {
            events.push(grouper.flush().expect("buffered group should flush"));
        }
        events.remove(0)
    }

    #[test]
    fn render_does_not_panic_for_every_category() {
        let mut renderer = Renderer::new(OutputMode::Classified, "1.21.1".to_string());
        renderer.render(
            &single_event(
                "[12:00:00] [Server thread/INFO]: Starting minecraft server version 1.21.1",
            ),
            RunPhase::ServerStartup,
        );
        renderer.render(
            &single_event("[12:00:01] [Server thread/INFO]: Done (1.0s)! For help, type \"help\""),
            RunPhase::ServerStartup,
        );
        renderer.render(
            &single_event("[12:00:02] [Server thread/INFO]: Toast joined the game"),
            RunPhase::Runtime,
        );
        renderer.render(
            &single_event("[12:00:03] [Server thread/WARN]: Can't keep up!"),
            RunPhase::Runtime,
        );
        renderer.render(
            &single_event(
                "[12:00:04] [Server thread/WARN]: Error loading function arcane:combat/dash",
            ),
            RunPhase::ServerStartup,
        );
        renderer.finish();
    }

    #[test]
    fn raw_mode_prints_raw_lines() {
        let mut renderer = Renderer::new(OutputMode::Raw, "1.21.1".to_string());
        renderer.render(
            &single_event(
                "[12:00:00] [Server thread/INFO]: Starting minecraft server version 1.21.1",
            ),
            RunPhase::ServerStartup,
        );
    }

    #[test]
    fn json_mode_only_emits_diagnostics() {
        let mut renderer = Renderer::new(OutputMode::Json, "1.21.1".to_string());
        // Non-diagnostic categories must not panic and must not print
        // terminal-formatted strings; we can't capture stdout here, but we
        // can at least assert this doesn't panic and behaves the same as a
        // diagnostic-producing line downstream.
        renderer.render(
            &single_event("[12:00:00] [Server thread/INFO]: Toast joined the game"),
            RunPhase::Runtime,
        );
        renderer.render(
            &single_event(
                "[12:00:04] [Server thread/WARN]: Error loading function arcane:combat/dash",
            ),
            RunPhase::ServerStartup,
        );
        renderer.finish();
    }

    #[test]
    fn json_diagnostic_serializes_with_stable_field_names() {
        let diag = build_diagnostic(
            &single_event(
                "[12:00:00] [Server thread/WARN]: Error loading function arcane:combat/dash",
            ),
            RunPhase::ServerStartup,
        )
        .unwrap();
        let json = serde_json::to_value(JsonDiagnostic::from(&diag)).unwrap();

        assert_eq!(json["phase"], "server_startup");
        assert_eq!(json["severity"], "warning");
        assert_eq!(json["code"], "command_parse_error");
        assert_eq!(json["fatal"], true);
        assert_eq!(json["resource"], "arcane:combat/dash");
        assert_eq!(json["subsystem"], "function");
        assert_eq!(
            json["file"],
            "dist/arcane/data/arcane/function/combat/dash.mcfunction"
        );
        assert!(json.get("raw").is_some());
    }

    #[test]
    fn ready_banner_is_neutral_when_health_already_degraded() {
        let mut renderer = Renderer::new(OutputMode::Classified, "26.2".to_string());
        renderer.render(
            &single_event(
                "[10:15:32] [Server thread/ERROR]: Failed to load function vanilla_plus:on_load",
            ),
            RunPhase::DatapackDiscovery,
        );
        assert_eq!(renderer.health(), RunHealth::Degraded);
        // Rendering the ready banner afterward must not reset health, and
        // must not panic while choosing the neutral (non-checkmark) form.
        renderer.render(
            &single_event(
                "[10:15:33] [Server thread/INFO]: Done (5.123s)! For help, type \"help\"",
            ),
            RunPhase::Runtime,
        );
        assert_eq!(renderer.health(), RunHealth::Degraded);
        renderer.finish();
    }
}
