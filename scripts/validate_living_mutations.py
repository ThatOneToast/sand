#!/usr/bin/env python3
"""Run the maintained living-mutation fixture with two real Minecraft 26.2 players."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def export_fixture():
    with tempfile.TemporaryDirectory(prefix="sand-living-mutations-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run([
            "cargo", "test", "-p", "sand", "--test", "living_mutations", "--",
            "--ignored", "--exact", "dump_living_mutations_pack",
        ], cwd=ROOT, env=dict(os.environ, SAND_LIVING_MUTATIONS_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


class LivingHarness(PlayerHarness):
    def prepare(self):
        super().prepare()
        with (self.server / "server.properties").open("a", encoding="utf-8") as properties:
            properties.write("server-ip=127.0.0.1\n")
        self.checks = []
        self.persist("running")

    def persist(self, status):
        (self.output / "evidence.json").write_text(json.dumps({
            "status": status, "minecraft": "26.2", **self.provenance,
            "server_ip": "127.0.0.1", "server_port": self.port,
            "checks": self.checks,
        }, indent=2))

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition})
        self.persist("running")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.load_test_chunk("minecraft:the_nether")
        self.send("gamerule minecraft:natural_health_regeneration false")
        self.send("fill -2 79 -2 5 79 2 minecraft:stone")
        self.send("fill -2 80 -2 5 83 2 minecraft:air")
        with self.clients():
            a, b = self.players
            for index, player in enumerate(self.players):
                self.send(f"tp {player} {index * 3 + 0.5} 80 0.5")
            # Vanilla gives freshly joined players a short damage immunity window.
            time.sleep(4)
            for player in self.players:
                self.eventually(f"JOIN_{player}", f'if entity @a[name={player},nbt={{active_effects:[{{id:"minecraft:speed"}}]}}]')
                self.send(f"gamemode survival {player}")
                self.send(f"effect give {player} minecraft:instant_health 1 10 true")
                self.send(f"scoreboard players set {player} {self.enabled} 1")
                self.send(f"scoreboard players set {player} {self.applications} 0")
            self.send("tick freeze")
            try:
                # Repeat the same action after a real reload while players stay connected.
                for phase in ("before_reload", "after_reload"):
                    if phase == "after_reload":
                        self.send("reload")
                        marker = "SAND_LIVING_RELOAD_COMPLETE"
                        offset = len(self.lines)
                        self.send(f"say {marker}")
                        self.wait_for(marker, "living-reload", offset)
                    for player in self.players:
                        self.send(f"effect clear {player}")
                        self.send(f"effect give {player} minecraft:speed 60")
                        self.send(f"scoreboard players set {player} {self.applications} 0")
                    self.send(f"scoreboard players set {a} {self.enabled} 0")
                    self.send(f"execute as {a} run function living:apply")
                    self.check(f"{phase}_STATE_GATE", f"if score {a} {self.applications} matches 0")
                    self.check(f"{phase}_STATE_GATE_EFFECT", f'if entity @a[name={a},nbt={{active_effects:[{{id:"minecraft:speed"}}]}}]')
                    self.send(f"scoreboard players set {a} {self.enabled} 1")
                    self.send(f"execute as {a} run function living:apply")
                    self.check(f"{phase}_APPLY_A", f"if score {a} {self.applications} matches 1")
                    self.check(f"{phase}_OTHER_STATE_UNTOUCHED", f"if score {b} {self.applications} matches 0")
                    self.check(f"{phase}_GIVE_A", f'if entity @a[name={a},nbt={{active_effects:[{{id:"minecraft:glowing"}}]}}]')
                    self.check(f"{phase}_REMOVE_A", f'unless entity @a[name={a},nbt={{active_effects:[{{id:"minecraft:speed"}}]}}]')
                    self.check(f"{phase}_OTHER_EFFECT_UNTOUCHED", f'if entity @a[name={b},nbt={{active_effects:[{{id:"minecraft:speed"}}]}}]')
                    if phase == "before_reload":
                        self.check("DAMAGE_A", f"if entity @a[name={a},nbt={{Health:18.0f}}]")
                        self.check("OTHER_HEALTH_UNTOUCHED", f"if entity @a[name={b},nbt={{Health:20.0f}}]")
                    self.send("function living:query_players")
                    self.check(f"{phase}_CONDITION_PREVENTS_REAPPLICATION", f"if score {a} {self.applications} matches 1")
                    self.check(f"{phase}_QUERY_APPLIES_B", f"if score {b} {self.applications} matches 1")
                    self.send(f"execute as {a} run function living:clear")
                    self.check(f"{phase}_CLEAR_A", f"unless data entity {a} active_effects[0]")
                    self.check(f"{phase}_CLEAR_ISOLATION", f'if entity @a[name={b},nbt={{active_effects:[{{id:"minecraft:glowing"}}]}}]')
                self.send('summon minecraft:zombie 0 85 0 {Tags:["living_zombie"],NoAI:1b,PersistenceRequired:1b}')
                self.send("execute as @e[tag=living_zombie,limit=1] run function living:nonplayer")
                self.check("NONPLAYER_DAMAGE", "if entity @e[tag=living_zombie,nbt={Health:18.0f}]")
                self.check("NONPLAYER_EFFECT", 'if entity @e[tag=living_zombie,nbt={active_effects:[{id:"minecraft:glowing"}]}]')
                self.send(f"tag {b} add living_other")
                self.send(f"execute as {a} run function living:scoped")
                self.check("SCOPED_ORIGINAL_TARGET", f'if entity @a[name={a},nbt={{active_effects:[{{id:"minecraft:glowing"}}]}}]')
                self.check("SCOPED_CURRENT_EXECUTOR", f"unless data entity {b} active_effects[0]")
                # A stronger hit during the frozen hurt window applies only its
                # excess over the earlier two-point hit, to the bound player.
                self.check("SCOPED_DAMAGE_OWNER", f"if entity @a[name={a},nbt={{Health:17.0f}}]")
                self.check("SCOPED_DAMAGE_OTHER_UNTOUCHED", f"if entity @a[name={b},nbt={{Health:18.0f}}]")
                for dimension in ("minecraft:overworld", "minecraft:the_nether"):
                    self.send(f"execute in {dimension} run fill 0 84 0 6 89 6 minecraft:air")
                self.send(f"tp {a} 20 85 20 0 0")
                self.send(f"execute as {a} in minecraft:the_nether positioned 0.5 85 0.5 rotated -90 0 anchored eyes run function living:frame")
                self.check("FRAME_POSITION_ROTATION_DIMENSION_ANCHOR", "in minecraft:the_nether if block 1 86 0 minecraft:gold_block")
                self.check("ORIGINAL_DIMENSION", "in minecraft:overworld if block 1 86 0 minecraft:air")
                self.check("NO_FEET_REANCHOR", "in minecraft:the_nether if block 1 85 0 minecraft:air")
            finally:
                if self.process and self.process.poll() is None:
                    self.send("tick unfreeze")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    fixture = export_fixture()
    records = fixture["components"]
    with tempfile.TemporaryDirectory(prefix="sand-living-mutations-pack-") as directory:
        pack = Path(directory) / "living"
        pack.mkdir()
        (pack / "pack.mcmeta").write_text(json.dumps({"pack": {
            "pack_format": 107, "min_format": 107, "max_format": 107,
            "description": "Sand living mutation regression",
        }}))
        for record in records:
            target = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(record["content"])
        args.pack = str(pack)
        harness = LivingHarness(args)
        harness.enabled = fixture["objectives"]["enabled"]
        harness.applications = fixture["objectives"]["applications"]
        harness.provenance = {
            "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "worktree_status": subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True),
            "fixture_sha256": digest(ROOT / "sand/tests/living_mutations.rs"),
            "harness_sha256": digest(__file__), "server_jar_sha256": digest(args.jar),
            "shared_harness_sha256": {name: digest(ROOT / "scripts" / name) for name in (
                "player_validation_harness.py", "validate_world_gameplay.py",
                "vanilla_reload_harness.py", "mc_validation/minimal_join_client.py",
            )},
            "pack_files": {p.relative_to(pack).as_posix(): digest(p) for p in sorted(pack.rglob("*")) if p.is_file()},
        }
        try:
            harness.run()
        except ValidationFailure as error:
            harness.persist("failed")
            print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
            return 1
        harness.persist("passed")
        print(f"PASSED: {len(harness.checks)} living mutation assertions; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
