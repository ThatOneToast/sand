//! `sand build --explain-rebuild` (issue #347 Phase 8).
//!
//! Reports only decisions Sand itself can observe directly and truthfully:
//!
//! - whether *each* exporter binary's mtime changed across the `cargo
//!   build` invocation (Cargo rebuilt it) or stayed the same (Cargo reused
//!   the existing artifact) -- `sand_export` and, when `--resourcepack` was
//!   requested, `sand_resource_export` independently, since one `cargo
//!   build` invocation compiling both binaries does not imply both were
//!   actually recompiled;
//! - the datapack/resource-pack output manifest's written/unchanged/removed
//!   counts, which Sand computes itself in `output_manifest.rs` and already
//!   knows with certainty.
//!
//! This deliberately does **not** reimplement Cargo's fingerprint engine to
//! explain *why* Cargo chose to rebuild an exporter — Sand only observes
//! *that* it did, via the binary's mtime, and says so plainly rather than
//! fabricating a cause it doesn't actually know.
//!
//! Generated-Rust codegen caching (`sand-build/src/codegen/cache.rs`) and
//! its content-hash validation *are* implemented (issue #347 Phase 3), but
//! their hit/miss decision happens inside the `cargo build` subprocess's
//! own build script (`sand-core/build.rs`), not in this process -- Sand
//! does not currently pipe that decision back out of the subprocess as
//! structured data, so this report does not claim a specific hit/miss for
//! it. The subprocess's own `cargo:warning` output (visible in the normal
//! `cargo build` output above this report) is the source of truth for that
//! today; threading it through as a structured field here is tracked as
//! follow-up. What this module must never do is claim generated-code
//! caching *doesn't exist* -- it does, and it is exercised on every
//! `sand build`.

use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use super::output_manifest::ChangeSummary;

/// Whether one exporter binary's mtime changed across compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExporterOutcome {
    /// The binary didn't exist before this build (first build, or a clean
    /// target/), so "rebuilt" is the only meaningful answer.
    Rebuilt,
    /// The binary's mtime is unchanged: Cargo decided nothing in that
    /// binary's dependency graph needed recompiling.
    Reused,
}

/// Per-binary rebuild outcomes for one `cargo build` invocation. `datapack`
/// is always populated; `resourcepack` is `Some` exactly when
/// `--resourcepack` was requested, mirroring [`super::export::ExportBinaries`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExporterRebuildOutcome {
    pub datapack: ExporterOutcome,
    pub resourcepack: Option<ExporterOutcome>,
}

