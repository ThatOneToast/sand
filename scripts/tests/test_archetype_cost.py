"""Regressions for the vanilla trace measurement boundary."""
import sys
import shutil
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from measure_archetype_idle import main, IdleCostHarness, pack_digest, reconcile_target, trace_counts, verified_runtime_version


class TraceCounts(unittest.TestCase):
    def test_wrong_or_missing_runtime_is_rejected_before_measurement(self):
        """Reject incorrect runtime attribution before sending workload commands."""
        for lines in (
            ["[Server thread/INFO]: Starting minecraft server version 26.1"],
            ["[Server thread/INFO]: Starting minecraft server version 26.2-snapshot"],
            ["[Server thread/INFO]: Done (1s)! For help, type help"],
        ):
            with self.subTest(lines=lines):
                harness = object.__new__(IdleCostHarness)
                harness.lines = lines
                harness.evidence = {"complete": False}
                def no_commands(command):
                    """Fail if a rejected runtime reaches measurement setup."""
                    self.fail("measurement commands started before runtime validation")
                harness.send = no_commands
                with self.assertRaisesRegex(ValueError, "runtime"):
                    harness.run_client()
                self.assertFalse(harness.evidence["complete"])

    def test_runtime_identity_accepts_exact_target_and_rejects_conflicting_versions(self):
        """Require one unambiguous target identity in the startup log."""
        line = "[Server thread/INFO]: Starting minecraft server version 26.2"
        self.assertEqual(verified_runtime_version([line]), "26.2")
        with self.assertRaisesRegex(ValueError, "runtime"):
            verified_runtime_version([line, line.replace("26.2", "26.1")])

    def test_dirty_workload_mutates_before_tracing_and_requires_derived_work(self):
        """Exclude authored mutation cost and reject traces that never reconciled it."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            debug = root / "server/debug"
            debug.mkdir(parents=True)
            trace = """    [F] sand_cost_probe:reconcile size=2
        [C] execute as @e run function rpg:entity/reconcile
        [F] rpg:entity/derive/0 size=1
            [C] say derived
        [C] say SAND_COST_MEASUREMENT_COMPLETE
