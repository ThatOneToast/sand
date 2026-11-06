#!/usr/bin/env python3
"""Real two-player ownership, death, disconnect and reconnect proof on 26.2 and 26.3.

Uses the repository's version-specific verified passive client; marker actors
do not prove player connection behavior.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from player_validation_harness import PlayerHarness
from mc_validation.protocols import profile_for_version
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, ValidationFailure


class OwnedPlayers(GameplayHarness, PlayerHarness):
    pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version", choices=("26.2", "26.3"), default="26.3")
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    pack.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, SAND_GAMEPLAY_PROOF_PACK=str(pack), SAND_GAMEPLAY_PROOF_VERSION=args.version)
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_lifetime"], cwd=ROOT, env=env, check=True)
    (pack / "data/owned/function/barrier.mcfunction").write_text("say __TICK_BARRIER__\n")
    settings = argparse.Namespace(version=args.version, pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    metadata = json.loads((pack / "fixture.json").read_text())
    h = OwnedPlayers(settings)
    h.prepare()
    h.start()
    tag = "sand.task." + b"owned:player_task".hex()
    try:
        h.wait_for(STARTUP, "startup")
        with h.clients(players=("SandAlpha",)):
            with h.clients(players=("SandBeta",)):
                h.send("gamemode creative @a")
                h.send("forceload add 0 0")
                h.send("tp SandAlpha 2 200 2")
                h.send("tp SandBeta 6 200 2")
                h.send("tag SandAlpha add owner_a")
                h.send("tag SandBeta add owner_b")
                h.send("tick freeze")
                # Player teleport and forceload request chunk loading asynchronously.
                # Task markers must be selectable before testing immediate cancel.
                h.wait_loaded("2 200 2")
                h.advance(1)
                for actor in ("owner_a", "owner_b"):
                    h.invoke("combat_attach", actor)
                h.invoke("player_combat", "owner_a")
                h.check(f"if score SandAlpha {metadata['player_combat_health']} matches 17", "player_scope_uses_its_own_lifecycle")
                h.check(f"if score SandBeta {metadata['player_combat_health']} matches 20", "player_scope_health_isolation")
                h.invoke("combat_hit", "owner_a")
                h.invoke("combat_critical", "owner_b")
                h.check(f"if score SandAlpha {metadata['combat_health']} matches 19", "real_player_shield_resolution")
                h.check(f"if score SandBeta {metadata['combat_health']} matches 13", "real_player_critical_isolation")
                h.invoke("combat_hit", "owner_a")
                h.check(f"if score SandAlpha {metadata['combat_hits']} matches 1", "real_player_invulnerability")
                h.invoke("combat_heal", "owner_b")
                h.check(f"if score SandBeta {metadata['combat_health']} matches 20", "real_player_custom_healing")
                h.send("gamemode survival @a")
                h.invoke("player_native_attributed", "owner_a")
                h.check('if entity @a[name=SandBeta,nbt={Health:17.5f}]', "real_player_runtime_native_damage")
                h.check('if entity @a[name=SandAlpha,nbt={Health:20.0f}]', "native_attacker_health_unchanged")
                h.send("gamemode creative @a")
                for actor in ("owner_a", "owner_b"):
                    h.invoke("attach", actor)
                    h.invoke("player_start", actor)
                h.expect_count("PLAYER_TASK_START", 2)
                h.invoke("player_start", "owner_a")
                h.expect_count("PLAYER_TASK_START", 2)
                h.invoke("player_cancel", "owner_a")
                h.expect_count("PLAYER_TASK_STOP", 1)
                h.advance(1)
                h.check(f"if entity @e[tag={tag}]", "other_player_still_active")
                h.invoke("player_start", "owner_a")
                h.send("kill SandAlpha")
                h.advance(1)
                h.expect_count("PLAYER_TASK_STOP", 2)
                # Beta's remaining task is moved to a distant forced chunk, then
                # genuinely unloaded before cancellation. Player lookup stays global.
                h.send("forceload add 1024 0")
                h.wait_loaded("1026 200 2")
                h.send(f"tp @e[tag={tag}] 1026 200 2")
                h.send("forceload remove 1024 0")
                for _ in range(40):
                    h.advance(20)
                    if h.matches("unless loaded 1026 200 2", "chunk_unloaded"):
                        break
                else:
                    raise ValidationFailure("unload", "remote task chunk stayed loaded")
                h.invoke("player_cancel", "owner_b")
                h.send("forceload add 1024 0")
                h.wait_loaded("1026 200 2")
                h.advance(1)
                h.check(f"unless entity @e[tag={tag}]", "unloaded_cancellation_applied_on_return")
                h.invoke("player_start", "owner_b")
            # Beta disconnected. Cleanup must not depend on owner callbacks.
            h.advance(1)
            h.check(f"unless entity @e[tag={tag}]", "disconnect_removes_work")
            with h.clients(players=("SandBeta",)):
                h.send("gamemode creative SandBeta")
                h.send("tp SandBeta 6 200 2")
                h.send("tag SandBeta add owner_b")
                h.invoke("attach", "owner_b")
                h.invoke("player_start", "owner_b")
                h.expect_count("PLAYER_TASK_START", 5)
                h.invoke("player_cancel", "owner_b")
                h.check(f"unless entity @e[tag={tag}]", "reconnected_owner_can_cancel")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("runtime", "server reported errors", h.errors(0, len(h.lines)))
        (root / "evidence.json").write_text(json.dumps({"minecraft":args.version, "protocol":profile_for_version(args.version)["version"]["protocol_version"], "passed":True, "checks":["two real players", "capacity rejection", "owner-isolated cancellation", "player death", "unloaded cancellation", "disconnect cleanup without callback", "reconnect and cancellation", "custom combat shield, critical, immunity and healing isolation", "runtime native damage between real players"], "log":str(h.log_path)}, indent=2)+"\n")
    finally:
        h.stop()


if __name__ == "__main__":
    main()
