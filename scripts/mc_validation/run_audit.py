#!/usr/bin/env python3
"""Orchestrate real Minecraft Java 26.2 runtime validation for the
`participant_audit` datapack (#265).

What this actually does, and what it honestly cannot claim — see
`scripts/mc_validation/README.md` for the full category breakdown
(server startup/reload vs. one-player runtime vs. two-player runtime vs.
structural-only).

Usage:
    python3 scripts/mc_validation/run_audit.py --jar <path/to/server.jar>

Requires the `participant_audit` example already built (`sand build` from
`examples/participant_audit/`, or pass --build to do it here) and a real
Minecraft server jar (`cargo run -p sand-build --bin ensure-server-jar --
26.2`).
"""

from __future__ import annotations

import argparse
import json
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_PACK_DIR = REPO_ROOT / "examples" / "participant_audit"
RCON_CLIENT = Path(__file__).resolve().parent / "rcon_client.py"
JOIN_CLIENT = Path(__file__).resolve().parent / "minimal_join_client.py"
RCON_PASSWORD = "sand-audit-rcon"


def available_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def rcon(host: str, port: int, *commands: str) -> list[str]:
    result = subprocess.run(
        [sys.executable, str(RCON_CLIENT), host, str(port), RCON_PASSWORD, *commands],
        capture_output=True,
        text=True,
        timeout=30,
    )
    if result.returncode != 0:
        raise RuntimeError(f"RCON call failed: {result.stdout}\n{result.stderr}")
    return result.stdout.splitlines()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True, help="Path to a real Minecraft 26.2 server.jar")
    parser.add_argument("--java", default="java")
    parser.add_argument("--build", action="store_true", help="Run `sand build` first")
    parser.add_argument("--timeout", type=float, default=90)
    args = parser.parse_args()

    if args.build:
        subprocess.run(["cargo", "build", "-p", "sand-cli", "--bin", "sand"], cwd=REPO_ROOT, check=True)
        sand_bin = REPO_ROOT / "target" / "debug" / "sand"
        subprocess.run([str(sand_bin), "build"], cwd=AUDIT_PACK_DIR, check=True)

    dist = AUDIT_PACK_DIR / "dist" / "paudit"
    if not dist.is_dir():
        print(f"error: {dist} missing — run with --build or `sand build` in {AUDIT_PACK_DIR}", file=sys.stderr)
        return 2

    server_dir = Path(tempfile.mkdtemp(prefix="sand-participant-audit-"))
    world_datapacks = server_dir / "world" / "datapacks"
    world_datapacks.mkdir(parents=True)
    shutil.copytree(dist, world_datapacks / "paudit")
    (server_dir / "eula.txt").write_text("eula=true\n", encoding="utf-8")

    server_port = available_port()
    rcon_port = available_port()
    (server_dir / "server.properties").write_text(
        "\n".join(
            [
                "level-name=world",
                "online-mode=false",
                "max-players=4",
                "enable-rcon=true",
                f"rcon.password={RCON_PASSWORD}",
                f"rcon.port={rcon_port}",
                f"server-port={server_port}",
                "spawn-npcs=false",
                "spawn-animals=false",
                "spawn-monsters=false",
                "generate-structures=false",
                "view-distance=4",
                "simulation-distance=4",
                "sync-chunk-writes=false",
                "",
            ]
        ),
        encoding="utf-8",
    )

    print(f"== Starting real Minecraft 26.2 server (port {server_port}, rcon {rcon_port}) ==")
    process = subprocess.Popen(
        [args.java, "-jar", str(Path(args.jar).resolve()), "nogui"],
        cwd=server_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    results: dict[str, str] = {}
    try:
        deadline = time.monotonic() + args.timeout
        ready = False
        errors: list[str] = []
        assert process.stdout
        for line in process.stdout:
            print(line, end="")
            if "Done (" in line and "For help" in line:
                ready = True
                break
            if any(marker in line for marker in ("Exception", "ERROR", "Failed to load")):
                errors.append(line.strip())
            if time.monotonic() > deadline:
                break
        results["startup"] = "PASS" if ready and not errors else f"FAIL ({errors[:3]})"
        if not ready:
            return 1

        print("\n== Real command-level checks (RCON, no player required) ==")
        out = rcon("127.0.0.1", rcon_port, "datapack list")
        results["datapack_loaded"] = "PASS" if "paudit" in "\n".join(out) else "FAIL"

        out = rcon("127.0.0.1", rcon_port, "function paudit:init")
        results["init_function_runs"] = "PASS" if "Running function" in "\n".join(out) else "FAIL"

        out = rcon("127.0.0.1", rcon_port, "data get storage paudit:audit")
        results["audit_storage_initialized"] = (
            "PASS" if "attacker" in "\n".join(out) and "present: 0b" in "\n".join(out) else "FAIL"
        )

        print("\n== Real /reload of the actual merged-#266 participant-plan pack ==")
        out = rcon("127.0.0.1", rcon_port, "reload")
        time.sleep(2)
        out2 = rcon("127.0.0.1", rcon_port, "datapack list")
        results["reload"] = "PASS" if "paudit" in "\n".join(out2) else "FAIL"

        print("\n== Best-effort real player join (see minimal_join_client.py docstring) ==")
        join = subprocess.run(
            [sys.executable, str(JOIN_CLIENT), "127.0.0.1", str(server_port), "776", "AuditRunner", "10"],
            capture_output=True,
            text=True,
            timeout=20,
        )
        joined = join.returncode == 0
        results["player_join"] = "PASS (joined, connection not sustained — see README)" if joined else "FAIL"
        print(join.stdout[-2000:])

    finally:
        try:
            rcon("127.0.0.1", rcon_port, "stop")
        except Exception:
            process.terminate()
        try:
            process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            process.kill()
        shutil.rmtree(server_dir, ignore_errors=True)

    print("\n== Summary ==")
    for key, value in results.items():
        print(f"{key}: {value}")
    print(json.dumps(results))
    return 0 if all(v.startswith("PASS") for v in results.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
