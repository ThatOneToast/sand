import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HARNESS = ROOT / "scripts" / "vanilla_reload_harness.py"


FAKE_JAVA = r"""#!/usr/bin/env python3
import os, sys, time
scenario = os.environ.get("FAKE_SCENARIO", "success")
commands = os.environ.get("FAKE_COMMANDS")
def emit(line):
    print(line, flush=True)
if scenario == "no_start":
    time.sleep(30)
    raise SystemExit(0)
if scenario == "exit_before_start":
    raise SystemExit(4)
if scenario == "initial_error":
    emit("[Server thread/ERROR]: Failed to load datapack sand")
emit('[Server thread/INFO]: Done (0.123s)! For help, type "help"')
for raw in sys.stdin:
    command = raw.strip()
    if commands:
        with open(commands, "a", encoding="utf-8") as output:
            output.write(command + "\n")
    if command.startswith("say __SAND_RELOAD_SUBMITTED__"):
        emit("[Server thread/INFO]: [Server] __SAND_RELOAD_SUBMITTED__")
    elif command == "reload":
        if scenario == "exit_reload":
            raise SystemExit(3)
        if scenario != "no_reload_start":
            emit("[Server thread/INFO]: Reloading!")
        if scenario == "reload_error":
            emit("[Server thread/ERROR]: Failed to parse recipe sand:bad")
    elif command.startswith("say __SAND_RELOAD_COMPLETE__"):
        if scenario != "no_reload_complete":
            emit("[Server thread/INFO]: [Server] __SAND_RELOAD_COMPLETE__")
    elif command == "stop":
        emit("[Server thread/INFO]: Stopping server")
        raise SystemExit(0)
"""


class HarnessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.pack = self.root / "pack"
        self.pack.mkdir()
        (self.pack / "pack.mcmeta").write_text("{}", encoding="utf-8")
        self.jar = self.root / "server.jar"
        self.jar.write_text("fake", encoding="utf-8")
        self.java = self.root / "java"
        self.java.write_text(textwrap.dedent(FAKE_JAVA), encoding="utf-8")
        self.java.chmod(0o755)

    def tearDown(self):
        self.temp.cleanup()

    def run_harness(self, scenario="success", timeout="0.5"):
        output = self.root / f"output-{scenario}"
        commands = self.root / f"commands-{scenario}.txt"
        env = os.environ.copy()
        env.update(FAKE_SCENARIO=scenario, FAKE_COMMANDS=str(commands))
        result = subprocess.run(
            [
                sys.executable,
                str(HARNESS),
                "--version", "test",
                "--pack", str(self.pack),
                "--jar", str(self.jar),
                "--java", str(self.java),
                "--output", str(output),
                "--timeout", timeout,
            ],
            text=True,
            capture_output=True,
            env=env,
            timeout=5,
        )
        return result, output, commands

    def test_success_emits_reload_and_stops_cleanly(self):
        result, output, commands = self.run_harness()
        self.assertEqual(result.returncode, 0, result.stderr)
        sent = commands.read_text(encoding="utf-8")
        self.assertIn("reload\n", sent)
        self.assertTrue(sent.endswith("stop\n"))
        self.assertIn("reload-complete=ok shutdown=ok", result.stdout)
        self.assertTrue((output / "latest.log").is_file())

    def test_startup_timeout_is_phase_specific_and_process_is_reaped(self):
        result, output, _ = self.run_harness("no_start", "0.2")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=startup", result.stderr)
        self.assertIn("timed out", result.stderr)
        self.assertTrue((output / "latest.log").exists())

    def test_missing_startup_marker_before_exit_fails(self):
        result, _, _ = self.run_harness("exit_before_start")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=startup", result.stderr)
        self.assertIn("exited with code 4", result.stderr)

    def test_initial_load_error_is_detected(self):
        result, _, _ = self.run_harness("initial_error")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=initial-load", result.stderr)
        self.assertIn("Failed to load datapack", result.stderr)

    def test_missing_reload_start_marker_fails(self):
        result, _, _ = self.run_harness("no_reload_start", "0.2")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=reload-start", result.stderr)

    def test_missing_reload_completion_marker_fails(self):
        result, _, _ = self.run_harness("no_reload_complete", "0.2")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=reload-complete", result.stderr)

    def test_reload_error_is_detected_after_dispatch(self):
        result, _, _ = self.run_harness("reload_error")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=reload", result.stderr)
        self.assertIn("Failed to parse recipe", result.stderr)

    def test_server_exit_during_reload_fails(self):
        result, _, _ = self.run_harness("exit_reload")
        self.assertEqual(result.returncode, 1)
        self.assertIn("phase=reload-start", result.stderr)
        self.assertIn("exited with code 3", result.stderr)


if __name__ == "__main__":
    unittest.main()
