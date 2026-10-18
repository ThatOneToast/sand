#!/usr/bin/env python3
"""Prove portable global State initialization and mutation without player entities."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from measure_archetype_idle import verified_runtime_version
from validate_world_gameplay import WorldGameplayHarness
from vanilla_reload_harness import RELOAD_START

ROOT = Path(__file__).resolve().parents[1]


class GlobalStateHarness(WorldGameplayHarness):
    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({"name": name, "condition": condition, "passed": True})

    def value(self, name, value):
        self.check(name, f"if score {self.holder} {self.objective} matches {value}")

    def run_client(self):
        verified_runtime_version(self.lines)
        self.send("tick freeze")
        self.check("NO_PLAYERS", "unless entity @a")
        self.value("INITIALIZED_WITHOUT_EXECUTOR", 10)
        self.send("function demo:choose")
        self.value("GLOBAL_BRANCH_EVALUATES_ONCE", 15)
        self.send("function demo:increment")
        self.value("SERVER_INCREMENT", 16)
        self.send("function demo:choose")
        self.value("GLOBAL_OTHERWISE", 1)
        self.send("function demo:fill")
        self.value("BOUNDED_ASSIGNMENT", 100)
        self.send("function demo:subtract")
        self.value("SIGNED_MUTATION", 97)
        start = len(self.lines)
        self.send("reload")
        self.wait_for(RELOAD_START, "global-reload", start)
        # Explicitly repeat the exported load entry too: its initialization
        # must remain idempotent even when hosts call it more than once.
        self.send("function demo:__sand_lifecycle_load")
        self.value("RELOAD_AND_REPEATED_LOAD_PRESERVE_STATE", 97)
        self.send("function demo:increment")
        self.value("POST_RELOAD_INCREMENT", 98)
        self.check("STILL_NO_PLAYERS", "unless entity @a")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    result = subprocess.run([str(args.binary.resolve()), "program", "compile", "--input",
        str(ROOT / "tests/fixtures/program/global.sand.json"), "--output", str(root / "pack")],
        check=True, capture_output=True, text=True, timeout=60)
    response = json.loads(result.stdout)
    assert response["success"]
    body = (root / "pack/data/demo/function/increment.mcfunction").read_text()
    matches = re.findall(r"^scoreboard players add (#sand_\S+) (\S+) 1$", body, re.MULTILINE)
    assert len(matches) == 1, body
    holder, objective = matches[0]
    digest = hashlib.sha256()
    for path in sorted((root / "pack").rglob("*")):
        if path.is_file() and path.name != ".sand-build-manifest.json":
            digest.update(path.relative_to(root / "pack").as_posix().encode() + b"\0")
            digest.update(path.read_bytes() + b"\0")
    options = argparse.Namespace(version="26.2", pack=str(root / "pack"), jar=str(args.jar.resolve()),
        java=args.java, timeout=90, output=str(root / "logs"), op_player=None, client_command=None)
    harness = GlobalStateHarness(options)
    harness.holder, harness.objective, harness.checks = holder, objective, []
    passed = False
    try:
        harness.run()
        passed = True
    finally:
        (root / "evidence.json").write_text(json.dumps({"minecraft": "26.2", "passed": passed,
            "scope": "server execution with zero player entities; automatic ticks frozen",
            "pack_sha256": digest.hexdigest(), "checks": harness.checks}, indent=2) + "\n")
    print("EVIDENCE", root / "evidence.json")


if __name__ == "__main__":
    main()
