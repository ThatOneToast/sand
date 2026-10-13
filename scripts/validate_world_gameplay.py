#!/usr/bin/env python3
"""Verify emitted world/block capabilities on a disposable vanilla server.

Reuses the reload harness's lifecycle and diagnostics. No player client is
required: the assertions cover world mutations and execution context, not
live-player or multiplayer behavior.
"""
from __future__ import annotations

import argparse
import re
import sys
import time

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

    def run_client(self) -> None:
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
