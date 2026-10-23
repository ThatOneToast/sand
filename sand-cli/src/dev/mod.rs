//! Local development orchestration; no framework or release-output changes.
mod rcon;
mod reload;
mod watch;

use crate::console::classify::Category;
use anyhow::{Context, Result, bail, ensure};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use watch::{Snapshot, Watch};

/// Inputs observed before the initial build, including a restart's build.
pub(crate) struct BuildInputs(Snapshot, PathBuf);

impl BuildInputs {
    pub(crate) fn capture() -> Result<Self> {
        let target = crate::build::cargo_target_dir()?;
        Ok(Self(
            watch::snapshot_excluding(&std::env::current_dir()?, std::slice::from_ref(&target))?,
            target,
        ))
    }
}

pub(crate) struct Session {
    root: PathBuf,
    namespace: String,
    profile: String,
    properties: PathBuf,
    original_session_settings: String,
    rcon: rcon::Rcon,
    watcher: Watch,
    target_directories: Vec<PathBuf>,
    last_poll: Instant,
    ready: bool,
    build: Option<(Child, Instant)>,
    restart_inputs: Snapshot,
    previous_report: Option<serde_json::Value>,
    reload_number: usize,
    pending_reload: Option<reload::Reload>,
    // Failures outside managed reloads (especially startup) remain sticky.
    persistent_health: crate::console::health::HealthTracker,
    reload_failed: bool,
}

impl Session {
    pub(crate) fn new(
        config: &crate::config::SandConfig,
        profile: &str,
        properties: &Path,
        inputs: BuildInputs,
    ) -> Result<Self> {
        Self::at(
            std::env::current_dir()?,
            config.pack.namespace.as_str().to_owned(),
            profile,
            properties,
            inputs,
        )
    }

    fn at(
        root: PathBuf,
        namespace: String,
        profile: &str,
        properties: &Path,
        inputs: BuildInputs,
    ) -> Result<Self> {
        let mut watcher = Watch::new(inputs.0);
        watcher.observe(
            watch::snapshot_excluding(&root, std::slice::from_ref(&inputs.1))?,
            Instant::now(),
        );
        let restart_inputs = restart_inputs(&root, &namespace)?;
        let existing = std::fs::read_to_string(properties)?;
        let game_port = existing
            .lines()
            .filter(|line| property_key(line) == "server-port")
            .map(|line| {
                line.trim_start()["server-port".len()..]
                    .trim_start_matches(['=', ':', ' ', '\t'])
                    .trim()
                    .parse::<u16>()
            })
            .next_back()
            .transpose()
            .context("invalid server-port in server.properties")?
            .unwrap_or(25565);
        let rcon = rcon::Rcon::new(game_port)?;
        let original_session_settings = existing
            .lines()
            .filter(|line| matches!(property_key(line), "pause-when-empty-seconds" | "server-ip"))
            .map(|line| format!("{line}\n"))
            .collect();
        // A final Java-properties continuation must not swallow server-ip.
        let text = format!(
            "{}\n# Sand development session\n{}pause-when-empty-seconds=-1\n",
            without_rcon(&existing),
            rcon.properties()
        );
        private_write(properties, text.as_bytes())?;
        eprintln!(
            "dev: loopback only; r + Enter rebuilds, R + Enter restarts, q + Enter quits. Reload re-runs load and join handlers."
        );
        Ok(Self {
            root,
            namespace,
            profile: profile.into(),
            properties: properties.into(),
            original_session_settings,
            rcon,
            watcher,
            target_directories: vec![inputs.1],
            last_poll: Instant::now(),
            ready: false,
            build: None,
            restart_inputs,
            previous_report: None,
            reload_number: 0,
            pending_reload: None,
            persistent_health: crate::console::health::HealthTracker::new(),
            reload_failed: false,
        })
    }

    pub(crate) fn is_reload_event(&self, event: &crate::console::diagnostic::GroupedEvent) -> bool {
        self.pending_reload
            .as_ref()
            .is_some_and(|pending| pending.includes(event.received_at))
    }

    pub(crate) fn event_phase(
        &self,
        event: &crate::console::diagnostic::GroupedEvent,
        current: crate::console::phase::RunPhase,
    ) -> crate::console::phase::RunPhase {
        use crate::console::phase::RunPhase;
        if self.is_reload_event(event) {
            RunPhase::Reload
        } else if self.pending_reload.is_some() {
            // Managed reloads start only from a ready server. A buffered
            // earlier headline remains runtime feedback even if a later
            // discovery line has already advanced the console phase.
            RunPhase::Runtime
        } else {
            current
        }
    }

    pub(crate) fn health(&self) -> crate::console::RunHealth {
        use crate::console::RunHealth;
        match self.persistent_health.current() {
            RunHealth::Healthy if self.reload_failed => RunHealth::Degraded,
            health => health,
        }
    }

    pub(crate) fn observe(
        &mut self,
        event: &crate::console::diagnostic::GroupedEvent,
        phase: crate::console::phase::RunPhase,
    ) {
        use crate::console::diagnostic::{Fatality, build_diagnostic};
        let phase = self.event_phase(event, phase);
        let is_reload = self.is_reload_event(event);
        let mut category = event.category;
        let message = &event.headline().message;
        if let Some(diagnostic) = build_diagnostic(event, phase) {
            match diagnostic.fatality() {
                Fatality::Nonfatal => category = Category::Warning,
                Fatality::FatalToDatapackHealth if is_reload => self.reload_failed = true,
                _ => self.persistent_health.observe(&diagnostic),
            }
        }
        if category == Category::Ready {
            self.ready = true;
        }
        if category == Category::Shutdown {
            self.ready = false;
        }
        if is_reload
            && let Some(pending) = self.pending_reload.as_mut()
            && let Some(outcome) = pending.observe(category, message)
        {
            self.finish_reload(outcome);
        }
    }

    fn finish_reload(&mut self, outcome: reload::Outcome) {
        self.pending_reload = None;
        self.reload_failed = !matches!(outcome, reload::Outcome::Complete { .. });
        match outcome {
            reload::Outcome::Complete { warnings } => eprintln!(
                "dev reload #{} complete ({warnings} warnings)",
                self.reload_number
            ),
            reload::Outcome::Failed {
                diagnostics,
                warnings,
            } => eprintln!(
                "dev reload #{} failed ({diagnostics} diagnostics, {warnings} warnings); see console",
                self.reload_number
            ),
            reload::Outcome::TimedOut => eprintln!(
                "dev reload #{} timed out waiting for load completion; press r to retry or R to restart",
                self.reload_number
            ),
        }
    }

