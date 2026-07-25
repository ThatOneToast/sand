#!/usr/bin/env python3
"""Prove the #272 bounded-item *storage primitive* against a real Minecraft
Java 26.2 server, in isolation from any event-graph code.

Background: the original bounded-item backend wrote arbitrary custom
top-level NBT keys onto entities (`data modify entity @s
__sand_bounded_item.<key> ...`). A live RCON round-trip proved vanilla
silently drops those writes. This script validates the *replacement*
backend before any Rust code is written against it:

    command storage (`sand:__bounded_item`), with the per-subject path
    segment supplied at runtime by a **function macro** whose substituted
    value is a **scoreboard-derived integer** player slot.

Checks (all RCON, no player client required — the subject key is a
scoreboard value, so fake players stand in for real ones):

  1. write   — persist a known item snapshot for subject slot 7
  2. read    — read it back, expect the exact item
  3. isolate — subject slot 8 sees nothing of slot 7's data
  4. replace — re-persist slot 7 with a different item, no remnants
  5. absent  — persisting from an absent source clears presence
  6. reload  — data survives `/reload`
  7. expire  — the expiry macro clears exactly one subject's slot

Usage:
    python3 scripts/mc_validation/run_bounded_item_proof.py \
        --jar ~/.sand/cache/26.2/server.jar

The stdout-drain threading below is load-bearing; see the long comment in
`run_audit.py` (a stalled pipe silently corrupts every post-startup RCON
check). Do not replace it with a bounded read or `break` out of it.
"""

from __future__ import annotations

import argparse
import json
import queue
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rcon_client  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
RCON_PASSWORD = "sand-bounded-rcon"
PACK_FORMAT = 107  # 26.2, per sand-core/src/version.rs

TRANSCRIPT: list[str] = []


def available_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def rcon(port: int, *commands: str) -> list[str]:
    """One response string per command, in order.

    Imported directly rather than shelled out to: `rcon_client`'s CLI
    interleaves a `>>> <command>` echo line with responses that may
    themselves be empty or multi-line, so splitting its stdout cannot
    reliably re-associate a response with the command that produced it —
    an off-by-one there would silently assert against the wrong output.
    """
    return rcon_client.run_commands("127.0.0.1", port, RCON_PASSWORD, list(commands))


def run(port: int, *commands: str) -> list[str]:
    """Issue commands and record an exact transcript of command -> output."""
    out = rcon(port, *commands)
    for cmd, response in zip(commands, out):
        TRANSCRIPT.append(f"> {cmd}")
        TRANSCRIPT.append(response)
        print(f"> {cmd}\n{response}")
    return out


def presence(port: int, slot: int) -> bool:
    """Evaluate the exact presence condition Sand generates, over RCON.

    `execute if data ... run say X` is unusable as an assertion here: `say`
    writes to the server log, not to the RCON response, so its result is
    invisible to this script. Storing the conditional's *success* into a
    scoreboard slot and reading that back keeps the assertion on the same
    channel as the command.
    """
    out = run(
        port,
        f"execute store success score #probe sand_pid if data storage sand:__bounded_item p{slot}.k1{{present:1b}}",
        "scoreboard players get #probe sand_pid",
    )
    return "has 1 " in out[1]


