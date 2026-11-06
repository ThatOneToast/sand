"""Prepared native servers must admit the scenario's declared player population."""
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from player_validation_harness import PlayerHarness
from validate_cooperative_arena import ArenaHarness
from validate_world_gameplay import WorldGameplayHarness


class PlayerCapacityTests(unittest.TestCase):
    def test_native_properties_match_declared_population(self):
        for harness_type, count in [(PlayerHarness, 2), (ArenaHarness, 4)]:
            with self.subTest(harness=harness_type.__name__), tempfile.TemporaryDirectory() as root:
                harness = object.__new__(harness_type)
                harness.args = SimpleNamespace(version="26.3")
                harness.server = Path(root)
                properties = harness.server / "server.properties"
                properties.write_text("max-players=1\n")
                with patch.object(WorldGameplayHarness, "prepare"):
                    harness.prepare()
                self.assertEqual(properties.read_text().splitlines()[-1], f"max-players={count}")


if __name__ == "__main__":
    unittest.main()
