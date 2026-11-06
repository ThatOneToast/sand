"""The aggregate must fail closed and retain the other scenario's evidence."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import validate_alpha


class AlphaRunnerTests(unittest.TestCase):
    def test_failed_scenario_does_not_hide_other_results_or_allow_false_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            jar = root / "server.jar"
            jar.touch()
            calls = []

            def scenario(command, **kwargs):
                output = Path(command[-1])
                output.mkdir()
                calls.append(output.name)
                (output / "evidence.json").write_text(json.dumps({"passed": True, "version": "26.3"}))
                return subprocess.CompletedProcess(command, 7 if output.name == "roster" else 0)

            with patch.object(validate_alpha.subprocess, "run", side_effect=scenario):
                result = validate_alpha.run(jar, root / "proof", "all")
            summary = json.loads((root / "proof/summary.json").read_text())
            self.assertEqual(result, 1)
            self.assertEqual(calls, ["roster", "arena"])
            self.assertTrue(summary["complete"])
            self.assertFalse(summary["passed"])
            self.assertTrue(summary["scenarios"][1]["passed"])
            with self.assertRaises(FileExistsError):
                validate_alpha.run(jar, root / "proof", "all")

    def test_zero_exit_without_verified_evidence_is_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            jar = root / "server.jar"
            jar.touch()
            with patch.object(validate_alpha.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)):
                self.assertEqual(validate_alpha.run(jar, root / "proof", "arena"), 1)
            summary = json.loads((root / "proof/summary.json").read_text())
            self.assertIn("Missing or invalid", summary["scenarios"][0]["error"])


if __name__ == "__main__":
    unittest.main()
