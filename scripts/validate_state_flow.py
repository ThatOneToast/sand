#!/usr/bin/env python3
"""Verify preserved Rust/player or portable/global StateFlow exports on vanilla 26.2."""
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
        self.scores.append({"name": name, "holder": player, "objective": objective,
                            "exists": True, "value": expected})

    def absent(self, name, holder, objective):
        self.check(name, f"unless score {holder} {objective} matches -2147483648..2147483647")
        self.scores.append({"name": name, "holder": holder, "objective": objective,
                            "exists": False, "value": None})

    def seed(self, holder, objective, value):
        if value is None:
            self.send(f"scoreboard players reset {holder} {objective}")
        else:
            self.send(f"scoreboard players set {holder} {objective} {value}")

    def boundary_probes(self, holder, state, counter, pulse, active, inactive):
        self.send("scoreboard objectives add fa_boundary dummy")
        for wrapper in ("plain", "returned"):
            for mode, phase, values in (("inactive", inactive, (None, 0, 1, -1, -2147483648, 2147483647)),
                                        ("active", active, (None, 0, 1))):
                for value in values:
                    for field in ("result", "success"):
                        label = f"BOUNDARY_{wrapper}_{mode}_{value}_{field}"
                        self.seed(holder, state, phase)
                        self.seed(holder, counter, value)
                        self.seed(holder, pulse, 0)
                        self.seed("#fellthrough", "fa_boundary", 0)
                        self.seed("#probe", "fa_boundary", -99)
                        context = f"as {holder} at @s " if holder in self.players else ""
                        self.send(f"execute {context}store {field} score #probe fa_boundary run function flow_audit:probe_{wrapper}")
                        # A marker flushes queued commands before collecting the
                        # native result. No guessed vanilla function result.
                        start = len(self.lines)
                        self.send(f"say {label}")
                        self.wait_for(self.marker_pattern(label), "boundary-marker", start)
                        start = len(self.lines)
                        self.send("scoreboard players get #probe fa_boundary")
                        pattern = re.compile(r"#probe has (-?\d+) \[fa_boundary\]")
                        index = self.wait_for(pattern, "flow-boundary", start)
                        self.boundaries.append({"wrapper": wrapper, "mode": mode,
                                                "counter_before": value, "field": field,
                                                "value": int(pattern.search(self.lines[index])[1])})
                        due = mode == "active" and value == 1
                        self.score(label + "_COUNTER", holder, counter,
                                   1 if mode == "active" and not due else 0)
                        self.score(label + "_PULSE", holder, pulse, int(due))
                        self.score(label + "_STATE", holder, state, phase)
                        self.score(label + "_FRAME", "#fellthrough", "fa_boundary", 0)

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
            self.send("tag SandBeta remove advance")
            for state in (None, 0, 7, 20, -2147483648, 2147483647):
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
                    if value is None:
                        self.absent(label + "_BEFORE", "SandAlpha", counter)
                    else:
                        self.score(label + "_BEFORE", "SandAlpha", counter, value)
                    self.invoke()
                    self.score(label + "_STATE", "SandAlpha", "fa_phase", 0 if state is None else state)
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
            self.absent("RELOAD_DOES_NOT_CREATE_MISSING_COUNTER", "SandAlpha", counter)
            self.score("RELOAD_PRESERVES_ACTIVE_COUNTER", "SandBeta", counter, 1)
            self.invoke()
            self.score("POST_RELOAD_MISSING_COUNTER_REPAIRED", "SandAlpha", counter, 0)
            self.score("RELOAD_PRESERVES_ACTIVE_STATE", "SandBeta", "fa_phase", 10)
            self.seed("SandBeta", pulse, 0)
            self.invoke("SandBeta")
            self.score("POST_RELOAD_DUE_COUNTER_RESET", "SandBeta", counter, 0)
            self.score("POST_RELOAD_DUE_PULSE_FIRED", "SandBeta", pulse, 1)
            self.score("POST_RELOAD_STILL_ACTIVE", "SandBeta", "fa_phase", 10)
            self.boundary_probes("SandAlpha", "fa_phase", counter, pulse, 10, 20)


