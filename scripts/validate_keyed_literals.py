#!/usr/bin/env python3
"""Validate literal NBT defaults and writes in the CLI-built unified-State pack."""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
from pathlib import Path
import re
import sys
import time

from player_validation_harness import PlayerHarness
from measure_archetype_idle import TRACE_FILE
from vanilla_reload_harness import ValidationFailure, parse_args

LITERAL = r'$(missing) $(u0) "quoted" \ path'
PAYLOAD = {"text": LITERAL, "nested": {"literal": "$(u0)", "quoted": r'"quoted" \ path'}}
PROBE = "keyed_literal_probe"
RELOADED = "SAND_KEYED_LITERAL_LOAD_COMPLETE"


def owner_resolver(directory):
    """Extract a known production write frame; reject unsupported probe shapes."""
    frames = [path.read_text().splitlines() for path in sorted(directory.glob("*.mcfunction"))]
    frame = next((lines for lines in frames if any("set from entity @s UUID" in line for line in lines)), None)
    if frame is None:
        raise ValueError("no keyed write frame with an owner resolver")
    resolver = frame[2:-2]
    prefix = "data modify storage rpg:state __sand_owner."
    old = [f"{prefix}u{i} set from entity @s UUID[{i}]" for i in range(4)]
    new = ["data remove storage rpg:state __sand_owner.uuid", f"{prefix}uuid set from entity @s UUID"]
    corrected = ["execute unless entity @s run " + new[0], new[1]]
    if (resolver not in (old, new, corrected)
            or frame[0] != "data remove storage rpg:state __sand_owner.value"
            or not frame[1].startswith("data modify storage rpg:state __sand_owner.value set value ")
            or not frame[-2].startswith("function unified_state:sand/state_data/keyed/")
            or not frame[-2].endswith(" with storage rpg:state __sand_owner")
            or frame[-1] != frame[0]):
        raise ValueError("unexpected keyed write frame")
    return resolver


def trace_metrics(content):
    """Count evaluated commands separately from successful storage operations.

    A positive [R] for a data mutation is one successful mutation command, not
    its numeric result or a count of NBT nodes. [C] includes the probe wrapper.
    Failed/unchanged data copies have no positive [R] and are not mutations.
    """
    lines = content.splitlines()
    successes = [match[1] for line in lines
                 if (match := re.search(r"\[R = [1-9][0-9]*\] (.*)$", line))]
    return {
        "command_evaluations_including_wrapper": sum("[C] " in line for line in lines),
        "probe_wrapper_evaluations": 1,
        "entity_uuid_read_commands": sum(
            "[C] data modify storage rpg:state __sand_owner." in line
            and "set from entity @s UUID" in line for line in lines),
        "successful_storage_mutation_commands": sum(
            bool(re.search(r"(?:^| run )data (?:modify|remove) storage rpg:state ", command))
            for command in successes),
        "successful_storage_get_commands": sum(
            command.startswith("data get storage rpg:state ") for command in successes),
    }


