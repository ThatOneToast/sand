//! Turns classified/grouped log events into what actually gets printed.
//!
//! Four output modes are supported (see [`OutputMode`]): `classified`
//! (default filtered/formatted output), `verbose` (classified output plus
//! the raw lines behind each event), `raw` (unfiltered passthrough, for
//! debugging the classifier itself), and `json` (structured diagnostics
//! only, one JSON object per line, for machine consumption).

use colored::Colorize;
use serde::Serialize;

use super::classify::Category;
use super::dedup::{Deduplicator, Observation, RepeatSummary};
use super::diagnostic::{Diagnostic, GroupedEvent, Severity, build_diagnostic};
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
    dedup: Deduplicator,
}

impl Renderer {
    pub fn new(mode: OutputMode, mc_version: String) -> Self {
        Self {
            mode,
            mc_version,
            printed_ready_banner: false,
            dedup: Deduplicator::new(),
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

    /// Flush any diagnostic run still open, so a trailing repeat count is
    /// never lost when the stream ends. Call once after the last `render`.
    pub fn finish(&mut self) {
        if let Some(summary) = self.dedup.flush() {
            self.print_repeat_summary(&summary);
        }
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
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

    fn render_ready(&mut self) {
        if self.printed_ready_banner {
            return;
        }
        self.printed_ready_banner = true;
        println!();
        println!(
            "{} Minecraft {} ready",
            "✓".green().bold(),
            self.mc_version.yellow()
        );
        println!("  Type server commands directly; use `stop` to exit.");
        println!();
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
                match (&diag.position, &diag.context) {
                    (Some(pos), Some(ctx)) => println!("  {pos}: {ctx}"),
                    (None, Some(ctx)) => println!("  {} {}", "context:".dimmed(), ctx),
                    _ => {}
                }
                println!("  {} {}", "reason:".dimmed(), diag.reason);
                if let Some(hint) = &diag.hint {
                    println!("  {} {}", "hint:".dimmed(), hint);
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
    resource: &'a Option<String>,
    subsystem: &'a Option<String>,
    file: &'a Option<String>,
    position: &'a Option<String>,
    context: &'a Option<String>,
    reason: &'a str,
    hint: &'a Option<String>,
    raw: &'a [String],
}

impl<'a> From<&'a Diagnostic> for JsonDiagnostic<'a> {
    fn from(diag: &'a Diagnostic) -> Self {
        Self {
            phase: diag.phase,
            severity: diag.severity,
            code: diag.code,
            resource: &diag.resource,
            subsystem: &diag.subsystem,
            file: &diag.file,
            position: &diag.position,
            context: &diag.context,
            reason: &diag.reason,
            hint: &diag.hint,
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
}
