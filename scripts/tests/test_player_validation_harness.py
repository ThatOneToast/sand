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
from validate_player_melee import InconclusiveSetup, PlayerMeleeHarness


class PlayerHarnessTests(unittest.TestCase):
    def test_passive_client_options_are_empty(self):
        self.assertEqual(PlayerHarness.__new__(PlayerHarness).client_options("SandAlpha"), [])

    def test_native_melee_acceptance_timeout_is_inconclusive_not_a_pass(self):
        harness = PlayerMeleeHarness.__new__(PlayerMeleeHarness)
        harness.args = argparse.Namespace(timeout=1)
        harness.poll_health = Mock()
        harness.passed = Mock(return_value=False)
        with patch("validate_player_melee.time.monotonic", side_effect=[0, 0, 2]), patch("validate_player_melee.time.sleep"):
            with self.assertRaises(InconclusiveSetup):
                harness.native_eventually("hit", "if score SandAlpha native matches 1..")
        harness.passed.assert_called_once()

    def test_native_melee_server_send_or_marker_failure_stays_failed(self):
        for phase in ("reload", "world-hit"):
            harness = PlayerMeleeHarness.__new__(PlayerMeleeHarness)
            harness.args = argparse.Namespace(timeout=1)
            harness.poll_health = Mock()
            failure = ValidationFailure(phase, "server/marker failure")
            harness.passed = Mock(side_effect=failure)
            with self.subTest(phase=phase), self.assertRaises(ValidationFailure) as raised:
                harness.native_eventually("hit", "if score SandAlpha native matches 1..")
            self.assertIs(raised.exception, failure)

    def test_all_melee_poll_stages_fail_on_server_exit_or_client_protocol_error(self):
        for stage in ("ready", "native_hit", "native_ticks"):
            for failure_kind in ("server", "client", "client_exit"):
                harness = PlayerMeleeHarness.__new__(PlayerMeleeHarness)
                harness.args = argparse.Namespace(timeout=1)
                harness.process = Mock()
                harness.process.poll.return_value = 1 if failure_kind == "server" else None
                client_process = Mock()
                client_process.poll.return_value = 1 if failure_kind == "client_exit" else None
                harness.client_processes = [client_process]
                harness.status = Mock(side_effect=ValidationFailure("client-control", "protocol failure"))
                harness.passed = Mock(return_value=True)
                refresh = Mock()
                with self.subTest(stage=stage, failure_kind=failure_kind), self.assertRaises(ValidationFailure) as raised:
                    if stage == "ready":
                        harness.wait_status("SandAlpha", lambda value: True, "ready")
                    else:
                        harness.native_eventually(stage, "if entity SandAlpha", refresh=refresh)
                self.assertNotIsInstance(raised.exception, InconclusiveSetup)
                self.assertEqual(raised.exception.phase, {
                    "server": "melee-server", "client": "client-control", "client_exit": "player-client",
                }[failure_kind])
                harness.passed.assert_not_called()
                refresh.assert_not_called()

    def test_client_error_published_during_native_poll_cannot_be_hidden_by_success(self):
        harness = PlayerMeleeHarness.__new__(PlayerMeleeHarness)
        harness.args = argparse.Namespace(timeout=1)
        harness.process = Mock()
        harness.process.poll.return_value = None
        failure = ValidationFailure("client-control", "late protocol failure")
        harness.status = Mock(side_effect=[None, None, failure])
        harness.passed = Mock(return_value=True)
        with self.assertRaises(ValidationFailure) as raised:
            harness.native_eventually("hit", "if entity SandAlpha")
        self.assertIs(raised.exception, failure)

    def test_melee_protocol_error_is_failure_not_inconclusive(self):
        with tempfile.TemporaryDirectory() as output:
            harness = PlayerMeleeHarness(argparse.Namespace(version="26.2", output=output, timeout=1))
            harness.client_options("SandAlpha")
            (Path(output) / "SandAlpha-control/status.json").write_text('{"error":"stale target"}')
            with self.assertRaises(ValidationFailure) as raised:
                harness.status("SandAlpha")
            self.assertNotIsInstance(raised.exception, InconclusiveSetup)

    def make_harness(self, output):
        harness = PlayerHarness(argparse.Namespace(version="26.2", output=output, timeout=1))
        harness.send = Mock()
        return harness

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

    def test_success_disconnects_on_server_before_terminating_clients(self):
        with tempfile.TemporaryDirectory() as output:
            harness = self.make_harness(output)
            process = Mock()
            process.poll.return_value = None
            def observed(pattern, phase, start):
                process.terminate.assert_not_called()
                if phase == "player-leave":
                    harness.send.assert_called_once_with("kick SandAlpha Validation complete")
                    self.assertIsNotNone(pattern.search("SandAlpha left the game"))
            harness.wait_for = Mock(side_effect=observed)
            with patch("player_validation_harness.subprocess.Popen", return_value=process):
                with harness.clients(players=("SandAlpha",)):
                    pass
            self.assertEqual(harness.wait_for.call_count, 2)
            process.terminate.assert_called_once()

    def test_missing_expected_nbt_cannot_pass_as_an_unchanged_copy(self):
        harness = PlayerParticipantHarness.__new__(PlayerParticipantHarness)
        harness.send = Mock()
        harness.check = Mock(side_effect=[None, ValidationFailure("expected", "expected data missing")])
        with self.assertRaisesRegex(ValidationFailure, "missing"):
            harness.equal_nbt("item", "storage audit:item actual", "storage audit:oracle expected")
        harness.send.assert_not_called()


if __name__ == "__main__":
    unittest.main()
