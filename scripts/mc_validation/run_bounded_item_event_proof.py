#!/usr/bin/env python3
"""Validate the #272 bounded item-transport **event path** — the functions
Sand actually generates — on a real Minecraft Java 26.2 server.

`run_bounded_item_proof.py` proves the raw storage primitive with a
hand-written pack. This script is the follow-up: it materializes the pack
*Sand exports* for `sand-core/tests/event_chain_bounded_item_transport.rs`
and drives the generated persist/load/expire functions over RCON.

What this proves, precisely:

  * The exported pack loads on a real 26.2 server with no errors, and
    survives `/reload` (parser/load evidence).
  * Running the generated **source dispatch** function as a subject
    allocates that subject a slot and persists the snapshot into
    per-subject command storage (runtime behavior).
  * Running the generated **child dispatch** function as a subject stages
    *that subject's* copy into the scratch path the read accessors name,
    and never another subject's (runtime behavior).
  * Repeated source occurrence replaces atomically; an absent source clears
    presence; the generated expiry function clears exactly one subject.

What this does **not** prove: that a real player swinging a real weapon
triggers the source event. That needs a real game client; this environment
has none (see `scripts/mc_validation/README.md` on the 26.2 Play-phase
client instability). Subjects here are `minecraft:marker` entities acting
as score holders, and the source's own item *capture* (#229/#267, validated
separately) is stood in for by writing its snapshot storage directly — so
what is exercised is the transport under test, not the capture feeding it.
The `execute as @a` coordinator lines are likewise not exercised at
runtime, because a zero-player world has no `@a`; their shape is covered by
the exact-output tests.

Usage:
    python3 scripts/mc_validation/run_bounded_item_event_proof.py \
        --jar ~/.sand/cache/26.2/server.jar

The stdout-drain threading is load-bearing; see `run_audit.py`. Do not
replace it with a bounded read or `break` out of it.
"""

from __future__ import annotations

import argparse
import json
import os
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
RCON_PASSWORD = "sand-bounded-evt"
PACK_FORMAT = 107  # 26.2, per sand-core/src/version.rs
NS = "boundeditempack"

TRANSCRIPT: list[str] = []


def available_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def rcon(port: int, *commands: str) -> list[str]:
    return rcon_client.run_commands("127.0.0.1", port, RCON_PASSWORD, list(commands))


def run(port: int, *commands: str) -> list[str]:
    out = rcon(port, *commands)
    for cmd, response in zip(commands, out):
        TRANSCRIPT.append(f"> {cmd}")
        TRANSCRIPT.append(response)
        print(f"> {cmd}\n{response}")
    return out


def export_records() -> list[dict]:
    """Run the exporter in its own process and return its records.

    Deliberately reuses the exact `#[ignore]`d dump helper the determinism
    check uses, so this harness validates the same bytes the test suite
    asserts on rather than a separately-derived pack.
    """
    dump = Path(tempfile.mkdtemp(prefix="sand-bounded-export-")) / "export.json"
    env = dict(os.environ, SAND_DETERMINISM_DUMP_PATH=str(dump))
    subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "sand-core",
            "--test",
            "event_chain_bounded_item_transport",
            "--",
            "--ignored",
            "--exact",
            "dump_export_to_file_for_external_hash_comparison",
        ],
        cwd=REPO_ROOT,
        env=env,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(dump.read_text())


def write_pack(datapacks: Path, records: list[dict]) -> None:
    pack = datapacks / "bevt"
    (pack / "data").mkdir(parents=True)
    (pack / "pack.mcmeta").write_text(
        json.dumps(
            {
                "pack": {
                    "pack_format": PACK_FORMAT,
                    "min_format": PACK_FORMAT,
                    "max_format": PACK_FORMAT,
                    "description": "sand #272 bounded item event-path proof",
                }
            }
        ),
        encoding="utf-8",
    )
    for record in records:
        target = (
            pack
            / "data"
            / record["namespace"]
            / record["dir"]
            / f"{record['path']}.{record['ext']}"
        )
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(record["content"], encoding="utf-8")


