#!/usr/bin/env python3
"""Measure executed idle reconcile commands in the CLI-built RPG example.

Uses vanilla debug-function traces, including nested calls and failed guards.
This measures command evaluations, not MSPT, complete tick cost, or due native
observations. Frozen game time keeps the native observation cadence idle.
"""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import shutil
import tempfile

from player_validation_harness import PlayerHarness

REPO_ROOT = Path(__file__).resolve().parents[1]
RESOURCE = r"[a-z0-9_.-]+:[a-z0-9_./-]+"
FRAME = re.compile(rf"^( *)(?:\[F\]) ({RESOURCE}) size=\d+$")
COMMAND = re.compile(r"^( *)\[C\] (.+)$")
TRACE_FILE = re.compile(r"to output file (debug-trace-[a-zA-Z0-9_.-]+\.txt)$")


def verified_runtime_version(lines: list[str]) -> str:
    """Reject absent or mismatched startup identity before collecting samples."""
    versions = {
        match[1] for line in lines
        if (match := re.search(r"Starting minecraft server version (\S+)$", line))
    }
    if versions != {"26.2"}:
        raise ValueError(f"expected Minecraft 26.2 runtime, observed {sorted(versions)}")
    return "26.2"


def trace_counts(content: str) -> dict[str, int]:
    """Count actual [C] evaluations by the enclosing vanilla function frame.

    Console 'Traced N command(s)' counts root commands only. [M]/[R] messages
    are not evaluations. Reject unknown frames and errors instead of reporting
    incomplete data as a smaller cost.
    """
    frames: list[tuple[int, str]] = []
    counts: Counter[str] = Counter()
    for line in content.splitlines():
        if re.match(r"^ *\[E\]", line):
            raise ValueError(f"vanilla trace reported an error: {line.strip()}")
        frame = FRAME.fullmatch(line)
        command = COMMAND.fullmatch(line)
        if frame:
            depth = len(frame[1])
            while frames and frames[-1][0] >= depth:
                frames.pop()
            frames.append((depth, frame[2]))
        elif command:
            depth = len(command[1])
            while frames and frames[-1][0] >= depth:
                frames.pop()
            if not frames:
                raise ValueError(f"command outside a recognized function frame: {line}")
            counts[frames[-1][1]] += 1
        elif re.match(r"^ *\[(?:F|C)\]", line):
            raise ValueError(f"unrecognized vanilla trace record: {line}")
    if not counts:
        raise ValueError("trace contains no command evaluations")
    return dict(sorted(counts.items()))


def pack_digest(pack: Path) -> str:
    """Identify the exact input datapack independently of its checkout path."""
    digest = hashlib.sha256()
    for path in sorted(p for p in pack.rglob("*") if p.is_file()):
        relative = path.relative_to(pack).as_posix().encode()
        content = path.read_bytes()
        for field in (relative, content):
            digest.update(len(field).to_bytes(8, "big"))
            digest.update(field)
    return digest.hexdigest()


def reconcile_target(pack: Path) -> tuple[str, str]:
    """Discover the RPG archetype's actual exported callback and membership tag."""
    scans = list((pack / "data/rpg/function").glob("__sand_entity/*/reconcile_scan.mcfunction"))
    if len(scans) != 1:
        raise ValueError(f"expected one RPG reconcile scan, found {len(scans)}; build examples/rpg_entity first")
    match = re.fullmatch(
        rf"execute as @e\[type=minecraft:zombie,tag=([^,\]]+)\] at @s run function ({RESOURCE})\s*",
        scans[0].read_text(),
    )
    if not match:
        raise ValueError("RPG reconcile scan changed; update the measurement setup explicitly")
    return match[2], match[1]


