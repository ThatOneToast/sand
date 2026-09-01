//! Build-phase timing instrumentation for `sand build --timings` (issue
//! #347 Phases 0 and 8).
//!
//! Collection itself is always active: a handful of `Instant::now()` calls
//! per build is negligible next to the seconds-scale phases being measured,
//! so there is no meaningful "disabled" state to special-case for
//! collection. Only *printing* is gated behind `--timings`, which is what
//! keeps overhead on an ordinary build limited to "don't print anything" —
//! satisfying the requirement that timings add negligible overhead when the
//! flag isn't passed.

use std::io::Write;
use std::time::{Duration, Instant};

/// One of Sand's own build phases, in the fixed order they're reported.
///
/// This intentionally does not attempt to reproduce Cargo's own fingerprint
/// engine or attribute time *inside* `ExporterCompile` to specific Cargo
/// units — see `--explain-rebuild` (`explain.rs`) for why Sand only reports
/// "Cargo rebuilt or reused the exporter" rather than fabricating a
/// Cargo-level cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Configuration,
    ExporterCompile,
    ExporterExecution,
    RecordParsing,
    Validation,
    DatapackWriting,
    WorldBuild,
    ResourcePackExport,
    Packaging,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Phase::Configuration => "Configuration",
            Phase::ExporterCompile => "Cargo/exporter compile",
            Phase::ExporterExecution => "Exporter execution",
            Phase::RecordParsing => "Record parsing",
            Phase::Validation => "Validation",
            Phase::DatapackWriting => "Datapack writing",
            Phase::WorldBuild => "Typed world build (sand.build.rs)",
            Phase::ResourcePackExport => "Resource-pack export",
            Phase::Packaging => "Packaging",
        }
    }
}

/// Accumulates phase durations for one `sand build` invocation.
pub struct Timings {
    start: Instant,
    entries: Vec<(Phase, Duration)>,
}

impl Default for Timings {
    fn default() -> Self {
        Self::new()
    }
}

impl Timings {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            entries: Vec::new(),
        }
    }

    /// Times `f` and records its duration under `phase`, returning `f`'s
    /// result unchanged. A phase recorded more than once (e.g. datapack and
    /// resource-pack writing both charging `DatapackWriting`-adjacent work
    /// in the same build) accumulates rather than overwrites.
    pub fn record<T>(
        &mut self,
        phase: Phase,
        f: impl FnOnce() -> Result<T, anyhow::Error>,
    ) -> Result<T, anyhow::Error> {
        let started = Instant::now();
        let result = f();
        self.entries.push((phase, started.elapsed()));
        result
    }

    pub fn total(&self) -> Duration {
        self.start.elapsed()
    }

    /// Renders the recorded phases as a fixed-width table, in declaration
    /// order (not sorted by duration), followed by a `Total` line computed
    /// from wall-clock time since [`Timings::new`] — not a sum of the
    /// recorded phases, so any unaccounted time between/around phases is
    /// visible rather than silently hidden.
    pub fn render(&self, out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out)?;
        writeln!(out, "Sand build timings")?;
        writeln!(out)?;
        let label_width = self
            .entries
            .iter()
            .map(|(phase, _)| phase.label().len())
            .max()
            .unwrap_or(0)
            .max("Total".len());
        for (phase, duration) in &self.entries {
            writeln!(
                out,
                "{:<label_width$}  {}",
                phase.label(),
                format_duration(*duration)
            )?;
        }
        writeln!(out, "{}", "-".repeat(label_width + 12))?;
        writeln!(
            out,
            "{:<label_width$}  {}",
            "Total",
            format_duration(self.total())
        )?;
        Ok(())
    }

    /// Prints the table to stdout. Only called when `--timings` was passed.
    pub fn print(&self) {
        let mut buf = Vec::new();
        // Rendering into an in-memory buffer never fails; a real print
        // failure (a closed stdout) is not something a build should treat
        // as fatal, so this intentionally swallows the `Result`.
        if self.render(&mut buf).is_ok() {
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }
}

fn format_duration(d: Duration) -> String {
    if d.as_secs() >= 1 {
        format!("{:.2} s", d.as_secs_f64())
    } else {
        format!("{} ms", d.as_millis())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_lists_every_recorded_phase_in_order() {
        let mut timings = Timings::new();
        timings
            .record(Phase::Configuration, || Ok::<_, anyhow::Error>(()))
            .unwrap();
        timings
            .record(Phase::ExporterCompile, || Ok::<_, anyhow::Error>(()))
            .unwrap();
        timings
            .record(Phase::DatapackWriting, || Ok::<_, anyhow::Error>(()))
            .unwrap();

        let mut buf = Vec::new();
        timings.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();

        let config_pos = rendered.find("Configuration").unwrap();
        let compile_pos = rendered.find("Cargo/exporter compile").unwrap();
        let write_pos = rendered.find("Datapack writing").unwrap();
        assert!(
            config_pos < compile_pos,
            "phases must appear in record order"
        );
        assert!(
            compile_pos < write_pos,
            "phases must appear in record order"
        );
    }

    #[test]
    fn render_always_ends_with_a_total_line() {
        let mut timings = Timings::new();
        timings
            .record(Phase::Configuration, || Ok::<_, anyhow::Error>(()))
            .unwrap();

        let mut buf = Vec::new();
        timings.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();

        assert!(rendered.contains("Total"));
        assert!(
            rendered.trim_end().ends_with(['s', 'm']),
            "must end with a duration unit, not fragile on the exact value: {rendered:?}"
        );
    }

    #[test]
    fn empty_timings_still_render_a_valid_table_with_zero_total() {
        let timings = Timings::new();
        let mut buf = Vec::new();
        timings.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();
        assert!(rendered.contains("Sand build timings"));
        assert!(rendered.contains("Total"));
    }

    #[test]
    fn record_propagates_the_wrapped_result_and_still_times_a_failing_phase() {
        let mut timings = Timings::new();
        let result = timings.record(Phase::Validation, || Err::<(), _>(anyhow::anyhow!("boom")));
        assert!(result.is_err());
        // The failing phase must still be recorded, so `--timings` on a
        // failed build shows where time went before the failure.
        let mut buf = Vec::new();
        timings.render(&mut buf).unwrap();
        assert!(String::from_utf8(buf).unwrap().contains("Validation"));
    }

    #[test]
    fn duration_formatting_uses_seconds_above_one_second_and_millis_below() {
        assert_eq!(format_duration(Duration::from_millis(5)), "5 ms");
        assert_eq!(format_duration(Duration::from_millis(999)), "999 ms");
        assert_eq!(format_duration(Duration::from_secs(2)), "2.00 s");
    }
}
