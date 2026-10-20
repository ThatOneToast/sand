#!/usr/bin/env python3
"""Prove overlapping archetype acknowledgements in both orders on vanilla 26.2."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

from measure_archetype_idle import verified_runtime_version
from player_validation_harness import PlayerHarness
from validate_player_joins import write_pack
from vanilla_reload_harness import RELOAD_START

ROOT = Path(__file__).resolve().parents[1]


class DirtyHarness(PlayerHarness):
    def run_client(self):
        verified_runtime_version(self.lines)
        functions = {r["path"]: r["content"] for r in self.fixture["records"]
                     if r["dir"] == "function" and r["namespace"] == "dirty_shared"}
        roots = []
        for attribute in ("attack_damage", "armor"):
            path = next(path for path, body in functions.items()
                        if "/property/" in path and f"minecraft:{attribute} " in body)
            roots.append(path.split("/property/")[0])
        pending = []
        causes = []
        for root in roots:
            body = functions[f"{root}/acknowledge/0"]
            pending.append(body.splitlines()[0].split()[4])
            causes.append(body.splitlines()[1].split()[4])
        self.send("forceload add 0 0")
        self.eventually("CHUNK", "if loaded 0 100 0")
        with self.clients():
            self.send("tick freeze")
            self.send('summon minecraft:zombie 0 100 0 {Tags:["dirty_probe"],NoAI:1b,NoGravity:1b,Invulnerable:1b}')
            entity = "@e[tag=dirty_probe,limit=1]"
            for root in roots:
                self.send(f"execute as {entity} run function dirty_shared:{root}/initialize")
            self.send("scoreboard objectives add dirty_result dummy")
            for root in roots:
                self.send(f"execute as {entity} run function dirty_shared:{root}/reconcile")
            for iteration, order in enumerate(((0, 1), (1, 0), (0, 1), (1, 0))):
                value = iteration + 3
                if iteration == 2:
                    start = len(self.lines)
                    self.send("reload")
                    self.wait_for(RELOAD_START, "reload", start)
                self.send(f"scoreboard players set {entity} {self.fixture['level']} {value}")
                self.send(f"scoreboard players set {entity} {self.fixture['dirty']} 1")
                first, second = order
                self.send(f"execute as {entity} run function dirty_shared:{roots[first]}/reconcile")
                self.check(f"{iteration}_first_ack", f"if score {entity} {pending[first]} matches 0")
                self.check(f"{iteration}_second_retains_generation", f"if score {entity} {pending[second]} matches 1")
                self.send(f"execute as {entity} run function dirty_shared:{roots[second]}/reconcile")
                self.check(f"{iteration}_second_ack", f"if score {entity} {pending[second]} matches 0")
                for attribute in ("attack_damage", "armor"):
                    self.send(f"execute store result score #{attribute} dirty_result run attribute {entity} minecraft:{attribute} base get 1")
                    self.check(f"{iteration}_{attribute}", f"if score #{attribute} dirty_result matches {value}")
            for cause in (1, -7):
                self.send(f"scoreboard players set {entity} {pending[0]} 0")
                self.send(f"scoreboard players set {entity} {causes[0]} {cause}")
                self.send(f"execute as {entity} run function dirty_shared:{roots[0]}/reconcile")
                self.check(f"cause_only_{cause}", f"if score {entity} {causes[0]} matches 0")
            for second_first in (False, True):
                # Reset the crossing, then let a real transition callback write
                # shared State after the refresh/ack phase. No generated edits.
                self.send(f"scoreboard players set {entity} {self.fixture['level']} 6")
                self.send(f"scoreboard players set {entity} {self.fixture['dirty']} 1")
                for root in roots:
                    self.send(f"execute as {entity} run function dirty_shared:{root}/reconcile")
                self.send(f"scoreboard players set {entity} {self.fixture['level']} 10")
                self.send(f"scoreboard players set {entity} {self.fixture['dirty']} 1")
                if second_first:
                    self.send(f"execute as {entity} run function dirty_shared:{roots[1]}/reconcile")
                self.send(f"execute as {entity} run function dirty_shared:{roots[0]}/reconcile")
                prefix = f"callback_{second_first}"
                self.check(prefix + "_wrote", f"if score {entity} {self.fixture['level']} matches 11")
                for index, objective in enumerate(pending):
                    self.check(prefix + f"_generation_{index}", f"if score {entity} {objective} matches 1")
                self.send(f"execute store result score #attack dirty_result run attribute {entity} minecraft:attack_damage base get 1")
                self.check(prefix + "_before_next_refresh", "if score #attack dirty_result matches 10")
                self.send(f"execute as {entity} run function dirty_shared:{roots[1]}/reconcile")
                self.check(prefix + "_first_still_pending", f"if score {entity} {pending[0]} matches 1")
                self.send(f"execute as {entity} run function dirty_shared:{roots[0]}/reconcile")
                for attribute in ("attack_damage", "armor"):
                    self.send(f"execute store result score #{attribute} dirty_result run attribute {entity} minecraft:{attribute} base get 1")
                    self.check(prefix + attribute, f"if score #{attribute} dirty_result matches 11")
                for index, objective in enumerate(pending):
                    self.check(prefix + f"_consumed_{index}", f"if score {entity} {objective} matches 0")
        (self.output / "evidence.json").write_text(json.dumps({
            "minecraft": "26.2", "complete": True,
            "assertions": 40, "orders": [[0, 1], [1, 0]],
            "reload_repeated": True, "callback_generation_orders": 2,
            "cause_only_values": [1, -7],
        }, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", required=True)
    parser.add_argument("--timeout", type=float, default=90)
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = "26.2", None, None
    with tempfile.TemporaryDirectory(prefix="sand-dirty-fixture-") as directory:
        root = Path(directory)
        dump = root / "fixture.json"
        subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "archetype_dirty_bookkeeping",
                        "--", "--ignored", "--exact", "dump_dirty_bookkeeping_pack"], cwd=ROOT,
                       env=dict(os.environ, SAND_DIRTY_DUMP_PATH=str(dump)), check=True)
        fixture = json.loads(dump.read_text())
        write_pack(root / "pack", fixture["records"])
        args.pack = str(root / "pack")
        harness = DirtyHarness(args)
        harness.fixture = fixture
        harness.run()
        print(f"PASSED: overlapping dirty ownership; evidence: {harness.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
