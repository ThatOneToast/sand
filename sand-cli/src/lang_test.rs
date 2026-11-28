//! Run Sand test entry points on a disposable vanilla server.
use anyhow::{Context, Result, bail, ensure};
use sand_lang::Analysis;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub(crate) fn run(
    analysis: &Analysis,
    jar: Option<PathBuf>,
    accept_eula: bool,
    timeout: Duration,
) -> Result<()> {
    ensure!(
        accept_eula,
        "pass --accept-eula to run the vanilla Minecraft server"
    );
    let pack = analysis.compile().map_err(|errors| {
        for error in &errors {
            eprint!("{}", error.render(&analysis.package));
        }
        anyhow::anyhow!("test pack compilation failed")
    })?;
    let namespace = &analysis
        .pack
        .as_ref()
        .expect("compiled pack metadata")
        .namespace;
    let jar = match jar {
        Some(jar) => jar.canonicalize().context("open test server JAR")?,
        None => {
            sand_build::ensure_server_jar(pack.target.strip_suffix(".0").unwrap_or(&pack.target))?
        }
    };
    let root = tempfile::tempdir()?;
    let path = root.path().canonicalize()?;
    let datapack = path.join("world/datapacks").join(namespace);
    crate::build::publication::publish_pack(&datapack, &pack.resources)?;
    let mut server = TestServer::start(&path, &jar)?;
    let mut version = None;
    server.wait_for(timeout, "server startup", |line| {
        if let Some((_, release)) = line.split_once("Starting minecraft server version ") {
            version = Some(release.trim().to_owned());
        }
        line.contains("Done (") && line.contains("For help")
    })?;
    let found = version.context("could not identify the Minecraft server version")?;
    let release = sand::version::MinecraftVersion::parse(&found)?;
    let expected = sand::version::MinecraftVersion::parse(&pack.target)?;
    ensure!(
        release.components() == expected.components(),
        "test server is Minecraft {found}, but the pack targets {}",
        pack.target
    );

    // Keep a deterministic origin available for tests that summon entities or
    // set blocks, even while there are no connected players.
    server.load_origin(timeout)?;

    let mut failed = 0;
    for (index, test) in analysis.tests.iter().enumerate() {
        let pass = format!(" __SAND_TEST_{index}_PASS__");
        let fail = format!(" __SAND_TEST_{index}_FAIL__");
        let done = format!("__SAND_TEST_{index}_DONE__");
        let done_suffix = format!(" {done}");
        server.send(&format!(
            "execute in minecraft:overworld positioned 0 80 0 run function {}",
            test.runner
        ))?;
        server.send(&format!("say {done}"))?;
        let mut passed = false;
        let mut completed = false;
        let mut failed_assertion = None;
        server.wait_for(timeout, &test.name, |line| {
            if line.ends_with(&pass) || line.ends_with(&fail) {
                passed = line.ends_with(&pass);
                completed = true;
            }
            if let Some(location) = line
                .rsplit_once(" __SAND_ASSERT_")
                .and_then(|(_, suffix)| suffix.strip_suffix("__"))
            {
                failed_assertion = location.split_once('_').and_then(|(file, offset)| {
                    Some((file.parse::<usize>().ok()?, offset.parse::<usize>().ok()?))
                });
            }
            line.ends_with(&done_suffix)
        })?;
        if completed && passed {
            println!("ok {}", test.name);
        } else {
            failed += 1;
            let assertion = failed_assertion.and_then(|(file, offset)| {
                let source = analysis.package.files.get(file)?;
                source.text.get(..offset)?;
                Some((source, offset))
            });
            let (source, offset) =
                assertion.unwrap_or((&analysis.package.files[test.file.0], test.span.start));
            let line = source.lines.chars(offset).line + 1;
            let reason = if assertion.is_some() {
                "assertion failed"
            } else if completed {
                "returned false"
            } else {
                "did not finish"
            };
            eprintln!(
                "FAILED {} ({}:{line}, {reason})",
                test.name,
                source.path.display()
            );
        }
    }
    server.stop(timeout)?;
    println!("{} passed, {failed} failed", analysis.tests.len() - failed);
    if failed > 0 {
        bail!("{failed} Sand test(s) failed");
    }
    Ok(())
}

struct TestServer {
    child: Child,
    input: ChildStdin,
    output: Receiver<String>,
    recent: VecDeque<String>,
}

impl TestServer {
    fn start(root: &Path, jar: &Path) -> Result<Self> {
        let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = socket.local_addr()?.port();
        std::fs::write(root.join("eula.txt"), "eula=true\n")?;
        std::fs::write(
            root.join("server.properties"),
            format!(
                "server-ip=127.0.0.1\nserver-port={port}\nonline-mode=false\n                 level-type=minecraft:flat\ngenerate-structures=false\nview-distance=2\n                 simulation-distance=2\npause-when-empty-seconds=0\n"
            ),
        )?;
        drop(socket);
        let stderr = std::fs::File::create(root.join("stderr.log"))?;
        let mut child = Command::new("java")
            .args(["-Xmx1G", "-jar"])
            .arg(jar)
            .arg("nogui")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .context("start Java for Sand tests")?;
        let input = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let (tx, output) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            recent: VecDeque::new(),
        })
    }

    fn send(&mut self, line: &str) -> Result<()> {
        writeln!(self.input, "{line}").context("send command to Minecraft test server")?;
        self.input.flush()?;
        Ok(())
    }

    fn wait_for(
        &mut self,
        timeout: Duration,
        phase: &str,
        mut matches: impl FnMut(&str) -> bool,
    ) -> Result<()> {
        let deadline = Instant::now() + timeout;
        while let Ok(line) = self
            .output
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        {
            if matches(&line) {
                return Ok(());
            }
            if self.recent.len() == 20 {
                self.recent.pop_front();
            }
            self.recent.push_back(line);
            if Instant::now() >= deadline {
                break;
            }
        }
        for line in &self.recent {
            eprintln!("{line}");
        }
        bail!("Minecraft test server did not complete {phase} within {timeout:?}");
    }

    fn load_origin(&mut self, timeout: Duration) -> Result<()> {
        self.send("execute in minecraft:overworld run forceload add 0 0")?;
        let deadline = Instant::now() + timeout;
        loop {
            let time_left = deadline.saturating_duration_since(Instant::now());
            ensure!(
                !time_left.is_zero(),
                "Minecraft did not load the test chunk"
            );
            self.send(
                "execute in minecraft:overworld if loaded 0 80 0 run say __SAND_TEST_READY__",
            )?;
            self.send("say __SAND_TEST_SETUP_DONE__")?;
            let mut ready = false;
            self.wait_for(time_left, "test world setup", |line| {
                ready |= line.ends_with(" __SAND_TEST_READY__");
                line.ends_with(" __SAND_TEST_SETUP_DONE__")
            })?;
            if ready {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn stop(&mut self, timeout: Duration) -> Result<()> {
        self.send("stop")?;
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                ensure!(
                    status.success(),
                    "Minecraft test server stopped with {status}"
                );
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        bail!("Minecraft test server did not stop within {timeout:?}");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
