#!/usr/bin/env python3
"""Native proof for the sand-only aether_tools composition consumer.

Reuses the maintained real-player harness, tick-completion accounting and actual
reload observer from Trailforge. Gameplay pack bytes are never modified.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

from validate_trailforge import TrailforgeHarness, unique_objective
from vanilla_reload_harness import ValidationFailure


class AbilityHarness(TrailforgeHarness):
    def load_fixture(self):
        functions = '\n'.join(p.read_text() for p in
                              (Path(self.args.pack) / 'data/aether/function').rglob('*.mcfunction'))
        self.charge = unique_objective(functions, 'remove', 7)
        self.cooldown = unique_objective(functions, 'set', 12)
        self.regen = unique_objective(functions, 'set', 7)
        self.reserve = unique_objective(functions, 'remove', 5)
        self.reserve_cd = unique_objective(functions, 'set', 9)
        self.count = unique_objective(functions, 'add', 1)

    def activate(self, player, ability='pulse'):
        self.send(f'execute as {player} at @s run function aether:{ability}')

    def reset_actor(self, player, charge=48):
        for objective, value in ((self.charge, charge), (self.cooldown, 0),
                                 (self.regen, 7), (self.reserve, 21),
                                 (self.reserve_cd, 0), (self.count, 0)):
            self.send(f'scoreboard players set {player} {objective} {value}')
        self.activate(player, 'grant')

    def state(self, name, player, charge, cooldown, count):
        self.score(name + '_CHARGE', player, self.charge, charge)
        self.score(name + '_COOLDOWN', player, self.cooldown, cooldown)
        self.score(name + '_EFFECT', player, self.count, count)

    def run_client(self):
        self.load_test_chunk('minecraft:overworld')
        self.send('tick freeze')
        try:
            with self.clients():
                self.send('scoreboard objectives add trail_audit dummy')
                self.step(2, 'INITIALIZED')
                for player, x in zip(self.players, (2, 10)):
                    self.send(f'gamemode creative {player}')
                    self.send(f'tp {player} {x}.5 80 0.5')
                    self.send(f'setblock {x} 79 0 minecraft:diamond_block')
                    self.reset_actor(player)
                a, b = self.players
                self.reset_actor(a, 7)
                self.activate(a)
                self.state('EXACT_COST_COMPLETES', a, 0, 12, 1)
                self.activate(a, 'flare')
                self.state('SHARED_COOLDOWN_REJECTS', a, 0, 12, 1)
                self.score('OTHER_PLAYER_UNCHANGED', b, self.charge, 48)
                for label, item in [('MISSING', 'air'), ('WRONG', 'amethyst_shard')]:
                    self.reset_actor(a)
                    self.send(f'item replace entity {a} weapon.offhand with minecraft:{item}')
                    self.activate(a)
                    self.state(label, a, 48, 0, 0)
                self.activate(a, 'grant')
                self.activate(a)
                self.state('CHANGED_TO_CORRECT', a, 41, 12, 1)
                self.reset_actor(a, 6)
                self.activate(a)
                self.state('INSUFFICIENT', a, 6, 0, 0)
                self.reset_actor(a)
                self.send('setblock 2 79 0 minecraft:stone')
                self.activate(a)
                self.state('TARGET_REJECTED', a, 48, 0, 0)
                self.send('setblock 2 79 0 minecraft:diamond_block')
                for player in self.players:
                    self.reset_actor(player)
                self.send('execute store result score #before trail_audit run time query gametime')
                self.send('execute as @a at @s run function aether:pulse')
                self.send('execute store result score #after trail_audit run time query gametime')
                self.check('MULTIPLAYER_SAME_TICK', 'if score #before trail_audit = #after trail_audit')
                for player in self.players:
                    self.state('SAME_TICK_' + player, player, 41, 12, 1)
                self.activate(a, 'reserve')
                self.score('INDEPENDENT_POOL', a, self.reserve, 16)
                self.score('INDEPENDENT_COOLDOWN', a, self.reserve_cd, 9)
                self.state('PRIMARY_UNCHANGED', a, 41, 12, 2)
                self.activate(a, 'flare')
                self.state('SHARED_COOLDOWN_WITH_SUFFICIENT_BALANCE', a, 41, 12, 2)
                self.reset_actor(a, 30)
                self.step(6, 'BEFORE_CADENCE')
                self.score('NO_EARLY_REGEN', a, self.charge, 30)
                self.step(1, 'AT_CADENCE')
                self.score('SHARED_POOL_REGENERATES_ONCE', a, self.charge, 33)
                self.reset_actor(a, 47)
                self.step(7, 'CLAMP_CADENCE')
                self.score('STATE_CAPACITY_CLAMP', a, self.charge, 48)
                self.reset_actor(a)
                self.check('FAILURE_TARGET_UNLOADED', 'unless loaded 1000000 80 1000000')
                self.activate(a, 'failure')
                self.state('NATIVE_FAILURE_NO_ROLLBACK', a, 41, 12, 1)
                self.reset_actor(a)
                self.activate(a, 'early_return')
                self.state('CALLBACK_RETURN_NO_ROLLBACK', a, 41, 12, 0)
                self.reset_actor(a, 30)
                self.activate(a, 'flare')
                self.state('SECOND_COST_CONFIGURATION', a, 19, 12, 1)
                self.reload_ticks = self.reload_pack()
                self.score('RELOAD_COOLDOWN_RETAINED', a, self.cooldown, max(0, 12 - self.reload_ticks))
                # Keep the recharge deadline beyond the observed load sequence.
                expected = 19 + 3 * (self.reload_ticks // 7)
                self.score('RELOAD_RESOURCE_TICK_ACCOUNTED', a, self.charge, expected)
            with self.clients():
                self.step(2, 'RECONNECT_COMPLETE')
                ticks = self.reload_ticks + 2
                self.score('RECONNECT_RESOURCE_RETAINED', a, self.charge, 19 + 3 * (ticks // 7))
                self.score('RECONNECT_COOLDOWN_RETAINED', a, self.cooldown, max(0, 12 - ticks))
        finally:
            self.send('tick unfreeze')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('pack', 'jar', 'source-root', 'output'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--java', default='java')
    parser.add_argument('--timeout', type=float, default=120)
    args = parser.parse_args(argv)
    args.version, args.op_player, args.client_command = '26.2', None, None
    harness = AbilityHarness(args)
    passed, failure = False, None
    try:
        harness.load_fixture()
        harness.run()
        passed = True
    except ValidationFailure as error:
        failure = {'phase': error.phase, 'message': str(error), 'matches': error.matches}
        print(f'FAILED: {error}; log: {harness.log_path}', file=sys.stderr)
    finally:
        root = Path(args.source_root)
        manifest = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
                    for base in ('examples/aether_tools', 'sand/src/systems')
                    for p in (root / base).rglob('*.rs')}
        harness.output.mkdir(parents=True, exist_ok=True)
        (harness.output / 'evidence.json').write_text(json.dumps({
            'passed': passed, 'checks': harness.checks, 'failure': failure,
            'minecraft': '26.2', 'java_version': harness.java_version,
            'pack_sha256': harness.pack_sha256, 'server_jar_sha256': harness.server_jar_sha256,
            'revision': harness.revision, 'dirty': harness.worktree_dirty,
            'source_manifest': manifest,
            'harness_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            'scope': 'Two real passive clients; console activation; frozen assertions and completed native ticks; observed actual load tag and reconnect. No client input, death, MSPT or rollback claim.',
        }, indent=2) + '\n')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
