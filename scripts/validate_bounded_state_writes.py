#!/usr/bin/env python3
"""Prove guarded bounded State writes preserve clamps for two real players."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]
NAMESPACE = "bounded_mutation"


def export_fixture():
    """Export the same Rust fixture used by the command regression tests."""
    with tempfile.TemporaryDirectory(prefix="sand-bounded-mutation-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run([
            "cargo", "test", "-p", "sand-core", "--test", "bounded_mutation_atomicity",
            "--", "--ignored", "--exact", "dump_bounded_mutation_pack",
        ], cwd=ROOT, env=dict(os.environ, SAND_BOUNDED_MUTATION_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


def write_pack(path, records):
    """Materialize compiler records without changing their command bodies."""
    path.mkdir(parents=True)
    (path / "pack.mcmeta").write_text(json.dumps({"pack": {
        "pack_format": 107, "min_format": 107, "max_format": 107,
        "description": "Sand bounded State mutation regression",
    }}))
    for record in records:
        target = path / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(record["content"])


class BoundedWriteHarness(PlayerHarness):
    """Drive exported guarded writes with independently controlled field values."""

    def __init__(self, args, objective):
        """Retain the fixture's declared State field for setup and assertions."""
        super().__init__(args)
        self.objective = objective

    def values(self, alpha, beta):
        """Set distinct public State field values before a generated operation."""
        for player, value in zip(self.players, (alpha, beta)):
            self.send(f"scoreboard players set {player} {self.objective} {value}")
            self.check(f"SETUP_{player}", f"if score {player} {self.objective} matches {value}")

    def invoke(self, function, alpha, beta):
        """Check both players immediately after one synchronous generated call."""
        self.send(f"function {NAMESPACE}:{function}")
        for player, expected in zip(self.players, (alpha, beta)):
            self.send(f"scoreboard players get {player} {self.objective}")
            self.check(f"{function}_{player}_{expected}", f"if score {player} {self.objective} matches {expected}")

    def run_client(self):
        """Exercise upper/lower clamps and repeated guards without tick races."""
        self.load_test_chunk("minecraft:overworld")
        with self.clients():
            for player in self.players:
                self.eventually(f"INITIALIZED_{player}",
                                f"if score {player} {self.objective} matches 95")
            self.send("tick freeze")
            try:
                self.values(95, 40)
                self.invoke("iterated_add", 100, 50)
                self.invoke("iterated_add", 100, 60)
                self.values(5, 40)
                self.invoke("iterated_subtract", 0, 30)
                self.invoke("iterated_subtract", 0, 20)
                self.values(95, 99)
                self.invoke("whole_write", 100, 100)
            finally:
                if self.process and self.process.poll() is None:
                    self.send("tick unfreeze")
        print("PASSED: upper/lower bounds, one guard per write, repeat gating and per-player isolation", flush=True)


def main():
    """Build the fixture and validate it on a disposable Minecraft 26.2 server."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    fixture = export_fixture()
    with tempfile.TemporaryDirectory(prefix="sand-bounded-mutation-pack-") as directory:
        pack = Path(directory) / NAMESPACE
        write_pack(pack, fixture["records"])
        args.pack = str(pack)
        harness = BoundedWriteHarness(args, fixture["objective"])
        try:
            harness.run()
        except ValidationFailure as error:
            print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
            return 1
        print(f"PASSED: Minecraft 26.2 bounded State writes; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
