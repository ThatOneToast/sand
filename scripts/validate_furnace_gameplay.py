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
import zipfile

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure

ROOT = Path(__file__).resolve().parents[1]
VERIFIED_INNER_SHA256 = "183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8"


def verified_server(jar):
    """Fail before export/start unless this bundle carries the inspected binary."""
    with zipfile.ZipFile(jar) as bundle:
        entries = [line.split() for line in bundle.read("META-INF/versions.list").decode().splitlines()]
        if len(entries) != 1 or len(entries[0]) != 3:
            raise ValueError("expected one bundled server version")
        digest, version, path = entries[0]
        inner = bundle.read(f"META-INF/versions/{path}")
        import io
        with zipfile.ZipFile(io.BytesIO(inner)) as server:
            identity = json.loads(server.read("version.json"))
        actual = hashlib.sha256(inner).hexdigest()
        if version != "26.2" or identity["id"] != "26.2" or digest != actual or actual != VERIFIED_INNER_SHA256:
            raise ValueError("server is not the exact inspected Minecraft 26.2 binary")
        return {"version": version, "bundled_path": path, "sha256": actual, "version_json": identity}


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

    def step(self, ticks, label):
        self.send("execute store result score #tick_end furnace_audit run time query gametime")
        self.send(f"scoreboard players add #tick_end furnace_audit {ticks}")
        self.send(f"tick step {ticks}")
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send("execute store result score #tick_now furnace_audit run time query gametime")
            if self.passed(label, f"execute if score #tick_now furnace_audit = #tick_end furnace_audit run say __SAND_WORLD_{label}_OK__"):
                return
            time.sleep(0.05)
        raise ValidationFailure(label, "stepped ticks did not finish")

    def nested_player_probe(self, label, outer, inner):
        # Poison the previously successful read so a skipped inner assignment
        # cannot satisfy the rebinding oracle with its old value.
        self.send(f"scoreboard players set {inner} {self.objectives['burn']} -707")
        self.call("nested_players", outer)
        self.score(f"{label}_NESTED_TARGET_REBINDS", inner, "burn", 93)
        self.score(f"{label}_NESTED_TARGET_RESTORES", outer, "elapsed", 71)

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
            self.step(1, f"{label}_{machine}_IGNITION_TICK")
            self.wait_condition(f"{label}_{machine}_LIT", f"if block 0 85 0 minecraft:{machine}[lit=true]")
            self.native_snapshot(f"{label}_{machine}_ACTIVE", a)
            self.score(f"{label}_{machine}_BURNING", a, "burn", "1..")
            self.score(f"{label}_{machine}_CONDITION", a, "ready", 0 if machine == "smoker" else 1)
            self.step(total - 1, f"{label}_{machine}_PRODUCTION_TICKS")
            self.wait_condition(f"{label}_{machine}_COOKED", f"if items block 0 85 0 container.2 minecraft:{product}")
            self.check(f"{label}_{machine}_PRODUCTION", f"if items block 0 85 0 container.2 minecraft:{product}")
            self.native_snapshot(f"{label}_{machine}_COMPLETED", a)
            self.score(f"{label}_{machine}_COMPLETED_PROGRESS", a, "elapsed", 0)

            # Empty fuel, unusable recipe and blocked output are distinct states.
            for edge in ("NO_FUEL", "INVALID_INPUT", "FULL_OUTPUT", "WRONG_OUTPUT"):
                self.send(f"setblock 0 85 0 minecraft:{machine}")
                self.call(seed)
                if edge == "NO_FUEL":
                    self.send("item replace block 0 85 0 container.1 with minecraft:air")
                elif edge == "INVALID_INPUT":
                    self.send("item replace block 0 85 0 container.0 with minecraft:stick")
                else:
                    item = product if edge == "FULL_OUTPUT" else "diamond"
                    self.send(f"item replace block 0 85 0 container.2 with minecraft:{item} 64")
                self.step(3, f"{label}_{machine}_{edge}_TICKS")
                self.native_snapshot(f"{label}_{machine}_{edge}", a)
                self.score(f"{label}_{machine}_{edge}_PROGRESS", a, "elapsed", 0)
                self.score(f"{label}_{machine}_{edge}_BURN", a, "burn", 0)

            self.send(f"setblock 0 85 0 minecraft:{machine}")
            self.call(seed)
            self.step(8, f"{label}_{machine}_PARTIAL_TICKS")
            self.native_snapshot(f"{label}_{machine}_PARTIAL", a)
            self.score(f"{label}_{machine}_PARTIAL_PROGRESS", a, "elapsed", 8)
            self.send("item replace block 0 85 0 container.0 with minecraft:stick")
            self.native_snapshot(f"{label}_{machine}_CHANGED_INPUT", a)
            self.score(f"{label}_{machine}_INPUT_RESET", a, "elapsed", 0)
            self.check(f"{label}_{machine}_INPUT_CHANGED", "if items block 0 85 0 container.0 minecraft:stick")
            # Native cooldown needs remaining progress without a burning fuel.
            # Arrange that native edge, then step real ticks (no author timer API).
            self.send(f"data merge block 0 85 0 {{lit_time_remaining:0s,cooking_time_spent:8s,Items:[]}}")
            self.step(1, f"{label}_{machine}_COOLDOWN_TICK")
            self.native_snapshot(f"{label}_{machine}_COOLDOWN", a)
            self.score(f"{label}_{machine}_COOLDOWN_PROGRESS", a, "elapsed", 6)

        # Deterministic serialized state now isolates context and failure checks.
        self.send('setblock 0 85 0 minecraft:furnace{lit_time_remaining:71s}')
        self.send('setblock 4 85 0 minecraft:smoker{lit_time_remaining:93s}')
        self.reset(a)
        self.reset(b)
        self.send(f"tp {a} 0 85 0")
        self.send(f"tp {b} 4 85 0")
        self.send("function furnace:both_players")
        self.score(f"{label}_PLAYER_A", a, "burn", 71)
        self.score(f"{label}_PLAYER_B", b, "burn", 93)
        self.send(f"tag {b} add furnace_other")
        self.nested_player_probe(label, a, b)
        self.send("function furnace:both_same_machine")
        self.score(f"{label}_SAME_MACHINE_A", a, "burn", 71)
        self.score(f"{label}_SAME_MACHINE_B", b, "burn", 71)
        self.send('execute in minecraft:the_nether run setblock 0 85 0 minecraft:furnace{lit_time_remaining:39s}')
        self.call("absolute_snapshot", a, "100 85 100", "the_nether")
        self.score(f"{label}_ABSOLUTE_NETHER", a, "burn", 39)
        self.call("absolute_snapshot", a, "100 85 100", "overworld")
        self.score(f"{label}_ABSOLUTE_OVERWORLD", a, "burn", 71)
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
        self.score(f"{label}_FAILURE_RETAINS_DESTINATION", a, "burn", -909)
        self.score(f"{label}_FAILURE_CONTINUES_BODY", a, "after", 2)
        self.call("snapshot", a)  # air: family guard must skip it
        self.score(f"{label}_MISSING_NOT_IDLE", a, "burn", -909)
        self.score(f"{label}_MISSING_NO_BODY", a, "hits", 1)
        self.call("snapshot", a, "20000000 85 20000000")
        self.score(f"{label}_UNLOADED_NO_BODY", a, "hits", 1)
        self.send("setblock 0 85 0 minecraft:chest")
        self.call("snapshot", a)
        self.score(f"{label}_NONFAMILY_BLOCK_ENTITY", a, "hits", 1)
        self.send('setblock 0 319 0 minecraft:furnace{lit_time_remaining:11s}')
        self.call("snapshot", a, "0 319 0")
        self.score(f"{label}_HEIGHT_LAST_VALID", a, "burn", 11)
        self.call("snapshot", a, "0 320 0")
        self.score(f"{label}_HEIGHT_OUTSIDE", a, "burn", 11)
        self.score(f"{label}_HEIGHT_OUTSIDE_NO_BODY", a, "hits", 2)
        self.send('setblock 0 85 0 minecraft:furnace{lit_time_remaining:-123s,cooking_time_spent:-7s,cooking_total_time:0s}')
        self.native_snapshot(f"{label}_SIGNED", a)
        self.score(f"{label}_SIGNED_BURN", a, "burn", -123)
        self.score(f"{label}_SIGNED_ELAPSED", a, "elapsed", -7)
        self.send("execute positioned 0 85 0 run function furnace:global_snapshot")
        self.send(f"execute store result score #global furnace_audit run {self.global_query}")
        self.check(f"{label}_GLOBAL_STATE", "if score #global furnace_audit matches -123")
        self.send(f'data modify storage furnace:macro args set value {{message:"__FURNACE_{label}_MACRO__"}}')
        start = len(self.lines)
        self.call("explicit_macro", a)
        self.wait_for(re.compile(f"__FURNACE_{label}_MACRO__$"), "explicit-macro", start)

    def run_client(self):
        executed = self.server / "versions" / self.provenance["verified_server"]["bundled_path"]
        if sha256(executed) != VERIFIED_INNER_SHA256:
            raise ValidationFailure("server-identity", "executed extracted server JAR differs from inspected binary")
        self.provenance["executed_inner_jar"] = {"path": str(executed), "sha256": sha256(executed)}
        self.persist("running")
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
    server_identity = verified_server(args.jar)
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
            "verified_server": server_identity,
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