def write_pack(datapacks: Path) -> None:
    """Hand-write the minimal proof pack (deliberately NOT Sand-generated:
    this step validates the raw vanilla mechanism, so the pack must be
    readable at a glance and free of any Sand codegen assumptions)."""
    pack = datapacks / "bproof"
    fn = pack / "data" / "bproof" / "function"
    fn.mkdir(parents=True)
    (pack / "pack.mcmeta").write_text(
        json.dumps(
            {
                "pack": {
                    "pack_format": PACK_FORMAT,
                    # Mandatory above format 81, and omitting them makes
                    # `/reload`'s pack rediscovery throw before it ever gets
                    # to the functions under test.
                    "min_format": PACK_FORMAT,
                    "max_format": PACK_FORMAT,
                    "description": "sand #272 bounded item storage proof",
                }
            }
        ),
        encoding="utf-8",
    )

    # The persist primitive: reset-then-conditionally-copy-then-mark, exactly
    # the shape of ItemSnapshot::capture, but with the per-subject path
    # segment `p$(subject)` substituted from the macro source.
    (fn / "persist.mcfunction").write_text(
        "\n".join(
            [
                "$data modify storage sand:__bounded_item p$(subject).k1.present set value 0b",
                "$data modify storage sand:__bounded_item p$(subject).k1.item set value {}",
                "$execute if data storage bproof:src snap{present:1b} run data modify storage sand:__bounded_item p$(subject).k1.item set from storage bproof:src snap.item",
                "$execute if data storage bproof:src snap{present:1b} run data modify storage sand:__bounded_item p$(subject).k1.present set value 1b",
                "",
            ]
        ),
        encoding="utf-8",
    )
    (fn / "expire.mcfunction").write_text(
        "\n".join(
            [
                "$data modify storage sand:__bounded_item p$(subject).k1.present set value 0b",
                "$data modify storage sand:__bounded_item p$(subject).k1.item set value {}",
                "",
            ]
        ),
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()

    server_dir = Path(tempfile.mkdtemp(prefix="sand-bounded-item-proof-"))
    datapacks = server_dir / "world" / "datapacks"
    datapacks.mkdir(parents=True)
    write_pack(datapacks)
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
    log_lines: "queue.Queue[str]" = queue.Queue()
    startup_errors: list[str] = []

    def drain_server_stdout() -> None:
        assert process.stdout
        for line in process.stdout:
            log_lines.put(line)

    reader_thread = threading.Thread(target=drain_server_stdout, daemon=True)
    reader_thread.start()

    try:
        deadline = time.monotonic() + args.timeout
        ready = False
        while time.monotonic() < deadline:
            try:
                line = log_lines.get(timeout=0.5)
            except queue.Empty:
                continue
            print(line, end="")
            if "Done (" in line and "For help" in line:
                ready = True
                break
            if any(m in line for m in ("Exception", "ERROR", "Failed to load")):
                startup_errors.append(line.strip())
        results["startup"] = "PASS" if ready and not startup_errors else f"FAIL ({startup_errors[:3]})"
        if not ready:
            return 1

        def keep_draining_in_background() -> None:
            while True:
                try:
                    print(log_lines.get(timeout=1), end="")
                except queue.Empty:
                    if not reader_thread.is_alive() and log_lines.empty():
                        return

        threading.Thread(target=keep_draining_in_background, daemon=True).start()

        out = run(rcon_port, "datapack list")
        results["datapack_loaded"] = "PASS" if "bproof" in "\n".join(out) else "FAIL"

        print("\n== 0. subject slots come from a scoreboard objective ==")
        run(
            rcon_port,
            "scoreboard objectives add sand_pid dummy",
            "scoreboard players set #alice sand_pid 7",
            "scoreboard players set #bob sand_pid 8",
        )

        print("\n== 1/2. write + read back for subject slot 7 ==")
        run(
            rcon_port,
            'data modify storage bproof:src snap set value {present:1b,item:{id:"minecraft:diamond_sword",count:1}}',
            "execute store result storage bproof:args subject int 1 run scoreboard players get #alice sand_pid",
            "data get storage bproof:args subject",
            "function bproof:persist with storage bproof:args",
        )
        read = run(rcon_port, "data get storage sand:__bounded_item p7.k1")
        joined = "\n".join(read)
        results["write_read_roundtrip"] = (
            "PASS" if "diamond_sword" in joined and "present: 1b" in joined else f"FAIL ({joined})"
        )
        results["presence_condition_matches"] = "PASS" if presence(rcon_port, 7) else "FAIL"

        print("\n== 3. isolation: slot 8 must not see slot 7's data ==")
        iso = run(rcon_port, "data get storage sand:__bounded_item p8.k1")
        results["isolation_unwritten_subject"] = (
            "PASS" if "Found no elements" in iso[0] and not presence(rcon_port, 8) else f"FAIL ({iso})"
        )
        # And a *written* second subject must hold its own distinct value.
        run(
            rcon_port,
            'data modify storage bproof:src snap set value {present:1b,item:{id:"minecraft:golden_apple",count:3}}',
            "execute store result storage bproof:args subject int 1 run scoreboard players get #bob sand_pid",
            "function bproof:persist with storage bproof:args",
        )
        both = run(
            rcon_port,
            "data get storage sand:__bounded_item p7.k1.item",
            "data get storage sand:__bounded_item p8.k1.item",
        )
        results["isolation_two_written_subjects"] = (
            "PASS"
            if "diamond_sword" in both[0] and "golden_apple" in both[1] and "golden_apple" not in both[0]
            else f"FAIL ({both})"
        )

        print("\n== 4. replacement: rewrite slot 7, no remnants of the old item ==")
        run(
            rcon_port,
            'data modify storage bproof:src snap set value {present:1b,item:{id:"minecraft:stone",count:64}}',
            "execute store result storage bproof:args subject int 1 run scoreboard players get #alice sand_pid",
            "function bproof:persist with storage bproof:args",
        )
        rep = run(rcon_port, "data get storage sand:__bounded_item p7.k1.item")
        results["replacement_no_remnants"] = (
            "PASS" if "stone" in rep[0] and "diamond_sword" not in rep[0] else f"FAIL ({rep})"
        )
        # slot 8 untouched by slot 7's replacement
        untouched = run(rcon_port, "data get storage sand:__bounded_item p8.k1.item")
        results["replacement_isolation_holds"] = "PASS" if "golden_apple" in untouched[0] else f"FAIL ({untouched})"

        print("\n== 5. absent source clears presence rather than leaving stale data ==")
        run(
            rcon_port,
            "data remove storage bproof:src snap",
            "execute store result storage bproof:args subject int 1 run scoreboard players get #alice sand_pid",
            "function bproof:persist with storage bproof:args",
        )
        absent = run(rcon_port, "data get storage sand:__bounded_item p7.k1")
        results["absent_source_clears_presence"] = (
            "PASS"
            if "present: 0b" in absent[0] and "stone" not in absent[0] and not presence(rcon_port, 7)
            else f"FAIL ({absent})"
        )

        print("\n== 6. survives /reload ==")
        # Re-populate slot 7 so there is something meaningful to survive.
        run(
            rcon_port,
            'data modify storage bproof:src snap set value {present:1b,item:{id:"minecraft:diamond_sword",count:1}}',
            "execute store result storage bproof:args subject int 1 run scoreboard players get #alice sand_pid",
            "function bproof:persist with storage bproof:args",
        )
        rel = run(rcon_port, "reload")
        time.sleep(3)
        after = run(
            rcon_port,
            "data get storage sand:__bounded_item p7.k1.item",
            "data get storage sand:__bounded_item p8.k1.item",
        )
        aft = "\n".join(after)
        results["reload"] = "PASS" if "reload" in "\n".join(rel).lower() else f"FAIL ({rel})"
        results["survives_reload"] = (
            "PASS"
            if "diamond_sword" in after[0] and "golden_apple" in after[1] and presence(rcon_port, 7)
            else f"FAIL ({aft})"
        )
        # The macro function must still *work* after reload, not merely parse:
        # write a new, distinguishable value through it and read that back.
        run(
            rcon_port,
            'data modify storage bproof:src snap set value {present:1b,item:{id:"minecraft:emerald",count:5}}',
            "execute store result storage bproof:args subject int 1 run scoreboard players get #alice sand_pid",
            "function bproof:persist with storage bproof:args",
        )
        post = run(rcon_port, "data get storage sand:__bounded_item p7.k1.item")
        results["macro_fn_works_after_reload"] = (
            "PASS" if "emerald" in post[0] and "diamond_sword" not in post[0] else f"FAIL ({post})"
        )

        print("\n== 7. expiry clears exactly one subject ==")
        run(
            rcon_port,
            "execute store result storage bproof:args subject int 1 run scoreboard players get #alice sand_pid",
            "function bproof:expire with storage bproof:args",
        )
        exp = run(
            rcon_port,
            "data get storage sand:__bounded_item p7.k1",
            "data get storage sand:__bounded_item p8.k1.item",
        )
        results["expiry_clears_subject"] = (
            "PASS"
            if "present: 0b" in exp[0] and "emerald" not in exp[0] and not presence(rcon_port, 7)
            else f"FAIL ({exp})"
        )
        results["expiry_isolation_holds"] = "PASS" if "golden_apple" in exp[1] else f"FAIL ({exp[1]})"

    finally:
        print("\n== Shutting down ==")
        try:
            rcon(rcon_port, "stop")
        except Exception as exc:  # noqa: BLE001
            print(f"(stop via rcon failed: {exc})")
        try:
            process.wait(timeout=45)
            results["clean_shutdown"] = "PASS" if process.returncode == 0 else f"FAIL (exit {process.returncode})"
        except subprocess.TimeoutExpired:
            process.terminate()
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
            results["clean_shutdown"] = "FAIL (timeout)"
        shutil.rmtree(server_dir, ignore_errors=True)

    transcript_path = REPO_ROOT / "target" / "bounded_item_proof_transcript.txt"
    transcript_path.parent.mkdir(parents=True, exist_ok=True)
    transcript_path.write_text("\n".join(TRANSCRIPT) + "\n", encoding="utf-8")

    print("\n===== RESULTS =====")
    for name, value in results.items():
        print(f"{name}: {value}")
    print(f"\ntranscript: {transcript_path}")
    return 0 if all(v.startswith("PASS") for v in results.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
