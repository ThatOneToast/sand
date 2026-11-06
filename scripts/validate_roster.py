#!/usr/bin/env python3
"""Prove bounded arena membership and role isolation with real 26.3 players."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from player_validation_harness import PlayerHarness
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, ValidationFailure


class RosterHarness(GameplayHarness, PlayerHarness):
    def invoke(self, name, player=None):
        command = f"function roster_proof:{name}"
        self.send(f"execute as {player} at @s run {command}" if player else command)
        self.barrier(name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_roster"], cwd=ROOT, env=dict(os.environ, SAND_ROSTER_PROOF_PACK=str(pack)), check=True)
    barrier = pack / "data/owned/function/barrier.mcfunction"
    barrier.parent.mkdir(parents=True, exist_ok=True)
    barrier.write_text("say __TICK_BARRIER__\n")
    fields = json.loads((pack / "fixture.json").read_text())
    system = pack / "data/roster_proof/function/__sand_system/lifetime/a"
    load = (system / "roster_load.mcfunction").read_text()
    sequence = re.search(r"unless score #next (\S+)", load)[1]
    h = RosterHarness(argparse.Namespace(version="26.3", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None))
    h.prepare()
    h.start()
    checks = []

    def check(condition, label):
        h.check(condition, label)
        checks.append(label)

    def visits(player, value, label):
        check(f"if score {player} {fields['visits']} matches {value}", label)

    def member(player, arena, expected=True):
        condition = "if" if expected else "unless"
        namespace = "roster_proof" if arena == "a" else "foreign"
        h.send(f"execute as {player} run execute {condition} function {namespace}:__sand_system/lifetime/{arena}/roster_contains run say ROSTER_MEMBER_{len(checks)}")
        h.barrier("membership")
        h.expect_count(f"ROSTER_MEMBER_{len(checks)}", 1)
        checks.append(f"{player}_{arena}_{expected}_{len(checks)}")

    try:
        h.wait_for(STARTUP, "startup")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("startup", "pack load errors", h.errors(0, len(h.lines)))
        h.send("scoreboard objectives add proof dummy")
        with h.clients(players=("SandAlpha",)):
            with h.clients(players=("SandBeta",)):
                h.send("tick freeze")
                h.advance(2)
                h.invoke("join_a", "SandAlpha")
                member("SandAlpha", "a", False)
                h.invoke("open_a")
                h.invoke("open_b")
                h.invoke("join_a", "SandAlpha")
                h.invoke("join_b", "SandBeta")
                member("SandAlpha", "a")
                member("SandBeta", "b")
                h.invoke("join_a", "SandBeta")
                member("SandBeta", "b")
                member("SandBeta", "a", False)
                h.invoke("join_a")  # Server context cannot impersonate a player.
                h.invoke("visit_a")
                visits("SandAlpha", 1, "visit_only_matching_arena")
                visits("SandBeta", 0, "other_arena_unchanged")
                h.invoke("spectate_a", "SandAlpha")
                check(f"if score SandAlpha {fields['role']} matches 0", "full_roster_allows_existing_member_role_change")
                h.invoke("fighters_a")
                visits("SandAlpha", 1, "spectators_excluded_from_fighter_iteration")
                h.invoke("join_a", "SandAlpha")
                h.invoke("reenter_a")
                visits("SandAlpha", 2, "iteration_callback_runs_once")
                member("SandAlpha", "a")
                h.expect_count("NESTED_ROSTER_BAD", 0)
                h.invoke("leave_b", "SandAlpha")
                member("SandAlpha", "a")
                h.invoke("leave_a", "SandAlpha")
                member("SandAlpha", "a", False)
                h.invoke("join_a", "SandBeta")
                member("SandBeta", "a")
                member("SandBeta", "b", False)
                h.invoke("open_a")
                member("SandBeta", "a", False)
                h.invoke("join_a", "SandBeta")
                h.invoke("close_a")
                member("SandBeta", "a", False)
                h.invoke("open_a")
                h.invoke("join_a", "SandBeta")
            # Slot is immediately reusable after disconnect. Reconnecting Beta
            # must not revive the old generation's membership or exceed capacity.
            h.invoke("join_a", "SandAlpha")
            member("SandAlpha", "a")
            with h.clients(players=("SandBeta",)):
                member("SandBeta", "a", False)
                h.invoke("join_a", "SandBeta")
                member("SandBeta", "a", False)
                member("SandAlpha", "a")
            offset = len(h.lines)
            h.send("reload")
            h.wait_for("Reloading!", "reload", offset)
            h.barrier("reloaded")
            h.advance(1)  # Vanilla dispatches the reloaded load tag on a tick.
            member("SandAlpha", "a", False)
            h.invoke("open_a")
            h.invoke("join_a", "SandAlpha")
            member("SandAlpha", "a")
            h.send(f"scoreboard players set #next {sequence} 2147483647")
            h.invoke("open_a")
            member("SandAlpha", "a", False)
            h.invoke("join_a", "SandAlpha")
            member("SandAlpha", "a", False)
        errors = h.errors(0, len(h.lines))
        if errors:
            raise ValidationFailure("runtime", "server errors", errors)
        (root / "evidence.json").write_text(json.dumps({"passed": True, "version": "26.3", "players": 2, "checks": checks}, indent=2) + "\n")
    finally:
        h.stop()


if __name__ == "__main__":
    main()
