"""The native workflow verifier must clean up failed process trees."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

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

    def test_dev_timeout_reaps_parent_and_kills_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dev = workflow.Dev(self.fixture(root), root, root / "console.log", dict(os.environ))
            try:
                deadline = time.monotonic() + 3
                while not (root / "child.pid").exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                pid = int((root / "child.pid").read_text())
                dev.close(timeout=0.1)
                self.assertIsNotNone(dev.process.returncode)
                self.assert_stopped(pid)
            finally:
                dev.close(timeout=0.1)

    def test_release_deadline_kills_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaises(subprocess.TimeoutExpired):
                workflow.run_bounded([str(self.fixture(root))], cwd=root, timeout=1,
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            self.assert_stopped(int((root / "child.pid").read_text()))


if __name__ == "__main__":
    unittest.main()
