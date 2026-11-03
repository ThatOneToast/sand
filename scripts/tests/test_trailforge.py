"""Regression tests for Trailforge's native evidence and synchronization."""
import argparse
import json
from pathlib import Path
import tempfile
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_trailforge import TrailforgeHarness, pack_digest, unique_objective, RELOAD_MARKER, main
from vanilla_reload_harness import ValidationFailure


class TrailforgeTests(unittest.TestCase):
    def harness(self):
        harness = TrailforgeHarness.__new__(TrailforgeHarness)
        harness.args = argparse.Namespace(timeout=1)
        harness.send = Mock()
        harness.checks = []
        harness.dispatches = 0
        harness.lines = []
        return harness

    def test_pack_digest_tracks_paths_and_bytes_not_creation_order(self):
        with tempfile.TemporaryDirectory() as directory:
            a, b = Path(directory) / 'a', Path(directory) / 'b'
            a.mkdir()
            b.mkdir()
            for root, names in ((a, ('one', 'two')), (b, ('two', 'one'))):
                for name in names:
                    (root / name).write_text(name)
            self.assertEqual(pack_digest(a), pack_digest(b))
            (b / 'one').rename(b / 'other')
            self.assertNotEqual(pack_digest(a), pack_digest(b))
            (b / 'other').rename(b / 'one')
            (b / 'one').write_text('changed')
            self.assertNotEqual(pack_digest(a), pack_digest(b))

    def test_discovery_rejects_ambiguous_or_missing_objectives(self):
        for commands in ('', 'scoreboard players set @s first 80\nscoreboard players set @s second 80\n'):
            with self.assertRaises(ValidationFailure):
                unique_objective(commands, 'set', 80)
        self.assertEqual(unique_objective('scoreboard players set @s ready 80\n' * 2, 'set', 80), 'ready')

    def test_failed_assertion_never_enters_evidence(self):
        harness = self.harness()
        harness.passed = Mock(return_value=False)
        with self.assertRaises(ValidationFailure):
            harness.check('FALSE', 'if entity SandAlpha')
        self.assertEqual(harness.checks, [])

    def test_fixture_failure_records_phase_and_never_starts_server(self):
        with tempfile.TemporaryDirectory() as directory:
            harness = self.harness()
            harness.output = Path(directory)
            harness.log_path = harness.output / 'latest.log'
            for field in ('pack_sha256', 'source_root', 'source_tree', 'revision',
                          'harness_revision', 'harness_sha256', 'java_version', 'server_jar_sha256'):
                setattr(harness, field, 'test')
            harness.source_manifest = {}
            harness.worktree_dirty = harness.harness_dirty = False
            harness.load_fixture = Mock(side_effect=ValidationFailure('fixture', 'missing recipe'))
            harness.run = Mock()
            with patch('validate_trailforge.TrailforgeHarness', return_value=harness), patch('builtins.print'):
                self.assertEqual(main(['--pack', directory, '--jar', 'unused']), 1)
            harness.run.assert_not_called()
            evidence = json.loads((harness.output / 'evidence.json').read_text())
            self.assertFalse(evidence['passed'])
            self.assertEqual(evidence['failure']['phase'], 'fixture')
            self.assertEqual(evidence['failure']['message'], 'missing recipe')
            self.assertEqual(evidence['checks'], [])

    def test_step_waits_for_actual_game_time_not_submission(self):
        harness = self.harness()
        harness.passed = Mock(side_effect=[False, True])
        with patch('validate_trailforge.time.sleep'):
            harness.step(40, 'REGEN')
        self.assertEqual(harness.passed.call_count, 2)
        self.assertIn('= #tick_end trail_audit', harness.passed.call_args.args[1])
        self.assertIn(('tick step 40',), [call.args for call in harness.send.call_args_list])

    def test_tick_timeout_is_failure(self):
        harness = self.harness()
        harness.passed = Mock(return_value=False)
        with patch('validate_trailforge.time.monotonic', side_effect=[0, 2]):
            with self.assertRaises(ValidationFailure):
                harness.step(1, 'NEVER')

    def test_invalid_tick_count_rejected_before_commands(self):
        for ticks in (0, -1, True, 1.5):
            harness = self.harness()
            with self.assertRaises(ValueError):
                harness.step(ticks, 'INVALID')
            harness.send.assert_not_called()

    def test_reload_requires_new_load_marker_and_completed_tick(self):
        harness = self.harness()
        harness.lines = ['[Server] ' + RELOAD_MARKER]
        harness.wait_for = Mock()
        def step(*_):
            if harness.step.call_count == 2:
                harness.lines.append('[Server] ' + RELOAD_MARKER)
        harness.step = Mock(side_effect=step)
        self.assertEqual(harness.reload_pack(), 2)
        self.assertEqual(harness.step.call_count, 2)
        self.assertEqual(harness.wait_for.call_args.args[2], 1)

    def test_reload_start_alone_never_proves_completion(self):
        harness = self.harness()
        harness.wait_for = Mock()
        harness.step = Mock()
        with patch('validate_trailforge.time.monotonic', side_effect=[0, 0, 2]):
            with self.assertRaises(ValidationFailure):
                harness.reload_pack()

    def test_both_players_queue_before_one_dispatch_and_clock_assertion(self):
        harness = self.harness()
        harness.check = Mock()
        harness.menu_commands = ['trigger menu set 1']
        harness.menu_many((('SandAlpha', 0), ('SandBeta', 0)))
        commands = [call.args[0] for call in harness.send.call_args_list]
        dispatches = [i for i, command in enumerate(commands) if command == 'function trail:__sand_dialog_tick']
        self.assertEqual(len(dispatches), 2)
        for player in ('SandAlpha', 'SandBeta'):
            self.assertLess(commands.index(f'execute as {player} at @s run trigger menu set 1'), dispatches[-1])
        harness.check.assert_called_once_with('DISPATCH_1_SAME_TICK', 'if score #before trail_audit = #after trail_audit')


if __name__ == '__main__':
    unittest.main()
