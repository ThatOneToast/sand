#!/usr/bin/env python3
"""Exercise issue #298's unified-State example on Minecraft Java 26.2."""

from __future__ import annotations

import argparse
import json
import queue
import re
import secrets
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

from local_rcon_server import write_properties
from rcon_client import run_commands

REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_DIR = REPO_ROOT / "examples" / "unified_state"
JOIN_CLIENT = Path(__file__).resolve().parent / "minimal_join_client.py"


def available_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def wait_for_startup(process, logs: queue.Queue[str], timeout: float) -> tuple[bool, list[str]]:
    """Wait for world startup and the RCON listener before issuing any commands."""
    deadline = time.monotonic() + timeout
    startup: list[str] = []
    world_ready = rcon_ready = False
    while time.monotonic() < deadline and process.poll() is None:
        try:
            line = logs.get(timeout=0.5)
        except queue.Empty:
            continue
        startup.append(line)
        world_ready |= "Done (" in line and "For help" in line
        rcon_ready |= "RCON running on " in line
        if world_ready and rcon_ready:
            return True, startup
    return False, startup


def reachable(functions: dict[str, str], path: str, seen: set[str] | None = None) -> str:
    """Follow the actual owned helpers emitted for a query's whole actions."""
    seen = set() if seen is None else seen
    if path in seen:
        return ""
    seen.add(path)
    body = functions[path]
    children = re.findall(r"(?:^| run )function unified_state:([a-z0-9_./-]+)", body, re.M)
    return "\n".join([body, *(reachable(functions, child, seen) for child in children)])


