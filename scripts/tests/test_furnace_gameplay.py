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


class FreshMachineModel(FurnaceHarness):
    """Model retained block-entity state while executing the real setup helper."""

    def __init__(self, machine, skip_removal=False, nonzero_default=None):
        self.machine = machine
        self.values = {"cooking_time_spent": 0, "cooking_total_time": 200,
                       "lit_time_remaining": 1401}
        self.skip_removal = skip_removal
        self.nonzero_default = nonzero_default
        self.events = []

    def send(self, command):
        self.events.append(command)
        block = command.removeprefix("setblock 0 85 0 minecraft:")
        if block == "air":
            if not self.skip_removal:
                self.machine = None
        else:
            if self.machine != block:
                self.values = dict.fromkeys(self.values, 0)
                if self.nonzero_default:
                    self.values[self.nonzero_default] = 7
            self.machine = block

    def check(self, name, condition):
        match = re.fullmatch(r"if data block 0 85 0 \{(\w+):0s\}", condition)
        if match is None:
            raise AssertionError(f"unexpected default assertion: {condition}")
        key = match[1]
        if self.values[key] != 0:
            raise ValidationFailure(name, condition)
        self.events.append(key)

    def native_snapshot(self, label, player):
        self.events.append("snapshot")

    def score(self, label, player, field, value):
        self.events.append(f"score:{field}={value}")

    def call(self, name):
        self.events.append(f"seed:{name}")


class FreshMachineTests(unittest.TestCase):
    def test_each_family_removes_checks_defaults_and_snapshots_before_seed(self):
        for machine in ("furnace", "blast_furnace", "smoker"):
            with self.subTest(machine=machine):
                harness = FreshMachineModel(machine)
                harness.fresh_machine("FRESH", machine, "seed", "Alpha")
                self.assertEqual(harness.events, [
                    "setblock 0 85 0 minecraft:air",
                    f"setblock 0 85 0 minecraft:{machine}",
                    "cooking_time_spent", "cooking_total_time", "lit_time_remaining",
                    "snapshot", "score:elapsed=0", "score:total=0", "score:burn=0",
                    "seed:seed"])

    def test_skipped_removal_rejects_retained_state_before_seed(self):
        harness = FreshMachineModel("furnace", skip_removal=True)
        with self.assertRaises(ValidationFailure):
            harness.fresh_machine("FRESH", "furnace", "seed", "Alpha")
        self.assertNotIn("seed:seed", harness.events)

    def test_each_nonzero_native_default_stops_before_seed(self):
        for key in ("cooking_time_spent", "cooking_total_time", "lit_time_remaining"):
            with self.subTest(key=key):
                harness = FreshMachineModel("furnace", nonzero_default=key)
                with self.assertRaises(ValidationFailure):
                    harness.fresh_machine("FRESH", "furnace", "seed", "Alpha")
                self.assertNotIn("seed:seed", harness.events)


if __name__ == "__main__":
    unittest.main()
