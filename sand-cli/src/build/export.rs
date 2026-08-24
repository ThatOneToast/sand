//! Exporter compilation and execution.
//!
//! `sand build` collects generated records by compiling and running exporter
//! binaries inside the user's project. There are two of them: `sand_export`
//! emits [`super::records::ComponentRecord`]s for the datapack, and
//! `sand_resource_export` emits [`super::records::ResourcePackRecord`]s for the
//! resource pack.
//!
//! The two exporters stay separate *processes* with separate record streams —
//! only their compilation is coordinated. A datapack-only build never mentions
//! the resource exporter; a `--resourcepack` build compiles both binaries with
//! one `cargo build` invocation instead of paying Cargo's resolve/analysis cost
//! twice.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};

/// One of the two exporter binaries a Sand project can expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Exporter {
    Datapack,
    ResourcePack,
}

impl Exporter {
    /// The `[[bin]]` target name in the user's `Cargo.toml`.
    pub(super) fn bin_name(self) -> &'static str {
        match self {
            Exporter::Datapack => "sand_export",
            Exporter::ResourcePack => "sand_resource_export",
        }
    }

    /// Human-readable name used to attribute failures to one exporter.
    pub(super) fn label(self) -> &'static str {
        match self {
            Exporter::Datapack => "datapack exporter",
            Exporter::ResourcePack => "resource-pack exporter",
        }
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
    exporters: Vec<Exporter>,
}

/// Compiled exporter binary paths.
///
/// `resource_pack` is `Some` exactly when the plan included the resource
/// exporter, so a datapack-only build cannot accidentally run it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExportBinaries {
    pub(super) datapack: PathBuf,
    pub(super) resource_pack: Option<PathBuf>,
}

impl ExportBuildPlan {
    /// Plans compilation for a build. The datapack exporter is always needed;
    /// the resource exporter is added only for `--resourcepack`.
    ///
    /// `sand build --release` semantics (zip packaging) live entirely outside
    /// this plan — see [`super::run`]. Exporter compilation itself never
    /// varies with Sand's release flag.
    pub(super) fn new(resourcepack: bool) -> Self {
        let mut exporters = vec![Exporter::Datapack];
        if resourcepack {
            exporters.push(Exporter::ResourcePack);
        }
        Self { exporters }
    }

    /// The exporters this plan compiles, in invocation order.
    #[cfg(test)]
    fn exporters(&self) -> &[Exporter] {
        &self.exporters
    }

    /// Arguments for the single `cargo build` invocation.
    ///
    /// `cargo build` accepts `--bin` repeatedly to select several binary
    /// targets from one package, which is what keeps this to one invocation.
    pub(super) fn cargo_args(&self) -> Vec<&'static str> {
        let mut args = vec!["build"];
        for exporter in &self.exporters {
            args.push("--bin");
            args.push(exporter.bin_name());
        }
        args
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
    /// Intentionally does not set `RUSTFLAGS` or any other compiler flag that
    /// would change Cargo's fingerprint for the exporter dependency graph:
    /// doing so used to split the artifact cache between `cargo
    /// build`/`cargo check` run directly and exporter compilation triggered
    /// by `sand build`, so equivalent work was paid for twice.
    pub(super) fn compile(&self) -> Result<()> {
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(self.cargo_args());
        let status = cmd
            .status()
            .with_context(|| format!("failed to invoke `{}`", self.command_line()))?;
        if !status.success() {
            bail!("`{}` failed", self.command_line());
        }
        Ok(())
    }

    /// Resolves the compiled binary paths under Cargo's target directory.
    pub(super) fn binaries(&self, cargo_target_dir: &Path) -> ExportBinaries {
        let path = |exporter: Exporter| {
            cargo_target_dir
                .join(self.profile_dir())
                .join(exporter.bin_name())
        };
        ExportBinaries {
            datapack: path(Exporter::Datapack),
            resource_pack: self
                .exporters
                .contains(&Exporter::ResourcePack)
                .then(|| path(Exporter::ResourcePack)),
        }
    }
}

