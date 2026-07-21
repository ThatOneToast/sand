#!/usr/bin/env python3
"""Bounded, synchronous real-server validation harness for `sand run` (#278).

Starts the actual `sand run` binary against a real Minecraft 26.2 server
jar, drives it to a terminal state (readiness / degraded / fatal failure)
within a fixed timeout, sends `stop` through its stdin, and waits a bounded
amount of time for clean shutdown before falling back to terminate/kill.
Never backgrounds a wait, never uses a FIFO, never depends on an
asynchronous notification arriving later — every step here is a blocking
call with an explicit timeout, driven entirely inside this process.

Usage:
    python3 scripts/mc_validation/run_harness.py --scenario broken --mode classified
    python3 scripts/mc_validation/run_harness.py --scenario clean --mode all
    python3 scripts/mc_validation/run_harness.py --scenario both --mode all

Requires a debug `sand` binary already built (`cargo build -p sand-cli
--bin sand`) and the 26.2 server jar already cached
(`cargo run -p sand-build --bin ensure-server-jar -- 26.2`).
"""

from __future__ import annotations

import argparse
import json
import queue
import re
import subprocess
import sys
import threading
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_DIR = REPO_ROOT / "examples" / "participant_audit"
SAND_BIN = REPO_ROOT / "target" / "debug" / "sand"
DIST_DIR = PACK_DIR / "dist" / "paudit"

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")

# A deliberately broken function: line 6 (1-indexed) is not a valid
# command, reproducing the real Minecraft 26.2 "Failed to load function" /
# "Whilst parsing command on line 6" / "Couldn't load tag minecraft:load"
# failure chain that sand-cli/src/console/render.rs's fixture tests cover.
BROKEN_FUNCTION = (
    "say line 1\n"
    "say line 2\n"
    "say line 3\n"
    "say line 4\n"
    "say line 5\n"
    "this is not a command\n"
    "say line 7\n"
)

ALL_MODES = ["classified", "verbose", "raw", "json"]


def strip_ansi(line: str) -> str:
    return ANSI_RE.sub("", line)


def cleanup_stale_processes() -> None:
    for pattern in ("target/debug/sand run", "server.jar"):
        subprocess.run(["pkill", "-TERM", "-f", pattern], check=False)
    time.sleep(1)
    for pattern in ("target/debug/sand run", "server.jar"):
        subprocess.run(["pkill", "-KILL", "-f", pattern], check=False)


def check_no_stray_processes() -> str:
    result = subprocess.run(
        ["pgrep", "-fl", "target/debug/sand run|server.jar"],
        capture_output=True,
        text=True,
        check=False,
    )
    return result.stdout.strip()


def build_pack() -> None:
    subprocess.run(
        [str(SAND_BIN), "build"], cwd=PACK_DIR, check=True, timeout=180,
        capture_output=True, text=True,
    )


def remove_injected_broken_function() -> None:
    """`sand build` does not clean stale files out of dist/ before writing,
    so anything injected by inject_broken_function() would otherwise leak
    into a later clean-scenario run against the same dist/ directory."""
    on_load = DIST_DIR / "data" / "paudit" / "function" / "on_load.mcfunction"
    on_load.unlink(missing_ok=True)
    load_tag = DIST_DIR / "data" / "minecraft" / "tags" / "function" / "load.json"
    load_tag.unlink(missing_ok=True)


def inject_broken_function() -> None:
    func_dir = DIST_DIR / "data" / "paudit" / "function"
    func_dir.mkdir(parents=True, exist_ok=True)
    (func_dir / "on_load.mcfunction").write_text(BROKEN_FUNCTION, encoding="utf-8")

    tag_dir = DIST_DIR / "data" / "minecraft" / "tags" / "function"
    tag_dir.mkdir(parents=True, exist_ok=True)
    load_tag = tag_dir / "load.json"
    load_tag.write_text(
        json.dumps({"values": ["paudit:on_load"]}, indent=2), encoding="utf-8"
    )


class ReaderThread:
    """Feeds subprocess stdout lines into a queue so the main loop can wait
    on them with an explicit, bounded timeout instead of blocking forever
    on readline()."""

    def __init__(self, stream) -> None:
        self.line_queue: "queue.Queue[str | None]" = queue.Queue()
        self._thread = threading.Thread(target=self._run, args=(stream,), daemon=True)
        self._thread.start()

    def _run(self, stream) -> None:
        try:
            for line in stream:
                self.line_queue.put(line)
        finally:
            self.line_queue.put(None)


