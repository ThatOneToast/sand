//! `sand build --explain-rebuild` reporting.
//!
//! Sand observes the datapack exporter binary's modification time around
//! Cargo compilation and reports whether Cargo rebuilt or reused it. Output
//! changes come from the deterministic output manifest.

use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use super::output_manifest::ChangeSummary;

/// Whether the datapack exporter binary changed across compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExporterOutcome {
    Rebuilt,
    Reused,
}

/// Datapack exporter outcome for one Cargo invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExporterRebuildOutcome {
    pub datapack: ExporterOutcome,
}

/// Observe whether Cargo rebuilt the datapack exporter.
pub fn observe_exporter_rebuild<T>(
    datapack_binary: &Path,
    compile: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<(T, ExporterRebuildOutcome)> {
    let before = mtime(datapack_binary);
    let result = compile()?;
    let after = mtime(datapack_binary);
    Ok((
        result,
        ExporterRebuildOutcome {
            datapack: outcome_from(before, after),
        },
    ))
}

fn outcome_from(before: Option<SystemTime>, after: Option<SystemTime>) -> ExporterOutcome {
    match (before, after) {
        (Some(before), Some(after)) if before == after => ExporterOutcome::Reused,
        _ => ExporterOutcome::Rebuilt,
    }
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// One build's explain-rebuild report.
pub struct RebuildExplanation {
    pub exporter: ExporterRebuildOutcome,
    pub datapack: ChangeSummary,
}

impl RebuildExplanation {
    pub fn render(&self, out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out, "\nsand build --explain-rebuild\n")?;
        writeln!(out, "Exporter")?;
        render_exporter_outcome(out, "datapack", self.exporter.datapack)?;
        writeln!(out, "\nDatapack")?;
        render_change_summary(out, &self.datapack)?;
        writeln!(
            out,
            "\nNote: generated-Rust codegen caching runs inside Cargo's build script; +             Cargo's own output is authoritative for that cache."
        )
    }

    pub fn print(&self) {
        let mut output = Vec::new();
        if self.render(&mut output).is_ok() {
            print!("{}", String::from_utf8_lossy(&output));
        }
    }
}

fn render_exporter_outcome(
    out: &mut impl Write,
    label: &str,
    outcome: ExporterOutcome,
) -> std::io::Result<()> {
    let value = match outcome {
        ExporterOutcome::Rebuilt => "rebuilt",
        ExporterOutcome::Reused => "reused",
    };
    writeln!(out, "  {label}: {value}")
}

fn render_change_summary(out: &mut impl Write, summary: &ChangeSummary) -> std::io::Result<()> {
    writeln!(out, "  written:   {}", summary.written)?;
    writeln!(out, "  unchanged: {}", summary.unchanged)?;
    writeln!(out, "  removed:   {}", summary.removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observes_reuse_and_renders_manifest_changes() {
        let temp = tempfile::tempdir().unwrap();
        let binary = temp.path().join("sand_export");
        std::fs::write(&binary, "stable").unwrap();
        let (_, outcome) = observe_exporter_rebuild(&binary, || Ok(())).unwrap();
        assert_eq!(outcome.datapack, ExporterOutcome::Reused);

        let report = RebuildExplanation {
            exporter: outcome,
            datapack: ChangeSummary {
                written: 1,
                unchanged: 2,
                removed: 3,
            },
        };
        let mut output = Vec::new();
        report.render(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("datapack: reused"));
        assert!(output.contains("written:   1"));
        assert!(output.contains("removed:   3"));
    }
}
