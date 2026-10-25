#!/usr/bin/env python3
"""Maintained 26.2 named equipment proof using the existing two-player harness."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from validate_living_mutations import LivingHarness, digest
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]
SLOTS = ("weapon.mainhand", "weapon.offhand", "armor.head", "armor.chest", "armor.legs", "armor.feet")
PATHS = ("equipment.mainhand", "equipment.offhand", "equipment.head", "equipment.chest", "equipment.legs", "equipment.feet")
ITEMS = ("diamond_sword", "shield", "diamond_helmet", "diamond_chestplate", "diamond_leggings", "diamond_boots")


class EquipmentHarness(LivingHarness):
    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        with self.clients():
            a, b = self.players
            for player in self.players:
                self.send(f"gamemode creative {player}")
                self.send(f"tp {player} 0 85 0")
            for phase in ("before_reload", "after_reload"):
                if phase == "after_reload":
                    offset = len(self.lines)
                    self.send("reload")
                    self.send("say EQUIPMENT_RELOAD_COMPLETE")
                    self.wait_for("EQUIPMENT_RELOAD_COMPLETE", "equipment-reload", offset)
                for player in self.players:
                    for slot in SLOTS:
                        self.send(f"item replace entity {player} {slot} with minecraft:air")
                self.send(f"execute as {a} run function equipment:equip")
                self.check(f"{phase}_CONDITION_GATE", f"unless items entity {a} armor.feet minecraft:diamond_boots")
                self.send(f"item replace entity {a} weapon.offhand with minecraft:stick")
                self.send(f"execute as {a} run function equipment:equip")
                for slot, path, item in zip(SLOTS, PATHS, ITEMS):
                    key = slot.replace(".", "_")
                    self.check(f"{phase}_{key}_A", f"if items entity {a} {slot} minecraft:{item}")
                    self.check(f"{phase}_{key}_B_UNTOUCHED", f"unless items entity {b} {slot} *")
                    player_path = "SelectedItem" if slot == "weapon.mainhand" else path
                    self.check(f"{phase}_{key}_NBT", f'if data entity {a} {player_path}{{id:"minecraft:{item}"}}')
                self.send(f"tag {b} add equipment_other")
                self.send(f"item replace entity {b} armor.head with minecraft:iron_helmet")
                self.send(f"execute as {a} run function equipment:scoped")
                self.check(f"{phase}_SCOPED_OWNER", f"if items entity {a} armor.feet minecraft:golden_boots")
                self.check(f"{phase}_SCOPED_OTHER_BOOTS", f"unless items entity {b} armor.feet *")
                self.check(f"{phase}_CURRENT_OTHER_CLEARED", f"unless items entity {b} armor.head *")
                self.check(f"{phase}_OWNER_HELMET_UNCHANGED", f"if items entity {a} armor.head minecraft:diamond_helmet")
                self.send("kill @e[tag=equipment_zombie]")
                self.send('summon minecraft:zombie 0 85 0 {Tags:["equipment_zombie"],NoAI:1b,Invulnerable:1b,PersistenceRequired:1b}')
                self.send("execute as @e[tag=equipment_zombie,limit=1] run function equipment:nonplayer")
                for slot, path, item in zip(SLOTS, PATHS, ITEMS):
                    self.check(f"{phase}_ZOMBIE_{slot.replace('.', '_')}", f'if data entity @e[tag=equipment_zombie,limit=1] {path}{{id:"minecraft:{item}"}}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    with tempfile.TemporaryDirectory(prefix="sand-named-equipment-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run(["cargo", "test", "-p", "sand", "--test", "named_equipment", "--",
                        "--ignored", "--exact", "dump_named_equipment_pack"], cwd=ROOT,
                       env=dict(os.environ, SAND_EQUIPMENT_DUMP_PATH=str(dump)), check=True)
        fixture = json.loads(dump.read_text())
        pack = Path(directory) / "equipment"
        pack.mkdir()
        (pack / "pack.mcmeta").write_text(json.dumps({"pack": {"pack_format": 107,
            "min_format": 107, "max_format": 107, "description": "Sand named equipment regression"}}))
        for record in fixture["components"]:
            target = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(record["content"])
        args.pack = str(pack)
        harness = EquipmentHarness(args)
        harness.provenance = {
            "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "worktree_status": subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True),
            "fixture_sha256": digest(ROOT / "sand/tests/named_equipment.rs"),
            "harness_sha256": digest(__file__), "server_jar_sha256": digest(args.jar),
            "shared_harness_sha256": {name: digest(ROOT / "scripts" / name) for name in (
                "validate_living_mutations.py", "player_validation_harness.py", "validate_world_gameplay.py",
                "vanilla_reload_harness.py", "mc_validation/minimal_join_client.py")},
            "pack_files": {p.relative_to(pack).as_posix(): digest(p) for p in sorted(pack.rglob("*")) if p.is_file()},
        }
        try:
            harness.run()
        except ValidationFailure as error:
            harness.persist("failed")
            print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
            return 1
        harness.persist("passed")
        print(f"PASSED: {len(harness.checks)} named equipment assertions; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
