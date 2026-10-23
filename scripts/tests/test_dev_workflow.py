"""The native workflow verifier must clean up failed process trees."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "dev_workflow", Path(__file__).resolve().parents[1] / "validate_dev_workflow.py")
workflow = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(workflow)


class FixtureResolution(unittest.TestCase):
    def test_seeded_lock_is_resolved_before_watching(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('')
            (root / 'Cargo.toml').write_text(
                '[package]\nname="devprobe"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
            lock = root / 'Cargo.lock'
            lock.write_text('version = 4\n[[package]]\nname = "workspace-seed"\nversion = "0.1.0"\n')
            seed = lock.read_bytes()
            workflow.resolve_fixture_lock(root, dict(os.environ))
            resolved = lock.read_bytes()
            self.assertNotEqual(seed, resolved)
            workflow.run_bounded(
                ['cargo', 'metadata', '--locked', '--offline', '--format-version', '1'],
                cwd=root, stdout=subprocess.DEVNULL)
            self.assertEqual(resolved, lock.read_bytes())


@unittest.skipUnless(os.name == "posix", "POSIX process-group regression")
class ProcessCleanup(unittest.TestCase):
    def assert_stopped(self, pid):
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            state = subprocess.run(["ps", "-o", "stat=", "-p", str(pid)],
                                   capture_output=True, text=True, timeout=2).stdout.strip()
            if not state or state.startswith("Z"):
                return
            time.sleep(0.05)
        self.fail(f"validation descendant {pid} is still running")

    def fixture(self, root):
        path = root / "fake-sand"
        path.write_text(
            f"#!{sys.executable}\n"
            "import subprocess, sys, time\n"
            "from pathlib import Path\n"
            "child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)'])\n"
            "Path('child.pid').write_text(str(child.pid))\n"
            "time.sleep(60)\n")
        path.chmod(0o755)
        return path

    def wait_for_descendant(self, root, process):
        deadline = time.monotonic() + 30
        path = root / "child.pid"
        while time.monotonic() < deadline:
            if path.exists() and path.read_text().strip():
                return int(path.read_text())
            self.assertIsNone(process.poll(), "fixture exited before creating its descendant")
            time.sleep(0.01)
        self.fail("fixture did not create its descendant within 30 seconds")

    def test_dev_timeout_reaps_parent_and_kills_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dev = workflow.Dev(self.fixture(root), root, root / "console.log", dict(os.environ))
            try:
                pid = self.wait_for_descendant(root, dev.process)
                dev.close(timeout=0.1)
                self.assertIsNotNone(dev.process.returncode)
                self.assert_stopped(pid)
            finally:
                dev.close(timeout=0.1)

    def test_release_deadline_kills_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            launch = subprocess.Popen
            descendant = None

            def launch_ready(*args, **kwargs):
                # Start the deadline only after a real descendant exists. Slow
                # interpreter startup must not turn this into a parent-only test.
                nonlocal descendant
                process = launch(*args, **kwargs)
                try:
                    descendant = self.wait_for_descendant(root, process)
                except BaseException:
                    workflow.kill_process_tree(process)
                    for pipe in (process.stdin, process.stdout, process.stderr):
                        if pipe is not None:
                            pipe.close()
                    raise
                return process

            with patch.object(workflow.subprocess, "Popen", side_effect=launch_ready):
                with self.assertRaises(subprocess.TimeoutExpired):
                    workflow.run_bounded([str(self.fixture(root))], cwd=root, timeout=1,
                                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            self.assertIsNotNone(descendant)
            self.assert_stopped(descendant)


if __name__ == "__main__":
    unittest.main()
