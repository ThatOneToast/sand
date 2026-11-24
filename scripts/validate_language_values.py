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
import time

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
                command = [sys.executable, str(Path(__file__).parent / "mc_validation/minimal_join_client.py"),
                           "127.0.0.1", str(self.port), "776", name, "20"]
                if name == "NumericA":
                    for token in ("SAND-JOURNAL-PROBE", "click_event", "copy_to_clipboard",
                                  "hover_event", "show_text", "Native text"):
                        command.extend(["--expect-system-chat", token])
                    command.extend(["--expect-particles", "10.25", "71", "20.5", "0.25", "0.5", "7"])
                    command.append("--expect-dialog-cycle")
                client = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
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
            for player, count in (("NumericA", 9), ("NumericB", 3)):
                self.send(f"execute as {player} run function runtime_values:inspect_inventory")
                self.check(f"INVENTORY_QUERY_{player}", f"if score {player} numeric_probe matches {count}")
                self.send(
                    f"execute store result score #remaining numeric_probe "
                    f"run clear {player} minecraft:bread 0"
                )
                self.check(f"INVENTORY_QUERY_KEEPS_ITEMS_{player}", f"if score #remaining numeric_probe matches {count}")
            self.check("STATE_IMPL_A", "if score #impl_a numeric_probe matches 1")
            self.check("STATE_IMPL_B", "if score #impl_b numeric_probe matches 1")
            self.check("PLAYER_PROPERTIES", "if score #property numeric_probe matches 1")
            for name, expected in (("NumericA", 5), ("NumericB", 3)):
                self.send(f"execute store result score #xp numeric_probe run experience query {name} points")
                self.check(f"{name}_EXPERIENCE", f"if score #xp numeric_probe matches {expected}")
            self.send("execute as NumericA at @s positioned 10.25 70 20.5 run function runtime_values:show_particles")
            self.send("execute as NumericA run function runtime_values:show_notice")
            self.send("execute as NumericA run function runtime_values:show_localized")
            self.check(
                "TRANSLATED_ITEM",
                'if data entity NumericA Inventory[{id:"minecraft:paper"}].components.'
                '"minecraft:custom_name".extra[{translate:"item.minecraft.paper"}]',
            )
            self.check("TRANSLATION_CLEANUP", "unless data storage runtime_values:__program_text p0_1_0_0_0")
            self.check(
                "ADVANCEMENT_ANY",
                "if entity @a[name=NumericA,advancements={runtime_values:first_find=true}]",
            )
            self.check(
                "ADVANCEMENT_ALL_PENDING",
                "if entity @a[name=NumericA,advancements={runtime_values:read_and_write=false}]",
            )
            self.send("execute as NumericA run function runtime_values:read_advancement_progress")
            self.check("ADVANCEMENT_CRITERION_QUERY", "if score NumericA numeric_probe matches 1")
            self.send("execute as NumericA run function runtime_values:read_advancement_handlers")
            self.check("ADVANCEMENT_NO_EARLY_HANDLERS", "if score NumericA numeric_probe matches 0")
            self.send("execute as NumericA run function runtime_values:give_handbook")
            self.check(
                "ADVANCEMENT_ALL_READY",
                "if entity @a[name=NumericA,advancements={runtime_values:read_and_write=true}]",
            )
            self.send("execute store result score #xp numeric_probe run experience query NumericA points")
            self.check("ADVANCEMENT_EXPERIENCE", "if score #xp numeric_probe matches 6")
            self.check(
                "ADVANCEMENT_PLAYER_ISOLATION",
                "if entity @a[name=NumericB,advancements={runtime_values:first_find=false,runtime_values:read_and_write=false}]",
            )
            for player, progress in (("NumericA", 2), ("NumericB", 0)):
                self.send(f"execute as {player} run function runtime_values:read_advancement_progress")
                self.check(f"ADVANCEMENT_QUERY_{player}", f"if score {player} numeric_probe matches {progress}")
            self.send("execute as NumericA run function runtime_values:give_handbook")
            for player, order in (("NumericA", 12), ("NumericB", 0)):
                self.send(f"execute as {player} run function runtime_values:read_advancement_handlers")
                self.check(f"ADVANCEMENT_HANDLERS_{player}", f"if score {player} numeric_probe matches {order}")
            book = 'Inventory[{id:"minecraft:written_book"}].components."minecraft:written_book_content"'
            self.check("WRITTEN_BOOK_PAGES", f"if data entity NumericA {book}.pages[1]")
            self.check(
                "WRITTEN_BOOK_NAVIGATION",
                f'if data entity NumericA {book}.pages[0].raw.extra[0].click_event{{action:"change_page",page:2}}',
            )
            self.load_test_chunk("minecraft:overworld")
            self.send('summon minecraft:pig 0 70 0 {NoAI:1b,Tags:["numeric_probe"]}')
            self.send("function runtime_values:mob")
            self.check("ENTITY_NBT_PATH", "if score #entity_nbt numeric_probe matches 1")
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
            self.check(
                "ADVANCEMENT_KILL_CREDIT",
                "if entity @a[name=NumericA,advancements={runtime_values:hunt_pig=true}] "
                "if entity @a[name=NumericB,advancements={runtime_values:hunt_pig=false}]",
            )
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
            self.send("execute as NumericA run function runtime_values:remove_inventory_sample")
            self.check("RUNTIME_ITEM_CLEAR", "if score #safe_clear numeric_probe matches 1")
            self.send("execute as NumericB run function runtime_values:inspect_inventory")
            self.check("RUNTIME_ITEM_CLEAR_ISOLATION", "if score NumericB numeric_probe matches 3")
            self.check_inventory_transfers()
            self.check_owner_huds()
        finally:
            self.send("tick unfreeze")
            self.send("execute in minecraft:overworld run teleport NumericB 0 80 0")


    def check_inventory_transfers(self):
        for player, count in (("NumericA", 5), ("NumericB", 7)):
            self.send(
                f"item replace entity {player} hotbar.0 with "
                f"minecraft:bread[minecraft:custom_data={{transfer_probe:9}}] {count}"
            )
            self.send(f"item replace entity {player} hotbar.1 with minecraft:air")

        def slot(player, index, count):
            entry = f'{{Slot:{index}b,id:"minecraft:bread",count:{count},components:'
            entry += '{"minecraft:custom_data":{transfer_probe:9}}}'
            return f"if data entity {player} Inventory[{entry}]"

        move = "execute as NumericA run function runtime_values:move_rations"
        self.send(move)
        self.check("TRANSFER_SPLIT_SOURCE", slot("NumericA", 0, 3))
        self.check("TRANSFER_SPLIT_DESTINATION", slot("NumericA", 1, 2))
        self.send(move)
        self.check("TRANSFER_MERGE_SOURCE", slot("NumericA", 0, 1))
        self.check("TRANSFER_MERGE_DESTINATION", slot("NumericA", 1, 4))
        self.send(move)
        self.check("TRANSFER_REJECT_SOURCE", slot("NumericA", 0, 1))
        self.check("TRANSFER_REJECT_DESTINATION", slot("NumericA", 1, 4))
        self.check("TRANSFER_OTHER_PLAYER_SOURCE", slot("NumericB", 0, 7))
        self.check("TRANSFER_OTHER_PLAYER_DESTINATION", "unless data entity NumericB Inventory[{Slot:1b}]")
        for player, state in (("NumericA", 21), ("NumericB", 0)):
            self.send(f"execute as {player} run function runtime_values:report_transfers")
            self.check(f"TRANSFER_CALLBACKS_{player}", f"if score {player} numeric_probe matches {state}")


    def check_owner_huds(self):
        namespace = "runtime_values:__sand_system/lifetime/field_status"

        def active_id(index):
            start = len(self.lines)
            pattern = re.compile(r"has the following contents: (\d+)$")
            self.send(f"data get storage {namespace}/hud_ledger active[{index}].instance")
            line = self.wait_for(pattern, "hud-identity", start)
            return f"{namespace}/hud_bar/{pattern.search(self.lines[line]).group(1)}"

        def value(bar, expected, label):
            self.send(f"execute store result score #hud_value numeric_probe run bossbar get {bar} value")
            self.check(label, f"if score #hud_value numeric_probe matches {expected}")

        def audience(bar, player):
            start = len(self.lines)
            self.send(f"bossbar get {bar} players")
            self.wait_for(re.compile(rf"has 1 player\(s\) currently online: {player}$"), f"hud-audience-{player}", start)

        for player in ("NumericA", "NumericB"):
            self.send(f"execute as {player} at @s run function runtime_values:begin_status")

        first, second = (active_id(index) for index in (0, 1))
        value(first, 25, "HUD_A_INITIAL")
        value(second, 25, "HUD_B_INITIAL")
        audience(first, "NumericA")
        audience(second, "NumericB")

        self.send("execute as NumericA run function runtime_values:update_status")
        start = len(self.lines)
        self.send("tick step 3")
        self.wait_for(re.compile("Stepping 3 ticks?"), "hud-refresh", start)
        for _ in range(20):
            self.send(f"execute store result score #hud_value numeric_probe run bossbar get {first} value")
            if self.passed("HUD_A_REFRESHED", "execute if score #hud_value numeric_probe matches 75 run say __SAND_WORLD_HUD_A_REFRESHED_OK__"):
                break
            time.sleep(0.05)
        else:
            raise ValidationFailure("hud-progress", "owner's State update did not reach the private bossbar")
        value(second, 25, "HUD_B_STAYS_PRIVATE")

        self.send("execute as NumericA run function runtime_values:end_status")
        self.send(f"execute store success score #hud_exists numeric_probe run bossbar get {first} value")
        self.check("HUD_A_CANCELLED", "if score #hud_exists numeric_probe matches 0")
        value(second, 25, "HUD_B_STILL_PRESENT")
        self.send("execute as NumericB run function runtime_values:end_status")
        self.send(f"execute store success score #hud_exists numeric_probe run bossbar get {second} value")
        self.check("HUD_B_CANCELLED", "if score #hud_exists numeric_probe matches 0")


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
