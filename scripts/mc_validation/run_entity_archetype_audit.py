#!/usr/bin/env python3
"""Run the #295/#362 entity-archetype audit on a real Minecraft 26.2 server.

The harness uses two unmarked command-summoned Zombies to exercise the same
type-constrained adoption scan used for natural Zombies. It proves adoption
and per-entity behavior, but deliberately does not label the spawn provenance
as a random natural mob spawn.
"""

from __future__ import annotations

import argparse
import json
import queue
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_DIR = REPO_ROOT / "examples" / "rpg_entity"
RCON_CLIENT = Path(__file__).resolve().parent / "rcon_client.py"
PASSWORD = "sand-entity-audit"


def available_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--build", action="store_true")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--evidence", type=Path)
    args = parser.parse_args()

    if args.build:
        subprocess.run(
            ["cargo", "build", "-p", "sand-cli", "--bin", "sand"],
            cwd=REPO_ROOT,
            check=True,
        )
        subprocess.run(
            [str(REPO_ROOT / "target/debug/sand"), "build"],
            cwd=PACK_DIR,
            check=True,
        )

    dist = PACK_DIR / "dist/rpg"
    if not dist.is_dir():
        print(f"missing {dist}; pass --build", file=sys.stderr)
        return 2

    export = subprocess.run(
        ["cargo", "run", "--quiet", "--bin", "sand_export"],
        cwd=PACK_DIR,
        env={**dict(__import__("os").environ), "SAND_EXPORT_MC_VERSION": "26.2"},
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    records = json.loads(export)
    functions = {
        record["path"]: record["content"]
        for record in records
        if record.get("dir") == "function"
    }
    initialize_path, initialize = next(
        (path, content)
        for path, content in functions.items()
        if path.endswith("/initialize")
    )
    root = initialize_path.removesuffix("/initialize")
    marker = re.search(r"tag @s add ([^\s]+)", initialize).group(1)
    provision = functions[f"{root}/provision"]
    field_objectives = re.findall(
        r"execute unless score @s ([^\s]+) matches [^\s]+ run scoreboard players set @s \1 -?\d+",
        provision,
    )
    if len(field_objectives) < 6:
        raise RuntimeError("could not resolve flattened State fields from provision function")
    level = field_objectives[0]
    health, max_health, attack_damage = field_objectives[3:6]
    health_property = functions[f"{root}/property/0"]
    current_dirty = re.search(
        r"execute unless score @s [^\s]+ matches 1 run scoreboard players set @s ([^\s]+) 1",
        health_property,
    ).group(1)
    refresh = functions[f"{root}/refresh"]
    health_clock = re.search(
        r"scoreboard players add @s ([^\s]+) 1\nexecute if score @s \1 matches 20\.\. run",
        refresh,
    ).group(1)
    version_objective = re.search(
        r"scoreboard players set @s ([^\s]+) 2\n"
        + rf"tag @s add {re.escape(marker)}",
        initialize,
    ).group(1)

    server_dir = Path(tempfile.mkdtemp(prefix="sand-entity-audit-"))
    datapacks = server_dir / "world/datapacks"
    datapacks.mkdir(parents=True)
    shutil.copytree(dist, datapacks / "rpg")
    (server_dir / "eula.txt").write_text("eula=true\n", encoding="utf-8")
    server_port, rcon_port = available_port(), available_port()
    (server_dir / "server.properties").write_text(
        "\n".join(
            [
                "level-name=world",
                "online-mode=false",
                "enable-rcon=true",
                f"rcon.password={PASSWORD}",
                f"rcon.port={rcon_port}",
                f"server-port={server_port}",
                "spawn-monsters=false",
                "spawn-animals=false",
                "generate-structures=false",
                "view-distance=4",
                "simulation-distance=4",
                "sync-chunk-writes=false",
                "",
            ]
        ),
        encoding="utf-8",
    )

    process = subprocess.Popen(
        [args.java, "-jar", str(Path(args.jar).resolve()), "nogui"],
        cwd=server_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    logs: queue.Queue[str] = queue.Queue()

    def drain() -> None:
        assert process.stdout
        for line in process.stdout:
            logs.put(line)

    threading.Thread(target=drain, daemon=True).start()
    checks: dict[str, dict[str, str | bool]] = {}

    def command(*commands: str) -> str:
        result = subprocess.run(
            [
                sys.executable,
                str(RCON_CLIENT),
                "127.0.0.1",
                str(rcon_port),
                PASSWORD,
                *commands,
            ],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    def check(name: str, passed: bool, evidence: str) -> None:
        checks[name] = {"passed": passed, "evidence": evidence.strip()}
        print(f"{'PASS' if passed else 'FAIL'} {name}: {evidence.strip()}")

    def response(output: str) -> str:
        return "\n".join(
            line for line in output.splitlines() if not line.startswith(">>> ")
        )

    def score(tag: str, objective: str) -> tuple[int | None, str]:
        output = command(
            f"scoreboard players get @e[type=minecraft:zombie,tag={tag},limit=1] {objective}"
        )
        match = re.search(r"has (-?\d+) \[", output)
        return (int(match.group(1)) if match else None, output)

    def native_health(tag: str) -> tuple[float | None, str]:
        output = command(f"data get entity @e[tag={tag},limit=1] Health")
        match = re.search(r"(-?\d+(?:\.\d+)?)f", output)
        return (float(match.group(1)) if match else None, output)

    try:
        deadline = time.monotonic() + args.timeout
        ready = False
        startup_log: list[str] = []
        while time.monotonic() < deadline:
            try:
                line = logs.get(timeout=0.5)
            except queue.Empty:
                continue
            startup_log.append(line)
            if "Done (" in line and "For help" in line:
                ready = True
                break
        check("server_startup", ready, "".join(startup_log[-5:]))
        if not ready:
            return 1

        loaded = command("datapack list")
        check("datapack_loaded", "file/rpg" in response(loaded), loaded)
        reloaded = command("reload")
        check(
            "reload_without_errors",
            "Reloading" in response(reloaded) or "reloaded" in response(reloaded).lower(),
            reloaded,
        )
        command("forceload add 0 0")
        command(
            'summon minecraft:zombie 0 200 0 {Tags:["audit_a","external_keep"],NoAI:1b,NoGravity:1b,Invulnerable:1b,PersistenceRequired:1b}',
            'summon minecraft:zombie 2 200 0 {Tags:["audit_b"],NoAI:1b,NoGravity:1b,Invulnerable:1b,PersistenceRequired:1b}',
        )
        time.sleep(1.5)

        adopted = command(
            f"execute if entity @e[tag=audit_a,tag={marker}] run data get entity @e[tag=audit_a,limit=1] UUID"
        )
        check(
            "unmarked_zombie_adopted_once",
            "following entity data" in response(adopted),
            adopted,
        )
        a_level, a_level_out = score("audit_a", level)
        b_level, b_level_out = score("audit_b", level)
        check(
            "initial_state_provisioned_per_entity",
            (a_level, b_level) == (1, 1),
            a_level_out + b_level_out,
        )

        initial_max = command(
            "attribute @e[tag=audit_a,limit=1] minecraft:max_health get 1"
        )
        initial_attack = command(
            "attribute @e[tag=audit_a,limit=1] minecraft:attack_damage get 1"
        )
        check(
            "initial_derived_attributes",
            "20" in initial_max and "3" in initial_attack,
            initial_max + initial_attack,
        )
        initial_name = command("data get entity @e[tag=audit_a,limit=1] CustomName")
        check(
            "initial_dynamic_colored_name",
            "Lv. " in initial_name
            and "gold" in initial_name
            and "Plagued Zombie" in initial_name,
            initial_name,
        )

        reconcile = f"execute as @e[tag=audit_a] run function rpg:{root}/reconcile"
        command("tick freeze")

        command(
            "data modify entity @e[tag=audit_a,limit=1] Health set value 17f",
            f"scoreboard players set @e[tag=audit_a,limit=1] {health_clock} 19",
            reconcile,
        )
        observed_health, observed_out = score("audit_a", health)
        check(
            "native_damage_observed_into_state",
            observed_health == 17,
            observed_out,
        )

        command(
            f"scoreboard players add @e[tag=audit_a,limit=1] {health} 2",
            f"scoreboard players set @e[tag=audit_a,limit=1] {current_dirty} 1",
            reconcile,
        )
        healed_native, healed_out = native_health("audit_a")
        check("state_heal_applied_to_native", healed_native == 19, healed_out)

        command(
            f"scoreboard players set @e[tag=audit_a,limit=1] {health} 10",
            f"scoreboard players set @e[tag=audit_a,limit=1] {current_dirty} 1",
            reconcile,
        )
        damaged_native, damaged_out = native_health("audit_a")
        check("state_damage_applied_to_native", damaged_native == 10, damaged_out)

        command(
            f"scoreboard players set @e[tag=audit_a,limit=1] {health} 999",
            f"scoreboard players set @e[tag=audit_a,limit=1] {current_dirty} 1",
            reconcile,
        )
        clamped_state, clamped_state_out = score("audit_a", health)
        clamped_native, clamped_native_out = native_health("audit_a")
        check(
            "state_heal_clamped_by_max_health",
            clamped_state == 20 and clamped_native == 20,
            clamped_state_out + clamped_native_out,
        )

        command(
            "data modify entity @e[tag=audit_a,limit=1] Health set value 12f",
            f"scoreboard players set @e[tag=audit_a,limit=1] {health} 18",
            f"scoreboard players set @e[tag=audit_a,limit=1] {current_dirty} 1",
            f"scoreboard players set @e[tag=audit_a,limit=1] {health_clock} 19",
            reconcile,
        )
        simultaneous_state, simultaneous_state_out = score("audit_a", health)
        simultaneous_native, simultaneous_native_out = native_health("audit_a")
        check(
            "simultaneous_state_write_wins",
            simultaneous_state == 18 and simultaneous_native == 18,
            simultaneous_state_out + simultaneous_native_out,
        )

        command(
            "data modify entity @e[tag=audit_a,limit=1] Health set value 11f",
            reconcile,
        )
        before_interval, before_interval_out = score("audit_a", health)
        check(
            "interval_20_does_not_observe_early",
            before_interval == 18,
            before_interval_out,
        )
        command(*([reconcile] * 19))
        at_interval, at_interval_out = score("audit_a", health)
        check("interval_20_observes_on_cadence", at_interval == 11, at_interval_out)
        command(reconcile)
        command(
            "data modify entity @e[tag=audit_a,limit=1] Health set value 10f",
            reconcile,
        )
        bounded_state, bounded_state_out = score("audit_a", health)
        check(
            "periodic_observer_does_not_self_trigger_forever",
            bounded_state == 11,
            bounded_state_out,
        )

        command("data modify entity @e[tag=audit_a,limit=1] Health set value 10f")
        command("execute as @e[tag=audit_a] run function rpg:level_up", reconcile)
        a_level2, level2_out = score("audit_a", level)
        b_level2, b_level2_out = score("audit_b", level)
        max2 = command("attribute @e[tag=audit_a,limit=1] minecraft:max_health get 1")
        attack2 = command(
            "attribute @e[tag=audit_a,limit=1] minecraft:attack_damage get 1"
        )
        ratio_health, ratio_health_out = native_health("audit_a")
        check(
            "dirty_refresh_isolated_between_entities",
            (a_level2, b_level2) == (2, 1) and "22" in max2 and "4" in attack2,
            level2_out + b_level2_out + max2 + attack2,
        )
        check(
            "health_preserve_ratio",
            ratio_health == 11,
            ratio_health_out,
        )
        name2 = command("data get entity @e[tag=audit_a,limit=1] CustomName")
        check("dynamic_name_refreshed", "Lv. " in name2 and '"2"' in name2, name2)

        command(
            f"scoreboard players set @e[tag=audit_a,limit=1] {version_objective} 1",
            reconcile,
        )
        version, version_out = score("audit_a", version_objective)
        check("ordered_migration_reaches_v2", version == 2, version_out)

        unrelated = command(
            "execute if entity @e[tag=audit_a,tag=external_keep] run data get entity @e[tag=audit_a,limit=1] UUID"
        )
        check(
            "preserve_mode_keeps_unrelated_tag",
            "following entity data" in response(unrelated),
            unrelated,
        )
        storage = command("data get storage rpg:__sand_entity")
        check(
            "macro_scratch_storage_cleaned",
            "No elements found" in storage or "{}" in storage or not storage.strip(),
            storage,
        )

        command("tick unfreeze", "forceload remove 0 0")
        time.sleep(0.5)
        command("forceload add 0 0")
        time.sleep(1.0)
        observed_after_load, after_load_out = score("audit_a", health)
        check(
            "unload_reload_reobservation",
            observed_after_load is not None,
            after_load_out,
        )

        command("tick freeze")
        cleanup = command(
            f"execute as @e[tag=audit_a] run function rpg:{root}/cleanup"
        )
        marker_after = command(
            f"execute if entity @e[tag=audit_a,tag={marker}] run data get entity @e[tag=audit_a,limit=1] UUID"
        )
        check(
            "explicit_cleanup_removes_sand_state",
            "following entity data" not in response(marker_after),
            cleanup + marker_after,
        )
        command("kill @e[tag=audit_a]", "kill @e[tag=audit_b]")
        time.sleep(0.25)
        removed = command(
            "data get entity @e[tag=audit_a,limit=1] UUID",
            "data get entity @e[tag=audit_b,limit=1] UUID",
        )
        check(
            "clean_removal",
            "following entity data" not in response(removed),
            removed,
        )
    finally:
        try:
            command("stop")
        except Exception:
            process.terminate()
        try:
            process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            process.kill()
        server_log = server_dir / "logs/latest.log"
        result = {
            "minecraft": "26.2",
            "java": args.java,
            "server_dir": str(server_dir),
            "marker": marker,
            "generated_root": root,
            "objectives": {
                "level": level,
                "health": health,
                "health_dirty": current_dirty,
                "health_observation_clock": health_clock,
                "max_health": max_health,
                "attack_damage": attack_damage,
                "archetype_version": version_objective,
            },
            "checks": checks,
            "server_log": server_log.read_text(encoding="utf-8")
            if server_log.exists()
            else "",
            "natural_spawn_provenance": False,
        }
        if args.evidence:
            args.evidence.parent.mkdir(parents=True, exist_ok=True)
            args.evidence.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
            print(f"evidence: {args.evidence}")

    return 0 if checks and all(item["passed"] for item in checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
