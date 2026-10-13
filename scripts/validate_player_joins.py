#!/usr/bin/env python3
"""Validate generated joins across real disconnects, reloads and datapacks."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure, RELOAD_START

ROOT = Path(__file__).resolve().parents[1]
NAMESPACES = ("joina", "joinb")


def export_packs():
    with tempfile.TemporaryDirectory(prefix="sand-join-export-") as directory:
        dump = Path(directory) / "packs.json"
        subprocess.run([
            "cargo", "test", "-p", "sand-core", "--test", "join_lifecycle_export",
            "--", "--ignored", "--exact", "dump_join_packs",
        ], cwd=ROOT, env=dict(os.environ, SAND_JOIN_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


def write_pack(path, records):
    path.mkdir(parents=True)
    (path / "pack.mcmeta").write_text(json.dumps({"pack": {
        "pack_format": 107, "min_format": 107, "max_format": 107,
        "description": "Sand namespace-owned join audit",
    }}))
    for record in records:
        target = path / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(record["content"])


class JoinHarness(PlayerHarness):
    def __init__(self, args, packs):
        super().__init__(args)
        self.packs = packs

    def prepare(self):
        super().prepare()
        shutil.copytree(Path(self.args.pack).with_name("joinb"), self.server / "world/datapacks/joinb")

    def leave_objective(self, namespace):
        load = next(record["content"] for record in self.packs[namespace]
                    if record["dir"] == "function" and record["path"] == "__sand_lifecycle_load")
        return next(line.split()[3] for line in load.splitlines()
                    if line.endswith(" minecraft.custom:minecraft.leave_game"))

    def counts(self, phase, player, a, b):
        for namespace, count in zip(NAMESPACES, (a, b)):
            for handler in ("a", "b"):
                self.eventually(f"{phase}_{player}_{namespace}_{handler}",
                                f"if score {player} {namespace}:{handler} matches {count}")

    def settle(self):
        """Observe five actual game ticks before checking unchanged counters."""
        self.send("execute store result score #deadline join_check run time query gametime")
        self.send("scoreboard players add #deadline join_check 5")
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send("execute store result score #now join_check run time query gametime")
            if self.passed("SETTLED", "execute if score #now join_check >= #deadline join_check run say __SAND_WORLD_SETTLED_OK__"):
                return
            time.sleep(0.1)
        raise ValidationFailure("join-settle", "server did not advance five game ticks")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.send("scoreboard objectives add join_check dummy")
        # A legacy/other pack's global marker must neither suppress nor be reset.
        self.send("scoreboard objectives add __sand_join dummy")
        self.send("scoreboard players set SandBeta __sand_join 1")
        for namespace in NAMESPACES:
            for handler in ("a", "b"):
                self.send(f"scoreboard objectives add {namespace}:{handler} dummy")
            self.check(f"{namespace}_INITIAL_ABSENT",
                       f"unless score SandBeta {self.leave_objective(namespace)} = SandBeta {self.leave_objective(namespace)}")
            # Exercise a retained nonzero statistic as well as Beta's absent one.
            self.send(f"scoreboard players set SandAlpha {self.leave_objective(namespace)} 7")
        with self.clients(players=("SandBeta",)):
            self.counts("FIRST", "SandBeta", 1, 1)
            with self.clients(players=("SandAlpha",)):
                self.counts("FIRST", "SandAlpha", 1, 1)
                self.settle()
                self.counts("NO_REPEAT", "SandBeta", 1, 1)
                self.counts("NO_REPEAT", "SandAlpha", 1, 1)
            self.eventually("ALPHA_LEFT", "unless entity @a[name=SandAlpha]")
            for namespace in NAMESPACES:
                self.check(f"{namespace}_VANILLA_LEAVE", f"if score SandAlpha {self.leave_objective(namespace)} matches 8")
            with self.clients(players=("SandAlpha",)):
                self.counts("RECONNECT", "SandAlpha", 2, 2)
                self.settle()
                self.counts("BETA_STAYS", "SandBeta", 1, 1)
            self.eventually("ALPHA_LEFT_AGAIN", "unless entity @a[name=SandAlpha]")
            # Run only one pack's generated load lifecycle while Alpha is offline.
            self.send("function joina:__sand_lifecycle_load")
            self.counts("ONLY_A_RELOAD", "SandBeta", 2, 1)
            self.settle()
            self.counts("B_NOT_RESET", "SandBeta", 2, 1)
            with self.clients(players=("SandAlpha",)):
                self.counts("OFFLINE_RELOAD", "SandAlpha", 3, 3)
                start = len(self.lines)
                self.send("reload")
                self.wait_for(RELOAD_START, "join-reload", start)
                self.counts("BOTH_RELOAD", "SandAlpha", 4, 4)
                self.counts("BOTH_RELOAD", "SandBeta", 3, 2)
                self.settle()
                self.counts("STABLE", "SandAlpha", 4, 4)
                self.counts("STABLE", "SandBeta", 3, 2)
                self.check("LEGACY_MARKER_UNTOUCHED", "if score SandBeta __sand_join matches 1")
        print("PASSED: two namespaces, real reconnects, retained/absent statistics, callback reentry and reload isolation", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    packs = export_packs()
    with tempfile.TemporaryDirectory(prefix="sand-join-packs-") as directory:
        for namespace, records in packs.items():
            write_pack(Path(directory) / namespace, records)
        args.pack = str(Path(directory) / "joina")
        harness = JoinHarness(args, packs)
        try:
            harness.run()
        except ValidationFailure as error:
            print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
            return 1
        print(f"PASSED: Minecraft 26.2 player joins; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
