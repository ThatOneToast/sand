#!/usr/bin/env python3
"""Exercise issue #298's unified-State example on Minecraft Java 26.2."""

from __future__ import annotations

import argparse
import json
import os
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
PACK_DIR = REPO_ROOT / "examples" / "unified_state"
RCON_CLIENT = Path(__file__).resolve().parent / "rcon_client.py"
PASSWORD = "sand-unified-state-audit"


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

    dist = PACK_DIR / "dist/unified_state"
    if not dist.is_dir():
        print(f"missing {dist}; pass --build", file=sys.stderr)
        return 2

    export = subprocess.run(
        ["cargo", "run", "--quiet", "--bin", "sand_export"],
        cwd=PACK_DIR,
        env={**os.environ, "SAND_EXPORT_MC_VERSION": "26.2"},
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
    query = next(
        content for path, content in functions.items() if path.startswith("sand/entity_query/")
    )
    attach = functions["attach_zombie_components"]
    load = functions["__sand_lifecycle_load"]
    system = next(
        content for path, content in functions.items() if path.startswith("__sand_system/s")
    )

    armor = re.search(r"scoreboard players add @s (\w+) 1", query).group(1)
    dead_presence = re.search(
        r"execute unless score @s (\w+) matches 1 run scoreboard players add", query
    ).group(1)
    attack_presence = re.search(
        r"execute if score @s (\w+) matches 1 run say migrated", attach
    ).group(1)
    required = re.search(r"scores=\{([^}]+)\}", system).group(1).split(",")
    required_objectives = [entry.split("=", 1)[0] for entry in required]
    defense_presence = next(item for item in required_objectives if item != attack_presence)
    attack_damage = re.search(
        r"scoreboard players set @s (\w+) 2", attach
    ).group(1)
    global_match = re.search(
        r"unless score (#[^ ]+) (\w+) matches .* players set \1 \2 1", load
    )
    global_holder, global_wave = global_match.groups()

    server_dir = Path(tempfile.mkdtemp(prefix="sand-unified-state-audit-"))
    datapacks = server_dir / "world/datapacks"
    datapacks.mkdir(parents=True)
    shutil.copytree(dist, datapacks / "unified_state")
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

    def score(objective: str) -> tuple[int | None, str]:
        output = command(f"scoreboard players get @e[tag=state_audit,limit=1] {objective}")
        match = re.search(r"has (-?\d+) \[", output)
        return (int(match.group(1)) if match else None, output)

    try:
        deadline = time.monotonic() + args.timeout
        startup: list[str] = []
        while time.monotonic() < deadline:
            try:
                line = logs.get(timeout=0.5)
            except queue.Empty:
                continue
            startup.append(line)
            if "Done (" in line and "For help" in line:
                break
        ready = any("Done (" in line and "For help" in line for line in startup)
        check("server_startup", ready, "".join(startup[-5:]))
        if not ready:
            return 1

        loaded = command("datapack list")
        check("datapack_loaded", "file/unified_state" in loaded, loaded)
        global_initial = command(
            f"scoreboard players get {global_holder} {global_wave}",
            "data get storage rpg:state components.world.settings",
        )
        check(
            "global_score_and_typed_data_initialized",
            "has 1 [" in global_initial and "difficulty" in global_initial,
            global_initial,
        )
        command(
            f"scoreboard players set {global_holder} {global_wave} 7",
            "data modify storage rpg:state components.world.settings.difficulty set value 4",
            "reload",
        )
        time.sleep(0.75)
        global_reload = command(
            f"scoreboard players get {global_holder} {global_wave}",
            "data get storage rpg:state components.world.settings.difficulty",
        )
        check(
            "global_reload_preserves_progress",
            "has 7 [" in global_reload and "contents: 4" in global_reload,
            global_reload,
        )

        command(
            "forceload add 0 0",
            "scoreboard objectives add audit_external dummy",
            'summon minecraft:zombie 0 200 0 {Tags:["state_audit"],NoAI:1b,NoGravity:1b,Invulnerable:1b,PersistenceRequired:1b}',
        )
        time.sleep(1.5)
        command(
            "scoreboard players set @e[tag=state_audit,limit=1] audit_external 41",
            "execute as @e[tag=state_audit] at @s run function unified_state:attach_zombie_components",
        )
        initial_armor, initial_out = score(armor)
        check("nested_bundle_attached", initial_armor == 0, initial_out)
        time.sleep(1.25)
        ticked_armor, ticked_out = score(armor)
        check("typed_query_system_ticks_owner", ticked_armor in {1, 2}, ticked_out)

        command(
            f"scoreboard players set @e[tag=state_audit] {attack_damage} 9",
            "execute as @e[tag=state_audit] run function unified_state:reattach_attack",
        )
        damage, damage_out = score(attack_damage)
        check("repeated_attach_preserves_progress", damage == 9, damage_out)

        command(f"scoreboard players set @e[tag=state_audit] {dead_presence} 1")
        blocked_before, _ = score(armor)
        time.sleep(1.25)
        blocked_after, blocked_out = score(armor)
        check(
            "forbidden_presence_filters_system",
            blocked_before is not None and blocked_after == blocked_before,
            blocked_out,
        )
        command(f"scoreboard players set @e[tag=state_audit] {dead_presence} 0")

        command(
            f"scoreboard players set @e[tag=state_audit] {attack_presence} 1",
            "execute as @e[tag=state_audit] run function unified_state:reattach_attack",
        )
        migrated, migrated_out = score(attack_presence)
        check("component_migration_reaches_v2", migrated == 2, migrated_out)

        command("execute as @e[tag=state_audit] run function unified_state:detach_attack")
        attack_after, attack_after_out = score(attack_presence)
        defense_after, defense_after_out = score(defense_presence)
        external_after, external_after_out = score("audit_external")
        check(
            "independent_detach_preserves_other_component_and_external_data",
            attack_after is None and defense_after == 1 and external_after == 41,
            attack_after_out + defense_after_out + external_after_out,
        )
        command("kill @e[tag=state_audit]")
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
            "objectives": {
                "armor": armor,
                "dead_presence": dead_presence,
                "attack_presence": attack_presence,
                "defense_presence": defense_presence,
                "attack_damage": attack_damage,
                "global_wave": global_wave,
            },
            "global_holder": global_holder,
            "checks": checks,
            "server_log": server_log.read_text(encoding="utf-8")
            if server_log.exists()
            else "",
            "multiple_online_players": False,
        }
        if args.evidence:
            args.evidence.parent.mkdir(parents=True, exist_ok=True)
            args.evidence.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
            print(f"evidence: {args.evidence}")

    return 0 if checks and all(item["passed"] for item in checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
