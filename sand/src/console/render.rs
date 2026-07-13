//! Turns classified/grouped log events into what actually gets printed.

use colored::Colorize;

use super::classify::Category;
use super::diagnostic::{Diagnostic, GroupedEvent, Severity, build_diagnostic};

/// Prints [`GroupedEvent`]s to stdout, either as filtered/formatted Sand
/// output or, in verbose mode, as the raw underlying lines.
pub struct Renderer {
    verbose: bool,
    mc_version: String,
    printed_ready_banner: bool,
}

impl Renderer {
    pub fn new(verbose: bool, mc_version: String) -> Self {
        Self {
            verbose,
            mc_version,
            printed_ready_banner: false,
        }
    }

    pub fn render(&mut self, event: &GroupedEvent) {
        if self.verbose {
            for line in &event.lines {
                println!("{}", line.raw);
            }
            return;
        }

        match event.category {
            Category::Suppress => {}
            Category::Ready => self.render_ready(),
            Category::Shutdown => println!("{}", message(event).yellow()),
            Category::PlayerEvent => println!("{}", message(event)),
            Category::CommandResponse => println!("{}", message(event)),
            Category::Warning => println!("{} {}", "warning:".yellow().bold(), message(event)),
            Category::Visible => println!("{}", message(event)),
            Category::DatapackError | Category::FatalError => match build_diagnostic(event) {
                Some(diag) => self.render_diagnostic(&diag),
                None => println!("{} {}", "error:".red().bold(), message(event)),
            },
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

    fn render_diagnostic(&self, diag: &Diagnostic) {
        let label = match diag.severity {
            Severity::Error => "datapack error".red().bold(),
            Severity::Warning => "datapack warning".yellow().bold(),
        };
        let resource = diag.resource.as_deref().unwrap_or("<unknown resource>");
        println!("{label}: failed to load {resource}");

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
        println!();
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
        let mut renderer = Renderer::new(false, "1.21.1".to_string());
        renderer.render(&single_event(
            "[12:00:00] [Server thread/INFO]: Starting minecraft server version 1.21.1",
        ));
        renderer.render(&single_event(
            "[12:00:01] [Server thread/INFO]: Done (1.0s)! For help, type \"help\"",
        ));
        renderer.render(&single_event(
            "[12:00:02] [Server thread/INFO]: Toast joined the game",
        ));
        renderer.render(&single_event(
            "[12:00:03] [Server thread/WARN]: Can't keep up!",
        ));
        renderer.render(&single_event(
            "[12:00:04] [Server thread/WARN]: Error loading function arcane:combat/dash",
        ));
    }

    #[test]
    fn verbose_mode_prints_raw_lines() {
        let mut renderer = Renderer::new(true, "1.21.1".to_string());
        renderer.render(&single_event(
            "[12:00:00] [Server thread/INFO]: Starting minecraft server version 1.21.1",
        ));
    }
}
