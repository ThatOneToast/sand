#!/usr/bin/env python3
"""Exercise generated bounded item transport with two real passive players.

The unmodified Rust export fixture supplies source/child/expiry functions.
Real held items feed the source capture; controlled direct dispatch isolates
transport from event detection. This does not prove combat input or automatic
window expiry. Normal tick coordination is frozen during these storage checks.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure, RELOAD_START

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_FORMAT = 107
NS = "boundeditempack"

def export_records() -> list[dict]:
    """Run the exporter in its own process and return its records.

    Deliberately reuses the exact `#[ignore]`d dump helper the determinism
    check uses, so this harness validates the same bytes the test suite
    asserts on rather than a separately-derived pack.
    """
    dump = Path(tempfile.mkdtemp(prefix="sand-bounded-export-")) / "export.json"
    env = dict(os.environ, SAND_DETERMINISM_DUMP_PATH=str(dump))
    subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "sand-core",
            "--test",
            "event_chain_bounded_item_transport",
            "--",
            "--ignored",
            "--exact",
            "dump_export_to_file_for_external_hash_comparison",
        ],
        cwd=REPO_ROOT,
        env=env,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(dump.read_text())


def write_pack(datapacks: Path, records: list[dict]) -> None:
    pack = datapacks / "bevt"
    (pack / "data").mkdir(parents=True)
    (pack / "pack.mcmeta").write_text(
        json.dumps(
            {
                "pack": {
                    "pack_format": PACK_FORMAT,
                    "min_format": PACK_FORMAT,
                    "max_format": PACK_FORMAT,
                    "description": "sand #272 bounded item event-path proof",
                }
            }
        ),
        encoding="utf-8",
    )
    for record in records:
        target = (
            pack
            / "data"
            / record["namespace"]
            / record["dir"]
            / f"{record['path']}.{record['ext']}"
        )
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(record["content"], encoding="utf-8")


def find(records: list[dict], prefix: str) -> str:
    matches = [r["path"] for r in records if r["path"].startswith(prefix)]
    if len(matches) != 1:
        raise SystemExit(f"expected exactly one function under {prefix!r}, got {matches!r}")
    return matches[0]


class BoundedItemHarness(PlayerHarness):
    def __init__(self, args, records):
        super().__init__(args)
        functions = [r for r in records if r["dir"] == "function"]
        self.entry = find(functions, "__sand_event_bounded_item_load/").split("/", 1)[1]
        self.expire = find(functions, "__sand_event_bounded_item_expire/")
        self.source = next(r["path"] for r in functions
                           if r["path"].startswith("__sand_event_dispatch/")
                           and "__sand_event_bounded_item_persist/" in r["content"])
        self.children = sorted(r["path"] for r in functions
                               if r["path"].startswith("__sand_event_dispatch/")
                               and "__sand_event_bounded_item_load/" in r["content"])
        if len(self.children) != 2:
            raise ValidationFailure("fixture", "expected two independent bounded consumers")

    def capture(self, player, item):
        self.send(f"item replace entity {player} weapon.mainhand with {item}")
        if item != "minecraft:air":
            self.check(f"{player}_EQUIPPED", f'if data entity {player} SelectedItem{{id:"{item}"}}')
            self.send(f"data modify storage bevth:oracle {player} set from entity {player} SelectedItem")
        else:
            self.check(f"{player}_EMPTY", f"unless data entity {player} SelectedItem")
        self.send(f"execute as {player} at @s run function {NS}:{self.source}")

    def read(self, player, phase, present=True, child=0):
        self.send(f"execute as {player} at @s run function {NS}:{self.children[child]}")
        path = f"storage sand:__bounded_item cur.{self.entry}"
        self.check(f"{phase}_PRESENCE", f"if data {path}{{present:{1 if present else 0}b}}")
        if present:
            self.check(f"{phase}_ITEM", f"if data {path}.item.id")
            self.check(f"{phase}_ORACLE", f"if data storage bevth:oracle {player}.id")
            self.send(f"data modify storage bevth:oracle comparison set from {path}.item")
            self.send(f"execute store success score #different bevth_check run data modify storage bevth:oracle comparison set from storage bevth:oracle {player}")
            self.check(f"{phase}_EQUAL", "if score #different bevth_check matches 0")
        else:
            self.check(f"{phase}_NO_ITEM", f"unless data {path}.item.id")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.send("tick freeze")
        try:
            with self.clients():
                self.send("scoreboard objectives add bevth_check dummy")
                for player in self.players:
                    self.send(f"gamemode creative {player}")
                    self.send(f"tp {player} 0 80 0")
                    self.send(f"scoreboard players set {player} p272_source 0")
                self.capture("SandAlpha", "minecraft:diamond_sword")
                self.read("SandBeta", "BETA_NEVER_CAPTURED", present=False)
                self.capture("SandBeta", "minecraft:golden_apple")
                for player in self.players:
                    self.check(f"{player}_SLOT", f"if score {player} sand_subj matches 1..")
                self.check("DISTINCT_SLOTS", "unless score SandAlpha sand_subj = SandBeta sand_subj")
                self.read("SandAlpha", "ALPHA")
                self.read("SandBeta", "BETA")
                self.read("SandAlpha", "ALPHA_AGAIN")
                self.read("SandAlpha", "SECOND_CONSUMER", child=1)
                self.send("item replace entity SandAlpha weapon.mainhand with minecraft:dirt")
                self.check("ALPHA_LIVE_CHANGED", 'if data entity SandAlpha SelectedItem{id:"minecraft:dirt"}')
                self.read("SandAlpha", "SNAPSHOT_FROZEN")
                self.capture("SandAlpha", "minecraft:stone")
                self.read("SandAlpha", "REPLACED")
                self.read("SandBeta", "BETA_UNCHANGED")
                self.capture("SandAlpha", "minecraft:air")
                self.read("SandAlpha", "ABSENT", present=False)
                self.read("SandBeta", "BETA_AFTER_ABSENT")
                self.capture("SandAlpha", "minecraft:diamond_sword")
                start = len(self.lines)
                self.send("reload")
                self.wait_for(RELOAD_START, "bounded-reload", start)
                self.send("say __SAND_BOUNDED_RELOADED__")
                self.wait_for(self.marker_pattern("__SAND_BOUNDED_RELOADED__"), "bounded-reload-complete", start)
                self.read("SandAlpha", "ALPHA_RELOAD")
                self.read("SandBeta", "BETA_RELOAD")
                self.send(f"execute as SandAlpha at @s run function {NS}:{self.expire}")
                self.read("SandAlpha", "EXPIRED", present=False)
                self.read("SandBeta", "BETA_AFTER_EXPIRY")
                print("PASSED: real held-item capture, per-player bounded transport, replacement, absence, reload and isolated expiry", flush=True)
        finally:
            self.send("tick unfreeze")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version = "26.2"
    args.op_player = None
    args.client_command = None
    records = export_records()
    with tempfile.TemporaryDirectory(prefix="sand-bounded-pack-") as directory:
        write_pack(Path(directory), records)
        args.pack = str(Path(directory) / "bevt")
        harness = BoundedItemHarness(args, records)
        try:
            harness.run()
        except ValidationFailure as error:
            print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
            return 1
        print(f"PASSED: Minecraft 26.2 bounded item event path; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
