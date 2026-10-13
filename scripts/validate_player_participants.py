#!/usr/bin/env python3
"""Verify real-player attacker observations and equipment snapshots on 26.2.

Uses normal advancement and tick dispatch, including real deaths and composed
participants across dimensions, rapid alternating-attacker hits in one synchronous server
invocation, and attacking-player weapon snapshots. Client attack-input packets and
mixed melee/projectile combat remain outside this audit.
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

    def participant_tags(self):
        root = Path(self.args.pack) / "data/paudit/function"
        text = "\n".join(path.read_text() for path in root.rglob("*.mcfunction"))
        tags = sorted(set(re.findall(r"__sand_observed_[0-9a-f]+", text)))
        if not tags:
            raise ValidationFailure("participant-cleanup", "no participant tags found in the fixture")
        return tags

    def prepare(self):
        super().prepare()
        # A separate harness-owned input pack makes the rapid sequence one
        # synchronous server invocation. It never invokes Sand's handlers or
        # grants advancements: actual damage must trigger the generated path.
        controls = self.server / "world/datapacks/paudit_controls"
        functions = controls / "data/paudit_controls/function"
        functions.mkdir(parents=True)
        (controls / "pack.mcmeta").write_text((Path(self.args.pack) / "pack.mcmeta").read_text())
        commands = ["execute store success score #rapid_clock_start paudit_check store result score #rapid_start paudit_check run time query gametime"]
        for count, actor in enumerate((0, 1, 0), 1):
            for field in ("attacker_uuid", "attacker_b_uuid"):
                commands.append(f"data remove storage paudit:audit state.{field}")
            commands.append(f"execute store success score #rapid_hit_{count} paudit_check run damage SandAlpha {count} minecraft:generic by @e[tag=paudit_actor_{actor},limit=1]")
            for field in ("attacker_uuid", "attacker_b_uuid"):
                commands.append(f"data modify storage paudit:oracle rapid_{count}_{field} set from storage paudit:audit state.{field}")
            for objective in ("paudit_att1", "paudit_att2"):
                for label, player in (("alpha", "SandAlpha"), ("beta", "SandBeta")):
                    commands.append(f"scoreboard players operation #rapid_{label}_{count} {objective} = {player} {objective}")
            absent = " ".join(f"unless entity @e[tag={tag}]" for tag in self.participant_tags())
            for dimension in ("overworld", "the_nether", "the_end"):
                commands.append(f"execute store success score #rapid_clean_{count}_{dimension} paudit_check in minecraft:{dimension} {absent}")
        commands.append("execute store success score #rapid_clock_end paudit_check store result score #rapid_end paudit_check run time query gametime")
        (functions / "rapid_hits.mcfunction").write_text("\n".join(commands) + "\n")

    def check_cleanup(self):
        for dimension in ("overworld", "the_nether", "the_end"):
            for index, tag in enumerate(self.participant_tags()):
                self.check(f"CLEANUP_{dimension}_{index}", f"in minecraft:{dimension} unless entity @e[tag={tag}]")

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

    def verify_attacking_weapons(self):
        for player in self.players:
            for objective in ("paudit_wpn", "paudit_kwpn"):
                self.send(f"scoreboard players set {player} {objective} 0")
        for index, player in enumerate(self.players):
            self.equip(player, index)
            victim = f"@e[tag=paudit_victim_{index},limit=1]"
            self.send(f'summon minecraft:cow {index * 3} 80 2 {{Tags:["paudit_victim_{index}"],NoAI:1b,PersistenceRequired:1b}}')
            self.eventually(f"{player}_TARGET", f"if entity {victim}")
            for field in ("weapon_item", "kill_weapon_item"):
                self.send(f"data remove storage paudit:audit state.{field}")
            self.send(f"damage {victim} 2 minecraft:generic by {player}")
            self.eventually(f"{player}_HIT", f"if score {player} paudit_wpn matches 1")
            self.check(f"{player}_HIT_PRESENT", "if data storage paudit:audit state{weapon_present:1b}")
            self.equal_nbt(f"{player}_HIT_WEAPON", "storage paudit:audit state.weapon_item", f"storage paudit:oracle {player}_main")
            self.send(f"damage {victim} 100 minecraft:generic by {player}")
            self.eventually(f"{player}_KILL_TARGET", f"if score {player} paudit_kwpn matches 1")
            self.check(f"{player}_KILL_PRESENT", "if data storage paudit:audit state{kill_weapon_present:1b}")
            self.equal_nbt(f"{player}_KILL_WEAPON", "storage paudit:audit state.kill_weapon_item", f"storage paudit:oracle {player}_main")
            self.send(f"item replace entity {player} weapon.mainhand with minecraft:air")
            for field in ("weapon_item", "kill_weapon_item"):
                self.equal_nbt(f"{player}_{field}_AFTER_CLEAR", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_main")
            other = self.players[1 - index]
            self.check(f"{other}_KILL_TARGET_ISOLATED", f"if score {other} paudit_kwpn matches {index}")
            self.check_cleanup()

    def verify_composed_attacker(self, player, has_attacker=True):
        fields = ("compose_parent_uuid", "compose_child_uuid", "compose_sibling_uuid")
        for field in fields:
            self.send(f"data remove storage paudit:audit state.{field}")
        # Trigger only the fixture gate; normal tick and graph dispatch must
        # capture as this subject and forward the same binding to descendants.
        self.send(f"scoreboard players set {player} paudit_cmp_trg 1")
        self.eventually(f"{player}_COMPOSED", f"if score {player} paudit_cmp_trg matches 0")
        for field in fields:
            if has_attacker:
                self.equal_nbt(f"{player}_{field}", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_attacker")
            else:
                self.check(f"{player}_{field}_ABSENT", f"unless data storage paudit:audit state.{field}")
        self.check_cleanup()

    def verify_composed_attackers(self):
        for index, player in enumerate(self.players):
            self.send(f"damage {player} 2 minecraft:generic by @e[tag=paudit_actor_{index},limit=1]")
            self.verify_composed_attacker(player)

    def verify_rapid_attackers(self):
        # Observe completed game ticks rather than assuming wall-clock speed.
        # Increasing amounts then make all three synchronous hits valid.
        self.send("execute store success score #rapid_clock_ready paudit_check store result score #rapid_ready paudit_check run time query gametime")
        self.check("RAPID_COOLDOWN_CLOCK", "if score #rapid_clock_ready paudit_check matches 1")
        self.send("scoreboard players add #rapid_ready paudit_check 21")
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send("execute store success score #rapid_clock_ready paudit_check store result score #rapid_now paudit_check run time query gametime")
            if self.passed("RAPID_COOLDOWN", "execute if score #rapid_clock_ready paudit_check matches 1 if score #rapid_now paudit_check >= #rapid_ready paudit_check run say __SAND_WORLD_RAPID_COOLDOWN_OK__"):
                break
            time.sleep(0.1)
        else:
            raise ValidationFailure("rapid-cooldown", "game time did not advance enough to clear earlier damage")
        for objective in ("paudit_att1", "paudit_att2"):
            self.send(f"scoreboard players set SandAlpha {objective} 0")
            self.send(f"scoreboard players operation #rapid_beta {objective} = SandBeta {objective}")
        for count in (1, 2, 3):
            for field in ("attacker_uuid", "attacker_b_uuid"):
                self.send(f"data remove storage paudit:oracle rapid_{count}_{field}")
        self.send("function paudit_controls:rapid_hits")
        for phase in ("start", "end"):
            self.check(f"RAPID_CLOCK_{phase}", f"if score #rapid_clock_{phase} paudit_check matches 1")
        self.check("RAPID_SAME_GAME_TICK", "if score #rapid_start paudit_check = #rapid_end paudit_check")
        for count, actor in enumerate((0, 1, 0), 1):
            self.check(f"RAPID_HIT_{count}", f"if score #rapid_hit_{count} paudit_check matches 1")
            for objective in ("paudit_att1", "paudit_att2"):
                self.check(f"RAPID_{objective}_{count}", f"if score #rapid_alpha_{count} {objective} matches {count}")
                self.check(f"RAPID_BETA_{objective}_{count}", f"if score #rapid_beta_{count} {objective} = #rapid_beta {objective}")
            for field in ("attacker_uuid", "attacker_b_uuid"):
                self.equal_nbt(f"RAPID_{field}_{count}", f"storage paudit:oracle rapid_{count}_{field}", f"entity @e[tag=paudit_actor_{actor},limit=1] UUID")
            for dimension in ("overworld", "the_nether", "the_end"):
                self.check(f"RAPID_CLEAN_{count}_{dimension}", f"if score #rapid_clean_{count}_{dimension} paudit_check matches 1")
        self.check_cleanup()

    def verify_staged_fan_in(self):
        for index, player in enumerate(self.players):
            self.equip(player, index)
            self.send(f"scoreboard players set {player} paudit_fan 0")
        self.load_test_chunk("minecraft:the_nether")
        self.wait_for_attacker_absence("SandAlpha")
        self.send("execute in minecraft:the_nether run fill -2 79 -2 8 83 3 minecraft:stone")
        self.send("execute in minecraft:the_nether run fill -2 80 -2 8 82 3 minecraft:air")
        self.send("execute as @e[tag=paudit_actor_1,limit=1] in minecraft:the_nether run tp @s 3 80 2")
        self.send("execute in minecraft:the_nether run tp SandBeta 3.5 80 0.5")
        try:
            self.check("BETA_NETHER", "in minecraft:the_nether if entity @a[name=SandBeta,distance=0..]")
            self.check("BETA_ACTOR_NETHER", "in minecraft:the_nether if entity @e[tag=paudit_actor_1]")
            self.verify_damage_ready("BETA_NETHER_HIT", "minecraft:the_nether", "SandBeta", 1)
            # Nether chunk preparation can outlive vanilla's attacker memory.
            # Establish Alpha's real relation immediately before consuming it.
            self.verify_damage_ready("ALPHA_FAN_HIT", "minecraft:overworld", "SandAlpha", 0)
            self.send("tick freeze")
            for phase in (1, 2):
                for field in ("fan_alpha_uuid", "fan_beta_uuid", "fan_alpha_item", "fan_beta_item"):
                    self.send(f"data remove storage paudit:audit state.{field}")
                # Queue both players before advancing exactly one game tick.
                # Phase 1 fires the named source; phase 2 fires only the other
                # alternative, so the named source's evidence must be absent.
                for player in self.players:
                    self.send(f"scoreboard players set {player} paudit_cmp_trg {phase}")
                self.send("tick step 1")
                for player, label in zip(self.players, ("alpha", "beta")):
                    self.eventually(f"{player}_FAN_PHASE_{phase}", f"if score {player} paudit_fan matches {phase}")
                    expected = 1 if phase == 1 else 0
                    for objective in ("paudit_fan_ent", "paudit_fan_item"):
                        self.check(f"{player}_{objective}_{phase}", f"if score {player} {objective} matches {expected}")
                    for field, oracle in ((f"fan_{label}_uuid", "attacker"), (f"fan_{label}_item", "main")):
                        if phase == 1:
                            self.equal_nbt(f"{player}_{field}", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_{oracle}")
                        else:
                            self.check(f"{player}_{field}_SOURCE_ABSENT", f"unless data storage paudit:audit state.{field}")
                self.check_cleanup()
        finally:
            self.send("tick unfreeze")
            self.send("execute in minecraft:the_nether as @e[tag=paudit_actor_1,limit=1] in minecraft:overworld run tp @s 3 80 2")
            self.send("execute in minecraft:overworld run tp SandBeta 3.5 80 0.5")
        self.check("BETA_RETURNED", "in minecraft:overworld if entity @a[name=SandBeta,distance=0..]")
        self.verify_damage_ready("BETA_RETURN_HIT", "minecraft:overworld", "SandBeta", 1)

    def wait_for_attacker_absence(self, player):
        # Exercise the slow-world-setup boundary on every run: an earlier
        # combat relation must expire before fan-in establishes fresh input.
        self.check(f"{player}_BEFORE_EXPIRY", f"if entity @a[name={player}]")
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send("scoreboard players set #old_attacker paudit_check 0")
            self.send(f"execute as {player} on attacker run scoreboard players set #old_attacker paudit_check 1")
            phase = f"{player}_PRIOR_ATTACKER_EXPIRED"
            if self.passed(phase, f"execute if score #old_attacker paudit_check matches 0 run say __SAND_WORLD_{phase}_OK__"):
                return
            time.sleep(0.1)
        raise ValidationFailure("attacker-expiry", f"earlier attacker relation did not expire for {player}")

    def verify_damage_ready(self, phase, dimension, player, actor):
        # Entity position changes before the client acknowledges loading.
        # Retry rejected damage until one hit succeeds, then stop. Both the
        # outbound and return teleports must pass this readiness boundary.
        # It also establishes a fresh attacker relation after slow world setup.
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send(f"execute store success score #dimension_hit paudit_check in {dimension} run damage {player} 2 minecraft:generic by @e[tag=paudit_actor_{actor},limit=1]")
            if self.passed(phase, f"execute if score #dimension_hit paudit_check matches 1 run say __SAND_WORLD_{phase}_OK__"):
                return
            time.sleep(0.1)
        raise ValidationFailure("damage-readiness", f"{phase}: no accepted hit for {player} in {dimension}")

    def verify_deaths(self):
        for player in self.players:
            self.send(f"scoreboard players set {player} paudit_kill 0")
        for index, player in enumerate(self.players):
            keep = index == 0
            self.send(f"gamerule minecraft:keep_inventory {str(keep).lower()}")
            self.equip(player, index)
            for field in ("killer_uuid", "bridge_killer_uuid", "bridge_weapon_item", "bridge_weapon_present"):
                self.send(f"data remove storage paudit:audit state.{field}")
            self.send(f"damage {player} 100 minecraft:generic by @e[tag=paudit_actor_{index},limit=1]")
            self.eventually(f"{player}_DEATH", f"if score {player} paudit_kill matches 1")
            for field in ("killer_uuid", "bridge_killer_uuid"):
                self.equal_nbt(f"{player}_{field}", f"storage paudit:audit state.{field}", f"storage paudit:oracle {player}_attacker")
            present = "1b" if keep else "0b"
            self.check(f"{player}_DEATH_WEAPON_PRESENCE", f"if data storage paudit:audit state{{bridge_weapon_present:{present}}}")
            if keep:
                self.equal_nbt(f"{player}_DEATH_WEAPON", "storage paudit:audit state.bridge_weapon_item", f"storage paudit:oracle {player}_main")
                self.send(f"item replace entity {player} weapon.mainhand with minecraft:air")
                self.equal_nbt(f"{player}_DEATH_WEAPON_FROZEN", "storage paudit:audit state.bridge_weapon_item", f"storage paudit:oracle {player}_main")
            else:
                # Vanilla drops inventory before the death advancement fires.
                # Absence here is event-time evidence, not the pre-hit weapon.
                self.check(f"{player}_DEATH_DROPPED", f"unless data entity {player} SelectedItem")
                self.check(f"{player}_NO_STALE_DEATH_ITEM", "unless data storage paudit:audit state.bridge_weapon_item")
            other = self.players[1 - index]
            self.check(f"{other}_DEATH_ISOLATED", f"if score {other} paudit_kill matches {index}")
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
                self.send(f"tag {player} add paudit_subject_{index}")
                self.equip(player, index)
            time.sleep(4)
            for player in self.players:
                self.check(f"{player}_SILENT", f"unless score {player} paudit_tracked matches -2147483648..2147483647")
                for objective in ("paudit_tracked", "paudit_att1", "paudit_att2"):
                    self.send(f"scoreboard players set {player} {objective} 0")
            for index, player in enumerate(self.players):
                self.send(f"damage {player} 2 minecraft:generic by @e[tag=paudit_actor_{index},limit=1]")
                self.verify_capture(player, 1)
                self.verify_composed_attacker(player)
                if index == 0:
                    # Beta has never been hit: it must not inherit Alpha's
                    # freshly captured attacker through shared scratch state.
                    self.verify_composed_attacker("SandBeta", has_attacker=False)
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
            self.verify_attacking_weapons()
            self.verify_composed_attackers()
            self.verify_rapid_attackers()
            self.verify_staged_fan_in()
            self.verify_deaths()
            print("PASSED: real attackers and killers, composed participants, tracked snapshots, isolation and reload", flush=True)


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