def inspect_system_scans(functions: dict[str, str]) -> dict[str, str]:
    """Identify the Combatants and Health callbacks independently of other scans."""
    regen_queries = [
        path for path, content in functions.items()
        if path.startswith("sand/entity_query/")
        and re.search(r"scoreboard players remove @s \w+ 1", content)
    ]
    if len(regen_queries) != 1:
        raise ValueError(f"expected one Combatants regeneration query, found {len(regen_queries)}")
    query_path = regen_queries[0]
    attach = functions["attach_zombie_components"]
    attack_damage = re.search(r"scoreboard players set @s (\w+) 2", attach).group(1)
    health_defaults = re.findall(r"scoreboard players set @s (\w+) 20(?:\n|$)", attach)
    if len(health_defaults) != 1:
        raise ValueError("expected one Health current-value default")
    health = health_defaults[0]
    health_queries = [
        path for path in functions if path.startswith("sand/entity_query/")
        and f"scoreboard players add @s {health} 1" in reachable(functions, path).splitlines()
    ]
    if len(health_queries) != 1:
        raise ValueError(f"expected one Health regeneration query, found {len(health_queries)}")
    health_query = health_queries[0]

    def scan_for(callback: str, label: str) -> tuple[str, str, str]:
        scans = []
        for line in functions["__sand_system_tick"].splitlines():
            call = re.search(r"run function unified_state:(__sand_system/[^\s]+)", line)
            if call and f"function unified_state:{callback}" in functions[call.group(1)].splitlines():
                selector = re.search(r"execute as (@e\[[^]]+\]) at @s", line)
                if not selector:
                    raise ValueError(f"{label} callback is missing its entity selector")
                scans.append((line, selector[1], functions[call.group(1)]))
        if len(scans) != 1:
            raise ValueError(f"expected one {label} scan, found {len(scans)}")
        return scans[0]

    combat_scan, combat_selector, shared_system = scan_for(query_path, "Combatants")
    health_scan, health_selector, _ = scan_for(health_query, "Health")
    if health_selector == combat_selector:
        raise ValueError("Health and Combatants must use distinct selectors")
    callbacks = re.findall(r"^function unified_state:(sand/entity_query/[^\s]+)$", shared_system, re.M)
    if (len(callbacks) != 2 or callbacks[0] != query_path
            or f"scoreboard players add @s {attack_damage} 1" not in reachable(functions, callbacks[1]).splitlines()):
        raise ValueError("Combatants callbacks must retain regeneration then attack-training order")
    if functions["__sand_system_tick"].count(f"execute as {combat_selector} at @s") != 1:
        raise ValueError("Combatants callbacks must share exactly one selector scan")
    return {
        "query_path": query_path,
        "combat_scan": combat_scan,
        "combat_selector": combat_selector,
        "shared_system": shared_system,
        "health_query": health_query,
        "health_scan": health_scan,
        "health_selector": health_selector,
        "health": health,
        "attack_damage": attack_damage,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--build", action="store_true")
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--evidence", type=Path)
    args = parser.parse_args()
    result = {
        "minecraft": "26.2", "java": args.java, "server_dir": None,
        "objectives": {}, "checks": {}, "server_log": "",
        "multiple_online_players": False,
    }
    try:
        return run_audit(args, result)
    except Exception as error:
        result["checks"]["audit_completed"] = {"passed": False, "evidence": str(error)}
        raise
    finally:
        if args.evidence:
            args.evidence.parent.mkdir(parents=True, exist_ok=True)
            args.evidence.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
            print(f"evidence: {args.evidence}")


def run_audit(args: argparse.Namespace, result: dict) -> int:
    """Build, inspect and exercise a pack while retaining failure evidence."""
    rcon_token = secrets.token_urlsafe(24)

    if args.build:
        subprocess.run(
            ["cargo", "run", "--manifest-path", str(REPO_ROOT / "sand-cli/Cargo.toml"),
             "--bin", "sand", "--", "build"],
            cwd=PACK_DIR, check=True,
        )

    dist = PACK_DIR / "dist/unified_state"
    if not dist.is_dir():
        raise ValueError(f"missing {dist}; pass --build")

    # Inspect the exact datapack installed below, including CLI transformations.
    function_root = dist / "data/unified_state/function"
    functions = {
        path.relative_to(function_root).with_suffix("").as_posix(): path.read_text(encoding="utf-8")
        for path in function_root.rglob("*.mcfunction")
    }

    scans = inspect_system_scans(functions)
    query_path = scans["query_path"]
    query = functions[query_path]
    attach = functions["attach_zombie_components"]
    load = functions["__sand_lifecycle_load"]
    player_init = functions["__sand_lifecycle_init"]
    combat_scan = scans["combat_scan"]

    armor = re.search(r"scoreboard players add @s (\w+) 1", reachable(functions, query_path)).group(1)
    dead_presence = re.search(
        r"execute unless score @s (\w+) matches 1 run ", query
    ).group(1)
    attack_presence = re.search(
        r"execute if score @s (\w+) matches 1 run say migrated", attach
    ).group(1)
    required = re.search(r"scores=\{([^}]+)\}", combat_scan).group(1).split(",")
    required_objectives = [entry.split("=", 1)[0] for entry in required]
    defense_presence = next(item for item in required_objectives if item != attack_presence)
    attack_damage = scans["attack_damage"]
    global_match = re.search(
        r"unless score (#[^ ]+) (\w+) matches .* players set \1 \2 1", load
    )
    global_holder, global_wave = global_match.groups()
    player_presence = re.findall(
        r"scoreboard players set @s (\w+) 1", player_init
    )[-1]

    server_dir = Path(tempfile.mkdtemp(prefix="sand-unified-state-audit-"))
    result["server_dir"] = str(server_dir)
    datapacks = server_dir / "world/datapacks"
    datapacks.mkdir(parents=True)
    shutil.copytree(dist, datapacks / "unified_state")
    (server_dir / "eula.txt").write_text("eula=true\n", encoding="utf-8")
    server_port, rcon_port = available_port(), available_port()
    write_properties(server_dir, server_port, rcon_port, rcon_token)
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
    checks = result["checks"]

    def command(*commands: str) -> str:
        responses = run_commands("127.0.0.1", rcon_port, rcon_token, list(commands), timeout=30)
        return "\n".join(f">>> {value}\n{response}" for value, response in zip(commands, responses))

    def wait_ticks(ticks: int) -> None:
        """Require server progress even when CI runs below its target tick rate."""
        def gametime() -> int:
            output = command("time query gametime")
            match = re.search(r"The game time is (\d+) tick\(s\)", output)
            if not match:
                raise ValueError(f"missing game-time response: {output}")
            return int(match.group(1))

        start = gametime()
        deadline = time.monotonic() + args.timeout
        while time.monotonic() < deadline:
            if gametime() - start >= ticks:
                return
            time.sleep(0.1)
        raise TimeoutError(f"server did not advance {ticks} game ticks")

    def check(name: str, passed: bool, evidence: str) -> None:
        checks[name] = {"passed": passed, "evidence": evidence.strip()}
        print(f"{'PASS' if passed else 'FAIL'} {name}: {evidence.strip()}")

    def score(objective: str) -> tuple[int | None, str]:
        output = command(f"scoreboard players get @e[tag=state_audit,limit=1] {objective}")
        match = re.search(r"has (-?\d+) \[", output)
        return (int(match.group(1)) if match else None, output)

    try:
        ready, startup = wait_for_startup(process, logs, args.timeout)
        check("server_startup", ready, "".join(startup[-5:]))
        if not ready:
            return 1

        loaded = command("datapack list")
        check("datapack_loaded", "file/unified_state" in loaded, loaded)
        check(
            "compatible_systems_share_one_ordered_scan",
            scans["health_selector"] != scans["combat_selector"],
            combat_scan + "\n" + scans["shared_system"] + "\n" + scans["health_scan"],
        )
        clients = []
        for name in ("StateAuditOne", "StateAuditTwo"):
            clients.append(
                subprocess.Popen(
                    [
                        sys.executable,
                        str(JOIN_CLIENT),
                        "127.0.0.1",
                        str(server_port),
                        "776",
                        name,
                        "8",
                    ],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                )
            )
        client_logs = []
        for client in clients:
            try:
                output, _ = client.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                client.kill()
                output, _ = client.communicate()
            client_logs.append(output)
        player_server_logs = []
        while True:
            try:
                player_server_logs.append(logs.get_nowait())
            except queue.Empty:
                break
        joined = [
            index
            for index, line in enumerate(player_server_logs)
            if "StateAudit" in line and " joined the game" in line
        ]
        left = [
            index
            for index, line in enumerate(player_server_logs)
            if "StateAudit" in line and " left the game" in line
        ]
        simultaneous = len(joined) == 2 and (
            not left or max(joined) < min(left)
        )
        player_scores = command(
            f"scoreboard players get StateAuditOne {player_presence}",
            f"scoreboard players get StateAuditTwo {player_presence}",
        )
        both_initialized = player_scores.count("has 1 [") == 2
        check(
            "two_real_players_join_and_initialize_independently",
            both_initialized,
            player_scores
            + "\nclients entered play: "
            + str(all("joined_play=True" in output for output in client_logs)),
        )
        check(
            "two_real_players_observed_online_together",
            simultaneous,
            "".join(player_server_logs),
        )
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
        wait_ticks(15)
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
        attach_deadline = time.monotonic() + args.timeout
        initial_armor, initial_out = score(armor)
        while initial_armor is None and time.monotonic() < attach_deadline:
            time.sleep(0.25)
            initial_armor, initial_out = score(armor)
        check("archetype_nested_bundle_attached", initial_armor is not None, initial_out)
        command(
            'summon minecraft:zombie 2 200 0 {Tags:["state_audit_b"],NoAI:1b,NoGravity:1b,Invulnerable:1b,PersistenceRequired:1b}'
        )
        wait_ticks(30)
        uuid_output = command(
            "data get entity @e[tag=state_audit,limit=1] UUID",
            "data get entity @e[tag=state_audit_b,limit=1] UUID",
        )
        uuids = re.findall(r"\[I;[^]]+\]", uuid_output)
        if len(uuids) == 2:
            first_path = f'owners[{{uuid:{uuids[0]}}}].components."status".details'
            second_path = f'owners[{{uuid:{uuids[1]}}}].components."status".details'
            command(
                "execute as @e[tag=state_audit,limit=1] run function unified_state:mark_status_data",
                "reload",
            )
            wait_ticks(15)
            keyed_values = command(
                f"data get storage rpg:state {first_path}",
                f"data get storage rpg:state {second_path}",
            )
            keyed_ok = (
                'note: "kept"' in keyed_values
                and "stacks: 3" in keyed_values
                and 'note: "clean"' in keyed_values
                and "stacks: 0" in keyed_values
            )
        else:
            first_path = second_path = ""
            keyed_values = uuid_output
            keyed_ok = False
        check(
            "keyed_typed_data_isolated_and_reload_safe",
            keyed_ok,
            keyed_values,
        )
        command("scoreboard players set @e[tag=state_audit,limit=1] audit_external 41")
        wait_ticks(30)
        ticked_armor, ticked_out = score(armor)
        check(
            "typed_query_system_ticks_owner",
            initial_armor is not None
            and ticked_armor is not None
            and ticked_armor > initial_armor,
            ticked_out,
        )

        command(
            f"scoreboard players set @e[tag=state_audit] {attack_damage} 9",
            "execute as @e[tag=state_audit] run function unified_state:reattach_attack",
        )
        damage, damage_out = score(attack_damage)
        check("repeated_attach_preserves_progress", damage == 9, damage_out)

        command(f"scoreboard players set @e[tag=state_audit] {dead_presence} 1")
        blocked_before, _ = score(armor)
        wait_ticks(30)
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
        if first_path and second_path:
            command(
                "execute as @e[tag=state_audit,limit=1] run function unified_state:detach_status"
            )
            detached_data = command(
                f"data get storage rpg:state {first_path}",
                f"data get storage rpg:state {second_path}",
            )
            check(
                "keyed_detach_removes_only_one_owner_component_path",
                'note: "kept"' not in detached_data
                and 'note: "clean"' in detached_data,
                detached_data,
            )
        command("kill @e[tag=state_audit]", "kill @e[tag=state_audit_b]")
    except Exception as error:
        check("audit_completed", False, str(error))
        raise
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
        result.update({
            "minecraft": "26.2",
            "java": args.java,
            "server_dir": str(server_dir),
            "objectives": {
                "armor": armor,
                "health": scans["health"],
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
            "multiple_online_players": checks.get(
                "two_real_players_observed_online_together", {}
            ).get("passed", False),
        })

    return 0 if checks and all(item["passed"] for item in checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
