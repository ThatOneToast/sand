#!/usr/bin/env python3
"""Run the maintained 26.3 alpha scenarios and retain one failure/evidence bundle.

This is a repository scenario runner, not a general-purpose sand test command.
The clients are passive; success does not prove GUI input/rendering or MSPT.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SCENARIOS = {"roster": "validate_roster.py", "arena": "validate_cooperative_arena.py"}


def run(jar: Path, output: Path, scenario: str) -> int:
    jar = jar.resolve()
    output = output.resolve()
    if not jar.is_file():
        raise ValueError(f"Minecraft 26.3 server jar not found: {jar}")
    # Never overwrite a prior proof or let a typo repurpose a populated directory.
    output.mkdir(parents=True, exist_ok=False)
    environment = dict(os.environ)
    for key in ("CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG", "CARGO_INCREMENTAL"):
        environment.setdefault(key, "0")
    results = []
    selected = SCENARIOS if scenario == "all" else {scenario: SCENARIOS[scenario]}
    for name, script in selected.items():
        artifact_dir = output / name
        command = [sys.executable, str(ROOT / "scripts" / script), "--jar", str(jar), "--output", str(artifact_dir)]
        log = output / f"{name}.log"
        print(f"Running {name}; log: {log}", flush=True)
        error = None
        try:
            with log.open("w") as stream:
                completed = subprocess.run(command, cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT, check=False)
            code = completed.returncode
        except OSError as exc:
            code = 1
            error = str(exc)
        evidence_path = artifact_dir / "evidence.json"
        evidence = None
        try:
            evidence = json.loads(evidence_path.read_text())
            if not isinstance(evidence, dict) or evidence.get("passed") is not True or evidence.get("version") != "26.3":
                error = "Scenario did not produce passing 26.3 evidence"
        except (OSError, ValueError) as exc:
            error = f"Missing or invalid scenario evidence: {exc}"
        passed = code == 0 and error is None
        results.append({"scenario": name, "passed": passed, "exit_code": code,
                        "command": command, "log": log.name,
                        "evidence": str(evidence_path.relative_to(output)) if evidence_path.is_file() else None,
                        "error": error})
        # Save after each scenario so earlier results survive a later interruption.
        summary = {"version": "26.3", "passed": len(results) == len(selected) and all(r["passed"] for r in results),
                   "complete": len(results) == len(selected), "scenarios": results,
                   "limits": ["Passive clients and test-only shared gameplay actions", "No GUI/input/rendering proof", "No performance envelope or pack uninstall proof"]}
        (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(f"{name}: {'PASS' if passed else 'FAIL'}", flush=True)
    print(f"Summary: {output / 'summary.json'}")
    return 0 if all(r["passed"] for r in results) else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True, help="Cached Minecraft 26.3 server jar")
    parser.add_argument("--output", type=Path, required=True, help="New directory for logs and evidence (must not exist)")
    parser.add_argument("--scenario", choices=["all", *SCENARIOS], default="all")
    args = parser.parse_args()
    try:
        return run(args.jar, args.output, args.scenario)
    except (OSError, ValueError) as exc:
        parser.exit(2, f"alpha validation: {exc}\n")


if __name__ == "__main__":
    sys.exit(main())
