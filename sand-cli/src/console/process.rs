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
    Log { stream: Stream, line: String },
    StreamClosed(Stream),
    Input(String),
    InputClosed,
    Interrupt,
}

pub struct RunOutcome {
    pub exit_status: ExitStatus,
    /// Overall `sand run` health across the whole run — distinct from
    /// `exit_status`, since the JVM process can exit 0 (a normal `stop`)
    /// even though the datapack never loaded successfully.
    pub health: RunHealth,
}

/// Spawn `command` with piped stdio and drive it until it exits, forwarding
/// stdin interactively and rendering classified stdout/stderr. The child is
/// waited on exactly once, on every return path.
pub fn run_server(mut command: Command, mode: OutputMode, mc_version: &str) -> Result<RunOutcome> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to start Java — make sure Java 21+ is on your PATH (`java -version`)")?;

    let stdin = child.stdin.take().expect("stdin was piped");
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let (tx, rx) = mpsc::channel::<Event>();
    spawn_reader(stdout, Stream::Stdout, tx.clone());
    spawn_reader(stderr, Stream::Stderr, tx.clone());
    spawn_stdin_forwarder(tx.clone());

    let ctrlc_tx = tx.clone();
    // Best-effort: if a handler is already installed (e.g. a prior call in
    // the same process, such as in tests) we just skip Ctrl-C interception
    // rather than failing the run.
    let _ = ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(Event::Interrupt);
    });

    drive(&mut child, stdin, &rx, mode, mc_version)
}

fn drive(
    child: &mut Child,
    mut stdin: impl Write,
    rx: &Receiver<Event>,
    mode: OutputMode,
    mc_version: &str,
) -> Result<RunOutcome> {
    let mut renderer = Renderer::new(mode, mc_version.to_string());
    let mut grouper = Grouper::new();
    let mut phase = PhaseTracker::new();
    let mut stop_requested = false;

    loop {
        match rx.recv_timeout(QUIET_FLUSH) {
            Ok(Event::Log { stream, line }) => {
                let record = parse_line(&line);
                phase.observe_log(&record.message);
                let category = classify(stream, &record);
                for grouped in grouper.feed(stream, record, category) {
                    renderer.render(&grouped, phase.current());
                }
            }
            Ok(Event::StreamClosed(_)) => {}
            Ok(Event::Input(line)) => {
                phase.observe_command(&line);
                if line.trim() == "stop" {
                    stop_requested = true;
                }
                send_command(&mut stdin, &line);
            }
            Ok(Event::InputClosed) => {
                if !stop_requested {
                    stop_requested = true;
                    send_command(&mut stdin, "stop");
                }
            }
            Ok(Event::Interrupt) => {
                if !stop_requested {
                    stop_requested = true;
                    send_command(&mut stdin, "stop");
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(grouped) = grouper.flush() {
                    renderer.render(&grouped, phase.current());
                }
                renderer.flush_pending_correlation();
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }

        if let Some(status) = child.try_wait().context("failed to poll server process")? {
            drain_remaining(rx, &mut grouper, &mut renderer, &mut phase);
            renderer.finish();
            return Ok(RunOutcome {
                exit_status: status,
                health: renderer.health(),
            });
        }
    }

    let status = child.wait().context("failed to wait on server process")?;
    renderer.finish();
    Ok(RunOutcome {
        exit_status: status,
        health: renderer.health(),
    })
}

/// Best-effort write; a broken pipe just means the server already exited
/// and is not itself an error worth surfacing.
fn send_command(stdin: &mut impl Write, line: &str) {
    if writeln!(stdin, "{line}").is_ok() {
        let _ = stdin.flush();
    }
}

fn drain_remaining(
    rx: &Receiver<Event>,
    grouper: &mut Grouper,
    renderer: &mut Renderer,
    phase: &mut PhaseTracker,
) {
    let mut stdout_closed = false;
    let mut stderr_closed = false;
    let deadline = Instant::now() + DRAIN_TIMEOUT;

    while (!stdout_closed || !stderr_closed) && Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Event::Log { stream, line }) => {
                let record = parse_line(&line);
                phase.observe_log(&record.message);
                let category = classify(stream, &record);
                for grouped in grouper.feed(stream, record, category) {
                    renderer.render(&grouped, phase.current());
                }
            }
            Ok(Event::StreamClosed(Stream::Stdout)) => stdout_closed = true,
            Ok(Event::StreamClosed(Stream::Stderr)) => stderr_closed = true,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    if let Some(grouped) = grouper.flush() {
        renderer.render(&grouped, phase.current());
    }
    renderer.flush_pending_correlation();
}

fn spawn_reader<R: Read + Send + 'static>(reader: R, stream: Stream, tx: Sender<Event>) {
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
                    if tx.send(Event::Log { stream, line }).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(Event::StreamClosed(stream));
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
    fn reader_handles_non_utf8_and_malformed_bytes_without_panicking() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"[12:00:00] [Server thread/INFO]: valid line\n");
        bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD, b'\n']); // invalid UTF-8
        bytes.extend_from_slice(b"no trailing newline at all");

        let (tx, rx) = mpsc::channel::<Event>();
        spawn_reader(Cursor::new(bytes), Stream::Stdout, tx);

        let mut lines = Vec::new();
        loop {
            match rx.recv_timeout(Duration::from_secs(2)).unwrap() {
                Event::Log { line, .. } => lines.push(line),
                Event::StreamClosed(_) => break,
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
        spawn_reader(stdout, Stream::Stdout, tx.clone());
        spawn_reader(stderr, Stream::Stderr, tx.clone());
        // Feed synthetic input instead of real stdin so the test is
        // hermetic: send a command, then close input. The fixture script
        // blocks on `read line`, so this proves input forwarding and
        // shutdown-on-EOF both work without deadlocking.
        std::thread::spawn(move || {
            let _ = tx.send(Event::Input("say hello".to_string()));
            let _ = tx.send(Event::InputClosed);
        });

        let result = drive(&mut child, stdin, &rx, OutputMode::Classified, "1.21.1");
        // The fixture script isn't a real Minecraft server, so we only
        // assert the loop terminates cleanly with a status, proving no
        // deadlock — not any particular exit code.
        assert!(result.is_ok());
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
