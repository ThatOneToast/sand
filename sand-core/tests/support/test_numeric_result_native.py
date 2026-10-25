"""Focused regressions for the native probe's reload assertion hook."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "numeric_result_native", Path(__file__).with_name("numeric_result_native.py")
)
probe_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe_module)


class ReloadAssertions(unittest.TestCase):
    def test_reload_waits_for_load_tag_before_repeating_assertions(self):
        probe = object.__new__(probe_module.Probe)
        probe.lines = ["console reload marker"]
        events = []

        def wait(_self, pattern, phase, start=0):
            events.append((pattern, phase, start))
            return 0

        def assertions(phase):
            events.append(("assertions", phase))
            probe.lines.append("assertions completed")

        probe.run_assertions = assertions
        with patch.object(probe_module.ServerHarness, "wait_for", wait):
            position = probe.wait_for("console marker", "reload-complete", 7)
        self.assertEqual(events, [
            ("console marker", "reload-complete", 7),
            ("NUMERIC_RESULT_LOADED", "numeric-reload-load", 7),
            ("assertions", "after_reload"),
        ])
        self.assertEqual(position, 1, "reload error checking includes repeated assertions")

    def test_other_waits_do_not_reenter_assertions(self):
        probe = object.__new__(probe_module.Probe)
        probe.run_assertions = lambda _: self.fail("unexpected assertion reentry")
        with patch.object(probe_module.ServerHarness, "wait_for", return_value=9):
            self.assertEqual(probe.wait_for("batch", "numeric results", 3), 9)


if __name__ == "__main__":
    unittest.main()
