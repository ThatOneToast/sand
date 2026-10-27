"""Operational consumers must use the helper's returned immutable path."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ServerJarConsumers(unittest.TestCase):
    def test_wrapper_uses_returned_path_and_explicit_jar_bypasses_helper(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            jar = root / "artifact with spaces" / "server.jar"
            jar.parent.mkdir()
            jar.write_bytes(b"synthetic")
            pack = root / "pack"
            pack.mkdir()
            (pack / "pack.mcmeta").write_text("{}")
            tools = root / "bin"
            tools.mkdir()
            cargo = tools / "cargo"
            cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$TEST_JAR"\nprintf called > "$TEST_CALLED"\n')
            python = tools / "python3"
            python.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$TEST_ARGS"\n')
            cargo.chmod(0o755)
            python.chmod(0o755)
            env = dict(os.environ, PATH=f"{tools}:{os.environ['PATH']}",
                       TEST_JAR=str(jar), TEST_CALLED=str(root / "called"),
                       TEST_ARGS=str(root / "args"))
            base = ["bash", str(ROOT / "scripts/validate-vanilla-reload.sh"),
                    "--version", "26.2", "--pack", str(pack)]
            subprocess.run(base, env=env, check=True, capture_output=True)
            args = (root / "args").read_text().splitlines()
            self.assertEqual(args[args.index("--jar") + 1], str(jar))
            self.assertTrue((root / "called").exists())
            (root / "called").unlink()
            subprocess.run(base + ["--jar", str(jar)], env=env, check=True, capture_output=True)
            self.assertFalse((root / "called").exists())

    def test_workflow_passes_captured_path_to_all_jar_consumers(self):
        workflow = (ROOT / ".github/workflows/vanilla-reload.yml").read_text()
        self.assertIn('echo "SAND_SERVER_JAR=$jar" >> "$GITHUB_ENV"', workflow)
        jar_lines = [line for line in workflow.splitlines() if "--jar " in line]
        self.assertGreaterEqual(len(jar_lines), 21)
        self.assertTrue(all('--jar "$SAND_SERVER_JAR"' in line for line in jar_lines))
        self.assertNotIn('/server.jar', workflow)


if __name__ == "__main__":
    unittest.main()
