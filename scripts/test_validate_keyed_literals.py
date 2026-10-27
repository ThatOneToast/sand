"""Focused checks for extraction of production owner resolution in native probes."""
import tempfile
import unittest
from pathlib import Path

from validate_keyed_literals import owner_resolver


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


if __name__ == "__main__":
    unittest.main()
