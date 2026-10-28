"""Focused checks for extraction of production owner resolution in native probes."""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from validate_keyed_literals import owner_resolver, owner_layout, trace_metrics


class OwnerResolverTests(unittest.TestCase):
    def frame(self, directory, resolver, name="write"):
        clear = "data remove storage rpg:state __sand_owner.value"
        (directory / f"{name}.mcfunction").write_text("\n".join([
            clear, 'data modify storage rpg:state __sand_owner.value set value "$(literal)"',
            *resolver,
            "function unified_state:sand/state_data/keyed/example with storage rpg:state __sand_owner",
            clear,
        ]))

    def test_before_and_after_resolvers_are_copied_exactly(self):
        for resolver in (
            [f"data modify storage rpg:state __sand_owner.u{i} set from entity @s UUID[{i}]" for i in range(4)],
            ["data remove storage rpg:state __sand_owner.uuid",
             "data modify storage rpg:state __sand_owner.uuid set from entity @s UUID"],
            ["execute unless entity @s run data remove storage rpg:state __sand_owner.uuid",
             "data modify storage rpg:state __sand_owner.uuid set from entity @s UUID"],
        ):
            with self.subTest(resolver=resolver), tempfile.TemporaryDirectory() as root:
                directory = Path(root)
                self.frame(directory, [], "default")
                self.frame(directory, resolver)
                self.assertEqual(owner_resolver(directory), resolver)

    def test_rejects_uncleared_single_uuid_and_missing_resolvers(self):
        for resolver in ([], ["data modify storage rpg:state __sand_owner.uuid set from entity @s UUID"]):
            with self.subTest(resolver=resolver), tempfile.TemporaryDirectory() as root:
                directory = Path(root)
                self.frame(directory, resolver)
                with self.assertRaises(ValueError):
                    owner_resolver(directory)


class OwnerLayoutTests(unittest.TestCase):
    def test_known_layouts_and_unknown_mixed_layouts(self):
        for lines, expected in (
            (['$data get storage rpg:state owners[{uuid:$(uuid)}].value'], "legacy"),
            (['$data get storage rpg:state owners_v2."$(uuid)".value'], "indexed"),
            ([], None),
            (['$data get storage rpg:state owners_v3."$(uuid)".value'], None),
            (['$data get storage rpg:state owners_v2."$(u0)".value'], None),
            (['$data get storage rpg:state owners[{uuid:$(uuid)}].value',
              '$data get storage rpg:state owners_v2."$(uuid)".value'], None),
        ):
            with self.subTest(lines=lines), tempfile.TemporaryDirectory() as root:
                path = Path(root)
                (path / "read.mcfunction").write_text("\n".join(lines))
                if expected is None:
                    with self.assertRaises(ValueError):
                        owner_layout(path)
                else:
                    self.assertEqual(owner_layout(path), expected)


class TraceMetricsTests(unittest.TestCase):
    def test_counts_evaluations_reads_and_mutations_independently(self):
        metrics = trace_metrics("""
[C] execute as @e[tag=test,limit=1] run function probe:get
[C] execute unless entity @s run data remove storage rpg:state __sand_owner.uuid
[C] data modify storage rpg:state __sand_owner.uuid set from entity @s UUID
[E] Nothing changed. The specified properties already have these values
[C] data get storage rpg:state owners[{uuid:[I;1,2,3,4]}].value
[R = 7] data get storage rpg:state owners[{uuid:[I;1,2,3,4]}].value
[R = 0] data remove storage rpg:state absent
[R = 1] data modify storage other:state value set value 1
""")
        self.assertEqual(metrics, {
            "command_evaluations_including_wrapper": 4,
            "probe_wrapper_evaluations": 1,
            "entity_uuid_read_commands": 1,
            "successful_storage_mutation_commands": 0,
            "successful_storage_get_commands": 1,
        })

    def test_positive_results_count_commands_not_result_magnitude(self):
        metrics = trace_metrics("""
[R = 3] data remove storage rpg:state values[]
[R = 1] execute unless entity @s run data remove storage rpg:state __sand_owner.uuid
[R = 1] data modify storage rpg:state __sand_owner.uuid set from entity @s UUID
""")
        self.assertEqual(metrics["successful_storage_mutation_commands"], 3)


if __name__ == "__main__":
    unittest.main()
