#!/usr/bin/env python3
"""Prove Sand's generated 26.3 compute builders against a disposable vanilla server."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from vanilla_reload_harness import ServerHarness, STARTUP, ValidationFailure


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=True)
    pack = root / "pack"
    env = dict(os.environ, SAND_MC_VERSION="26.3", SAND_STRICT_CODEGEN="1", SAND_COMPUTE_PROOF_PACK=str(pack))
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "compute_26_3"], env=env, check=True)
    settings = argparse.Namespace(version="26.3", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    harness = ServerHarness(settings)
    harness.prepare()
    properties = harness.server / "server.properties"
    with properties.open("a") as stream:
        stream.write("pause-when-empty-seconds=0\n")
    harness.start()
    try:
        ready = harness.wait_for(STARTUP, "startup")
        if harness.errors(0, ready + 1):
            raise ValidationFailure("startup", "compute fixture failed to load", harness.errors(0, ready + 1))
        harness.wait_for("__SAND_COMPUTE_LOADED__", "initial load")
        for phase in ("initial", "reload"):
            if phase == "reload":
                reload_start = len(harness.lines)
                harness.send("reload")
                harness.wait_for("__SAND_COMPUTE_LOADED__", "reload load tag", reload_start)
            offset = len(harness.lines)
            harness.send("forceload add 0 0")
            harness.send("schedule function compute_probe:run 20t")
            end = harness.wait_for("__SAND_COMPUTE_END__", phase, offset)
            if not any("__SAND_COMPUTE_PASS__" in line for line in harness.lines[offset:end + 1]):
                raise ValidationFailure(phase, "compute result, rounding, failure, or entity isolation assertion failed")
        (root / "evidence.json").write_text(json.dumps({"minecraft":"26.3", "passed":True, "checks":["integer addition", "float result floor", "square root", "block context", "entity context isolation", "division failure", "repeat after reload"], "log":str(harness.log_path)}, indent=2) + "\n")
    finally:
        harness.stop()


if __name__ == "__main__":
    main()