    pub(crate) fn rebuild(&mut self) {
        self.watcher.request(Instant::now());
    }

    pub(crate) fn poll(&mut self) -> Result<()> {
        self.poll_with_cleanup(|path| std::fs::remove_dir_all(path))
    }

    fn poll_with_cleanup(
        &mut self,
        cleanup: impl FnOnce(&Path) -> std::io::Result<()>,
    ) -> Result<()> {
        if !self.ready {
            return Ok(());
        }
        let now = Instant::now();
        if let Some(outcome) = self
            .pending_reload
            .as_ref()
            .and_then(|pending| pending.timeout(now))
        {
            self.finish_reload(outcome);
        }
        if now.duration_since(self.last_poll) >= Duration::from_millis(250) {
            self.last_poll = now;
            self.watcher.observe(
                watch::snapshot_excluding(&self.root, &self.target_directories)?,
                now,
            );
        }
        if let Some((child, _)) = self.build.as_mut()
            && let Some(status) = child.try_wait()?
        {
            let (_, start) = self.build.take().expect("build exists");
            let bytes = std::fs::read(self.build_log())?;
            let result: serde_json::Value = serde_json::from_slice(&bytes)
                .context("invalid sand build JSON; see dist/.sand-dev-build.json")?;
            if !status.success() || result["success"] != true {
                eprintln!(
                    "dev: build failed after {:.2}s; live pack unchanged\n{}",
                    start.elapsed().as_secs_f64(),
                    result["diagnostics"]
                );
                return Ok(());
            }
            ensure!(
                result["namespace"] == self.namespace,
                "namespace changed: restart required; press R + Enter"
            );
            if restart_inputs(&self.root, &self.namespace)? != self.restart_inputs {
                eprintln!(
                    "dev: restart required — startup resources or configuration changed; live pack unchanged. Press R + Enter to restart (world reset policy applies)."
                );
                return Ok(());
            }
            let pending = reload::Reload::new()?;
            let installed = install_with_cleanup(
                &self.root.join("dist").join(&self.namespace),
                &self
                    .root
                    .join("dist/server/world/datapacks")
                    .join(&self.namespace),
                Some((&self.namespace, &pending.marker)),
                cleanup,
            )?;
            self.reload_number += 1;
            eprintln!(
                "dev build {:.2}s: {}; sync {} copied, {} skipped, {} pruned",
                start.elapsed().as_secs_f64(),
                result["datapack_changes"],
                installed.stats.copied,
                installed.stats.skipped,
                installed.stats.pruned
            );
            report_change(self.previous_report.as_ref(), &result["runtime_report"]);
            self.previous_report = Some(result["runtime_report"].clone());
            self.pending_reload = Some(pending);
            self.reload_failed = true;
            let pending = self.pending_reload.as_mut().expect("installed reload");
            match self.rcon.reload(|| pending.dispatching(Instant::now())) {
                Ok(()) => {
                    self.pending_reload
                        .as_mut()
                        .expect("installed reload")
                        .dispatched(Instant::now());
                    eprintln!("dev reload #{} pending…", self.reload_number);
                }
                Err(error) => {
                    self.pending_reload = None;
                    eprintln!(
                        "dev reload #{} failed: {error:#}; new pack is on disk, press r to retry or R to restart",
                        self.reload_number
                    );
                }
            }
        }
        if self.ready
            && self.build.is_none()
            && self.pending_reload.is_none()
            && self.watcher.take_ready(now)
        {
            self.refresh_target_directory(crate::build::cargo_target_dir_at(&self.root)?)?;
            let output = std::fs::File::create(self.build_log())?;
            let child = Command::new(std::env::current_exe()?)
                .args(["build", "--format", "json", "--profile", &self.profile])
                .current_dir(&self.root)
                .stdin(Stdio::null())
                .stdout(output)
                .stderr(Stdio::inherit())
                .spawn()
                .context("failed to start sand build")?;
            self.build = Some((child, Instant::now()));
            eprintln!("dev: building ({})…", self.profile);
        }
        Ok(())
    }
    fn refresh_target_directory(&mut self, target: PathBuf) -> Result<()> {
        if !self.target_directories.contains(&target) {
            self.target_directories.push(target);
            // The imminent build consumes current inputs. Keep all prior output
            // roots excluded, and observe edits made after this baseline normally.
            self.watcher = Watch::new(watch::snapshot_excluding(
                &self.root,
                &self.target_directories,
            )?);
        }
        Ok(())
    }

    fn build_log(&self) -> PathBuf {
        self.root.join("dist/.sand-dev-build.json")
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some((mut child, _)) = self.build.take() {
            // Reap the CLI build worker. Cargo descendants can finish their
            // cache writes, but cannot install a pack or issue a reload.
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Ok(text) = std::fs::read_to_string(&self.properties) {
            let cleaned = format!(
                "{}enable-rcon=false\n{}",
                without_rcon(&text),
                self.original_session_settings
            );
            if private_write(&self.properties, cleaned.as_bytes()).is_err() {
                eprintln!("dev: could not clear local RCON settings in server.properties");
            }
        }
    }
}

fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        options.mode(0o600);
        if path.exists() {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
    }
    options.open(path)?.write_all(bytes)?;
    Ok(())
}

