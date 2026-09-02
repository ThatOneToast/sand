#!/usr/bin/env python3
"""Exercise dev -> release world-build profile switching on real vanilla."""

from __future__ import annotations

import argparse
import re
import subprocess
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
READY = re.compile(r"Minecraft .* ready|Done \([^)]+\)! For help")
ERROR = re.compile(
    r"Failed to load|Could not load|Couldn't load|Failed to parse|Unknown (?:command|function|tag)|"
    r"DatapackLoadFailedException|Exception in server tick loop|\[Server thread/ERROR\]",
    re.IGNORECASE,
)


def parse_properties(path: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line and not line.startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            result[key] = value
    return result


def run_profile(sand: Path, project: Path, profile: str, timeout: float, output: Path) -> dict[str, str]:
    subprocess.run([str(sand), "build", "--profile", profile], cwd=project, check=True)
    process = subprocess.Popen(
        [str(sand), "run", "--no-build", "--offline", "--profile", profile],
        cwd=project,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    lines: list[str] = []
    deadline = time.monotonic() + timeout
    try:
        assert process.stdout
        while time.monotonic() < deadline:
            line = process.stdout.readline()
            if line:
                lines.append(line.rstrip())
                if READY.search(line):
                    break
            elif process.poll() is not None:
                raise RuntimeError(f"{profile} server exited with {process.returncode}")
            else:
                time.sleep(0.05)
        else:
            raise TimeoutError(f"{profile} server did not become ready in {timeout}s")

        errors = [line for line in lines if ERROR.search(line)]
        if errors:
            raise RuntimeError(f"{profile} datapack errors: {' | '.join(errors)}")
        properties = parse_properties(project / "dist" / "server" / "server.properties")
        dimension = (project / "dist" / "trail" / "data" / "minecraft" / "dimension" / "overworld.json")
        properties["__dimension"] = dimension.read_text(encoding="utf-8")
        return properties
    finally:
        if process.poll() is None:
            try:
                assert process.stdin
                process.stdin.write("stop\n")
                process.stdin.flush()
                process.wait(timeout=30)
            except (BrokenPipeError, subprocess.TimeoutExpired):
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
        output.mkdir(parents=True, exist_ok=True)
        (output / f"{profile}.log").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", default="examples/book_project")
    parser.add_argument("--timeout", type=float, default=180)
    parser.add_argument("--output", default="target/worldbuild-profiles")
    args = parser.parse_args()
    project = (REPO_ROOT / args.project).resolve()
    output = (REPO_ROOT / args.output).resolve()
    sand = REPO_ROOT / "target" / "debug" / "sand"
    subprocess.run(["cargo", "build", "-p", "sand-cli"], cwd=REPO_ROOT, check=True)

    dev = run_profile(sand, project, "dev", args.timeout, output)
    release = run_profile(sand, project, "release", args.timeout, output)
    if dev.get("view-distance") != "6" or release.get("view-distance") != "10":
        raise RuntimeError("managed server.properties did not reconcile dev -> release view distance")
    # Vanilla rewrites an omitted random seed as an empty property on shutdown.
    if dev.get("level-seed") != "1337" or release.get("level-seed") not in (None, ""):
        raise RuntimeError("managed server.properties did not remove the dev-only fixed seed")
    if '"minecraft:flat"' not in dev["__dimension"]:
        raise RuntimeError("dev profile did not produce a flat Overworld")
    if '"minecraft:noise"' not in release["__dimension"]:
        raise RuntimeError("release profile did not replace it with a noise Overworld")
    print("dev -> release world-build profile switching passed on a real server")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