/// Observes whether `datapack_binary` and (if present) `resourcepack_binary`
/// were rebuilt, by comparing each one's mtime before and after `compile`
/// runs. `compile` is expected to be the single exporter `cargo build`
/// invocation that may produce either or both binaries -- one Cargo
/// invocation compiling both does not mean both were actually recompiled
/// (e.g. only the datapack exporter's source changed), so each binary's
/// mtime is watched independently rather than inferring one from the
/// other.
pub fn observe_exporter_rebuild<T>(
    datapack_binary: &Path,
    resourcepack_binary: Option<&Path>,
    compile: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<(T, ExporterRebuildOutcome)> {
    let dp_before = mtime(datapack_binary);
    let rp_before = resourcepack_binary.map(mtime);

    let result = compile()?;

    let dp_after = mtime(datapack_binary);
    let rp_after = resourcepack_binary.map(mtime);

    let outcome = ExporterRebuildOutcome {
        datapack: outcome_from(dp_before, dp_after),
        resourcepack: resourcepack_binary
            .map(|_| outcome_from(rp_before.flatten(), rp_after.flatten())),
    };
    Ok((result, outcome))
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
    pub resourcepack: Option<ChangeSummary>,
}

impl RebuildExplanation {
    pub fn render(&self, out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out)?;
        writeln!(out, "sand build --explain-rebuild")?;
        writeln!(out)?;
        writeln!(out, "Exporter")?;
        render_exporter_outcome(out, "datapack", self.exporter.datapack)?;
        if let Some(rp_outcome) = self.exporter.resourcepack {
            render_exporter_outcome(out, "resource pack", rp_outcome)?;
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
            "Note: generated-Rust codegen caching is implemented (issue #347 Phase 3) and \
             runs on every build, but its hit/miss decision happens inside the `cargo build` \
             subprocess (sand-core's build script) and is not yet threaded through to this \
             report as structured data -- see that subprocess's own `cargo:warning` output, \
             printed above, for cache-population diagnostics. Per-crate API-contract manifest \
             reuse (issue #347 Phase 5) status is not yet covered by this report either."
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

fn render_exporter_outcome(
    out: &mut impl Write,
    label: &str,
    outcome: ExporterOutcome,
) -> std::io::Result<()> {
    match outcome {
        ExporterOutcome::Rebuilt => {
            writeln!(out, "  {label}: rebuilt")?;
            writeln!(
                out,
                "    reason: Cargo rebuilt this binary (its mtime changed). Sand does not \
                 reimplement Cargo's fingerprint engine, so it cannot say which specific \
                 source file or dependency caused this -- run `cargo build -v` in the \
                 project for that detail."
            )?;
        }
        ExporterOutcome::Reused => {
            writeln!(out, "  {label}: reused")?;
            writeln!(
                out,
                "    reason: Cargo reused the existing binary (its mtime did not change)."
            )?;
        }
    }
    Ok(())
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
        let (result, outcome) = observe_exporter_rebuild(&binary, None, || Ok(())).unwrap();
        assert_eq!(result, ());
        assert_eq!(outcome.datapack, ExporterOutcome::Rebuilt);
        assert_eq!(outcome.resourcepack, None);
    }

    #[test]
    fn unchanged_mtime_reports_reused() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("sand_export");
        std::fs::write(&binary, b"stub").unwrap();
        let (_, outcome) = observe_exporter_rebuild(&binary, None, || Ok(())).unwrap();
        assert_eq!(outcome.datapack, ExporterOutcome::Reused);
    }

    #[test]
    fn changed_mtime_reports_rebuilt() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("sand_export");
        std::fs::write(&binary, b"stub").unwrap();
        let (_, outcome) = observe_exporter_rebuild(&binary, None, || {
            // Simulate Cargo rewriting the binary: sleep past typical
            // filesystem mtime resolution, then rewrite it.
            std::thread::sleep(std::time::Duration::from_millis(20));
            std::fs::write(&binary, b"rebuilt").unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(outcome.datapack, ExporterOutcome::Rebuilt);
    }

    /// The core fix for review item 4: one `cargo build` compiling both
    /// exporters must not report the resource-pack exporter's status by
    /// inferring it from the datapack exporter's mtime. Here only the
    /// datapack binary changes; the resource-pack binary must independently
    /// report `Reused`.
    #[test]
    fn datapack_and_resourcepack_outcomes_are_tracked_independently() {
        let dir = tempfile::tempdir().unwrap();
        let dp_binary = dir.path().join("sand_export");
        let rp_binary = dir.path().join("sand_resource_export");
        std::fs::write(&dp_binary, b"stub").unwrap();
        std::fs::write(&rp_binary, b"stub").unwrap();

        let (_, outcome) = observe_exporter_rebuild(&dp_binary, Some(&rp_binary), || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            // Only the datapack binary is rewritten by this simulated build.
            std::fs::write(&dp_binary, b"rebuilt").unwrap();
            Ok(())
        })
        .unwrap();

        assert_eq!(outcome.datapack, ExporterOutcome::Rebuilt);
        assert_eq!(outcome.resourcepack, Some(ExporterOutcome::Reused));
    }

    #[test]
    fn both_exporters_rebuilt_are_both_reported_as_rebuilt() {
        let dir = tempfile::tempdir().unwrap();
        let dp_binary = dir.path().join("sand_export");
        let rp_binary = dir.path().join("sand_resource_export");
        std::fs::write(&dp_binary, b"stub").unwrap();
        std::fs::write(&rp_binary, b"stub").unwrap();

        let (_, outcome) = observe_exporter_rebuild(&dp_binary, Some(&rp_binary), || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            std::fs::write(&dp_binary, b"rebuilt").unwrap();
            std::fs::write(&rp_binary, b"rebuilt").unwrap();
            Ok(())
        })
        .unwrap();

        assert_eq!(outcome.datapack, ExporterOutcome::Rebuilt);
        assert_eq!(outcome.resourcepack, Some(ExporterOutcome::Rebuilt));
    }

    #[test]
    fn render_reports_exporter_and_datapack_sections() {
        let explanation = RebuildExplanation {
            exporter: ExporterRebuildOutcome {
                datapack: ExporterOutcome::Reused,
                resourcepack: None,
            },
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
        assert!(rendered.contains("datapack: reused"));
        assert!(!rendered.contains("resource pack:"));
        assert!(rendered.contains("Datapack"));
        assert!(rendered.contains("written: 1"));
        assert!(rendered.contains("unchanged: 4"));
        assert!(rendered.contains("removed: 2"));
        assert!(!rendered.contains("Resource pack"));
    }

    #[test]
    fn render_includes_resourcepack_lines_only_when_requested() {
        let explanation = RebuildExplanation {
            exporter: ExporterRebuildOutcome {
                datapack: ExporterOutcome::Rebuilt,
                resourcepack: Some(ExporterOutcome::Reused),
            },
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
        assert!(rendered.contains("datapack: rebuilt"));
        assert!(rendered.contains("resource pack: reused"));
        assert!(rendered.contains("Resource pack"));
        assert!(rendered.contains("written: 2"));
    }

    #[test]
    fn render_never_fabricates_a_cargo_level_cause() {
        let explanation = RebuildExplanation {
            exporter: ExporterRebuildOutcome {
                datapack: ExporterOutcome::Rebuilt,
                resourcepack: None,
            },
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

    /// Review item 4: the report must never claim generated-code caching
    /// doesn't exist -- it does (Phase 3). It's fine for it to say the
    /// hit/miss *decision* isn't surfaced as structured data yet.
    #[test]
    fn render_does_not_claim_generated_code_caching_is_unimplemented() {
        let explanation = RebuildExplanation {
            exporter: ExporterRebuildOutcome {
                datapack: ExporterOutcome::Reused,
                resourcepack: None,
            },
            datapack: ChangeSummary::default(),
            resourcepack: None,
        };
        let mut buf = Vec::new();
        explanation.render(&mut buf).unwrap();
        let rendered = String::from_utf8(buf).unwrap();
        assert!(
            !rendered.to_lowercase().contains("caching") || rendered.contains("is implemented"),
            "must not describe generated-code caching as unimplemented: {rendered}"
        );
        assert!(rendered.contains("generated-Rust codegen caching is implemented"));
    }
}
