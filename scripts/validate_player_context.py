#!/usr/bin/env python3
"""Run the maintained player-handle fixture with two real Minecraft 26.2 players."""
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


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def export_fixture():
    with tempfile.TemporaryDirectory(prefix="sand-player-context-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run([
            "cargo", "test", "-p", "sand", "--test", "player_context", "--",
            "--ignored", "--exact", "dump_player_context_pack",
        ], cwd=ROOT, env=dict(os.environ, SAND_PLAYER_CONTEXT_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


class ContextHarness(PlayerHarness):
    def prepare(self):
        super().prepare()
        self.checks = []
        self.persist("running")

    def persist(self, status):
        (self.output / "evidence.json").write_text(json.dumps({
            "status": status, "minecraft": "26.2", **self.provenance,
            "checks": self.checks,
        }, indent=2))

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition})
        self.persist("running")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.load_test_chunk("minecraft:the_nether")
        with self.clients():
            a, b = self.players
            for player in self.players:
                self.eventually(f"JOIN_{player}", f"if score {player} {self.calls} matches 2..")
                self.check(f"EVENT_HANDLE_{player}", f"if score {player} {self.calls} matches 2..")
            self.send("tick freeze")
            try:
                for player, count in ((a, 3), (b, 7)):
                    self.send(f"gamemode creative {player}")
                    self.send(f"clear {player}")
                    self.send(f"give {player} minecraft:diamond {count}")
                    self.send(f"item replace entity {player} weapon.offhand with minecraft:stick")
                    self.send(f"scoreboard players set {player} {self.calls} 0")
                    self.send(f"scoreboard players set {player} {self.diamonds} 0")
                self.send(f"execute as {a} run function handles:invoke")
                self.check("TYPED_CALL_A", f"if score {a} {self.calls} matches 1")
                self.check("COUNT_A", f"if score {a} {self.diamonds} matches 3")
                self.check("OTHER_PLAYER_UNTOUCHED", f"if score {b} {self.calls} matches 0")
                self.check("OFFHAND_A_CLEARED", f"unless items entity {a} weapon.offhand minecraft:stick")
                self.check("OFFHAND_B_RETAINED", f"if items entity {b} weapon.offhand minecraft:stick")
                self.send("function handles:query_players")
                for player, calls, diamonds in ((a, 2, 3), (b, 1, 7)):
                    self.check(f"QUERY_CALLS_{player}", f"if score {player} {self.calls} matches {calls}")
                    self.check(f"QUERY_COUNT_{player}", f"if score {player} {self.diamonds} matches {diamonds}")
                    self.check(f"QUERY_OFFHAND_{player}", f"unless items entity {player} weapon.offhand minecraft:stick")
                    self.check(f"COUNT_NONMUTATING_{player}", f"if items entity {player} hotbar.0 minecraft:diamond[count={diamonds}]")
                self.send('summon minecraft:marker 0 85 0 {Tags:["context_nonplayer"]}')
                self.send("execute as @e[tag=context_nonplayer] run function handles:assumed_context")
                self.check("NONPLAYER_GUARD", f"unless score @e[tag=context_nonplayer,limit=1] {self.calls} matches -2147483648..2147483647")
                self.send("function handles:assumed_context")
                self.check("SERVER_GUARD_A", f"if score {a} {self.calls} matches 2")
                self.check("SERVER_GUARD_B", f"if score {b} {self.calls} matches 1")
                for dimension in ("minecraft:overworld", "minecraft:the_nether"):
                    self.send(f"execute in {dimension} run fill 0 84 0 6 89 6 minecraft:air")
                self.send(f"tp {a} 20 85 20 0 0")
                self.send(f"execute as {a} in minecraft:the_nether positioned 0.5 85 0.5 rotated -90 0 anchored eyes run function handles:position")
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
    with tempfile.TemporaryDirectory(prefix="sand-player-context-pack-") as directory:
        pack = Path(directory) / "handles"
        pack.mkdir()
        (pack / "pack.mcmeta").write_text(json.dumps({"pack": {
            "pack_format": 107, "min_format": 107, "max_format": 107,
            "description": "Sand player context regression",
        }}))
        for record in records:
            target = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(record["content"])
        args.pack = str(pack)
        harness = ContextHarness(args)
        harness.calls = fixture["objectives"]["calls"]
        harness.diamonds = fixture["objectives"]["diamonds"]
        harness.provenance = {
            "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "worktree_status": subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True),
            "fixture_sha256": digest(ROOT / "sand/tests/player_context.rs"),
            "harness_sha256": digest(__file__), "server_jar_sha256": digest(args.jar),
            "pack_files": {p.relative_to(pack).as_posix(): digest(p) for p in sorted(pack.rglob("*")) if p.is_file()},
        }
        try:
            harness.run()
        except ValidationFailure as error:
            harness.persist("failed")
            print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
            return 1
        harness.persist("passed")
        print(f"PASSED: {len(harness.checks)} player context assertions; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
