//! Exporter compilation and execution.
//!
//! `sand build` collects generated records by compiling and running exporter
//! binaries inside the user's project. The `sand_export` binary emits
//! [`super::records::ComponentRecord`]s for the datapack.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Exporter {
    Datapack,
}

impl Exporter {
    /// The `[[bin]]` target name in the user's `Cargo.toml`.
    pub(super) fn bin_name(self) -> &'static str {
        "sand_export"
    }

    /// Human-readable name used to attribute failures to one exporter.
    pub(super) fn label(self) -> &'static str {
        "datapack exporter"
    }
}

/// The exporter binaries this build needs.
///
/// Exporters are always compiled with Cargo's plain dev profile, regardless
/// of whether the overall Sand build is `sand build` or `sand build
/// --release`. Sand's "release" concept controls *packaging* (zip
/// generation, output semantics) — it is not a request for an optimized
/// exporter binary. Keeping one profile means `sand build` followed by
/// `sand build --release` reuses the same exporter compilation artifacts
/// instead of paying for a second dependency-graph compile under a
/// different Cargo artifact identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExportBuildPlan {
    exporter: Exporter,
}

/// Compiled exporter binary paths.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExportBinaries {
    pub(super) datapack: PathBuf,
}

impl ExportBuildPlan {
    /// Plans compilation of the datapack exporter.
    /// `sand build --release` semantics (zip packaging) live entirely outside
    /// this plan — see [`super::run`]. Exporter compilation itself never
    /// varies with Sand's release flag.
    pub(super) fn new() -> Self {
        Self {
            exporter: Exporter::Datapack,
        }
    }

    /// The exporters this plan compiles, in invocation order.
    #[cfg(test)]
    fn exporter(&self) -> Exporter {
        self.exporter
    }

    /// Arguments for the single `cargo build` invocation.
    ///
    /// `cargo build` accepts `--bin` repeatedly to select several binary
    /// targets from one package, which is what keeps this to one invocation.
    pub(super) fn cargo_args(&self) -> Vec<&'static str> {
        vec!["build", "--bin", self.exporter.bin_name()]
    }

    /// The command line as shown to the user in diagnostics.
    pub(super) fn command_line(&self) -> String {
        format!("cargo {}", self.cargo_args().join(" "))
    }

    /// Cargo's output subdirectory for the compiled profile.
    ///
    /// Always `debug`: exporters are always compiled with Cargo's plain dev
    /// profile (see the [`ExportBuildPlan`] docs), independent of `sand
    /// build --release`.
    pub(super) fn profile_dir(&self) -> &'static str {
        "debug"
    }

    /// Compiles every planned exporter with one `cargo build`.
    ///
    /// Sets only `SAND_MC_VERSION`, which deliberately selects the generated
    /// registry snapshot. It does not set `RUSTFLAGS` or another compiler flag:
    /// doing so used to split the artifact cache between `cargo
    /// build`/`cargo check` run directly and exporter compilation triggered
    /// by `sand build`, so equivalent work was paid for twice.
    pub(super) fn compile(&self, mc_version: &str, quiet: bool) -> Result<()> {
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(self.cargo_args())
            .env("SAND_MC_VERSION", mc_version);
        if quiet {
            cmd.arg("--message-format=json");
            let output = cmd
                .output()
                .with_context(|| format!("failed to invoke `{}`", self.command_line()))?;
            if !output.status.success() {
                bail!(
                    "`{}` failed:\n{}",
                    self.command_line(),
                    format!(
                        "{}\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    )
                    .trim()
                );
            }
        } else {
            let status = cmd
                .status()
                .with_context(|| format!("failed to invoke `{}`", self.command_line()))?;
            if !status.success() {
                bail!("`{}` failed", self.command_line());
            }
        }
        Ok(())
    }

    /// Resolves the compiled binary paths under Cargo's target directory.
    pub(super) fn binaries(&self, cargo_target_dir: &Path) -> ExportBinaries {
        ExportBinaries {
            datapack: cargo_target_dir
                .join(self.profile_dir())
                .join(self.exporter.bin_name()),
        }
    }
}

