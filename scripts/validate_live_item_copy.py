#!/usr/bin/env python3
"""Maintained live-slot copy proof on Minecraft 26.2; run only after root release.

Uses PlayerHarness unchanged. Rust owns all gameplay; console commands set up
worlds/inventories and compare exact resulting NBT. Requires an isolated absolute
Cargo target and builds with two jobs. No move or full #382 completion is claimed.

Proof limits: source-read ordering and differently addressed aliases are delegated
to the exported native instruction, not independently distinguished here. Self-copy
no-elision is an export assertion; replay success is native-only evidence. Failure
callbacks replay native commands separately from fixture unchanged-state checks.
Forked execute chains suppress textual errors in 26.2; preseeded success/result
scores must both be overwritten with zero, with explicit failure preconditions. Invalid destination containers are not fully inventory-snapshotted.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure, RELOAD_START, RELOAD_COMPLETE

ROOT = Path(__file__).resolve().parents[1]
NS = "live_copy"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def export_fixture(target):
    if not Path(target).is_absolute():
        raise ValueError("--target-dir must be an isolated absolute path")
    with tempfile.TemporaryDirectory(prefix="sand-live-copy-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run([
            "cargo", "test", "-j", "2", "--target-dir", target, "-p", "sand",
            "--test", "live_item_copy", "--", "--ignored", "--exact", "dump_live_item_copy_pack",
        ], cwd=ROOT, env=dict(os.environ, SAND_LIVE_ITEM_COPY_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


class CopyHarness(PlayerHarness):
    def prepare(self):
        super().prepare()
        self.checks = []
        self.persist("running")

    def persist(self, status):
        (self.output / "evidence.json").write_text(json.dumps({
            "status": status, "minecraft": "26.2", **self.provenance,
            "server_exit": self.process.poll() if self.process else None,
            "checks": self.checks,
        }, indent=2))

    def check(self, name, condition):
        name = f"{self.phase}_{name}"
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition})
        self.persist("running")

    def call(self, path, frame=""):
        prefix = f"execute {frame} run " if frame else ""
        self.send(f"{prefix}function {NS}:{path}")

    def equip(self, player, item, count=1, slot="weapon.mainhand"):
        self.send(f"item replace entity {player} {slot} with {item} {count}")

    def capture(self, key, source):
        self.send(f"data modify storage live_copy:oracle {key} set from {source}")

    def equal(self, label, expected, actual):
        # data-modify success is zero only when the complete NBT values are equal.
        # Assert both paths exist first, so absent data cannot masquerade as equality.
        self.check(label + "_EXPECTED", f"if data storage live_copy:oracle {expected}")
        self.send("data remove storage live_copy:oracle actual")
        self.capture("actual", actual)
        self.check(label + "_ACTUAL", "if data storage live_copy:oracle actual")
        self.send(f"data modify storage live_copy:oracle compare set from storage live_copy:oracle {expected}")
        self.send("execute store success score #changed copy_oracle run data modify storage live_copy:oracle compare set from storage live_copy:oracle actual")
        self.check(label, "if score #changed copy_oracle matches 0")

    def expected_failure(self, path, frame, player, precondition):
        command = self.native[path]
        if "\n" in command or not command.startswith("item replace "):
            raise ValidationFailure("copy-oracle", f"expected one native copy: {path}")
        self.check(path + "_PRECONDITION", precondition)
        source = f"entity {player} SelectedItem"
        destination = f"entity {player} equipment.offhand"
        for key, address in (("failure_source", source), ("failure_destination", destination)):
            self.send(f"data remove storage live_copy:oracle {key}")
            self.capture(key, address)
            self.check(path + "_" + key.upper(), f"if data storage live_copy:oracle {key}")
        self.call(path, frame)
        self.equal(path + "_FIXTURE_NO_WRITE", "failure_destination", destination)
        self.equal(path + "_FIXTURE_NO_MOVE", "failure_source", source)

        # In 26.2 execute-as forks suppress sendFailure text. Numeric callbacks
        # still report (false, 0). Distinct sentinels reject a skipped callback;
        # zero is required for BOTH fields, so silence never implies failure.
        self.send("scoreboard players set #failure_success copy_oracle -71")
        self.send("scoreboard players set #failure_result copy_oracle -93")
        self.send(f"execute {frame} store success score #failure_success copy_oracle "
                  f"store result score #failure_result copy_oracle run {command}")
        self.check(path + "_NATIVE_ONLY_FAILURE_SUCCESS", "if score #failure_success copy_oracle matches 0")
        self.check(path + "_NATIVE_ONLY_FAILURE_RESULT", "if score #failure_result copy_oracle matches 0")
        self.equal(path + "_NATIVE_ONLY_NO_WRITE", "failure_destination", destination)
        self.equal(path + "_NATIVE_ONLY_NO_MOVE", "failure_source", source)

    def suite(self):
        a, b = self.players
        zombie = "@e[tag=copy_zombie,limit=1]"
        marker = "@e[tag=copy_marker,limit=1]"
        self.send("kill @e[tag=copy_zombie]")
        self.send("kill @e[tag=copy_marker]")
        self.send('summon minecraft:zombie 4 85 4 {Tags:["copy_zombie"],NoAI:1b,PersistenceRequired:1b}')
        self.send('summon minecraft:marker 4 85 5 {Tags:["copy_marker","copy_destination"]}')
        for i, player in enumerate(self.players):
            self.send(f"gamemode creative {player}")
            self.send(f"tp {player} {8 + i * 2} 85 8 0 0")
            self.send(f"clear {player}")
            self.equip(player, f'minecraft:diamond[minecraft:custom_data={{owner:{i},nested:{{value:7}}}},minecraft:custom_name="Copy {i}"]', 3 + i * 4)
            self.capture(f"source{i}", f"entity {player} SelectedItem")
            self.equip(player, "minecraft:stick", slot="weapon.offhand")
        self.call("copy", f"as {a}")
        self.equal("COPY_A", "source0", f"entity {a} equipment.offhand")
        self.equal("SOURCE_A", "source0", f"entity {a} SelectedItem")
        self.check("OTHER_UNTOUCHED", f"if items entity {b} weapon.offhand minecraft:stick")
        self.call("self_copy", f"as {a}")
        self.equal("SELF_FIXTURE_CONTENTS", "source0", f"entity {a} SelectedItem")
        # Native-only setter success; no-elision is proved by the Rust export test.
        self.send("scoreboard players set #self copy_oracle -71")
        self.send("scoreboard players set #self_result copy_oracle -93")
        self.send(f"execute as {a} store success score #self copy_oracle "
                  f"store result score #self_result copy_oracle run {self.native['self_copy']}")
        self.check("SELF_NATIVE_ONLY_SUCCESS", "if score #self copy_oracle matches 1")
        self.check("SELF_NATIVE_ONLY_RESULT", "if score #self_result copy_oracle matches 1")
        self.equal("SELF_NATIVE_ONLY_CONTENTS", "source0", f"entity {a} SelectedItem")
        for player, enabled in ((a, 0), (b, 1)):
            self.equip(player, "minecraft:stick", slot="weapon.offhand")
            self.send(f"scoreboard players set {player} {self.objectives['enabled']} {enabled}")
        self.call("query_players")
        self.check("GUARD_FALSE", f"if items entity {a} weapon.offhand minecraft:stick")
        self.equal("GUARD_TRUE", "source1", f"entity {b} equipment.offhand")
        for player, count in ((a, 3), (b, 14)):
            self.check(f"COUNT_{player}", f"if score {player} {self.objectives['diamonds']} matches {count}")
        # Capture owners in both directions while executing as the other player.
        for owner, other, oi, ti in ((a, b, 0, 1), (b, a, 1, 0)):
            self.send("tag @a remove copy_other")
            self.send(f"tag {other} add copy_other")
            for destination in (owner, other):
                self.equip(destination, "minecraft:stick", slot="weapon.offhand")
                self.check(f"SCOPED_START_{owner}_{destination}",
                           f"if items entity {destination} weapon.offhand minecraft:stick")
            self.call("scoped", f"as {owner}")
            self.equal(f"SCOPED_DEST_{owner}", f"source{ti}", f"entity {owner} equipment.offhand")
            self.equal(f"SCOPED_SOURCE_{owner}", f"source{oi}", f"entity {other} equipment.offhand")
            self.equal(f"SCOPED_OWNER_UNCHANGED_{owner}", f"source{oi}", f"entity {owner} SelectedItem")
            self.equal(f"SCOPED_OTHER_UNCHANGED_{owner}", f"source{ti}", f"entity {other} SelectedItem")
        self.equip(zombie, 'minecraft:diamond[minecraft:custom_data={zombie:1b}]', 5)
        self.capture("zombie", f"entity {zombie} equipment.mainhand")
        self.call("zombie", f"as {zombie}")
        self.equal("ZOMBIE", "zombie", f"entity {zombie} equipment.offhand")
        self.equal("ZOMBIE_SOURCE", "zombie", f"entity {zombie} equipment.mainhand")
        for x in (0, 1):
            self.send(f"setblock {x} 85 0 minecraft:air")
            self.send(f"setblock {x} 85 0 minecraft:chest")
        self.call("to_block", f"as {a} positioned 0 85 0")
        # Container entries carry Slot; normalize only that address metadata.
        self.send("data remove storage live_copy:oracle block")
        self.capture("block", "block 0 85 0 Items[{Slot:0b}]")
        self.send("data remove storage live_copy:oracle block.Slot")
        self.equal("ENTITY_BLOCK", "source0", "storage live_copy:oracle block")
        self.call("block_to_block", "positioned 0 85 0")
        self.send("data remove storage live_copy:oracle block")
        self.capture("block", "block 1 85 0 Items[{Slot:0b}]")
        self.send("data remove storage live_copy:oracle block.Slot")
        self.equal("BLOCK_BLOCK", "source0", "storage live_copy:oracle block")
        self.equip(b, "minecraft:stick", slot="weapon.offhand")
        self.check("BLOCK_ENTITY_START", f"if items entity {b} weapon.offhand minecraft:stick")
        self.call("from_block", f"as {b} positioned 1 85 0")
        self.equal("BLOCK_ENTITY", "source0", f"entity {b} equipment.offhand")
        self.send("data remove storage live_copy:oracle block")
        self.capture("block", "block 1 85 0 Items[{Slot:0b}]")
        self.send("data remove storage live_copy:oracle block.Slot")
        self.equal("BLOCK_ENTITY_SOURCE_UNCHANGED", "source0", "storage live_copy:oracle block")
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            for y in (85, 86):
                self.send(f"execute in {dimension} run setblock 1 {y} 0 minecraft:air")
                self.send(f"execute in {dimension} run setblock 1 {y} 0 minecraft:chest")
        self.call("frame", f"as {a} in minecraft:the_nether positioned 0.5 85 0.5 rotated -90 0 anchored eyes")
        self.send("data remove storage live_copy:oracle block")
        self.send("execute in minecraft:the_nether run data modify storage live_copy:oracle block set from block 1 86 0 Items[{Slot:0b}]")
        self.send("data remove storage live_copy:oracle block.Slot")
        self.equal("FRAME", "source0", "storage live_copy:oracle block")
        self.check("NO_FEET_REANCHOR", "in minecraft:the_nether unless data block 1 85 0 Items[0]")
        self.check("NO_DIMENSION_CHANGE", "in minecraft:overworld unless data block 1 86 0 Items[0]")
        self.send(f"tag {b} add copy_destination")
        self.equip(b, "minecraft:stick")
        self.check("PLURAL_FIXTURE_START", f"if items entity {b} weapon.mainhand minecraft:stick")
        self.call("plural", f"as {a}")
        # Observe the fixture before any replay can repair an ineffective call.
        self.equal("PLURAL_FIXTURE_ACCEPTED", "source0", f"entity {b} SelectedItem")
        self.check("PLURAL_FIXTURE_REJECTED", f"unless data entity {marker} Inventory")
        self.equal("PLURAL_FIXTURE_SOURCE", "source0", f"entity {a} SelectedItem")
        # Separate native-only acceptance-count experiment, with a fresh start.
        self.equip(b, "minecraft:stick")
        self.check("PLURAL_NATIVE_ONLY_START", f"if items entity {b} weapon.mainhand minecraft:stick")
        self.send(f"execute store result score #accepted copy_oracle as {a} run {self.native['plural']}")
        self.check("PLURAL_NATIVE_ONLY_ONE_ACCEPTED", "if score #accepted copy_oracle matches 1")
        self.equal("PLURAL_NATIVE_ONLY_ACCEPTED", "source0", f"entity {b} SelectedItem")
        self.check("PLURAL_NATIVE_ONLY_REJECTED", f"unless data entity {marker} Inventory")
        self.equal("PLURAL_NATIVE_ONLY_SOURCE", "source0", f"entity {a} SelectedItem")
        for path, frame, precondition in (
            ("missing_source", f"as {a}", "unless entity @e[tag=copy_absent]"),
            ("incompatible_source", f"as {a}", "if entity @e[type=minecraft:marker,tag=copy_marker,limit=1]"),
            ("incompatible_destination", f"as {a}", "if entity @e[type=minecraft:marker,tag=copy_marker,limit=1]"),
            ("invalid_block_source", f"as {a} positioned 0 85 0", "if block 0 85 0 minecraft:chest"),
            ("invalid_block_destination", f"as {a} positioned 0 85 0", "if block 0 85 0 minecraft:chest"),
            ("from_block", f"as {a} positioned 6 85 6", "if block 6 85 6 minecraft:air"),
            ("to_block", f"as {a} positioned 6 85 6", "if block 6 85 6 minecraft:air"),
        ):
            self.send("setblock 6 85 6 minecraft:air")
            self.equip(a, "minecraft:stick", slot="weapon.offhand")
            self.expected_failure(path, frame, a, precondition)
            self.check(path + "_NO_WRITE", f"if items entity {a} weapon.offhand minecraft:stick")
            self.equal(path + "_NO_MOVE", "source0", f"entity {a} SelectedItem")
        self.equip(a, "minecraft:air")
        self.call("copy", f"as {a}")
        self.check("EMPTY_CLEARS", f"unless items entity {a} weapon.offhand *")

    def run_client(self):
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            self.load_test_chunk(dimension)
        self.send("scoreboard objectives add copy_oracle dummy")
        self.phase = "initial"
        with self.clients():
            for player in self.players:
                self.eventually(f"JOIN_{player}", f"if score {player} {self.objectives['joined']} matches 1..")
                self.check(f"EVENT_{player}", f"if score {player} {self.objectives['joined']} matches 1..")
            self.send("tick freeze")
            try:
                self.suite()
                # Persist distinct stacks for the real event-driven copy on rejoin.
                for i, player in enumerate(self.players):
                    self.equip(player, f'minecraft:diamond[minecraft:custom_data={{rejoin:{i}}}]', i + 2)
                    self.capture(f"rejoin{i}", f"entity {player} SelectedItem")
                    self.equip(player, "minecraft:stick", slot="weapon.offhand")
            finally:
                if self.process and self.process.poll() is None:
                    self.send("tick unfreeze")
        offset = len(self.lines)
        self.send("reload")
        self.send(f"say {RELOAD_COMPLETE}")
        start = self.wait_for(RELOAD_START, "copy-reload-start", offset)
        self.wait_for(RELOAD_COMPLETE, "copy-reload-complete", start + 1)
        self.phase = "reloaded"
        with self.clients():
            for i, player in enumerate(self.players):
                self.eventually(f"REJOIN_{player}", f"if score {player} {self.objectives['joined']} matches 2..")
                self.equal(f"EVENT_COPY_{player}", f"rejoin{i}", f"entity {player} equipment.offhand")
                self.equal(f"EVENT_SOURCE_{player}", f"rejoin{i}", f"entity {player} SelectedItem")
            self.send("tick freeze")
            try:
                self.suite()
            finally:
                if self.process and self.process.poll() is None:
                    self.send("tick unfreeze")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--target-dir", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    fixture = export_fixture(args.target_dir)
    with tempfile.TemporaryDirectory(prefix="sand-live-copy-pack-") as directory:
        pack = Path(directory) / NS
        pack.mkdir()
        (pack / "pack.mcmeta").write_text(json.dumps({"pack": {
            "pack_format": 107, "min_format": 107, "max_format": 107,
            "description": "Sand live-slot copy regression",
        }}))
        for record in fixture["components"]:
            path = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(record["content"])
        args.pack = str(pack)
        harness = CopyHarness(args)
        harness.objectives = fixture["objectives"]
        harness.native = {r["path"]: r["content"].strip() for r in fixture["components"]
                          if r["dir"] == "function" and r["namespace"] == NS}
        harness.provenance = {
            "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "worktree_status": subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True),
            "fixture_sha256": digest(ROOT / "sand/tests/live_item_copy.rs"),
            "harness_sha256": digest(__file__), "server_jar_sha256": digest(args.jar),
            "shared_harness_sha256": digest(ROOT / "scripts/player_validation_harness.py"),
            "client_sha256": digest(ROOT / "scripts/mc_validation/minimal_join_client.py"),
            "pack_files": {p.relative_to(pack).as_posix(): digest(p) for p in sorted(pack.rglob("*")) if p.is_file()},
        }
        try:
            harness.run()
        except ValidationFailure as error:
            harness.persist("failed")
            print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
            return 1
        harness.persist("passed")
        print(f"PASSED: {len(harness.checks)} live-copy assertions; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
