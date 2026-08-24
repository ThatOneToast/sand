//! `sand build --explain-rebuild` (issue #347 Phase 8).
//!
//! Reports only decisions Sand itself can observe directly and truthfully:
//!
//! - whether the exporter binary's mtime changed across the `cargo build`
//!   invocation (Cargo rebuilt it) or stayed the same (Cargo reused the
//!   existing artifact);
//! - the datapack/resource-pack output manifest's written/unchanged/removed
//!   counts, which Sand computes itself in `output_manifest.rs` and already
//!   knows with certainty.
//!
//! This deliberately does **not** reimplement Cargo's fingerprint engine to
//! explain *why* Cargo chose to rebuild the exporter — Sand only observes
//! *that* it did, via the binary's mtime, and says so plainly rather than
//! fabricating a cause it doesn't actually know. It also does not yet cover
//! MC report/generated-code cache or API-manifest hit/miss reasons: those
//! caches (issue #347 Phases 3-5) are not implemented yet, so this report
//! says so explicitly instead of silently omitting them.

use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use super::output_manifest::ChangeSummary;

/// Whether the exporter binary's mtime changed across compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExporterOutcome {
    /// The binary didn't exist before this build (first build, or a clean
    /// target/), so "rebuilt" is the only meaningful answer.
    Rebuilt,
    /// The binary's mtime is unchanged: Cargo decided nothing in the
    /// exporter's dependency graph needed recompiling.
    Reused,
}

/// Observes whether `binary_path` was rebuilt by comparing its mtime before
/// and after `compile` runs. `compile` is expected to be the exporter
/// `cargo build` invocation itself.
pub fn observe_exporter_rebuild<T>(
    binary_path: &Path,
    compile: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<(T, ExporterOutcome)> {
    let before = mtime(binary_path);
    let result = compile()?;
    let after = mtime(binary_path);
    let outcome = match (before, after) {
        (Some(before), Some(after)) if before == after => ExporterOutcome::Reused,
        _ => ExporterOutcome::Rebuilt,
    };
    Ok((result, outcome))
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// One pack root's (datapack or resource pack) explain-rebuild report.
pub struct RebuildExplanation {
    pub exporter: ExporterOutcome,
    pub datapack: ChangeSummary,
    pub resourcepack: Option<ChangeSummary>,
}

impl RebuildExplanation {
    pub fn render(&self, out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out)?;
        writeln!(out, "sand build --explain-rebuild")?;
        writeln!(out)?;
        writeln!(out, "Exporter")?;
        match self.exporter {
            ExporterOutcome::Rebuilt => {
                writeln!(out, "  rebuilt: yes")?;
                writeln!(
                    out,
                    "  reason: Cargo rebuilt the exporter binary (its mtime changed). \
                     Sand does not reimplement Cargo's fingerprint engine, so it cannot \
                     say which specific source file or dependency caused this -- \
                     run `cargo build -v` in the project for that detail."
                )?;
            }
            ExporterOutcome::Reused => {
                writeln!(out, "  rebuilt: no")?;
                writeln!(
                    out,
                    "  reason: Cargo reused the existing exporter binary (its mtime \
                     did not change)."
                )?;
            }
        }
        writeln!(out)?;
        writeln!(out, "Datapack")?;
        render_change_summary(out, &self.datapack)?;
        if let Some(rp) = &self.resourcepack {
            writeln!(out)?;
            writeln!(out, "Resource pack")?;
            render_change_summary(out, rp)?;
        }
        writeln!(out)?;
        writeln!(
            out,
            "Note: Minecraft-report/generated-code caching and per-crate API-contract \
             manifest reuse (issue #347 Phases 3-5) are not implemented yet, so this \
             report cannot explain codegen or API-manifest hit/miss decisions."
        )?;
        Ok(())
    }

    pub fn print(&self) {
        let mut buf = Vec::new();
        if self.render(&mut buf).is_ok() {
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }
}

fn render_change_summary(out: &mut impl Write, summary: &ChangeSummary) -> std::io::Result<()> {
    writeln!(out, "  written: {}", summary.written)?;
    writeln!(out, "  unchanged: {}", summary.unchanged)?;
    writeln!(out, "  removed: {}", summary.removed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_binary_before_and_after_reports_rebuilt() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("does_not_exist");
        let (result, outcome) = observe_exporter_rebuild(&binary, || Ok(())).unwrap();
        assert_eq!(result, ());
        assert_eq!(outcome, ExporterOutcome::Rebuilt);
    }

    #[test]
    fn unchanged_mtime_reports_reused() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("sand_export");
        std::fs::write(&binary, b"stub").unwrap();
        let (_, outcome) = observe_exporter_rebuild(&binary, || Ok(())).unwrap();
        assert_eq!(outcome, ExporterOutcome::Reused);
    }

    #[test]
    fn changed_mtime_reports_rebuilt() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("sand_export");
        std::fs::write(&binary, b"stub").unwrap();
        let (_, outcome) = observe_exporter_rebuild(&binary, || {
            // Simulate Cargo rewriting the binary: sleep past typical
            // filesystem mtime resolution, then rewrite it.
            std::thread::sleep(std::time::Duration::from_millis(20));
            std::fs::write(&binary, b"rebuilt").unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(outcome, ExporterOutcome::Rebuilt);
    }

    #[test]
    fn render_reports_exporter_and_datapack_sections() {
        let explanation = RebuildExplanation {
            exporter: ExporterOutcome::Reused,
            datapack: ChangeSummary {
                written: 1,
                unchanged: 4,
                removed: 2,
            },
            resourcepack: None,
        };
        let mut buf = Vec::new();
        explanation.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();
        assert!(rendered.contains("Exporter"));
        assert!(rendered.contains("rebuilt: no"));
        assert!(rendered.contains("Datapack"));
        assert!(rendered.contains("written: 1"));
        assert!(rendered.contains("unchanged: 4"));
        assert!(rendered.contains("removed: 2"));
        assert!(!rendered.contains("Resource pack"));
    }

    #[test]
    fn render_includes_resourcepack_section_only_when_present() {
        let explanation = RebuildExplanation {
            exporter: ExporterOutcome::Rebuilt,
            datapack: ChangeSummary::default(),
            resourcepack: Some(ChangeSummary {
                written: 2,
                unchanged: 0,
                removed: 0,
            }),
        };
        let mut buf = Vec::new();
        explanation.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();
        assert!(rendered.contains("Resource pack"));
        assert!(rendered.contains("written: 2"));
    }

    #[test]
    fn render_never_fabricates_a_cargo_level_cause() {
        let explanation = RebuildExplanation {
            exporter: ExporterOutcome::Rebuilt,
            datapack: ChangeSummary::default(),
            resourcepack: None,
        };
        let mut buf = Vec::new();
        explanation.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();
        // Must be honest about not reimplementing Cargo's fingerprint
        // engine, not claim to know which file caused the rebuild.
        assert!(rendered.contains("does not reimplement Cargo's fingerprint engine"));
    }
}
