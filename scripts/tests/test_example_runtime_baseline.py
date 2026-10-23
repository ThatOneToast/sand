"""Exercise baseline enforcement through its file/CLI boundary."""

import contextlib
import copy
import io
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
import tempfile
import unittest

from scripts.example_runtime_baseline import main, normalize


def build():
    work = {"minimum": 2, "maximum": 4, "unknown": []}
    entry = {"reference": "demo:tick", "required": True,
             "contributors": ["demo:b", "demo:a"], "evaluations": work}
    return {"schema_version": 1, "success": True, "namespace": "demo",
            "minecraft_version": "26.2", "profile": "dev",
            "datapack_output": "/temporary/output", "datapack_changes": {"written": 4},
            "runtime_report": {
                "schema_version": 1,
                "tick": [entry, {**entry, "reference": "demo:later"}],
                "load": [entry, {**entry, "reference": "demo:load"}],
                "functions": {"demo:tick": {
                    "owners": ["demo:b", "demo:a"], "evaluations": copy.deepcopy(work),
                    "selector_sites": [
                        {"line": 1, "operation": "as", "selector": "@a"},
                        {"line": 2, "operation": "at", "selector": "@s"}]}},
                "limitations": ["not MSPT", "not command budget"],
                "provenance": {
                    "function_owners": {"demo:tick": ["demo:b", "demo:a"]},
                    "tag_contributors": [], "archetypes": [], "lifecycle_owners": {},
                    "systems": [{"owners": ["demo:b", "demo:a"], "function": "demo:tick",
                                 "every": 2, "selector": "@a"}]}}}


