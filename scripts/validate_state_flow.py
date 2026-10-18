#!/usr/bin/env python3
"""Verify StateFlow priority, state dispatch, hook ordering and player isolation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from measure_archetype_idle import verified_runtime_version
from player_validation_harness import PlayerHarness
from vanilla_reload_harness import RELOAD_START

ROOT = Path(__file__).resolve().parents[1]


class FlowHarness(PlayerHarness):
    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition, "passed": True})

    def score(self, name, player, objective, expected):
        self.check(name, f"if score {player} {objective} matches {expected}")

    def invoke(self, player="SandAlpha"):
        self.send(f"execute as {player} at @s run function flow_audit:{self.fixture['root']}")

    def run_client(self):
        verified_runtime_version(self.lines)
        self.load_test_chunk("minecraft:overworld")
        self.send("tick freeze")
        trace, pulse = self.fixture["trace"], self.fixture["pulse"]
        with self.clients():
            for player in self.players:
                for objective in ("fa_phase", trace, pulse):
                    self.send(f"scoreboard players set {player} {objective} 0")
            self.send("scoreboard players set SandBeta fa_phase 20")
            self.send("tag SandAlpha add start")
            self.send("tag SandAlpha add alternate")
            self.send("tag SandBeta add advance")
            self.invoke("@a")
            self.score("ALPHA_PRIORITY_AND_TIE_ORDER", "SandAlpha", "fa_phase", 10)
            self.score("ALPHA_EXIT_ENTER_AND_SAME_TICK_HOOK", "SandAlpha", trace, 111)
            self.score("OR_GUARD_EXECUTES_ONCE", "SandAlpha", pulse, 0)
            self.score("BETA_INDEPENDENT_STATE", "SandBeta", "fa_phase", 2147483647)
            self.score("BETA_INDEPENDENT_HOOK", "SandBeta", trace, 10000)
            self.invoke()
            self.score("ALPHA_SECOND_TICK", "SandAlpha", trace, 211)
            self.score("INTERVAL_SECOND_TICK", "SandAlpha", pulse, 1)
            self.score("BETA_UNTOUCHED", "SandBeta", trace, 10000)
            self.send("tag SandAlpha add advance")
            self.invoke()
            self.score("NO_TRANSITION_CHAIN_IN_ONE_CYCLE", "SandAlpha", "fa_phase", 20)
            self.score("NEW_STATE_TICKS_AFTER_TRANSITION", "SandAlpha", trace, 1233)
            self.invoke()
            self.score("NEXT_CYCLE_TRANSITION", "SandAlpha", "fa_phase", 2147483647)
            self.score("NEXT_CYCLE_HOOK", "SandAlpha", trace, 11233)
            self.send("tag SandAlpha remove advance")
            self.send("scoreboard players set SandAlpha fa_phase 10")
            self.send(f"scoreboard players set SandAlpha {trace} 0")
            self.send(f"scoreboard players set SandAlpha {pulse} 0")
            self.send("tag SandAlpha add mutate")
            self.invoke()
            self.score("HOOK_MUTATES_STATE", "SandAlpha", "fa_phase", 20)
            self.score("LATER_HOOK_OBSERVES_MUTATION_WITHOUT_ENTER_EXIT", "SandAlpha", trace, 1100)
            self.send("tag SandAlpha remove mutate")
            self.invoke()
            self.send("scoreboard players set SandAlpha fa_phase 10")
            self.invoke()
            self.score("RAW_REENTRY_BYPASSES_ENTER_EXIT", "SandAlpha", trace, 2200)
            self.score("OBSERVED_LEAVE_RESETS_CADENCE", "SandAlpha", pulse, 0)
            self.invoke()
            self.score("REENTERED_INTERVAL", "SandAlpha", pulse, 1)
            self.send("scoreboard players set SandAlpha fa_phase -2147483648")
            self.send(f"scoreboard players set SandAlpha {trace} 0")
            self.invoke()
            self.score("SIGNED_DISPATCH_AND_NO_FALLTHROUGH", "SandAlpha", "fa_phase", 0)
            self.score("NO_UNRELATED_HOOK", "SandAlpha", trace, 0)
            self.send("scoreboard players set SandAlpha fa_phase 7")
            self.invoke()
            self.score("SPARSE_UNKNOWN_STATE_PRESERVED", "SandAlpha", "fa_phase", 7)
            self.score("UNKNOWN_STATE_HAS_NO_HOOK", "SandAlpha", trace, 0)
            self.send("tag SandAlpha remove start")
            self.send("tag SandAlpha remove alternate")
            self.send("scoreboard players reset SandAlpha fa_phase")
            self.invoke()
            self.score("MISSING_SCORE_DEFAULT", "SandAlpha", "fa_phase", 0)
            self.send("scoreboard players set SandAlpha fa_phase 20")
            self.send(f"scoreboard players set SandAlpha {trace} 321")
            start = len(self.lines)
            self.send("reload")
            self.wait_for(RELOAD_START, "flow-reload", start)
            self.score("RELOAD_STATE", "SandAlpha", "fa_phase", 20)
            self.score("RELOAD_HOOK_STATE", "SandAlpha", trace, 321)
            self.score("RELOAD_OTHER_PLAYER", "SandBeta", trace, 10000)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    dump = root / "fixture.json"
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "state_flow_runtime",
        "--", "--ignored", "--exact", "dump_state_flow_pack"], cwd=ROOT,
        env=dict(os.environ, SAND_STATE_FLOW_DUMP_PATH=str(dump)), check=True)
    fixture = json.loads(dump.read_text())
    pack = root / "pack"
    pack.mkdir()
    (pack / "pack.mcmeta").write_text(json.dumps({"pack": {"pack_format": 107,
        "min_format": 107, "max_format": 107, "description": "Sand StateFlow gameplay audit"}}))
    for record in fixture["records"]:
        path = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(record["content"])
    digest = hashlib.sha256()
    for path in sorted(pack.rglob("*")):
        if path.is_file():
            digest.update(path.relative_to(pack).as_posix().encode() + b"\0")
            digest.update(path.read_bytes() + b"\0")
    options = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar.resolve()),
        java=args.java, timeout=90, output=str(root / "logs"), op_player=None, client_command=None)
    harness = FlowHarness(options)
    harness.fixture, harness.checks = fixture, []
    passed = False
    try:
        harness.run()
        passed = True
    finally:
        (root / "evidence.json").write_text(json.dumps({"minecraft": "26.2", "passed": passed,
            "scope": "two real passive players; explicit flow invocations with automatic ticks frozen",
            "pack_sha256": digest.hexdigest(), "checks": harness.checks}, indent=2) + "\n")
    print("EVIDENCE", root / "evidence.json")


if __name__ == "__main__":
    main()
