import importlib.util
import unittest
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1] / "bench_runtime.py"
SPEC = importlib.util.spec_from_file_location("bench_runtime", MODULE)
bench_runtime = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(bench_runtime)


class BenchRuntimeTests(unittest.TestCase):
    def test_parse_mspt(self):
        self.assertEqual(bench_runtime.parse_mspt("The average is 3.75 ms per tick"), 3.75)
        self.assertEqual(bench_runtime.parse_mspt("Average time per tick: 12.1ms"), 12.1)

    def test_parse_mspt_fails_closed(self):
        with self.assertRaises(ValueError):
            bench_runtime.parse_mspt("The game is running")

    def test_loaded_requires_positive_time_response(self):
        self.assertTrue(bench_runtime.loaded("The time is 42"))
        self.assertFalse(bench_runtime.loaded("Position is not loaded"))


if __name__ == "__main__":
    unittest.main()
