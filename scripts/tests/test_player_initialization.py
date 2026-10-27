"""Completion and failure regressions for native player initialization evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_player_initialization import InitializationHarness, completed_trace, trace_filename, main, pack_digest
from vanilla_reload_harness import ValidationFailure

TRACE = '''init_audit:probe
    [F] init_audit:probe size=2
        [C] execute as @a at @s run function trail:__sand_lifecycle_init
        [F] trail:__sand_lifecycle_init size=1
            [C] execute unless entity @s[scores={state=1}] run function trail:slow
        [C] say __INIT_TRACE_COMPLETE__ -> 1
'''


class PlayerInitializationTests(unittest.TestCase):
    def test_pack_digest_uses_component_order_and_includes_hidden_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'grapple').mkdir()
            records = [('.sand-build-manifest.json', b'manifest'),
                       ('grapple/execute.mcfunction', b'child'),
                       ('grapple.mcfunction', b'parent')]
            expected = hashlib.sha256()
            for relative, content in records:
                (root / relative).write_bytes(content)
                expected.update(relative.encode() + b'\0' + content + b'\0')
            self.assertEqual(pack_digest(root), expected.hexdigest())
            string_order = hashlib.sha256()
            for relative, content in sorted(records):
                string_order.update(relative.encode() + b'\0' + content + b'\0')
            self.assertNotEqual(pack_digest(root), string_order.hexdigest())

    def test_trace_counts_evaluations_and_wrapper_separately(self):
        self.assertEqual(completed_trace(TRACE), {'init_audit:probe': 2, 'trail:__sand_lifecycle_init': 1})

    def test_rejects_missing_completion_result_errors_and_wrong_wrapper(self):
        variants = (
            TRACE.replace(' -> 1', ''),
            TRACE.replace('say __INIT_TRACE_COMPLETE__ -> 1', 'say unfinished -> 1'),
            TRACE + '        [E] Unknown function\n',
            TRACE.replace('[F] init_audit:probe', '[F] wrong:probe'),
            TRACE + '        [C] say extra -> 1\n',
        )
        for trace in variants:
            with self.subTest(trace=trace), self.assertRaises(ValueError):
                completed_trace(trace)

    def test_requires_exactly_one_completed_trace_file(self):
        line = 'Traced 2 command(s) to output file debug-trace-test.txt'
        self.assertEqual(trace_filename([line]), 'debug-trace-test.txt')
        for lines in ([], [line, line], ['to output file ../../other.txt']):
            with self.assertRaises(ValueError):
                trace_filename(lines)

    def test_failed_check_not_retained(self):
        harness = InitializationHarness.__new__(InitializationHarness)
        harness.checks = []
        harness.passed = Mock(return_value=False)
        with self.assertRaises(ValidationFailure):
            harness.check('failed', 'if entity SandAlpha')
        self.assertEqual(harness.checks, [])

    def test_mismatched_pack_provenance_rejects_before_server_preparation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            pack = root / 'pack'
            pack.mkdir()
            provenance = root / 'provenance.json'
            provenance.write_text('{"pack_sha256":"wrong"}')
            harness = InitializationHarness.__new__(InitializationHarness)
            harness.args = argparse.Namespace(pack=str(pack), provenance=str(provenance))
            with patch('player_validation_harness.PlayerHarness.prepare') as prepare:
                with self.assertRaises(ValidationFailure) as raised:
                    harness.prepare()
                prepare.assert_not_called()
            self.assertEqual(raised.exception.phase, 'provenance')

    def test_failure_records_partial_checks_without_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            harness = argparse.Namespace(
                output=Path(directory), log_path=Path(directory) / 'latest.log',
                provenance={'pack_sha256': 'test'}, samples=[], checks=[{'name': 'prior'}],
                run=Mock(side_effect=ValidationFailure('trace', 'missing completion')))
            with patch('validate_player_initialization.InitializationHarness', return_value=harness), patch('builtins.print'):
                status = main(['--pack', 'unused', '--provenance', 'unused', '--jar', 'unused', '--output', directory])
            self.assertEqual(status, 1)
            evidence = json.loads((Path(directory) / 'evidence.json').read_text())
            self.assertFalse(evidence['passed'])
            self.assertEqual(evidence['failure'], {'phase': 'trace', 'message': 'missing completion'})
            self.assertEqual(evidence['checks'], [{'name': 'prior'}])


if __name__ == '__main__':
    unittest.main()
