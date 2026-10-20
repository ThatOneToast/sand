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
pub(crate) struct BuildInputs(Snapshot);

impl BuildInputs {
    pub(crate) fn capture() -> Result<Self> {
        Ok(Self(watch::snapshot(&std::env::current_dir()?)?))
    }
}

pub(crate) struct Session {
    root: PathBuf,
    namespace: String,
    profile: String,
    properties: PathBuf,
    original_pause_settings: String,
    rcon: rcon::Rcon,
    watcher: Watch,
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
        watcher.observe(watch::snapshot(&root)?, Instant::now());
        let restart_inputs = restart_inputs(&root, &namespace)?;
        let rcon = rcon::Rcon::new()?;
        let existing = std::fs::read_to_string(properties)?;
        let original_pause_settings = existing
            .lines()
            .filter(|line| property_key(line) == "pause-when-empty-seconds")
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
            original_pause_settings,
            rcon,
            watcher,
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

    pub(crate) fn is_reloading(&self) -> bool {
        self.pending_reload.is_some()
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
        let mut category = event.category;
        let message = &event.headline().message;
        if let Some(diagnostic) = build_diagnostic(event, phase) {
            match diagnostic.fatality() {
                Fatality::Nonfatal => category = Category::Warning,
                Fatality::FatalToDatapackHealth if self.is_reloading() => self.reload_failed = true,
                _ => self.persistent_health.observe(&diagnostic),
            }
        }
        if category == Category::Ready {
            self.ready = true;
        }
        if category == Category::Shutdown {
            self.ready = false;
        }
        if let Some(pending) = self.pending_reload.as_mut()
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
            self.watcher.observe(watch::snapshot(&self.root)?, now);
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
            let stats = install(
                &self.root.join("dist").join(&self.namespace),
                &self
                    .root
                    .join("dist/server/world/datapacks")
                    .join(&self.namespace),
                Some((&self.namespace, &pending.marker)),
            )?;
            self.reload_number += 1;
            eprintln!(
                "dev build {:.2}s: {}; sync {} copied, {} skipped, {} pruned",
                start.elapsed().as_secs_f64(),
                result["datapack_changes"],
                stats.copied,
                stats.skipped,
                stats.pruned
            );
            report_change(self.previous_report.as_ref(), &result["runtime_report"]);
            self.previous_report = Some(result["runtime_report"].clone());
            self.pending_reload = Some(pending);
            self.reload_failed = true;
            match self.rcon.reload() {
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
                "{}server-ip=127.0.0.1\nenable-rcon=false\n{}",
                without_rcon(&text),
                self.original_pause_settings
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

fn install(
    src: &Path,
    dest: &Path,
    marker: Option<(&str, &str)>,
) -> Result<crate::run_cmd::SyncStats> {
    ensure!(!dest.is_symlink(), "live datapack cannot be a symlink");
    let stage = dest.with_extension("sand-dev-stage");
    let backup = dest.with_extension("sand-dev-backup");
    ensure!(
        !stage.exists() && !backup.exists(),
        "previous dev install staging exists; inspect it before retrying"
    );
    // Stage a copy of the live tree so the canonical incremental mirror can
    // report skipped/pruned files, without ever writing into the live tree.
    let prepare = (|| {
        if dest.exists() {
            crate::run_cmd::sync_dir(dest, &stage)?;
        }
        let stats = crate::run_cmd::sync_dir(src, &stage)?;
        if let Some((namespace, marker)) = marker {
            reload::add_marker(&stage, namespace, marker)?;
        }
        Ok(stats)
    })();
    let stats = match prepare {
        Ok(stats) => stats,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&stage);
            return Err(error);
        }
    };
    let existed = dest.exists();
    if existed && let Err(error) = std::fs::rename(dest, &backup) {
        let _ = std::fs::remove_dir_all(&stage);
        return Err(error.into());
    }
    if let Err(error) = std::fs::rename(&stage, dest) {
        if existed && std::fs::rename(&backup, dest).is_err() {
            bail!(
                "install failed ({error}); restore live pack from {}",
                backup.display()
            );
        }
        let _ = std::fs::remove_dir_all(&stage);
        return Err(error.into());
    }
    if existed {
        std::fs::remove_dir_all(backup)?;
    }
    Ok(stats)
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
            BuildInputs(watch::snapshot(root).unwrap()),
        )
        .unwrap()
    }

    fn log_event(raw: &str) -> crate::console::diagnostic::GroupedEvent {
        use crate::console::{
            classify::classify,
            diagnostic::GroupedEvent,
            log_record::{Stream, parse_line},
        };
        let record = parse_line(raw);
        GroupedEvent {
            category: classify(Stream::Stdout, &record),
            stream: Stream::Stdout,
            lines: vec![record],
        }
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
    fn dev_recovers_only_managed_reload_health_after_a_confirmed_sentinel() {
        use crate::console::{
            RunHealth,
            phase::RunPhase,
            render::{OutputMode, Renderer},
        };
        let tmp = tempfile::tempdir().unwrap();
        let mut session = health_session(tmp.path());
        let mut historical = Renderer::new(OutputMode::Classified, "26.2".into());
        let error =
            log_event("[12:00:00] [Server thread/ERROR]: Failed to load function demo:broken");
        session.pending_reload = Some(reload::Reload::new().unwrap());
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
            let inputs = BuildInputs(watch::snapshot(root).unwrap());
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
            let inputs = BuildInputs(watch::snapshot(root).unwrap());
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
    fn properties_remove_duplicates_preserve_unrelated_and_clear_secret() {
        let original = "# note\nrcon.password=secret\nrcon.password:other\nserver-ip=0.0.0.0\nenable-rcon=true\nserver-port=25566\n";
        assert_eq!(without_rcon(original), "# note\nserver-port=25566\n");
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
        let stats = install(&src, &dest, None).unwrap();
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
