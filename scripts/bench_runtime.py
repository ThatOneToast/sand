#!/usr/bin/env python3
"""Benchmark a real bench-profile server without touching unrelated processes."""

from __future__ import annotations

import argparse
import json
import re
import secrets
import socket
import statistics
import subprocess
import sys
import threading
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
sys.path.insert(0, str(SCRIPT_DIR / "mc_validation"))
from rcon_client import run_commands  # noqa: E402

READY = re.compile(r"Minecraft .* ready|Done \([^)]+\)! For help")
MSPT_PATTERNS = [
    re.compile(r"([0-9]+(?:\.[0-9]+)?)\s*ms per tick", re.IGNORECASE),
    re.compile(r"Average time per tick:\s*([0-9]+(?:\.[0-9]+)?)\s*ms", re.IGNORECASE),
]


def parse_mspt(response: str) -> float:
    for pattern in MSPT_PATTERNS:
        if match := pattern.search(response):
            return float(match.group(1))
    raise ValueError(f"tick query did not report ms/tick: {response!r}")


def available_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


class OwnedServer:
    def __init__(self, command: list[str], cwd: Path, log: Path):
        self.command = command
        self.cwd = cwd
        self.log = log
        self.lines: list[str] = []
        self.process: subprocess.Popen[str] | None = None
        self.reader: threading.Thread | None = None

    def start(self) -> None:
        self.process = subprocess.Popen(
            self.command,
            cwd=self.cwd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )

        def read() -> None:
            assert self.process and self.process.stdout
            with self.log.open("w", encoding="utf-8") as output:
                for line in self.process.stdout:
                    output.write(line)
                    output.flush()
                    self.lines.append(line.rstrip())

        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def wait_ready(self, timeout: float) -> None:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if any(READY.search(line) for line in self.lines):
                return
            if self.process and self.process.poll() is not None:
                raise RuntimeError(f"server exited with {self.process.returncode}; see {self.log}")
            time.sleep(0.1)
        raise TimeoutError(f"server did not become ready in {timeout}s; see {self.log}")

    def stop(self, port: int, password: str) -> None:
        if not self.process or self.process.poll() is not None:
            return
        try:
            run_commands("127.0.0.1", port, password, ["stop"])
            self.process.wait(timeout=30)
        except Exception:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        if self.reader:
            self.reader.join(timeout=2)


def write_rcon_properties(project: Path, port: int, password: str) -> None:
    server = project / "dist" / "server"
    server.mkdir(parents=True, exist_ok=True)
    properties = server / "server.properties"
    existing = properties.read_text(encoding="utf-8") if properties.exists() else ""
    managed = {"enable-rcon", "rcon.password", "rcon.port", "broadcast-rcon-to-ops"}
    lines = [line for line in existing.splitlines() if line.split("=", 1)[0] not in managed]
    lines.extend([
        "enable-rcon=true",
        f"rcon.password={password}",
        f"rcon.port={port}",
        "broadcast-rcon-to-ops=false",
    ])
    # Minecraft requires the RCON secret in server.properties. This disposable
    # benchmark file is restricted to the current user.
    properties.write_text(
        "\n".join(lines) + "\n",  # lgtm[py/clear-text-storage-sensitive-data]
        encoding="utf-8",
    )
    properties.chmod(0o600)


def loaded(response: str) -> bool:
    lowered = response.lower()
    return "not loaded" not in lowered and "time is" in lowered


def measure_sample(sand: Path, project: Path, sample: int, timeout: float, output: Path) -> dict[str, float]:
    port = available_port()
    password = secrets.token_urlsafe(24)
    write_rcon_properties(project, port, password)
    log = output / f"sample-{sample}.log"
    server = OwnedServer(
        [str(sand), "run", "--no-build", "--offline", "--profile", "bench"], project, log
    )
    started = time.monotonic()
    server.start()
    try:
        server.wait_ready(timeout)
        startup_seconds = time.monotonic() - started
        tick_response = run_commands("127.0.0.1", port, password, ["tick query"])[0]
        mspt = parse_mspt(tick_response)

        block = 2_000_000 + sample * 8192
        end = block + 255
        probes = [(block, block), (end, block), (block, end), (end, end), (block + 128, block + 128)]
        generation_started = time.monotonic()
        run_commands(
            "127.0.0.1", port, password,
            [f"execute in minecraft:overworld run forceload add {block} {block} {end} {end}"],
        )
        deadline = time.monotonic() + timeout
        while True:
            responses = run_commands(
                "127.0.0.1", port, password,
                [f"execute in minecraft:overworld if loaded {x} 80 {z} run time query gametime" for x, z in probes],
            )
            if all(loaded(response) for response in responses):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError("timed out waiting for forced chunks to finish loading")
            time.sleep(0.1)
        generation_seconds = time.monotonic() - generation_started
        run_commands(
            "127.0.0.1", port, password,
            [f"execute in minecraft:overworld run forceload remove {block} {block} {end} {end}"],
        )
        return {
            "startup_seconds": round(startup_seconds, 4),
            "ms_per_tick": round(mspt, 4),
            "chunks": 256,
            "chunk_generation_seconds": round(generation_seconds, 4),
            "chunks_per_second": round(256 / generation_seconds, 4),
        }
    finally:
        server.stop(port, password)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("samples", nargs="?", type=int, default=1)
    parser.add_argument("project", nargs="?", default="examples/book_project")
    parser.add_argument("--timeout", type=float, default=180)
    parser.add_argument("--output", default="target/bench-runtime")
    args = parser.parse_args()
    if args.samples < 1:
        parser.error("samples must be positive")
    project = (REPO_ROOT / args.project).resolve()
    sand = REPO_ROOT / "target" / "debug" / "sand"
    output = (REPO_ROOT / args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    subprocess.run(["cargo", "build", "-p", "sand-cli"], cwd=REPO_ROOT, check=True)
    subprocess.run([str(sand), "build", "--profile", "bench"], cwd=project, check=True)
    samples = [measure_sample(sand, project, index, args.timeout, output) for index in range(1, args.samples + 1)]
    result = {
        "schema_version": 1,
        "profile": "bench",
        "project": str(project.relative_to(REPO_ROOT)),
        "samples": samples,
        "median_ms_per_tick": statistics.median(item["ms_per_tick"] for item in samples),
        "median_chunks_per_second": statistics.median(item["chunks_per_second"] for item in samples),
    }
    (output / "results.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
