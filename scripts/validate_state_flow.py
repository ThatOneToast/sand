#!/usr/bin/env python3
"""Verify StateFlow priority, state dispatch, hook ordering and player isolation."""
import argparse
import hashlib
import json
import os
import re
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
        counter = self.fixture["counter"]
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
            # Direct writes must repair the exact private score at the live hook
            # position, including absent scores and signed extremes.
            for state in (None, 7, 20):
                for value in (None, 0, 1, -1, -2147483648, 2147483647):
                    label = f"INACTIVE_{state}_{value}"
                    if state is None:
                        self.send("scoreboard players reset SandAlpha fa_phase")
                    else:
                        self.send(f"scoreboard players set SandAlpha fa_phase {state}")
                    if value is None:
                        self.send(f"scoreboard players reset SandAlpha {counter}")
                    else:
                        self.send(f"scoreboard players set SandAlpha {counter} {value}")
                    self.send("scoreboard players set SandBeta fa_phase 10")
                    self.send(f"scoreboard players set SandBeta {counter} 1")
                    self.invoke()
                    self.score(label, "SandAlpha", counter, 0)
                    self.score(label + "_OTHER_SUBJECT", "SandBeta", counter, 1)
            self.send("scoreboard players set SandAlpha fa_phase 10")
            self.send(f"scoreboard players set SandAlpha {counter} 1")
            self.send(f"scoreboard players set SandAlpha {pulse} 0")
            self.send(f"scoreboard players set SandAlpha {trace} 0")
            self.send("tag SandAlpha add mutate_early")
            self.invoke()
            self.score("EARLIER_HOOK_RESETS_AT_LIVE_POSITION", "SandAlpha", counter, 0)
            self.score("EARLIER_HOOK_SUPPRESSES_DUE_PULSE", "SandAlpha", pulse, 0)
            self.score("LATER_HOOK_SEES_EARLY_WRITE", "SandAlpha", trace, 1000)
            self.send("tag SandAlpha remove mutate_early")
            self.send(f"scoreboard players reset SandAlpha {counter}")
            start = len(self.lines)
            self.send("reload")
            self.wait_for(RELOAD_START, "counter-reload", start)
            self.check("RELOAD_DOES_NOT_CREATE_MISSING_COUNTER",
                       f"unless score SandAlpha {counter} matches -2147483648..2147483647")
            self.score("RELOAD_PRESERVES_ACTIVE_COUNTER", "SandBeta", counter, 1)
            self.invoke()
            self.score("POST_RELOAD_MISSING_COUNTER_REPAIRED", "SandAlpha", counter, 0)
            self.invoke("SandBeta")
            self.score("POST_RELOAD_DUE_COUNTER_RESET", "SandBeta", counter, 0)
            # Capture function result/success and return-frame boundaries in the
            # native log for old/new comparison. No presumed vanilla result is
            # substituted for actually running these probes.
            self.send("scoreboard objectives add fa_boundary dummy")
            for wrapper in ("plain", "returned"):
                for value in (None, 0, 1):
                    for field in ("result", "success"):
                        if value is None:
                            self.send(f"scoreboard players reset SandAlpha {counter}")
                        else:
                            self.send(f"scoreboard players set SandAlpha {counter} {value}")
                        self.send("scoreboard players set #fellthrough fa_boundary 0")
                        self.send("scoreboard players set #probe fa_boundary -99")
                        self.send(f"execute as SandAlpha at @s store {field} score #probe fa_boundary run function flow_audit:probe_{wrapper}")
                        self.send(f"say BOUNDARY_{wrapper}_{value}_{field}")
                        start = len(self.lines)
                        self.send("scoreboard players get #probe fa_boundary")
                        pattern = re.compile(r"#probe has (-?\d+) \[fa_boundary\]")
                        index = self.wait_for(pattern, "flow-boundary", start)
                        self.boundaries.append({"wrapper": wrapper, "counter_before": value,
                                                "field": field, "value": int(pattern.search(self.lines[index])[1])})
                        self.score(f"BOUNDARY_{wrapper}_{value}_{field}_COUNTER", "SandAlpha", counter, 0)
                        self.score(f"BOUNDARY_{wrapper}_{value}_{field}_FRAME", "#fellthrough", "fa_boundary", 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--fixture", type=Path, help="Use an exact preserved export; do not invoke Cargo")
    parser.add_argument("--compare-evidence", type=Path,
                        help="Require checks and result/success boundaries to match a prior native run")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    dump = root / "fixture.json"
    if args.fixture:
        dump.write_bytes(args.fixture.read_bytes())
    else:
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
    functions = pack / "data/flow_audit/function"
    (functions / "probe_plain.mcfunction").write_text(f"function flow_audit:{fixture['root']}\n")
    (functions / "probe_returned.mcfunction").write_text(f"return run function flow_audit:{fixture['root']}\nscoreboard players set #fellthrough fa_boundary 1\n")
    digest = hashlib.sha256()
    for path in sorted(pack.rglob("*")):
        if path.is_file():
            digest.update(path.relative_to(pack).as_posix().encode() + b"\0")
            digest.update(path.read_bytes() + b"\0")
    options = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar.resolve()),
        java=args.java, timeout=90, output=str(root / "logs"), op_player=None, client_command=None)
    harness = FlowHarness(options)
    harness.fixture, harness.checks, harness.boundaries = fixture, [], []
    passed = False
    try:
        harness.run()
        if args.compare_evidence:
            baseline = json.loads(args.compare_evidence.read_text())
            assert baseline["passed"], "baseline native run did not pass"
            assert baseline["checks"] == harness.checks, "native gameplay checks differ"
            assert baseline["boundaries"] == harness.boundaries, "native result/success boundaries differ"
        passed = True
    finally:
        (root / "evidence.json").write_text(json.dumps({"minecraft": "26.2", "passed": passed,
            "scope": "two real passive players; explicit flow invocations with automatic ticks frozen",
            "pack_sha256": digest.hexdigest(), "checks": harness.checks,
            "boundaries": harness.boundaries}, indent=2) + "\n")
    print("EVIDENCE", root / "evidence.json")


if __name__ == "__main__":
    main()
