#!/usr/bin/env python3
"""Exercise sand dev against vanilla 26.2; retains logs but never RCON secrets.

Run after cargo build -p sand-cli --bin sand. Requires Java 25 on PATH.
The fixture uses a temporary project and Sand's normal server jar cache.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import signal
import socket
import subprocess
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[1]

def process_group_options():
    if os.name == "nt":
        return {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
    return {"start_new_session": True}


def kill_process_tree(process):
    """Kill only this validation subprocess's dedicated process group."""
    if os.name == "nt":
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(process.pid)],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10)
    else:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    process.wait(timeout=10)


def run_bounded(command, timeout=300, **kwargs):
    process = subprocess.Popen(command, **kwargs, **process_group_options())
    try:
        output, _ = process.communicate(timeout=timeout)
        if process.returncode:
            raise subprocess.CalledProcessError(process.returncode, command, output=output)
        return output
    finally:
        kill_process_tree(process)
        for pipe in (process.stdin, process.stdout, process.stderr):
            if pipe is not None:
                pipe.close()


def snapshot(path):
    return {str(p.relative_to(path)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in path.rglob('*') if p.is_file()}


def fixture(path):
    (path / 'src/target').mkdir(parents=True)
    (path / 'src/bin').mkdir()
    (path / 'Cargo.toml').write_text(f'''[package]
name = "devprobe"
version = "0.1.0"
edition = "2024"
[workspace]
[dependencies]
sand = {{ path = {json.dumps(str(ROOT / 'sand'))} }}
[[bin]]
name = "sand_export"
path = "src/bin/sand_export.rs"
''')
    (path / 'sand.toml').write_text('[pack]\nnamespace="devprobe"\ndescription="Dev probe"\nmc_version="26.2"\n')
    (path / 'src/lib.rs').write_text('''#[path="target/mod.rs"]
mod authored;
pub use authored::*;
pub fn export(version: &str) {
    println!("{}", sand::advanced::try_export_pack_json("devprobe", version).unwrap());
}
''')
    (path / 'src/bin/sand_export.rs').write_text('fn main() { devprobe::export(&std::env::var("SAND_EXPORT_MC_VERSION").unwrap()); }')
    (path / 'Cargo.lock').write_bytes((ROOT / 'Cargo.lock').read_bytes())
    source(path, 'v1')
    server = path / 'dist/server'
    server.mkdir(parents=True)
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    (server / 'server.properties').write_text(f'server-port={port}\nserver-ip=0.0.0.0\nlevel-type=minecraft:flat\n')
    return port


def source(path, version):
    (path / 'src/target/mod.rs').write_text(f'''use sand::prelude::*;
#[function]
pub fn probe() {{ cmd::say("dev_probe_{version}"); }}
''')


def resolve_fixture_lock(project, env):
    """Finish fixture dependency resolution before the watcher takes its baseline."""
    # The workspace lock seeds pinned versions, but names a different root
    # package. --no-deps metadata does not update it; the first build otherwise
    # changes a watched input and correctly queues an unrelated startup rebuild.
    run_bounded(['cargo', 'metadata', '--offline', '--format-version', '1'],
                cwd=project, env=env, stdout=subprocess.DEVNULL)


class Dev:
    def __init__(self, binary, project, log, env, mode="dev"):
        self.lines = []
        self.events = queue.Queue()
        self.log = log
        self.process = subprocess.Popen([str(binary), mode, '--offline', '--ram', '1G'],
                                        cwd=project, env=env, stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                        text=True, bufsize=1, **process_group_options())
        self.thread = threading.Thread(target=self.read, daemon=True)
        self.thread.start()

    def read(self):
        with self.log.open('w') as output:
            for line in self.process.stdout:
                output.write(line)
                output.flush()
                self.lines.append(line)
                self.events.put(line)

    def wait(self, text, timeout=300):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                line = self.events.get(timeout=0.2)
                if text in line:
                    return line
            except queue.Empty:
                if self.process.poll() is not None:
                    raise AssertionError(f'dev exited {self.process.returncode} waiting for {text}; see {self.log}')
        raise AssertionError(f'timed out waiting for {text}; see {self.log}')

    def send(self, command):
        self.process.stdin.write(command + '\n')
        self.process.stdin.flush()

    def close(self, timeout=45):
        if self.process.poll() is None:
            try:
                self.send('q')
                self.process.wait(timeout=timeout)
            except (BrokenPipeError, ValueError, subprocess.TimeoutExpired):
                pass
        kill_process_tree(self.process)
        self.thread.join(timeout=5)
        for pipe in (self.process.stdin, self.process.stdout, self.process.stderr):
            if pipe is not None:
                pipe.close()


def properties(project):
    text = (project / 'dist/server/server.properties').read_text()
    return dict(line.split('=', 1) for line in text.splitlines() if '=' in line and not line.startswith('#'))


def wait_for_file(path, process, timeout=120):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.exists():
            return
        assert process.poll() is None, f'dev exited while waiting for {path.name}'
        time.sleep(0.02)
    raise AssertionError(f'timed out waiting for {path.name}')


def cancellation_fixture(project):
    """Block restart export before an AlwaysReset world can be removed."""
    with (project / 'Cargo.toml').open('a') as manifest:
        manifest.write('\n[[bin]]\nname="sand_build_world"\npath="sand.build.rs"\n')
    (project / 'sand.build.rs').write_text('''use sand::build::{BuildContext, BuildProfile, SandBuild, ServerConfig, WorldResetPolicy};
fn main() {
    let context = BuildContext::new(BuildProfile::parse("dev")).with_mc_version("26.2");
    sand::build::run_and_print("devprobe", context, |_| SandBuild::new().server(
        ServerConfig::new().world_reset_policy(WorldResetPolicy::AlwaysReset)));
}
''')
    block_restart_export(project)


def block_restart_export(project):
    (project / 'src/bin/sand_export.rs').write_text('''fn main() {
    let flag = std::path::Path::new("dist/block-restart-build");
    if flag.exists() {
        std::fs::write("dist/restart-build-blocked", "waiting").unwrap();
        let start = std::time::Instant::now();
        while !std::path::Path::new("dist/release-restart-build").exists() {
            assert!(start.elapsed().as_secs() < 120, "fixture release timeout");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    devprobe::export(&std::env::var("SAND_EXPORT_MC_VERSION").unwrap());
}
''')


def validate_restart_cancellation(binary, project, output, env, secrets, checks):
    cancellation_fixture(project)
    kinds = ['q', 'eof'] + ([] if os.name == 'nt' else ['interrupt'])
    for kind in kinds:
        dev = Dev(binary, project, output / f'cancel-{kind}-console.log', env)
        flags = [project / 'dist' / name for name in
                 ['block-restart-build', 'restart-build-blocked', 'release-restart-build']]
        try:
            dev.wait('Minecraft 26.2 ready')
            secrets.append(properties(project)['rcon.password'])
            marker = project / 'dist/server/world/retain-on-cancel'
            marker.write_text('world must survive queued cancellation')
            flags[0].touch()
            dev.send('R')
            dev.wait('server stopped; rebuilding and restarting')
            wait_for_file(flags[1], dev.process)
            world = snapshot(project / 'dist/server/world')
            if kind == 'q':
                dev.send('q')
            elif kind == 'eof':
                dev.process.stdin.close()
            else:
                os.kill(dev.process.pid, signal.SIGINT)
            # Let the console thread queue the event while preparation is
            # deliberately blocked; the production checkpoint must consume it.
            time.sleep(0.1)
            flags[2].touch()
            dev.wait('dev: restart cancelled')
            assert dev.process.wait(timeout=45) == 0
            assert marker.is_file(), 'AlwaysReset deleted the world after cancellation'
            assert snapshot(project / 'dist/server/world') == world
            assert sum('Minecraft 26.2 ready' in line for line in dev.lines) == 1
            assert 'rcon.password' not in properties(project)
            assert properties(project)['server-ip'] == '0.0.0.0'
            checks.append(f'{kind} during restart build prevents AlwaysReset and new JVM launch')
        finally:
            flags[2].touch()
            dev.close()
            for flag in flags:
                flag.unlink(missing_ok=True)


def validate_initial_cancellation(binary, project, output, env, checks):
    """Capture cancellation on the first build, before dev mutates the server."""
    for kind in ['q', 'eof'] + ([] if os.name == 'nt' else ['interrupt']):
        flags = [project / 'dist' / name for name in
                 ['block-restart-build', 'restart-build-blocked', 'release-restart-build']]
        flags[0].touch()
        before = snapshot(project / 'dist/server')
        dev = Dev(binary, project, output / f'initial-cancel-{kind}-console.log', env)
        try:
            wait_for_file(flags[1], dev.process)
            if kind == 'q':
                dev.send('q')
            elif kind == 'eof':
                dev.process.stdin.close()
            else:
                os.kill(dev.process.pid, signal.SIGINT)
            time.sleep(0.1)
            flags[2].touch()
            assert dev.process.wait(timeout=45) == 0
            assert snapshot(project / 'dist/server') == before
            assert not any('Minecraft 26.2 ready' in line for line in dev.lines)
            checks.append(f'{kind} during initial build preserves server files and prevents JVM launch')
        finally:
            flags[2].touch()
            dev.close()
            for flag in flags:
                flag.unlink(missing_ok=True)


def validate_ordinary_shutdown(binary, project, output, env, checks):
    if os.name == 'nt':
        return  # Deterministic portable fake-child coverage remains in Rust.
    props_path = project / 'dist/server/server.properties'
    original = props_path.read_text()
    props_path.write_text(original.replace('server-ip=0.0.0.0', 'server-ip=127.0.0.1'))
    run = Dev(binary, project, output / 'ordinary-run-console.log', env, mode='run')
    frozen = None
    try:
        run.wait('Minecraft 26.2 ready')
        rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,comm='], text=True)
        children = [int(parts[0]) for row in rows.splitlines()
                    if len(parts := row.strip().split(None, 2)) == 3
                    and int(parts[1]) == run.process.pid and Path(parts[2]).name == 'java']
        assert len(children) == 1
        frozen = children[0]
        os.kill(frozen, signal.SIGSTOP)
        run.send('stop')
        time.sleep(32)
        assert run.process.poll() is None, 'ordinary run was force-killed during graceful shutdown'
        os.kill(frozen, 0)
        os.kill(frozen, signal.SIGCONT)
        frozen = None
        assert run.process.wait(timeout=45) == 0
        assert not any('shutdown deadline reached' in line for line in run.lines)
        checks.append('ordinary sand run survives beyond 30 seconds during shutdown then exits gracefully')
    finally:
        if frozen is not None:
            try:
                os.kill(frozen, signal.SIGCONT)
            except ProcessLookupError:
                pass
        if run.process.poll() is None:
            run.send('stop')
        run.close()
        props_path.write_text(original)


def validate_namespace_replacement(binary, project, output, env, secrets, checks):
    block_restart_export(project)
    dev = Dev(binary, project, output / 'namespace-replacement-console.log', env)
    config = project / 'sand.toml'
    library = project / 'src/lib.rs'
    original_config, original_library = config.read_text(), library.read_text()
    flags = [project / 'dist' / name for name in
             ['block-restart-build', 'restart-build-blocked', 'release-restart-build']]
    try:
        dev.wait('Minecraft 26.2 ready')
        secrets.append(properties(project)['rcon.password'])
        config.write_text(original_config.replace('namespace="devprobe"', 'namespace="renamed"'))
        library.write_text(original_library.replace('try_export_pack_json("devprobe"', 'try_export_pack_json("renamed"'))
        dev.wait('namespace changed: restart required')
        flags[0].touch()
        dev.send('R')
        dev.wait('server stopped; rebuilding and restarting')
        wait_for_file(flags[1], dev.process)
        live = project / 'dist/server/world/datapacks/devprobe'
        import shutil
        shutil.rmtree(live)
        live.mkdir()
        (live / 'operator-data').write_text('preserve replacement')
        flags[2].touch()
        dev.wait('previous managed datapack was replaced')
        assert dev.process.wait(timeout=45) != 0
        assert (live / 'operator-data').read_text() == 'preserve replacement'
        assert sum('Minecraft 26.2 ready' in line for line in dev.lines) == 1
        assert 'rcon.password' not in properties(project)
        assert properties(project)['server-ip'] == '0.0.0.0'
        checks.append('ordinary pack replacement during blocked namespace restart is preserved; no new JVM launches')
    finally:
        flags[2].touch()
        dev.close()
        for flag in flags:
            flag.unlink(missing_ok=True)
        config.write_text(original_config)
        library.write_text(original_library)


def validate(binary, output):
    project = output / 'project'
    project.mkdir()
    port = fixture(project)
    # The fixture also builds sand_export; sharing workspace target binaries
    # would overwrite sand-example's exporter during concurrent checks.
    target = project / 'build-cache'
    # The private cache remains under the disposable output directory, now
    # inside the fixture to prove Cargo's actual output path is not watched.
    if (output / 'target').exists():
        (output / 'target').rename(target)
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), NO_COLOR='1')
    resolve_fixture_lock(project, env)
    secrets = []
    checks = []
    live = project / 'dist/server/world/datapacks/devprobe'
    dev = Dev(binary, project, output / 'console.log', env)
    try:
        dev.wait('Minecraft 26.2 ready')
        props = properties(project)
        assert props['server-ip'] == '127.0.0.1'
        assert int(props['rcon.port']) != port
        secrets.append(props['rcon.password'])
        assert len(secrets[-1]) == 64
        with socket.create_connection(('127.0.0.1', port), timeout=2):
            pass
        with socket.create_connection(('127.0.0.1', int(props['rcon.port'])), timeout=2):
            pass
        checks.append('loopback game and RCON reachable; fresh 256-bit credential')
        builds = sum('dev: building (' in line for line in dev.lines)
        (target / 'watcher-output-probe').write_text('compiler output must be ignored')
        time.sleep(2)
        assert sum('dev: building (' in line for line in dev.lines) == builds
        checks.append('custom in-tree Cargo target output is excluded from source watching')
        before = snapshot(live)
        (project / 'src/target/mod.rs').write_text('this is not valid Rust')
        dev.wait('live pack unchanged')
        assert snapshot(live) == before
        dev.send('function devprobe:probe')
        dev.wait('dev_probe_v1')
        checks.append('failed build preserves disk pack and live v1 handler')
        source(project, 'v2')
        dev.wait('dev reload #1 complete')
        dev.send('function devprobe:probe')
        dev.wait('dev_probe_v2')
        assert 'dev_probe_v2' in (live / 'data/devprobe/function/probe.mcfunction').read_text()
        checks.append('src/target edit debounces, builds, syncs and changes live behavior')
        builds = sum('dev: building (' in line for line in dev.lines)
        time.sleep(2)
        assert sum('dev: building (' in line for line in dev.lines) == builds, 'Cargo outputs caused a rebuild loop'
        # Unmanaged copied output is included by the publisher. This intentionally
        # malformed vanilla function tests Minecraft loading, not Rust validation.
        broken = project / 'dist/devprobe/data/devprobe/function/broken.mcfunction'
        broken.write_text('not_a_minecraft_command\n')
        dev.send('r')
        dev.wait('dev reload #2 failed')
        broken.unlink()
        dev.send('r')
        dev.wait('dev reload #3 complete')
        checks.append('manual r; native parse failure isolated to one reload; next reload completes')
        dev.send('q')
        assert dev.process.wait(timeout=45) == 0, 'confirmed reload recovery must allow successful quit without R'
        dev.close()
        checks.append('bad native reload -> good reload -> q exits zero without restarting')
        dev = Dev(binary, project, output / 'restart-console.log', env)
        dev.wait('Minecraft 26.2 ready')
        secrets.append(properties(project)['rcon.password'])
        config = project / 'sand.toml'
        config.write_text(config.read_text().replace('Dev probe', 'Changed description'))
        dev.wait('restart required')
        assert 'Changed description' not in (live / 'pack.mcmeta').read_text()
        for _ in range(2):
            dev.send('R')
            dev.wait('server stopped; rebuilding and restarting')
            dev.wait('Minecraft 26.2 ready')
            secrets.append(properties(project)['rcon.password'])
            assert len(set(secrets)) == len(secrets)
            dev.send('function devprobe:probe')
            dev.wait('dev_probe_v2')
        assert 'Changed description' in (live / 'pack.mcmeta').read_text()
        checks.append('startup-resource guard; two graceful R restarts with fresh credentials and live handler')
        manual = live.parent / 'devprobe.sand-dev-backup'
        manual.mkdir()
        (manual / 'pack.mcmeta').write_bytes((live / 'pack.mcmeta').read_bytes())
        (manual / 'keep').write_text('operator-owned')
        library = project / 'src/lib.rs'
        config.write_text(config.read_text().replace('namespace="devprobe"', 'namespace="devprobe.sand-dev-stage"'))
        library.write_text(library.read_text().replace('try_export_pack_json("devprobe"', 'try_export_pack_json("devprobe.sand-dev-stage"'))
        dev.wait('namespace changed: restart required')
        assert live.exists(), 'namespace edit must retain previous live pack before R'
        dev.send('R')
        dev.wait('server stopped; rebuilding and restarting')
        dev.wait('Minecraft 26.2 ready')
        secrets.append(properties(project)['rcon.password'])
        renamed = live.parent / 'devprobe.sand-dev-stage'
        assert renamed.is_dir() and not live.exists()
        assert (manual / 'keep').read_text() == 'operator-owned'
        dev.send('datapack list enabled')
        enabled = dev.wait('file/devprobe.sand-dev-stage')
        assert '[file/devprobe (world)]' not in enabled
        dev.send('r')
        dev.wait('dev reload #1 complete')
        dev.send('function devprobe.sand-dev-stage:probe')
        dev.wait('dev_probe_v2')
        # Switch back within the same dev invocation, proving ownership advances.
        config.write_text(config.read_text().replace('namespace="devprobe.sand-dev-stage"', 'namespace="devprobe"'))
        library.write_text(library.read_text().replace('try_export_pack_json("devprobe.sand-dev-stage"', 'try_export_pack_json("devprobe"'))
        dev.wait('namespace changed: restart required')
        dev.send('R')
        dev.wait('server stopped; rebuilding and restarting')
        dev.wait('Minecraft 26.2 ready')
        secrets.append(properties(project)['rcon.password'])
        assert live.is_dir() and not renamed.exists()
        assert (manual / 'keep').is_file()
        checks.append('stage-suffix namespace restarts and reloads safely; backup-suffix operator neighbor retained')
        if os.name != 'nt':
            # Freeze only this harness's direct JVM child. R must enforce its
            # real 30-second bound, reap that JVM, and launch a replacement.
            rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,comm='], text=True)
            children = [int(parts[0]) for row in rows.splitlines()
                        if len(parts := row.strip().split(None, 2)) == 3
                        and int(parts[1]) == dev.process.pid and Path(parts[2]).name == 'java']
            assert len(children) == 1, 'expected one owned JVM child'
            os.kill(children[0], signal.SIGSTOP)
            dev.send('R')
            dev.wait('server shutdown deadline reached; process terminated')
            dev.wait('server stopped; rebuilding and restarting')
            dev.wait('Minecraft 26.2 ready')
            secrets.append(properties(project)['rcon.password'])
            dev.send('function devprobe:probe')
            dev.wait('dev_probe_v2')
            checks.append('R force-terminates an unresponsive owned JVM after the deadline and restarts successfully')
        dev.send('q')
        assert dev.process.wait(timeout=45) == 0
        checks.append('q gracefully exits after repeated restart')
    finally:
        dev.close()
    assert 'rcon.password' not in properties(project)
    assert properties(project)['enable-rcon'] == 'false'
    assert properties(project)['server-ip'] == '0.0.0.0'
    checks.append('original game bind restored after repeated dev restarts and quit')
    dev = Dev(binary, project, output / 'unrecovered-console.log', env)
    try:
        dev.wait('Minecraft 26.2 ready')
        secrets.append(properties(project)['rcon.password'])
        broken = project / 'dist/devprobe/data/devprobe/function/broken.mcfunction'
        broken.write_text('not_a_minecraft_command\n')
        dev.send('r')
        dev.wait('dev reload #1 failed')
        dev.send('q')
        assert dev.process.wait(timeout=45) != 0, 'unrecovered native load failure must fail at quit'
        checks.append('unrecovered native reload failure -> q exits nonzero')
    finally:
        dev.close()
        broken.unlink(missing_ok=True)
    dev = Dev(binary, project, output / 'eof-console.log', env)
    try:
        dev.wait('Minecraft 26.2 ready')
        secrets.append(properties(project)['rcon.password'])
        dev.process.stdin.close()
        assert dev.process.wait(timeout=45) == 0
        checks.append('EOF gracefully exits; credentials removed on shutdown')
    finally:
        dev.close()
    assert 'rcon.password' not in properties(project)
    validate_ordinary_shutdown(binary, project, output, env, checks)
    validate_namespace_replacement(binary, project, output, env, secrets, checks)
    validate_restart_cancellation(binary, project, output, env, secrets, checks)
    validate_initial_cancellation(binary, project, output, env, checks)
    for log in output.glob('*console.log'):
        text = log.read_text()
        assert all(secret not in text for secret in secrets), 'credential appeared in console'
    # Neither development instrumentation nor credentials may enter release output.
    release = run_bounded([str(binary), 'build', '--release'], cwd=project, env=env,
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    (output / 'release.log').write_text(release)
    import zipfile
    archives = list((project / 'dist').glob('*.zip'))
    assert archives, 'release archive missing'
    for archive in archives:
        with zipfile.ZipFile(archive) as zipped:
            for name in zipped.namelist():
                assert 'sand_dev_reload_' not in name
                assert '.sand-dev-owner' not in name
                data = zipped.read(name)
                assert all(secret.encode() not in data for secret in secrets)
    checks.append('release archive excludes reload sentinel and credentials')
    evidence = {'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                'minecraft': '26.2', 'platform': os.sys.platform, 'checks': checks}
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(json.dumps(evidence, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sand-bin', type=Path, default=ROOT / 'target/debug/sand')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    output = args.output or Path(tempfile.mkdtemp(prefix='sand-dev-validation-'))
    output.mkdir(parents=True, exist_ok=True)
    validate(args.sand_bin.resolve(), output.resolve())


if __name__ == '__main__':
    main()