class GlobalFlowHarness(FlowHarness):
    """Exercise the canonical portable global holder without connecting players."""

    def run_client(self):
        verified_runtime_version(self.lines)
        self.load_test_chunk("minecraft:overworld")
        self.send("tick freeze")
        self.check("GLOBAL_NO_PLAYERS", "unless entity @a")
        holder, state, counter, pulse = (self.fixture[key] for key in ("holder", "state", "counter", "pulse"))
        invoke = lambda: self.send(f"function {self.fixture['root']}")
        for phase in (2, 7, -2147483648, 2147483647):
            for value in (None, 0, 1, -1, -2147483648, 2147483647):
                label = f"GLOBAL_INACTIVE_{phase}_{value}"
                self.seed(holder, state, phase)
                self.seed(holder, counter, value)
                self.seed(holder, pulse, 0)
                if value is None:
                    self.absent(label + "_BEFORE", holder, counter)
                else:
                    self.score(label + "_BEFORE", holder, counter, value)
                invoke()
                self.score(label + "_COUNTER", holder, counter, 0)
                self.score(label + "_STATE", holder, state, phase)
                self.score(label + "_NO_PULSE", holder, pulse, 0)
        # Missing/default state legitimately transitions to Active in this
        # fixture. Assert that path rather than pretending it stays inactive.
        for phase in (None, 0):
            label = f"GLOBAL_DEFAULT_{phase}"
            self.seed(holder, state, phase)
            self.seed(holder, counter, None)
            self.seed(holder, pulse, 0)
            if phase is None:
                self.absent(label + "_STATE_BEFORE", holder, state)
            self.absent(label + "_COUNTER_BEFORE", holder, counter)
            invoke()
            self.score(label + "_ACTIVE", holder, state, 1)
            self.score(label + "_FIRST_COUNTER", holder, counter, 1)
            self.score(label + "_FIRST_NO_PULSE", holder, pulse, 0)
            invoke()
            self.score(label + "_DUE_COUNTER", holder, counter, 0)
            self.score(label + "_DUE_PULSE", holder, pulse, 1)
        self.seed(holder, counter, 1)
        self.seed(holder, pulse, 0)
        start = len(self.lines)
        self.send("reload")
        self.wait_for(RELOAD_START, "global-active-reload", start)
        self.score("GLOBAL_RELOAD_ACTIVE_STATE", holder, state, 1)
        self.score("GLOBAL_RELOAD_ACTIVE_COUNTER", holder, counter, 1)
        self.score("GLOBAL_RELOAD_NO_PULSE", holder, pulse, 0)
        invoke()
        self.score("GLOBAL_RELOAD_DUE_COUNTER", holder, counter, 0)
        self.score("GLOBAL_RELOAD_DUE_PULSE", holder, pulse, 1)
        self.seed(holder, state, 7)
        self.seed(holder, counter, None)
        start = len(self.lines)
        self.send("reload")
        self.wait_for(RELOAD_START, "global-missing-reload", start)
        self.absent("GLOBAL_RELOAD_MISSING_COUNTER", holder, counter)
        invoke()
        self.score("GLOBAL_RELOAD_REPAIRS_COUNTER", holder, counter, 0)
        self.seed(holder, state, 1)
        self.seed(holder, pulse, 0)
        invoke()
        self.score("GLOBAL_REENTRY_FIRST", holder, counter, 1)
        self.score("GLOBAL_REENTRY_NO_PULSE", holder, pulse, 0)
        invoke()
        self.score("GLOBAL_REENTRY_DUE", holder, counter, 0)
        self.score("GLOBAL_REENTRY_PULSE", holder, pulse, 1)
        self.boundary_probes(holder, state, counter, pulse, 1, 7)
        self.check("GLOBAL_STILL_NO_PLAYERS", "unless entity @a")


