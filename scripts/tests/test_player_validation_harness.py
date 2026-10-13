"""Regressions for client cleanup and missing-data comparison guards."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure
from validate_player_participants import PlayerParticipantHarness


class PlayerHarnessTests(unittest.TestCase):
    def make_harness(self, output):
        return PlayerHarness(argparse.Namespace(version="26.2", output=output, timeout=1))

    def test_second_join_failure_reaps_both_created_clients(self):
        with tempfile.TemporaryDirectory() as output:
            harness = self.make_harness(output)
            processes = [Mock(), Mock()]
            harness.wait_for = Mock(side_effect=[None, ValidationFailure("join", "missing second player")])
            with patch("player_validation_harness.subprocess.Popen", side_effect=processes):
                with self.assertRaisesRegex(ValidationFailure, "missing second"):
                    with harness.clients():
                        self.fail("scenario must not start before both players joined")
            for process in processes:
                process.terminate.assert_called_once()
                process.wait.assert_called_once_with(timeout=5)

    def test_premature_client_exit_fails_after_scenario(self):
        with tempfile.TemporaryDirectory() as output:
            harness = self.make_harness(output)
            harness.wait_for = Mock()
            processes = [Mock(), Mock()]
            processes[0].poll.return_value = None
            processes[1].poll.return_value = 1
            with patch("player_validation_harness.subprocess.Popen", side_effect=processes):
                with self.assertRaisesRegex(ValidationFailure, "exited"):
                    with harness.clients():
                        pass
            for process in processes:
                process.terminate.assert_called_once()
                process.wait.assert_called_once_with(timeout=5)

    def test_unresponsive_client_is_killed_and_reaped_on_failure(self):
        with tempfile.TemporaryDirectory() as output:
            harness = self.make_harness(output)
            harness.wait_for = Mock()
            processes = [Mock(), Mock()]
            processes[0].wait.side_effect = [subprocess.TimeoutExpired("client", 5), 0]
            with patch("player_validation_harness.subprocess.Popen", side_effect=processes):
                with self.assertRaisesRegex(ValidationFailure, "scenario"):
                    with harness.clients():
                        raise ValidationFailure("scenario", "scenario failed")
            processes[0].kill.assert_called_once()
            self.assertEqual(processes[0].wait.call_count, 2)
            processes[1].wait.assert_called_once_with(timeout=5)

    def test_nested_reconnect_preserves_logs_and_waits_for_new_join(self):
        with tempfile.TemporaryDirectory() as output:
            harness = self.make_harness(output)
            harness.wait_for = Mock()
            processes = [Mock(), Mock(), Mock()]
            for process in processes:
                process.poll.return_value = None
            with patch("player_validation_harness.subprocess.Popen", side_effect=processes):
                with harness.clients(players=("SandBeta",)):
                    with harness.clients(players=("SandAlpha",)):
                        harness.lines.append("SandAlpha joined the game")
                    processes[0].terminate.assert_not_called()
                    with harness.clients(players=("SandAlpha",)):
                        processes[0].terminate.assert_not_called()
                        self.assertEqual(harness.wait_for.call_args.args[2], 1)
            self.assertEqual({p.name for p in Path(output).glob("*.log")},
                             {"SandBeta.log", "SandAlpha.log", "SandAlpha-2.log"})
            for process in processes:
                process.terminate.assert_called_once()
                process.wait.assert_called_once_with(timeout=5)

    def test_missing_expected_nbt_cannot_pass_as_an_unchanged_copy(self):
        harness = PlayerParticipantHarness.__new__(PlayerParticipantHarness)
        harness.send = Mock()
        harness.check = Mock(side_effect=[None, ValidationFailure("expected", "expected data missing")])
        with self.assertRaisesRegex(ValidationFailure, "missing"):
            harness.equal_nbt("item", "storage audit:item actual", "storage audit:oracle expected")
        harness.send.assert_not_called()


if __name__ == "__main__":
    unittest.main()
