#!/usr/bin/env python3
"""Prove owned HUD isolation, State progress and cleanup with two players on 26.2 or 26.3.

Observes native bossbar values/player counts and framework actionbar leases.
This is server-state evidence, not a screenshot or client text-rendering test.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from player_validation_harness import PlayerHarness
from mc_validation.protocols import profile_for_version
from validate_gameplay import GameplayHarness, ROOT
from vanilla_reload_harness import STARTUP, RELOAD_COMPLETE, ValidationFailure


class PresentationHarness(GameplayHarness, PlayerHarness):
    def invoke(self, name, player="SandAlpha"):
        self.send(f"execute as {player} at @s run function presentation:{name}")
        self.barrier(name)

    def inspect(self, actor, *, expected=None, exists=True):
        self.send(f"function presentation:proof_inspect with storage presentation:proof {actor}")
        self.barrier(f"inspect_{actor}")
        self.check(f"if score #exists hud_proof matches {int(exists)}", f"{actor}_exists_{exists}")
        if exists:
            self.check("if score #players hud_proof matches 1", f"{actor}_one_viewer")
            self.check(f"if score #value hud_proof matches {expected}", f"{actor}_value_{expected}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version", choices=("26.2", "26.3"), default="26.3")
    args = parser.parse_args()
    root = args.output.resolve()
    pack = root / "pack"
    env = dict(os.environ, SAND_PRESENTATION_PROOF_PACK=str(pack), SAND_PRESENTATION_PROOF_VERSION=args.version)
    subprocess.run(["cargo", "test", "-p", "sand-core", "--test", "gameplay_presentation"], cwd=ROOT, env=env, check=True)
    functions = pack / "data/presentation/function"
    barrier = pack / "data/owned/function/barrier.mcfunction"
    barrier.parent.mkdir(parents=True, exist_ok=True)
    barrier.write_text("say __TICK_BARRIER__\n")
    prefix = "presentation:__sand_system/lifetime/status"
    (functions / "proof_inspect.mcfunction").write_text(
        f"$execute store success score #exists hud_proof run bossbar get {prefix}/hud_bar/$(instance) value\n"
        f"$execute if score #exists hud_proof matches 1 store result score #value hud_proof run bossbar get {prefix}/hud_bar/$(instance) value\n"
        f"$execute if score #exists hud_proof matches 1 store result score #players hud_proof run bossbar get {prefix}/hud_bar/$(instance) players\n")
    metadata = json.loads((pack / "fixture.json").read_text())
    claim = (functions / "__sand_system/lifetime/status/hud_claim.mcfunction").read_text()
    lease = re.search(r"scoreboard players set @a\[.*?\] (\S+) \$\(hud_lease\)", claim)[1]
    foreign_tags = {}
    for namespace, definition in [("hud_import_a", "foreign_first"), ("hud_import_b", "foreign_second")]:
        foreign_tags[definition] = "sand.task." + f"{namespace}:{definition}".encode().hex()
        # Instrument entry into each emitted actionbar body. The production
        # owner resolution and lease comparison remain unchanged.
        actionbar = pack / f"data/{namespace}/function/__sand_system/lifetime/{definition}/hud_actionbar.mcfunction"
        actionbar.write_text(actionbar.read_text() + f"\nscoreboard players add #{definition} hud_proof 1\n")
    tag = "sand.task." + b"presentation:status".hex()
    ledger = f"{prefix}/hud_ledger"
    settings = argparse.Namespace(version=args.version, pack=str(pack), jar=str(args.jar.resolve()), output=str(root / "runtime"), timeout=120, java="java", op_player=None, client_command=None)
    h = PresentationHarness(settings)
    h.prepare()
    h.start()
    try:
        h.wait_for(STARTUP, "startup")
        h.send("scoreboard objectives add hud_proof dummy")
        with h.clients(players=("SandAlpha",)):
            with h.clients(players=("SandBeta",)):
                h.send("gamemode creative @a")
                h.send("forceload add 0 0")
                h.send("tp SandAlpha 2 200 2")
                h.send("tp SandBeta 6 200 2")
                h.send("tick freeze")
                h.wait_loaded("2 200 2")
                h.advance(1)
                for player, value in [("SandAlpha", 25), ("SandBeta", 75)]:
                    h.send(f"scoreboard players set {player} {metadata['progress']} {value}")
                    h.invoke("start_hud", player)
                h.invoke("start_hud")
                h.expect_count("HUD_STARTED", 2)
                for actor, x in [("a", 2), ("b", 6)]:
                    h.send(f"data modify storage presentation:proof {actor} set from entity @e[tag={tag},x={x + 0.5},y=200,z=2.5,distance=..0.1,limit=1] data")
                h.inspect("a", expected=25)
                h.inspect("b", expected=75)
                h.send(f"execute store result score #entries hud_proof run data get storage {ledger} active")
                h.check("if score #entries hud_proof matches 2", "compact_two_live_entries")
                h.send(f"scoreboard players set SandAlpha {metadata['progress']} 150")
                h.advance(1)
                h.inspect("a", expected=100)
                h.inspect("b", expected=75)
                # Imported namespaces each issue Lifetime instance 2 first.
                # Their actionbar identities must still be distinct pack claims.
                h.send(f"scoreboard players operation #beta_lease hud_proof = SandBeta {lease}")
                h.invoke("start_foreign_first")
                h.send(f"scoreboard players operation #first_lease hud_proof = SandAlpha {lease}")
                h.invoke("start_foreign_second")
                h.send(f"scoreboard players operation #second_lease hud_proof = SandAlpha {lease}")
                h.check("unless score #first_lease hud_proof = #second_lease hud_proof", "foreign_claims_have_distinct_tokens")
                for definition, tag_value in foreign_tags.items():
                    h.check(f"if entity @e[tag={tag_value},nbt={{data:{{instance:2}}}}]", f"{definition}_lifetime_token_collision")
                    h.send(f"scoreboard players set #{definition} hud_proof 0")
                h.advance(1)
                h.check("if score #foreign_first hud_proof matches 0", "superseded_foreign_hud_does_not_refresh")
                h.check("if score #foreign_second hud_proof matches 1..", "newest_foreign_hud_refreshes")
                h.invoke("cancel_foreign_first")
                h.check(f"if score SandAlpha {lease} = #second_lease hud_proof", "foreign_old_cleanup_preserves_newer_claim")
                h.check(f"if score SandBeta {lease} = #beta_lease hud_proof", "foreign_claim_preserves_other_player")
                h.invoke("cancel_foreign_second")
                h.check(f"unless score SandAlpha {lease} matches 0..", "foreign_winner_releases_slot")
                h.send(f"scoreboard players operation #saved_next hud_proof = #next {lease}")
                h.send(f"scoreboard players set #next {lease} 2147483647")
                h.invoke("start_foreign_first")
                h.expect_count("HUD_FOREIGN_FIRST", 1)
                h.check(f"unless entity @e[tag={foreign_tags['foreign_first']}]", "exhausted_pack_claim_rejected_before_start")
                h.send(f"scoreboard players operation #next {lease} = #saved_next hud_proof")
                # A different HUD definition supersedes Alpha's actionbar lease.
                h.invoke("start_other")
                h.send(f"scoreboard players operation #lease hud_proof = SandAlpha {lease}")
                h.invoke("cancel_hud")
                h.check(f"if score SandAlpha {lease} = #lease hud_proof", "stale_cleanup_preserves_newer_actionbar")
                h.inspect("a", exists=False)
                h.inspect("b", expected=75)
                h.invoke("cancel_other")
                h.check(f"unless score SandAlpha {lease} matches 0..", "winner_releases_actionbar")
                h.invoke("start_other")
                h.advance(20)
                h.check(f"unless score SandAlpha {lease} matches 0..", "expiry_releases_actionbar")
                h.check("unless data storage presentation:__sand_system/lifetime/other/hud_ledger active[0]", "expiry_reclaims_native_bar_entry")
            # Native bar removal must work after Beta is offline.
            h.advance(1)
            h.inspect("b", exists=False)
            h.check(f"unless data storage {ledger} active[0]", "offline_cleanup_reclaims_ledger")
            with h.clients(players=("SandBeta",)):
                h.send("gamemode creative SandBeta")
                h.send("tp SandBeta 6 200 2")
                h.advance(1)
                h.send(f"scoreboard players set SandBeta {metadata['progress']} 35")
                h.invoke("start_hud", "SandBeta")
                h.send(f"data modify storage presentation:proof b set from entity @e[tag={tag},limit=1] data")
                h.inspect("b", expected=35)
                h.send("forceload add 1024 0")
                h.wait_loaded("1026 200 2")
                h.send(f"tp @e[tag={tag}] 1026 200 2")
                h.send("forceload remove 1024 0")
                for _ in range(40):
                    h.advance(20)
                    if h.matches("unless loaded 1026 200 2", "hud_chunk_unloaded"):
                        break
                else:
                    raise ValidationFailure("unload", "remote HUD root chunk stayed loaded")
                # The root is unavailable; load cleanup must use the compact ledger.
                h.send(f"scoreboard players operation #before_reload hud_proof = #next {lease}")
                offset = len(h.lines)
                h.send("reload")
                h.send(f"say {RELOAD_COMPLETE}")
                h.wait_for(RELOAD_COMPLETE, "reload", offset)
                h.barrier("reloaded")
                # Vanilla schedules minecraft:load on the next server tick;
                # the world remains deliberately frozen during the probe.
                h.advance(1)
                h.inspect("b", exists=False)
                h.check(f"unless data storage {ledger} active", "reload_reclaims_unloaded_bar_ledger")
                h.check(f"if score #next {lease} = #before_reload hud_proof", "reload_preserves_pack_claim_sequence")
                h.invoke("start_hud", "SandBeta")
                h.send(f"scoreboard players operation #new_lease hud_proof = SandBeta {lease}")
                h.check(f"if score #new_lease hud_proof > #before_reload hud_proof", "post_reload_claim_never_reuses_old_token")
                h.send("forceload add 1024 0")
                h.wait_loaded("1026 200 2")
                h.advance(1)
                h.check(f"if score SandBeta {lease} = #new_lease hud_proof", "unloaded_old_cleanup_preserves_post_reload_claim")
                h.invoke("cancel_hud", "SandBeta")
        if h.errors(0, len(h.lines)):
            raise ValidationFailure("runtime", "server reported errors", h.errors(0, len(h.lines)))
        (root / "evidence.json").write_text(json.dumps({"minecraft":args.version, "protocol":profile_for_version(args.version)["version"]["protocol_version"], "passed":True, "checks":["two real player bossbar values and single viewers", "capacity rejection before presentation", "owner State progress and clamping", "literal macro text retention", "cross-definition actionbar lease safety", "cross-namespace supersession despite equal lifetime instance IDs", "stale foreign refresh and cleanup suppression", "pack lease exhaustion rejects starts", "owner-isolated cancellation", "expiry removes presentation", "offline bossbar removal", "compact live ledger reclamation", "reload removes unloaded root bossbars", "reload preserves claim sequence and delayed old cleanup cannot clear new claim"], "limits":["server state and emitted actionbar-body entry evidence; no client rendering or actionbar packet content assertion"], "log":str(h.log_path)}, indent=2)+"\n")
    finally:
        h.stop()


if __name__ == "__main__":
    main()