def portable_global_fixture(resources):
    """Read canonical IDs from the exact compiler export, for either lowering."""
    functions = {path: bytes(data).decode() for path, data in resources.items()
                 if path.endswith(".mcfunction")}
    candidates = [(path, line) for path, body in functions.items() for line in body.splitlines()
                  if line.startswith("execute unless score #") and "run scoreboard players set" in line
                  and "__sf_" in line]
    if len(candidates) != 1:
        raise ValueError(f"expected one global interval reset, got {len(candidates)}")
    path, line = candidates[0]
    holder, state = line.split()[3:5]
    counter = line.split("run scoreboard players set ")[1].split()[1]
    namespace = path.split('/')[1]
    root = namespace + ':' + path.split('/function/')[1].removesuffix('.mcfunction')
    pulse_line = functions[f"data/{namespace}/function/world_tick.mcfunction"].split()
    if pulse_line[:3] != ["scoreboard", "players", "add"] or pulse_line[3] != holder:
        raise ValueError("unexpected global pulse fixture")
    return {"root": root, "holder": holder, "state": state, "counter": counter, "pulse": pulse_line[4]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    source = parser.add_mutually_exclusive_group()
    source.add_argument("--fixture", type=Path, help="Use an exact preserved Rust export; do not invoke Cargo")
    source.add_argument("--portable-fixture", type=Path, help="Use exact portable resources for the zero-player global matrix")
    parser.add_argument("--compare-evidence", type=Path,
                        help="Require checks and result/success boundaries to match a prior native run")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    dump = root / "fixture.json"
    if args.fixture or args.portable_fixture:
        dump.write_bytes((args.fixture or args.portable_fixture).read_bytes())
    else:
        subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "state_flow_runtime",
            "--", "--ignored", "--exact", "dump_state_flow_pack"], cwd=ROOT,
            env=dict(os.environ, SAND_STATE_FLOW_DUMP_PATH=str(dump)), check=True)
    exported = json.loads(dump.read_text())
    pack = root / "pack"
    pack.mkdir()
    if args.portable_fixture:
        fixture = portable_global_fixture(exported)
        for resource, data in exported.items():
            path = pack / resource
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(bytes(data))
        root_function = fixture["root"]
    else:
        fixture = exported
        (pack / "pack.mcmeta").write_text(json.dumps({"pack": {"pack_format": 107,
            "min_format": 107, "max_format": 107, "description": "Sand StateFlow gameplay audit"}}))
        for record in fixture["records"]:
            path = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(record["content"])
        root_function = "flow_audit:" + fixture["root"]
    functions = pack / "data/flow_audit/function"
    functions.mkdir(parents=True, exist_ok=True)
    (functions / "probe_plain.mcfunction").write_text(f"function {root_function}\n")
    (functions / "probe_returned.mcfunction").write_text(f"return run function {root_function}\nscoreboard players set #fellthrough fa_boundary 1\n")
    digest = hashlib.sha256()
    for path in sorted(pack.rglob("*")):
        if path.is_file():
            digest.update(path.relative_to(pack).as_posix().encode() + b"\0")
            digest.update(path.read_bytes() + b"\0")
    options = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar.resolve()),
        java=args.java, timeout=90, output=str(root / "logs"), op_player=None, client_command=None)
    harness = (GlobalFlowHarness if args.portable_fixture else FlowHarness)(options)
    harness.fixture, harness.checks, harness.boundaries, harness.scores = fixture, [], [], []
    passed = False
    try:
        harness.run()
        if args.compare_evidence:
            baseline = json.loads(args.compare_evidence.read_text())
            assert baseline["passed"], "baseline native run did not pass"
            assert baseline["checks"] == harness.checks, "native gameplay checks differ"
            assert baseline["boundaries"] == harness.boundaries, "native result/success boundaries differ"
            assert baseline["scores"] == harness.scores, "native score existence/value snapshots differ"
        passed = True
    finally:
        (root / "evidence.json").write_text(json.dumps({"minecraft": "26.2", "passed": passed,
            "scope": "zero-player global" if args.portable_fixture else "two real passive players",
            "fixture_sha256": hashlib.sha256(dump.read_bytes()).hexdigest(),
            "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "server_jar_sha256": hashlib.sha256(args.jar.read_bytes()).hexdigest(),
            "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "port": harness.port,
            "pack_sha256": digest.hexdigest(), "checks": harness.checks,
            "boundaries": harness.boundaries, "scores": harness.scores}, indent=2) + "\n")
    print("EVIDENCE", root / "evidence.json")


if __name__ == "__main__":
    main()
