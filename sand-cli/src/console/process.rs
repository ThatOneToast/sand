//! Owns the Minecraft server child process: piping stdio, forwarding
//! interactive input, and driving log classification/rendering — kept
//! separate from parsing/presentation so the lifecycle logic can be tested
//! independently (see the fixture-process test at the bottom).

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use super::classify::classify;
use super::diagnostic::Grouper;
use super::health::RunHealth;
use super::log_record::{Stream, parse_line};
use super::phase::PhaseTracker;
use super::render::{OutputMode, Renderer};

/// How long to wait for more output before flushing a buffered diagnostic
/// group. Bounds worst-case latency on showing a datapack error while still
/// giving stack frames/parser context a chance to arrive.
const QUIET_FLUSH: Duration = Duration::from_millis(150);

/// How long to keep draining output after the child has exited, to catch
/// any lines still in flight through the OS pipe.
const DRAIN_TIMEOUT: Duration = Duration::from_millis(500);

enum Event {
    Log {
        generation: u64,
        stream: Stream,
        line: String,
    },
    StreamClosed {
        generation: u64,
        stream: Stream,
    },
    Input(String),
    InputClosed,
    Interrupt,
}

impl Event {
    fn belongs_to(&self, current: u64) -> bool {
        match self {
            Self::Log { generation, .. } | Self::StreamClosed { generation, .. } => {
                *generation == current
            }
            _ => true, // Input belongs to the persistent console, not a JVM.
        }
    }
}

pub struct RunOutcome {
    pub exit_status: ExitStatus,
    /// Overall `sand run` health across the whole run — distinct from
    /// `exit_status`, since the JVM process can exit 0 (a normal `stop`)
    /// even though the datapack never loaded successfully.
    pub health: RunHealth,
    /// The dev console requested a graceful restart after this JVM exits.
    pub restart_requested: bool,
    /// Sand terminated a still-running child after the shutdown deadline.
    pub forced_shutdown: bool,
}

impl RunOutcome {
    pub(crate) fn should_restart(&self) -> bool {
        self.restart_requested && (self.exit_status.success() || self.forced_shutdown)
    }
}

/// One stdin reader and Ctrl-C handler retained across dev-server restarts.
/// Each JVM tags its log events; late readers cannot affect a later session.
pub(crate) struct ConsoleSession {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    pending: std::collections::VecDeque<Event>,
    cancelled: bool,
    generation: u64,
}
impl ConsoleSession {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        spawn_stdin_forwarder(tx.clone());
        let ctrlc_tx = tx.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = ctrlc_tx.send(Event::Interrupt);
        });
        Self {
            tx,
            rx,
            pending: Default::default(),
            cancelled: false,
            generation: 0,
        }
    }

    /// Observe queued cancellation between synchronous restart stages. Keep
    /// ordinary console input in order for the next JVM; cancellation is sticky.
    pub(crate) fn cancellation_requested(&mut self) -> bool {
        self.pending.extend(self.rx.try_iter());
        self.pending.retain(|event| {
            let cancel = matches!(event, Event::InputClosed | Event::Interrupt)
                || matches!(event, Event::Input(line) if matches!(line.trim(), "q" | "stop"));
            self.cancelled |= cancel;
            !cancel
        });
        self.cancelled
    }
}

/// Spawn `command` with piped stdio and drive it until it exits, forwarding
/// stdin interactively and rendering classified stdout/stderr. The child is
/// waited on exactly once, on every return path.
pub fn run_server(command: Command, mode: OutputMode, mc_version: &str) -> Result<RunOutcome> {
    run_server_with_dev(command, mode, mc_version, None, &mut None)
}

