import importlib.util
import unittest
from unittest.mock import patch
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

    def run_generation(self, generation_duration):
        clock = [0.0]
        requests = []

        def commands(_host, _port, _password, values, **options):
            command = values[0]
            requests.append((command, options))
            if command == "tick query":
                return ["Average time per tick: 12.1ms"]
            if "forceload add" in command:
                clock[0] += generation_duration
                return ["Marked chunks"]
            return ["The time is 42"]

        with (
            patch.object(bench_runtime, "available_port", return_value=25575),
            patch.object(bench_runtime, "write_rcon_properties"),
            patch.object(bench_runtime, "OwnedServer") as server,
            patch.object(bench_runtime, "run_commands", side_effect=commands),
            patch.object(bench_runtime.time, "monotonic", side_effect=lambda: clock[0]),
        ):
            try:
                result = bench_runtime.measure_sample(Path("sand"), Path("project"), 1, 60, Path("output"))
                return result, requests
            finally:
                server.return_value.stop.assert_called_once()

    def test_chunk_generation_uses_benchmark_timeout_and_remaining_budget(self):
        result, requests = self.run_generation(17)
        self.assertEqual(result["chunk_generation_seconds"], 17)
        generation = [options for command, options in requests if "forceload add" in command]
        probes = [options for command, options in requests if "if loaded" in command]
        self.assertEqual(generation, [{"timeout": 60}])
        self.assertEqual(probes, [{"timeout": 43}] * 5)

    def test_initial_generation_consumes_deadline(self):
        with self.assertRaisesRegex(TimeoutError, "forced chunks"):
            self.run_generation(61)


if __name__ == "__main__":
    unittest.main()
