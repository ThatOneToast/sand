"""Negative controls for the furnace harness oracle; no Minecraft execution."""
from pathlib import Path
import re
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_furnace_gameplay import FurnaceHarness
from vanilla_reload_harness import ValidationFailure


class NestedProbeModel(FurnaceHarness):
    """Interpret the real probe's commands against controlled read outcomes."""

    def __init__(self, inner_result):
        self.objectives = {"burn": "burn", "elapsed": "elapsed"}
        # Match the preceding both_players result: Beta already holds 93.
        self.scores = {("Beta", "burn"): 93, ("Alpha", "elapsed"): 0}
        self.inner_result = inner_result
        self.checked = []

    def send(self, command):
        match = re.fullmatch(r"scoreboard players set (\w+) (\w+) (-?\d+)", command)
        if match is None:
            raise AssertionError(f"unexpected probe setup: {command}")
        player, objective, value = match.groups()
        self.scores[player, objective] = int(value)

    def call(self, name, player):
        if (name, player) != ("nested_players", "Alpha"):
            raise AssertionError(f"unexpected probe call: {name}, {player}")
        # Always retain the outer read. None injects only the skipped-inner bug.
        self.scores["Alpha", "elapsed"] = 71
        if self.inner_result is not None:
            self.scores["Beta", "burn"] = self.inner_result

    def check(self, name, condition):
        match = re.fullmatch(r"if score (\w+) (\w+) matches (-?\d+)", condition)
        if match is None:
            raise AssertionError(f"unexpected probe assertion: {condition}")
        player, objective, expected = match.groups()
        if self.scores[player, objective] != int(expected):
            raise ValidationFailure(name, condition)
        self.checked.append(name)


class FurnaceOracleTests(unittest.TestCase):
    def test_success_checks_inner_rebinding_and_outer_restoration(self):
        harness = NestedProbeModel(93)
        harness.nested_player_probe("PROBE", "Alpha", "Beta")
        self.assertEqual(harness.checked, [
            "PROBE_NESTED_TARGET_REBINDS", "PROBE_NESTED_TARGET_RESTORES"])

    def test_skipped_inner_read_fails_despite_preceding_success(self):
        harness = NestedProbeModel(None)
        with self.assertRaises(ValidationFailure) as failure:
            harness.nested_player_probe("PROBE", "Alpha", "Beta")
        self.assertEqual(failure.exception.phase, "PROBE_NESTED_TARGET_REBINDS")
        self.assertEqual(harness.scores["Alpha", "elapsed"], 71)

    def test_wrong_outer_address_read_fails_rebinding(self):
        with self.assertRaises(ValidationFailure) as failure:
            NestedProbeModel(71).nested_player_probe("PROBE", "Alpha", "Beta")
        self.assertEqual(failure.exception.phase, "PROBE_NESTED_TARGET_REBINDS")


if __name__ == "__main__":
    unittest.main()
