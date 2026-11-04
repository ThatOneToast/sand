#!/usr/bin/env python3
"""Prove bounded player-slot moves, exact-component merges and whole-slot payments on 26.2."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from player_validation_harness import PlayerHarness
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, ValidationFailure


class InventoryHarness(GameplayHarness, PlayerHarness):
    def invoke(self, name, player="SandAlpha"):
        self.send(f"execute as {player} at @s run function inventory_proof:{name}")
        self.barrier(name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    env = dict(os.environ, SAND_INVENTORY_PROOF_PACK=str(pack), SAND_INVENTORY_PROOF_VERSION="26.2")
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_inventory"], cwd=ROOT, env=env, check=True)
    barrier = pack / "data/owned/function/barrier.mcfunction"
    barrier.parent.mkdir(parents=True, exist_ok=True)
    barrier.write_text("say __TICK_BARRIER__\n")
    system = pack / "data/inventory_proof/function/__sand_system/lifetime"
    move = (system / "move/inventory_transfer_move.mcfunction").read_text()
    lock = re.search(r"execute if score #busy (\S+) matches 1", move)[1]
    # Deliberately invoke a second Sand workflow synchronously from a native
    # inventory listener. The pack lock must reject it before any mutation.
    functions = pack / "data/inventory_proof/function"
    (functions / "reentry.mcfunction").write_text(f"advancement revoke @s only inventory_proof:reentry\nexecute if score #armed proof matches 1 if score #busy {lock} matches 1 run scoreboard players add #reentry proof 1\nexecute if score #armed proof matches 1 run function inventory_proof:move_stack\nexecute if score #armed proof matches 1 run function inventory_proof:foreign_attempt\nexecute if score #tamper proof matches 1 if items entity @s hotbar.2 minecraft:diamond run function inventory_proof:tamper\nexecute if score #source_tamper proof matches 1 if score #busy {lock} matches 1 if items entity @s hotbar.1 * run function inventory_proof:tamper_source\nexecute if score #destination_tamper proof matches 1 if score #busy {lock} matches 1 if items entity @s hotbar.1 * run function inventory_proof:tamper_destination\n")
    (functions / "tamper.mcfunction").write_text("scoreboard players set #tamper proof 0\nitem replace entity @s hotbar.3 with minecraft:stone\nexecute if score #reward_mode proof matches 1 run item replace entity @s hotbar.2 with minecraft:gold_ingot 4\nexecute if score #reward_mode proof matches 2 run item replace entity @s hotbar.2 with minecraft:diamond[custom_data={replacement:1}] 1\nexecute if score #reward_mode proof matches 3 run item replace entity @s hotbar.2 with minecraft:diamond 2\n")
    (functions / "tamper_source.mcfunction").write_text("scoreboard players set #source_tamper proof 0\nitem replace entity @s hotbar.0 with minecraft:diamond 2\n")
    (functions / "tamper_destination.mcfunction").write_text("scoreboard players set #destination_tamper proof 0\nitem replace entity @s hotbar.1 with minecraft:gold_ingot 2\n")
    advancements = pack / "data/inventory_proof/advancement"
    advancements.mkdir(parents=True, exist_ok=True)
    (advancements / "reentry.json").write_text(json.dumps({"criteria": {"changed": {"trigger": "minecraft:inventory_changed"}}, "rewards": {"function": "inventory_proof:reentry"}}))
    settings = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    h = InventoryHarness(settings)
    h.prepare()
    h.start()
    try:
        h.wait_for(STARTUP, "startup")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("startup", "pack load errors", h.errors(0, len(h.lines)))
        h.send("scoreboard objectives add proof dummy")
        h.send("scoreboard players set #armed proof 0")
        h.send("scoreboard players set #reentry proof 0")
        h.send("scoreboard players set #tamper proof 0")
        h.send("scoreboard players set #reward_mode proof 0")
        h.send("scoreboard players set #source_tamper proof 0")
        h.send("scoreboard players set #destination_tamper proof 0")
        with h.clients(players=("SandAlpha", "SandBeta")):
            h.send("gamemode survival @a")
            h.send("forceload add 0 0")
            h.send("tp @a 2 200 2")
            h.send("tick freeze")
            h.advance(1)
            h.send("clear @a")
            h.invoke("move_stack")
            h.expect_count("MOVE_REJECTED", 1)
            h.send('item replace entity SandAlpha hotbar.0 with minecraft:stone[custom_data={proof:"kept"}] 7')
            h.send('item replace entity SandBeta hotbar.0 with minecraft:dirt 4')
            h.send('item replace entity SandAlpha hotbar.4 with minecraft:dirt 9')
            h.send("scoreboard players set #armed proof 1")
            h.invoke("move_stack")
            h.send("scoreboard players set #armed proof 0")
            h.check('if items entity SandAlpha hotbar.1 minecraft:stone[count=7,custom_data~{proof:"kept"}] unless items entity SandAlpha hotbar.0 *', "empty_move_preserves_count_components_and_removes_source")
            h.check("if items entity SandBeta hotbar.0 minecraft:dirt[count=4] unless items entity SandBeta hotbar.1 *", "other_player_unchanged")
            h.check(f"if score #busy {lock} matches 0", "lock_released")
            h.check("if score #reentry proof matches 1..", "native_listener_reentered_while_locked")
            h.expect_count("MOVE_OK", 1)
            h.check("if items entity SandAlpha hotbar.4 minecraft:dirt[count=9] unless items entity SandAlpha hotbar.5 * unless items entity SandAlpha hotbar.6 * unless items entity SandAlpha hotbar.7 *", "cross_namespace_transfer_purchase_journal_reentry_cannot_mutate")
            for marker in ("FOREIGN_MOVE", "FOREIGN_PURCHASE", "FOREIGN_JOURNAL"):
                h.expect_count(marker + "_OK", 0)
                h.expect_count(marker + "_REJECTED", 0)
            h.invoke("foreign_attempt")
            h.check("unless items entity SandAlpha hotbar.4 * if items entity SandAlpha hotbar.5 minecraft:dirt[count=9] if items entity SandAlpha hotbar.6 minecraft:emerald if items entity SandAlpha hotbar.7 minecraft:written_book", "cross_namespace_workflows_work_after_unlock")
            for marker in ("FOREIGN_MOVE", "FOREIGN_PURCHASE", "FOREIGN_JOURNAL"):
                h.expect_count(marker + "_OK", 1)
            h.send("clear SandAlpha")
            for target, count in ((0, 3), (1, 5)):
                h.send(f'item replace entity SandAlpha hotbar.{target} with minecraft:stone[max_stack_size=16,custom_data={{proof:"merge"}}] {count}')
            h.invoke("move_stack")
            h.check('if items entity SandAlpha hotbar.1 minecraft:stone[count=8,max_stack_size=16,custom_data~{proof:"merge"}] unless items entity SandAlpha hotbar.0 *', "exact_components_merge")
            h.send('item replace entity SandAlpha hotbar.0 with minecraft:stone[max_stack_size=16,custom_data={proof:"other"}] 3')
            h.invoke("move_stack")
            h.check('if items entity SandAlpha hotbar.0 minecraft:stone[count=3] if items entity SandAlpha hotbar.1 minecraft:stone[count=8]', "component_mismatch_preserves_both")
            h.send('item replace entity SandAlpha hotbar.0 with minecraft:stone[max_stack_size=16,custom_data={proof:"merge"}] 10')
            h.invoke("move_stack")
            h.check('if items entity SandAlpha hotbar.0 minecraft:stone[count=10] if items entity SandAlpha hotbar.1 minecraft:stone[count=8]', "capacity_failure_preserves_both")
            h.send("item replace entity SandAlpha hotbar.0 with minecraft:stone 2")
            h.send("item replace entity SandAlpha hotbar.1 with minecraft:stone 2")
            h.invoke("move_stack")
            h.check("if items entity SandAlpha hotbar.0 minecraft:stone[count=2] if items entity SandAlpha hotbar.1 minecraft:stone[count=2]", "unknown_default_capacity_rejects_merge")
            h.invoke("move_stack", "SandBeta")
            h.check("if items entity SandBeta hotbar.1 minecraft:dirt[count=4] unless items entity SandBeta hotbar.0 *", "second_player_independent_move")
            h.send("clear SandAlpha")
            h.send("item replace entity SandAlpha hotbar.0 with minecraft:stone 7")
            h.send("scoreboard players set #source_tamper proof 1")
            h.invoke("move_stack")
            h.check("if items entity SandAlpha hotbar.0 minecraft:diamond[count=2] unless items entity SandAlpha hotbar.1 *", "empty_move_preserves_listener_replaced_source_and_compensates_copy")
            h.send('item replace entity SandAlpha hotbar.0 with minecraft:stone[max_stack_size=16] 3')
            h.send('item replace entity SandAlpha hotbar.1 with minecraft:stone[max_stack_size=16] 5')
            h.send("scoreboard players set #source_tamper proof 1")
            h.invoke("move_stack")
            h.check("if items entity SandAlpha hotbar.0 minecraft:diamond[count=2] if items entity SandAlpha hotbar.1 minecraft:stone[count=5,max_stack_size=16]", "merge_preserves_listener_replaced_source_and_restores_prior_count")
            h.send("item replace entity SandAlpha hotbar.0 with minecraft:stone 3")
            h.send("item replace entity SandAlpha hotbar.1 with air")
            h.send("scoreboard players set #destination_tamper proof 1")
            h.invoke("move_stack")
            h.check("if items entity SandAlpha hotbar.0 minecraft:stone[count=3] if items entity SandAlpha hotbar.1 minecraft:gold_ingot[count=2]", "listener_replaced_destination_is_preserved_and_source_retained")
            h.send("clear SandAlpha")
            h.send("item replace entity SandAlpha hotbar.3 with minecraft:emerald 2")
            h.invoke("buy")
            h.check("if items entity SandAlpha hotbar.3 minecraft:emerald[count=2] unless items entity SandAlpha hotbar.2 *", "insufficient_payment_preserved")
            h.send("item replace entity SandAlpha hotbar.3 with minecraft:emerald 4")
            h.invoke("buy")
            h.check("if items entity SandAlpha hotbar.3 minecraft:emerald[count=4] unless items entity SandAlpha hotbar.2 *", "whole_slot_payment_rejects_excess")
            h.send("item replace entity SandAlpha hotbar.3 with minecraft:emerald 3")
            h.send("item replace entity SandAlpha hotbar.2 with minecraft:stone")
            h.invoke("buy")
            h.check("if items entity SandAlpha hotbar.3 minecraft:emerald[count=3] if items entity SandAlpha hotbar.2 minecraft:stone", "full_reward_slot_does_not_charge")
            h.send("item replace entity SandAlpha hotbar.2 with air")
            h.send("scoreboard players set #tamper proof 1")
            h.invoke("buy")
            h.check("if items entity SandAlpha hotbar.3 minecraft:stone unless items entity SandAlpha hotbar.2 *", "changed_payment_compensates_reward_without_consuming_replacement")
            for mode, replacement in ((1, "minecraft:gold_ingot[count=4]"), (2, "minecraft:diamond[count=1,custom_data~{replacement:1}]"), (3, "minecraft:diamond[count=2]")):
                h.send("item replace entity SandAlpha hotbar.2 with air")
                h.send("item replace entity SandAlpha hotbar.3 with minecraft:emerald 3")
                h.send(f"scoreboard players set #reward_mode proof {mode}")
                h.send("scoreboard players set #tamper proof 1")
                h.invoke("buy")
                h.check(f"if items entity SandAlpha hotbar.3 minecraft:stone if items entity SandAlpha hotbar.2 {replacement}", f"changed_payment_preserves_replaced_reward_mode_{mode}")
            h.send("scoreboard players set #reward_mode proof 0")
            h.send("item replace entity SandAlpha hotbar.2 with air")
            h.send("item replace entity SandAlpha hotbar.3 with minecraft:emerald 3")
            h.invoke("buy")
            h.check("if items entity SandAlpha hotbar.2 minecraft:diamond unless items entity SandAlpha hotbar.3 *", "whole_payment_consumed_after_reward")
            h.invoke("buy")
            h.expect_count("PAYMENT_OK", 1)
            h.send("clear SandBeta")
            h.send("item replace entity SandBeta hotbar.3 with minecraft:emerald 3")
            h.invoke("buy", "SandBeta")
            h.check("if items entity SandBeta hotbar.2 minecraft:diamond unless items entity SandBeta hotbar.3 *", "payment_owner_isolation")
            h.expect_count("PAYMENT_OK", 2)
            h.send("clear SandBeta")
            h.send("item replace entity SandBeta hotbar.3 with minecraft:emerald 3")
            h.invoke("buy_marked", "SandBeta")
            h.check("if items entity SandBeta hotbar.3 minecraft:emerald[count=3] unless items entity SandBeta hotbar.2 *", "component_payment_rejects_unmarked_identity")
            h.send('item replace entity SandBeta hotbar.3 with minecraft:emerald[custom_data={currency:true,other:1}] 3')
            h.invoke("buy_marked", "SandBeta")
            h.check("if items entity SandBeta hotbar.2 minecraft:diamond unless items entity SandBeta hotbar.3 *", "component_payment_accepts_canonical_partial_identity")
            h.expect_count("COMPONENT_PAYMENT_OK", 1)
        errors = h.errors(0, len(h.lines))
        if errors:
            raise ValidationFailure("runtime", "server errors", errors)
        (root / "evidence.json").write_text(json.dumps({"passed": True, "version": "26.2", "players": 2, "checks": ["exact live count/components", "source removal", "source-replacement compensation for empty and merged slots", "destination-replacement preservation", "empty source", "capacity and component mismatch rejection", "conservative default capacity", "native listener reentry", "cross-namespace transfer/purchase/journal reentry", "whole-slot payment quantity", "full reward slot", "changed-payment compensation", "changed payment and reward preserve different ID/components/count", "replay without payment", "multiplayer isolation"]}, indent=2) + "\n")
    finally:
        h.stop()


if __name__ == "__main__":
    main()