class KeyedLiteralHarness(PlayerHarness):
    """Exercise normal State initialization and owned writes for four subjects."""

    def __init__(self, args):
        """Retain each assertion alongside the shared server and client logs."""
        super().__init__(args)
        self.checks = {}
        self.traces = {}

    def prepare(self):
        """Add read-only observation helpers to the disposable server copy."""
        if self.args.output and self.output.exists():
            raise ValueError(f"refusing to overwrite evidence: {self.output}")
        super().prepare()
        pack = self.server / "world/datapacks" / Path(self.args.pack).resolve().name
        probe = pack / "data" / PROBE
        if probe.exists():
            raise ValueError(f"input pack collides with reserved audit namespace {PROBE}")
        functions = probe / "function"
        functions.mkdir(parents=True)
        (functions / "read.mcfunction").write_text(
            f'$data modify storage {PROBE}:result current set from storage rpg:state '
            'owners[{uuid:$(uuid)}].components."$(component)"\n'
        )
        # Reuse the exact production resolver from a compiled write frame.
        # Probe macro bodies isolate get/remove/nested behavior without changing
        # the input pack. Rust regressions separately pin these emitted bodies.
        resolver = owner_resolver(pack / "data/unified_state/function/sand/state_data/write")
        self.resolver = resolver
        self.owner_read_count = sum("set from entity @s UUID" in line for line in resolver)
        uuid = "$(uuid)" if self.owner_read_count == 1 else "[I;$(u0),$(u1),$(u2),$(u3)]"
        path = f'owners[{{uuid:{uuid}}}].components."status".template'
        for name, body in {
            "get": f"$data get storage rpg:state {path}",
            "remove": f"$data remove storage rpg:state {path}",
            "present": f"$execute if data storage rpg:state {path} run function {PROBE}:nested",
        }.items():
            (functions / f"{name}_macro.mcfunction").write_text(body + "\n")
            (functions / f"{name}.mcfunction").write_text("\n".join([
                *resolver, f"function {PROBE}:{name}_macro with storage rpg:state __sand_owner"
            ]) + "\n")
        for label in ("inner", "outer"):
            (functions / f"record_{label}.mcfunction").write_text(
                f"$data modify storage {PROBE}:result {label} set from storage rpg:state {path}\n")
        (functions / "nested.mcfunction").write_text(
            f"execute as @e[tag=keyed_literal_b,limit=1] run function {PROBE}:get\n"
            f"function {PROBE}:record_inner with storage rpg:state __sand_owner\n"
            f"function {PROBE}:get\n"
            f"function {PROBE}:record_outer with storage rpg:state __sand_owner\n"
            f"scoreboard players add #nested keyed_probe 1\n"
        )
        for name, actor, target in (
            ("entity_write", "@e[tag=keyed_literal_a,limit=1]", "set_entity_literals"),
            ("player_write", "SandAlpha", "set_player_literals"),
            ("attach", "@e[tag=keyed_literal_a,limit=1]", "attach_zombie_components"),
        ):
            (functions / f"trace_{name}.mcfunction").write_text(
                f"execute as {actor} run function unified_state:{target}\n")
        (functions / "loaded.mcfunction").write_text(f"say {RELOADED}\n")
        (functions / "trace_get.mcfunction").write_text(
            f"execute as @e[tag=keyed_literal_a,limit=1] run function {PROBE}:get\n")
        load_path = pack / "data/minecraft/tags/function/load.json"
        load = json.loads(load_path.read_text())
        load["values"].append(f"{PROBE}:loaded")
        load_path.write_text(json.dumps(load) + "\n")

    def check(self, name, condition):
        """Keep failed assertions in the evidence before propagating them."""
        self.checks[name] = False
        super().check(name, condition)
        self.checks[name] = True

    def observe(self, actor, component):
        """Refresh the independent oracle; missing reads cannot reuse old data."""
        self.send(f"data remove storage {PROBE}:result current")
        self.send(f"data remove storage {PROBE}:args uuid")
        self.send(f"execute as {actor} run data modify storage {PROBE}:args uuid set from entity @s UUID")
        self.send(f"data modify storage {PROBE}:args component set value {json.dumps(component)}")
        self.send(f"function {PROBE}:read with storage {PROBE}:args")

    @staticmethod
    def matches(expected):
        """Compare NBT string values in Minecraft, independent of printed SNBT quoting."""
        return f"if data storage {PROBE}:result current{json.dumps(expected)}"

    def expect(self, name, actor, component, expected):
        """Observe one named owner and verify its expected component values."""
        self.observe(actor, component)
        self.check(name, self.matches(expected))

    def wait_player_default(self, actor):
        """Wait for natural player attachment, refreshing the owner on each attempt."""
        name = f"NATURAL_DEFAULT_{actor}"
        marker = f"__SAND_WORLD_{name}_OK__"
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.observe(actor, "progression")
            if self.passed(name, f"execute {self.matches({'template': '$(player)'})} run say {marker}"):
                self.checks[name] = True
                return
            time.sleep(0.1)
        self.checks[name] = False
        raise ValidationFailure(name, f"literal default never initialized for {actor}")

    def trace(self, label, function):
        """Count executed entity UUID reads, separately from command evaluations."""
        start = len(self.lines)
        self.send(f"debug function {PROBE}:trace_{function}")
        self.send(f"say __KEYED_TRACE_{label}__")
        self.wait_for(self.marker_pattern(f"__KEYED_TRACE_{label}__"), "keyed-trace", start)
        names = [match[1] for line in self.lines[start:] if (match := TRACE_FILE.search(line))]
        if len(names) != 1:
            raise ValidationFailure("keyed-trace", f"expected one trace output: {names}")
        content = (self.server / "debug" / names[0]).read_text()
        (self.output / f"{label}.trace.txt").write_text(content)
        metrics = trace_metrics(content)
        reads = metrics["entity_uuid_read_commands"]
        self.traces[label] = {**metrics, "sha256": hashlib.sha256(content.encode()).hexdigest()}
        if label in ("existing_attach", "existing_attach_repeat", "same_owner_get"):
            mutations = metrics["successful_storage_mutation_commands"]
            if mutations != 0:
                raise ValidationFailure("keyed-trace", f"{label}: expected zero successful storage mutations, got {mutations}")
        if label == "same_owner_get" and metrics["successful_storage_get_commands"] != 1:
            raise ValidationFailure("keyed-trace", "unchanged UUID copy must still execute the get macro")
        # The fixture's attach builder already deduplicates identical inline
        # commands. Measure that exported shape, not a per-field multiplication.
        expected = ({"fresh_attach": 12, "existing_attach": 4, "existing_attach_repeat": 4, "same_owner_get": 4}.get(label, 8)
                    if self.owner_read_count == 4 else
                    {"fresh_attach": 1, "existing_attach": 1, "existing_attach_repeat": 1, "same_owner_get": 1}.get(label, 2))
        if reads != expected:
            raise ValidationFailure("keyed-trace", f"{label}: expected {expected} UUID reads, got {reads}")

    def run_client(self):
        """Verify defaults, interleaved ownership, reload retention and selective detach."""
        self.wait_for(re.compile(r"Starting minecraft server version " + re.escape(self.args.version) + r"$"), "runtime-version")
        self.send("forceload add 0 0")
        self.eventually("LITERAL_CHUNK", "if loaded 0 100 0")
        first = "@e[tag=keyed_literal_a,limit=1]"
        second = "@e[tag=keyed_literal_b,limit=1]"
        with self.clients():
            for player in self.players:
                self.wait_player_default(player)
            self.send("tick freeze")
            for tag in ("keyed_literal_a", "keyed_literal_b"):
                self.send(f'summon minecraft:zombie 0 100 0 {{Tags:["{tag}"],NoAI:1b,NoGravity:1b,Invulnerable:1b,PersistenceRequired:1b}}')
            for actor in (first, second):
                self.send(f"execute as {actor} run function unified_state:attach_zombie_components")
            self.expect("ENTITY_A_DEFAULT", first, "status", {"template": "$(u0)"})
            self.expect("ENTITY_B_DEFAULT", second, "status", {"template": "$(u0)"})
            self.check("GLOBAL_DEFAULT", 'if data storage rpg:state components.world{template:"$(global)"}')

            self.send("execute as SandAlpha run function unified_state:set_player_literals")
            self.expect("PLAYER_A_LITERALS", "SandAlpha", "progression", {"template": LITERAL, "preferences": PAYLOAD})
            self.expect("PLAYER_B_UNCHANGED", "SandBeta", "progression", {"template": "$(player)"})
            self.send(f"execute as {first} run function unified_state:set_entity_literals")
            self.expect("ENTITY_A_LITERALS", first, "status", {"template": LITERAL, "details": PAYLOAD})
            self.expect("ENTITY_B_UNCHANGED", second, "status", {"template": "$(u0)"})
            self.send("function unified_state:set_global_literal")
            self.check("GLOBAL_LITERAL", "if data storage rpg:state components.world" + json.dumps({"template": LITERAL}))
            self.check("STAGING_CLEAN", "unless data storage rpg:state __sand_owner.value")

            self.send("scoreboard objectives add keyed_probe dummy")
            self.send("scoreboard players set #nested keyed_probe 0")
            self.send(f"execute as {first} run function {PROBE}:present")
            self.check("NESTED_CALLBACK", "if score #nested keyed_probe matches 1")
            self.check("NESTED_OWNER_RESOLUTION", f"if data storage {PROBE}:result " + json.dumps({"inner": "$(u0)", "outer": LITERAL}))
            self.expect("NESTED_OUTER_RETAINED", first, "status", {"template": LITERAL})
            self.expect("NESTED_INNER_RETAINED", second, "status", {"template": "$(u0)"})
            # Observe the final macro call's result/success callback contract.
            # Ordinary non-returning functions leave the observer untouched.
            for actor in (first, second):
                for line in self.resolver:
                    self.send(f"execute as {actor} run {line}")
                for key in ("result", "success"):
                    self.send(f"scoreboard players set #{key} keyed_probe -99")
                self.send(f"execute as {actor} store result score #result keyed_probe store success score #success keyed_probe run function {PROBE}:get_macro with storage rpg:state __sand_owner")
                for key in ("result", "success"):
                    self.check(f"VALID_{'A' if actor == first else 'B'}_{key.upper()}", f"if score #{key} keyed_probe matches -99")

            # Vanilla executes the load tag on a game tick after reload.
            # Keep ticks running until our load-completion marker arrives.
            self.send("tick unfreeze")
            start = len(self.lines)
            self.send("reload")
            self.wait_for(self.marker_pattern(RELOADED), "literal-reload", start)
            self.send("tick freeze")
            start = len(self.lines)
            self.send("tick sprint 20")
            self.wait_for("Sprint completed with", "post-reload-ticks", start)
            self.expect("PLAYER_A_RELOAD", "SandAlpha", "progression", {"template": LITERAL, "preferences": PAYLOAD})
            self.expect("PLAYER_B_RELOAD", "SandBeta", "progression", {"template": "$(player)"})
            self.expect("ENTITY_A_RELOAD", first, "status", {"template": LITERAL, "details": PAYLOAD})
            self.expect("ENTITY_B_RELOAD", second, "status", {"template": "$(u0)"})
            self.check("GLOBAL_RELOAD", "if data storage rpg:state components.world" + json.dumps({"template": LITERAL}))
            self.check("RELOAD_STAGING_CLEAN", "unless data storage rpg:state __sand_owner.value")

            self.send(f"execute as {first} run function unified_state:detach_status")
            self.observe(first, "status")
            self.check("DETACHED_TEMPLATE", f"unless data storage {PROBE}:result current.template")
            self.check("DETACHED_PAYLOAD", f"unless data storage {PROBE}:result current.details")
            self.expect("OTHER_ENTITY_RETAINED", second, "status", {"template": "$(u0)"})
            self.expect("PLAYER_RETAINED", "SandAlpha", "progression", {"template": LITERAL, "preferences": PAYLOAD})

            self.trace("entity_write", "entity_write")
            self.trace("player_write", "player_write")
            self.send(f"execute as {first} run function unified_state:detach_status")
            self.trace("fresh_attach", "attach")
            self.trace("existing_attach", "attach")
            self.trace("existing_attach_repeat", "attach")
            self.trace("same_owner_get", "get")

            # Seed the ambient production arguments with B, then invoke the same
            # compiled write without an executor. It must not overwrite B.
            self.send(f"execute as {second} run function unified_state:set_entity_literals")
            self.send(f"execute as {second} run function unified_state:detach_status")
            self.send(f"execute as {second} run function unified_state:attach_zombie_components")
            self.send("function unified_state:set_entity_literals")
            self.expect("MISSING_EXECUTOR_NO_STALE_WRITE", second, "status", {"template": "$(u0)"})
            self.check("MISSING_EXECUTOR_STAGING_CLEAN", "unless data storage rpg:state __sand_owner.value")
            self.send(f"execute as {second} run function {PROBE}:get")
            for line in self.resolver:
                self.send(line)
            for key in ("result", "success"):
                self.send(f"scoreboard players set #{key} keyed_probe -99")
            self.send(f"execute store result score #result keyed_probe store success score #success keyed_probe run function {PROBE}:get_macro with storage rpg:state __sand_owner")
            for key in ("result", "success"):
                self.check(f"MISSING_GET_{key.upper()}", f"if score #{key} keyed_probe matches 0")
            self.send(f"execute as {second} run function {PROBE}:get")
            self.send(f"function {PROBE}:remove")
            self.expect("MISSING_EXECUTOR_NO_STALE_REMOVE", second, "status", {"template": "$(u0)"})
            self.send(f"execute as {second} run function {PROBE}:get")
            self.send(f"function {PROBE}:present")
            self.check("MISSING_EXECUTOR_NO_CALLBACK", "if score #nested keyed_probe matches 1")
            self.check("MISSING_EXECUTOR_UUID_CLEARED", "unless data storage rpg:state __sand_owner.uuid")


