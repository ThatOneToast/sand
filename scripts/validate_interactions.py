#!/usr/bin/env python3
"""Prove menu sessions, personal rewards and interaction cleanup with two 26.2 players."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from player_validation_harness import PlayerHarness
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, ValidationFailure


class InteractionHarness(GameplayHarness, PlayerHarness):
    def invoke(self, name, player="SandAlpha"):
        self.send(f"execute as {player} at @s run function menus:{name}")
        self.barrier(name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    env = dict(os.environ, SAND_INTERACTION_PROOF_PACK=str(pack), SAND_INTERACTION_PROOF_VERSION="26.2")
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_menu"], cwd=ROOT, env=env, check=True)
    barrier = pack / "data/owned/function/barrier.mcfunction"
    barrier.parent.mkdir(parents=True, exist_ok=True)
    barrier.write_text("say __TICK_BARRIER__\n")
    metadata = json.loads((pack / "fixture.json").read_text())
    functions = pack / "data/menus/function/__sand_system/lifetime"
    open_body = (functions / "shop/menu_admit.mcfunction").read_text() + (functions / "shop/menu_page_0.mcfunction").read_text()
    trigger = re.search(r"scoreboard players enable @s (\S+)", open_body)[1]
    token = re.search(r"storage menus:shop menu.a0 int 1 run scoreboard players get @s (\S+)", open_body)[1]
    (pack / "data/menus/function/proof_submit.mcfunction").write_text(f"$trigger {trigger} set $(value)\n")
    settings = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    h = InteractionHarness(settings)
    h.prepare()
    h.start()
    try:
        h.wait_for(STARTUP, "startup")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("startup", "pack load errors", h.errors(0, len(h.lines)))
        h.send("scoreboard objectives add proof dummy")
        with h.clients(players=("SandAlpha",)):
            with h.clients(players=("SandBeta",)):
                h.send("gamemode creative @a")
                h.send("forceload add 0 0")
                h.send("tp @a 2 200 2")
                h.send("tick freeze")
                h.advance(1)
                for player in ("SandAlpha", "SandBeta"):
                    h.invoke("attach", player)
                    h.send(f"clear {player}")
                h.invoke("open")
                h.send(f"scoreboard players operation #stale proof = SandAlpha {token}")
                h.invoke("open")
                h.send(f"scoreboard players operation SandAlpha {trigger} = #stale proof")
                h.advance(1)
                h.expect_count("PURCHASE", 0)
                h.invoke("open")
                h.send(f"execute store result storage menus:proof submit.value int 1 run scoreboard players get SandAlpha {token}")
                h.send("execute as SandAlpha run function menus:proof_submit with storage menus:proof submit")
                h.advance(1)
                h.expect_count("PURCHASE", 1)
                h.send(f"scoreboard players operation SandAlpha {trigger} = #stale proof")
                h.advance(1)
                h.expect_count("PURCHASE", 1)
                h.invoke("open")
                h.send(f"scoreboard players operation SandBeta {trigger} = SandAlpha {token}")
                h.advance(1)
                h.expect_count("PURCHASE", 1)
                h.invoke("open")
                h.send(f"scoreboard players operation #invalid proof = SandAlpha {token}")
                h.send(f"scoreboard players set SandAlpha {metadata['coins']} 0")
                h.send(f"scoreboard players operation SandAlpha {trigger} = #invalid proof")
                h.advance(1)
                h.expect_count("PURCHASE", 1)
                h.send(f"scoreboard players set SandAlpha {metadata['coins']} 10")
                h.invoke("open")
                h.send(f"scoreboard players operation #expired proof = SandAlpha {token}")
                h.advance(201)
                h.send(f"scoreboard players operation SandAlpha {trigger} = #expired proof")
                h.advance(1)
                h.expect_count("PURCHASE", 1)
                h.send("item replace entity SandAlpha hotbar.0 with minecraft:stone")
                h.invoke("buy")
                h.check(f"if score SandAlpha {metadata['coins']} matches 10", "full_slot_does_not_charge")
                h.send("item replace entity SandAlpha hotbar.0 with minecraft:air")
                h.invoke("buy")
                h.check(f"if score SandAlpha {metadata['coins']} matches 7", "successful_purchase_charged_once")
                h.check("if items entity SandAlpha hotbar.0 minecraft:written_book", "native_book_delivered")
                h.send("clear SandAlpha")
                h.invoke("buy")
                h.check(f"if score SandAlpha {metadata['coins']} matches 7", "claim_replay_does_not_charge")
                h.invoke("buy", "SandBeta")
                h.check(f"if score SandBeta {metadata['coins']} matches 7", "personal_claims_are_independent")
                h.invoke("shared_buy")
                h.invoke("shared_buy", "SandBeta")
                h.check("if items entity SandAlpha hotbar.1 minecraft:diamond", "shared_reward_first_claim")
                h.check("unless items entity SandBeta hotbar.1 minecraft:diamond", "shared_reward_second_claim_rejected")
                h.invoke("object_start")
                h.advance(1)
                tag = "sand.task." + b"menus:object".hex() + ".interaction"
                target = f"@e[type=minecraft:interaction,tag={tag},limit=1]"
                h.check(f"if entity {target}", "owned_hitbox_spawned")
                h.send("execute as SandAlpha run function menus:__sand_system/lifetime/shop/menu_close")
                h.send("data modify storage menus:proof event set value {timestamp:1L}")
                h.send("data modify storage menus:proof event.player set from entity SandAlpha UUID")
                h.send(f"data modify entity {target} interaction set from storage menus:proof event")
                h.check(f"if data entity {target} interaction.player", "valid_interaction_record_seeded")
                h.advance(1)
                h.check(f"unless data entity {target} interaction", "interaction_record_consumed")
                h.check(f"if score SandAlpha {token} matches 1..", "interaction_opens_actor_menu")
                h.invoke("object_stop")
                h.check(f"unless entity {target}", "owned_hitbox_cleanup")
                # Exercise pagination through the exact native hitbox binding.
                bound_dir = functions / "bound"
                bound_trigger = re.search(r"scoreboard players enable @s (\S+)", (bound_dir / "menu_admit.mcfunction").read_text())[1]
                def bound_token(index, page):
                    return re.search(rf"menu.a{index} int 1 run scoreboard players get @s (\S+)", (bound_dir / f"menu_page_{page}.mcfunction").read_text())[1]
                first = bound_token(0, 0)
                third = bound_token(2, 1)
                next_token = bound_token(3, 0)
                bound_tag = "sand.task." + b"menus:bound_object".hex() + ".interaction"
                bound_target = f"@e[type=minecraft:interaction,tag={bound_tag},limit=1]"
                h.invoke("__sand_system/lifetime/bound_object/start")
                h.advance(1)
                def click_bound():
                    h.send("data modify storage menus:proof event set value {timestamp:1L}")
                    h.send("data modify storage menus:proof event.player set from entity SandAlpha UUID")
                    h.send(f"data modify entity {bound_target} interaction set from storage menus:proof event")
                    h.advance(1)
                click_bound()
                h.check(f"if score SandAlpha {first} matches 1..", "bound_menu_opened")
                h.check(f"unless score SandAlpha {third} matches 1..", "off_page_choice_has_no_token")
                h.send(f"scoreboard players operation #old_page proof = SandAlpha {first}")
                h.send(f"scoreboard players operation SandAlpha {bound_trigger} = SandAlpha {next_token}")
                h.advance(1)
                h.check(f"if score SandAlpha {third} matches 1..", "bound_navigation_preserves_instance")
                h.check(f"unless score SandAlpha {first} matches 1..", "previous_page_token_revoked")
                h.send(f"scoreboard players operation SandAlpha {bound_trigger} = #old_page proof")
                h.advance(1)
                h.expect_count("ONE", 0)
                click_bound()
                h.send(f"scoreboard players operation SandAlpha {bound_trigger} = SandAlpha {next_token}")
                h.advance(1)
                h.send(f"scoreboard players operation SandAlpha {bound_trigger} = SandAlpha {third}")
                h.advance(1)
                h.expect_count("THREE", 1)
                click_bound()
                h.send("tp SandAlpha 20 200 20")
                h.advance(1)
                h.check(f"unless score SandAlpha {first} matches 1..", "bound_menu_closes_out_of_range")
                h.send("tp SandAlpha 2 200 2")
                click_bound()
                h.invoke("__sand_system/lifetime/bound_object/cancel")
                h.advance(1)
                h.check(f"unless score SandAlpha {first} matches 1..", "bound_menu_closes_after_object_cleanup")
                h.invoke("open", "SandBeta")
                h.send(f"scoreboard players operation #disconnected proof = SandBeta {token}")
            with h.clients(players=("SandBeta",)):
                h.send(f"scoreboard players operation SandBeta {trigger} = #disconnected proof")
                h.advance(1)
                h.expect_count("PURCHASE", 1)
            h.invoke("open")
            h.send(f"scoreboard players operation #reload proof = SandAlpha {token}")
            offset = len(h.lines)
            h.send("reload")
            h.wait_for("Reloading!", "reload", offset)
            h.barrier("reloaded")
            h.send(f"scoreboard players operation SandAlpha {trigger} = #reload proof")
            h.advance(1)
            h.expect_count("PURCHASE", 1)
        errors = h.errors(0, len(h.lines))
        if errors:
            raise ValidationFailure("runtime", "server errors", errors)
        (root / "evidence.json").write_text(json.dumps({"passed": True, "version": "26.2", "players": 2, "checks": ["pagination", "object-bound session", "bound range and cleanup invalidation", "stale tokens", "context invalidation", "timeout", "native trigger submission", "single consumption", "cross-player tokens", "full inventory", "payment ordering", "personal claims", "shared claims", "typed book", "interaction actor routing", "owned cleanup", "disconnect", "reload"]}, indent=2))
    finally:
        h.stop()


if __name__ == "__main__":
    main()
