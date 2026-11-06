#!/usr/bin/env python3
"""Exercise two integrated arena rounds with four passive players on 26.3.

Calls the same typed actions as the menu/sneak handlers through test-only entry
points. This proves server gameplay, not graphical dialog input or rendering.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from player_validation_harness import PlayerHarness
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, ValidationFailure


class ArenaHarness(GameplayHarness, PlayerHarness):
    players = ("SandAlpha", "SandBeta", "SandGamma", "SandDelta")

    def invoke(self, name, player=None):
        command = f"function cooperative_arena:{name}"
        self.send(f"execute as {player} at @s run {command}" if player else command)
        self.barrier(name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"), SAND_ARENA_PROOF_PACK=str(pack))
    subprocess.run(["cargo", "test", "--manifest-path", str(ROOT / "examples/cooperative_arena/Cargo.toml")], cwd=ROOT, env=env, check=True)
    barrier = pack / "data/owned/function/barrier.mcfunction"
    barrier.parent.mkdir(parents=True, exist_ok=True)
    barrier.write_text("say __TICK_BARRIER__\n")
    fields = json.loads((pack / "fixture.json").read_text())
    h = ArenaHarness(argparse.Namespace(version="26.3", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None))
    h.prepare()
    h.start()
    checks = []

    def check(condition, label):
        h.check(condition, label)
        checks.append(label)

    def score(player, field, value, label):
        check(f"if score {player} {fields[field]} matches {value}", label)

    try:
        h.wait_for(STARTUP, "startup")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("startup", "pack load errors", h.errors(0, len(h.lines)))
        h.send("scoreboard objectives add sand_proof dummy")
        h.send("tick freeze")
        h.send("forceload add -16 -16 48 16")
        h.send("schedule function owned:barrier 40t replace")
        h.advance(40)
        with h.clients(players=("SandAlpha", "SandBeta", "SandGamma", "SandDelta")):
            h.send("gamemode creative @a")
            h.send("tp @a 0 102 4")
            h.advance(5)
            h.invoke("setup")
            h.advance(2)
            for player, arena in [("SandAlpha", "a"), ("SandBeta", "a"), ("SandGamma", "b"), ("SandDelta", "b")]:
                h.invoke(f"proof_join_{arena}", player)
            check(f"if score SandAlpha {fields['member']} = SandBeta {fields['member']} unless score SandAlpha {fields['member']} = SandGamma {fields['member']}", "two_distinct_arena_rosters")
            h.send("gamemode survival @a")
            h.invoke("proof_start_a", "SandAlpha")
            h.invoke("proof_start_b", "SandGamma")
            h.advance(2)
            # A nonmember cannot start/restart the other arena.
            h.invoke("proof_start_b", "SandAlpha")
            for _ in range(10):
                h.invoke("proof_strike", "SandAlpha")
                h.invoke("proof_strike", "SandBeta")
                h.advance(10)
            h.advance(25)
            score("SandAlpha", "wins", 1, "arena_a_fighter_reward")
            score("SandBeta", "wins", 1, "arena_a_cooperative_reward")
            score("SandGamma", "wins", 0, "arena_b_no_cross_reward")
            score("SandDelta", "wins", 0, "arena_b_second_player_no_cross_reward")
            check(f"if score @e[tag=cooperative_arena.host_b,limit=1] {fields['boss_health']} matches 100", "arena_a_strikes_do_not_damage_b")
            # Completed A can be opened again while B remains active.
            h.invoke("proof_reopen_a")
            h.invoke("proof_join_a", "SandAlpha")
            h.invoke("proof_start_a", "SandAlpha")
            h.advance(2)
            score("SandAlpha", "wins", 1, "new_round_does_not_replay_reward")
            check(f"if score @e[tag=cooperative_arena.host_a,limit=1] {fields['boss_health']} matches 100", "new_round_restores_boss_health")
            h.send(f"scoreboard players set SandGamma {fields['health']} 0")
            h.advance(2)
            score("SandGamma", "role", 0, "defeat_transitions_to_spectator_outside_iteration")
            h.send(f"scoreboard players set SandDelta {fields['health']} 0")
            h.advance(25)
            score("SandDelta", "wins", 0, "all_defeated_round_has_no_rewards")
            # Explicit repeated reset cancels loaded descendants and presentations.
            h.invoke("reset")
            h.invoke("reset")
            h.advance(3)
            check("unless entity @e[tag=sand.owned]", "reset_removes_owned_runtime_entities")
            score("SandAlpha", "wins", 1, "reset_preserves_saved_wins")
            offset = len(h.lines)
            h.send("reload")
            h.wait_for("Reloading!", "reload", offset)
            h.advance(2)
            score("SandAlpha", "wins", 1, "reload_preserves_saved_wins")
            check("unless entity @e[tag=sand.owned]", "reload_does_not_resume_tasks")
        errors = h.errors(0, len(h.lines))
        if errors:
            raise ValidationFailure("runtime", "server errors", errors)
        (root / "evidence.json").write_text(json.dumps({"passed": True, "version": "26.3", "players": 4, "input": "test-only calls to shared typed gameplay actions; passive clients", "checks": checks}, indent=2) + "\n")
    finally:
        h.stop()


if __name__ == "__main__":
    main()
