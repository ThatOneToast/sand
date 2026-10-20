#!/usr/bin/env python3
"""Prove XP bookkeeping, snapshot timing, lifecycle and idle command cost on 26.2.

Generated functions are never patched. A separate tick tag freezes automatic
observations; explicit calls exercise consecutive samples deterministically.
The unchanged detector ownership audit covers normal tick dispatch and namespaces.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from measure_archetype_idle import TRACE_FILE, trace_counts, verified_runtime_version
from player_validation_harness import PlayerHarness
from validate_player_joins import JoinHarness, write_pack
from vanilla_reload_harness import RELOAD_START, ValidationFailure

ROOT = Path(__file__).resolve().parents[1]


class XpHarness(PlayerHarness):
    auto_respawn = True
    settle = JoinHarness.settle

    def prepare(self):
        super().prepare()
        write_pack(self.server / "world/datapacks/xp_level", self.records)
        with (self.server / "server.properties").open("a") as properties:
            properties.write("initial-disabled-packs=file/xp_level\n")

    def sync(self, name):
        start = len(self.lines)
        self.send(f"say {name}")
        self.wait_for(self.marker_pattern(name), name, start)

    def sample(self, player="@a"):
        self.send(f"execute as {player} at @s run function xp_level:__sand_xp_check")
        self.sync("SAMPLE_DONE")

    def score(self, player, objective, value):
        self.check(f"{player}_{objective}_{value}", f"if score {player} {objective} matches {value}")

    def reload_pack(self):
        start = len(self.lines)
        self.send("reload")
        self.wait_for(RELOAD_START, "reload", start)
        self.sync("XP_RELOADED")
        self.settle()

    def trace(self, players, phase):
        self.send("tick freeze")
        name = "one" if players == 1 else "two"
        start = len(self.lines)
        self.send(f"debug function xp_probe:{name}")
        self.sync("TRACE_DONE")
        paths = [m[1] for line in self.lines[start:] if (m := TRACE_FILE.search(line))]
        assert len(paths) == 1, paths
        self.send("tick unfreeze")
        content = (self.server / "debug" / paths[0]).read_text()
        assert f"__TRACE_COMPLETE_{name}__" in content
        counts = trace_counts(content)
        detector_count = sum(v for k, v in counts.items() if k.startswith("xp_level:"))
        assert detector_count == players * 3, counts
        assert counts[f"xp_probe:{name}"] == 2, counts
        (self.output / f"{phase}-{name}.txt").write_text(content)
        return {"players": players, "xp_evaluations": detector_count, "counts": counts,
                "phase": phase, "trace": f"{phase}-{name}.txt"}

    def death_and_orb_cases(self):
        self.load_test_chunk("minecraft:overworld")
        self.send("gamemode survival @a")
        self.send("fill 0 79 0 31 79 3 minecraft:stone")
        self.send("spawnpoint SandAlpha 17 80 1")
        self.send("tp SandBeta 30 80 1")
        for keep in (False, True):
            self.send(f"gamerule keep_inventory {str(keep).lower()}")
            self.send("tp SandAlpha 1 80 1")
            self.send("experience set SandAlpha 5 levels")
            self.send("experience set SandAlpha 0 points")
            self.settle()
            self.sample("SandAlpha")
            self.send("scoreboard players operation #before xp_calls = SandAlpha xp_calls")
            self.send("kill SandAlpha")
            self.eventually(f"RESPAWN_{keep}", "if entity @a[name=SandAlpha,nbt={Health:20.0f}]")
            self.settle()
            self.send("kill @e[type=minecraft:experience_orb]")
            expected = 5 if keep else 0
            self.send("execute store result score SandAlpha xp_actual run experience query SandAlpha levels")
            self.score("SandAlpha", "xp_actual", expected)
            self.sample("SandAlpha")
            self.score("SandAlpha", "__sx_xp_level.p", expected)
            self.check(f"NO_DEATH_LEVEL_UP_{keep}", "if score SandAlpha xp_calls = #before xp_calls")
            # A real vanilla pickup, with no /experience command between death
            # and the assertion. Respawn is away from the death-drop location.
            self.send('execute at SandAlpha run summon minecraft:experience_orb ~ ~ ~ {Value:100s}')
            self.eventually(f"ORB_GAIN_{keep}", f"if entity @a[name=SandAlpha,level={expected + 1}..]")
            self.send("execute store result score SandAlpha xp_actual run experience query SandAlpha levels")
            self.sample("SandAlpha")
            self.send("scoreboard players add #before xp_calls 1")
            self.check(f"ONE_PICKUP_EVENT_{keep}", "if score SandAlpha xp_calls = #before xp_calls")
            self.check(f"PICKUP_BASELINE_{keep}", "if score SandAlpha __sx_xp_level.p = SandAlpha xp_actual")
            self.settle()
            self.sample("SandAlpha")
            self.check(f"NO_STALE_EVENT_{keep}", "if score SandAlpha xp_calls = #before xp_calls")

    def run_client(self):
        verified_runtime_version(self.lines)
        for objective in ("xp_calls", "xp_first", "xp_second", "other_owner", "join_check", "xp_actual"):
            self.send(f"scoreboard objectives add {objective} dummy")
        self.send("scoreboard players set #sentinel other_owner 37")
        with self.clients():
            self.send("experience set @a 5 levels")
            self.settle()
            for objective in ("xp_calls", "xp_first", "xp_second"):
                self.send(f"scoreboard players set @a {objective} 0")
            # Faithful old layout: all four objectives dummy, retained Alpha
            # baseline; Beta is unseen. Install the generated pack while online.
            for role in ("c", "p", "d", "s"):
                self.send(f"scoreboard objectives add __sx_xp_level.{role} dummy")
            for role, value in (("c", 3), ("p", 3), ("d", 0), ("s", 1)):
                self.send(f"scoreboard players set SandAlpha __sx_xp_level.{role} {value}")
            self.send('datapack enable "file/xp_level" before "file/control"')
            self.settle()
            self.sample()
            for player in self.players:
                self.score(player, "xp_calls", 1 if player == "SandAlpha" else 0)
                self.score(player, "__sx_xp_level.p", 5)
            samples = [self.trace(1, "initialized"), self.trace(2, "initialized")]
            # Freeze player ticks: an earlier mutation must be observed now.
            # The first callback changes live XP to 9; later callbacks must
            # still see 7/5/2, and the next observation must see 9/7/2.
            self.send("tick freeze")
            self.send("tag SandAlpha add xp_mutate")
            self.send("function xp_probe:mutation_then_observe")
            self.sync("ATOMIC_SAMPLE_DONE")
            self.score("SandAlpha", "xp_calls", 2)
            self.score("SandAlpha", "xp_first", 1)
            self.score("SandAlpha", "__sx_xp_level.p", 7)
            self.score("SandAlpha", "__sx_xp_level.c", 7)
            self.send("execute store result score SandAlpha xp_actual run experience query SandAlpha levels")
            self.score("SandAlpha", "xp_actual", 9)
            self.sample("SandAlpha")
            self.score("SandAlpha", "xp_calls", 3)
            self.score("SandAlpha", "xp_second", 1)
            self.score("SandBeta", "xp_calls", 0)
            self.send("tick unfreeze")
            self.send("tag SandAlpha remove xp_mutate")
            self.send("experience set SandAlpha 2 levels")
            self.sample()
            self.score("SandAlpha", "xp_calls", 3)
            self.score("SandAlpha", "__sx_xp_level.p", 2)
            self.sample()
            self.score("SandAlpha", "xp_calls", 3)
            # Reload preserves the baseline and defers the event to observation.
            self.send("experience set SandAlpha 4 levels")
            self.reload_pack()
            self.score("SandAlpha", "__sx_xp_level.p", 2)
            self.score("SandAlpha", "xp_calls", 3)
            self.sample()
            self.score("SandAlpha", "xp_calls", 4)
            self.score("SandBeta", "__sx_xp_level.p", 5)
            self.score("#sentinel", "other_owner", 37)
            self.send("experience set @a 5 levels")
            self.sample()
            samples.extend([self.trace(1, "stable"), self.trace(2, "stable")])
        with self.clients():
            self.sample()
            self.score("SandAlpha", "xp_calls", 5)
            self.score("SandBeta", "xp_calls", 0)
            # Reconnect retains both previously sampled baselines.
            for player in self.players:
                self.score(player, "__sx_xp_level.p", 5)
            self.send("experience set SandBeta 7 levels")
            self.sample()
            self.score("SandBeta", "xp_calls", 1)
            self.death_and_orb_cases()
        self.send("tick unfreeze")
        (self.output / "evidence.json").write_text(json.dumps({
            "server_jar_sha256": hashlib.sha256(Path(self.args.jar).read_bytes()).hexdigest(),
            "export_records_sha256": hashlib.sha256(json.dumps(self.records, sort_keys=True).encode()).hexdigest(),
            "minecraft": "26.2", "metric": "native debug [C] evaluations, excludes wrapper; not MSPT",
            "samples": samples, "idle_unconditional_score_mutations_per_player": 1,
            "passed": ["retained old baseline", "silent first sample", "frozen same-cycle mutation",
                       "callback snapshot and next-observation reentry", "decrease", "idle",
                       "reload retains baseline and pending increase", "reconnect", "two players",
                       "unrelated objective untouched", "death/respawn with and without keep_inventory",
                       "real XP orb pickup after respawn"],
        }, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", required=True)
    parser.add_argument("--timeout", type=float, default=60)
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    with tempfile.TemporaryDirectory(prefix="sand-xp-level-") as directory:
        root = Path(directory)
        dump = root / "pack.json"
        subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "xp_level_bookkeeping",
                        "--", "--ignored", "--exact", "dump_xp_level_pack"], cwd=ROOT,
                       env=dict(os.environ, SAND_XP_DUMP_PATH=str(dump)), check=True)
        records = json.loads(dump.read_text())
        # No change to generated resources: separate control pack owns only its
        # own trace wrappers and a tick-tag override for deterministic sampling.
        control = [{"namespace": "minecraft", "dir": "tags/function", "path": "tick",
                    "ext": "json", "content": json.dumps({"replace": True, "values": []})}]
        for name, target in (("one", "@a[name=SandAlpha]"), ("two", "@a")):
            control.append({"namespace": "xp_probe", "dir": "function", "path": name,
                            "ext": "mcfunction", "content":
                            f"execute as {target} at @s run function xp_level:__sand_xp_check\nsay __TRACE_COMPLETE_{name}__"})
        control.append({"namespace": "xp_probe", "dir": "function", "path": "mutation_then_observe",
                        "ext": "mcfunction", "content":
                        "experience set SandAlpha 7 levels\nexecute as SandAlpha at @s run function xp_level:__sand_xp_check"})
        # Wrappers reference the pack enabled later; vanilla resolves calls
        # lazily. The empty tick override also keeps reload samples controlled.
        write_pack(root / "control", control)
        args.pack = str(root / "control")
        harness = XpHarness(args)
        harness.records = records
        try:
            harness.run()
        except ValidationFailure as error:
            print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}")
            return 1
        print(f"PASSED: XP lifecycle, snapshots and cost; evidence: {harness.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
