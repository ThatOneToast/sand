#!/usr/bin/env python3
"""Prove atomic inventory match branches on Minecraft 26.2 with two real players.

Exports the maintained Rust regression fixture unchanged. Console commands only
prepare inventory/world state and assert the effects of generated functions.
"""
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
NAMESPACE = "match_actions"


def export_fixture():
    with tempfile.TemporaryDirectory(prefix="sand-inventory-match-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run([
            "cargo", "test", "-p", "sand-core", "--test", "inventory_match_actions",
            "--", "--ignored", "--exact", "dump_inventory_match_pack",
        ], cwd=ROOT, env=dict(os.environ, SAND_INVENTORY_MATCH_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


def write_pack(path, records):
    path.mkdir(parents=True)
    (path / "pack.mcmeta").write_text(json.dumps({"pack": {
        "pack_format": 107, "min_format": 107, "max_format": 107,
        "description": "Sand inventory match regression",
    }}))
    for record in records:
        target = path / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(record["content"])


class InventoryMatchHarness(PlayerHarness):
    def equip(self, player, slot, item):
        self.send(f"item replace entity {player} weapon.{slot} with {item}")

    def item(self, label, player, slot, item):
        condition = (f"unless items entity {player} weapon.{slot} *" if item == "air"
                     else f"if items entity {player} weapon.{slot} minecraft:{item}")
        self.check(label, condition)

    def run_client(self):
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            self.load_test_chunk(dimension)
        with self.clients():
            for index, player in enumerate(self.players):
                self.send(f"gamemode creative {player}")
                self.send(f"tp {player} {index * 3 + 0.5} 85 0.5")
            self.send("tick freeze")
            try:
                # Swap matching/nonmatching players to catch stale shared guards.
                for matching, other in (self.players, self.players[::-1]):
                    for player in self.players:
                        self.equip(player, "mainhand", "minecraft:dirt")
                        self.equip(player, "offhand", "minecraft:paper")
                    self.equip(matching, "offhand",
                               'minecraft:paper[minecraft:custom_data={"match_actions:token":1b,extra:7}]')
                    self.send(f"function {NAMESPACE}:consume_token")
                    self.item(f"{matching}_CONSUMED", matching, "offhand", "air")
                    self.item(f"{matching}_SECOND_ACTION", matching, "mainhand", "diamond")
                    self.item(f"{other}_NO_MATCH_OFFHAND", other, "offhand", "paper")
                    self.item(f"{other}_NO_MATCH_MAINHAND", other, "mainhand", "dirt")
                    self.equip(matching, "mainhand", "minecraft:dirt")
                    self.send(f"function {NAMESPACE}:consume_token")
                    self.item(f"{matching}_NO_REPEAT", matching, "mainhand", "dirt")

                # Outer/inner true, outer true/inner false, outer false/inner true.
                # Both executors remain in the Overworld while their function's
                # supplied origin is in the Nether and away from either player.
                for player, other in (self.players, self.players[::-1]):
                    for outer, inner in ((True, True), (True, False), (False, True)):
                        label = f"{player}_{outer}_{inner}"
                        self.equip(player, "offhand", "minecraft:paper" if outer else "minecraft:stone")
                        self.equip(player, "mainhand", "minecraft:emerald" if inner else "minecraft:dirt")
                        self.equip(other, "offhand", "minecraft:paper")
                        self.equip(other, "mainhand", "minecraft:emerald")
                        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
                            self.send(f"execute in {dimension} run setblock 8 90 8 minecraft:air")
                        self.send(f"execute as {player} in minecraft:the_nether positioned 8 90 8 run function {NAMESPACE}:nested_match")
                        self.item(label + "_OUTER", player, "offhand", "stick" if outer else "stone")
                        self.item(label + "_INNER", player, "mainhand",
                                  "air" if outer and inner else "emerald" if inner else "dirt")
                        self.item(label + "_OTHER_OFFHAND", other, "offhand", "paper")
                        self.item(label + "_OTHER_MAINHAND", other, "mainhand", "emerald")
                        expected = "gold_block" if outer and inner else "air"
                        self.check(label + "_ORIGIN", f"in minecraft:the_nether if block 8 90 8 minecraft:{expected}")
                        self.check(label + "_DIMENSION", "in minecraft:overworld if block 8 90 8 minecraft:air")
            finally:
                if self.process and self.process.poll() is None:
                    self.send("tick unfreeze")
        print("PASSED: destructive match, partial custom identity, repeat gating, nested true/false branches, two-player isolation, executor/position/dimension", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    records = export_fixture()
    with tempfile.TemporaryDirectory(prefix="sand-inventory-match-pack-") as directory:
        pack = Path(directory) / NAMESPACE
        write_pack(pack, records)
        args.pack = str(pack)
        harness = InventoryMatchHarness(args)
        try:
            harness.run()
        except ValidationFailure as error:
            print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
            return 1
        print(f"PASSED: Minecraft 26.2 inventory matches; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
