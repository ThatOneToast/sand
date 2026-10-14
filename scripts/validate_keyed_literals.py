#!/usr/bin/env python3
"""Validate literal NBT defaults and writes in the CLI-built unified-State pack."""
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
import time

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import ValidationFailure, parse_args

LITERAL = r'$(missing) $(u0) "quoted" \ path'
PAYLOAD = {"text": LITERAL, "nested": {"literal": "$(u0)", "quoted": r'"quoted" \ path'}}
PROBE = "keyed_literal_probe"
RELOADED = "SAND_KEYED_LITERAL_LOAD_COMPLETE"


class KeyedLiteralHarness(PlayerHarness):
    """Exercise normal State initialization and owned writes for four subjects."""

    def __init__(self, args):
        """Retain each assertion alongside the shared server and client logs."""
        super().__init__(args)
        self.checks = {}

    def prepare(self):
        """Add read-only observation helpers to the disposable server copy."""
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
        (functions / "loaded.mcfunction").write_text(f"say {RELOADED}\n")
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


def main():
    """Run the shared isolated lifecycle and retain partial evidence on failure."""
    args = parse_args(sys.argv[1:])
    if args.version != "26.2":
        raise ValueError("the passive-player protocol for this audit targets Minecraft 26.2")
    harness = KeyedLiteralHarness(args)
    evidence = {"minecraft": args.version, "complete": False, "checks": harness.checks}
    try:
        harness.run()
        evidence["complete"] = True
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        path = harness.output / "keyed-literal-evidence.json"
        path.write_text(json.dumps(evidence, indent=2) + "\n")
        print(f"Keyed literal evidence: {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
