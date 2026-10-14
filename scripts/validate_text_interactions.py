#!/usr/bin/env python3
"""Verify CLI-emitted typed interactions survive vanilla component decoding.

Reads the audit resources unchanged and observes the decoded text on disposable
text displays. This proves codec retention, not client click behavior.
"""
from __future__ import annotations

import json
from pathlib import Path
import re
import sys

from vanilla_reload_harness import parse_args
from validate_world_gameplay import WorldGameplayHarness


# Independent vanilla NBT oracles; do not derive expected keys from the export.
EXPECTED = {
    "run_command": 'click_event{action:"run_command",command:"/say hello"}',
    "suggest_command": 'click_event{action:"suggest_command",command:"/tell @s "}',
    "open_url": 'click_event{action:"open_url",url:"https://example.com"}',
    "copy": 'click_event{action:"copy_to_clipboard",value:"copy"}',
    "page": 'click_event{action:"change_page",page:2}',
    "text": 'hover_event{action:"show_text",value:{text:"hint",bold:1b}}',
    "item": 'hover_event{action:"show_item",id:"minecraft:stone",count:3}',
    "entity": 'hover_event{action:"show_entity",id:"minecraft:zombie",'
              'uuid:[I;306070887,-392490285,-1537850778,337068032],name:"Undead"}',
}


class TextInteractionHarness(WorldGameplayHarness):
    """Decode each exported description and inspect the server's saved value."""

    def __init__(self, args):
        """Retain per-case results even when a later assertion fails."""
        super().__init__(args)
        self.checks = {}

    def check(self, name, condition):
        """Record failed assertions before forwarding the shared diagnostic."""
        self.checks[name] = False
        super().check(name, condition)
        self.checks[name] = True

    def run_client(self):
        """Use real runtime decoding with expected fields independent of Rust."""
        self.wait_for(re.compile(r"Starting minecraft server version "
                                 + re.escape(self.args.version) + r"$"), "runtime-version")
        self.load_test_chunk("minecraft:overworld")
        root = Path(self.args.pack) / "data/sand_audit/instrument/typed_text"
        for name, expected in EXPECTED.items():
            description = json.loads((root / f"{name}.json").read_text())["description"]
            tag = f"sand_text_probe_{name}"
            target = f"@e[type=minecraft:text_display,tag={tag},limit=1]"
            nbt = json.dumps({"Tags": [tag], "text": description})
            self.send(f"summon minecraft:text_display 0 80 0 {nbt}")
            self.check(f"{name}_exists", f"if entity {target}")
            self.check(f"{name}_retained", f"if data entity {target} text.{expected}")
            self.send(f"kill {target}")


def main():
    """Run startup, round trips, reload and shutdown, retaining partial evidence."""
    args = parse_args(sys.argv[1:])
    if args.version != "26.2":
        raise ValueError("the text component codec oracle targets Minecraft 26.2")
    harness = TextInteractionHarness(args)
    evidence = {"minecraft": args.version, "complete": False, "checks": harness.checks}
    try:
        harness.run()
        evidence["complete"] = True
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        path = harness.output / "text-interaction-evidence.json"
        path.write_text(json.dumps(evidence, indent=2) + "\n")
        print(f"Text interaction evidence: {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
