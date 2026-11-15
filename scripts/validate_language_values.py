#!/usr/bin/env python3
"""Validate language runtime operands with two real players on vanilla 26.2.

Uses the existing jar-derived passive client. This proves server-side inventory,
player properties, dynamic movement/damage, range guards and argument ordering;
it does not test rendered UI or interactive controls.
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys

from validate_world_gameplay import WorldGameplayHarness
from vanilla_reload_harness import ValidationFailure


class ValuesHarness(WorldGameplayHarness):
    def prepare(self):
        super().prepare()
        with (self.server / "server.properties").open("a") as properties:
            properties.write("max-players=2\n")

    def run_client(self):
        clients = []
        logs = []
        try:
            for name in ("NumericA", "NumericB"):
                log = (self.output / f"{name}.log").open("w")
                logs.append(log)
                start = len(self.lines)
                client = subprocess.Popen([sys.executable,
                    str(Path(__file__).parent / "mc_validation/minimal_join_client.py"),
                    "127.0.0.1", str(self.port), "776", name, "20"],
                    stdout=log, stderr=subprocess.STDOUT)
                clients.append(client)
                self.wait_for(re.compile(re.escape(name) + r" joined the game"), "player-join", start)
                self.send(f"gamemode creative {name}")
            self.send("execute as NumericA run function runtime_values:player_a")
            self.send("execute as NumericB run function runtime_values:player_b")
            self.send("execute store result score #a numeric_probe run clear NumericA minecraft:bread 0")
            self.send("execute store result score #b numeric_probe run clear NumericB minecraft:bread 0")
            self.check("PLAYER_A_ITEMS", "if score #a numeric_probe matches 9")
            self.check("PLAYER_B_ITEMS", "if score #b numeric_probe matches 3")
            self.check("PLAYER_PROPERTIES", "if score #property numeric_probe matches 1")
            for name, expected in (("NumericA", 5), ("NumericB", 3)):
                self.send(f"execute store result score #xp numeric_probe run experience query {name} points")
                self.check(f"{name}_EXPERIENCE", f"if score #xp numeric_probe matches {expected}")
            self.load_test_chunk("minecraft:overworld")
            self.send('summon minecraft:pig 0 70 0 {NoAI:1b,Tags:["numeric_probe"]}')
            self.send("function runtime_values:mob")
            try:
                self.check("MOB_VALUES", "if score #mob numeric_probe matches 1")
            except ValidationFailure:
                self.send('data get entity @e[tag=numeric_probe,limit=1]')
                function_dir = Path(self.args.pack) / "data/runtime_values/function"
                holders = set()
                for path in function_dir.rglob("*.mcfunction"):
                    holders.update(re.findall(r"#sand_[^\s]+", path.read_text()))
                for holder in sorted(holders):
                    self.send(f"scoreboard players list {holder}")
                start = len(self.lines)
                self.send("say __NUMERIC_DIAGNOSTICS_DONE__")
                self.wait_for(self.marker_pattern("__NUMERIC_DIAGNOSTICS_DONE__"), "diagnostics", start)
                raise
            for client in clients:
                if client.wait(timeout=30) != 0:
                    raise RuntimeError("passive player client failed; inspect retained client logs")
        finally:
            for client in clients:
                if client.poll() is None:
                    client.terminate()
                    client.wait(timeout=10)
            for log in logs:
                log.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    root = options.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    fixture = Path(__file__).resolve().parents[1] / "sand-lang/tests/fixtures/runtime_values.sand"
    subprocess.run([str(options.binary.resolve()), "lang", "build", str(fixture), "--output", str(root / "pack")], check=True, timeout=60)
    args = argparse.Namespace(version="26.2", pack=str(root / "pack"), jar=options.jar,
        java=options.java, timeout=120, output=str(root / "evidence"), op_player=None, client_command=None)
    harness = ValuesHarness(args)
    harness.run()
    print(f"PASSED: runtime operands and player isolation on vanilla 26.2; log: {harness.log_path}")


if __name__ == "__main__":
    main()
