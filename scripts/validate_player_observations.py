#!/usr/bin/env python3
"""Validate player observations through real clients and normal tick dispatch.

Uses a disposable loopback-only Minecraft 26.2 server. This covers player health,
effect transitions, isolation, and reload; it does not prove combat participant
correlation or client-driven inventory interactions.
"""
from __future__ import annotations

import argparse
from contextlib import ExitStack
from pathlib import Path
import re
import subprocess
import sys
import time

from validate_world_gameplay import WorldGameplayHarness
from vanilla_reload_harness import RELOAD_START, ValidationFailure


class PlayerObservationHarness(WorldGameplayHarness):
    players = ("SandAlpha", "SandBeta")
    counters = ("sand_obs_native", "sand_obs_nlost", "sand_obs_speed", "sand_obs_nspeed")

    def prepare(self):
        super().prepare()
        with (self.server / "server.properties").open("a", encoding="utf-8") as properties:
            properties.write("max-players=2\n")

    def eventually(self, name, condition):
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            marker = f"__SAND_WORLD_{name}_OK__"
            if self.passed(name, f"execute {condition} run say {marker}"):
                return
            time.sleep(0.1)
        raise ValidationFailure(name, f"condition did not become true: {condition}")

    def counts(self, player, values):
        for objective, value in zip(self.counters, values):
            self.check(f"{player}_{objective}_{value}", f"if score {player} {objective} matches {value}")

    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        self.send("gamerule minecraft:natural_health_regeneration false")
        self.send("fill -2 79 -2 5 79 2 minecraft:stone")
        self.send("fill -2 80 -2 5 83 2 minecraft:air")
        # Other subscribed audit handlers share this representative pack.
        # Provision their sinks too, without changing its read-only criteria.
        fixture_counters = self.counters + (
            "sand_obs_change", "sand_obs_lost", "sand_obs_gain", "sand_obs_low", "sand_obs_back", "sand_obs_gold",
        )
        for objective in fixture_counters:
            self.send(f"scoreboard objectives add {objective} dummy")
        with ExitStack() as stack:
            processes = []
            try:
                for player in self.players:
                    log = stack.enter_context((self.output / f"{player}.log").open("w"))
                    process = subprocess.Popen([
                        sys.executable, "-u", str(Path(__file__).parent / "mc_validation/minimal_join_client.py"),
                        "127.0.0.1", str(self.port), "776", player, str(self.args.timeout * 4 + 90),
                    ], stdout=log, stderr=subprocess.STDOUT)
                    processes.append(process)
                    self.wait_for(re.compile(re.escape(player) + r" joined the game$"), "player-join")
                for index, player in enumerate(self.players):
                    self.send(f"tp {player} {index * 3 + 0.5} 80 0.5")
                time.sleep(4)
                for player in self.players:
                    self.check(f"{player}_PRESENT", f"if entity @a[name={player}]")
                    for objective in self.counters:
                        self.check(f"{player}_{objective}_SILENT", f"unless score {player} {objective} matches -2147483648..2147483647")
                        self.send(f"scoreboard players set {player} {objective} 0")
                self.send("damage SandAlpha 4 minecraft:generic")
                self.eventually("ALPHA_DAMAGED", "if score SandAlpha sand_obs_nlost matches 1")
                self.counts("SandAlpha", (1, 1, 0, 0))
                self.counts("SandBeta", (0, 0, 0, 0))
                self.send("effect give SandBeta minecraft:speed infinite 0 true")
                self.eventually("BETA_SPEED", "if score SandBeta sand_obs_speed matches 1")
                self.counts("SandAlpha", (1, 1, 0, 0))
                self.counts("SandBeta", (0, 0, 1, 0))
                start = len(self.lines)
                self.send("reload")
                self.wait_for(RELOAD_START, "player-reload", start)
                self.send("say __SAND_PLAYERS_RELOADED__")
                self.wait_for(self.marker_pattern("__SAND_PLAYERS_RELOADED__"), "player-reload-complete", start)
                # Keep both real players present over several keep-alive cycles.
                for index in range(35):
                    for player in self.players:
                        self.check(f"{player}_ALIVE_{index}", f"if entity @a[name={player}]")
                    time.sleep(1)
                self.counts("SandAlpha", (1, 1, 0, 0))
                self.counts("SandBeta", (0, 0, 1, 0))
                self.send("effect clear SandBeta minecraft:speed")
                self.eventually("BETA_SPEED_STOPPED", "if score SandBeta sand_obs_nspeed matches 1")
                self.send("damage SandBeta 2 minecraft:generic")
                self.eventually("BETA_DAMAGED", "if score SandBeta sand_obs_nlost matches 1")
                self.counts("SandAlpha", (1, 1, 0, 0))
                self.counts("SandBeta", (1, 1, 1, 1))
                if any(process.poll() is not None for process in processes):
                    raise ValidationFailure("player-client", "a client exited during the assertions")
                print("PASSED: two real players, silent baselines, health/effect transitions, isolation, and reload", flush=True)
            finally:
                for process in processes:
                    process.terminate()
                for process in processes:
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)


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
    harness = PlayerObservationHarness(args)
    try:
        harness.run()
    except ValidationFailure as error:
        print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
        return 1
    print(f"PASSED: Minecraft {args.version} player observations; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
