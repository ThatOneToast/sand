import argparse
from contextlib import redirect_stderr, redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "bench_build.py"
SPEC = importlib.util.spec_from_file_location("bench_build", SCRIPT)
bench = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bench)


class BuildBenchmarkTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name).resolve()
        self.root = self.directory / "checkout"
        self.root.mkdir()
        subprocess.run(["git", "init", "-q", self.root], check=True)
        (self.root / "sand/src").mkdir(parents=True)
        (self.root / "sand/src/lib.rs").write_text("// original facade\n")
        (self.root / "deleted.txt").write_text("deleted later")
        (self.root / ".gitignore").write_text("target/\ndist/\nignored.txt\n")
        subprocess.run(["git", "-C", self.root, "add", "."], check=True)
        subprocess.run(["git", "-C", self.root, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.test", "commit", "-qm", "fixture"], check=True)
        (self.root / "sand/src/lib.rs").write_text("// dirty facade\n")
        (self.root / "deleted.txt").unlink()
        (self.root / "untracked.txt").write_text("current untracked bytes")
        (self.root / "ignored.txt").write_text("excluded")
        (self.root / "target").mkdir()
        (self.root / "target/artifact").write_text("excluded")
        (self.root / "dist").mkdir()
        (self.root / "dist/pack").write_text("excluded")

    def test_snapshot_includes_dirty_untracked_and_deletions_without_sharing_files(self):
        snapshot = self.directory / "snapshot"
        entries = bench.snapshot_tree(self.root, snapshot)
        self.assertEqual((snapshot / "sand/src/lib.rs").read_text(), "// dirty facade\n")
        self.assertTrue((snapshot / "untracked.txt").exists())
        for omitted in (".git", "target", "dist", "ignored.txt", "deleted.txt"):
            self.assertFalse((snapshot / omitted).exists(), omitted)
        (snapshot / "sand/src/lib.rs").write_text("snapshot-only edit")
        self.assertEqual((self.root / "sand/src/lib.rs").read_text(), "// dirty facade\n")
        second = bench.snapshot_tree(self.root, self.directory / "snapshot2")
        self.assertEqual(bench.fingerprint(entries), bench.fingerprint(second))
        (self.root / "untracked.txt").write_text("changed")
        self.assertNotEqual(bench.fingerprint(entries), bench.fingerprint(bench.tree_entries(self.root)))

    def test_rejects_external_dangling_and_excluded_symlinks(self):
        external = self.directory / "external"
        external.write_text("outside source")
        link = self.root / "link"
        for target in (external, self.root / "missing", self.root / "target/artifact"):
            with self.subTest(target=target):
                link.symlink_to(target)
                with self.assertRaisesRegex(ValueError, "symlink"):
                    bench.tree_entries(self.root)
                link.unlink()

    def test_internal_absolute_symlink_is_rebased_into_snapshot(self):
        (self.root / "link").symlink_to(self.root / "untracked.txt")
        snapshot = self.directory / "snapshot"
        bench.snapshot_tree(self.root, snapshot)
        self.assertFalse(os.path.isabs(os.readlink(snapshot / "link")))
        self.assertEqual((snapshot / "link").resolve(), snapshot / "untracked.txt")
        (snapshot / "link").write_text("snapshot only")
        self.assertEqual((self.root / "untracked.txt").read_text(), "current untracked bytes")

    def test_symlink_to_ignored_input_fails_instead_of_using_checkout(self):
        (self.root / "link").symlink_to("ignored.txt")
        with self.assertRaisesRegex(ValueError, "absent from snapshot"):
            bench.snapshot_tree(self.root, self.directory / "snapshot")

    def test_concurrent_source_edit_rejects_snapshot(self):
        original = bench.tree_entries
        calls = []

        def read(root):
            calls.append(1)
            if len(calls) == 2:
                (root / "untracked.txt").write_text("concurrent edit")
            return original(root)

        with patch.object(bench, "tree_entries", side_effect=read):
            with self.assertRaisesRegex(ValueError, "changed during snapshot"):
                bench.snapshot_tree(self.root, self.directory / "snapshot")

    def test_output_inside_source_is_rejected_before_creation(self):
        with self.assertRaisesRegex(ValueError, "outside"):
            bench.output_directory(str(self.root / "benchmark"), self.root)
        self.assertFalse((self.root / "benchmark").exists())

    def test_temporary_output_inside_source_is_rejected_before_creation(self):
        before = sorted(self.root.iterdir())
        with patch.object(bench.tempfile, "gettempdir", return_value=str(self.root)):
            with self.assertRaisesRegex(ValueError, "outside"):
                bench.output_directory(None, self.root)
        self.assertEqual(sorted(self.root.iterdir()), before)

    def fake_cargo(self, fail_rebuild=False):
        path = self.directory / "fake-cargo"
        path.write_text(f"#!{sys.executable}\n" + '''import hashlib, os
from pathlib import Path
import sys
if sys.argv[1:] == ["--version"]:
    print("cargo fixture 1")
    raise SystemExit(0)
target = Path(os.environ["CARGO_TARGET_DIR"])
facade = Path("sand/src/lib.rs").read_bytes()
with (target / "calls").open("a") as output:
    output.write(hashlib.sha256(facade).hexdigest() + "\\n")
print("fixture cargo check output")
''' + ('''if b"isolated facade benchmark" in facade:
    print("injected facade build failure")
    raise SystemExit(9)
''' if fail_rebuild else ""))
        path.chmod(0o755)
        return path

    def run_suite(self, cargo, **options):
        args = argparse.Namespace(source=str(self.root), output=str(self.directory / "result"),
                                  version="26.3", cold_samples=1, warm_samples=2, cargo=str(cargo))
        vars(args).update(options)
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            status = bench.suite(args)
        return status, json.loads((self.directory / "result/results.json").read_text())

    def test_core_test_compilation_uses_snapshot_and_restores_changed_source(self):
        core = self.root / "sand-core/src/lib.rs"
        core.parent.mkdir(parents=True)
        core.write_text("// core fixture\n")
        cargo = self.fake_cargo()
        cargo.write_text(cargo.read_text().replace("sand/src/lib.rs", "sand-core/src/lib.rs"))
        before = bench.tree_entries(self.root)
        status, result = self.run_suite(cargo, edit_scope="core", test_compile=True)
        self.assertEqual(status, 0)
        self.assertEqual(result["edited_path"], "sand-core/src/lib.rs")
        self.assertEqual(result["command"][1:], ["test", "--workspace", "--locked", "--all-features", "--no-run"])
        self.assertIn("warm-core-rebuild", result["summary"])
        calls = (self.directory / "result/targets/warm/calls").read_text().splitlines()
        self.assertEqual(len(set(calls[:3])), 3)
        self.assertEqual(calls[3:], [calls[0]] * 3)
        self.assertEqual(bench.tree_entries(self.root), before)
        self.assertEqual((self.directory / "result/source/sand-core/src/lib.rs").read_text(), core.read_text())

    def test_missing_edit_source_records_failure_without_running_cargo(self):
        status, result = self.run_suite(self.fake_cargo(), edit_scope="codegen")
        self.assertEqual(status, 1)
        self.assertEqual(result["scenarios"], [])
        self.assertIn("FileNotFoundError", result["error"])

    def test_suite_uses_empty_cold_targets_and_warm_rebuilds_without_live_edits(self):
        before = bench.tree_entries(self.root)
        (self.root / "target/sentinel").write_text("live build output")
        with patch.dict(os.environ, {"CARGO_TARGET_DIR": str(self.root / "target")}):
            status, result = self.run_suite(self.fake_cargo())
        self.assertFalse((self.root / "target/calls").exists())
        self.assertEqual((self.root / "target/sentinel").read_text(), "live build output")
        self.assertEqual(status, 0)
        self.assertTrue(result["passed"])
        self.assertEqual(bench.tree_entries(self.root), before)
        self.assertEqual(result["tree_sha256"], bench.fingerprint(before))
        self.assertEqual(len(result["scenarios"]), 7)
        cold = self.directory / "result/targets/cold-1/calls"
        self.assertEqual(len(cold.read_text().splitlines()), 1)
        warm = (self.directory / "result/targets/warm/calls").read_text().splitlines()
        self.assertEqual(len(warm), 6)
        self.assertEqual(len(set(warm[:3])), 3)
        self.assertEqual(warm[0], warm[3])
        self.assertEqual(warm[3:], [warm[0]] * 3)
        self.assertEqual((self.directory / "result/source/sand/src/lib.rs").read_text(), "// dirty facade\n")
        self.assertIn("rustc", result["toolchain"])
        self.assertTrue((self.directory / "result/tree-manifest.json").exists())
        self.assertNotIn("p95_seconds", result["summary"]["no-op"])

    def test_relative_cache_and_cargo_paths_resolve_before_entering_snapshot(self):
        cargo = self.fake_cargo()
        cache = self.directory / "external-cargo"
        with patch.dict(os.environ, {"CARGO_HOME": os.path.relpath(cache, Path.cwd())}):
            status, result = self.run_suite(Path(os.path.relpath(cargo, Path.cwd())))
        self.assertEqual(status, 0)
        self.assertEqual(result["external_caches"]["cargo_home"], str(cache))
        self.assertEqual(result["scenarios"][0]["command"][0], str(cargo))

    def test_failed_build_retains_log_status_and_restores_snapshot_facade(self):
        before = bench.tree_entries(self.root)
        status, result = self.run_suite(self.fake_cargo(fail_rebuild=True))
        self.assertEqual(status, 1)
        self.assertFalse(result["passed"])
        self.assertEqual(result["failure"], {"scenario": "warm-facade-rebuild", "sample": 1, "returncode": 9})
        self.assertIn("injected facade build failure", Path(result["scenarios"][-1]["log"]).read_text())
        self.assertEqual((self.directory / "result/source/sand/src/lib.rs").read_text(), "// dirty facade\n")
        self.assertEqual(bench.tree_entries(self.root), before)
        self.assertFalse(any(sample["scenario"] == "no-op" for sample in result["scenarios"]))

    def test_snapshot_failure_still_writes_machine_readable_result(self):
        (self.root / "unsafe").symlink_to(self.directory)
        status, result = self.run_suite(self.fake_cargo())
        self.assertEqual(status, 1)
        self.assertFalse(result["passed"])
        self.assertIn("unsafe external", result["error"])
        self.assertEqual(result["scenarios"], [])

    def test_single_entrypoint_preserves_failure_exit_and_log(self):
        output = self.directory / "single"
        command = [str(SCRIPT.with_suffix(".sh")), "--output", str(output),
                   "failure", "3", "--", sys.executable, "-c", "print('failure evidence'); raise SystemExit(7)"]
        completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.assertEqual(completed.returncode, 7)
        result = json.loads((output / "results.json").read_text())
        self.assertFalse(result["passed"])
        self.assertEqual(len(result["samples"]), 1)
        self.assertIn("failure evidence", (output / "sample-1.log").read_text())

    def test_summary_omits_small_sample_tail_and_uses_defined_nearest_rank(self):
        records = [{"seconds": value, "returncode": 0} for value in range(1, 21)]
        self.assertNotIn("p95_seconds", bench.summary(records[:5]))
        self.assertEqual(bench.summary(records)["p95_seconds"], 19)
        self.assertEqual(bench.summary(records)["median_seconds"], 10.5)
        self.assertEqual(bench.summary([{"seconds": 100, "returncode": 7}]), {"successful_samples": 0})

    def test_missing_command_is_reported_as_sample_failure(self):
        log = self.directory / "missing.log"
        result = bench.measure([str(self.directory / "missing-command")], self.root, os.environ.copy(), log)
        self.assertEqual(result["returncode"], 127)
        self.assertIn("FileNotFoundError", log.read_text())


if __name__ == "__main__":
    unittest.main()