def main():
    """Run the shared isolated lifecycle and retain partial evidence on failure."""
    args = parse_args(sys.argv[1:])
    if args.version != "26.2":
        raise ValueError("the passive-player protocol for this audit targets Minecraft 26.2")
    if args.output and Path(args.output).exists():
        raise ValueError(f"refusing to overwrite evidence: {args.output}")
    harness = KeyedLiteralHarness(args)
    pack = Path(args.pack).resolve()
    digest = hashlib.sha256()
    for path in sorted(p for p in pack.rglob("*") if p.is_file()):
        for field in (path.relative_to(pack).as_posix().encode(), path.read_bytes()):
            digest.update(len(field).to_bytes(8, "big"))
            digest.update(field)
    evidence = {
        "minecraft": args.version, "complete": False, "checks": harness.checks,
        "input_pack_sha256": digest.hexdigest(),
        "source_revision": os.environ.get("SAND_SOURCE_REVISION") or subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True).strip(),
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    }
    try:
        harness.run()
        evidence["complete"] = True
    except Exception as error:
        evidence["failure"] = {"type": type(error).__name__, "message": str(error)}
        raise
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        evidence["static_owner_entity_nbt_reads_per_resolution"] = getattr(harness, "owner_read_count", None)
        evidence["native_traces"] = harness.traces
        evidence["production_resolver"] = getattr(harness, "resolver", None)
        path = harness.output / "keyed-literal-evidence.json"
        path.write_text(json.dumps(evidence, indent=2) + "\n")
        print(f"Keyed literal evidence: {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