/// Runs one exporter and returns its stdout.
///
/// Failures are attributed to the datapack exporter.
pub(super) fn run_exporter(
    exporter: Exporter,
    binary: &Path,
    env: &[(&str, &str)],
) -> Result<Vec<u8>> {
    let mut cmd = std::process::Command::new(binary);
    for (key, value) in env {
        cmd.env(key, value);
    }
    // Cargo has just produced these executables. Some Unix filesystems can
    // briefly reject their first launch with ETXTBSY while the writer closes;
    // retry only that transient condition, keeping ordinary spawn errors
    // immediate and attributable to the selected exporter.
    let mut busy_retries = 0;
    let output = loop {
        match cmd.output() {
            Ok(output) => break output,
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy && busy_retries < 3 =>
            {
                busy_retries += 1;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to run {} '{}'", exporter.label(), binary.display())
                });
            }
        }
    };
    if !output.status.success() {
        bail!(
            "{} `{}` failed:\n{}",
            exporter.label(),
            exporter.bin_name(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_requests_the_datapack_exporter() {
        let plan = ExportBuildPlan::new();
        assert_eq!(plan.exporter(), Exporter::Datapack);
        assert_eq!(plan.cargo_args(), ["build", "--bin", "sand_export"]);
    }

    /// `sand build` and `sand build --release` are both driven through
    /// `ExportBuildPlan`, which has no notion of Sand's release flag at all.
    /// This is what guarantees the two Sand commands share one exporter
    /// compilation artifact identity: there is no `--release` Cargo arg to
    /// diverge on in the first place.
    #[test]
    fn plan_never_requests_cargos_release_profile() {
        let plan = ExportBuildPlan::new();
        assert!(!plan.cargo_args().contains(&"--release"));
        assert_eq!(plan.profile_dir(), "debug");
    }

    #[test]
    fn binary_paths_always_resolve_under_the_debug_profile_dir() {
        let target = Path::new("/tmp/custom-target");

        let binaries = ExportBuildPlan::new().binaries(target);
        assert_eq!(binaries.datapack, target.join("debug/sand_export"));
    }

    #[test]
    fn compile_failure_names_the_exact_cargo_command() {
        let plan = ExportBuildPlan::new();
        assert_eq!(plan.command_line(), "cargo build --bin sand_export");
    }

    // ── Exporter execution (unix: needs an executable stub) ────────────────────

    /// Writes an executable `/bin/sh` stub and returns its path.
    #[cfg(unix)]
    fn stub(dir: &Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt as _;

        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    #[test]
    fn datapack_exporter_receives_the_resolved_mc_version() {
        let temp = tempfile::tempdir().unwrap();
        let binary = stub(
            temp.path(),
            "sand_export",
            "printf '%s' \"$SAND_EXPORT_MC_VERSION\"",
        );

        let stdout = run_exporter(
            Exporter::Datapack,
            &binary,
            &[("SAND_EXPORT_MC_VERSION", "26.2")],
        )
        .unwrap();
        assert_eq!(String::from_utf8(stdout).unwrap(), "26.2");
    }

    #[cfg(unix)]
    #[test]
    fn datapack_exporter_failure_is_attributed_to_the_datapack_exporter() {
        let temp = tempfile::tempdir().unwrap();
        let binary = stub(temp.path(), "sand_export", "echo 'boom' >&2\nexit 1");

        let err = run_exporter(Exporter::Datapack, &binary, &[]).unwrap_err();
        let rendered = err.to_string();
        assert!(
            rendered.contains("datapack exporter") && rendered.contains("sand_export"),
            "failure must name the datapack exporter: {rendered}"
        );
        assert!(
            rendered.contains("boom"),
            "exporter stderr must not be suppressed: {rendered}"
        );
    }

    #[test]
    fn missing_exporter_binary_reports_which_exporter_could_not_run() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("does_not_exist");

        let err = run_exporter(Exporter::Datapack, &missing, &[]).unwrap_err();
        let rendered = format!("{err:#}");
        assert!(
            rendered.contains("datapack exporter") && rendered.contains("does_not_exist"),
            "missing binary must be attributed and named: {rendered}"
        );
    }
}
