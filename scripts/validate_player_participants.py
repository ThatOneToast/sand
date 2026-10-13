#!/usr/bin/env python3
"""Verify real-player attacker observations and equipment snapshots on 26.2.

Uses normal advancement and tick dispatch. The remaining killer/weapon-event,
rapid-hit and composed-participant matrix in #265 is outside this audit.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys
import time

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import RELOAD_START, ValidationFailure


class PlayerParticipantHarness(PlayerHarness):
    def equal_nbt(self, name, actual, expected):
        # Missing data must fail before using data-modify's unchanged result.
        self.check(f"{name}_ACTUAL_EXISTS", f"if data {actual}")
        self.check(f"{name}_EXPECTED_EXISTS", f"if data {expected}")
        self.send(f"data get {actual}")
        self.send(f"data get {expected}")
        self.send(f"data modify storage paudit:oracle comparison set from {actual}")
        self.send(f"execute store success score #different paudit_check run data modify storage paudit:oracle comparison set from {expected}")
        self.check(f"{name}_EQUAL", "if score #different paudit_check matches 0")

    def check_cleanup(self):
        root = Path(self.args.pack) / "data/paudit/function"
        text = "\n".join(path.read_text() for path in root.rglob("*.mcfunction"))
        tags = set(re.findall(r"__sand_observed_[0-9a-f]+", text))
        if not tags:
            raise ValidationFailure("participant-cleanup", "no participant tags found in the fixture")
        for index, tag in enumerate(sorted(tags)):
            self.check(f"CLEANUP_{index}", f"unless entity @e[tag={tag}]")

    def equip(self, player, index):
        actor = f"@e[tag=paudit_actor_{index},limit=1]"
        items = ("diamond_sword", "shield", "diamond_helmet") if index == 0 else ("iron_axe", "shield", "iron_helmet")
        for slot, path, item, key in zip(
            ("weapon.mainhand", "weapon.offhand", "armor.head"),
            ("SelectedItem", "equipment.offhand", "equipment.head"), items,
            ("main", "off", "helmet"),
        ):
            self.send(f'item replace entity {player} {slot} with minecraft:{item}[minecraft:custom_data={{owner:"{player}",slot:"{key}"}}]')
            self.check(f"{player}_{key}_LIVE", f'if data entity {player} {path}{{id:"minecraft:{item}"}}')
            self.send(f"data modify storage paudit:oracle {player}_{key} set from entity {player} {path}")
        self.send(f"data modify storage paudit:oracle {player}_uuid set from entity {player} UUID")
        self.send(f"data modify storage paudit:oracle {player}_attacker set from entity {actor} UUID")
        self.send(f"data modify storage paudit:oracle {player}_actor_helmet set from entity {actor} equipment.head")

    def verify_capture(self, player, count, equipped=True):
        self.eventually(f"{player}_TRACKED", f"if score {player} paudit_tracked matches {count}")
        for counter in ("paudit_att1", "paudit_att2"):
            self.check(f"{player}_{counter}", f"if score {player} {counter} matches {count}")
        for field, expected in (
            ("subject_uuid", "uuid"), ("attacker_uuid", "attacker"), ("attacker_b_uuid", "attacker"),
            ("tracked_subject_uuid", "uuid"), ("tracked_attacker_uuid", "attacker"),
            ("tracked_helmet_item", "helmet"), ("tracked_attacker_helmet", "actor_helmet"),
        ):
            self.equal_nbt(f"{player}_{field}", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_{expected}")
        present = "1b" if equipped else "0b"
        self.check(f"{player}_PRESENCE", f"if data storage paudit:audit state{{tracked_weapon_present:{present},tracked_offhand_present:{present}}}")
        if equipped:
            for field, expected in (("tracked_weapon_item", "main"), ("tracked_offhand_item", "off")):
                self.equal_nbt(f"{player}_{field}", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_{expected}")
        self.check_cleanup()

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.send("difficulty normal")
        self.send("gamerule minecraft:natural_health_regeneration false")
        self.send("fill -2 79 -2 8 79 3 minecraft:stone")
        self.send("fill -2 80 -2 8 84 3 minecraft:air")
        # Sunlight wears a zombie's helmet even with NoAI/Invulnerable set.
        # Keep the independently recorded item unchanged during the scenario.
        self.send("fill -2 83 -2 8 83 3 minecraft:stone")
        self.send("function paudit:init")
        self.send("scoreboard objectives add paudit_check dummy")
        for index in range(2):
            actor = f"@e[tag=paudit_actor_{index},limit=1]"
            self.send(f'summon minecraft:zombie {index * 3} 80 2 {{Tags:["paudit_actor_{index}"],NoAI:1b,Invulnerable:1b,PersistenceRequired:1b}}')
            self.eventually(f"ACTOR_{index}", f"if entity {actor}")
            helmet = "chainmail_helmet" if index == 0 else "leather_helmet"
            self.send(f'item replace entity {actor} armor.head with minecraft:{helmet}[minecraft:custom_data={{actor:{index}}}]')
        with self.clients():
            for index, player in enumerate(self.players):
                self.send(f"tp {player} {index * 3 + 0.5} 80 0.5")
                self.equip(player, index)
            time.sleep(4)
            for player in self.players:
                self.check(f"{player}_SILENT", f"unless score {player} paudit_tracked matches -2147483648..2147483647")
                for objective in ("paudit_tracked", "paudit_att1", "paudit_att2"):
                    self.send(f"scoreboard players set {player} {objective} 0")
            for index, player in enumerate(self.players):
                self.send(f"damage {player} 2 minecraft:generic by @e[tag=paudit_actor_{index},limit=1]")
                self.verify_capture(player, 1)
                other = self.players[1 - index]
                other_count = index
                self.check(f"{other}_ISOLATED", f"if score {other} paudit_tracked matches {other_count}")
                # The captured compounds must not follow later live inventory edits.
                self.send(f"item replace entity {player} weapon.mainhand with minecraft:air")
                self.send(f"item replace entity {player} weapon.offhand with minecraft:air")
                for field, expected in (("tracked_weapon_item", "main"), ("tracked_offhand_item", "off")):
                    self.equal_nbt(f"{player}_{field}_FROZEN", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_{expected}")
            start = len(self.lines)
            self.send("reload")
            self.wait_for(RELOAD_START, "participant-reload", start)
            self.send("say __SAND_PARTICIPANTS_RELOADED__")
            self.wait_for(self.marker_pattern("__SAND_PARTICIPANTS_RELOADED__"), "participant-reload-complete", start)
            time.sleep(2)
            for player in self.players:
                self.check(f"{player}_RELOAD_BASELINE", f"if score {player} paudit_tracked matches 1")
            self.send("damage SandAlpha 2 minecraft:generic by @e[tag=paudit_actor_0,limit=1]")
            self.verify_capture("SandAlpha", 2, equipped=False)
            self.check("BETA_AFTER_ALPHA_EMPTY", "if score SandBeta paudit_tracked matches 1")
            print("PASSED: real advancement attackers, tracked participants, equipped/empty snapshots, isolation and reload", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, choices=["26.2"])
    parser.add_argument("--pack", required=True)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--output")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--java", default="java")
    args = parser.parse_args()
    args.op_player = None
    args.client_command = None
    harness = PlayerParticipantHarness(args)
    try:
        harness.run()
    except ValidationFailure as error:
        print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
        return 1
    print(f"PASSED: Minecraft {args.version} player participants; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
