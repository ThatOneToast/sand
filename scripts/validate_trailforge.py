#!/usr/bin/env python3
"""Verify Trailforge's upgrade and ability with two real passive players.

Uses the CLI-built pack unchanged. Item inputs come from the actual recipe JSON;
this proves result identity and gameplay consumption, not client crafting input.
Automatic ticks are frozen during assertions to isolate authored transitions.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import RELOAD_START, ValidationFailure


class TrailforgeHarness(PlayerHarness):
    def __init__(self, args):
        super().__init__(args)
        self.checks = []
        pack = Path(args.pack)
        digest = hashlib.sha256()
        for path in sorted(path for path in pack.rglob('*') if path.is_file()):
            digest.update(path.relative_to(pack).as_posix().encode() + b'\0')
            digest.update(path.read_bytes() + b'\0')
        self.pack_sha256 = digest.hexdigest()
        recipe = json.loads((pack / 'data/trail/recipe/grapple_core.json').read_text())
        result = recipe['result']
        if result['id'] != 'minecraft:heart_of_the_sea' or result.get('count', 1) != 1:
            raise ValidationFailure('recipe', 'expected one Grapple Core')
        if result.get('components', {}).get('minecraft:custom_data', {}).get('grapple_core') is not True:
            raise ValidationFailure('recipe', 'recipe result lacks the Grapple Core identity')
        components = ','.join(f'{key}={json.dumps(value, separators=(",", ":"))}'
                              for key, value in result['components'].items())
        self.core = f'{result["id"]}[{components}]'
        functions = '\n'.join(path.read_text() for path in (pack / 'data/trail/function').rglob('*.mcfunction'))
        stamina = set(re.findall(r'scoreboard players remove @s (\w+) 30(?:\n|$)', functions))
        if len(stamina) != 1:
            raise ValidationFailure('fixture', 'expected one authored stamina cost objective')
        self.stamina = stamina.pop()

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append(name)

    def count(self, name, player, item, expected):
        self.send(f'execute store result score #count trail_audit run clear {player} {item} 0')
        self.check(name, f'if score #count trail_audit matches {expected}')

    def claim(self, player):
        self.send(f'execute as {player} at @s run function trail:claim_striders')

    def run_client(self):
        if not any(re.search(r'Starting minecraft server version 26\.2$', line.strip()) for line in self.lines):
            raise ValidationFailure('version', 'audit requires an observed Minecraft 26.2 server')
        self.load_test_chunk('minecraft:overworld')
        self.send('tick freeze')
        try:
            with self.clients():
                self.send('scoreboard objectives add trail_audit dummy')
                for player in self.players:
                    self.send(f'gamemode creative {player}')
                    self.send(f'tp {player} 0 80 0')
                core = 'minecraft:heart_of_the_sea[minecraft:custom_data~{grapple_core:1b}]'
                boots = 'minecraft:leather_boots[minecraft:custom_data~{trail_striders:1b}]'
                self.send('give SandAlpha minecraft:heart_of_the_sea 2')
                self.send(f'give SandBeta {self.core} 2')
                self.claim('SandAlpha')
                self.count('PLAIN_ITEM_REJECTED', 'SandAlpha', boots, 0)
                self.count('PLAIN_ITEM_PRESERVED', 'SandAlpha', 'minecraft:heart_of_the_sea', 2)
                self.count('OTHER_PLAYER_COST_UNTOUCHED', 'SandBeta', core, 2)
                self.send(f'give SandAlpha {self.core} 2')
                self.claim('SandAlpha')
                self.count('ONE_CORE_CONSUMED', 'SandAlpha', core, 1)
                self.count('ONE_REWARD_GRANTED', 'SandAlpha', boots, 1)
                self.count('BETA_STILL_HAS_TWO_CORES', 'SandBeta', core, 2)
                self.count('BETA_HAS_NO_REWARD', 'SandBeta', boots, 0)
                self.claim('SandAlpha')
                self.count('REPEAT_COST_UNCHANGED', 'SandAlpha', core, 1)
                self.count('REPEAT_REWARD_UNCHANGED', 'SandAlpha', boots, 1)
                self.claim('SandBeta')
                self.count('BETA_INDEPENDENT_UPGRADE', 'SandBeta', boots, 1)
                self.count('BETA_INDEPENDENT_COST', 'SandBeta', core, 1)
                for player in self.players:
                    self.check(f'{player}_INITIAL_STAMINA', f'if score {player} {self.stamina} matches 100')
                self.send('execute as SandAlpha at @s run function trail:grapple')
                self.check('ALPHA_ABILITY_COST', f'if score SandAlpha {self.stamina} matches 70')
                self.check('BETA_ABILITY_ISOLATION', f'if score SandBeta {self.stamina} matches 100')
                self.send('execute as SandAlpha at @s run function trail:grapple')
                self.check('COOLDOWN_PREVENTS_REPEAT_COST', f'if score SandAlpha {self.stamina} matches 70')
                start = len(self.lines)
                self.send('reload')
                self.wait_for(RELOAD_START, 'trail-reload', start)
                self.send('say __TRAIL_RELOAD_DONE__')
                self.wait_for(self.marker_pattern('__TRAIL_RELOAD_DONE__'), 'trail-reload-complete', start)
                self.claim('SandAlpha')
                self.count('RELOAD_PRESERVES_UNLOCK', 'SandAlpha', boots, 1)
                self.count('RELOAD_PRESERVES_COST', 'SandAlpha', core, 1)
                self.check('RELOAD_PRESERVES_STAMINA', f'if score SandAlpha {self.stamina} matches 70')
        finally:
            self.send('tick unfreeze')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True)
    parser.add_argument('--jar', required=True)
    parser.add_argument('--java', default='java')
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--output')
    args = parser.parse_args()
    args.version, args.op_player, args.client_command = '26.2', None, None
    harness = TrailforgeHarness(args)
    passed = False
    try:
        harness.run()
        passed = True
    except ValidationFailure as error:
        print(f'FAILED: {error.phase}: {error}; log: {harness.log_path}', file=sys.stderr)
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        (harness.output / 'evidence.json').write_text(json.dumps({
            'minecraft': '26.2', 'passed': passed, 'checks': harness.checks,
            'pack_sha256': harness.pack_sha256,
            'scope': 'Two real passive players; recipe-result items granted by commands; explicit gameplay calls with automatic ticks frozen. Does not prove client crafting input.',
        }, indent=2) + '\n')
    print(f'{"PASSED" if passed else "FAILED"}: Trailforge gameplay; {harness.log_path}')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
