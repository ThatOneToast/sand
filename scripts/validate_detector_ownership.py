#!/usr/bin/env python3
"""Exercise two generated datapacks' built-in events in both vanilla tick orders."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from player_validation_harness import PlayerHarness
from validate_player_joins import JoinHarness, write_pack
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]
NAMESPACES = ("detecta", "detectb")
EVENTS = ("level", "equip", "unequip", "death", "respawn")


def export_packs():
    with tempfile.TemporaryDirectory(prefix="sand-detector-export-") as directory:
        dump = Path(directory) / "packs.json"
        subprocess.run([
            "cargo", "test", "-p", "sand-core", "--test", "builtin_detector_ownership",
            "--", "--ignored", "--exact", "dump_detector_packs",
        ], cwd=ROOT, env=dict(os.environ, SAND_DETECTOR_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


class DetectorHarness(PlayerHarness):
    auto_respawn = True
    settle = JoinHarness.settle

    def __init__(self, args, packs, order):
        super().__init__(args)
        self.packs, self.order = packs, order

    def prepare(self):
        super().prepare()
        source = Path(self.args.pack).parent
        for pack in ("detectb", "tick_order"):
            shutil.copytree(source / pack, self.server / "world/datapacks" / pack)

    def counts(self, phase, expected):
        for player in self.players:
            for event in EVENTS:
                count = expected[player][event]
                for namespace in NAMESPACES:
                    self.eventually(f"{phase}_{player}_{namespace}_{event}",
                                    f"if score #{namespace}:{player} audit_{event} matches {count}")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.send('datapack disable "file/tick_order"')
        self.send('datapack enable "file/tick_order" last')
        self.send("scoreboard objectives add join_check dummy")
        expected = {player: dict.fromkeys(EVENTS, 0) for player in self.players}
        for event in EVENTS:
            self.send(f"scoreboard objectives add audit_{event} dummy")
            for player in self.players:
                for namespace in NAMESPACES:
                    self.send(f"scoreboard players set #{namespace}:{player} audit_{event} 0")
        with self.clients():
            self.send("gamemode survival @a")
            self.send("gamerule keep_inventory true")
            self.send("experience set @a 0 levels")
            self.settle()
            self.counts("BASELINE", expected)
            for player in self.players:
                self.send(f"experience set {player} 5 levels")
                expected[player]["level"] += 1
                self.counts(f"XP_{player}", expected)
                self.send(f"item replace entity {player} armor.head with minecraft:leather_helmet")
                expected[player]["equip"] += 1
                self.counts(f"EQUIP_{player}", expected)
                self.send(f"item replace entity {player} armor.head with minecraft:air")
                expected[player]["unequip"] += 1
                self.counts(f"UNEQUIP_{player}", expected)
                self.send(f"kill {player}")
                expected[player]["death"] += 1
                expected[player]["respawn"] += 1
                self.counts(f"DEATH_RESPAWN_{player}", expected)
                self.eventually(f"ALIVE_{player}", f"if entity @a[name={player},nbt={{Health:20.0f}}]")
            self.settle()
            self.counts("STABLE", expected)
            legacy = {
                "__sand_dc": "deathCount", "__sand_rp": "dummy",
                "__sand_tsd": "minecraft.custom:minecraft.time_since_death",
                "__sand_xp_lvl": "dummy", "__sand_xp_prev": "dummy",
                "__sand_xp_delta": "dummy", "__sand_xp_seen": "dummy",
            }
            for objective, criterion in legacy.items():
                self.send(f"scoreboard objectives add {objective} {criterion}")
                self.send(f"scoreboard players set #legacy {objective} 37")
                self.send(f"scoreboard players set SandAlpha {objective} 37")
            self.send("tag SandAlpha add __sand_just_died")
            # The old allocator hashed the watch tuple without an export namespace.
            old_hash = 2166136261
            for byte in b"103|s24:minecraft:leather_helmet|n":
                old_hash = ((old_hash ^ byte) * 16777619) & 0xffffffff
            legacy_tags = [f"__armor_head_{old_hash:08x}_{phase}" for phase in ("now", "had")]
            for tag in legacy_tags:
                self.send(f"tag SandAlpha add {tag}")
            self.settle()
            for objective in legacy:
                self.check(f"LEGACY_{objective}", f"if score #legacy {objective} matches 37")
                expected_value = "37.." if objective == "__sand_tsd" else "37"
                self.check(f"LEGACY_PLAYER_{objective}", f"if score SandAlpha {objective} matches {expected_value}")
            for tag in legacy_tags:
                self.check(f"LEGACY_{tag}", f"if entity @a[name=SandAlpha,tag={tag}]")
            self.check("LEGACY_DEATH_TAG", "if entity @a[name=SandAlpha,tag=__sand_just_died]")
            self.counts("LEGACY_UNTOUCHED", expected)
        print(f"PASSED: tick order {self.order}; both players and packs receive exactly one of each event", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    packs = export_packs()
    output = args.output
    for order in (NAMESPACES, tuple(reversed(NAMESPACES))):
        with tempfile.TemporaryDirectory(prefix="sand-detector-packs-") as directory:
            root = Path(directory)
            for namespace, records in packs.items():
                write_pack(root / namespace, records)
            # Keep every exported command unchanged. Only control vanilla's tag order.
            values = []
            for namespace in order:
                for record in packs[namespace]:
                    if record["namespace"] == "minecraft" and record["dir"] == "tags/function" and record["path"] == "tick":
                        values.extend(json.loads(record["content"])["values"])
            if not values:
                raise ValidationFailure("tick-order", "exported tick tag is missing")
            write_pack(root / "tick_order", [{"namespace": "minecraft", "dir": "tags/function",
                       "path": "tick", "ext": "json", "content": json.dumps({"replace": True, "values": values})}])
            args.pack = str(root / "detecta")
            args.output = str(Path(output) / "-".join(order)) if output else None
            harness = DetectorHarness(args, packs, order)
            try:
                harness.run()
            except ValidationFailure as error:
                print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
                return 1
            print(f"PASSED: Minecraft 26.2 built-in ownership; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
