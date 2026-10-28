#!/usr/bin/env python3
"""Validate literal NBT defaults and writes in the CLI-built unified-State pack."""
from __future__ import annotations

import gzip
import hashlib
import json
import os
import platform
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


def pack_identity(directory):
    """Hash relative paths and bytes with unambiguous length framing."""
    digest = hashlib.sha256()
    for path in sorted(p for p in directory.rglob("*") if p.is_file()):
        for field in (path.relative_to(directory).as_posix().encode(), path.read_bytes()):
            digest.update(len(field).to_bytes(8, "big"))
            digest.update(field)
    return digest.hexdigest()


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


def owner_layout(directory):
    """Recognize complete known layouts; unknown or mixed owner paths fail closed."""
    lines = [line for path in sorted(directory.glob("*.mcfunction"))
             for line in path.read_text().splitlines() if "owners" in line]
    if not lines:
        raise ValueError("no keyed owner paths")
    legacy = all("owners[{uuid:" in line and "owners_v2" not in line for line in lines)
    indexed = all('owners_v2."$(uuid)"' in line and "owners[" not in line for line in lines)
    if not (legacy or indexed):
        raise ValueError("mixed or unsupported keyed owner layout")
    return "indexed" if indexed else "legacy"


def trace_metrics(content):
    """Count evaluated commands separately from successful storage operations.

    A positive [R] for a data mutation is one successful mutation command, not
    its numeric result or a count of NBT nodes. [C] includes the probe wrapper.
    Failed/unchanged data copies have no positive [R] and are not mutations.
    """
    lines = content.splitlines()
    successes = [match[1] for line in lines
                 if (match := re.search(r"\[R = [1-9][0-9]*\] (.*)$", line))]
    uuid_reads = 0
    for index, line in enumerate(lines):
        match = re.search(r"\[C\] (.*)$", line)
        if not match:
            continue
        command = match[1]
        if ("data modify storage rpg:state __sand_owner." not in command
                or "set from entity @s UUID" not in command):
            continue
        if command.startswith("data modify "):
            uuid_reads += 1
            continue
        # Guarded commands are sites, not necessarily executed UUID copies.
        # Vanilla emits an inline -> result for unchanged copies, or a matching
        # [R] record when a guarded copy succeeds. A false guard emits neither.
        evaluated = bool(re.search(r" -> -?[0-9]+$", command))
        for following in lines[index + 1:]:
            if "[C] " in following:
                break
            if re.search(r"\[R = -?[0-9]+\] " + re.escape(command) + r"$", following):
                evaluated = True
        uuid_reads += int(evaluated)
    return {
        "command_evaluations_including_wrapper": sum("[C] " in line for line in lines),
        "probe_wrapper_evaluations": 1,
        "entity_uuid_read_commands": uuid_reads,
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
        self.scaling = []

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
        indexed = owner_layout(pack / "data/unified_state/function/sand/state_data/keyed") == "indexed"
        self.indexed = indexed
        oracle_owner = 'owners_v2."$(uuid)"' if indexed else 'owners[{uuid:$(uuid)}]'
        (functions / "read.mcfunction").write_text(
            f'$data modify storage {PROBE}:result current set from storage rpg:state '
            f'{oracle_owner}.components."$(component)"\n'
        )
        # Reuse the exact production resolver from a compiled write frame.
        # Probe macro bodies isolate get/remove/nested behavior without changing
        # the input pack. Rust regressions separately pin these emitted bodies.
        resolver = owner_resolver(pack / "data/unified_state/function/sand/state_data/write")
        if indexed:
            steps = list((pack / "data/unified_state/function/sand/state_data/migrate").glob("*/step.mcfunction"))
            if len(steps) != 1:
                raise ValueError("expected exactly one migration step for rpg:state")
            self.migration_step = "unified_state:" + steps[0].relative_to(pack / "data/unified_state/function").with_suffix("").as_posix()
        self.resolver = resolver
        self.owner_read_count = sum("set from entity @s UUID" in line for line in resolver)
        uuid = "$(uuid)" if self.owner_read_count == 1 else "[I;$(u0),$(u1),$(u2),$(u3)]"
        owner = f'owners_v2."{uuid}"' if indexed else f'owners[{{uuid:{uuid}}}]'
        path = f'{owner}.components."status".template'
        for name, body in {
            "get": f"$data get storage rpg:state {path}",
            "remove": f"$data remove storage rpg:state {path}",
            "present": f"$execute if data storage rpg:state {path} run function {PROBE}:nested",
        }.items():
            (functions / f"{name}_macro.mcfunction").write_text(body + "\n")
            (functions / f"{name}.mcfunction").write_text("\n".join([
                *resolver, f"function {PROBE}:{name}_macro with storage rpg:state __sand_owner"
            ]) + "\n")
        self.get_macro_call = f"function {PROBE}:get_macro with storage rpg:state __sand_owner"
        if indexed:
            exported_get = (pack / "data/unified_state/function/get_entity_literal.mcfunction").read_text().splitlines()
            if (exported_get[:-1] != resolver or not exported_get[-1].startswith("function unified_state:sand/state_data/keyed/")
                    or not exported_get[-1].endswith(" with storage rpg:state __sand_owner")):
                raise ValueError("unexpected exported KeyedData get frame")
            self.get_macro_call = exported_get[-1]
            for operation in ("get", "remove"):
                (functions / f"{operation}.mcfunction").write_text(f"function unified_state:{operation}_entity_literal\n")
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
        if indexed:
            (functions / "trace_present.mcfunction").write_text(
                "execute as @e[tag=keyed_literal_a,limit=1] run function unified_state:present_entity_literal\n")
        (functions / "loaded.mcfunction").write_text(f"say {RELOADED}\n")
        get = "unified_state:get_entity_literal" if indexed else f"{PROBE}:get"
        (functions / "trace_get.mcfunction").write_text(
            f"execute as @e[tag=keyed_literal_a,limit=1] run function {get}\n")
        if os.environ.get("SAND_KEYED_SCALE") == "1":
            self.prepare_scaling(functions, indexed)
        load_path = pack / "data/minecraft/tags/function/load.json"
        load = json.loads(load_path.read_text())
        load["values"].append(f"{PROBE}:loaded")
        load_path.write_text(json.dumps(load) + "\n")
        self.probe_pack_sha256 = pack_identity(pack)

    def prepare_scaling(self, functions, indexed):
        """Keep seeding, migration, warm reads, and warm writes separate."""
        active = '{uuid:[I;-2147483648,2147483647,-1,1],components:{status:{template:"seed",details:{seed:1}},retained:{n:23}},unknown:{keep:1}}'
        for count in (10, 1000, 10000):
            rows = [active] + [f'{{uuid:[I;0,0,1,{i}],components:{{dormant:{{value:{i}}}}},unknown:{{keep:1}}}}' for i in range(1, count)]
            body = ["data remove storage rpg:state owners_v2",
                    "data modify storage rpg:state owners set value [" + ",".join(rows) + "]"]
            if indexed:
                body.append("data modify storage rpg:state __sand_keyed set value {version:1,cursor:0}")
            (functions / f"scale_seed_{count}.mcfunction").write_text("\n".join(body) + "\n")
        if indexed:
            (functions / "scale_migrate.mcfunction").write_text(("function " + self.migration_step + "\n") * 100)
        get = "unified_state:get_entity_literal" if indexed else f"{PROBE}:get"
        for kind, function in (("read", get), ("write", "unified_state:set_entity_literals")):
            command = f"execute as @e[tag=keyed_scale,limit=1] run function {function}\n"
            (functions / f"scale_{kind}.mcfunction").write_text(command * 100)
            (functions / f"scale_trace_{kind}.mcfunction").write_text(command)

    def scaling_cases(self):
        """Fixed one-active-owner workload; wall time includes console round trips."""
        self.send("tick freeze")
        self.send('kill @e[tag=keyed_boundary]')
        self.send('summon minecraft:zombie 0 100 0 {UUID:[I;-2147483648,2147483647,-1,1],Tags:["keyed_scale"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b}')
        self.check("SCALE_ACTIVE_EXECUTOR", 'if entity @e[tag=keyed_scale,limit=1]')
        def barrier(label, commands):
            start = len(self.lines)
            now = time.monotonic()
            for command in commands:
                self.send(command)
            marker = f"SAND_SCALE_{label}_DONE"
            self.send("say " + marker)
            self.wait_for(self.marker_pattern(marker), "scaling-" + label, start)
            return time.monotonic() - now
        def capture(label, function, wrapper=True):
            start = len(self.lines)
            barrier(label, ["debug function " + function])
            names = [match[1] for line in self.lines[start:] if (match := TRACE_FILE.search(line))]
            if len(names) != 1:
                raise ValidationFailure("scaling-trace", f"expected one trace: {names}")
            content = (self.server / "debug" / names[0]).read_text()
            (self.output / f"{label}.trace.txt").write_text(content)
            return {**trace_metrics(content), "probe_wrapper_evaluations": int(wrapper),
                    "sha256": hashlib.sha256(content.encode()).hexdigest()}
        def storage_snapshot(label):
            barrier("save_" + label, ["save-all flush"])
            saved = self.server / "world/data/rpg/command_storage.dat"
            raw = saved.read_bytes()
            (self.output / f"storage_{label}.dat").write_bytes(raw)
            return {"namespace": "rpg", "gzip_bytes": len(raw),
                    "nbt_bytes": len(gzip.decompress(raw)),
                    "sha256": hashlib.sha256(raw).hexdigest()}
        for count in (10, 1000, 10000):
            entry = {"total_retained_owners": count, "active_owners": 1,
                     "layout": "indexed" if self.indexed else "legacy",
                     "warm_operations_per_timing": 1000,
                     "cleanup": "not measured; retention unchanged"}
            self.scaling.append(entry)
            entry["seed_wall_seconds"] = barrier(f"seed_{count}", [f"function {PROBE}:scale_seed_{count}"])
            entry["seed_storage"] = storage_snapshot(f"seed_{count}")
            if self.indexed:
                entry["migration_first_step"] = capture(f"migration_first_{count}", self.migration_step, wrapper=False)
                entry["migration_checkpoint_storage"] = storage_snapshot(f"checkpoint_{count}")
                entry["migration_step_invocations"] = 2 * count + 1
                entry["migration_wall_seconds"] = barrier(f"migration_{count}",
                    [f"function {PROBE}:scale_migrate"] * (2 * count // 100)
                    + ["function " + self.migration_step] * (2 * count % 100))
                self.check(f"SCALE_{count}_MIGRATED", "if data storage rpg:state __sand_keyed{version:2}")
                self.check(f"SCALE_{count}_LEGACY_RETIRED", "unless data storage rpg:state owners")
                self.check(f"SCALE_{count}_DORMANT", f'if data storage rpg:state owners_v2."[I;0,0,1,{count-1}]".components.dormant{{value:{count-1}}}')
                entry["migrated_storage"] = storage_snapshot(f"migrated_{count}")
            entry["changed_write"] = capture(f"changed_write_{count}", f"{PROBE}:scale_trace_write")
            self.expect(f"SCALE_{count}_CHANGED_VALUE", '@e[tag=keyed_scale,limit=1]', "status", {"template": LITERAL, "details": PAYLOAD})
            entry["warm_read"] = capture(f"warm_read_{count}", f"{PROBE}:scale_trace_read")
            entry["warm_write"] = capture(f"warm_write_{count}", f"{PROBE}:scale_trace_write")
            for kind, expected_reads in (("changed_write", 2), ("warm_read", 1), ("warm_write", 2)):
                if entry[kind]["entity_uuid_read_commands"] != expected_reads:
                    raise ValidationFailure("scaling", f"{kind}: expected {expected_reads} reached UUID reads")
            if entry["warm_read"]["successful_storage_get_commands"] != 1:
                raise ValidationFailure("scaling", "warm read must reach the value get")
            if entry["warm_read"]["successful_storage_mutation_commands"] != 0:
                raise ValidationFailure("scaling", "warm same-owner reads mutated storage")
            for kind in ("read", "write"):
                entry[f"warm_{kind}_wall_seconds"] = barrier(f"{kind}_{count}", [f"function {PROBE}:scale_{kind}"] * 10)
            entry["steady_storage"] = storage_snapshot(f"steady_{count}")

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
        # The player setter also invokes lifecycle initialization: its two
        # guarded UUID copies are reached (one unchanged) in this fixture.
        expected = ({"fresh_attach": 12, "existing_attach": 4, "existing_attach_repeat": 4, "same_owner_get": 4}.get(label, 8)
                    if self.owner_read_count == 4 else
                    {"fresh_attach": 1, "existing_attach": 1, "existing_attach_repeat": 1, "same_owner_get": 1, "actual_nested": 3, "player_write": 4}.get(label, 2))
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
            if self.indexed:
                start = len(self.lines)
                self.send(f"execute as {first} run function unified_state:present_entity_literal")
                self.wait_for("SAND_KEYED_PRESENT", "actual-presence-callback", start)
                self.checks["ACTUAL_PRESENCE_NESTED_CALLBACK"] = True
                self.trace("actual_nested", "present")
                nested_trace = (self.output / "actual_nested.trace.txt").read_text()
                returned_lengths = [int(value) for value in re.findall(
                    r"\[R = ([0-9]+)\] data get storage rpg:state .*\.template$", nested_trace, re.MULTILINE)]
                if returned_lengths != [len("$(u0)"), len(LITERAL)]:
                    raise ValidationFailure("actual-nested-values", f"expected B then A values, got get results {returned_lengths}")
                self.checks["ACTUAL_NESTED_VALUE_ORACLE"] = True
                self.expect("ACTUAL_NESTED_OUTER_RETAINED", first, "status", {"template": LITERAL})
                self.expect("ACTUAL_NESTED_INNER_RETAINED", second, "status", {"template": "$(u0)"})
            # Observe the final macro call's result/success callback contract.
            # Ordinary non-returning functions leave the observer untouched.
            for actor in (first, second):
                for line in self.resolver:
                    self.send(f"execute as {actor} run {line}")
                for key in ("result", "success"):
                    self.send(f"scoreboard players set #{key} keyed_probe -99")
                self.send(f"execute as {actor} store result score #result keyed_probe store success score #success keyed_probe run {self.get_macro_call}")
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
            self.send(f"execute store result score #result keyed_probe store success score #success keyed_probe run {self.get_macro_call}")
            for key in ("result", "success"):
                self.check(f"MISSING_GET_{key.upper()}", f"if score #{key} keyed_probe matches 0")
            self.send(f"execute as {second} run function {PROBE}:get")
            self.send(f"function {PROBE}:remove")
            self.expect("MISSING_EXECUTOR_NO_STALE_REMOVE", second, "status", {"template": "$(u0)"})
            self.send(f"execute as {second} run function {PROBE}:get")
            self.send(f"function {PROBE}:present")
            self.check("MISSING_EXECUTOR_NO_CALLBACK", "if score #nested keyed_probe matches 1")
            self.check("MISSING_EXECUTOR_UUID_CLEARED", "unless data storage rpg:state __sand_owner.uuid")
            if self.indexed:
                start = len(self.lines)
                self.send("function unified_state:present_entity_literal")
                self.send("say ACTUAL_MISSING_PRESENCE_DONE")
                self.wait_for(self.marker_pattern("ACTUAL_MISSING_PRESENCE_DONE"), "missing-presence", start)
                if any("SAND_KEYED_PRESENT" in line for line in self.lines[start:]):
                    raise ValidationFailure("missing-presence", "actual callback ran without an executor")
                self.checks["ACTUAL_MISSING_PRESENCE"] = True

            # Both players and three entities write in one frozen game tick.
            self.send("execute store result score #before keyed_probe run time query gametime")
            for player in self.players:
                self.send(f"execute as {player} run function unified_state:set_player_literals")
                self.expect("SAME_TICK_" + player, player, "progression", {"template": LITERAL, "preferences": PAYLOAD})
            for label, actor in (("A", first), ("B", second)):
                self.send(f"execute as {actor} run function unified_state:set_entity_literals")
                self.expect("SAME_TICK_ENTITY_" + label, actor, "status", {"template": LITERAL, "details": PAYLOAD})
            self.absent_owner_cases()
            self.expect("THIRD_OWNER_NO_BLEED_A", first, "status", {"template": LITERAL})
            self.expect("THIRD_OWNER_NO_BLEED_B", second, "status", {"template": LITERAL})
            self.send("execute store result score #after keyed_probe run time query gametime")
            self.check("TWO_PLAYERS_THREE_ENTITIES_ONE_TICK", "if score #before keyed_probe = #after keyed_probe")
        if self.indexed:
            with self.clients():
                self.expect("PLAYER_A_RECONNECT", "SandAlpha", "progression", {"template": LITERAL, "preferences": PAYLOAD})
                self.expect("PLAYER_B_RECONNECT", "SandBeta", "progression", {"template": LITERAL, "preferences": PAYLOAD})
            self.migration_cases()
        if os.environ.get("SAND_KEYED_SCALE") == "1":
            self.scaling_cases()

    def absent_owner_cases(self):
        """Legacy data-modify filters create owners; read/remove/presence do not."""
        uuid = "[I;2147483647,-2147483648,0,-1]"
        owner = f'owners_v2."{uuid}"' if self.indexed else f'owners[{{uuid:{uuid}}}]'
        actor = '@e[tag=keyed_absent,limit=1]'
        self.send('summon minecraft:zombie 0 100 0 {UUID:' + uuid + ',Tags:["keyed_absent"],NoAI:1b,NoGravity:1b,Invulnerable:1b,PersistenceRequired:1b}')
        for operation in ("get", "remove", "present"):
            self.send(f"execute as {actor} run function {PROBE}:{operation}")
        self.check("ABSENT_READS_DO_NOT_CREATE", "unless data storage rpg:state " + owner)
        self.send(f"execute as {actor} run function unified_state:set_entity_literals")
        self.check("ABSENT_SET_CREATES_OWNER", f"if data storage rpg:state {owner}.components.status" + json.dumps({"template": LITERAL, "details": PAYLOAD}))
        self.send(f"data remove storage {PROBE}:result absent")
        self.send(f"data modify storage {PROBE}:result absent set from storage rpg:state {owner}")
        self.check("ABSENT_SET_RETAINS_UUID", f"if data storage {PROBE}:result absent{{uuid:{uuid}}}")
        if self.indexed:
            self.send(f"execute as {actor} run function unified_state:set_entity_uuid_literal")
            self.check("LITERAL_UUID_PARAMETER", f"if data storage rpg:state {owner}.components.status" + json.dumps({"template": r'literal $(uuid) \"quoted\" \\ path'}))
            self.check("LITERAL_UUID_STAGING_CLEAN", "unless data storage rpg:state __sand_owner.value")

    def migration_cases(self):
        """Drive the exported migrator with independent literal UUID/value oracles."""
        self.send("tick freeze")
        self.send(f"data modify storage {PROBE}:backup state set from storage rpg:state")
        uuid = "[I;-2147483648,2147483647,-1,0]"
        key = f'owners_v2."{uuid}"'
        row = '{uuid:' + uuid + ',components:{status:{template:"legacy $(uuid)"},other:{n:23}},unknown:{keep:1}}'
        dormant = '{uuid:[I;0,-1,2147483647,-2147483648],components:{offline:{value:91}}}'
        def seed(rows, metadata="{version:1,cursor:0}"):
            self.send("data remove storage rpg:state owners_v2")
            self.send("data modify storage rpg:state owners set value " + rows)
            self.send("data modify storage rpg:state __sand_keyed set value " + metadata)
        def steps(count):
            for _ in range(count):
                self.send("function " + self.migration_step)
        seed(f"[{row},{dormant}]")
        self.send('summon minecraft:zombie 0 100 0 {UUID:' + uuid + ',Tags:["keyed_boundary"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b}')
        actor = '@e[tag=keyed_boundary,limit=1]'
        self.send(f"execute as {actor} run function unified_state:attach_zombie_components")
        self.send(f"execute as {actor} run function unified_state:set_entity_literals")
        self.check("MIGRATION_DEFAULTS_BLOCKED", "unless data storage rpg:state owners_v2")
        self.check("MIGRATION_LITERAL_CLEAN", "unless data storage rpg:state __sand_owner.value")
        steps(1)
        self.check("MIGRATION_COPY_BEFORE_COMMIT", "if data storage rpg:state __sand_keyed{cursor:0,journal:{done:1b}}")
        self.check("MIGRATION_SOURCE_RETAINED", "if data storage rpg:state owners[1].components.offline{value:91}")
        self.check("MIGRATION_UNKNOWN_FIELDS", f"if data storage rpg:state {key}.unknown{{keep:1}}")
        steps(5)
        self.check("MIGRATION_COMPLETE", "if data storage rpg:state __sand_keyed{version:2}")
        self.check("MIGRATION_LEGACY_RETIRED", "unless data storage rpg:state owners")
        steps(2)
        self.send(f"data modify storage {PROBE}:result metadata set from storage rpg:state __sand_keyed")
        self.send(f"execute store success score #metadata keyed_probe run data modify storage {PROBE}:result metadata set value {{version:2}}")
        self.check("STEADY_METADATA_EXACT", "if score #metadata keyed_probe matches 0")
        self.check("MIGRATION_DORMANT_RETAINED", 'if data storage rpg:state owners_v2."[I;0,-1,2147483647,-2147483648]".components.offline{value:91}')
        self.send(f"execute as {actor} run function unified_state:attach_zombie_components")
        self.check("MIGRATION_LEGACY_BEATS_DEFAULT", f'if data storage rpg:state {key}.components.status{{template:"legacy $(uuid)"}}')
        self.send(f"execute as {actor} run function unified_state:set_entity_literals")
        self.check("SIGNED_BOUNDARY_LITERAL", f"if data storage rpg:state {key}.components.status" + json.dumps({"template": LITERAL, "details": PAYLOAD}))
        self.send(f"execute as {actor} run function unified_state:remove_entity_literal")
        self.check("ACTUAL_REMOVE", f"unless data storage rpg:state {key}.components.status.template")
        self.check("REMOVE_PRESERVES_OTHER_COMPONENT", f"if data storage rpg:state {key}.components.other{{n:23}}")
        self.send(f"execute as {actor} run function unified_state:set_entity_literals")
        self.send("forceload add 10000 10000")
        self.send("tick unfreeze")
        self.eventually("UNLOAD_DESTINATION_LOADED", "if loaded 10000 100 10000")
        self.send(f"tp {actor} 10000 100 10000")
        self.send("forceload remove 10000 10000")
        self.eventually("OWNER_ACTUALLY_UNLOADED", "unless entity " + actor)
        self.check("UNLOADED_OWNER_RETAINED", f"if data storage rpg:state {key}.components.status" + json.dumps({"template": LITERAL}))
        self.send("forceload add 10000 10000")
        self.eventually("OWNER_RETURNED", "if entity " + actor)
        self.check("RETURNED_OWNER_VALUE", f"if data storage rpg:state {key}.components.status" + json.dumps({"template": LITERAL}))
        self.send(f"execute as {actor} run function unified_state:get_entity_literal")
        self.send(f"tp {actor} 0 100 0")
        self.send("forceload remove 10000 10000")
        self.send("tick freeze")
        # Every durable intermediate state is resumed twice. A journal owns a
        # destination only after intent; an unrelated existing record conflicts.
        for label, journal, destination, success in (
            ("SNAPSHOT", '{row:' + row + '}', None, True),
            ("INTENT", '{row:' + row + ',intent:1b}', None, True),
            ("COPY", '{row:' + row + ',intent:1b}', row, True),
            ("DONE", '{row:' + row + ',intent:1b,next:1,done:1b}', row, True),
            ("DONE_MISSING_DEST", '{row:' + row + ',intent:1b,next:1,done:1b}', None, True),
            ("DONE_PARTIAL_DEST", '{row:' + row + ',intent:1b,next:1,done:1b}', '{uuid:' + uuid + ',components:{}}', False),
            ("DESTINATION", None, row, False),
            ("PARTIAL", '{row:' + row + ',intent:1b}', '{uuid:' + uuid + ',components:{}}', False),
            ("WRONG_TYPE", '{row:' + row + ',intent:1b}', '7', False),
        ):
            seed(f"[{row}]")
            if journal:
                self.send("data modify storage rpg:state __sand_keyed.journal set value " + journal)
            if destination:
                self.send(f"data modify storage rpg:state {key} set value {destination}")
            steps(4)
            steps(2)
            self.check("RESUME_" + label, "if data storage rpg:state __sand_keyed" + ("{version:2}" if success else ".error"))
            if not success:
                self.check("PRESERVE_SOURCE_" + label, "if data storage rpg:state owners[0].unknown{keep:1}")
                self.check("PRESERVE_DEST_" + label, 'if data storage rpg:state {owners_v2:{' + json.dumps(uuid) + ':' + destination + '}}')
        seed(f"[{row},{dormant}]", '{version:1,cursor:1,journal:{row:' + row + ',intent:1b,next:1,done:1b}}')
        self.send(f"data modify storage rpg:state {key} set value {row}")
        steps(6)
        self.check("CURSOR_COMMITTED_RETRY", 'if data storage rpg:state owners_v2."[I;0,-1,2147483647,-2147483648]".components.offline{value:91}')
        self.check("CURSOR_COMMITTED_COMPLETE", "if data storage rpg:state __sand_keyed{version:2}")
        # Retirement itself is durable on either side of source removal.
        for removed in (False, True):
            seed(f"[{row}]", "{version:1,cursor:1,finishing:1b}")
            self.send(f"data modify storage rpg:state {key} set value {row}")
            if removed:
                self.send("data remove storage rpg:state owners")
            steps(2)
            suffix = "AFTER_REMOVE" if removed else "BEFORE_REMOVE"
            self.check("FINISH_" + suffix, "if data storage rpg:state __sand_keyed{version:2}")
            self.check("FINISH_RETAINS_" + suffix, f"if data storage rpg:state {key}.unknown{{keep:1}}")
            self.check("FINISH_RETIRED_" + suffix, "unless data storage rpg:state owners")
        for label, rows in (
            ("DUPLICATE", f"[{row},{row}]"),
            ("CONFLICTING_DUPLICATE", f"[{row},{{uuid:{uuid},components:{{other:{{n:999}}}}}}]"),
            ("STRING_UUID", '[{uuid:"oops",components:{}}]'),
            ("LIST_UUID", '[{uuid:[0,0,0,0],components:{}}]'),
            ("SHORT_UUID", '[{uuid:[I;1,2,3],components:{}}]'),
            ("BAD_COMPONENTS", '[{uuid:[I;1,2,3,4],components:7}]'),
            ("BAD_CONTAINER", '{}'),
            ("EMPTY_STRING_CONTAINER", '\"\"'),
        ):
            seed(rows)
            steps(6)
            self.check(label + "_BLOCKED", "if data storage rpg:state __sand_keyed.error")
            self.check(label + "_SOURCE_RETAINED", "if data storage rpg:state {owners:" + rows + "}")
            self.check(label + "_NOT_COMPLETE", "unless data storage rpg:state __sand_keyed{version:2}")
        for label, next_value in (("SKIP", "2"), ("STRING_NEXT", '\"1\"')):
            seed(f"[{row},{dormant}]", '{version:1,cursor:0,journal:{row:' + row + ',intent:1b,next:' + next_value + ',done:1b}}')
            self.send(f"data modify storage rpg:state {key} set value {row}")
            steps(3)
            self.check(label + "_JOURNAL_BLOCKED", "if data storage rpg:state __sand_keyed.error")
            self.check(label + "_DORMANT_SOURCE_RETAINED", "if data storage rpg:state owners[1].components.offline{value:91}")
        for label, metadata in (
            ("STRING_CURSOR", '{version:1,cursor:"0"}'),
            ("BYTE_CURSOR", "{version:1,cursor:0b}"),
            ("MISSING_CURSOR", "{version:1}"),
            ("NEGATIVE_CURSOR", "{version:1,cursor:-1}"),
            ("LARGE_CURSOR", "{version:1,cursor:99}"),
        ):
            seed(f"[{row},{dormant}]", metadata)
            steps(2)
            self.check(label + "_BLOCKED", "if data storage rpg:state __sand_keyed.error")
            self.check(label + "_SOURCE_RETAINED", "if data storage rpg:state owners[0].unknown{keep:1}")
        seed(f"[{row},{dormant}]", "7")
        steps(2)
        self.check("MALFORMED_METADATA_PRESERVED", "if data storage rpg:state {__sand_keyed:7}")
        self.check("MALFORMED_METADATA_SOURCE", "if data storage rpg:state owners[0].unknown{keep:1}")
        seed(f"[{row}]", "{version:3,cursor:0}")
        steps(2)
        self.check("UNKNOWN_VERSION_BLOCKED", "if data storage rpg:state __sand_keyed.error")
        self.check("UNKNOWN_VERSION_SOURCE", "if data storage rpg:state owners[0].unknown{keep:1}")
        seed(f"[{row}]")
        self.send("data modify storage rpg:state owners_v2 set value 7")
        steps(2)
        self.check("MALFORMED_DESTINATION_MAP_BLOCKED", "if data storage rpg:state __sand_keyed.error")
        self.check("MALFORMED_DESTINATION_MAP_PRESERVED", "if data storage rpg:state {owners_v2:7}")
        self.check("MALFORMED_DESTINATION_MAP_SOURCE", "if data storage rpg:state owners[0].unknown{keep:1}")
        # Reload between copy and cursor commit. The load-tag sentinel, rather
        # than a console echo after `reload`, proves resources actually loaded.
        seed(f"[{row},{dormant}]")
        steps(1)
        self.send("tick unfreeze")
        start = len(self.lines)
        self.send("reload")
        self.wait_for(self.marker_pattern(RELOADED), "migration-reload", start)
        self.eventually("MIGRATION_RELOAD_FINISHED", "if data storage rpg:state __sand_keyed{version:2}")
        self.check("MIGRATION_RELOAD_DORMANT", 'if data storage rpg:state owners_v2."[I;0,-1,2147483647,-2147483648]".components.offline{value:91}')
        self.send("tick freeze")
        for field in ("owners_v2", "__sand_keyed", "__sand_owner"):
            self.send(f"data modify storage rpg:state {field} set from storage {PROBE}:backup state.{field}")


def main():
    """Run the shared isolated lifecycle and retain partial evidence on failure."""
    args = parse_args(sys.argv[1:])
    if args.version != "26.2":
        raise ValueError("the passive-player protocol for this audit targets Minecraft 26.2")
    if args.output and Path(args.output).exists():
        raise ValueError(f"refusing to overwrite evidence: {args.output}")
    harness = KeyedLiteralHarness(args)
    pack = Path(args.pack).resolve()
    evidence = {
        "minecraft": args.version, "complete": False, "checks": harness.checks,
        "input_pack_sha256": pack_identity(pack),
        "source_revision": os.environ.get("SAND_SOURCE_REVISION") or subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True).strip(),
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "server_jar_sha256": hashlib.sha256(Path(args.jar).read_bytes()).hexdigest(),
        "host": {"platform": platform.platform(), "machine": platform.machine()},
        "harness_source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "harness_worktree_dirty": bool(subprocess.check_output(["git", "status", "--porcelain", "--", "scripts"], text=True).strip()),
        "harness_dependencies": {name: hashlib.sha256((Path(__file__).parent / name).read_bytes()).hexdigest()
            for name in ("vanilla_reload_harness.py", "player_validation_harness.py", "validate_world_gameplay.py", "measure_archetype_idle.py", "mc_validation/minimal_join_client.py")},
        "java_version": subprocess.run([args.java, "-version"], capture_output=True, text=True).stderr.strip(),
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
        evidence["historical_owner_scaling"] = harness.scaling
        evidence["probe_pack_sha256"] = getattr(harness, "probe_pack_sha256", None)
        evidence["production_resolver"] = getattr(harness, "resolver", None)
        path = harness.output / "keyed-literal-evidence.json"
        path.write_text(json.dumps(evidence, indent=2) + "\n")
        print(f"Keyed literal evidence: {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