class RuntimeBaselines(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.report = Path(self.temp.name) / "build.json"
        self.baseline = Path(self.temp.name) / "demo.json"
        self.original = build()
        self.write(self.original)
        self.assertEqual(self.run_mode("update"), 0)

    def write(self, value):
        self.report.write_text(json.dumps(value))

    def run_mode(self, mode):
        self.output = io.StringIO()
        with contextlib.redirect_stdout(self.output), contextlib.redirect_stderr(self.output):
            return main([mode, str(self.report), str(self.baseline)])

    def drift(self, mutate):
        value = copy.deepcopy(self.original)
        mutate(value["runtime_report"])
        self.write(value)
        before = self.baseline.read_bytes()
        self.assertEqual(self.run_mode("check"), 1)
        self.assertIn("Runtime baseline drift", self.output.getvalue())
        self.assertEqual(before, self.baseline.read_bytes())
        self.assertEqual(self.run_mode("update"), 0)
        self.assertEqual(self.run_mode("check"), 0)
        # Every subtest starts against the original baseline again.
        self.write(self.original)
        self.assertEqual(self.run_mode("update"), 0)

    def test_real_metric_changes_fail_and_require_explicit_update(self):
        for key, value in (("minimum", 3), ("maximum", 5), ("maximum", 3)):
            with self.subTest(key=key, value=value):
                self.drift(lambda r: r["functions"]["demo:tick"]["evaluations"].update({key: value}))

    def test_unknown_bounds_and_reasons_are_not_zero_or_dropped(self):
        self.drift(lambda r: r["tick"][0]["evaluations"].update(
            maximum=None, unknown=["unknown selector multiplicity", "additional unknown causes omitted"]))
        value = self.original["runtime_report"]["functions"]["demo:tick"]["evaluations"]
        value.update(maximum=None, unknown=["macro arguments", "external function"])
        self.write(self.original)
        self.assertEqual(self.run_mode("update"), 0)
        self.drift(lambda r: r["functions"]["demo:tick"]["evaluations"].update(unknown=["different reason"]))
        self.drift(lambda r: r["functions"]["demo:tick"]["evaluations"].update(maximum=4, unknown=[]))

    def test_unordered_input_and_build_location_do_not_cause_drift(self):
        value = copy.deepcopy(self.original)
        value["datapack_output"] = "/different/checkout"
        value["datapack_changes"] = {"written": 0, "unchanged": 4}
        report = value["runtime_report"]
        report["limitations"].reverse()
        report["functions"]["demo:tick"]["owners"].reverse()
        report["functions"]["demo:tick"]["selector_sites"].reverse()
        report["provenance"]["function_owners"]["demo:tick"].reverse()
        for phase in ("tick", "load"):
            for entry in report[phase]:
                entry["contributors"] = sorted(entry["contributors"])
        self.report.write_text(json.dumps(value, sort_keys=True))
        self.assertEqual(self.run_mode("check"), 0)
        self.assertEqual(normalize(value), normalize(self.original))

    def test_unknown_reason_order_is_not_drift(self):
        work = self.original["runtime_report"]["functions"]["demo:tick"]["evaluations"]
        work.update(maximum=None, unknown=["macro arguments", "external function"])
        self.write(self.original)
        self.assertEqual(self.run_mode("update"), 0)
        work["unknown"].reverse()
        self.write(self.original)
        self.assertEqual(self.run_mode("check"), 0)

    def test_execution_order_is_semantic(self):
        for phase in ("tick", "load"):
            with self.subTest(phase=phase):
                self.drift(lambda r: r[phase].reverse())
        self.drift(lambda r: r["provenance"]["systems"][0]["owners"].reverse())

    def test_ownership_cadence_selector_and_function_inventory_drift(self):
        self.drift(lambda r: r["provenance"]["systems"][0].update(every=1))
        self.drift(lambda r: r["functions"]["demo:tick"]["owners"].append("demo:new"))
        self.drift(lambda r: r["functions"]["demo:tick"]["selector_sites"][0].update(selector="@e"))
        self.drift(lambda r: r["functions"]["demo:tick"]["selector_sites"].append(
            copy.deepcopy(r["functions"]["demo:tick"]["selector_sites"][0])))
        self.drift(lambda r: r["functions"].update({"demo:new": copy.deepcopy(r["functions"]["demo:tick"])}))
        self.drift(lambda r: r["limitations"].append("new limitation"))

    def test_bad_missing_and_new_schema_reports_fail_closed(self):
        for value in ({}, {**self.original, "success": False},
                      {**self.original, "schema_version": 2},
                      {**self.original, "runtime_report": {}}):
            with self.subTest(value=value):
                self.write(value)
                self.assertEqual(self.run_mode("check"), 1)
        for change in ({"schema_version": 2}, {"future_metric": 3}):
            value = copy.deepcopy(self.original)
            value["runtime_report"].update(change)
            self.write(value)
            self.assertEqual(self.run_mode("check"), 1)
        self.report.write_text("not JSON")
        self.assertEqual(self.run_mode("check"), 1)
        self.write(self.original)
        self.baseline.unlink()
        self.assertEqual(self.run_mode("check"), 1)

    def test_inventory_rejects_missing_and_stale_examples(self):
        examples = Path(self.temp.name) / "examples"
        baselines = Path(self.temp.name) / "baselines"
        (examples / "demo").mkdir(parents=True)
        (examples / "demo" / "Cargo.toml").touch()
        baselines.mkdir()
        args = ["inventory", str(examples), str(baselines)]
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(main(args), 1)
            (baselines / "demo.json").write_text("{}")
            self.assertEqual(main(args), 0)
            (baselines / "stale.json").write_text("{}")
            self.assertEqual(main(args), 1)


class ExampleBuildIntegration(unittest.TestCase):
    def test_single_export_per_example_and_build_failures_remain_visible(self):
        repository = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts/tests").mkdir(parents=True)
            # The real regression suite is already running; the copied driver
            # needs a stub suite so it does not recursively invoke this test.
            (root / "scripts/tests/test_example_runtime_baseline.py").write_text(
                "import unittest\nclass Stub(unittest.TestCase):\n"
                "    def test_driver(self): pass\n")
            for name in ("check-examples.sh", "example_runtime_baseline.py"):
                shutil.copy(repository / "scripts" / name, root / "scripts" / name)
            for name in ("one", "two"):
                project = root / "examples" / name
                project.mkdir(parents=True)
                (project / "Cargo.toml").touch()
                (project / "build.json").write_text(json.dumps(build()))
            binary = root / "bin"
            binary.mkdir()
            cargo = binary / "cargo"
            cargo.write_text(f"#!{sys.executable}\n" +
                "import json, os, pathlib, sys\n"
                "with open(os.environ['CALL_LOG'], 'a') as log:\n"
                "    log.write(json.dumps(sys.argv[1:]) + '\\n')\n"
                "if 'build' in sys.argv:\n"
                "    if os.environ.get('FAIL_BUILD'):\n"
                "        print('BUILD_FAILURE_DIAGNOSTIC')\n"
                "        sys.exit(1)\n"
                "    print(pathlib.Path('build.json').read_text())\n")
            cargo.chmod(0o755)
            log = root / "calls.jsonl"
            env = {**os.environ, "PATH": str(binary) + os.pathsep + os.environ["PATH"],
                   "CALL_LOG": str(log)}
            for arguments in (["--update-runtime-baselines"], []):
                log.write_text("")
                result = subprocess.run(["bash", "scripts/check-examples.sh", *arguments],
                                        cwd=root, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                calls = [json.loads(line) for line in log.read_text().splitlines()]
                self.assertEqual(sum(call[0] == "test" for call in calls), 2)
                builds = [call for call in calls if "build" in call]
                self.assertEqual(len(builds), 2)
                self.assertTrue(all(call[-3:] == ["build", "--format", "json"] for call in builds))
            result = subprocess.run(["bash", "scripts/check-examples.sh"], cwd=root,
                                    env={**env, "FAIL_BUILD": "1"}, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("BUILD_FAILURE_DIAGNOSTIC", result.stderr)


if __name__ == "__main__":
    unittest.main()
