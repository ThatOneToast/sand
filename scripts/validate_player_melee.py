#!/usr/bin/env python3
"""Bounded protocol-776 reciprocal melee through real participant dispatch.

Requires the existing participant_audit export. Setup is server-directed;
movement and both nonfatal attacks originate in connected clients. No /damage,
generated handler invocation, advancement grant, or automatic attack retry.
"""
import argparse
import json
import struct
import sys
import time
import uuid

from mc_validation.scripted_input import atomic_json
from validate_player_participants import PlayerParticipantHarness
from vanilla_reload_harness import ValidationFailure


class InconclusiveSetup(ValidationFailure):
    """Native input preconditions/action acceptance could not be established."""


class PlayerMeleeHarness(PlayerParticipantHarness):
    def client_options(self, player):
        directory = self.output / f"{player}-control"
        directory.mkdir(mode=0o700)
        return ["--control-dir", str(directory)]

    def status(self, player):
        path = self.output / f"{player}-control/status.json"
        if not path.exists():
            return None
        status = json.loads(path.read_text())
        if status["error"]:
            raise ValidationFailure("client-control", f"{player}: {status['error']}")
        return status

    def wait_status(self, player, predicate, phase):
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.poll_health()
            status = self.status(player)
            if status is not None and predicate(status):
                return status
            time.sleep(0.05)
        self.poll_health()
        raise InconclusiveSetup(phase, f"{player}: bounded client readiness/ack wait expired")

    def poll_health(self):
        """Failures in either session or the server are never setup timeouts."""
        if self.process is None or self.process.poll() is not None:
            raise ValidationFailure("melee-server", "server exited during melee polling")
        if any(process.poll() is not None for process in getattr(self, "client_processes", ())):
            raise ValidationFailure("player-client", "a client exited during melee polling")
        for player in self.players:
            self.status(player)  # Raises on a published protocol/control error.

    def command(self, player, action, **fields):
        status = self.wait_status(player, lambda value: value["ready"], "client-ready")
        sequence = status["sent_id"] + 1
        atomic_json(self.output / f"{player}-control/command.json", {
            "id": sequence, "action": action,
            "deadline": time.monotonic() + min(20, self.args.timeout), **fields,
        })
        self.wait_status(player, lambda value: value["sent_id"] == sequence, "command-sent")
        print(f"SENT ONLY: {player} {action} command {sequence}", flush=True)

    def native_eventually(self, name, condition, *, refresh=None):
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.poll_health()
            if refresh is not None:
                refresh()
            marker = f"__SAND_WORLD_{name}_OK__"
            # Send/server-exit/marker failures propagate as FAILED. Only a
            # healthy server response lacking the condition may be inconclusive.
            satisfied = self.passed(name, f"execute {condition} run say {marker}")
            self.poll_health()
            if satisfied:
                return
            time.sleep(0.1)
        self.poll_health()
        raise InconclusiveSetup(name, "native setup/action acceptance not established; no attack retried")

    def position(self, player, x, yaw):
        prior = self.wait_status(player, lambda value: value["ready"], "client-ready")["teleports"]
        self.send(f"tp {player} {x} 80 0.5 {yaw} 0")
        self.wait_status(player, lambda value: value["ready"] and value["teleports"] > prior,
                         "teleport-ack")
        # A small real position packet, with server NBT acceptance checked below.
        self.command(player, "move_pos_rot", position=[x + 0.125, 80, 0.5, yaw, 0])
        self.native_eventually(f"{player}_MOVE", f"if data entity {player} {{Pos:[{x + 0.125}d,80.0d,0.5d]}}")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.send("difficulty normal")
        self.send("gamerule minecraft:natural_health_regeneration false")
        self.send("fill -2 79 -2 8 79 3 minecraft:stone")
        self.send("fill -2 80 -2 8 84 3 minecraft:air")
        self.send("function paudit:init")
        self.send("scoreboard objectives add paudit_check dummy")
        self.send("scoreboard objectives add melee_native minecraft.custom:minecraft.damage_dealt")
        with self.clients() as processes:
            self.client_processes = processes
            for index, player in enumerate(self.players):
                self.send(f"gamemode survival {player}")
                self.send(f"tag {player} add paudit_subject_{index}")
                self.send(f"effect give {player} minecraft:instant_health 1 10 true")
                self.send(f'item replace entity {player} weapon.mainhand with minecraft:diamond_sword[minecraft:unbreakable={{}},minecraft:custom_data={{melee_owner:"{player}"}}]')
                self.check(f"{player}_ITEM", f'if data entity {player} SelectedItem{{id:"minecraft:diamond_sword"}}')
                self.send(f"data modify storage paudit:oracle {player}_main set from entity {player} SelectedItem")
                self.send(f"data modify storage paudit:oracle {player}_uuid set from entity {player} UUID")
                # Check the independent server UUID before using its offline
                # identity to select a tracked wire entity in the other client.
                expected = struct.unpack(">iiii", uuid.UUID(self._offline_uuid(player)).bytes)
                self.check(f"{player}_UUID", f"if data entity {player} {{UUID:[I;{','.join(map(str, expected))}]}}")
                for objective in ("paudit_att1", "paudit_att2", "paudit_wpn", "melee_native"):
                    self.send(f"scoreboard players set {player} {objective} 0")
            for index, player in enumerate(self.players):
                self.position(player, 0.5 + index * 2, -90 if index == 0 else 90)
                self.native_eventually(f"{player}_HEALTH", f"if data entity {player} {{Health:20.0f}}")
            for player in self.players:
                other = self.players[1 - self.players.index(player)]
                target = self._offline_uuid(other)
                self.wait_status(player, lambda value: target in value["entities"], "target-tracked")
            # Native ticking, not wall-clock sleep, clears initial attack delay.
            self.send("execute store success score #clock_ok paudit_check store result score #start paudit_check run time query gametime")
            self.check("CLOCK_START", "if score #clock_ok paudit_check matches 1")
            self.send("scoreboard players add #start paudit_check 21")
            def refresh_clock():
                self.send("execute store success score #clock_ok paudit_check store result score #now paudit_check run time query gametime")
                self.check("CLOCK_NOW", "if score #clock_ok paudit_check matches 1")
            self.native_eventually("MELEE_TICKS", "if score #now paudit_check >= #start paudit_check",
                                   refresh=refresh_clock)
            for index, attacker in enumerate(self.players):
                victim = self.players[1 - index]
                for field in ("weapon_item", "attacker_uuid", "attacker_b_uuid", "subject_uuid"):
                    self.send(f"data remove storage paudit:audit state.{field}")
                self.send(f"execute store success score #health_ok paudit_check store result score #health_before paudit_check run data get entity {victim} Health 100")
                self.check("HEALTH_BEFORE_EXISTS", "if score #health_ok paudit_check matches 1")
                self.send(f"scoreboard players operation #other_native melee_native = {victim} melee_native")
                self.command(attacker, "attack", uuid=self._offline_uuid(victim))
                self.native_eventually(f"{attacker}_NATIVE_HIT", f"if score {attacker} melee_native matches 1..")
                self.send(f"execute store success score #health_ok paudit_check store result score #health_after paudit_check run data get entity {victim} Health 100")
                self.check("HEALTH_AFTER_EXISTS", "if score #health_ok paudit_check matches 1")
                self.check("NONFATAL_NATIVE_DAMAGE", "if score #health_after paudit_check matches 1.. if score #health_after paudit_check < #health_before paudit_check")
                print(f"ACCEPTED NATIVE HIT: {attacker} -> {victim}", flush=True)
                self.eventually(f"{attacker}_WEAPON_EVENT", f"if score {attacker} paudit_wpn matches 1")
                for objective in ("paudit_att1", "paudit_att2"):
                    self.check(f"{victim}_{objective}", f"if score {victim} {objective} matches 1")
                    self.check(f"{attacker}_{objective}_ISOLATED", f"if score {attacker} {objective} matches {index}")
                self.check("OTHER_WEAPON_ISOLATED", f"if score {victim} paudit_wpn matches {index}")
                self.check("OTHER_NATIVE_ISOLATED", f"if score {victim} melee_native = #other_native melee_native")
                for field, owner in (("subject_uuid", victim), ("attacker_uuid", attacker), ("attacker_b_uuid", attacker)):
                    self.equal_nbt(field, f"storage paudit:audit state.{field}", f"storage paudit:oracle {owner}_uuid")
                self.check("WEAPON_PRESENT", "if data storage paudit:audit state{weapon_present:1b}")
                self.equal_nbt("MELEE_WEAPON", "storage paudit:audit state.weapon_item", f"storage paudit:oracle {attacker}_main")
                self.send(f"item replace entity {attacker} weapon.mainhand with minecraft:air")
                self.equal_nbt("MELEE_SNAPSHOT_AFTER_EDIT", "storage paudit:audit state.weapon_item", f"storage paudit:oracle {attacker}_main")
                self.check_cleanup()
            print("PASSED: two sequential client melee hits, native acceptance, UUID/item oracles and isolation", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, choices=["26.2"])
    parser.add_argument("--pack", required=True)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--output")
    parser.add_argument("--timeout", type=float, default=60)
    parser.add_argument("--java", default="java")
    args = parser.parse_args()
    if not 0 < args.timeout <= 300:
        parser.error("timeout must be between 0 and 300 seconds")
    args.op_player = args.client_command = None
    harness = PlayerMeleeHarness(args)
    try:
        harness.run()
    except InconclusiveSetup as error:
        print(f"INCONCLUSIVE: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
        return 2
    except ValidationFailure as error:
        print(f"FAILED: {error.phase}: {error}; log: {harness.log_path}", file=sys.stderr)
        return 1
    print(f"PASSED: bounded 26.2 melee; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