def run_scenario(scenario: str, mode: str, ready_timeout: float, stop_timeout: float) -> dict:
    """Run one bounded `sand run` invocation and return a result dict.

    Every wait in this function has an explicit deadline; nothing here can
    block indefinitely, and the child process is guaranteed dead (terminate
    then kill) before this function returns.
    """
    args = [str(SAND_BIN), "run", "--no-build", "--offline", "--server-log", mode]
    print(f"\n=== scenario={scenario} mode={mode} ===")
    process = subprocess.Popen(
        args,
        cwd=PACK_DIR,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    reader = ReaderThread(process.stdout)
    captured: list[str] = []
    outcome: dict = {
        "scenario": scenario,
        "mode": mode,
        "ready": False,
        "fatal": False,
        "timed_out": False,
        "process_exited_early": False,
        "final_health": None,
        "stopped_cleanly": False,
    }

    # JSON mode never emits a readiness signal (only diagnostics and a
    # final `{"health": ...}` line) -- see render.rs: Category::Ready is
    # swallowed in Json mode. So for json we bound the wait by a fixed
    # settle window instead of watching for a marker; every other mode
    # bounds its wait by `ready_timeout` via a readiness marker instead.
    json_settle_seconds = min(45.0, ready_timeout)
    start = time.monotonic()
    deadline = start + ready_timeout
    try:
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                outcome["timed_out"] = True
                break
            if mode == "json" and (time.monotonic() - start) >= json_settle_seconds:
                outcome["ready"] = True  # settled quietly; proceed to stop
                break
            try:
                line = reader.line_queue.get(timeout=min(remaining, 1.0))
            except queue.Empty:
                continue
            if line is None:
                outcome["process_exited_early"] = True
                break

            captured.append(line)
            clean = strip_ansi(line).rstrip("\n")
            print(clean)

            if mode == "raw":
                if "Done (" in clean and "For help" in clean:
                    outcome["ready"] = True
                    break
            elif mode == "json":
                try:
                    obj = json.loads(clean)
                except json.JSONDecodeError:
                    obj = None
                if obj and obj.get("fatality") == "fatal_to_startup":
                    outcome["fatal"] = True
                    break
            else:  # classified / verbose
                if "Minecraft" in clean and "ready" in clean:
                    outcome["ready"] = True
                    break
                if "did not start successfully" in clean:
                    outcome["fatal"] = True
                    break
    finally:
        # Bounded shutdown: try a clean `stop`, then terminate, then kill.
        # No step here can wait longer than its own timeout.
        if process.poll() is None:
            try:
                process.stdin.write("stop\n")
                process.stdin.flush()
            except Exception:
                pass
            drain_deadline = time.monotonic() + stop_timeout
            while time.monotonic() < drain_deadline:
                try:
                    line = reader.line_queue.get(timeout=max(0.0, drain_deadline - time.monotonic()))
                except queue.Empty:
                    break
                if line is None:
                    break
                captured.append(line)
                print(strip_ansi(line).rstrip("\n"))
            try:
                process.wait(timeout=max(0.0, drain_deadline - time.monotonic()) + 1)
                outcome["stopped_cleanly"] = True
            except subprocess.TimeoutExpired:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)
        else:
            outcome["stopped_cleanly"] = True

        # Drain anything left in the queue without blocking further.
        while True:
            try:
                line = reader.line_queue.get_nowait()
            except queue.Empty:
                break
            if line is None:
                break
            captured.append(line)
            print(strip_ansi(line).rstrip("\n"))

    # Determine final health from captured output.
    clean_lines = [strip_ansi(l).rstrip("\n") for l in captured]
    if mode == "json":
        for line in reversed(clean_lines):
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            if set(obj.keys()) == {"health"}:
                outcome["final_health"] = obj["health"]
                break
    elif mode in ("classified", "verbose"):
        joined = "\n".join(clean_lines)
        if "did not start successfully" in joined:
            outcome["final_health"] = "failed"
        elif "process started, but the datapack failed to load" in joined:
            outcome["final_health"] = "degraded"
        elif outcome["ready"]:
            outcome["final_health"] = "healthy"
    else:  # raw: no health tracking, by design
        outcome["final_health"] = None

    outcome["stray_processes"] = check_no_stray_processes()
    return outcome


def expected_health(scenario: str) -> str:
    return "healthy" if scenario == "clean" else "degraded"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", choices=["broken", "clean", "both"], default="both")
    parser.add_argument("--mode", choices=[*ALL_MODES, "all"], default="all")
    parser.add_argument("--ready-timeout", type=float, default=90.0)
    parser.add_argument("--stop-timeout", type=float, default=30.0)
    args = parser.parse_args()

    scenarios = ["broken", "clean"] if args.scenario == "both" else [args.scenario]
    modes = ALL_MODES if args.mode == "all" else [args.mode]

    print("== Cleaning up any stale processes before starting ==")
    cleanup_stale_processes()
    stray = check_no_stray_processes()
    if stray:
        print(f"error: stray processes still present, aborting:\n{stray}", file=sys.stderr)
        return 2
    print("no stray processes")

    all_results: list[dict] = []
    ok = True
    try:
        for scenario in scenarios:
            print(f"\n##### building pack for scenario={scenario} #####")
            build_pack()
            remove_injected_broken_function()
            if scenario == "broken":
                inject_broken_function()

            for mode in modes:
                result = run_scenario(scenario, mode, args.ready_timeout, args.stop_timeout)
                all_results.append(result)

                if mode != "raw":
                    want = expected_health(scenario)
                    got = result["final_health"]
                    result["expected_health"] = want
                    result["health_matches_expectation"] = got == want
                    if got != want:
                        ok = False
                        print(f"MISMATCH: expected health={want!r}, got {got!r}")

                if result["stray_processes"]:
                    ok = False
                    print(f"MISMATCH: stray processes after run:\n{result['stray_processes']}")

                if not result["stopped_cleanly"]:
                    ok = False
                    print("MISMATCH: process required terminate/kill instead of clean stop")
    finally:
        cleanup_stale_processes()

    print("\n===== SUMMARY =====")
    for r in all_results:
        print(json.dumps(r))

    final_stray = check_no_stray_processes()
    if final_stray:
        print(f"error: stray processes present after full run:\n{final_stray}", file=sys.stderr)
        ok = False
    else:
        print("no stray processes after full run")

    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