/// Runs one exporter and returns its stdout.
///
/// Failures are attributed to the specific exporter so a broken resource-pack
/// export is never reported as a datapack export failure, or vice versa.
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
    fn datapack_only_plan_requests_only_the_datapack_exporter() {
        let plan = ExportBuildPlan::new(false);
        assert_eq!(plan.exporters(), [Exporter::Datapack]);
        assert_eq!(plan.cargo_args(), ["build", "--bin", "sand_export"]);
        assert!(
            !plan.command_line().contains("sand_resource_export"),
            "datapack-only builds must not compile the resource exporter: {}",
            plan.command_line()
        );
    }

    #[test]
    fn resourcepack_plan_compiles_both_exporters_in_one_invocation() {
        let plan = ExportBuildPlan::new(true);
        assert_eq!(
            plan.exporters(),
            [Exporter::Datapack, Exporter::ResourcePack]
        );
        let args = plan.cargo_args();
        assert_eq!(
            args,
            [
                "build",
                "--bin",
                "sand_export",
                "--bin",
                "sand_resource_export"
            ]
        );
        // One invocation: exactly one `build` subcommand for two `--bin`s.
        assert_eq!(args.iter().filter(|a| **a == "build").count(), 1);
        assert_eq!(args.iter().filter(|a| **a == "--bin").count(), 2);
    }

    /// `sand build` and `sand build --release` are both driven through
    /// `ExportBuildPlan`, which has no notion of Sand's release flag at all.
    /// This is what guarantees the two Sand commands share one exporter
    /// compilation artifact identity: there is no `--release` Cargo arg to
    /// diverge on in the first place.
    #[test]
    fn plan_never_requests_cargos_release_profile() {
        let with_resourcepack = ExportBuildPlan::new(true);
        assert!(!with_resourcepack.cargo_args().contains(&"--release"));
        assert_eq!(with_resourcepack.profile_dir(), "debug");

        let datapack_only = ExportBuildPlan::new(false);
        assert!(!datapack_only.cargo_args().contains(&"--release"));
        assert_eq!(datapack_only.profile_dir(), "debug");
    }

    #[test]
    fn binary_paths_always_resolve_under_the_debug_profile_dir() {
        let target = Path::new("/tmp/custom-target");

        let binaries = ExportBuildPlan::new(true).binaries(target);
        assert_eq!(binaries.datapack, target.join("debug/sand_export"));
        assert_eq!(
            binaries.resource_pack,
            Some(target.join("debug/sand_resource_export"))
        );
    }

    #[test]
    fn datapack_only_plan_resolves_no_resource_binary() {
        let binaries = ExportBuildPlan::new(false).binaries(Path::new("/tmp/t"));
        assert_eq!(binaries.datapack, Path::new("/tmp/t/debug/sand_export"));
        assert_eq!(
            binaries.resource_pack, None,
            "datapack-only builds must not resolve a resource exporter to run"
        );
    }

    #[test]
    fn compile_failure_names_the_exact_cargo_command() {
        let plan = ExportBuildPlan::new(true);
        assert_eq!(
            plan.command_line(),
            "cargo build --bin sand_export --bin sand_resource_export"
        );
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
            &[("SAND_EXPORT_MC_VERSION", "1.21.4")],
        )
        .unwrap();
        assert_eq!(String::from_utf8(stdout).unwrap(), "1.21.4");
    }

    #[cfg(unix)]
    #[test]
    fn resource_exporter_runs_without_the_datapack_version_env() {
        let temp = tempfile::tempdir().unwrap();
        let binary = stub(
            temp.path(),
            "sand_resource_export",
            "printf 'version=[%s]' \"$SAND_EXPORT_MC_VERSION\"",
        );

        let stdout = run_exporter(Exporter::ResourcePack, &binary, &[]).unwrap();
        assert_eq!(String::from_utf8(stdout).unwrap(), "version=[]");
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
            !rendered.contains("resource-pack"),
            "datapack failure must not blame the resource exporter: {rendered}"
        );
        assert!(
            rendered.contains("boom"),
            "exporter stderr must not be suppressed: {rendered}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resource_exporter_failure_is_attributed_to_the_resource_exporter() {
        let temp = tempfile::tempdir().unwrap();
        let binary = stub(
            temp.path(),
            "sand_resource_export",
            "echo 'rp boom' >&2\nexit 3",
        );

        let err = run_exporter(Exporter::ResourcePack, &binary, &[]).unwrap_err();
        let rendered = err.to_string();
        assert!(
            rendered.contains("resource-pack exporter")
                && rendered.contains("sand_resource_export"),
            "failure must name the resource-pack exporter: {rendered}"
        );
        assert!(
            rendered.contains("rp boom"),
            "exporter stderr must not be suppressed: {rendered}"
        );
    }

    #[test]
    fn missing_exporter_binary_reports_which_exporter_could_not_run() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("does_not_exist");

        let err = run_exporter(Exporter::ResourcePack, &missing, &[]).unwrap_err();
        let rendered = format!("{err:#}");
        assert!(
            rendered.contains("resource-pack exporter") && rendered.contains("does_not_exist"),
            "missing binary must be attributed and named: {rendered}"
        );
    }
}
