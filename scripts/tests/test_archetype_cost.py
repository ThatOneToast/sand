"""Regressions for the vanilla trace measurement boundary."""
import sys
import tempfile
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from measure_archetype_idle import pack_digest, reconcile_target, trace_counts


class TraceCounts(unittest.TestCase):
    def test_nested_and_sibling_frames_return_to_their_caller(self):
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
        trace = '''    [F] demo:scan size=1
        [C] execute as @e run function demo:body
        [F] demo:body size=1
            [C] say one
        [F] demo:body size=1
            [C] say two
'''
        self.assertEqual(trace_counts(trace), {"demo:body": 2, "demo:scan": 1})

    def test_multiple_tag_roots_do_not_inherit_frames(self):
        trace = '''    [F] demo:a size=1
        [C] say a
    [F] demo:b size=1
        [C] say b
'''
        self.assertEqual(trace_counts(trace), {"demo:a": 1, "demo:b": 1})

    def test_errors_and_unrecognized_records_never_become_zero_cost(self):
        for trace in (
            "", "Traced 1 command(s)", "    [C] say orphan",
            "    [F] demo:root size=1\n        [C] say ok\n        [E] unknown function",
            "    [F] demo:root unknown-format\n        [C] say ok",
            "    [F] demo:root size=1\n        [C]",
        ):
            with self.subTest(trace=trace), self.assertRaises(ValueError):
                trace_counts(trace)

    def test_pack_digest_is_path_independent_and_content_sensitive(self):
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

    def test_discovery_uses_exported_identity_and_rejects_ambiguity(self):
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