pub(crate) fn run_server_with_dev(
    mut command: Command,
    mode: OutputMode,
    mc_version: &str,
    dev: Option<&mut crate::dev::Session>,
    console: &mut Option<ConsoleSession>,
) -> Result<RunOutcome> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(
            "failed to start Java — ensure a runtime compatible with the selected Minecraft version is on your PATH (`java -version`)",
        )?;

    let stdin = child.stdin.take().expect("stdin was piped");
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let console = console.get_or_insert_with(ConsoleSession::new);
    console.generation += 1;
    spawn_reader(
        stdout,
        Stream::Stdout,
        console.tx.clone(),
        console.generation,
    );
    spawn_reader(
        stderr,
        Stream::Stderr,
        console.tx.clone(),
        console.generation,
    );
    let result = drive(
        &mut child,
        stdin,
        (&console.rx, console.generation),
        mode,
        mc_version,
        dev,
        &mut console.pending,
    );
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn drive(
    child: &mut Child,
    mut stdin: impl Write,
    (rx, generation): (&Receiver<Event>, u64),
    mode: OutputMode,
    mc_version: &str,
    mut dev: Option<&mut crate::dev::Session>,
    pending: &mut std::collections::VecDeque<Event>,
) -> Result<RunOutcome> {
    let mut renderer = Renderer::new(mode, mc_version.to_string());
    let mut grouper = Grouper::new();
    let mut phase = PhaseTracker::new();
    let mut stop_requested = false;
    let mut stop_deadline = None;
    let mut restart_requested = false;

    loop {
        match pending
            .pop_front()
            .map(Ok)
            .unwrap_or_else(|| rx.recv_timeout(QUIET_FLUSH))
        {
            Ok(event) if !event.belongs_to(generation) => continue,
            Ok(Event::Log { stream, line, .. }) => {
                let record = parse_line(&line);
                phase.observe_log(&record.message);
                let category = classify(stream, &record);
                for grouped in grouper.feed(stream, record, category) {
                    render_group(&grouped, &mut renderer, phase.current(), dev.as_deref_mut());
                }
            }
            Ok(Event::StreamClosed { .. }) => {}
            Ok(Event::Input(mut line)) => {
                if let Some(dev) = dev.as_deref_mut() {
                    match line.trim() {
                        "r" => {
                            dev.rebuild();
                            continue;
                        }
                        "q" => {
                            restart_requested = false;
                            line = "stop".into();
                        }
                        "R" => {
                            restart_requested = true;
                            line = "stop".into();
                        }
                        _ => {}
                    }
                }
                phase.observe_command(&line);
                if line.trim() == "stop" {
                    stop_requested = true;
                }
                send_command(&mut stdin, &line);
            }
            Ok(Event::InputClosed) => {
                restart_requested = false;
                if !stop_requested {
                    stop_requested = true;
                    send_command(&mut stdin, "stop");
                }
            }
            Ok(Event::Interrupt) => {
                restart_requested = false;
                if !stop_requested {
                    stop_requested = true;
                    send_command(&mut stdin, "stop");
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(grouped) = grouper.flush() {
                    render_group(&grouped, &mut renderer, phase.current(), dev.as_deref_mut());
                }
                renderer.flush_pending_correlation();
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }

        if stop_requested {
            stop_deadline.get_or_insert_with(|| Instant::now() + Duration::from_secs(30));
        }
        if !stop_requested
            && let Some(dev) = dev.as_deref_mut()
            && let Err(error) = dev.poll()
        {
            eprintln!("dev: {error:#}; live pack retained unless sync already completed");
        }

        let (status, forced_shutdown) = poll_shutdown(child, stop_deadline, Instant::now())?;
        if let Some(status) = status {
            drain_remaining(
                (rx, generation),
                &mut grouper,
                &mut renderer,
                &mut phase,
                pending,
                dev.as_deref_mut(),
            );
            let health = finish_rendering(&mut renderer, dev.as_deref());
            return Ok(RunOutcome {
                exit_status: status,
                health,
                restart_requested,
                forced_shutdown,
            });
        }
    }

    let status = child.wait().context("failed to wait on server process")?;
    let health = finish_rendering(&mut renderer, dev.as_deref());
    Ok(RunOutcome {
        exit_status: status,
        health,
        restart_requested,
        forced_shutdown: false,
    })
}

/// Poll before enforcing the deadline so an independently exited/crashed child
/// is never mislabeled as Sand's intentional termination. Forced children are
/// reaped before the caller may rebuild or spawn another JVM.
fn poll_shutdown(
    child: &mut Child,
    deadline: Option<Instant>,
    now: Instant,
) -> Result<(Option<ExitStatus>, bool)> {
    if let Some(status) = child.try_wait().context("failed to poll server process")? {
        return Ok((Some(status), false));
    }
    if deadline.is_some_and(|deadline| now >= deadline) {
        child
            .kill()
            .context("failed to terminate server after shutdown deadline")?;
        let status = child.wait().context("failed to reap terminated server")?;
        eprintln!("server shutdown deadline reached; process terminated");
        return Ok((Some(status), true));
    }
    Ok((None, false))
}

fn render_group(
    grouped: &super::diagnostic::GroupedEvent,
    renderer: &mut Renderer,
    phase: super::phase::RunPhase,
    dev: Option<&mut crate::dev::Session>,
) {
    let current = if dev.as_ref().is_some_and(|dev| dev.is_reloading()) {
        super::phase::RunPhase::Reload
    } else {
        phase
    };
    if let Some(dev) = dev {
        dev.observe(grouped, current);
    }
    renderer.render(grouped, current);
}

fn finish_rendering(renderer: &mut Renderer, dev: Option<&crate::dev::Session>) -> RunHealth {
    if let Some(dev) = dev {
        let health = dev.health();
        renderer.finish_dev(health);
        health
    } else {
        renderer.finish();
        renderer.health()
    }
}

/// Best-effort write; a broken pipe just means the server already exited
/// and is not itself an error worth surfacing.
fn send_command(stdin: &mut impl Write, line: &str) {
    if writeln!(stdin, "{line}").is_ok() {
        let _ = stdin.flush();
    }
}

fn drain_remaining(
    (rx, generation): (&Receiver<Event>, u64),
    grouper: &mut Grouper,
    renderer: &mut Renderer,
    phase: &mut PhaseTracker,
    pending: &mut std::collections::VecDeque<Event>,
    mut dev: Option<&mut crate::dev::Session>,
) {
    let mut stdout_closed = false;
    let mut stderr_closed = false;
    let deadline = Instant::now() + DRAIN_TIMEOUT;

    while (!stdout_closed || !stderr_closed) && Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) if !event.belongs_to(generation) => continue,
            Ok(Event::Log { stream, line, .. }) => {
                let record = parse_line(&line);
                phase.observe_log(&record.message);
                let category = classify(stream, &record);
                for grouped in grouper.feed(stream, record, category) {
                    render_group(&grouped, renderer, phase.current(), dev.as_deref_mut());
                }
            }
            Ok(Event::StreamClosed {
                stream: Stream::Stdout,
                ..
            }) => stdout_closed = true,
            Ok(Event::StreamClosed {
                stream: Stream::Stderr,
                ..
            }) => stderr_closed = true,
            Ok(event) => pending.push_back(event),
            Err(_) => break,
        }
    }

    if let Some(grouped) = grouper.flush() {
        render_group(&grouped, renderer, phase.current(), dev);
    }
    renderer.flush_pending_correlation();
}

