#!/usr/bin/env python3
"""Run Python-authored player and global flows through the prebuilt portable compiler."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from measure_archetype_idle import verified_runtime_version
from player_validation_harness import PlayerHarness
from portable_program.producer import encounter_flows
from vanilla_reload_harness import RELOAD_START


class PortableFlowHarness(PlayerHarness):
    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition, "passed": True})

    def score(self, name, holder, objective, value):
        self.check(name, f"if score {holder} {objective} matches {value}")

    def run_client(self):
        verified_runtime_version(self.lines)
        self.load_test_chunk("minecraft:overworld")
        self.send("tick freeze")
        self.check("NO_PLAYERS", "unless entity @a")
        self.send(f"scoreboard players reset {self.world_holder}")
        self.send("function demo:__sand_lifecycle_load")
        self.send(f"function {self.world_root}")
        self.score("GLOBAL_TRANSITION_WITHOUT_PLAYERS", self.world_holder, self.world_phase, 1)
        self.score("GLOBAL_CADENCE_FIRST_TICK", self.world_holder, self.world_ticks, 0)
        self.send(f"function {self.world_root}")
        self.score("GLOBAL_CADENCE_SECOND_TICK", self.world_holder, self.world_ticks, 1)
        with self.clients():
            self.send("execute as @a run function demo:__sand_lifecycle_init")
            self.send(f"scoreboard players set SandBeta {self.energy} 5")
            self.send("function demo:__sand_lifecycle_tick")
            self.score("ALPHA_GUARDED_TRANSITION", "SandAlpha", self.phase, 1)
            self.score("ALPHA_ENTER_COST", "SandAlpha", self.energy, 70)
            self.score("ALPHA_ENTER_THEN_TICK", "SandAlpha", self.events_objective, 11)
            self.score("BETA_FALLBACK_TRANSITION", "SandBeta", self.phase, 2)
            self.score("BETA_ENTER_ONCE", "SandBeta", self.events_objective, 100)
            self.score("BETA_COST_ISOLATED", "SandBeta", self.energy, 5)
            self.send(f"execute as SandAlpha at @s run function {self.player_root}")
            self.score("ALPHA_INTERVAL_COST", "SandAlpha", self.energy, 65)
            self.score("ALPHA_SECOND_TICK", "SandAlpha", self.events_objective, 12)
            self.score("BETA_HOOK_ISOLATED", "SandBeta", self.events_objective, 100)
            self.send(f"scoreboard players set SandAlpha {self.energy} 10")
            self.send(f"execute as SandAlpha at @s run function {self.player_root}")
            self.score("ALPHA_REST_TRANSITION", "SandAlpha", self.phase, 2)
            self.score("REST_ENTER_WITHOUT_ACTIVE_TICK", "SandAlpha", self.events_objective, 112)
            self.score("REST_HAS_NO_PULSE", "SandAlpha", self.energy, 10)
            self.score("PLAYER_FLOW_DOES_NOT_ADVANCE_GLOBAL_CLOCK", self.world_holder, self.world_ticks, 1)
            start = len(self.lines)
            self.send("reload")
            self.wait_for(RELOAD_START, "portable-flow-reload", start)
            self.send("function demo:__sand_lifecycle_load")
            self.score("RELOAD_PLAYER_STATE", "SandAlpha", self.phase, 2)
            self.score("RELOAD_PLAYER_HOOK_STATE", "SandAlpha", self.events_objective, 112)
            self.score("RELOAD_GLOBAL_STATE", self.world_holder, self.world_ticks, 1)
            self.send(f"function {self.world_root}")
            self.score("RELOAD_PRESERVES_GLOBAL_CADENCE", self.world_holder, self.world_ticks, 2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    program = root / "program.sand.json"
    program.write_text(json.dumps(encounter_flows(), indent=2) + "\n")
    pack = root / "pack"
    result = subprocess.run([str(args.binary.resolve()), "program", "compile", "--input",
        str(program), "--output", str(pack)], check=True, capture_output=True, text=True,
        env={"PATH": str(root / "no-cargo")}, timeout=60)
    assert json.loads(result.stdout)["success"]
    functions = {p.relative_to(pack / "data/demo/function").as_posix().removesuffix(".mcfunction"): p.read_text()
                 for p in (pack / "data/demo/function").rglob("*.mcfunction")}
    def one(pattern, body):
        matches = re.findall(pattern, body, re.MULTILINE)
        assert len(matches) == 1, (pattern, body)
        return matches[0]
    options = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar.resolve()),
        java=args.java, timeout=90, output=str(root / "logs"), op_player=None, client_command=None)
    harness = PortableFlowHarness(options)
    harness.energy = one(r"^scoreboard players remove @s (\S+) 30$", functions["start"])
    harness.events_objective = one(r"^scoreboard players add @s (\S+) 1$", functions["active"])
    harness.world_holder, harness.world_ticks = one(r"^scoreboard players add (\S+) (\S+) 1$", functions["world_tick"])
    tick = functions["__sand_lifecycle_tick"]
    harness.player_root = one(r"^execute as @a at @s run function (\S+)$", tick)
    harness.world_root = one(r"^function (\S+)$", tick)
    harness.phase = one(r"^execute unless score @s (\S+) matches -2147483648\.\.2147483647 run scoreboard players set @s \S+ 0$",
                        functions[harness.player_root.split(":", 1)[1]])
    harness.world_phase = one(r"^execute unless score #\S+ (\S+) matches -2147483648\.\.2147483647 run scoreboard players set #\S+ \S+ 0$",
                              functions[harness.world_root.split(":", 1)[1]])
    digest = hashlib.sha256()
    for path in sorted(pack.rglob("*")):
        if path.is_file() and path.name != ".sand-build-manifest.json":
            digest.update(path.relative_to(pack).as_posix().encode() + b"\0")
            digest.update(path.read_bytes() + b"\0")
    harness.checks = []
    passed = False
    try:
        harness.run()
        passed = True
    finally:
        (root / "evidence.json").write_text(json.dumps({"minecraft": "26.2", "passed": passed,
            "scope": "Python producer compiled without Cargo; zero-player global flow and two real passive players; automatic ticks frozen",
            "pack_sha256": digest.hexdigest(), "checks": harness.checks}, indent=2) + "\n")
    print("EVIDENCE", root / "evidence.json")


if __name__ == "__main__":
    main()