class IdleCostHarness(PlayerHarness):
    def __init__(self, args: argparse.Namespace, target: str, marker: str, digest: str):
        super().__init__(args)
        self.target = target
        self.marker = marker
        with Path(args.jar).open("rb") as server_jar:
            jar_digest = hashlib.file_digest(server_jar, "sha256").hexdigest()
        self.evidence = {
            "schema_version": 1,
            "measurement": "vanilla_debug_function_command_evaluations",
            "target_minecraft_version": args.version,
            "minecraft_version": None,
            "server_jar_sha256": jar_digest,
            "input_pack_sha256": digest,
            "reconcile_function": target,
            "warmup_calls": 25,
            "connected_players": 2,
            "game_ticks_frozen": True,
            "complete": False,
            "samples": [],
        }

    def measure(self, count: int, sample: int) -> None:
        start = len(self.lines)
        self.send("debug function sand_cost_probe:reconcile")
        self.send(f"say SAND_COST_TRACE_DONE_{count}_{sample}")
        self.wait_for(f"SAND_COST_TRACE_DONE_{count}_{sample}", "trace", start)
        paths = [match[1] for line in self.lines[start:] if (match := TRACE_FILE.search(line))]
        if len(paths) != 1:
            raise ValueError(f"expected one successful trace file announcement, got {paths}")
        content = (self.server / "debug" / paths[0]).read_text()
        counts = trace_counts(content)
        if counts.get("sand_cost_probe:reconcile") != 2:
            raise ValueError("trace must contain probe dispatch and completion evaluations")
        path = Path("traces") / f"entities-{count}-sample-{sample}.txt"
        (self.output / path).parent.mkdir(parents=True, exist_ok=True)
        (self.output / path).write_text(content)
        total = sum(counts.values())
        result = {
            "entities": count,
            "sample": sample,
            "command_evaluations": total,
            "probe_control_evaluations": 2,
            "reconcile_evaluations": total - 2,
            "by_function": counts,
            "trace": path.as_posix(),
        }
        if count == 0 and total != 2:
            raise ValueError("empty selection unexpectedly executed reconcile work")
        self.evidence["samples"].append(result)
        print(f"COST entities={count} sample={sample} commands={total} reconcile={total - 2}", flush=True)

    def run_client(self) -> None:
        self.evidence["minecraft_version"] = verified_runtime_version(self.lines)
        self.send("forceload add 0 0")
        self.eventually("COST_CHUNK", "if loaded 0 100 0")
        self.send("gamerule minecraft:advance_time false")
        self.send("time set midnight")
        with self.clients():
            self.send("tick freeze")
            for count in (0, 1, 10):
                self.send("kill @e[tag=sand_cost_probe]")
                for _ in range(count):
                    self.send('summon minecraft:zombie 0 100 0 {Tags:["sand_cost_probe"],NoAI:1b,NoGravity:1b,Silent:1b,Invulnerable:1b,PersistenceRequired:1b}')
                # Run the exported adoption scan, then settle all dirty/initial
                # state through the unchanged exported reconcile callback.
                self.send(f"function {self.target.rsplit('/', 1)[0]}/scan")
                for _ in range(25):
                    self.send("function sand_cost_probe:reconcile")
                self.send("scoreboard objectives add sand_cost_count dummy")
                for holder, selector in (
                    ("#entities", "@e[tag=sand_cost_probe]"),
                    ("#adopted", f"@e[tag=sand_cost_probe,tag={self.marker}]"),
                ):
                    self.send(f"scoreboard players set {holder} sand_cost_count 0")
                    self.send(f"execute as {selector} run scoreboard players add {holder} sand_cost_count 1")
                    self.check(f"COST_{holder[1:]}_{count}", f"if score {holder} sand_cost_count matches {count}")
                for sample in range(3):
                    self.measure(count, sample)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=REPO_ROOT / "examples/rpg_entity/dist/rpg")
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=90)
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    target, marker = reconcile_target(args.pack)
    digest = pack_digest(args.pack)
    with tempfile.TemporaryDirectory(prefix="sand-cost-probe-pack-") as directory:
        pack = Path(directory) / "rpg"
        shutil.copytree(args.pack, pack)
        control = pack / "data/sand_cost_probe/function/reconcile.mcfunction"
        if control.exists():
            raise ValueError("input pack collides with the measurement control function")
        control.parent.mkdir(parents=True, exist_ok=True)
        control.write_text(
            f"execute as @e[tag=sand_cost_probe] at @s run function {target}\n"
            "say SAND_COST_MEASUREMENT_COMPLETE\n"
        )
        args.pack = str(pack)
        args.version = "26.2"
        args.output = args.op_player = args.client_command = None
        harness = IdleCostHarness(args, target, marker, digest)
        try:
            harness.run()
            harness.evidence["complete"] = True
        finally:
            evidence = harness.output / "cost-evidence.json"
            evidence.write_text(json.dumps(harness.evidence, indent=2) + "\n")
            print(f"Cost evidence: {evidence}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
