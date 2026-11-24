#!/usr/bin/env python3
"""Validate language runtime operands with two real players on vanilla 26.2.

Uses the existing jar-derived passive client. This proves server-side inventory,
player properties, dynamic movement/damage, range guards and argument ordering;
it does not test rendered UI or interactive controls.
"""
import argparse
from pathlib import Path
import re
import shutil
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
            self.send(
                "execute store result score #trim numeric_probe "
                "run recipe give NumericA runtime_values:guard_trim"
            )
            self.check("SMITHING_TRIM_RECIPE", "if score #trim numeric_probe matches 1")
            self.send("execute as NumericA run function runtime_values:player_a")
            self.send("execute as NumericB run function runtime_values:player_b")
            self.send("execute store result score #a numeric_probe run clear NumericA minecraft:bread 0")
            self.send("execute store result score #b numeric_probe run clear NumericB minecraft:bread 0")
            self.check("PLAYER_A_ITEMS", "if score #a numeric_probe matches 9")
            self.check("PLAYER_B_ITEMS", "if score #b numeric_probe matches 3")
            self.check("STATE_IMPL_A", "if score #impl_a numeric_probe matches 1")
            self.check("STATE_IMPL_B", "if score #impl_b numeric_probe matches 1")
            self.check("PLAYER_PROPERTIES", "if score #property numeric_probe matches 1")
            self.send("execute as NumericA run function runtime_values:show_localized")
            self.check(
                "TRANSLATED_ITEM",
                'if data entity NumericA Inventory[{id:"minecraft:paper"}].components.'
                '"minecraft:custom_name".extra[{translate:"item.minecraft.paper"}]',
            )
            self.check("TRANSLATION_CLEANUP", "unless data storage runtime_values:__program_text p0_1_0_0_0")
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
            self.send("execute as NumericA run function runtime_values:store_first")
            self.send("execute as NumericB run function runtime_values:store_second")
            self.send("execute as NumericA run function runtime_values:player_maps/store_a")
            self.send("execute as NumericB run function runtime_values:player_maps/store_b")
            self.send("reload")
            self.send("execute as NumericA run function runtime_values:player_maps/check_a")
            self.send("execute as NumericB run function runtime_values:player_maps/check_b")
            self.check("PLAYER_COLLECTIONS_RELOAD_A", "if score #maps_a numeric_probe matches 1")
            self.check("PLAYER_COLLECTIONS_RELOAD_B", "if score #maps_b numeric_probe matches 1")
            self.send("execute as NumericA run function runtime_values:check_player_text_a")
            self.send("execute as NumericB run function runtime_values:check_player_text_b")
            self.check("PLAYER_TEXT_RELOAD_A", "if score #text_a numeric_probe matches 1")
            self.check("PLAYER_TEXT_RELOAD_B", "if score #text_b numeric_probe matches 1")
            self.send("execute as NumericA run function runtime_values:check_player_vector_a")
            self.send("execute as NumericB run function runtime_values:check_player_vector_b")
            self.check("PLAYER_VECTOR_RELOAD_A", "if score #vec_a numeric_probe matches 1")
            self.check("PLAYER_VECTOR_RELOAD_B", "if score #vec_b numeric_probe matches 1")
            self.check_owned_waits()
            self.send("execute as NumericA run function runtime_values:check_reference")
            self.check("REFERENCE_RELOAD", "if score #stable numeric_probe matches 1")
            self.send("execute in minecraft:the_nether run teleport NumericA 0 80 0")
            self.send("function runtime_values:resolve_references")
            self.send("execute store result score #emerald_a numeric_probe run clear NumericA minecraft:emerald 0")
            self.send("execute store result score #emerald_b numeric_probe run clear NumericB minecraft:emerald 0")
            self.check("REFERENCE_PLAYER_A", "if score #emerald_a numeric_probe matches 2")
            self.check("REFERENCE_PLAYER_B", "if score #emerald_b numeric_probe matches 3")
            self.check("REFERENCE_EMPTY", "unless score #empty numeric_probe matches 1")
            self.send("scoreboard objectives add reference_kills totalKillCount")
            self.send("scoreboard players set NumericA reference_kills 0")
            self.send("scoreboard players set NumericB reference_kills 0")
            self.send('summon minecraft:pig 0 70 0 {NoAI:1b,Tags:["reference_victim"]}')
            self.send("function runtime_values:attributed_hit")
            self.check("REFERENCE_KILL_CREDIT_A", "if score NumericA reference_kills matches 1")
            self.check("REFERENCE_KILL_CREDIT_B", "if score NumericB reference_kills matches 0")
            start = len(self.lines)
            self.send("schedule function runtime_values:check_dead_reference 25t")
            self.wait_for(self.marker_pattern("__SAND_REFERENCE_DEAD_OK__"), "dead-reference", start)
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

    def check_owned_waits(self):
        self.send("scoreboard objectives add wait_origin dummy")
        self.send("scoreboard objectives add wait_clock dummy")
        for name in ("NumericA", "NumericB"):
            self.send(f"execute in minecraft:overworld run teleport {name} 0 80 0")
            self.send(f"scoreboard players set {name} wait_origin 0")
        self.send("tick freeze")
        try:
            self.send("execute store result score #zero wait_clock run time query gametime")
            self.send("execute as NumericA at @s run function runtime_values:player_waits/check_invalid")
            self.check("WAIT_BAD_DURATION_RESUMES_IMMEDIATELY", "if score #wait_invalid numeric_probe matches 1")
            self.send("execute as NumericB at @s run function runtime_values:player_waits/queue_seconds")
            for name in ("NumericA", "NumericB"):
                self.send(f"execute as {name} at @s run function runtime_values:player_waits/queue_cancellable")
            self.send("execute as NumericA at @s run function runtime_values:player_waits/cancel")
            self.send("execute as NumericA at @s run function runtime_values:player_waits/queue_a")
            self.send("execute as NumericB at @s run function runtime_values:player_waits/queue_b")
            self.send("execute in minecraft:the_nether run teleport NumericB 0 80 0")
            start = len(self.lines)
            self.send("tick step 40")
            for label, owner in (
                ("A", "NumericA"), ("A2", "NumericA"), ("B", "NumericB"),
                ("CANCELLABLE", "NumericB"), ("SECONDS", "NumericB"),
            ):
                marker = f"__SAND_WAIT_{label}_DONE__"
                self.wait_for(re.compile(rf"\[{owner}\] {marker}$"), "wait-completion", start)
            for timer, ticks in (("A", 12), ("A2", 18), ("B", 23), ("seconds", 20)):
                self.send(f"scoreboard players operation #{timer} wait_clock -= #zero wait_clock")
                self.check(f"WAIT_DELAY_{timer.upper()}", f"if score #{timer} wait_clock matches {ticks}")
            self.send("execute as NumericA run function runtime_values:player_waits/check_a")
            self.send("execute as NumericB run function runtime_values:player_waits/check_b")
            self.check("WAIT_A_OWNED_AND_CANCELLED", "if score #wait_a numeric_probe matches 1")
            self.check("WAIT_B_OWNED_AND_NOT_CANCELLED", "if score #wait_b numeric_probe matches 1")
            self.check("WAIT_A_ORIGINAL_DIMENSION", "if score NumericA wait_origin matches 2")
            self.check("WAIT_B_ORIGINAL_DIMENSION", "if score NumericB wait_origin matches 1")
        finally:
            self.send("tick unfreeze")
            self.send("execute in minecraft:overworld run teleport NumericB 0 80 0")


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
    source = root / "source"
    source.mkdir()
    (source / "pack.sand").write_text(fixture.read_text() + "\nmod player_maps;\nmod player_waits;\n")
    shutil.copyfile(fixture.with_name("player_maps.sand"), source / "player_maps.sand")
    shutil.copyfile(fixture.with_name("player_waits.sand"), source / "player_waits.sand")
    binary = str(options.binary.resolve())
    subprocess.run([binary, "lang", "add", "collections", str(source)], check=True, timeout=60)
    subprocess.run([binary, "lang", "build", str(source), "--output", str(root / "pack")], check=True, timeout=60)
    args = argparse.Namespace(version="26.2", pack=str(root / "pack"), jar=options.jar,
        java=options.java, timeout=120, output=str(root / "evidence"), op_player=None, client_command=None)
    harness = ValuesHarness(args)
    harness.run()
    print(f"PASSED: runtime operands and player isolation on vanilla 26.2; log: {harness.log_path}")


if __name__ == "__main__":
    main()
