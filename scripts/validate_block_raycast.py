#!/usr/bin/env python3
"""Prove Sand block raycasts on vanilla 26.2 with two real players.

Exports the maintained Rust fixture; console commands only arrange world state,
invoke its public cast functions, and assert observable effects. No replacement
raycast algorithm is embedded in this harness.
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
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]
NS = "raycast"


def export_fixture():
    with tempfile.TemporaryDirectory(prefix="sand-raycast-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run([
            "cargo", "test", "-p", "sand", "--test", "block_raycast", "--",
            "--ignored", "--exact", "dump_raycast_pack",
        ], cwd=ROOT, env=dict(os.environ, SAND_RAYCAST_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


def write_pack(path, records):
    path.mkdir(parents=True)
    (path / "pack.mcmeta").write_text(json.dumps({"pack": {
        "pack_format": 107, "min_format": 107, "max_format": 107,
        "description": "Sand block raycast regression",
    }}))
    for record in records:
        target = path / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(record["content"])


def sha256(path):
    with Path(path).open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def pack_identity(path):
    files = {file.relative_to(path).as_posix(): sha256(file)
             for file in sorted(path.rglob("*")) if file.is_file()}
    encoded = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    return {"sha256": hashlib.sha256(encoded).hexdigest(), "files": files,
            "encoding": "SHA-256 of sorted compact JSON mapping relative POSIX paths to file SHA-256"}


class RaycastHarness(PlayerHarness):
    def prepare(self):
        super().prepare()
        self.checks = []
        self.persist("running")

    def persist(self, status):
        (self.output / "evidence.json").write_text(json.dumps({
            "schema_version": 1, "status": status, "minecraft": "26.2",
            "fixture": "sand/tests/block_raycast.rs", **self.provenance,
            "checks": self.checks,
        }, indent=2))

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition})
        self.persist("running")

    def reset(self):
        self.send("fill 0 84 0 7 88 9 minecraft:air")
        for objective in ("ray_hits", "ray_misses", "ray_other", "ray_player"):
            self.send(f"scoreboard players set @e {objective} 0")
        for index, player in enumerate(self.players):
            self.send(f"tp {player} {index * 4 + 0.5} 85 0.5 0 0")
            for tag in ("ray_nested", "ray_other_nested"):
                self.send(f"tag {player} remove {tag}")

    def cast(self, name="primary", target=None, origin="at @s"):
        target = target or self.players[0]
        self.send(f"execute as {target} {origin} run function {NS}:cast_{name}")

    def score(self, label, target, objective, value):
        self.check(label, f"if score {target} ray_{objective} matches {value}")

    def block(self, label, x, y, z, block, dimension="minecraft:overworld"):
        self.check(label, f"in {dimension} if block {x} {y} {z} minecraft:{block}")

    def cleared(self, label, target, definition):
        self.check(label, f"unless score {target} {self.counters[definition]} matches -2147483648..2147483647")

    def run_client(self):
        self.checks = []
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            self.load_test_chunk(dimension)
        with self.clients():
            a, b = self.players
            for player in self.players:
                self.send(f"gamemode creative {player}")
            self.send("tick freeze")
            try:
                # A nested cast uses the SAME definition/counter. Terminal return
                # prevents any outer traversal resuming after that callback.
                self.reset()
                for x in (0, 4):
                    self.send(f"setblock {x} 85 2 minecraft:gold_block")
                self.send(f"tag {a} add ray_nested")
                self.cast(target="@a")
                self.score("NESTED_HIT_ONCE", a, "hits", 1)
                self.score("NESTED_MISS_ONCE", a, "misses", 1)
                self.score("OTHER_PLAYER_HIT", b, "hits", 1)
                self.score("OTHER_PLAYER_NO_MISS", b, "misses", 0)
                self.block("OUTER_CONTEXT_RESTORED", 0, 85, 2, "emerald_block")
                self.block("NESTED_ENDPOINT", 0, 85, 5, "redstone_block")
                self.block("OTHER_PLAYER_POSITION", 4, 85, 2, "diamond_block")
                for player in self.players:
                    self.cleared(f"NESTED_CLEAN_{player}", player, "primary")

                self.reset()
                self.send("setblock 0 85 2 minecraft:gold_block")
                self.send("setblock 0 85 4 minecraft:iron_block")
                self.send(f"tag {a} add ray_other_nested")
                self.cast()
                self.score("TWO_DEFINITIONS_PRIMARY", a, "hits", 1)
                self.score("TWO_DEFINITIONS_SECONDARY", a, "other", 1)
                self.score("TWO_DEFINITIONS_NO_MISS", a, "misses", 0)
                self.block("TWO_DEFINITIONS_RESTORE", 0, 85, 2, "emerald_block")
                self.block("SECONDARY_POSITION", 0, 85, 4, "lapis_block")
                for definition in ("primary", "secondary"):
                    self.cleared(f"TWO_CLEAN_{definition}", a, definition)

                for mode, hit_z in (("primary", 3), ("collision", 1)):
                    self.reset()
                    self.send("setblock 0 85 1 minecraft:stone")
                    self.send("setblock 0 85 3 minecraft:gold_block")
                    self.cast(mode)
                    self.block(f"MODE_{mode}", 0, 85, hit_z, "diamond_block")
                    self.score(f"MODE_{mode}_ONCE", a, "hits", 1)
                    self.score(f"MODE_{mode}_NO_MISS", a, "misses", 0)

                self.reset()
                self.send("setblock 0 85 1 minecraft:stone")
                self.send("setblock 0 85 2 minecraft:oak_log")
                self.cast("tagged")
                self.block("TAG_MATCH", 0, 85, 2, "diamond_block")
                self.block("TAG_NONMATCH", 0, 85, 1, "stone")

                self.reset()
                self.send("setblock 0 86 2 minecraft:gold_block")
                self.cast("eyes")
                self.block("EYES_ANCHOR_ONCE", 0, 86, 2, "diamond_block")
                self.score("EYES_HIT", a, "hits", 1)

                for name, endpoint in (("short", 0), ("remainder", 2)):
                    self.reset()
                    self.cast(name, origin="positioned 0.5 85 0.25 rotated 0 0")
                    self.block(f"CLAMP_{name}", 0, 85, endpoint, "redstone_block")
                    self.block(f"NO_OVERSHOOT_{name}", 0, 85, endpoint + 1, "air")
                    self.score(f"MISS_ONCE_{name}", a, "misses", 1)

                # Generic callbacks work with a nonplayer entity. Player-only
                # callbacks terminate silently at both hit and final miss.
                self.send('summon minecraft:marker 0.5 85 0.5 {Tags:["ray_marker"]}')
                marker = "@e[tag=ray_marker,limit=1]"
                self.reset()
                self.send("setblock 0 85 2 minecraft:gold_block")
                self.cast(target=marker)
                self.score("ENTITY_CALLBACK", marker, "hits", 1)
                for target, label in ((marker, "ENTITY"), (a, "PLAYER")):
                    for hit in (True, False):
                        self.reset()
                        if hit:
                            self.send("setblock 0 85 2 minecraft:gold_block")
                        self.cast("player_callbacks", target=target)
                        self.score(f"{label}_PLAYER_CALLBACK_{hit}", target, "player", int(target == a))
                        self.score(f"{label}_NO_FALLBACK_MISS_{hit}", target, "misses", 0)
                        self.cleared(f"{label}_CALLBACK_CLEAN_{hit}", target, "player_callbacks")

                for name in ("collision", "hit_only"):
                    self.reset()
                    self.cast(name, origin="positioned 1000000 85 0 rotated 0 0")
                    self.score(f"UNLOADED_NO_COLLISION_{name}", a, "hits", 0)
                    self.score(f"UNLOADED_MISS_{name}", a, "misses", int(name == "collision"))
                    self.cleared(f"UNLOADED_CLEAN_{name}", a, name)
                for name in ("collision", "hit_only"):
                    self.reset()
                    self.cast(name, origin="positioned 0.5 1000 0.5 rotated 0 0")
                    self.score(f"OUT_OF_WORLD_NO_COLLISION_{name}", a, "hits", 0)
                    self.score(f"OUT_OF_WORLD_MISS_{name}", a, "misses", int(name == "collision"))
                    self.cleared(f"OUT_OF_WORLD_CLEAN_{name}", a, name)
                self.reset()
                self.cast("hit_only")
                self.score("HIT_ONLY_EXHAUSTION", a, "hits", 0)
                self.score("HIT_ONLY_SILENT", a, "misses", 0)
                self.block("HIT_ONLY_NO_CALLBACK", 0, 85, 3, "air")
                self.cleared("HIT_ONLY_CLEAN", a, "hit_only")

                self.reset()
                self.send(f"function {NS}:cast_primary")
                self.block("SERVER_NO_EXECUTOR_WORLD", 0, 85, 3, "air")
                for player in self.players:
                    self.score(f"SERVER_NO_EXECUTOR_{player}", player, "hits", 0)
                    self.cleared(f"SERVER_NO_STATE_{player}", player, "primary")

                self.reset()
                self.send("execute in minecraft:the_nether run fill 0 84 0 7 88 9 minecraft:air")
                self.send("execute in minecraft:the_nether run setblock 2 85 0 minecraft:gold_block")
                self.cast(origin="in minecraft:the_nether positioned 0.5 85 0.5 rotated -90 0")
                self.block("DIMENSION_ROTATION_POSITION", 2, 85, 0, "diamond_block", "minecraft:the_nether")
                self.block("ORIGINAL_DIMENSION_UNCHANGED", 2, 85, 0, "air")
                self.score("DIMENSION_EXECUTOR", a, "hits", 1)
            finally:
                if self.process and self.process.poll() is None:
                    self.send("tick unfreeze")
        print(f"PASSED: {len(self.checks)} block raycast runtime assertions", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output")
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    source = {
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
        "worktree_status": subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True),
        "fixture_sha256": sha256(ROOT / "sand/tests/block_raycast.rs"),
        "harness_sha256": sha256(Path(__file__)),
    }
    records = export_fixture()
    with tempfile.TemporaryDirectory(prefix="sand-raycast-pack-") as directory:
        pack = Path(directory) / NS
        write_pack(pack, records)
        args.pack = str(pack)
        harness = RaycastHarness(args)
        harness.provenance = {
            "source": source,
            "pack": pack_identity(pack),
            "server_jar": {"sha256": sha256(args.jar)},
        }
        harness.counters = {
            record["path"].split("/")[-2]: record["content"].split()[3]
            for record in records
            if record["dir"] == "function" and record["path"].startswith("__sand_system/raycast/")
            and record["path"].endswith("/load")
        }
        try:
            harness.run()
        except ValidationFailure as error:
            harness.persist("failed")
            print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
            return 1
        harness.persist("passed")
        print(f"PASSED: Minecraft 26.2 block raycasts; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