def find(records: list[dict], prefix: str) -> str:
    matches = [r["path"] for r in records if r["path"].startswith(prefix)]
    if len(matches) != 1:
        raise SystemExit(f"expected exactly one function under {prefix!r}, got {matches!r}")
    return matches[0]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()

    print("== Exporting the pack under test ==")
    records = export_records()
    functions = [r for r in records if r["dir"] == "function"]
    load_fn = find(functions, "__sand_event_bounded_item_load/")
    persist_fn = find(functions, "__sand_event_bounded_item_persist/")
    expire_fn = find(functions, "__sand_event_bounded_item_expire/")
    entry_key = load_fn.split("/", 1)[1]
    # The source is the only dispatch function containing a persist call.
    source_dispatch = next(
        r["path"]
        for r in functions
        if r["path"].startswith("__sand_event_dispatch/")
        and "__sand_event_bounded_item_persist/" in r["content"]
    )
    child_dispatches = sorted(
        r["path"]
        for r in functions
        if r["path"].startswith("__sand_event_dispatch/")
        and "__sand_event_bounded_item_load/" in r["content"]
    )
    # The source's own transient snapshot storage, read by the persist body.
    persist_body = next(r["content"] for r in functions if r["path"] == persist_fn)
    snap_key = persist_body.split("snap.")[1].split("{")[0]
    print(f"entry key {entry_key}, source snapshot key {snap_key}")
    print(f"source dispatch {source_dispatch}, children {child_dispatches}")

    server_dir = Path(tempfile.mkdtemp(prefix="sand-bounded-event-proof-"))
    datapacks = server_dir / "world" / "datapacks"
    datapacks.mkdir(parents=True)
    write_pack(datapacks, records)
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

    def load_for(tag: str) -> str:
        """Run the generated child dispatch as `tag`'s subject, then read the
        staged item back."""
        run(rcon_port, f"execute as @e[tag={tag},limit=1] run function {NS}:{child_dispatches[0]}")
        return run(rcon_port, f"data get storage sand:__bounded_item cur.{entry_key}.item")[0]

    def staged_present() -> bool:
        out = run(
            rcon_port,
            f"execute store success score #probe sand_subj if data storage sand:__bounded_item "
            f"cur.{entry_key}{{present:1b}}",
            "scoreboard players get #probe sand_subj",
        )
        return "has 1 " in out[1]

    def persist_for(tag: str) -> None:
        run(rcon_port, f"execute as @e[tag={tag},limit=1] run function {NS}:{source_dispatch}")

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
        results["datapack_loaded"] = "PASS" if "bevt" in "\n".join(out) else "FAIL"

        # The load tag ran the generated setup, so the slot objective exists.
        obj = run(rcon_port, "scoreboard objectives add sand_subj dummy")
        results["slot_objective_created_by_pack"] = (
            "PASS" if "already exists" in obj[0] else f"FAIL (setup did not run: {obj})"
        )

        print("\n== Subjects (armor stands standing in for players) ==")
        # A freshly summoned entity is not reliably selector-matchable in the
        # same RCON batch as its own summon in this 26.2 build — see
        # `run_audit.py`'s combat-pair helper and the README. Summon in its
        # own call, settle, then verify; retry the whole thing rather than
        # reporting a spurious failure.
        run(rcon_port, "forceload add 0 0")
        selectable = False
        for attempt in range(5):
            run(rcon_port, "kill @e[tag=subjA]", "kill @e[tag=subjB]")
            run(
                rcon_port,
                'summon armor_stand 0 200 0 {Tags:["subjA"],NoGravity:1b,PersistenceRequired:1b,Marker:1b}',
            )
            run(
                rcon_port,
                'summon armor_stand 2 200 0 {Tags:["subjB"],NoGravity:1b,PersistenceRequired:1b,Marker:1b}',
            )
            time.sleep(1.0)
            sel = run(
                rcon_port,
                "execute as @e[tag=subjA,limit=1] run data get entity @s Tags",
                "execute as @e[tag=subjB,limit=1] run data get entity @s Tags",
            )
            if "subjA" in sel[0] and "subjB" in sel[1]:
                selectable = True
                break
            print(f"(subjects not selectable on attempt {attempt + 1}, retrying)")
        # Every check below depends on this one; if it fails they all will,
        # and that cascade is a harness/environment failure, not evidence
        # about the feature.
        results["subjects_selectable"] = "PASS" if selectable else "FAIL (never became selectable)"

        print("\n== Source occurrence persists into per-subject storage ==")
        run(
            rcon_port,
            f'data modify storage sand:__participants snap.{snap_key} set value '
            f'{{present:1b,item:{{id:"minecraft:diamond_sword",count:1}}}}',
        )
        persist_for("subjA")
        slots = run(
            rcon_port,
            "execute as @e[tag=subjA,limit=1] run scoreboard players get @s sand_subj",
        )
        results["slot_allocated_on_first_occurrence"] = (
            "PASS" if " has 1 " in slots[0] else f"FAIL ({slots})"
        )

        run(
            rcon_port,
            f'data modify storage sand:__participants snap.{snap_key} set value '
            f'{{present:1b,item:{{id:"minecraft:golden_apple",count:3}}}}',
        )
        persist_for("subjB")
        slots_b = run(
            rcon_port,
            "execute as @e[tag=subjB,limit=1] run scoreboard players get @s sand_subj",
        )
        results["distinct_slots_per_subject"] = (
            "PASS" if " has 2 " in slots_b[0] else f"FAIL ({slots_b})"
        )

        print("\n== Child dispatch stages the *reading* subject's own copy ==")
        a = load_for("subjA")
        results["child_reads_own_subject_snapshot"] = (
            "PASS" if "diamond_sword" in a else f"FAIL ({a})"
        )
        results["staged_presence_flag_set"] = "PASS" if staged_present() else "FAIL"
        b = load_for("subjB")
        results["cross_subject_isolation"] = (
            "PASS" if "golden_apple" in b and "diamond_sword" not in b else f"FAIL ({b})"
        )
        # Re-read A after B, to prove staging is not sticky.
        a2 = load_for("subjA")
        results["staging_is_not_sticky_between_consumers"] = (
            "PASS" if "diamond_sword" in a2 and "golden_apple" not in a2 else f"FAIL ({a2})"
        )

        print("\n== Second consumer in the same cycle sees the same copy ==")
        run(rcon_port, f"execute as @e[tag=subjA,limit=1] run function {NS}:{child_dispatches[1]}")
        second = run(rcon_port, f"data get storage sand:__bounded_item cur.{entry_key}.item")[0]
        results["second_consumer_not_starved"] = (
            "PASS" if "diamond_sword" in second else f"FAIL ({second})"
        )

        print("\n== Repeated source occurrence replaces atomically ==")
        run(
            rcon_port,
            f'data modify storage sand:__participants snap.{snap_key} set value '
            f'{{present:1b,item:{{id:"minecraft:stone",count:64}}}}',
        )
        persist_for("subjA")
        replaced = load_for("subjA")
        results["repeated_occurrence_replaces"] = (
            "PASS" if "stone" in replaced and "diamond_sword" not in replaced else f"FAIL ({replaced})"
        )
        untouched = load_for("subjB")
        results["replacement_does_not_disturb_other_subjects"] = (
            "PASS" if "golden_apple" in untouched else f"FAIL ({untouched})"
        )

        print("\n== Absent item clears presence rather than leaving stale data ==")
        run(
            rcon_port,
            f"data remove storage sand:__participants snap.{snap_key}",
        )
        persist_for("subjA")
        load_for("subjA")
        results["absent_source_clears_presence"] = "PASS" if not staged_present() else "FAIL"

        print("\n== Survives /reload ==")
        run(
            rcon_port,
            f'data modify storage sand:__participants snap.{snap_key} set value '
            f'{{present:1b,item:{{id:"minecraft:diamond_sword",count:1}}}}',
        )
        persist_for("subjA")
        rel = run(rcon_port, "reload")
        time.sleep(3)
        results["reload"] = "PASS" if "reload" in "\n".join(rel).lower() else f"FAIL ({rel})"
        after = load_for("subjA")
        results["survives_reload"] = "PASS" if "diamond_sword" in after else f"FAIL ({after})"
        after_b = load_for("subjB")
        results["isolation_survives_reload"] = (
            "PASS" if "golden_apple" in after_b else f"FAIL ({after_b})"
        )

        print("\n== Generated expiry clears exactly one subject ==")
        run(rcon_port, f"execute as @e[tag=subjA,limit=1] run function {NS}:{expire_fn}")
        load_for("subjA")
        results["expiry_clears_subject"] = "PASS" if not staged_present() else "FAIL"
        surviving = load_for("subjB")
        results["expiry_does_not_touch_other_subjects"] = (
            "PASS" if "golden_apple" in surviving else f"FAIL ({surviving})"
        )

        run(rcon_port, "kill @e[tag=subjA]", "kill @e[tag=subjB]", "forceload remove all")

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

    transcript_path = REPO_ROOT / "target" / "bounded_item_event_proof_transcript.txt"
    transcript_path.parent.mkdir(parents=True, exist_ok=True)
    transcript_path.write_text("\n".join(TRANSCRIPT) + "\n", encoding="utf-8")

    print("\n===== RESULTS =====")
    for name, value in results.items():
        print(f"{name}: {value}")
    print(f"\ntranscript: {transcript_path}")
    return 0 if all(v.startswith("PASS") for v in results.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
