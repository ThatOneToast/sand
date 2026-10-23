#!/usr/bin/env python3
"""Prove exported biome audio loads, reloads, and survives native 26.2 decoding."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from vanilla_reload_harness import parse_args
from validate_world_gameplay import WorldGameplayHarness

CASES = ("reference", "inline", "fixed", "zero", "negative")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class BiomeAmbientHarness(WorldGameplayHarness):
    def __init__(self, args):
        super().__init__(args)
        self.checks = {}

    def run_client(self):
        self.wait_for(re.compile(r"Starting minecraft server version 26\.2$"), "runtime-version")
        self.load_test_chunk("minecraft:overworld")
        for name in CASES:
            resource = f"sand_audit:ambient/{name}"
            self.checks[name] = False
            self.send(f"execute in minecraft:overworld run fillbiome 0 80 0 0 80 0 {resource}")
            self.check(name, f"in minecraft:overworld if biome 0 80 0 {resource}")
            self.checks[name] = True


def main():
    args = parse_args(sys.argv[1:])
    if args.version != "26.2":
        raise ValueError("biome ambient oracle requires Minecraft 26.2")
    repo = Path(__file__).resolve().parents[1]
    dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=repo, text=True).strip()
    if dirty:
        raise ValueError("native evidence requires a clean committed source tree")
    root = Path(args.pack).resolve()
    files = {path.relative_to(root).as_posix(): sha256(path)
             for path in sorted(root.rglob("*")) if path.is_file()}
    sources = ["sand-vanilla-audit/src/lib.rs", "scripts/validate_biome_ambient_sounds.py",
               "scripts/mc_validation/BiomeAmbientSoundProbe.java", "scripts/vanilla_reload_harness.py",
               "scripts/validate_world_gameplay.py"]
    evidence = {
        "minecraft": "26.2", "complete": False,
        "source_head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
        "source_dirty": False,
        "source_sha256": {name: sha256(repo / name) for name in sources},
        "server_jar_sha256": sha256(Path(args.jar)),
        "pack_file_count": len(files),
        "pack_sha256": hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
        "resources": {},
    }
    for name in CASES:
        relative = f"data/sand_audit/worldgen/biome/ambient/{name}.json"
        evidence["resources"][relative] = {"sha256": files[relative], "json": json.loads((root / relative).read_text())}
    harness = BiomeAmbientHarness(args)
    evidence["checks"] = harness.checks
    evidence["port"] = harness.port
    try:
        harness.run()
        evidence["lifecycle"] = {"startup": True, "reload": True, "shutdown": True}
        server_jars = sorted((harness.server / "versions").rglob("*.jar"))
        evidence["extracted_server_jar_sha256"] = {path.relative_to(harness.server).as_posix(): sha256(path) for path in server_jars}
        jars = server_jars + sorted((harness.server / "libraries").rglob("*.jar"))
        classpath = os.pathsep.join(map(str, jars))
        probe_dir = harness.output / "probe"
        probe_dir.mkdir()
        javac = str(Path(args.java).with_name("javac")) if Path(args.java).parent != Path(".") else "javac"
        commands = [
            ("javac", [javac, "-cp", classpath, "-d", str(probe_dir), str(repo / sources[2])]),
            ("codec_probe", [args.java, "-Xmx1G", "-cp", os.pathsep.join([str(probe_dir), classpath]),
                             "BiomeAmbientSoundProbe", str(root)]),
        ]
        for phase, command in commands:
            log = harness.output / f"{phase}.log"
            try:
                result = subprocess.run(command, text=True, stdout=subprocess.PIPE,
                                        stderr=subprocess.STDOUT, timeout=args.timeout)
            except subprocess.TimeoutExpired as error:
                output = error.output or ""
                if isinstance(output, bytes):
                    output = output.decode(errors="replace")
                log.write_text(output)
                evidence[phase] = {"timed_out": True, "timeout_seconds": args.timeout, "output": output}
                raise
            log.write_text(result.stdout)
            evidence[phase] = {"returncode": result.returncode, "output": result.stdout}
            result.check_returncode()
        evidence["complete"] = True
        print("PASSED: five biome registry resolutions, reload, shutdown, native attribute getters and re-encoding")
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        (harness.output / "biome-ambient-sound-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
