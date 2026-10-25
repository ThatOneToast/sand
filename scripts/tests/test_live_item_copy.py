"""Failure-oracle regressions: silence, stale callbacks and mutation cannot pass."""
from pathlib import Path
import re
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_live_item_copy import CopyHarness
from vanilla_reload_harness import ValidationFailure


class OracleProbe(CopyHarness):
    def __init__(self, callback=(0, 0), mutation=None, precondition=True):
        self.native = {"missing_source": "item replace entity @s weapon.offhand from entity @e[tag=copy_absent,limit=1] weapon.mainhand"}
        self.callback = callback
        self.mutation = mutation
        self.precondition = precondition
        self.commands = []
        self.scores = {}
        self.stacks = {"entity Alpha SelectedItem": "complete source",
                       "entity Alpha equipment.offhand": "complete destination"}
        self.saved = {}
        self.labels = []

    def send(self, command):
        self.commands.append(command)
        words = command.split()
        if command.startswith("scoreboard players set "):
            self.scores[words[3]] = int(words[-1])
        if command.startswith("execute "):
            for key, value in zip(("#failure_success", "#failure_result"), self.callback):
                if value is not None:
                    self.scores[key] = value
            self.mutate("native")

    def mutate(self, stage):
        if self.mutation == stage + "_destination":
            self.stacks["entity Alpha equipment.offhand"] = "changed"
        if self.mutation == stage + "_source":
            self.stacks["entity Alpha SelectedItem"] = "cleared"

    def capture(self, key, address):
        self.saved[key] = self.stacks[address]

    def call(self, path, frame):
        self.commands.append("fixture call")
        self.mutate("fixture")

    def equal(self, label, expected, actual):
        self.labels.append(label)
        if self.saved[expected] != self.stacks[actual]:
            raise ValidationFailure(label, "stack changed")

    def check(self, label, condition):
        self.labels.append(label)
        match = re.fullmatch(r"if score (\S+) copy_oracle matches (-?\d+)", condition)
        valid = self.scores.get(match[1]) == int(match[2]) if match else self.precondition
        if not valid:
            raise ValidationFailure(label, condition)

    def exercise(self):
        self.expected_failure("missing_source", "as Alpha", "Alpha", "unless entity @e[tag=copy_absent]")


class FailureOracleTests(unittest.TestCase):
    def test_silent_failure_requires_two_fresh_zero_callbacks(self):
        probe = OracleProbe()
        probe.exercise()
        self.assertEqual(probe.scores, {"#failure_success": 0, "#failure_result": 0})
        fixture = probe.commands.index("fixture call")
        self.assertEqual(probe.commands[fixture + 1:fixture + 3], [
            "scoreboard players set #failure_success copy_oracle -71",
            "scoreboard players set #failure_result copy_oracle -93",
        ])
        self.assertLess(probe.labels.index("missing_source_FIXTURE_NO_MOVE"),
                        probe.labels.index("missing_source_NATIVE_ONLY_FAILURE_SUCCESS"))
        self.assertIn("missing_source_NATIVE_ONLY_NO_WRITE", probe.labels)
        self.assertIn("missing_source_NATIVE_ONLY_NO_MOVE", probe.labels)

    def test_success_silence_or_partial_callback_is_not_failure(self):
        for callback in ((1, 1), (1, 0), (0, 1), (None, None), (None, 0), (0, None)):
            with self.subTest(callback=callback), self.assertRaises(ValidationFailure):
                OracleProbe(callback=callback).exercise()

    def test_fixture_or_native_mutation_fails_even_with_failure_callback(self):
        for stage in ("fixture", "native"):
            for endpoint in ("source", "destination"):
                probe = OracleProbe(mutation=stage + "_" + endpoint)
                with self.subTest(stage=stage, endpoint=endpoint), self.assertRaises(ValidationFailure):
                    probe.exercise()
                if stage == "fixture":
                    self.assertFalse(any(c.startswith("execute ") for c in probe.commands))

    def test_missing_precondition_stops_before_fixture_or_replay(self):
        probe = OracleProbe(precondition=False)
        with self.assertRaises(ValidationFailure):
            probe.exercise()
        self.assertEqual(probe.commands, [])

    def test_oracle_rejects_multicommand_replay(self):
        probe = OracleProbe()
        probe.native["missing_source"] += "\nsay unexpected"
        with self.assertRaises(ValidationFailure):
            probe.exercise()
        self.assertEqual(probe.commands, [])


if __name__ == "__main__":
    unittest.main()