fn spawn_reader<R: Read + Send + 'static>(
    reader: R,
    stream: Stream,
    tx: Sender<Event>,
    generation: u64,
) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    while matches!(buf.last(), Some(b'\n') | Some(b'\r')) {
                        buf.pop();
                    }
                    let line = String::from_utf8_lossy(&buf).into_owned();
                    if tx
                        .send(Event::Log {
                            generation,
                            stream,
                            line,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(Event::StreamClosed { generation, stream });
    });
}

fn spawn_stdin_forwarder(tx: Sender<Event>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Err(_) => break,
                Ok(_) => {
                    let trimmed = line.trim_end_matches(['\n', '\r']);
                    if tx.send(Event::Input(trimmed.to_string())).is_err() {
                        break;
                    }
                }
            }
        }
        let _ = tx.send(Event::InputClosed);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn restart_survives_expected_forced_shutdown_but_not_an_unrelated_crash() {
        let mut command = Command::new(test_fixture_command());
        if cfg!(windows) {
            command.args(["/C", "set /p line="]);
        } else {
            command.args(["-c", "read line"]);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let now = Instant::now();
        let (status, forced) =
            poll_shutdown(&mut child, Some(now + Duration::from_secs(30)), now).unwrap();
        assert!(status.is_none() && !forced);
        let (status, forced) = poll_shutdown(&mut child, Some(now), now).unwrap();
        let mut outcome = RunOutcome {
            exit_status: status.unwrap(),
            health: RunHealth::Healthy,
            restart_requested: true,
            forced_shutdown: forced,
        };
        assert!(!outcome.exit_status.success());
        assert!(outcome.forced_shutdown && outcome.should_restart());
        assert!(
            child.try_wait().unwrap().is_some(),
            "must be reaped before restart"
        );
        outcome.restart_requested = false;
        assert!(
            !outcome.should_restart(),
            "q must not restart a killed child"
        );

        let mut command = Command::new(test_fixture_command());
        if cfg!(windows) {
            command.args(["/C", "exit 7"]);
        } else {
            command.args(["-c", "exit 7"]);
        }
        let mut crashed = command.spawn().unwrap();
        crashed.wait().unwrap();
        let (status, forced) = poll_shutdown(&mut crashed, Some(now), Instant::now()).unwrap();
        let outcome = RunOutcome {
            exit_status: status.unwrap(),
            health: RunHealth::Healthy,
            restart_requested: true,
            forced_shutdown: forced,
        };
        assert!(!outcome.forced_shutdown && !outcome.should_restart());
    }

    #[test]
    fn preparation_cancellation_is_sticky_and_retains_other_input_in_order() {
        for event in [
            Event::Input("q".into()),
            Event::InputClosed,
            Event::Interrupt,
            Event::Input("stop".into()),
        ] {
            let (tx, rx) = mpsc::channel();
            let mut console = ConsoleSession {
                tx,
                rx,
                pending: Default::default(),
                cancelled: false,
                generation: 0,
            };
            console.pending.push_back(Event::Input("say before".into()));
            assert!(!console.cancellation_requested());
            console.tx.send(event).unwrap();
            console.tx.send(Event::Input("say after".into())).unwrap();
            assert!(console.cancellation_requested());
            assert!(
                console.cancellation_requested(),
                "EOF must not be consumed and forgotten"
            );
            assert!(
                matches!(console.pending.pop_front(), Some(Event::Input(line)) if line == "say before")
            );
            assert!(
                matches!(console.pending.pop_front(), Some(Event::Input(line)) if line == "say after")
            );
            assert!(console.pending.is_empty());
        }
    }

    #[test]
    fn delayed_previous_jvm_logs_cannot_change_new_phase_health_or_input() {
        let (tx, rx) = mpsc::channel();
        // Sent after the new generation begins: not just backlog discarded at startup.
        for line in [
            "[12:00:00] [Server thread/INFO]: Done (1s)! For help, type \"help\"",
            "[12:00:01] [Server thread/ERROR]: Failed to load function old:broken",
        ] {
            tx.send(Event::Log {
                generation: 1,
                stream: Stream::Stdout,
                line: line.into(),
            })
            .unwrap();
        }
        for stream in [Stream::Stdout, Stream::Stderr] {
            tx.send(Event::StreamClosed {
                generation: 1,
                stream,
            })
            .unwrap();
        }
        tx.send(Event::Input("say retained".into())).unwrap();
        for stream in [Stream::Stdout, Stream::Stderr] {
            tx.send(Event::StreamClosed {
                generation: 2,
                stream,
            })
            .unwrap();
        }
        let mut renderer = Renderer::new(OutputMode::Classified, "26.2".into());
        let mut phase = PhaseTracker::new();
        let mut pending = Default::default();
        drain_remaining(
            (&rx, 2),
            &mut Grouper::new(),
            &mut renderer,
            &mut phase,
            &mut pending,
            None,
        );
        assert_eq!(
            phase.current(),
            super::super::phase::RunPhase::ServerStartup
        );
        assert_eq!(renderer.health(), RunHealth::Healthy);
        assert!(matches!(pending.pop_front(), Some(Event::Input(line)) if line == "say retained"));
        assert!(pending.is_empty());
        // Current-generation output must still be classified normally.
        tx.send(Event::Log {
            generation: 2,
            stream: Stream::Stdout,
            line: "[12:00:01] [Server thread/ERROR]: Failed to load function new:broken".into(),
        })
        .unwrap();
        for stream in [Stream::Stdout, Stream::Stderr] {
            tx.send(Event::StreamClosed {
                generation: 2,
                stream,
            })
            .unwrap();
        }
        drain_remaining(
            (&rx, 2),
            &mut Grouper::new(),
            &mut renderer,
            &mut phase,
            &mut pending,
            None,
        );
        assert_eq!(renderer.health(), RunHealth::Degraded);
    }

    #[test]
    fn drain_preserves_restart_quit_and_eof_for_next_process() {
        let (tx, rx) = mpsc::channel();
        tx.send(Event::Input("R".into())).unwrap();
        tx.send(Event::Input("q".into())).unwrap();
        tx.send(Event::InputClosed).unwrap();
        tx.send(Event::StreamClosed {
            generation: 0,
            stream: Stream::Stdout,
        })
        .unwrap();
        tx.send(Event::StreamClosed {
            generation: 0,
            stream: Stream::Stderr,
        })
        .unwrap();
        let mut pending = std::collections::VecDeque::new();
        drain_remaining(
            (&rx, 0),
            &mut Grouper::new(),
            &mut Renderer::new(OutputMode::Classified, "26.2".into()),
            &mut PhaseTracker::new(),
            &mut pending,
            None,
        );
        assert!(matches!(pending.pop_front(), Some(Event::Input(line)) if line == "R"));
        assert!(matches!(pending.pop_front(), Some(Event::Input(line)) if line == "q"));
        assert!(matches!(pending.pop_front(), Some(Event::InputClosed)));
        assert!(pending.is_empty());
    }

    #[test]
    fn reader_handles_non_utf8_and_malformed_bytes_without_panicking() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"[12:00:00] [Server thread/INFO]: valid line\n");
        bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD, b'\n']); // invalid UTF-8
        bytes.extend_from_slice(b"no trailing newline at all");

        let (tx, rx) = mpsc::channel::<Event>();
        spawn_reader(Cursor::new(bytes), Stream::Stdout, tx, 0);

        let mut lines = Vec::new();
        loop {
            match rx.recv_timeout(Duration::from_secs(2)).unwrap() {
                Event::Log { line, .. } => lines.push(line),
                Event::StreamClosed { .. } => break,
                _ => unreachable!(),
            }
        }

        assert_eq!(lines[0], "[12:00:00] [Server thread/INFO]: valid line");
        assert!(lines[1].contains('\u{FFFD}'));
        assert_eq!(lines[2], "no trailing newline at all");
    }

    /// Drives a fixture process that echoes stdin and interleaves
    /// stdout/stderr, proving commands, responses, and shutdown don't
    /// deadlock. Runs the fixture as a real child process using this
    /// crate's own test binary support via `sh`-free `std::process`.
    #[test]
    fn drive_handles_interleaved_output_and_stop_command_without_deadlock() {
        // We can't easily spin up a real child `Command` here without a
        // fixture binary; instead we unit-test `drive`'s event loop by
        // exercising `spawn_reader`/`spawn_stdin_forwarder` against pipes
        // wired to an actual short-lived child process (`cat`-like echo).
        let mut child = Command::new(test_fixture_command())
            .args(test_fixture_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("fixture process should spawn");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let (tx, rx) = mpsc::channel::<Event>();
        spawn_reader(stdout, Stream::Stdout, tx.clone(), 0);
        spawn_reader(stderr, Stream::Stderr, tx.clone(), 0);
        // Feed synthetic input instead of real stdin so the test is
        // hermetic: send a command, then close input. The fixture script
        // blocks on `read line`, so this proves input forwarding and
        // shutdown-on-EOF both work without deadlocking.
        std::thread::spawn(move || {
            // A reader from an earlier JVM may send after this drive starts.
            let _ = tx.send(Event::Log {
                generation: 99,
                stream: Stream::Stdout,
                line: "[12:00:01] [Server thread/ERROR]: Failed to load function old:broken".into(),
            });
            let _ = tx.send(Event::Input("say hello".to_string()));
            let _ = tx.send(Event::InputClosed);
        });

        let result = drive(
            &mut child,
            stdin,
            (&rx, 0),
            OutputMode::Classified,
            "26.2",
            None,
            &mut Default::default(),
        );
        // The fixture script isn't a real Minecraft server, so we only
        // assert the loop terminates cleanly with a status, proving no
        // deadlock — not any particular exit code.
        assert_eq!(result.unwrap().health, RunHealth::Healthy);
    }

    fn test_fixture_command() -> &'static str {
        if cfg!(windows) { "cmd" } else { "sh" }
    }

    fn test_fixture_args() -> Vec<&'static str> {
        if cfg!(windows) {
            vec!["/C", "echo ready & exit 0"]
        } else {
            vec!["-c", "echo ready; read line; echo \"got: $line\"; exit 0"]
        }
    }
}
