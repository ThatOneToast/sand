#!/usr/bin/env python3
"""Prove ordered progression and two independent entity-host sessions on 26.3."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, RELOAD_START, ValidationFailure


class ProgressionHarness(GameplayHarness):
    def invoke(self, name, actor=None):
        command = f"function progression:{name}"
        self.send(f"execute as @e[tag={actor},limit=1] at @s run {command}" if actor else command)
        self.barrier(name)

    def advance(self, ticks):
        offset = len(self.lines)
        self.send(f"schedule function progression:barrier {ticks}t replace")
        self.send(f"tick step {ticks + 1}t")
        self.wait_for("__TICK_BARRIER__", f"advance {ticks} ticks", offset)
        self.barrier(f"stepped_{ticks}")


def run(h, fields):
    h.wait_for(STARTUP, "startup")
    if h.errors(0, len(h.lines)):
        raise ValidationFailure("startup", "generated pack failed to load", h.errors(0, len(h.lines)))
    h.send("forceload add 0 0")
    h.send("tick freeze")
    h.send("schedule function progression:wait_loaded 1t replace")
    for _ in range(20):
        h.advance(20)
        if any("__CHUNK_READY__" in line for line in h.lines):
            break
    else:
        raise ValidationFailure("preparation", "forced chunk did not become loaded")
    h.send("tick step stop")
    h.barrier("frozen")
    actors = ["host_a", "host_b"]
    for actor, x in zip(actors, [2, 6]):
        h.send(f'summon minecraft:marker {x} 200 2 {{Tags:["{actor}"]}}')
        h.invoke("attach", actor)

    def score(actor, field, value, label):
        h.check(f"if score @e[tag={actor},limit=1] {fields[field]} matches {value}", label)

    def points(actor, value):
        h.send(f"scoreboard players set @e[tag={actor},limit=1] {fields['points']} {value}")
        h.barrier("points")

    def no_tasks(label):
        h.check("unless entity @e[tag=sand.owned]", label)

    h.invoke("branch_advance", "host_a")
    score("host_a", "branching", 0, "branch_requires_objective_eligibility")
    points("host_a", 2)
    points("host_b", 1)
    for actor in actors:
        h.invoke("branch_advance", actor)
    score("host_a", "branching", 2, "first_matching_branch_wins")
    score("host_b", "branching", 1, "unmatched_branch_uses_sequential_successor")
    score("host_a", "branch_rewards", 1, "branch_reward_reentry_cannot_advance_or_reset")
    for _ in range(3):
        for actor in actors:
            h.invoke("branch_advance", actor)
    score("host_a", "branch_rewards", 101, "skipped_objective_never_rewards_and_completion_is_once")
    score("host_b", "branch_rewards", 111, "unbranched_owner_receives_all_rewards")
    h.invoke("branch_reset", "host_a")
    h.invoke("branch_advance", "host_a")
    score("host_a", "branch_rewards", 102, "branch_reset_explicitly_allows_replay")
    points("host_b", 0)

    points("host_a", 2)
    h.invoke("quest_advance", "host_a")
    score("host_a", "quest", 1, "one_objective_per_dispatch")
    score("host_a", "rewards", 1, "reentrant_reset_and_advance_rejected")
    score("host_b", "quest", 0, "quest_owner_isolation")
    h.invoke("quest_advance", "host_a")
    h.invoke("quest_advance", "host_a")
    score("host_a", "quest", 2, "terminal_progress")
    score("host_a", "rewards", 11, "terminal_reward_once_and_return_isolated")
    h.invoke("quest_reset", "host_a")
    score("host_a", "quest", 0, "explicit_reset_without_reward")
    h.invoke("quest_advance", "host_a")
    score("host_a", "rewards", 12, "explicit_reset_permits_replay")

    points("host_a", 1)
    for actor in actors:
        h.invoke("start", actor)
    h.exact(1)
    score("host_a", "phase", 1, "first_session_progress")
    score("host_a", "rewards", 13, "reward_lifecycle_reentry_rejected")
    score("host_b", "phase", 0, "second_session_independent")
    h.exact(1)
    score("host_a", "phase", 1, "ineligible_phase_stays_pending")
    for actor in actors:
        points(actor, 2)
    h.exact(1)
    score("host_a", "phase", 2, "first_session_complete")
    score("host_a", "rewards", 23, "session_terminal_reward_once")
    score("host_a", "stops", 1, "completion_stop_once")
    score("host_b", "phase", 1, "at_most_one_phase_per_loaded_tick")
    h.exact(1)
    score("host_b", "phase", 2, "second_session_complete")
    score("host_b", "rewards", 11, "second_session_reward_isolation")
    no_tasks("completion_cleans_parents_and_descendants")
    for actor in actors:
        h.invoke("start", actor)
    no_tasks("completed_start_rejected")

    # Stable owner identities already exist, so new task score holders must be
    # reclaimed exactly after another start/cancel cycle.
    h.send(f"scoreboard players set #before {fields['points']} 0")
    h.send(f"scoreboard players set #after {fields['points']} 0")
    h.send(f"execute store result score #before {fields['points']} run scoreboard players list")
    for actor in actors:
        points(actor, 0)
        h.invoke("restart", actor)
    h.invoke("cancel", "host_a")
    score("host_a", "phase", 0, "cancel_preserves_progress")
    score("host_a", "stops", 2, "stop_callback_controls_rejected")
    h.check("if entity @e[tag=sand.owned]", "other_session_survives_cancel")
    h.invoke("reset", "host_b")
    no_tasks("reset_cleans_descendants")
    h.send(f"execute store result score #after {fields['points']} run scoreboard players list")
    h.check(f"if score #before {fields['points']} = #after {fields['points']}", "no_orphan_runtime_score_holders")

    h.invoke("start", "host_a")
    h.invoke("detach", "host_a")
    h.exact(1)
    no_tasks("detached_progress_state_ends_session")
    h.invoke("attach", "host_a")
    h.invoke("start", "host_a")
    h.invoke("detach", "host_a")
    h.invoke("reset", "host_a")
    no_tasks("explicit_reset_cleans_even_with_detached_state")
    h.invoke("attach", "host_a")
    h.invoke("reset", "host_a")
    h.invoke("start", "host_a")
    points("host_a", 1)
    h.exact(1)
    score("host_a", "phase", 1, "saved_progress_before_reload")
    offset = len(h.lines)
    h.send("reload")
    h.send("say __PROGRESSION_RELOAD_DONE__")
    h.wait_for(RELOAD_START, "reload-start", offset)
    h.wait_for("__PROGRESSION_RELOAD_DONE__", "reload-done", offset)
    # Load functions run at the next tick and reset the proof counter. Use a
    # scheduled barrier for this boundary before resuming exact-tick checks.
    h.advance(1)
    no_tasks("reload_invalidates_ephemeral_controllers")
    score("host_a", "phase", 1, "reload_preserves_canonical_progress")
    h.invoke("start", "host_a")
    points("host_a", 2)
    h.exact(1)
    score("host_a", "phase", 2, "explicit_resume_after_reload")
    no_tasks("resumed_completion_cleanup")
    if h.errors(0, len(h.lines)):
        raise ValidationFailure("runtime", "Minecraft reported command errors", h.errors(0, len(h.lines)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--reuse-export", action="store_true", help="Reuse the existing output/pack fixture when iterating on native harness assertions.")
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    pack.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, SAND_MC_VERSION="26.3", SAND_STRICT_CODEGEN="1", SAND_PROGRESSION_PROOF_PACK=str(pack))
    if args.reuse_export:
        if not (pack / "fixture.json").is_file():
            parser.error("--reuse-export requires an existing output/pack/fixture.json")
    else:
        subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_progression"], cwd=ROOT, env=env, check=True)
    functions = pack / "data/progression/function"
    (functions / "barrier.mcfunction").write_text("say __TICK_BARRIER__\n")
    (functions / "wait_loaded.mcfunction").write_text("execute if loaded 2 200 2 run return run say __CHUNK_READY__\nschedule function progression:wait_loaded 1t replace\n")
    (functions / "proof_tick.mcfunction").write_text("execute if score #remaining sand_proof matches 1.. run scoreboard players remove #remaining sand_proof 1\nexecute if score #remaining sand_proof matches 0 run say __EXACT_TICKS__\nexecute if score #remaining sand_proof matches 0 run scoreboard players set #remaining sand_proof -1\n")
    (functions / "proof_load.mcfunction").write_text("scoreboard objectives add sand_proof dummy\nscoreboard players set #remaining sand_proof -1\n")
    for phase in ["load", "tick"]:
        path = pack / f"data/minecraft/tags/function/{phase}.json"
        tag = json.loads(path.read_text())
        value = f"progression:proof_{phase}"
        if value not in tag["values"]:
            tag["values"].append(value)
        path.write_text(json.dumps(tag))
    settings = argparse.Namespace(version="26.3", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    harness = ProgressionHarness(settings)
    harness.prepare()
    with (harness.server / "server.properties").open("a") as stream:
        stream.write("pause-when-empty-seconds=0\n")
    harness.start()
    try:
        run(harness, json.loads((pack / "fixture.json").read_text()))
        (root / "evidence.json").write_text(json.dumps({"minecraft": "26.3", "passed": True, "actors": "two independent loaded marker hosts", "checks": ["forward branch priority and skipped rewards", "branch eligibility and owner isolation", "ordered objective commits", "once-per-reset rewards", "reward and stop reentrancy rejection", "two concurrent sessions", "one phase per loaded tick", "owner-safe cancellation", "descendant and score-holder cleanup", "State detachment cleanup", "reload task invalidation with canonical progress preservation", "explicit resume after reload"], "log": str(harness.log_path)}, indent=2) + "\n")
    finally:
        harness.stop()


if __name__ == "__main__":
    main()
