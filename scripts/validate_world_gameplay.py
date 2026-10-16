#!/usr/bin/env python3
"""Verify emitted world/block capabilities on a disposable vanilla server.

Reuses the reload harness's lifecycle and diagnostics. No player client is
required: assertions cover world mutations, execution context, and controlled
observation samples. They do not prove native player health updates or
end-to-end multiplayer behavior.
"""
from __future__ import annotations

import argparse
import re
import sys
import time
from pathlib import Path

from vanilla_reload_harness import ServerHarness, ValidationFailure


class WorldGameplayHarness(ServerHarness):
    def prepare(self) -> None:
        super().prepare()
        with (self.server / "server.properties").open("a", encoding="utf-8") as properties:
            properties.write("server-ip=127.0.0.1\n")

    @staticmethod
    def marker_pattern(marker: str) -> re.Pattern[str]:
        # Match actual server chat, not an error echoing a submitted command.
        return re.compile(r"\[Server\] " + re.escape(marker) + r"$")

    def passed(self, name: str, command: str, marker: str | None = None) -> bool:
        marker = marker or f"__SAND_WORLD_{name}_OK__"
        done = f"__SAND_WORLD_{name}_DONE__"
        start = len(self.lines)
        self.send(command)
        self.send(f"say {done}")
        end = self.wait_for(self.marker_pattern(done), f"world-{name}", start)
        return any(self.marker_pattern(marker).search(line) for line in self.lines[start:end])

    def check(self, name: str, condition: str) -> None:
        marker = f"__SAND_WORLD_{name}_OK__"
        if not self.passed(name, f"execute {condition} run say {marker}"):
            raise ValidationFailure(f"world-{name}", f"runtime assertion failed: {condition}")

    def load_test_chunk(self, dimension: str) -> None:
        self.send(f"execute in {dimension} run forceload add 0 0")
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            marker = "__SAND_WORLD_LOADED_OK__"
            if self.passed("LOADED", f"execute in {dimension} if loaded 0 80 0 run say {marker}"):
                return
            time.sleep(0.1)
        raise ValidationFailure("world-load", f"test chunk did not load in {dimension}")

    def observation_sampler(self, handler: str) -> str:
        root = Path(self.args.pack) / "data" / "sand_audit" / "function"
        paths = [path for path in (root / "__sand_transition").glob("*.mcfunction")
                 if f"function sand_audit:{handler}" in path.read_text(encoding="utf-8")]
        if len(paths) != 1:
            raise ValidationFailure("observation-plan", f"expected one shared sampler for {handler}, got {paths}")
        return "sand_audit:" + paths[0].relative_to(root).with_suffix("").as_posix()

    def check_observations(self) -> None:
        numeric = self.observation_sampler("audit_health_changed")
        if numeric != self.observation_sampler("audit_health_lost") or numeric != self.observation_sampler("audit_health_gained"):
            raise ValidationFailure("observation-plan", "health consumers did not share one sample")
        boolean = self.observation_sampler("audit_health_low")
        if boolean != self.observation_sampler("audit_health_recovered"):
            raise ValidationFailure("observation-plan", "threshold consumers did not share one sample")
        # Drive the generated observer with controlled scores. Vanilla's health
        # criterion is read-only; native player health updates need live clients.
        self.send("scoreboard objectives remove sand_health")
        self.send("scoreboard objectives add sand_health dummy")
        objectives = ("sand_obs_change", "sand_obs_lost", "sand_obs_gain", "sand_obs_low", "sand_obs_back")
        for objective in objectives:
            self.send(f"scoreboard objectives add {objective} dummy")
        targets = {}
        for label, x in (("a", 12), ("b", 13)):
            tag = f"sand_observe_{label}"
            targets[label] = f"@e[tag={tag},limit=1]"
            self.send(f'execute in minecraft:overworld run summon minecraft:armor_stand {x} 80 12 {{Tags:["{tag}"],NoGravity:1b}}')
            for objective in objectives:
                self.send(f"execute in minecraft:overworld run scoreboard players set {targets[label]} {objective} 0")

        def sample(label: str, health: int | None, function: str = numeric) -> None:
            target = targets[label]
            value = f"set {target} sand_health {health}" if health is not None else f"reset {target} sand_health"
            self.send(f"execute in minecraft:overworld run scoreboard players {value}")
            self.send(f"execute in minecraft:overworld as {target} at @s run function {function}")

        def counts(label: str, changed: int, lost: int, gained: int) -> None:
            for objective, value in zip(objectives, (changed, lost, gained)):
                self.check(f"OBS_{label}_{objective}_{value}", f"in minecraft:overworld if score {targets[label]} {objective} matches {value}")

        sample("a", 20)
        sample("b", 10)
        counts("a", 0, 0, 0)
        counts("b", 0, 0, 0)
        sample("a", 17)
        counts("a", 1, 1, 0)
        counts("b", 0, 0, 0)
        sample("a", 23)
        counts("a", 2, 1, 1)
        sample("a", None)
        counts("a", 2, 1, 1)
        self.check("OBS_MISSING_SOURCE", f"in minecraft:overworld unless score {targets['a']} sand_health matches -2147483648..")
        sample("a", 24)
        counts("a", 3, 1, 2)
        sample("a", 24, boolean)
        sample("a", 5, boolean)
        sample("a", 5, boolean)
        sample("a", 10, boolean)
        self.check("OBS_BECAME_TRUE", f"in minecraft:overworld if score {targets['a']} sand_obs_low matches 1")
        self.check("OBS_BECAME_FALSE", f"in minecraft:overworld if score {targets['a']} sand_obs_back matches 1")
        print("PASSED: shared observation sampling, first baseline, numeric changes, missing samples, boolean transitions, and entity isolation", flush=True)

    def check_public_observations(self) -> None:
        health = self.observation_sampler("audit_native_health_changed")
        effect = self.observation_sampler("audit_speed_started")
        if health != self.observation_sampler("audit_native_health_lost") or effect != self.observation_sampler("audit_speed_stopped"):
            raise ValidationFailure("public-observation-plan", "equivalent custom sources were not shared")
        # Invoke player-scoped generated functions on one controlled living
        # entity to prove native reads and transitions without a player client.
        target = "@e[tag=sand_public_observer,limit=1]"
        self.send('execute in minecraft:overworld run summon minecraft:cow 14 80 14 {Tags:["sand_public_observer"],NoAI:1b,NoGravity:1b,Invulnerable:1b}')
        for objective in ("sand_obs_native", "sand_obs_nlost", "sand_obs_speed", "sand_obs_nspeed", "sand_obs_gold"):
            self.send(f"scoreboard objectives add {objective} dummy")
            self.send(f"execute in minecraft:overworld run scoreboard players set {target} {objective} 0")
        def sample(function: str) -> None:
            self.send(f"execute in minecraft:overworld as {target} at @s run function {function}")
        def count(objective: str, value: int) -> None:
            self.check(f"PUBLIC_{objective}_{value}", f"in minecraft:overworld if score {target} {objective} matches {value}")
        sample(health)
        sample(effect)
        count("sand_obs_native", 0)
        count("sand_obs_speed", 0)
        self.send(f"execute in minecraft:overworld run data merge entity {target} {{Health:7.5f}}")
        sample(health)
        count("sand_obs_native", 1)
        count("sand_obs_nlost", 1)
        self.send(f"execute in minecraft:overworld run data merge entity {target} {{Health:9.0f}}")
        sample(health)
        count("sand_obs_native", 2)
        count("sand_obs_nlost", 1)
        self.send(f"execute in minecraft:overworld run effect give {target} minecraft:speed 60 0 true")
        sample(effect)
        sample(effect)
        count("sand_obs_speed", 1)
        self.send(f"execute in minecraft:overworld run effect clear {target} minecraft:speed")
        sample(effect)
        count("sand_obs_nspeed", 1)
        self.send("execute in minecraft:overworld run setblock 14 79 14 minecraft:air")
        coordinator = "sand_audit:__sand_observation_tick"
        # Deliberately start at a different origin: each source must anchor to
        # its observed entity rather than the server/function caller position.
        self.send(f"execute in minecraft:overworld positioned 0 80 0 as {target} run function {coordinator}")
        count("sand_obs_gold", 0)
        self.send("execute in minecraft:overworld run setblock 14 79 14 minecraft:gold_block")
        self.send(f"execute in minecraft:overworld positioned 0 80 0 as {target} run function {coordinator}")
        count("sand_obs_gold", 1)
        print("PASSED: public observations share native health/effect samples and anchor block conditions to the observed entity", flush=True)

    def run_client(self) -> None:
        """Verify exported world actions, scoped queries, and block offsets on vanilla."""
        self.send("scoreboard objectives add sand_rule_audit dummy")
        self.send("execute store result score #keep sand_rule_audit run gamerule keep_inventory")
        self.send("execute store result score #ticks sand_rule_audit run gamerule random_tick_speed")
        self.check("BOOLEAN_SETTING", "if score #keep sand_rule_audit matches 1")
        self.check("INTEGER_SETTING", "if score #ticks sand_rule_audit matches 0")
        # Only test setup loads chunks; the generated gameplay must not do so.
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            self.load_test_chunk(dimension)
        # Isolate the Nether test cells from random terrain and fluid updates.
        self.send("execute in minecraft:the_nether run fill 0 79 0 15 83 15 minecraft:barrier")
        self.send("execute in minecraft:the_nether run fill 1 80 1 14 82 14 minecraft:air")
        self.send("execute in minecraft:overworld run setblock 8 80 8 minecraft:dirt")
        for position, block in (("4 80 4", "air"), ("5 80 4", "stone"), ("6 80 4", "stone"), ("1 80 1", "air")):
            self.send(f"execute in minecraft:the_nether run setblock {position} minecraft:{block}")
        self.check("UNLOADED_BEFORE", "in minecraft:overworld unless loaded 1000000 80 1000000")
        if not self.passed(
            "INVOKE",
            "execute in minecraft:overworld positioned 8 80 8 run function sand_audit:audit_runtime_world_blocks",
            "__SAND_WORLD_UNLOADED_FALSE__",
        ):
            raise ValidationFailure("world-unloaded", "negated unavailable-block Condition did not run")
        checks = {
            "CHEST": "in minecraft:the_nether if block 4 80 4 minecraft:chest",
            "INVENTORY": "in minecraft:the_nether if items block 4 80 4 container.0 minecraft:diamond[minecraft:count~3]",
            'DATA': 'in minecraft:the_nether if data block 4 80 4 {CustomName:"Sand world audit"}',
            "REPLACE": "in minecraft:the_nether if block 5 80 4 minecraft:gold_block",
            "DESTROY": "in minecraft:the_nether if block 6 80 4 minecraft:air",
            "SCALED_ORIGIN": "in minecraft:the_nether if block 1 80 1 minecraft:emerald_block",
            "CALLER_CONTEXT": "in minecraft:overworld if block 8 80 8 minecraft:stone",
            "NO_IMPLICIT_LOAD": "in minecraft:overworld unless loaded 1000000 80 1000000",
        }
        for name, condition in checks.items():
            self.check(name, condition)
        self.send('execute in minecraft:overworld positioned 16 80 16 run function sand_audit:audit_world_macro {block:"minecraft:diamond_block"}')
        self.check("MACRO_ARGUMENT", "in minecraft:the_nether if block 2 80 2 minecraft:diamond_block")
        for dimension in ("minecraft:overworld", "minecraft:the_nether"):
            self.send(f'execute in {dimension} run summon minecraft:armor_stand 10 80 10 {{Tags:["sand_world_probe"],NoGravity:1b}}')
            self.send(f"execute in {dimension} run setblock 10 80 10 minecraft:air")
        self.send("execute in minecraft:overworld positioned 8 80 8 run function sand_audit:audit_world_targets")
        self.check("ENTITY_DIMENSION", "in minecraft:the_nether if entity @e[tag=sand_world_probe,tag=sand_world_visited,distance=0..]")
        self.check("ENTITY_OTHER_DIMENSION", "in minecraft:overworld unless entity @e[tag=sand_world_probe,tag=sand_world_visited,distance=0..]")
        self.check("ENTITY_POSITION", "in minecraft:the_nether if block 10 80 10 minecraft:gold_block")
        self.check("ENTITY_OTHER_BLOCK", "in minecraft:overworld if block 10 80 10 minecraft:air")
        self.check("MACRO_MULTILINE", "in minecraft:the_nether if block 3 80 2 minecraft:lapis_block")
        # Decimal origins avoid execute positioned's integer X/Z centering.
        self.send("execute in minecraft:overworld positioned 0.0 100.0 0.0 rotated 0 0 run function sand_audit:audit_world_block_offsets")
        self.check("BLOCK_RELATIVE_FRACTION", "in minecraft:overworld if block 0 100 0 minecraft:stone")
        self.check("BLOCK_LOCAL_FRACTION", "in minecraft:overworld if block 0 100 1 minecraft:gold_block")
        self.check("BLOCK_RELATIVE_DATA", 'in minecraft:overworld if data block 1 100 0 {CustomName:"Relative offset"}')
        self.check("BLOCK_LOCAL_DATA", 'in minecraft:overworld if data block 0 100 2 {CustomName:"Local offset"}')
        self.check_observations()
        self.check_public_observations()
        print("PASSED: runtime block queries, writes, inventory, NBT, dimension scaling, caller context, and unloaded-block behavior", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--pack", required=True)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--output")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--java", default="java")
    args = parser.parse_args()
    args.op_player = None
    args.client_command = None
    harness = WorldGameplayHarness(args)
    try:
        harness.run()
    except ValidationFailure as error:
        print(f"FAILED: {error.phase}: {error}\nlog: {harness.log_path}", file=sys.stderr)
        return 1
    print(f"PASSED: Minecraft {args.version} world gameplay and reload; log: {harness.log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