"""
            (debug / "debug-trace-fixture.txt").write_text(trace)
            harness = object.__new__(IdleCostHarness)
            harness.server = root / "server"
            harness.output = root / "output"
            harness.lines = []
            harness.target = "rpg:entity/reconcile"
            harness.workload = "level-up"
            harness.evidence = {"samples": []}
            sent = []

            def send(command):
                """Simulate a completed trace announcement without starting a server."""
                sent.append(command)
                if command.startswith("debug function"):
                    harness.lines.append("Traced 2 command(s) to output file debug-trace-fixture.txt")

            harness.send = send
            harness.wait_for = lambda *args: None
            harness.measure(1, 0)
            self.assertEqual(sent[:2], [
                "execute as @e[tag=sand_cost_probe] at @s run function rpg:level_up",
                "debug function sand_cost_probe:reconcile",
            ])
            self.assertEqual(harness.evidence["samples"][0]["reconcile_evaluations"], 1)
            (debug / "debug-trace-fixture.txt").write_text(trace.replace("rpg:entity/derive/0", "rpg:entity/unrelated"))
            with self.assertRaisesRegex(ValueError, "dependent derivation"):
                harness.measure(1, 1)
            self.assertEqual(len(harness.evidence["samples"]), 1)
            harness.workload = "idle"
            sent.clear()
            harness.measure(1, 2)
            self.assertEqual(sent[0], "debug function sand_cost_probe:reconcile")

    def test_dirty_workload_rejects_a_pack_without_the_authored_entry(self):
        """Fail before server startup when the requested mutation is unavailable."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scan = root / "data/rpg/function/__sand_entity/one/reconcile_scan.mcfunction"
            scan.parent.mkdir(parents=True)
            scan.write_text("execute as @e[type=minecraft:zombie,tag=owned] at @s run function rpg:__sand_entity/one/reconcile\n")
            with patch("measure_archetype_idle.IdleCostHarness") as harness, patch.object(
                sys, "argv", ["measure_archetype_idle", "--pack", str(root), "--jar", "unused.jar", "--workload", "level-up"]
            ):
                with self.assertRaisesRegex(ValueError, "rpg:level_up"):
                    main()
                harness.assert_not_called()

    def test_nested_and_sibling_frames_return_to_their_caller(self):
        """Attribute nested work correctly after returning to sibling and parent frames."""
        trace = '''demo:root
    [F] demo:root size=3
        [C] function demo:first
        [F] demo:first size=1
            [C] execute if score @s idle matches 1 run say changed
        [C] function demo:second
        [F] demo:second size=1
            [C] function demo:nested
            [F] demo:nested size=1
                [C] say nested
                [M] [C] this is a message, not an evaluation
                [R = 1] say nested
        [C] say end
'''
        self.assertEqual(trace_counts(trace), {
            "demo:first": 1, "demo:nested": 1, "demo:root": 3, "demo:second": 1,
        })

    def test_repeated_frames_accumulate_per_entity_work(self):
        """Count each entity invocation rather than deduplicating function frames."""
        trace = '''    [F] demo:scan size=1
        [C] execute as @e run function demo:body
        [F] demo:body size=1
            [C] say one
        [F] demo:body size=1
            [C] say two
'''
        self.assertEqual(trace_counts(trace), {"demo:body": 2, "demo:scan": 1})

    def test_multiple_tag_roots_do_not_inherit_frames(self):
        """Keep separate root functions from sharing an obsolete caller frame."""
        trace = '''    [F] demo:a size=1
        [C] say a
    [F] demo:b size=1
        [C] say b
'''
        self.assertEqual(trace_counts(trace), {"demo:a": 1, "demo:b": 1})

    def test_errors_and_unrecognized_records_never_become_zero_cost(self):
        """Fail closed when traces contain errors, malformed records or no work."""
        for trace in (
            "", "Traced 1 command(s)", "    [C] say orphan",
            "    [F] demo:root size=1\n        [C] say ok\n        [E] unknown function",
            "    [F] demo:root unknown-format\n        [C] say ok",
            "    [F] demo:root size=1\n        [C]",
        ):
            with self.subTest(trace=trace), self.assertRaises(ValueError):
                trace_counts(trace)

    def test_pack_digest_is_path_independent_and_content_sensitive(self):
        """Identify resource paths and bytes independently of checkout location."""
        with tempfile.TemporaryDirectory() as directory:
            roots = [Path(directory) / name for name in ("a", "b")]
            for root in roots:
                root.mkdir()
                (root / "pack.mcmeta").write_text("metadata")
                (root / "file").write_text("commands")
            original = pack_digest(roots[0])
            self.assertEqual(original, pack_digest(roots[1]))
            (roots[1] / "file").write_text("different")
            self.assertNotEqual(original, pack_digest(roots[1]))
            (roots[0] / "file").rename(roots[0] / "renamed")
            self.assertNotEqual(original, pack_digest(roots[0]))

    def test_main_hashes_materialized_symlinks_before_instrumentation(self):
        """Linked input changes must alter the hash of the uninstrumented copy."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            functions = source / "data/rpg/function"
            scan = functions / "__sand_entity/one/reconcile_scan.mcfunction"
            scan.parent.mkdir(parents=True)
            scan.write_text("execute as @e[type=minecraft:zombie,tag=owned] at @s run function rpg:__sand_entity/one/reconcile\n")
            (scan.parent / "reconcile.mcfunction").write_text("say reconcile\n")
            linked = root / "linked"
            linked.mkdir()
            (functions / "linked").symlink_to(linked, target_is_directory=True)
            digests = []
            case = self

            class FakeHarness:
                def __init__(self, args, target, marker, digest):
                    """Capture the materialized input and metadata without starting Java."""
                    self.pack = Path(args.pack)
                    self.output = root / "output"
                    self.output.mkdir(exist_ok=True)
                    self.evidence = {"input_pack_sha256": digest}
                    digests.append(digest)

                def run(self):
                    """Check the copied linked resource and added probe at the launch boundary."""
                    case.assertEqual((self.pack / "data/rpg/function/linked/body.mcfunction").read_text(), content)
                    case.assertTrue((self.pack / "data/sand_cost_probe/function/reconcile.mcfunction").exists())

            for index, content in enumerate(("say first\n", "say second\n")):
                (linked / "body.mcfunction").write_text(content)
                expected = root / f"expected-{index}"
                shutil.copytree(source, expected)
                with patch("measure_archetype_idle.IdleCostHarness", FakeHarness), patch.object(
                    sys, "argv", ["measure_archetype_idle", "--pack", str(source), "--jar", "unused.jar"]
                ):
                    self.assertEqual(main(), 0)
                self.assertEqual(digests[-1], pack_digest(expected))
            self.assertNotEqual(digests[0], digests[1])

    def test_discovery_uses_exported_identity_and_rejects_ambiguity(self):
        """Discover the exported callback and reject multiple possible archetypes."""
        with tempfile.TemporaryDirectory() as directory:
            pack = Path(directory)
            root = pack / "data/rpg/function/__sand_entity/one"
            root.mkdir(parents=True)
            (root / "reconcile_scan.mcfunction").write_text(
                "execute as @e[type=minecraft:zombie,tag=__sand.a.one] at @s run function rpg:__sand_entity/one/reconcile\n"
            )
            self.assertEqual(reconcile_target(pack), ("rpg:__sand_entity/one/reconcile", "__sand.a.one"))
            other = root.parent / "two"
            other.mkdir()
            (other / "reconcile_scan.mcfunction").write_text("anything")
            with self.assertRaises(ValueError):
                reconcile_target(pack)


if __name__ == "__main__":
    unittest.main()
