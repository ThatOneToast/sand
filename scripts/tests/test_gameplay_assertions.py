"""A command parser's echoed input must never satisfy a native assertion."""
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_gameplay import GameplayHarness
from vanilla_reload_harness import ValidationFailure


class NativeAssertionTests(unittest.TestCase):
    def harness(self, emitted):
        harness = GameplayHarness.__new__(GameplayHarness)
        harness.lines = []
        harness.send = Mock(side_effect=lambda _: harness.lines.extend(emitted))
        harness.barrier = Mock(side_effect=lambda _: len(harness.lines) - 1)
        return harness

    def test_parser_echo_is_failure_even_with_exact_assertion_token(self):
        harness = self.harness([
            '[00:00:00] [Server thread/INFO]: Invalid path in execute if data ... run say __SAND_CHECK_book__<--[HERE]',
            '[00:00:00] [Server thread/INFO]: execute if data ... run say __SAND_CHECK_book__',
        ])
        with self.assertRaises(ValidationFailure):
            harness.check('if data storage test:proof invalid', 'book')

    def test_completed_server_and_player_chat_emissions_pass(self):
        for speaker in ('Server', 'SandAlpha'):
            harness = self.harness([
                f'[00:00:00] [Server thread/INFO]: [Not Secure] [{speaker}] __SAND_CHECK_book__'
            ])
            self.assertTrue(harness.matches('if score #ok proof matches 1', 'book'))

    def test_stale_and_similar_markers_do_not_pass(self):
        harness = self.harness([
            '[00:00:00] [Server thread/INFO]: [Server] __SAND_CHECK_book__extra',
            '[00:00:00] [Server thread/INFO]: [Server] __SAND_CHECK_other__',
        ])
        harness.lines.append('[00:00:00] [Server thread/INFO]: [Server] __SAND_CHECK_book__')
        self.assertFalse(harness.matches('if score #ok proof matches 1', 'book'))


if __name__ == '__main__':
    unittest.main()