fn without_rcon(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let key = property_key(line);
        if !matches!(
            key,
            "server-ip"
                | "enable-rcon"
                | "rcon.port"
                | "rcon.password"
                | "broadcast-rcon-to-ops"
                | "pause-when-empty-seconds"
        ) {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn property_key(line: &str) -> &str {
    line.trim_start()
        .split(['=', ':', ' ', '\t'])
        .next()
        .unwrap_or("")
}

/// Conservatively refuse updates to anything except known reloadable data.
/// Registry/worldgen resources and pack metadata remain launch-time inputs.
fn restart_inputs(root: &Path, namespace: &str) -> Result<Snapshot> {
    let mut inputs = watch::snapshot(&root.join("dist").join(namespace))?;
    inputs.retain(|path, _| {
        let parts: Vec<_> = path.iter().filter_map(|s| s.to_str()).collect();
        if parts.last() == Some(&"__sand_world_init.mcfunction") { return true; }
        if parts.last() == Some(&crate::build::output_manifest::MANIFEST_FILE_NAME) { return false; }
        !matches!(parts.as_slice(), ["data", _, kind, ..] if matches!(*kind,
            "function" | "tags" | "advancement" | "loot_table" | "predicate" | "item_modifier" | "recipe"))
    });
    let server = root
        .join("dist")
        .join(crate::build::SERVER_CONFIG_FILE_NAME);
    if server.exists() {
        inputs.insert(
            PathBuf::from("@server"),
            sand_build::fingerprint::hash_bytes(&std::fs::read(server)?),
        );
    }
    let config = root.join("sand.toml");
    inputs.insert(
        PathBuf::from("@config"),
        sand_build::fingerprint::hash_bytes(&std::fs::read(config)?),
    );
    Ok(inputs)
}

/// The one live pack owned by this dev invocation, retained across JVM restarts.
#[derive(Default)]
pub(crate) struct ManagedPack(Option<(PathBuf, String)>);

const OWNER_FILE: &str = ".sand-dev-owner";

fn pack_owner(path: &Path) -> Result<Option<String>> {
    let marker = path.join(OWNER_FILE);
    ensure!(
        !marker.is_symlink(),
        "dev ownership marker cannot be a symlink"
    );
    match std::fs::read_to_string(marker) {
        Ok(owner) => Ok(Some(owner)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl ManagedPack {
    /// Verify the committed directory before claiming it or removing the old
    /// namespace. Finish this handoff before honoring queued cancellation.
    /// Never remove a pack in a different world or follow a directory symlink.
    pub(crate) fn installed(&mut self, dest: &Path, committed: &LiveIdentity) -> Result<()> {
        ensure!(
            LiveIdentity::capture(dest)?.as_ref() == Some(committed),
            "committed datapack was replaced; preserving it and the previous namespace at {}",
            dest.display()
        );
        ensure!(!dest.is_symlink(), "live datapack cannot be a symlink");
        let current = std::fs::canonicalize(dest)?;
        if let Some((previous, owner)) = &self.0
            && previous != &current
            && previous.parent() == current.parent()
            && previous.exists()
        {
            ensure!(
                !previous.is_symlink(),
                "previous managed datapack became a symlink"
            );
            ensure!(
                pack_owner(previous)?.as_ref() == Some(owner),
                "previous managed datapack was replaced; preserving it at {}",
                previous.display()
            );
            std::fs::remove_dir_all(previous)
                .context("failed to remove previous session-managed datapack")?;
        }
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| anyhow::anyhow!("OS entropy unavailable"))?;
        let owner: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        std::fs::write(current.join(OWNER_FILE), &owner)?;
        self.0 = Some((current, owner));
        Ok(())
    }
}

/// Directory identity carried across staging, commit, and ownership handoff.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LiveIdentity {
    owner: Option<String>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(not(unix))]
    created: std::time::SystemTime,
}

impl LiveIdentity {
    fn capture(path: &Path) -> Result<Option<Self>> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        ensure!(
            metadata.is_dir() && !metadata.is_symlink(),
            "live datapack must remain an ordinary directory"
        );
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Ok(Some(Self {
            owner: pack_owner(path)?,
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
            #[cfg(not(unix))]
            created: metadata
                .created()
                .context("cannot establish live datapack identity")?,
        }))
    }
}

/// Successful live swap, including the identity needed for ownership handoff.
/// Private staging cleanup failures are warnings, not failed installations.
pub(crate) struct InstalledPack {
    pub stats: crate::run_cmd::SyncStats,
    pub identity: LiveIdentity,
}

pub(crate) fn install(
    src: &Path,
    dest: &Path,
    marker: Option<(&str, &str)>,
) -> Result<InstalledPack> {
    install_with_cleanup(src, dest, marker, |path| std::fs::remove_dir_all(path))
}

fn install_with_cleanup(
    src: &Path,
    dest: &Path,
    marker: Option<(&str, &str)>,
    cleanup: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<InstalledPack> {
    ensure!(!dest.is_symlink(), "live datapack cannot be a symlink");
    // Keep the invocation's ownership through staged hot reloads. This file
    // exists only in the live server tree, never the build/release output.
    let expected = LiveIdentity::capture(dest)?;
    let owner = expected
        .as_ref()
        .and_then(|identity| identity.owner.as_ref());
    let container = install_container(dest)?;
    let stage = container.path().join("stage");
    // Stage a copy of the live tree so the canonical incremental mirror can
    // report skipped/pruned files, without ever writing into the live tree.
    let prepare: Result<_> = (|| {
        if dest.exists() {
            crate::run_cmd::sync_dir(dest, &stage)?;
        }
        let stats = crate::run_cmd::sync_dir(src, &stage)?;
        if let Some(owner) = &owner {
            std::fs::write(stage.join(OWNER_FILE), owner)?;
        }
        if let Some((namespace, marker)) = marker {
            reload::add_marker(&stage, namespace, marker)?;
        }
        Ok(stats)
    })();
    let stats = prepare?;
    let identity = commit_install(
        container,
        dest,
        expected,
        |from, to| std::fs::rename(from, to),
        cleanup,
    )?;
    Ok(InstalledPack { stats, identity })
}

fn install_container(dest: &Path) -> Result<tempfile::TempDir> {
    let parent = dest
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    for _ in 0..32 {
        // mkdir is atomic: pre-existing operator paths are never adopted or
        // cleaned up. Children stay on the destination filesystem for rename.
        let container = tempfile::Builder::new()
            .prefix(".sand-dev-install-")
            .tempdir_in(parent)?;
        if container.path().file_name() != dest.file_name() {
            return Ok(container);
        }
    }
    bail!("could not allocate dev install staging distinct from the live pack")
}

fn commit_install(
    container: tempfile::TempDir,
    dest: &Path,
    expected: Option<LiveIdentity>,
    mut rename: impl FnMut(&Path, &Path) -> std::io::Result<()>,
    cleanup: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<LiveIdentity> {
    let stage = container.path().join("stage");
    let backup = container.path().join("backup");
    ensure!(
        LiveIdentity::capture(dest)? == expected,
        "live datapack changed during installation; preserving {}",
        dest.display()
    );
    // Capture the staged directory before commit: recapturing dest afterward
    // could accidentally bless an operator replacement as our own install.
    let committed = LiveIdentity::capture(&stage)?
        .ok_or_else(|| anyhow::anyhow!("staged datapack disappeared"))?;
    let existed = expected.is_some();
    if existed {
        rename(dest, &backup)?;
    }
    if let Err(error) = rename(&stage, dest) {
        if existed && rename(&backup, dest).is_err() {
            // Drop must not destroy the last good copy when rollback fails.
            let retained = container.keep().join("backup");
            bail!(
                "install failed ({error}); restore live pack from {}",
                retained.display()
            );
        }
        return Err(error.into());
    }
    // The live swap is committed. Cleanup cannot turn it into a failed install:
    // callers must still track reload health and dispatch the new on-disk pack.
    let retained = container.keep();
    if let Err(error) = cleanup(&retained) {
        eprintln!(
            "dev: datapack installed; failed to clean staging at {}: {error}",
            retained.display()
        );
    }
    Ok(committed)
}

fn report_change(previous: Option<&serde_json::Value>, report: &serde_json::Value) {
    let changed = previous.map(|old| old != report);
    eprintln!(
        "dev runtime report: {}; final-build evidence in dist/.sand-dev-build.json. Explicit evaluation bounds only; no per-tick scan or MSPT estimate.",
        match changed {
            None => "baseline captured",
            Some(true) => "changed",
            Some(false) => "unchanged",
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn health_session(root: &Path) -> Session {
        std::fs::create_dir_all(root.join("dist/demo")).unwrap();
        std::fs::write(root.join("sand.toml"), "config").unwrap();
        let props = root.join("dist/server.properties");
        std::fs::write(&props, "").unwrap();
        Session::at(
            root.into(),
            "demo".into(),
            "dev",
            &props,
            BuildInputs(watch::snapshot(root).unwrap(), root.join("target")),
        )
        .unwrap()
    }

    #[test]
    fn changing_cargo_target_during_session_excludes_old_and_new_outputs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::create_dir(root.join(".cargo")).unwrap();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "// initial").unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='dev-target-transition'\nversion='0.1.0'\n[workspace]\n",
        )
        .unwrap();
        // Resolve real metadata with isolated environment, without mutating the
        // test process's environment or current directory.
        let resolve = || {
            let output = Command::new("cargo")
                .args(["metadata", "--format-version=1", "--no-deps"])
                .current_dir(&root)
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_BUILD_TARGET_DIR")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            PathBuf::from(metadata["target_directory"].as_str().unwrap())
        };
        std::fs::write(
            root.join(".cargo/config.toml"),
            "[build]\ntarget-dir='build-one'\n",
        )
        .unwrap();
        let mut session = health_session(&root);
        session.refresh_target_directory(resolve()).unwrap();
        std::fs::create_dir(root.join("build-one")).unwrap();
        std::fs::write(root.join("build-one/artifact"), "first build").unwrap();
        std::fs::write(
            root.join(".cargo/config.toml"),
            "[build]\ntarget-dir='build-two'\n",
        )
        .unwrap();
        let now = Instant::now();
        session.watcher.observe(
            watch::snapshot_excluding(&root, &session.target_directories).unwrap(),
            now,
        );
        assert!(session.watcher.take_ready(now + Duration::from_secs(1)));
        let target = resolve();
        assert_eq!(target, root.join("build-two"));
        session.refresh_target_directory(target).unwrap();
        std::fs::create_dir(root.join("build-two")).unwrap();
        for name in ["build-one", "build-two"] {
            std::fs::write(root.join(name).join("artifact"), "new compiler output").unwrap();
        }
        session.watcher.observe(
            watch::snapshot_excluding(&root, &session.target_directories).unwrap(),
            now,
        );
        assert!(!session.watcher.take_ready(now + Duration::from_secs(1)));
        // Edits during compilation must still enqueue the next build.
        std::fs::write(root.join("src/lib.rs"), "// edited while building").unwrap();
        session.watcher.observe(
            watch::snapshot_excluding(&root, &session.target_directories).unwrap(),
            now,
        );
        assert!(session.watcher.take_ready(now + Duration::from_secs(1)));
        assert!(!session.watcher.take_ready(now + Duration::from_secs(2)));
    }

    fn log_event(raw: &str) -> crate::console::diagnostic::GroupedEvent {
        use crate::console::{
            classify::classify,
            diagnostic::GroupedEvent,
            log_record::{Stream, parse_line},
        };
        let record = parse_line(raw);
        GroupedEvent {
            received_at: Instant::now(),
            category: classify(Stream::Stdout, &record),
            stream: Stream::Stdout,
            lines: vec![record],
        }
    }

    #[test]
    fn command_text_cannot_enable_or_disable_dev_readiness() {
        let root = tempfile::tempdir().unwrap();
        let mut session = health_session(root.path());
        for ready in [false, true] {
            session.ready = ready;
            for message in [
                "[Server] Stopping server",
                "[Server: Stopping the server]",
                "[Server] Done (1s)! For help, type \"help\"",
                "<Player> Done (1s)! For help, type \"help\"",
            ] {
                session.observe(
                    &log_event(&format!("[12:00:00] [Server thread/INFO]: {message}")),
                    crate::console::phase::RunPhase::Runtime,
                );
                assert_eq!(session.ready, ready, "{message}");
            }
        }
        session.observe(
            &log_event("[12:00:00] [Server thread/INFO]: Stopping server"),
            crate::console::phase::RunPhase::Shutdown,
        );
        assert!(!session.ready);
        session.observe(
            &log_event("[12:00:00] [Server thread/INFO]: Done (1s)! For help, type \"help\""),
            crate::console::phase::RunPhase::ServerStartup,
        );
        assert!(session.ready);
    }

    fn complete_reload(session: &mut Session) {
        use crate::console::phase::RunPhase;
        let marker = session.pending_reload.as_ref().unwrap().marker.clone();
        session.observe(
            &log_event(&format!(
                "[12:00:00] [Server thread/INFO]: [Not Secure] [Server] {marker}"
            )),
            RunPhase::Reload,
        );
    }

    #[test]
    fn queued_diagnostics_use_headline_time_across_dispatch_and_continuations() {
        use crate::console::{
            RunHealth,
            classify::classify,
            diagnostic::Grouper,
            log_record::{Stream, parse_line},
            phase::RunPhase,
        };
        for before_dispatch in [true, false] {
            let tmp = tempfile::tempdir().unwrap();
            let mut session = health_session(tmp.path());
            let boundary = Instant::now();
            let received = if before_dispatch {
                boundary - Duration::from_secs(1)
            } else {
                boundary
            };
            let mut grouper = Grouper::new();
            let record =
                parse_line("[12:00:00] [Server thread/ERROR]: Failed to load function demo:broken");
            let category = classify(Stream::Stdout, &record);
            assert!(
                grouper
                    .feed_at(Stream::Stdout, record, category, received)
                    .is_empty()
            );
            let mut reload = reload::Reload::new().unwrap();
            reload.dispatching(boundary);
            // Both the buffered headline and continuation are consumed after
            // dispatch; receipt of the headline defines ownership of the group.
            session.pending_reload = Some(reload);
            let record = parse_line("    parser continuation");
            let category = classify(Stream::Stdout, &record);
            assert!(
                grouper
                    .feed_at(
                        Stream::Stdout,
                        record,
                        category,
                        boundary + Duration::from_millis(1)
                    )
                    .is_empty()
            );
            let group = grouper.flush().unwrap();
            assert_eq!(group.received_at, received);
            assert_eq!(session.is_reload_event(&group), !before_dispatch);
            assert_eq!(
                session.event_phase(&group, RunPhase::Reload),
                if before_dispatch {
                    RunPhase::Runtime
                } else {
                    RunPhase::Reload
                }
            );
            session.observe(&group, RunPhase::Reload);
            // Model logs arriving during the synchronous RCON response: receipt
            // predates acknowledgement but must still fail this reload.
            session
                .pending_reload
                .as_mut()
                .unwrap()
                .dispatched(boundary + Duration::from_secs(2));
            complete_reload(&mut session);
            assert_eq!(
                session.health(),
                if before_dispatch {
                    RunHealth::Healthy
                } else {
                    RunHealth::Degraded
                }
            );
            assert_eq!(session.reload_failed, !before_dispatch);
            let mut recovered = reload::Reload::new().unwrap();
            recovered.dispatching(Instant::now());
            session.pending_reload = Some(recovered);
            complete_reload(&mut session);
            assert_eq!(session.health(), RunHealth::Healthy);
        }
    }

    #[test]
    fn queued_server_failure_is_not_cleared_by_managed_reload() {
        use crate::console::{RunHealth, phase::RunPhase};
        let tmp = tempfile::tempdir().unwrap();
        let mut session = health_session(tmp.path());
        let error =
            log_event("[12:00:00] [Server thread/ERROR]: Encountered an unexpected exception");
        let mut reload = reload::Reload::new().unwrap();
        reload.dispatching(Instant::now());
        session.pending_reload = Some(reload);
        assert!(!session.is_reload_event(&error));
        session.observe(&error, RunPhase::Reload);
        assert_eq!(session.health(), RunHealth::Failed);
        complete_reload(&mut session);
        assert_eq!(session.health(), RunHealth::Failed);
    }

    #[test]
    fn dev_recovers_only_managed_reload_health_after_a_confirmed_sentinel() {
        use crate::console::{
            RunHealth,
            phase::RunPhase,
            render::{OutputMode, Renderer},
        };
        let tmp = tempfile::tempdir().unwrap();
        let mut session = health_session(tmp.path());
        let mut historical = Renderer::new(OutputMode::Classified, "26.2".into());
        session.pending_reload = Some(reload::Reload::new().unwrap());
        session
            .pending_reload
            .as_mut()
            .unwrap()
            .dispatching(Instant::now());
        let error =
            log_event("[12:00:00] [Server thread/ERROR]: Failed to load function demo:broken");
        session.observe(&error, RunPhase::Reload);
        historical.render(&error, RunPhase::Reload);
        complete_reload(&mut session);
        assert_eq!(
            session.health(),
            RunHealth::Degraded,
            "unrecovered error must fail at quit"
        );
        session.observe(
            &log_event("[12:00:00] [Server thread/INFO]: Done (1s)! For help, type \"help\""),
            RunPhase::Runtime,
        );
        assert_eq!(
            session.health(),
            RunHealth::Degraded,
            "ready cannot prove reload recovery"
        );
        session.pending_reload = Some(reload::Reload::new().unwrap());
        session
            .pending_reload
            .as_mut()
            .unwrap()
            .dispatching(Instant::now());
        assert_eq!(
            session.health(),
            RunHealth::Degraded,
            "dispatch is not recovery"
        );
        complete_reload(&mut session);
        assert_eq!(session.health(), RunHealth::Healthy);
        assert_eq!(
            historical.health(),
            RunHealth::Degraded,
            "ordinary run remains monotonic"
        );
        session.finish_reload(reload::Outcome::TimedOut);
        assert_eq!(session.health(), RunHealth::Degraded);
    }

    #[test]
    fn startup_and_unrelated_fatal_errors_survive_a_successful_dev_reload() {
        use crate::console::{RunHealth, phase::RunPhase};
        for phase in [RunPhase::ServerStartup, RunPhase::Shutdown] {
            let tmp = tempfile::tempdir().unwrap();
            let mut session = health_session(tmp.path());
            let raw = if phase == RunPhase::ServerStartup {
                "[12:00:00] [Server thread/ERROR]: Failed to load function demo:startup"
            } else {
                "[12:00:00] [Server thread/ERROR]: Encountered an unexpected exception"
            };
            session.observe(&log_event(raw), phase);
            let before = session.health();
            assert_ne!(before, RunHealth::Healthy);
            session.pending_reload = Some(reload::Reload::new().unwrap());
            session
                .pending_reload
                .as_mut()
                .unwrap()
                .dispatching(Instant::now());
            complete_reload(&mut session);
            assert_eq!(session.health(), before);
        }
    }

    #[test]
    fn startup_edits_queue_a_rebuild_and_empty_server_pause_is_restored() {
        for pause in ["", "pause-when-empty-seconds=17\n"] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            std::fs::create_dir_all(root.join("dist/demo")).unwrap();
            std::fs::write(root.join("sand.toml"), "config").unwrap();
            let source = root.join("main.rs");
            std::fs::write(&source, "before build").unwrap();
            let inputs = BuildInputs(watch::snapshot(root).unwrap(), root.join("target"));
            // Compilation consumed the old input; an edit during compilation
            // must remain pending even though the session has not started yet.
            std::fs::write(&source, "saved while building").unwrap();
            let props = root.join("dist/server.properties");
            std::fs::write(&props, format!("server-port=25565\n{pause}")).unwrap();
            let mut session =
                Session::at(root.into(), "demo".into(), "dev", &props, inputs).unwrap();
            assert!(
                session
                    .watcher
                    .take_ready(Instant::now() + Duration::from_secs(1))
            );
            assert!(
                !session
                    .watcher
                    .take_ready(Instant::now() + Duration::from_secs(2))
            );
            let running = std::fs::read_to_string(&props).unwrap();
            assert!(running.contains("pause-when-empty-seconds=-1\n"));
            assert!(!running.contains("pause-when-empty-seconds=17"));
            drop(session);
            let stopped = std::fs::read_to_string(&props).unwrap();
            assert!(!stopped.contains("pause-when-empty-seconds=-1"));
            if pause.is_empty() {
                assert!(!stopped.contains("pause-when-empty-seconds"));
            } else {
                assert!(stopped.contains(pause));
            }
        }
    }

    #[test]
    fn failed_or_malformed_build_preserves_live_pack_and_reaps_worker() {
        for (success, output) in [
            (false, ""),
            (false, "not JSON"),
            (false, r#"{"success":false,"diagnostics":[]}"#),
            (true, "not JSON"),
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            let dest = root.join("dist/server/world/datapacks/demo");
            std::fs::create_dir_all(&dest).unwrap();
            std::fs::create_dir_all(root.join("dist/demo")).unwrap();
            std::fs::write(dest.join("pack.mcmeta"), "last good").unwrap();
            std::fs::write(root.join("sand.toml"), "config").unwrap();
            let props = root.join("dist/server/server.properties");
            std::fs::write(&props, "server-port=25565\n").unwrap();
            let inputs = BuildInputs(watch::snapshot(root).unwrap(), root.join("target"));
            let mut session =
                Session::at(root.into(), "demo".into(), "dev", &props, inputs).unwrap();
            session.ready = true;
            let mut command = if cfg!(windows) {
                Command::new("cmd")
            } else {
                Command::new("sh")
            };
            if cfg!(windows) {
                command.args(["/C", if success { "exit 0" } else { "exit 1" }]);
            } else {
                command.args(["-c", if success { "exit 0" } else { "exit 1" }]);
            }
            let mut child = command.spawn().unwrap();
            child.wait().unwrap();
            session.build = Some((child, Instant::now()));
            std::fs::write(session.build_log(), output).unwrap();
            let _ = session.poll();
            assert!(session.build.is_none());
            assert!(session.pending_reload.is_none());
            assert_eq!(
                std::fs::read_to_string(dest.join("pack.mcmeta")).unwrap(),
                "last good"
            );
            assert!(
                session.poll().is_ok(),
                "failure must be consumed, not retried every poll"
            );
            drop(session);
            let properties = std::fs::read_to_string(props).unwrap();
            assert!(!properties.contains("rcon.password"));
            assert!(properties.contains("enable-rcon=false"));
        }
    }

    #[test]
    fn dev_binding_is_temporary_and_restores_absent_blank_or_custom_settings() {
        for setting in [
            "",
            "server-ip=\n",
            "server-ip=0.0.0.0\n",
            "server-ip: 192.0.2.1\n",
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            std::fs::create_dir_all(root.join("dist/demo")).unwrap();
            std::fs::write(root.join("sand.toml"), "config").unwrap();
            let props = root.join("server.properties");
            std::fs::write(&props, format!("server-port=49152\n{setting}")).unwrap();
            for _ in 0..2 {
                let session = Session::at(
                    root.into(),
                    "demo".into(),
                    "dev",
                    &props,
                    BuildInputs(watch::snapshot(root).unwrap(), root.join("target")),
                )
                .unwrap();
                assert_ne!(session.rcon.port, 49152);
                let running = std::fs::read_to_string(&props).unwrap();
                assert_eq!(
                    running
                        .lines()
                        .filter(|line| property_key(line) == "server-ip")
                        .collect::<Vec<_>>(),
                    ["server-ip=127.0.0.1"]
                );
                drop(session);
                let stopped = std::fs::read_to_string(&props).unwrap();
                assert_eq!(
                    stopped
                        .lines()
                        .filter(|line| property_key(line) == "server-ip")
                        .collect::<Vec<_>>(),
                    setting.lines().collect::<Vec<_>>()
                );
                assert!(!stopped.contains("rcon.password"));
            }
        }
    }

    #[test]
    fn namespace_switch_removes_only_previous_pack_after_success_and_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src = root.join("source");
        let old = root.join("world/datapacks/old");
        let new = root.join("world/datapacks/new");
        let manual = root.join("world/datapacks/manual");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("pack.mcmeta"), "pack").unwrap();
        std::fs::create_dir_all(&manual).unwrap();
        std::fs::write(manual.join("keep"), "operator").unwrap();
        let mut managed = ManagedPack::default();
        let old_install = install(&src, &old, None).unwrap();
        managed.installed(&old, &old_install.identity).unwrap();
        assert!(install(&root.join("missing"), &new, None).is_err());
        assert!(old.exists(), "failed preparation retains previous pack");
        let new_install = install(&src, &new, None).unwrap();
        assert!(
            old.exists(),
            "cancellation before commit retains previous pack"
        );
        managed.installed(&new, &new_install.identity).unwrap();
        assert!(!old.exists());
        assert!(new.join("pack.mcmeta").exists());
        assert!(manual.join("keep").exists());
        let other_world = root.join("other-world/datapacks/third");
        let other_install = install(&src, &other_world, None).unwrap();
        managed
            .installed(&other_world, &other_install.identity)
            .unwrap();
        assert!(new.exists(), "never prune a previous world");
    }

    #[test]
    fn namespace_switch_preserves_an_ordinary_replacement_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let old = tmp.path().join("old");
        let new = tmp.path().join("new");
        let src = tmp.path().join("src");
        for path in [&old, &new, &src] {
            std::fs::create_dir(path).unwrap();
        }
        let mut managed = ManagedPack::default();
        managed
            .installed(&old, &LiveIdentity::capture(&old).unwrap().unwrap())
            .unwrap();
        let owner = pack_owner(&old).unwrap();
        install(&src, &old, None).unwrap();
        assert_eq!(
            pack_owner(&old).unwrap(),
            owner,
            "reload preserves ownership"
        );
        std::fs::remove_dir_all(&old).unwrap();
        std::fs::create_dir(&old).unwrap();
        std::fs::write(old.join("operator-data"), "do not delete").unwrap();
        let error = managed
            .installed(&new, &LiveIdentity::capture(&new).unwrap().unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains("was replaced"));
        assert_eq!(
            std::fs::read_to_string(old.join("operator-data")).unwrap(),
            "do not delete"
        );
        assert!(pack_owner(&old).unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn namespace_switch_refuses_replaced_pack_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let old = tmp.path().join("old");
        let new = tmp.path().join("new");
        let external = tmp.path().join("external");
        for path in [&old, &new, &external] {
            std::fs::create_dir(path).unwrap();
        }
        std::fs::write(external.join("keep"), "operator").unwrap();
        let mut managed = ManagedPack::default();
        managed
            .installed(&old, &LiveIdentity::capture(&old).unwrap().unwrap())
            .unwrap();
        std::fs::remove_dir_all(&old).unwrap();
        std::os::unix::fs::symlink(&external, &old).unwrap();
        assert!(
            managed
                .installed(&new, &LiveIdentity::capture(&new).unwrap().unwrap())
                .is_err()
        );
        assert!(external.join("keep").exists());
    }

    #[test]
    fn properties_remove_duplicates_preserve_unrelated_and_clear_secret() {
        let original = "# note\nrcon.password=secret\nrcon.password:other\nserver-ip=0.0.0.0\nenable-rcon=true\nserver-port=25566\n";
        assert_eq!(without_rcon(original), "# note\nserver-port=25566\n");
    }
    #[test]
    fn staged_swap_preserves_replacement_or_newly_arrived_destination() {
        for initial in ["absent", "unmanaged", "managed"] {
            let initially_present = initial != "absent";
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("live");
            if initially_present {
                std::fs::create_dir(&dest).unwrap();
                if initial == "managed" {
                    ManagedPack::default()
                        .installed(&dest, &LiveIdentity::capture(&dest).unwrap().unwrap())
                        .unwrap();
                }
            }
            let expected = LiveIdentity::capture(&dest).unwrap();
            let container = install_container(&dest).unwrap();
            let staging_path = container.path().to_path_buf();
            std::fs::create_dir(staging_path.join("stage")).unwrap();
            std::fs::write(staging_path.join("stage/new"), "new pack").unwrap();
            // Deterministic replacement after staging started, before swap.
            if initially_present {
                // Keep the old inode allocated so this tests identity rather
                // than relying on filesystem inode-reuse timing.
                std::fs::rename(&dest, tmp.path().join("removed-original")).unwrap();
            }
            std::fs::create_dir(&dest).unwrap();
            std::fs::write(dest.join("operator-data"), "preserve").unwrap();
            let result = commit_install(
                container,
                &dest,
                expected,
                |_, _| panic!("must not rename replacement"),
                |path| std::fs::remove_dir_all(path),
            );
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("changed during installation")
            );
            assert_eq!(
                std::fs::read_to_string(dest.join("operator-data")).unwrap(),
                "preserve"
            );
            assert!(!dest.join("new").exists());
            assert!(!staging_path.exists());
        }
    }

    #[test]
    fn staging_never_aliases_dotted_suffix_namespaces_or_operator_neighbors() {
        for namespace in [
            "demo.sand-dev-stage",
            "demo.sand-dev-backup",
            "demo.one",
            "demo.two",
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let src = tmp.path().join("source");
            let packs = tmp.path().join("datapacks");
            let dest = packs.join(namespace);
            std::fs::create_dir_all(&src).unwrap();
            std::fs::create_dir_all(&packs).unwrap();
            std::fs::write(src.join("pack.mcmeta"), "first").unwrap();
            let neighbors: Vec<_> = [
                "demo.sand-dev-stage",
                "demo.sand-dev-backup",
                "demo.one",
                "demo.two",
                ".sand-dev-install-operator",
            ]
            .into_iter()
            .filter(|name| *name != namespace)
            .map(|name| packs.join(name))
            .collect();
            for neighbor in &neighbors {
                std::fs::create_dir(neighbor).unwrap();
                std::fs::write(neighbor.join("keep"), "operator").unwrap();
            }
            install(&src, &dest, None).unwrap();
            std::fs::write(src.join("pack.mcmeta"), "second").unwrap();
            install(&src, &dest, None).unwrap();
            assert!(install(&tmp.path().join("missing"), &dest, None).is_err());
            assert_eq!(
                std::fs::read_to_string(dest.join("pack.mcmeta")).unwrap(),
                "second"
            );
            for neighbor in &neighbors {
                assert_eq!(
                    std::fs::read_to_string(neighbor.join("keep")).unwrap(),
                    "operator"
                );
            }
            assert_eq!(
                std::fs::read_dir(&packs).unwrap().count(),
                neighbors.len() + 1
            );
        }
    }

    #[test]
    fn failed_publish_restores_live_pack_or_retains_backup_when_rollback_fails() {
        for fail_rollback in [false, true] {
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("live");
            std::fs::create_dir(&dest).unwrap();
            std::fs::write(dest.join("old"), "last good").unwrap();
            let container = install_container(&dest).unwrap();
            let retained = container.path().to_path_buf();
            std::fs::create_dir(retained.join("stage")).unwrap();
            let mut calls = 0;
            let result = commit_install(
                container,
                &dest,
                LiveIdentity::capture(&dest).unwrap(),
                |from, to| {
                    calls += 1;
                    if calls == 2 || (calls == 3 && fail_rollback) {
                        Err(std::io::Error::other("injected rename failure"))
                    } else {
                        std::fs::rename(from, to)
                    }
                },
                |path| std::fs::remove_dir_all(path),
            );
            assert!(result.is_err());
            if fail_rollback {
                assert!(!dest.exists());
                assert_eq!(
                    std::fs::read_to_string(retained.join("backup/old")).unwrap(),
                    "last good"
                );
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains(&retained.display().to_string())
                );
            } else {
                assert_eq!(
                    std::fs::read_to_string(dest.join("old")).unwrap(),
                    "last good"
                );
                assert!(!retained.exists());
            }
        }
    }

    #[test]
    fn cleanup_failure_still_dispatches_reload_and_requires_confirmation() {
        use crate::console::RunHealth;
        use std::io::{Read, Write};
        let tmp = tempfile::tempdir().unwrap();
        let mut session = health_session(tmp.path());
        session.ready = true;
        let dest = tmp.path().join("dist/server/world/datapacks/demo");
        std::fs::create_dir_all(&dest).unwrap();
        std::fs::write(dest.join("old"), "old pack").unwrap();
        let mut command = if cfg!(windows) {
            Command::new("cmd")
        } else {
            Command::new("sh")
        };
        command.args(if cfg!(windows) {
            ["/C", "exit 0"]
        } else {
            ["-c", "exit 0"]
        });
        let mut child = command.spawn().unwrap();
        child.wait().unwrap();
        session.build = Some((child, Instant::now()));
        std::fs::write(
            session.build_log(),
            r#"{"success":true,"namespace":"demo","runtime_report":{}}"#,
        )
        .unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        session.rcon.port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "reload was never dispatched");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("{error}"),
                }
            };
            socket.set_nonblocking(false).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            for (id, kind) in [(1_i32, 2_i32), (2, 0)] {
                let mut length = [0; 4];
                socket.read_exact(&mut length).unwrap();
                let length = i32::from_le_bytes(length);
                assert!((10..=4110).contains(&length));
                let mut body = vec![0; length as usize];
                socket.read_exact(&mut body).unwrap();
                assert_eq!(&body[..4], &id.to_le_bytes());
                if id == 2 {
                    assert_eq!(&body[8..body.len() - 2], b"reload");
                }
                let reply = [10_i32.to_le_bytes(), id.to_le_bytes(), kind.to_le_bytes()].concat();
                socket
                    .write_all(&[reply.as_slice(), &[0, 0]].concat())
                    .unwrap();
            }
        });
        let mut retained = None;
        session
            .poll_with_cleanup(|path| {
                retained = Some(path.to_path_buf());
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "injected locked backup",
                ))
            })
            .unwrap();
        server.join().unwrap();
        assert_eq!(session.reload_number, 1);
        assert!(
            session.pending_reload.is_some(),
            "committed install must await its sentinel"
        );
        assert!(session.reload_failed);
        assert_ne!(
            session.health(),
            RunHealth::Healthy,
            "quit before confirmation must fail"
        );
        assert!(retained.unwrap().join("backup/old").exists());
        assert!(!dest.join("old").exists());
        complete_reload(&mut session);
        assert!(session.pending_reload.is_none());
        assert_eq!(session.health(), RunHealth::Healthy);
    }

    #[test]
    fn committed_install_survives_cleanup_failure_with_identity_and_load_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("live");
        std::fs::create_dir(&dest).unwrap();
        std::fs::write(dest.join("old"), "last good").unwrap();
        let container = install_container(&dest).unwrap();
        let retained = container.path().to_path_buf();
        let stage = retained.join("stage");
        std::fs::create_dir(&stage).unwrap();
        std::fs::write(stage.join("new"), "new pack").unwrap();
        let pending = reload::Reload::new().unwrap();
        reload::add_marker(&stage, "demo", &pending.marker).unwrap();
        let committed = commit_install(
            container,
            &dest,
            LiveIdentity::capture(&dest).unwrap(),
            |from, to| std::fs::rename(from, to),
            |path| {
                assert_eq!(path, retained);
                assert_eq!(
                    std::fs::read_to_string(dest.join("new")).unwrap(),
                    "new pack"
                );
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "injected locked backup",
                ))
            },
        )
        .expect("post-commit cleanup must not report an uncommitted install");
        assert_eq!(
            LiveIdentity::capture(&dest).unwrap().as_ref(),
            Some(&committed)
        );
        assert_eq!(
            std::fs::read_to_string(retained.join("backup/old")).unwrap(),
            "last good"
        );
        assert!(!dest.join("old").exists());
        assert!(
            dest.join(format!("data/demo/function/{}.mcfunction", pending.marker))
                .exists()
        );
        let mut managed = ManagedPack::default();
        managed.installed(&dest, &committed).unwrap();
        assert!(pack_owner(&dest).unwrap().is_some());
    }

    #[test]
    fn install_prunes_only_managed_pack_and_failed_stage_preserves_live_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dest = tmp.path().join("live");
        std::fs::create_dir(&src).unwrap();
        std::fs::create_dir(&dest).unwrap();
        std::fs::write(src.join("new"), "new").unwrap();
        std::fs::write(dest.join("old"), "old").unwrap();
        std::fs::write(tmp.path().join("other"), "keep").unwrap();
        let stats = install(&src, &dest, None).unwrap().stats;
        assert_eq!(stats.copied, 1);
        assert_eq!(stats.pruned, 1);
        assert!(!dest.join("old").exists());
        assert!(install(&tmp.path().join("missing"), &dest, None).is_err());
        assert_eq!(std::fs::read_to_string(dest.join("new")).unwrap(), "new");
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("other")).unwrap(),
            "keep"
        );
    }
    #[test]
    fn function_changes_reload_but_registry_metadata_and_config_require_restart() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let pack = root.join("dist/demo");
        std::fs::create_dir_all(pack.join("data/demo/function")).unwrap();
        std::fs::write(root.join("sand.toml"), "config").unwrap();
        std::fs::write(pack.join("pack.mcmeta"), "metadata").unwrap();
        let before = restart_inputs(root, "demo").unwrap();
        std::fs::write(
            pack.join("data/demo/function/test.mcfunction"),
            "say changed",
        )
        .unwrap();
        assert_eq!(before, restart_inputs(root, "demo").unwrap());
        std::fs::create_dir_all(pack.join("data/demo/dimension")).unwrap();
        std::fs::write(pack.join("data/demo/dimension/test.json"), "{}").unwrap();
        assert_ne!(before, restart_inputs(root, "demo").unwrap());
    }
}
