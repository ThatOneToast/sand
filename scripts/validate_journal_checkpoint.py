#!/usr/bin/env python3
"""Prove literal journal refresh, explicit State checkpoints and safe names with two players."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from player_validation_harness import PlayerHarness
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, RELOAD_START, ValidationFailure


class JournalCheckpointHarness(GameplayHarness, PlayerHarness):
    def invoke(self, name, player="SandAlpha"):
        self.send(f"execute as {player} at @s run function journal_checkpoint:{name}")
        self.barrier(name)


def run(h, fields, journal_lock, checkpoint_lock):
    h.wait_for(STARTUP, "startup")
    if h.errors(0, len(h.lines)):
        raise ValidationFailure("startup", "pack load errors", h.errors(0, len(h.lines)))
    h.send("scoreboard objectives add proof dummy")

    def set_score(player, field, value):
        h.send(f"scoreboard players set {player} {fields[field]} {value}")
        h.barrier(f"set_{field}")

    def score(player, field, value, label):
        h.check(f"if score {player} {fields[field]} matches {value}", label)

    def nbt_text(source, path, text, label):
        h.send("data modify storage journal_checkpoint:proof expected set value " + json.dumps(text))
        h.send("scoreboard players set #text_changed proof 1")
        h.send(f"execute if data {source} {path} store success score #text_changed proof run data modify storage journal_checkpoint:proof expected set from {source} {path}")
        h.check("if score #text_changed proof matches 0", label)

    def page_text(player, index, text, label):
        root = f'Inventory[{{Slot:0b}}].components."minecraft:written_book_content".pages[0]'
        for suffix in [f".raw.extra[{index}].text", f".raw.extra[{index}]", f".extra[{index}].text", f".extra[{index}]"]:
            path=root+suffix
            if h.matches(f"if data entity {player} {path}", label+"_present"):
                nbt_text("entity "+player,path,text,label)
                return
        h.send(f"data get entity {player} Inventory[{{Slot:0b}}]")
        h.barrier("book_diagnostic")
        raise ValidationFailure(label, "serialized book does not contain the expected literal page segment")

    def no_tasks(label):
        h.check("unless entity @e[tag=sand.owned]", label)

    with h.clients(players=("SandAlpha", "SandBeta")):
        h.send("gamemode creative @a")
        h.send("forceload add 0 0")
        h.send("tp @a 2 200 2")
        h.send("tick freeze")
        h.advance(1)
        for player in ["SandAlpha", "SandBeta"]:
            h.invoke("attach", player)
            h.send(f"clear {player}")
        set_score("SandAlpha", "health", 7)
        set_score("SandBeta", "health", 3)
        h.send("item replace entity SandAlpha hotbar.0 with minecraft:stone")
        h.invoke("journal")
        h.check("if items entity SandAlpha hotbar.0 minecraft:stone", "unrelated_destination_preserved")
        score("SandAlpha", "rejected", 1, "full_slot_rejects")
        h.send("item replace entity SandAlpha hotbar.0 with minecraft:air")
        h.invoke("journal")
        score("SandAlpha", "issued", 1, "journal_native_delivery_success")
        page_text("SandAlpha", 0, 'quote " \\ $(missing)\nHealth: ', "literal_macro_tokens_survive")
        page_text("SandAlpha", 1, "7", "numeric_snapshot")
        page_text("SandAlpha", 3, "Started", "enum_snapshot")
        page_text("SandAlpha", 5, 'no " $(missing)', "quoted_flag_snapshot")
        set_score("SandAlpha", "health", 9)
        page_text("SandAlpha", 1, "7", "book_stays_snapshot_until_refresh")
        h.invoke("status_finished")
        h.send("scoreboard players set #armed proof 1")
        h.invoke("journal")
        page_text("SandAlpha", 1, "9", "explicit_identity_refresh")
        page_text("SandAlpha", 3, "Finished", "enum_refresh")
        page_text("SandAlpha", 5, 'yes \\ $(missing)', "escaped_flag_refresh")
        h.check("if score #reentry proof matches 1..", "native_inventory_listener_reentry_blocked")
        score("SandAlpha", "issued", 2, "reentry_does_not_duplicate_delivery")
        h.invoke("journal", "SandBeta")
        page_text("SandBeta", 1, "3", "other_player_journal_isolation")
        page_text("SandAlpha", 1, "9", "first_player_journal_preserved")
        set_score("SandAlpha", "health", 8)
        h.send("scoreboard players set #tamper proof 1")
        h.invoke("journal")
        h.check("if items entity SandAlpha hotbar.0 minecraft:gold_ingot[count=2]", "listener_replacement_preserved")
        h.check("if score #tamper proof matches 0", "tamper_listener_ran_on_changed_snapshot")
        score("SandAlpha", "rejected", 2, "postwrite_listener_interference_rejects")
        h.send("scoreboard players set #armed proof 0")
        h.check(f"if score #busy {journal_lock} matches 0", "inventory_lock_released")
        h.check("unless data storage journal_checkpoint:__sand_system/lifetime/journal/journal_scratch operation", "journal_scratch_reclaimed")

        set_score("SandAlpha", "health", 5)
        set_score("SandAlpha", "phase", 1)
        h.invoke("restore")
        score("SandAlpha", "prepared", 0, "unsaved_restore_rejected_before_policy")
        h.invoke("save")
        score("SandAlpha", "saved_health", 5, "explicit_saved_projection")
        score("SandAlpha", "saved_valid", 1, "validity_commits_last")
        set_score("SandBeta", "health", 2)
        h.invoke("save", "SandBeta")
        set_score("SandAlpha", "health", 9)
        set_score("SandAlpha", "phase", 0)
        h.invoke("start")
        h.invoke("restore")
        score("SandAlpha", "health", 5, "restore_live_value")
        score("SandAlpha", "phase", 1, "restore_all_fields")
        score("SandAlpha", "prepared", 1, "preparation_once_with_reentry_blocked")
        score("SandAlpha", "restored", 1, "continuation_once_with_return_isolated")
        score("SandAlpha", "saved_valid", 1, "reentrant_clear_rejected")
        score("SandBeta", "health", 2, "checkpoint_player_isolation")
        h.check("if entity @e[tag=sand.owned]", "restoration_explicitly_starts_fresh_work")
        h.check(f"if score #busy {checkpoint_lock} matches 0", "checkpoint_lock_released_after_return")

        # Corrupt the second saved field: the first live field must not commit.
        set_score("SandAlpha", "saved_health", 4)
        set_score("SandAlpha", "saved_phase", 3)
        set_score("SandAlpha", "health", 9)
        set_score("SandAlpha", "phase", 0)
        h.invoke("restore")
        score("SandAlpha", "health", 9, "all_saved_bounds_validate_before_first_write")
        score("SandAlpha", "phase", 0, "invalid_restore_no_partial_fields")
        score("SandAlpha", "prepared", 1, "invalid_saved_values_do_not_run_preparation")
        set_score("SandAlpha", "saved_phase", 1)
        set_score("SandAlpha", "health", 11)
        h.invoke("save")
        score("SandAlpha", "saved_health", 4, "out_of_bounds_save_preserves_prior_snapshot")
        set_score("SandAlpha", "health", 9)
        set_score("SandAlpha", "invalidate", 1)
        h.invoke("restore")
        score("SandAlpha", "health", 9, "preparation_invalidation_revalidated")
        score("SandAlpha", "restored", 1, "rejected_revalidation_skips_continuation")
        no_tasks("preparation_cancels_ephemeral_work_without_implicit_restoration")
        set_score("SandAlpha", "invalidate", 0)
        h.invoke("save")
        set_score("SandAlpha", "detach", 1)
        h.invoke("restore")
        h.check(f"unless score SandAlpha {fields['health']} matches -2147483648..2147483647", "detached_owner_state_not_recreated")
        set_score("SandAlpha", "detach", 0)
        h.invoke("attach")
        set_score("SandAlpha", "saved_valid", 1)
        set_score("SandAlpha", "saved_presence", 1)
        h.advance(1)
        score("SandAlpha", "saved_presence", 2, "canonical_saved_schema_migrates")
        score("SandAlpha", "saved_valid", 0, "author_migration_explicitly_invalidates_old_snapshot")
        set_score("SandAlpha", "health", 6)
        set_score("SandAlpha", "phase", 0)
        h.invoke("save")
        set_score("SandAlpha", "health", 9)
        offset = len(h.lines)
        h.send("reload")
        h.send("say __JOURNAL_CHECKPOINT_RELOAD_DONE__")
        h.wait_for(RELOAD_START, "reload-start", offset)
        h.wait_for("__JOURNAL_CHECKPOINT_RELOAD_DONE__", "reload-done", offset)
        h.advance(1)
        score("SandAlpha", "saved_health", 6, "checkpoint_persists_through_normal_state_reload")
        score("SandAlpha", "saved_valid", 1, "current_checkpoint_remains_valid_after_reload")
        score("SandAlpha", "health", 9, "reload_does_not_implicitly_restore")
        h.invoke("restore")
        score("SandAlpha", "health", 6, "explicit_restore_after_reload")
        h.check("unless data storage journal_checkpoint:__sand_system/lifetime/saved/checkpoint_scratch operation", "checkpoint_scratch_reclaimed")
        h.invoke("named")
        h.send("execute as @e[type=minecraft:marker,tag=!sand.owned,limit=1] at @s run function journal_checkpoint:named_update")
        h.advance(1)
        named="entity @e[type=minecraft:marker,tag=!sand.owned,limit=1]"
        literal_path="CustomName.extra[0].text" if h.matches(f"if data {named} CustomName.extra[0].text","name_literal_compound") else "CustomName.extra[0]"
        nbt_text(named,literal_path,'quote " \\ $(missing)\n',"dynamic_name_literal_macro_tokens_survive")
        nbt_text(named,"CustomName.extra[1].text","42","dynamic_name_numeric_materialized")
        nbt_text(named,"CustomName.extra[2].text",'on \\ $(missing)',"dynamic_name_flag_literal_materialized")
        h.send("kill @e[type=minecraft:marker,tag=!sand.owned]")

    h.advance(1)
    with h.clients(players=("SandAlpha",)):
        h.advance(1)
        score("SandAlpha", "saved_health", 6, "checkpoint_survives_player_reconnect")
        h.invoke("restore")
        score("SandAlpha", "health", 6, "explicit_restore_after_reconnect")
        h.invoke("clear_checkpoint")
        score("SandAlpha", "saved_valid", 0, "explicit_invalidation")
    if h.errors(0, len(h.lines)):
        raise ValidationFailure("runtime", "Minecraft reported command errors", h.errors(0, len(h.lines)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version", choices=["26.2", "26.3"], default="26.3")
    parser.add_argument("--reuse-export", action="store_true")
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    if not args.reuse_export:
        env = dict(os.environ, SAND_JOURNAL_CHECKPOINT_PROOF_PACK=str(pack), SAND_JOURNAL_CHECKPOINT_PROOF_VERSION=args.version)
        subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_journal_checkpoint"], cwd=ROOT, env=env, check=True)
    elif not (pack / "fixture.json").is_file():
        parser.error("--reuse-export requires an existing fixture")
    barrier = pack / "data/owned/function/barrier.mcfunction"
    barrier.parent.mkdir(parents=True, exist_ok=True)
    barrier.write_text("say __TICK_BARRIER__\n")
    functions = pack / "data/journal_checkpoint/function"
    system = functions / "__sand_system/lifetime"
    journal_lock = re.search(r"execute if score #busy (\S+) matches 1", (system / "journal/journal_refresh.mcfunction").read_text())[1]
    checkpoint_lock = re.search(r"execute if score #busy (\S+) matches 1", (system / "saved/checkpoint_restore.mcfunction").read_text())[1]
    (functions / "listener.mcfunction").write_text(f"advancement revoke @s only journal_checkpoint:listener\nexecute if score #armed proof matches 1 if score #busy {journal_lock} matches 1 run scoreboard players add #reentry proof 1\nexecute if score #armed proof matches 1 if score #busy {journal_lock} matches 1 run function journal_checkpoint:journal\nexecute if score #tamper proof matches 1 if items entity @s hotbar.0 minecraft:written_book run function journal_checkpoint:tamper\n")
    (functions / "tamper.mcfunction").write_text("scoreboard players set #tamper proof 0\nitem replace entity @s hotbar.0 with minecraft:gold_ingot 2\n")
    advancements = pack / "data/journal_checkpoint/advancement"
    advancements.mkdir(parents=True, exist_ok=True)
    (advancements / "listener.json").write_text(json.dumps({"criteria": {"changed": {"trigger": "minecraft:inventory_changed"}}, "rewards": {"function": "journal_checkpoint:listener"}}))
    settings = argparse.Namespace(version=args.version, pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    harness = JournalCheckpointHarness(settings)
    harness.prepare()
    with (harness.server / "server.properties").open("a") as stream:
        stream.write("pause-when-empty-seconds=0\n")
    harness.start()
    try:
        run(harness, json.loads((pack / "fixture.json").read_text()), journal_lock, checkpoint_lock)
        (root / "evidence.json").write_text(json.dumps({"passed": True, "minecraft": args.version, "players": 2, "checks": ["literal numeric/enum/flag journal snapshots", "safe literal macro tokens and quoting", "explicit fixed-slot identity replacement", "unrelated item preservation", "shared inventory lock native listener reentry", "listener replacement rejection/preservation", "two-player journal and checkpoint isolation", "all-field bounds prevalidation", "restore preparation revalidation after invalidation/detachment", "callback reentry and early-return cleanup", "explicit fresh session restoration", "canonical saved-schema migration/invalidation", "reload and reconnect State persistence with explicit restore", "shared dynamic entity-name materializer"], "log": str(harness.log_path)}, indent=2) + "\n")
    finally:
        harness.stop()


if __name__ == "__main__":
    main()
