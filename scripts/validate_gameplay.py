#!/usr/bin/env python3
"""Validate owned gameplay against a disposable vanilla 26.3 server.

Rust fixtures own gameplay and expose canonical State objective metadata. This
harness only arranges actors, advances server ticks and checks observable results.
Marker actors test entity ownership; real player connection tests use the existing
player_validation_harness on its verified client protocol separately.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from vanilla_reload_harness import ServerHarness, STARTUP, ValidationFailure

ROOT = Path(__file__).resolve().parents[1]


class GameplayHarness(ServerHarness):
    """Command barriers and tick advancement shared by native gameplay proofs."""

    def barrier(self, label):
        token = f"__SAND_BARRIER_{label}__"
        offset = len(self.lines)
        self.send(f"say {token}")
        return self.wait_for(token, label, offset)

    def advance(self, ticks):
        offset = len(self.lines)
        self.send(f"schedule function owned:barrier {ticks}t replace")
        self.send(f"tick step {ticks + 1}t")
        self.wait_for("__TICK_BARRIER__", f"advance {ticks} ticks", offset)
        self.barrier(f"stepped_{ticks}")

    def exact(self, ticks):
        """Advance with a counter at the end of the tick tag, not a scheduled barrier."""
        offset = len(self.lines)
        self.send(f"scoreboard players set #remaining sand_proof {ticks}")
        self.send(f"tick step {ticks}t")
        self.wait_for("__EXACT_TICKS__", f"exactly {ticks} ticks", offset)
        self.barrier("exact_ticks_done")

    def invoke(self, name, actor=None):
        command = f"function owned:{name}"
        self.send(f"execute as @e[tag={actor},limit=1] at @s run {command}" if actor else command)
        self.barrier(name)

    def matches(self, condition, label):
        token = f"__SAND_CHECK_{label}__"
        offset = len(self.lines)
        self.send(f"execute {condition} run say {token}")
        end = self.barrier(f"checked_{label}")
        # A parser diagnostic can echo the entire submitted command, including
        # its marker. Only a completed chat emission proves the condition ran.
        marker = re.compile(rf"\] {re.escape(token)}$")
        return any(marker.search(line) for line in self.lines[offset:end + 1])

    def check(self, condition, label):
        if not self.matches(condition, label):
            raise ValidationFailure(label, f"failed runtime assertion: {condition}")

    def wait_loaded(self, position):
        for _ in range(40):
            self.advance(20)
            if self.matches(f"if loaded {position}", "chunk_loaded"):
                return
        raise ValidationFailure("chunk preparation", "forced chunk did not become loaded")

    def count(self, marker, offset=0):
        return sum(re.search(rf"\] {re.escape(marker)}$", line) is not None for line in self.lines[offset:])

    def expect_count(self, marker, count, offset=0):
        actual = self.count(marker, offset)
        if actual != count:
            raise ValidationFailure(marker, f"expected {count} occurrences, observed {actual}")


def run(h, metadata):
    h.wait_for(STARTUP, "startup")
    if h.errors(0, len(h.lines)):
        raise ValidationFailure("startup", "generated pack failed to load", h.errors(0, len(h.lines)))
    h.send("forceload add 0 0")
    # The fixture's load probe is scheduled in-world; it never guesses how long
    # asynchronous chunk generation needs in wall-clock seconds.
    h.send("tick freeze")
    h.send("schedule function owned:wait_loaded 1t replace")
    for _ in range(20):
        h.advance(20)
        if any("__CHUNK_READY__" in line for line in h.lines):
            break
    else:
        raise ValidationFailure("chunk preparation", "forced chunk did not become loaded")
    for actor, x in [("owner_a", 2), ("owner_b", 6)]:
        h.send(f'summon minecraft:marker {x} 200 2 {{Tags:["{actor}"]}}')
    h.advance(1)
    for actor in ["owner_a", "owner_b"]:
        h.invoke("attach", actor)
        h.invoke("activate", actor)
    h.invoke("activate", "owner_a")
    h.expect_count("OWNED_START", 2)
    mana = metadata["mana"]
    for actor in ["owner_a", "owner_b"]:
        h.check(f"if score @e[tag={actor},limit=1] {mana} matches 7", f"cost_{actor}")
    h.invoke("cancel", "owner_a")
    h.expect_count("OWNED_STOP", 1)
    h.advance(4)
    h.expect_count("OWNED_STOP", 2)

    # A stop inside on_tick must not recreate scores on the removed marker.
    h.send(f"scoreboard players set #before {mana} 0")
    h.send(f"scoreboard players set #after {mana} 0")
    h.send(f"execute store result score #before {mana} run scoreboard players list")
    h.invoke("self_stopping", "owner_a")
    h.advance(1)
    h.send(f"execute store result score #after {mana} run scoreboard players list")
    h.check(f"if score #before {mana} = #after {mana}", "self_stop_does_not_leak_score_holders")

    # Per-instance job budgets, cancellation, completion and stop-from-step cleanup.
    h.send("tick step stop")
    h.barrier("stop_pending_steps")
    offset = len(h.lines)
    for actor in ["owner_a", "owner_b"]:
        h.invoke("job_start", actor)
    h.exact(2)
    h.expect_count("JOB_STEP", 8, offset)
    h.expect_count("JOB_COMPLETE", 0, offset)
    h.invoke("job_cancel", "owner_a")
    h.exact(2)
    h.expect_count("JOB_STEP", 9, offset)
    h.expect_count("JOB_COMPLETE", 1, offset)
    h.send(f"execute store result score #before {mana} run scoreboard players list")
    h.invoke("job_stop_inside", "owner_a")
    h.exact(1)
    h.send(f"execute store result score #after {mana} run scoreboard players list")
    h.check(f"if score #before {mana} = #after {mana}", "job_stop_does_not_leak_scores")
    h.expect_count("UNEXPECTED_COMPLETE", 0)

    h.invoke("one_tick_job", "owner_a")
    h.exact(1)
    h.expect_count("ONE_TICK_STEP", 1)
    h.expect_count("ONE_TICK_COMPLETE", 1)

    # Ordered actions share a tick; waits are exact and cadence-independent.
    offset = len(h.lines)
    for actor in ["owner_a", "owner_b"]:
        h.invoke("timeline", actor)
    h.expect_count("SEQUENCE_FIRST", 0, offset)
    h.exact(1)
    h.expect_count("SEQUENCE_FIRST", 2, offset)
    h.expect_count("SEQUENCE_SECOND", 2, offset)
    observed = [line.rsplit("] ", 1)[-1] for line in h.lines[offset:] if re.search(r"\] SEQUENCE_(FIRST|SECOND)$", line)]
    if observed != ["SEQUENCE_FIRST", "SEQUENCE_SECOND"] * 2:
        raise ValidationFailure("sequence order", str(observed))
    h.invoke("timeline_cancel", "owner_a")
    h.exact(1)
    h.expect_count("SEQUENCE_THIRD", 0, offset)
    h.exact(1)
    h.expect_count("SEQUENCE_THIRD", 1, offset)
    h.expect_count("SEQUENCE_COMPLETE", 0, offset)
    h.exact(1)
    h.expect_count("SEQUENCE_COMPLETE", 1, offset)
    h.expect_count("SEQUENCE_STOP", 2, offset)
    h.invoke("stopped_timeline", "owner_a")
    h.invoke("expired_timeline", "owner_b")
    h.exact(3)
    h.expect_count("UNEXPECTED_SEQUENCE_ACTION", 0)
    h.expect_count("UNEXPECTED_COMPLETE", 0)

    offset = len(h.lines)
    for actor in ["owner_a", "owner_b"]:
        h.invoke("conditional_timeline", actor)
    h.exact(2)
    h.expect_count("GATE_START", 2, offset)
    h.expect_count("GATE_READY", 0, offset)
    h.send(f"scoreboard players set @e[tag=owner_a,limit=1] {mana} 9")
    h.exact(1)
    h.expect_count("GATE_READY", 1, offset)
    h.expect_count("GATE_COMPLETE", 0, offset)
    h.exact(1)
    h.expect_count("GATE_COMPLETE", 1, offset)
    h.expect_count("GATE_STOP", 2, offset)
    h.send(f"scoreboard players set @e[tag=owner_a,limit=1] {mana} 7")

    # Two parents owned by the same entity must not cancel each other's children.
    offset = len(h.lines)
    h.invoke("parents_start", "owner_a")
    h.expect_count("CHILD_STARTED", 2, offset)
    h.invoke("parent_a_cancel", "owner_a")
    h.expect_count("CHILD_STOPPED", 1, offset)
    h.expect_count("CHILD_STARTED", 2, offset)
    h.exact(1)
    h.expect_count("CHILD_STEP", 1, offset)
    h.invoke("parent_b_cancel", "owner_a")
    h.expect_count("CHILD_STOPPED", 2, offset)
    # A cancelled unloaded parent invalidates a loaded child through its ledger.
    offset = len(h.lines)
    h.invoke("parent_a_start", "owner_a")
    parent_tag = "sand.task." + "owned:parent_a".encode().hex()
    h.send(f"tp @e[tag={parent_tag}] 100000 200 100000")
    h.exact(1)
    h.invoke("parent_a_cancel", "owner_a")
    h.exact(1)
    h.expect_count("CHILD_STOPPED", 1, offset)
    child_tag = "sand.task." + "owned:owned_child".encode().hex()
    h.check(f"unless entity @e[tag={child_tag}]", "unloaded_parent_invalidates_child")

    # A loaded grandchild must notice cancellation through an unloaded intermediate.
    offset = len(h.lines)
    h.invoke("parent_a_start", "owner_a")
    h.send(f"tp @e[tag={parent_tag}] 100032 200 100032")
    h.send(f"tp @e[tag={child_tag}] 100032 200 100032")
    h.exact(1)
    h.invoke("parent_a_cancel", "owner_a")
    h.exact(1)
    h.expect_count("GRANDCHILD_STOP", 1, offset)
    h.expect_count("CHILD_STOPPED", 0, offset)
    offset = len(h.lines)
    h.invoke("join_parent", "owner_a")
    h.exact(1)
    h.expect_count("JOIN_PARENT_DONE", 0, offset)
    h.exact(4)
    h.expect_count("JOIN_CHILD_DONE", 2, offset)
    h.expect_count("JOIN_PARENT_DONE", 1, offset)
    ordered = [line.rsplit("] ", 1)[-1] for line in h.lines[offset:] if re.search(r"\] JOIN_(CHILD|PARENT)_DONE$", line)]
    if ordered != ["JOIN_CHILD_DONE", "JOIN_CHILD_DONE", "JOIN_PARENT_DONE"]:
        raise ValidationFailure("child join order", str(ordered))

    # Sphere origins versus box hitboxes, translated centers, nested queries,
    # candidate limits and dimension isolation. Actors have no AI or gravity.
    h.send('summon minecraft:marker 0.0 200.0 0.0 {Tags:["query_origin"]}')
    for name, x in [("near", 0.5), ("edge", 2.0), ("outside", 2.01)]:
        h.send(f'summon minecraft:marker {x} 200.0 0.0 {{Tags:["hit_candidate","hit_{name}"]}}')
    h.send('summon minecraft:zombie 1.2 200.0 0.0 {NoAI:1b,NoGravity:1b,Silent:1b,Tags:["hit_candidate","hit_overlap"]}')
    h.invoke("sphere_hits", "query_origin")
    for name in ["near", "edge", "overlap"]:
        h.check(f"if entity @e[tag=hit_{name},tag=sphere_hit]", f"sphere_{name}")
    h.check("unless entity @e[tag=hit_outside,tag=sphere_hit]", "sphere_outside")
    h.invoke("box_hits", "query_origin")
    h.check("if entity @e[tag=hit_overlap,tag=box_hit]", "box_intersects_hitbox_outside_origin")
    h.check("unless entity @e[tag=hit_edge,tag=box_hit]", "box_outside")
    h.invoke("nested_hits", "query_origin")
    for name in ["near", "edge", "overlap"]:
        h.check(f"if entity @e[tag=hit_{name},tag=nested_hit,tag=outer_hit]", f"nested_query_{name}")
    h.invoke("budgeted_hits", "query_origin")
    h.check("unless entity @e[tag=budget_hit]", "limit_applies_before_translated_volume")
    h.invoke("first_hit", "query_origin")
    h.check("if entity @e[tag=hit_edge,tag=first_hit]", "first_successful_hit_skips_outside_candidates")
    h.check("unless entity @e[tag=hit_outside,tag=first_hit]", "first_hit_returns_even_when_callback_returns_zero")
    h.expect_count("AFTER_FIRST_HIT", 1)
    h.send("kill @e[tag=hit_candidate]")
    h.send('summon minecraft:armor_stand 0.0 200.0 0.1 {NoGravity:1b,Tags:["thin_hit"]}')
    h.invoke("narrow_projectile", "query_origin")
    h.exact(1)
    h.expect_count("NARROW_HIT", 1)
    h.send("kill @e[tag=thin_hit]")
    h.send("kill @e[tag=query_origin]")

    # Moving from an earlier scanned dimension to a later one cannot double-step.
    h.send("execute in minecraft:the_nether run forceload add 0 0")
    for _ in range(40):
        h.exact(20)
        if h.matches("in minecraft:the_nether if loaded 2 200 2", "nether_ready"):
            break
    else:
        raise ValidationFailure("nether preparation", "forced chunk did not load")
    h.send('execute in minecraft:the_nether run summon minecraft:marker 2.5 200 2.5 {Tags:["owner_cross"]}')
    h.invoke("crossing_job", "owner_cross")
    h.exact(1)
    h.expect_count("CROSSING_STEP", 1)
    h.expect_count("CROSSING_COMPLETE", 0)
    h.exact(1)
    h.expect_count("CROSSING_STEP", 2)
    h.expect_count("CROSSING_COMPLETE", 1)

    h.send("execute in minecraft:the_nether run tp @e[tag=owner_cross,limit=1] 10.5 200 2.5")
    h.invoke("crossing_timeline", "owner_cross")
    h.exact(1)
    h.expect_count("SEQUENCE_CROSSED", 1)
    h.expect_count("SEQUENCE_CROSS_COMPLETE", 0)
    h.exact(1)
    h.expect_count("SEQUENCE_CROSS_COMPLETE", 1)
    sequence_tag = "sand.task." + "owned:crossing_timeline".encode().hex()
    h.check(f"unless entity @e[tag={sequence_tag}]", "crossing_sequence_cleanup")

    # Non-player teleports replace the entity object. Continuations and cleanup
    # must reacquire the live copy, including a second transfer during on_stop.
    for definition in ["crossing_expiry", "crossing_stopped_job", "crossing_scene"]:
        h.send("execute in minecraft:the_nether run tp @e[tag=owner_cross,limit=1] 10.5 200 2.5")
        h.send(f"execute store result score #before {mana} run scoreboard players list")
        h.invoke(definition, "owner_cross")
        h.exact(1)
        tag = "sand.task." + f"owned:{definition}".encode().hex()
        h.check(f"unless entity @e[tag={tag}]", f"{definition}_removed_live_copy")
        h.send(f"execute store result score #after {mana} run scoreboard players list")
        h.check(f"if score #before {mana} = #after {mana}", f"{definition}_no_score_leak")
    h.expect_count("CROSS_EXPIRY_STOP", 1)
    h.expect_count("CROSS_JOB_STOP", 1)
    h.expect_count("UNEXPECTED_COMPLETE", 0)
    h.check("unless entity @e[type=minecraft:block_display]", "crossing_scene_no_orphan_parts")

    # Display parts are associated with their instance, never nearest-owner guesses.
    h.send(f"execute store result score #before {mana} run scoreboard players list")
    for actor in ["owner_a", "owner_b"]:
        h.invoke("scene_start", actor)
    h.exact(1)
    h.check('if entity @e[type=minecraft:block_display,x=2.5,y=200,z=2.5,distance=..0.1,nbt={block_state:"minecraft:gold_block",transformation:{translation:[0.0f,3.0f,0.0f]},interpolation_duration:4}]', "native_keyframe_target")
    h.check('if entity @e[type=minecraft:item_display,x=2.5,y=200,z=2.5,distance=..0.1,nbt={item:{id:"minecraft:diamond",count:3,components:{"minecraft:custom_name":{text:"Gem",color:"gold"}}}}]', "native_scene_item_components")
    h.check('if entity @e[type=minecraft:text_display,x=2.5,y=200,z=2.5,distance=..0.1,nbt={text:{text:"Ready",bold:1b}}]', "native_scene_rich_text")
    h.send("tp @e[tag=owner_a] 3.5 200 2.5")
    h.exact(1)
    h.check('if entity @e[type=minecraft:block_display,x=3.5,y=200,z=2.5,distance=..0.1]', "scene_follows_owner")
    h.invoke("scene_cancel", "owner_a")
    h.check('unless entity @e[type=minecraft:block_display,x=3.5,y=200,z=2.5,distance=..0.1]', "scene_cancel_isolated_a")
    for kind in ["item_display", "text_display"]:
        h.check(f'unless entity @e[type=minecraft:{kind},x=3.5,y=200,z=2.5,distance=..0.1]', f"scene_cancel_isolated_a_{kind}")
        h.check(f'if entity @e[type=minecraft:{kind},x=6.5,y=200,z=2.5,distance=..0.1]', f"scene_cancel_preserves_b_{kind}")
    h.check('if entity @e[type=minecraft:block_display,x=6.5,y=200,z=2.5,distance=..0.1]', "scene_cancel_preserves_b")
    h.invoke("scene_cancel", "owner_b")
    h.check('unless entity @e[type=minecraft:block_display]', "scene_cleanup_survives_callback_return")
    h.check('unless entity @e[type=minecraft:item_display]', "scene_item_cleanup_survives_callback_return")
    h.check('unless entity @e[type=minecraft:text_display]', "scene_text_cleanup_survives_callback_return")
    h.send(f"execute store result score #after {mana} run scoreboard players list")
    h.check(f"if score #before {mana} = #after {mana}", "scene_cleanup_resets_scores")
    h.send("tp @e[tag=owner_a] 2.5 200 2.5")

    # Native numeric read, exact add, and abort-before-commit at intermediate overflow.
    h.invoke("numeric_step", "owner_a")
    h.check(f"if score @e[tag=owner_a,limit=1] {mana} matches 8", "numeric_add")
    h.send(f"scoreboard players set @e[tag=owner_a] {mana} 2147483")
    h.invoke("numeric_step", "owner_a")
    h.check(f"if score @e[tag=owner_a,limit=1] {mana} matches 2147483", "numeric_overflow_preserves_destination")
    h.send(f"scoreboard players reset @e[tag=owner_a] {mana}")
    h.invoke("numeric_step", "owner_a")
    h.check(f"unless score @e[tag=owner_a,limit=1] {mana} matches -2147483648..", "missing_input_preserves_destination")
    h.send(f"scoreboard players set @e[tag=owner_a] {mana} 10")

    phase = metadata["phase"]
    offset = len(h.lines)
    h.invoke("transition", "owner_a")
    h.expect_count("PHASE_EXIT", 1, offset)
    h.expect_count("PHASE_ENTER", 1, offset)
    h.send(f"scoreboard players set @e[tag=owner_a] {phase} 0")
    h.invoke("transition_nested", "owner_a")
    h.expect_count("NESTED_ENTER", 1, offset)
    h.expect_count("SUPERSEDED_ENTER", 0, offset)

    offset = len(h.lines)
    h.invoke("regenerate", "owner_a")
    h.invoke("regenerate", "owner_a")
    h.expect_count("STATUS_START", 1, offset)
    h.advance(2)
    h.invoke("regenerate", "owner_a")
    h.advance(2)
    h.expect_count("STATUS_STOP", 0, offset)
    h.advance(2)
    h.expect_count("STATUS_STOP", 1, offset)
    h.invoke("regenerate", "owner_a")
    h.invoke("detach", "owner_a")
    h.advance(1)
    h.expect_count("STATUS_STOP", 2, offset)
    h.invoke("attach", "owner_a")

    offset = len(h.lines)
    for _ in range(3):
        h.invoke("stack", "owner_a")
    h.expect_count("STACK_START", 2, offset)
    h.advance(4)
    h.expect_count("STACK_STOP", 2, offset)
    h.check("unless data storage owned:__sand_system/lifetime/stack/ledger owners.o1", "stack_ledger_cleaned")

    offset = len(h.lines)
    h.send('summon minecraft:zombie 2 200 4 {Tags:["hit_target"],NoAI:1b,NoGravity:1b,Silent:1b}')
    h.advance(1)
    h.invoke("fireball", "owner_a")
    h.invoke("fireball", "owner_a")
    h.check(f"if score @e[tag=owner_a,limit=1] {mana} matches 8", "projectile_capacity_rejection_does_not_charge")
    h.advance(3)
    h.expect_count("PROJECTILE_TARGET", 1, offset)
    h.expect_count("PROJECTILE_OWNER", 1, offset)
    h.check('if entity @e[tag=hit_target,nbt={Health:18.0f}]', "attributed_damage")
    h.send("kill @e[tag=hit_target]")
    h.send("setblock 2 200 3 minecraft:stone")
    offset = len(h.lines)
    h.invoke("fireball", "owner_a")
    h.advance(2)
    h.expect_count("PROJECTILE_TARGET", 0, offset)
    h.expect_count("PROJECTILE_OWNER", 1, offset)
    h.send("setblock 2 200 3 minecraft:air")

    offset = len(h.lines)
    h.invoke("regenerate", "owner_a")
    h.send("reload")
    h.barrier("reload_requested")
    h.send("schedule function owned:barrier 2t replace")
    h.send("tick step 3t")
    h.wait_for("__TICK_BARRIER__", "reload ticks", offset)
    h.barrier("reload_cleanup")
    h.expect_count("STATUS_STOP", 1, offset)

    offset = len(h.lines)
    h.invoke("test")
    h.invoke("test")
    h.advance(4)
    h.expect_count("SCENARIO_SETUP", 1, offset)
    h.expect_count("SCENARIO_CLEANUP", 1, offset)
    h.expect_count("SAND_TEST_PASS owned:scenario", 1, offset)
    validate_combat(h, metadata)
    failures = h.errors(0, len(h.lines)) + [line for line in h.lines if "Failed to decode value" in line]
    if failures:
        raise ValidationFailure("runtime", "server reported a command or NBT decoding failure", failures)


def validate_combat(h, metadata):
    """Observe committed combat State and real native health, never emulate damage."""
    for actor, x in [("fighter_a", 10), ("fighter_b", 12)]:
        h.send(f'summon minecraft:marker {x} 200 2 {{Tags:["{actor}"]}}')
        h.invoke("combat_attach", actor)
    def score(actor, field, value, label):
        h.check(f"if score @e[tag={actor},limit=1] {metadata['combat_' + field]} matches {value}", label)
    def set_score(actor, field, value):
        h.send(f"scoreboard players set @e[tag={actor},limit=1] {metadata['combat_' + field]} {value}")
    h.invoke("combat_hit", "fighter_a")
    score("fighter_a", "health", 19, "shield_before_health")
    score("fighter_a", "shield", 0, "shield_consumed")
    score("fighter_a", "hits", 1, "one_hit_callback")
    score("fighter_b", "health", 20, "combat_multiplayer_isolation")
    h.invoke("combat_hit", "fighter_a")
    score("fighter_a", "health", 19, "invulnerability_blocks_hit")
    score("fighter_a", "hits", 1, "blocked_hit_no_event")
    h.invoke("combat_missing", "fighter_b")
    score("fighter_b", "health", 20, "failed_math_preserves_health")
    score("fighter_b", "shield", 5, "failed_math_preserves_shield")
    score("fighter_b", "immunity", 0, "failed_math_preserves_cooldown")
    h.invoke("combat_critical", "fighter_b")
    score("fighter_b", "health", 13, "critical_applies_once")
    score("fighter_b", "hits", 1, "overlapping_conditions_one_event")
    h.invoke("combat_heal", "fighter_b")
    score("fighter_b", "health", 20, "healing_caps_at_maximum")
    score("fighter_b", "heals", 1, "healing_event")
    h.invoke("combat_heal", "fighter_b")
    score("fighter_b", "heals", 1, "full_health_no_event")
    h.advance(4)
    h.invoke("combat_nested", "fighter_a")
    score("fighter_a", "health", 11, "nested_damage_sees_committed_health")
    h.advance(4)
    h.invoke("combat_defeat", "fighter_a")
    score("fighter_a", "health", 0, "overkill_clamps_zero")
    score("fighter_a", "defeats", 1, "defeat_event_once")
    h.invoke("combat_defeat", "fighter_a")
    h.invoke("combat_heal", "fighter_a")
    score("fighter_a", "defeats", 1, "dead_target_not_defeated_twice")
    score("fighter_a", "health", 0, "healing_does_not_revive")
    h.invoke("combat_fractional", "fighter_a")
    score("fighter_a", "fractional_health", 19500, "fractional_health_uses_declared_scale")
    score("fighter_a", "fractional_shield", 0, "fractional_shield_absorption")
    set_score("fighter_a", "health", 20)
    set_score("fighter_a", "hits", 0)
    h.invoke("combat_return", "fighter_a")
    score("fighter_a", "hits", 1, "callback_return_cannot_escape_caller")
    offset = len(h.lines)
    h.invoke("encounter_start", "fighter_a")
    h.invoke("encounter_start", "fighter_b")
    h.advance(2)
    h.expect_count("ENCOUNTER_ATTACK", 2, offset)
    h.invoke("encounter_cancel", "fighter_a")
    h.expect_count("ENCOUNTER_CHILD_STOP", 1, offset)
    h.expect_count("ENCOUNTER_STOP", 1, offset)
    set_score("fighter_b", "health", 0)
    h.advance(1)
    h.expect_count("ENCOUNTER_CHILD_STOP", 2, offset)
    h.expect_count("ENCOUNTER_STOP", 2, offset)
    h.advance(12)
    h.expect_count("ENCOUNTER_ATTACK", 2, offset)
    set_score("fighter_a", "health", 20)
    h.invoke("encounter_start", "fighter_a")
    h.advance(2)
    h.invoke("encounter_start", "fighter_a")
    h.advance(2)
    h.expect_count("ENCOUNTER_ATTACK", 4, offset)
    h.expect_count("ENCOUNTER_CHILD_STOP", 3, offset)
    h.invoke("encounter_cancel", "fighter_a")
    h.send('summon minecraft:zombie 10 200 5 {Tags:["dynamic_victim"],NoAI:1b,NoGravity:1b,Silent:1b}')
    h.invoke("combat_attach", "dynamic_victim")
    h.invoke("native_dynamic", "dynamic_victim")
    h.check('if entity @e[tag=dynamic_victim,nbt={Health:12.0f}]', "runtime_native_damage")
    h.advance(11)
    h.invoke("native_negative", "dynamic_victim")
    h.check('if entity @e[tag=dynamic_victim,nbt={Health:12.0f}]', "negative_native_damage_rejected")
    h.invoke("native_attributed", "fighter_a")
    h.check('if entity @e[tag=dynamic_victim,nbt={Health:9.5f}]', "scoped_fractional_native_damage")
    h.send("kill @e[tag=dynamic_victim]")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    pack.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, SAND_MC_VERSION="26.3", SAND_STRICT_CODEGEN="1", SAND_GAMEPLAY_PROOF_PACK=str(pack))
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_lifetime"], cwd=ROOT, env=env, check=True)
    functions = pack / "data/owned/function"
    (functions / "barrier.mcfunction").write_text("say __TICK_BARRIER__\n")
    (functions / "wait_loaded.mcfunction").write_text("execute if loaded 2 200 2 run return run say __CHUNK_READY__\nschedule function owned:wait_loaded 1t replace\n")
    (functions / "proof_tick.mcfunction").write_text("execute if score #remaining sand_proof matches 1.. run scoreboard players remove #remaining sand_proof 1\nexecute if score #remaining sand_proof matches 0 run say __EXACT_TICKS__\nexecute if score #remaining sand_proof matches 0 run scoreboard players set #remaining sand_proof -1\n")
    (functions / "proof_load.mcfunction").write_text("scoreboard objectives add sand_proof dummy\nscoreboard players set #remaining sand_proof -1\n")
    load_tag = pack / "data/minecraft/tags/function/load.json"
    load_values = json.loads(load_tag.read_text())
    load_values["values"].append("owned:proof_load")
    load_tag.write_text(json.dumps(load_values))
    tick_tag = pack / "data/minecraft/tags/function/tick.json"
    tick_values = json.loads(tick_tag.read_text())
    tick_values["values"].append("owned:proof_tick")
    tick_tag.write_text(json.dumps(tick_values))
    settings = argparse.Namespace(version="26.3", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    h = GameplayHarness(settings)
    h.prepare()
    with (h.server / "server.properties").open("a") as stream:
        stream.write("pause-when-empty-seconds=0\n")
    h.start()
    try:
        run(h, json.loads((pack / "fixture.json").read_text()))
        (root / "evidence.json").write_text(json.dumps({"minecraft":"26.3", "passed":True, "actors":"two entity owners and native zombie target", "checks":["cost and cooldown admission", "owner isolation", "cancel and expiry", "reentrant cleanup", "checked numeric addition and overflow", "enum transition hooks", "status refresh and stacks", "State detachment", "projectile entity and block hits", "owner-attributed damage", "reload cleanup", "scheduled gameplay test", "bounded job steps and cancellation", "one-tick completion and dimension crossing", "native scene block/item/text parts, typed components and ownership", "scene follow and cleanup", "sequence order, waits, cancellation and dimension changes", "native hit volumes and nested queries", "parent-scoped children and unloaded cancellation", "condition waits and timeout", "first successful hit", "concurrent child join and unloaded ancestry", "radius-aware projectile sampling", "State combat mitigation, shield, critical, healing and defeat", "combat reentry and owner isolation", "encounter selection, reset and inactive cleanup", "runtime native damage and scoped attribution"], "log":str(h.log_path)}, indent=2)+"\n")
    finally:
        h.stop()


if __name__ == "__main__":
    main()
