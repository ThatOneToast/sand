#!/usr/bin/env python3
"""Load exported sound resources in 26.2 and resolve each through an item codec.

Build sand-vanilla-audit through `sand build` first; pass its unchanged dist pack
and a verified 26.2 server jar using the standard reload harness arguments.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import sys
import subprocess

from vanilla_reload_harness import parse_args
from validate_world_gameplay import WorldGameplayHarness


class SoundEventHarness(WorldGameplayHarness):
    def __init__(self, args):
        super().__init__(args)
        self.checks = {}

    def check(self, name, condition):
        self.checks[name] = False
        super().check(name, condition)
        self.checks[name] = True

    def run_client(self):
        self.wait_for(re.compile(r"Starting minecraft server version 26\.2$"), "runtime-version")
        self.load_test_chunk("minecraft:overworld")
        for kind, item_id, component in [
            ("instrument", "minecraft:goat_horn", "minecraft:instrument"),
            ("jukebox_song", "minecraft:music_disc_13", "minecraft:jukebox_playable"),
        ]:
            for name in ["reference", "inline", "fixed", "zero", "negative"]:
                resource_id = f"sand_audit:sound/{name}"
                value = resource_id if kind == "instrument" else {"song": resource_id}
                tag = f"sand_sound_{kind}_{name}"
                item = {"id": item_id, "count": 1, "components": {component: value}}
                nbt = json.dumps({"Tags": [tag], "Item": item})
                target = f"@e[type=minecraft:item,tag={tag},limit=1]"
                self.send(f"summon minecraft:item 0 80 0 {nbt}")
                # ItemStack decoding resolves the dynamic registry reference. An
                # unloaded instrument/song cannot survive this parse as this item.
                self.check(f"{kind}_{name}_resolved", f'if data entity {target} Item.components{json.dumps({component: value})}')
                self.send(f"kill {target}")


def main():
    args = parse_args(sys.argv[1:])
    if args.version != "26.2":
        raise ValueError("sound-event fixture oracle targets Minecraft 26.2")
    evidence = {"minecraft": "26.2", "complete": False, "resources": {}}
    root = Path(args.pack)
    repo = Path(__file__).resolve().parents[1]
    evidence["source_head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    evidence["source_dirty"] = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=repo, text=True).strip())
    evidence["source_sha256"] = {
        name: hashlib.sha256((repo / name).read_bytes()).hexdigest()
        for name in ["sand-vanilla-audit/src/lib.rs", "scripts/validate_sound_events.py", "scripts/mc_validation/SoundEventResourceProbe.java", "scripts/vanilla_reload_harness.py", "scripts/validate_world_gameplay.py"]
    }
    evidence["server_jar_sha256"] = hashlib.sha256(Path(args.jar).read_bytes()).hexdigest()
    pack_files = {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(root.rglob("*")) if path.is_file()}
    evidence["pack_sha256"] = hashlib.sha256(json.dumps(pack_files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    evidence["pack_file_count"] = len(pack_files)
    for kind in ["instrument", "jukebox_song"]:
        for name in ["reference", "inline", "fixed", "zero", "negative"]:
            path = Path("data/sand_audit") / kind / "sound" / f"{name}.json"
            content = (root / path).read_bytes()
            evidence["resources"][str(path)] = {"sha256": hashlib.sha256(content).hexdigest(), "json": json.loads(content)}
    harness = SoundEventHarness(args)
    evidence["checks"] = harness.checks
    try:
        harness.run()
        evidence["complete"] = True
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        (harness.output / "sound-event-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
