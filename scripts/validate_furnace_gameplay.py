#!/usr/bin/env python3
"""Dedicated 26.2 furnace-family proof using the real public Sand fixture.

Only this harness arranges chunks, ticks and raw assertions. Gameplay comes from
sand/tests/furnace_gameplay.rs. Uses private disposable server state and ports.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]


def sha256(path):
    with Path(path).open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def export_fixture():
    with tempfile.TemporaryDirectory(prefix="sand-furnace-export-") as directory:
        dump = Path(directory) / "fixture.json"
        subprocess.run(["cargo", "test", "-p", "sand", "--test", "furnace_gameplay",
                        "--", "--ignored", "--exact", "dump_furnace_pack"], cwd=ROOT,
                       env=dict(os.environ, SAND_FURNACE_DUMP_PATH=str(dump)), check=True)
        return json.loads(dump.read_text())


def write_pack(path, records):
    path.mkdir(parents=True)
    (path / "pack.mcmeta").write_text(json.dumps({"pack": {
        "pack_format": 107, "min_format": 107, "max_format": 107,
        "description": "Sand furnace-family gameplay proof"}}))
    for record in records:
        file = path / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(record["content"])


class FurnaceHarness(PlayerHarness):
    def prepare(self):
        super().prepare()
        self.checks = []
        self.persist("running")

    def persist(self, status):
        (self.output / "evidence.json").write_text(json.dumps({
            "schema_version": 1, "minecraft": "26.2", "status": status,
            **self.provenance, "checks": self.checks}, indent=2))

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition})
        self.persist("running")

    def call(self, name, player=None, position="0 85 0", dimension="overworld"):
        player = player or self.players[0]
        self.send(f"execute as {player} in minecraft:{dimension} positioned {position} run function furnace:{name}")

    def score(self, name, player, field, value):
        self.check(name, f"if score {player} {self.objectives[field]} matches {value}")

    def reset(self, player):
        for objective in self.objectives.values():
            self.send(f"scoreboard players set {player} {objective} 0")

    def wait_condition(self, label, condition):
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            if self.passed(label, f"execute {condition} run say __SAND_WORLD_{label}_OK__"):
                return
            time.sleep(0.05)
        raise ValidationFailure(label, f"condition never became true: {condition}")

    def native_snapshot(self, label, player, position="0 85 0", dimension="overworld"):
        self.call("snapshot", player, position, dimension)
        for field, key in (("elapsed", "cooking_time_spent"), ("total", "cooking_total_time"), ("burn", "lit_time_remaining")):
            self.send(f"execute in minecraft:{dimension} store result score #native furnace_audit run data get block {position} {key}")
            self.check(f"{label}_{field}", f"if score {player} {self.objectives[field]} = #native furnace_audit")

    def scenarios(self, label):
        a, b = self.players
        for player in self.players:
            self.reset(player)
        for machine, seed, product, total in (
                ("furnace", "seed_iron", "iron_ingot", 200),
                ("blast_furnace", "seed_iron", "iron_ingot", 100),
                ("smoker", "seed_food", "cooked_beef", 100)):
            self.send(f"setblock 0 85 0 minecraft:{machine}")
            self.native_snapshot(f"{label}_{machine}_ZERO", a)
            self.score(f"{label}_{machine}_ZERO_BURN", a, "burn", 0)
            self.call(seed)
            self.native_snapshot(f"{label}_{machine}_SEEDED", a)
            self.score(f"{label}_{machine}_RECIPE_TOTAL", a, "total", total)
            self.send("tick step 1")
            self.wait_condition(f"{label}_{machine}_LIT", f"if block 0 85 0 minecraft:{machine}[lit=true]")
            self.native_snapshot(f"{label}_{machine}_ACTIVE", a)
            self.score(f"{label}_{machine}_BURNING", a, "burn", "1..")
            self.score(f"{label}_{machine}_CONDITION", a, "ready", 0 if machine == "smoker" else 1)
            self.send(f"tick step {total - 1}")
            self.wait_condition(f"{label}_{machine}_COOKED", f"if items block 0 85 0 container.2 minecraft:{product}")
            self.check(f"{label}_{machine}_PRODUCTION", f"if items block 0 85 0 container.2 minecraft:{product}")

        # Deterministic serialized state now isolates context and failure checks.
        self.send('setblock 0 85 0 minecraft:furnace{lit_time_remaining:71s}')
        self.send('setblock 4 85 0 minecraft:smoker{lit_time_remaining:93s}')
        self.reset(a)
        self.reset(b)
        self.call("snapshot", a)
        self.call("snapshot", b, "4 85 0")
        self.score(f"{label}_PLAYER_A", a, "burn", 71)
        self.score(f"{label}_PLAYER_B", b, "burn", 93)
        self.send('setblock 0 86 0 minecraft:blast_furnace{lit_time_remaining:27s}')
        self.call("nested", a)
        self.score(f"{label}_NESTED_INNER", a, "total", 27)
        self.score(f"{label}_NESTED_RESTORE", a, "elapsed", 71)
        self.send('execute in minecraft:the_nether run setblock 1 85 0 minecraft:furnace{lit_time_remaining:37s}')
        self.call("in_nether", a, "8 85 0")
        self.score(f"{label}_SCALED_DIMENSION", a, "burn", 37)
        self.score(f"{label}_CALLER_CONTINUES", a, "after", 12)
        self.send('setblock 0 85 1 minecraft:furnace{lit_time_remaining:19s}')
        self.send(f"execute as {a} positioned 0 85 0 rotated 0 0 run function furnace:local_snapshot")
        self.score(f"{label}_LOCAL_POSITION", a, "burn", 19)
        self.reset(a)
        self.call("mutate_once", a)
        self.score(f"{label}_MUTATE_ONCE", a, "hits", 1)
        self.score(f"{label}_RETURN_ISOLATION", a, "after", 1)
        self.send('setblock 0 85 0 minecraft:furnace{lit_time_remaining:45s}')
        self.call("fail_after_valid", a)
        self.score(f"{label}_FAILURE_RETAINS_DESTINATION", a, "burn", 45)
        self.score(f"{label}_FAILURE_CONTINUES_BODY", a, "after", 2)
        self.call("snapshot", a)  # air: family guard must skip it
        self.score(f"{label}_MISSING_NOT_IDLE", a, "burn", 45)
        self.score(f"{label}_MISSING_NO_BODY", a, "hits", 1)
        self.call("snapshot", a, "20000000 85 20000000")
        self.score(f"{label}_UNLOADED_NO_BODY", a, "hits", 1)
        self.send('setblock 0 85 0 minecraft:furnace{lit_time_remaining:-123s,cooking_time_spent:-7s,cooking_total_time:0s}')
        self.native_snapshot(f"{label}_SIGNED", a)
        self.score(f"{label}_SIGNED_BURN", a, "burn", -123)
        self.score(f"{label}_SIGNED_ELAPSED", a, "elapsed", -7)
        self.send("execute positioned 0 85 0 run function furnace:global_snapshot")
        self.send(f"execute store result score #global furnace_audit run {self.global_query}")
        self.check(f"{label}_GLOBAL_STATE", "if score #global furnace_audit matches -123")

    def run_client(self):
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            self.load_test_chunk(dimension)
        self.send("scoreboard objectives add furnace_audit dummy")
        with self.clients():
            for player in self.players:
                self.send(f"gamemode creative {player}")
                self.send(f"tp {player} 0 90 0")
            self.send("tick freeze")
            try:
                self.scenarios("INITIAL")
                start = len(self.lines)
                self.send("reload")
                self.send("say __FURNACE_RELOAD_COMPLETE__")
                self.wait_for(re.compile(r"\[Server\] __FURNACE_RELOAD_COMPLETE__$"), "furnace-reload", start)
                self.scenarios("RELOAD")
            finally:
                self.send("tick unfreeze")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    fixture = export_fixture()
    with tempfile.TemporaryDirectory(prefix="sand-furnace-pack-") as directory:
        pack = Path(directory) / "furnace"
        write_pack(pack, fixture["components"])
        files = {str(p.relative_to(pack)): sha256(p) for p in sorted(pack.rglob("*")) if p.is_file()}
        args.pack = str(pack)
        harness = FurnaceHarness(args)
        harness.objectives = fixture["objectives"]
        harness.global_query = fixture["global_burn_query"]
        harness.provenance = {
            "source": {key: subprocess.check_output(command, cwd=ROOT, text=True).strip()
                       for key, command in {"commit": ["git", "rev-parse", "HEAD"],
                                            "tree": ["git", "rev-parse", "HEAD^{tree}"],
                                            "worktree_status": ["git", "status", "--porcelain"]}.items()},
            "fixture_sha256": sha256(ROOT / "sand/tests/furnace_gameplay.rs"),
            "harness_sha256": sha256(Path(__file__)), "server_jar_sha256": sha256(args.jar),
            "pack_files": files,
            "pack_sha256": hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
        }
        try:
            harness.run()
        except ValidationFailure as error:
            harness.persist("failed")
            print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
            return 1
        harness.persist("passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
