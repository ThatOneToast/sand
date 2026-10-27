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
import subprocess
import time
from pathlib import Path

from player_validation_harness import PlayerHarness
from vanilla_reload_harness import RELOAD_START, ValidationFailure


ROOT = Path(__file__).resolve().parents[1]
RELOAD_MARKER = '__TRAIL_AUDIT_LOAD__'


def pack_digest(pack):
    """Hash relative paths and bytes deterministically, independent of checkout."""
    digest = hashlib.sha256()
    for path in sorted(path for path in Path(pack).rglob('*') if path.is_file()):
        digest.update(path.relative_to(pack).as_posix().encode() + b'\0')
        digest.update(path.read_bytes() + b'\0')
    return digest.hexdigest()


def unique_objective(functions, operation, value):
    objectives = set(re.findall(
        rf'scoreboard players {operation} @s ([\w.]+) {value}(?:\n|$)', functions))
    if len(objectives) != 1:
        raise ValidationFailure('fixture', f'expected one objective for {operation} {value}: {objectives}')
    return objectives.pop()


class TrailforgeHarness(PlayerHarness):
    def __init__(self, args):
        super().__init__(args)
        self.checks = []
        self.dispatches = 0
        self.harness_sha256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        self.harness_dirty = bool(subprocess.check_output(
            ['git', 'status', '--porcelain', '--', 'scripts/validate_trailforge.py'], cwd=ROOT, text=True).strip())
        self.java_version = subprocess.run([args.java, '-version'], text=True, capture_output=True, check=True).stderr.strip()
        self.server_jar_sha256 = hashlib.sha256(Path(args.jar).read_bytes()).hexdigest()
        pack = Path(args.pack)
        self.pack_sha256 = pack_digest(pack)
        source_root = Path(args.source_root).resolve() if args.source_root else pack.resolve()
        self.source_root = str(source_root)
        self.source_tree = subprocess.check_output(
            ['git', 'rev-parse', 'HEAD^{tree}'], cwd=source_root, text=True).strip()
        self.source_manifest = {}
        for relative in subprocess.check_output(
                ['git', 'ls-files', 'examples/book_project', 'Cargo.toml', 'Cargo.lock'],
                cwd=source_root, text=True).splitlines():
            path = source_root / relative
            if path.is_file():
                self.source_manifest[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
        self.revision = subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], cwd=source_root, text=True).strip()
        self.worktree_dirty = bool(subprocess.check_output(
            ['git', 'status', '--porcelain'], cwd=source_root, text=True).strip())
        self.harness_revision = subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()

    def load_fixture(self):
        pack = Path(self.args.pack)
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
        self.cooldown = unique_objective(functions, 'set', 80)
        self.regen = unique_objective(functions, 'set', 40)
        recover = (pack / 'data/trail/function/recover.mcfunction').read_text()
        self.exhausted = unique_objective(recover, 'set', 0)
        boots = set(re.findall(r'give @s (minecraft:leather_boots\[.*\])(?: 1)?(?:\n|$)', functions))
        if len(boots) != 1:
            raise ValidationFailure('fixture', 'expected one canonical Trail Striders reward')
        self.boots = boots.pop()
        dialog = json.loads((pack / 'data/trail/dialog/trailhead.json').read_text())
        self.menu_commands = [button['action']['command'].removeprefix('/')
                              for button in dialog['actions']]
        if len(self.menu_commands) != 2 or any(
                not re.fullmatch(r'trigger [\w.]+ set [1-9][0-9]*', command)
                for command in self.menu_commands):
            raise ValidationFailure('dialog', 'menu buttons must use survival-safe triggers')

    def prepare(self):
        super().prepare()
        # A separate observation-only pack leaves the exported gameplay intact.
        # Its load marker proves the real load tag ran after asynchronous reload.
        probe = self.server / 'world/datapacks/trail_audit_probe'
        (probe / 'data/minecraft/tags/function').mkdir(parents=True)
        (probe / 'data/trail_audit/function').mkdir(parents=True)
        (probe / 'pack.mcmeta').write_text((Path(self.args.pack) / 'pack.mcmeta').read_text())
        (probe / 'data/minecraft/tags/function/load.json').write_text(
            json.dumps({'values': ['trail_audit:loaded']}))
        (probe / 'data/trail_audit/function/loaded.mcfunction').write_text(
            f'say {RELOAD_MARKER}\n')

    def step(self, ticks, label):
        """Advance real ticks and await their completion, not command submission."""
        if type(ticks) is not int or ticks <= 0:
            raise ValueError('ticks must be a positive integer')
        self.send('execute store result score #tick_end trail_audit run time query gametime')
        self.send(f'scoreboard players add #tick_end trail_audit {ticks}')
        self.send(f'tick step {ticks}')
        deadline = time.monotonic() + self.args.timeout
        while time.monotonic() < deadline:
            self.send('execute store result score #tick_now trail_audit run time query gametime')
            if self.passed(label, f'execute if score #tick_now trail_audit = #tick_end trail_audit run say __SAND_WORLD_{label}_OK__'):
                return
            time.sleep(0.05)
        raise ValidationFailure(label, 'stepped ticks did not finish')

    def reload_pack(self):
        """Observe a fresh load tag and finish its tick before inspecting state."""
        start = len(self.lines)
        self.send('reload')
        self.wait_for(RELOAD_START, 'trail-reload', start)
        deadline = time.monotonic() + self.args.timeout
        stepped = 0
        while time.monotonic() < deadline:
            self.step(1, 'RELOAD_TICK')
            stepped += 1
            if any(self.marker_pattern(RELOAD_MARKER).search(line) for line in self.lines[start:]):
                return stepped
        raise ValidationFailure('trail-reload', 'actual load tag never ran')

    def score(self, name, player, objective, expected):
        self.check(name, f'if score {player} {objective} matches {expected}')

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append(name)

    def count(self, name, player, item, expected):
        self.send(f'execute store result score #count trail_audit run clear {player} {item} 0')
        self.check(name, f'if score #count trail_audit matches {expected}')

    def menu_many(self, requests):
        """Queue player triggers then dispatch all in one frozen game tick."""
        self.send('function trail:__sand_dialog_tick')
        self.send('execute store result score #before trail_audit run time query gametime')
        for player, index in requests:
            self.send(f'execute as {player} at @s run {self.menu_commands[index]}')
        self.send('function trail:__sand_dialog_tick')
        self.send('execute store result score #after trail_audit run time query gametime')
        self.dispatches += 1
        self.check(f'DISPATCH_{self.dispatches}_SAME_TICK', 'if score #before trail_audit = #after trail_audit')

    def menu(self, player, index):
        self.menu_many(((player, index),))

    def claim(self, player):
        self.menu(player, 1)

    def equip(self, player, item=None):
        self.send(f'item replace entity {player} armor.feet with {item or self.boots}')

    def reset_actor(self, player, stamina=100, cooldown=0):
        for objective, value in ((self.stamina, stamina), (self.cooldown, cooldown),
                                 (self.regen, 40), (self.exhausted, 0)):
            self.send(f'scoreboard players set {player} {objective} {value}')
        self.send(f'effect clear {player}')
        self.equip(player)

    def lane(self, player, block='gold_block', distance=3, wall=False):
        """Horizontal eye-height lane, isolated per player within loaded chunk."""
        x = 2 if player == 'SandAlpha' else 10
        self.send(f'fill {x} 80 0 {x} 83 12 minecraft:air')
        self.send(f'setblock {x} 79 0 minecraft:stone')
        self.send(f'tp {player} {x + 0.5} 80 0.5 0 0')
        if block:
            self.send(f'setblock {x} 81 {distance} minecraft:{block}')
        if wall:
            self.send(f'setblock {x} 81 1 minecraft:stone')

    def activate(self, player, ability='grapple'):
        if ability == 'grapple':
            self.menu(player, 0)
        else:
            self.send(f'execute as {player} at @s run function trail:{ability}')

    def state(self, label, player, stamina, cooldown):
        self.score(label + '_STAMINA', player, self.stamina, stamina)
        self.score(label + '_COOLDOWN', player, self.cooldown, cooldown)

    def no_effect(self, label, player):
        self.check(label + '_NO_SPEED',
                   f'unless data entity {player} active_effects[{{id:"minecraft:speed"}}]')

    def gameplay(self):
        a, b = self.players
        for player in self.players:
            self.reset_actor(player)
            self.lane(player)
        self.activate(a)
        self.state('SUCCESS', a, 70, 80)
        self.send(f'data get entity {a} active_effects')
        self.check('SUCCESS_EFFECT', f'if data entity {a} active_effects[{{id:"minecraft:speed",amplifier:2b}}]')
        self.state('OTHER_PLAYER', b, 100, 0)
        self.no_effect('OTHER_PLAYER', b)
        self.activate(a)
        self.state('ACTIVE_COOLDOWN', a, 70, 80)

        for label, stamina, item in (
                ('INSUFFICIENT', 29, None), ('UNEQUIPPED', 100, 'minecraft:air'),
                ('PLAIN_BOOTS', 100, 'minecraft:leather_boots')):
            self.reset_actor(a, stamina)
            if item:
                self.equip(a, item)
            self.activate(a)
            self.state(label, a, stamina, 0)
            self.no_effect(label, a)
        self.reset_actor(a)
        self.send(f'scoreboard players set {a} {self.exhausted} 1')
        self.activate(a)
        self.state('EXHAUSTED_POLICY', a, 100, 0)
        self.no_effect('EXHAUSTED_POLICY', a)

        for label, block, distance, wall in (
                ('MISS', None, 3, False), ('INVALID_ANCHOR', 'iron_block', 3, False),
                ('OBSTRUCTED', 'gold_block', 3, True), ('OUT_OF_RANGE', 'gold_block', 9, False)):
            self.reset_actor(a)
            self.lane(a, block, distance, wall)
            self.activate(a)
            self.state(label, a, 100, 0)
            self.no_effect(label, a)
        self.reset_actor(a)
        self.lane(a, distance=8)
        self.activate(a)
        self.state('LAST_BLOCK_IN_RANGE', a, 70, 80)

        # The invoked entry must anchor at the player, not the external context.
        self.load_test_chunk('minecraft:the_nether')
        self.reset_actor(a)
        self.lane(a, None)
        self.send('execute in minecraft:the_nether run setblock 2 81 3 minecraft:gold_block')
        self.send(f'execute as {a} in minecraft:the_nether positioned 2.5 80 0.5 rotated 0 0 run function trail:grapple')
        self.state('WRONG_DIMENSION_ANCHOR', a, 100, 0)
        self.no_effect('WRONG_DIMENSION_ANCHOR', a)
        self.lane(a)
        self.send(f'execute as {a} in minecraft:the_nether positioned 100 100 100 run function trail:grapple')
        self.state('EXTERNAL_CONTEXT_REANCHORED', a, 70, 80)

        for player in self.players:
            self.reset_actor(player)
            self.lane(player)
        self.menu_many(((a, 0), (b, 0)))
        for player in self.players:
            self.state('SAME_TICK_' + player, player, 70, 80)
            self.check('SAME_TICK_EFFECT_' + player,
                       f'if data entity {player} active_effects[{{id:"minecraft:speed",amplifier:2b}}]')

        self.reset_actor(a)
        self.lane(a, 'iron_block')
        self.activate(a, 'scout')
        self.state('SCOUT', a, 90, 20)
        self.check('SCOUT_EFFECT', f'if data entity {a} active_effects[{{id:"minecraft:speed"}}] unless data entity {a} active_effects[{{id:"minecraft:speed"}}].amplifier')
        self.lane(a)
        self.activate(a)
        self.state('SHARED_COOLDOWN', a, 90, 20)
        self.reset_actor(a)
        self.lane(a, 'iron_block', 5)
        self.activate(a, 'scout')
        self.state('SCOUT_SHORT_RANGE', a, 100, 0)
        self.no_effect('SCOUT_SHORT_RANGE', a)

        self.reset_actor(a)
        self.lane(a, None)
        self.send(f'scoreboard players reset {a} {self.stamina}')
        self.activate(a)
        self.state('MISSING_STATE_DEFAULT', a, 100, 0)

        self.reset_actor(a, 30)
        self.lane(a)
        self.activate(a)
        self.state('EXACT_COST_LOWER_BOUND', a, 0, 80)
        self.activate(a)
        self.state('NO_UNDERFLOW', a, 0, 80)
        self.reset_actor(a, 95)
        self.equip(a, 'minecraft:air')
        self.step(39, 'REGEN_BEFORE')
        self.score('NO_EARLY_REGEN', a, self.stamina, 95)
        self.step(1, 'REGEN_BOUNDARY')
        self.score('UNEQUIPPED_REGEN_CLAMP', a, self.stamina, 100)
        self.step(40, 'REGEN_FULL')
        self.score('FULL_CAP_REGEN', a, self.stamina, 100)
        self.reset_actor(a, 60, 80)
        self.step(39, 'CADENCE_BEFORE')
        self.state('CADENCE_BEFORE', a, 60, 41)
        self.step(1, 'CADENCE_FIRST')
        self.state('CADENCE_FIRST', a, 70, 40)
        self.step(39, 'CADENCE_BETWEEN')
        self.state('CADENCE_BETWEEN', a, 70, 1)
        self.step(1, 'CADENCE_SECOND')
        self.state('CADENCE_SECOND', a, 80, 0)
        self.lane(a)
        self.activate(a)
        self.state('COOLDOWN_EXPIRES', a, 50, 80)

        self.reset_actor(a, 61, 70)
        self.reset_actor(b, 83, 50)
        self.reload_ticks = self.reload_pack()
        self.score('RELOAD_PRESERVES_STAMINA', a, self.stamina, 61)
        self.score('RELOAD_PRESERVES_OTHER_STAMINA', b, self.stamina, 83)
        self.score('RELOAD_RETAINS_COOLDOWN', a, self.cooldown, 70 - self.reload_ticks)
        self.score('RELOAD_RETAINS_OTHER_COOLDOWN', b, self.cooldown, 50 - self.reload_ticks)
        self.check('RELOAD_RETAINS_EQUIPMENT', f'if items entity {a} armor.feet minecraft:leather_boots[minecraft:custom_data~{{trail_striders:1b}}]')

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
                self.step(2, 'FIRST_JOIN_SETTLED')
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
                self.gameplay()
                self.claim('SandAlpha')
                self.count('RELOAD_PRESERVES_UNLOCK', 'SandAlpha', core, 1)
            # Actual disconnect/rejoin; native ticks dispatch lifecycle hooks.
            with self.clients():
                self.step(2, 'RECONNECT_SETTLED')
                self.score('RECONNECT_PRESERVES_STAMINA', 'SandAlpha', self.stamina, 61)
                self.score('RECONNECT_PRESERVES_OTHER_STAMINA', 'SandBeta', self.stamina, 83)
                self.score('RECONNECT_PRESERVES_COOLDOWN', 'SandAlpha', self.cooldown, 68 - self.reload_ticks)
                self.score('RECONNECT_PRESERVES_OTHER_COOLDOWN', 'SandBeta', self.cooldown, 48 - self.reload_ticks)
                self.check('RECONNECT_RETAINS_EQUIPMENT', 'if items entity SandAlpha armor.feet minecraft:leather_boots[minecraft:custom_data~{trail_striders:1b}]')
                self.claim('SandAlpha')
                self.count('RECONNECT_PRESERVES_UNLOCK', 'SandAlpha', core, 1)
        finally:
            self.send('tick unfreeze')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True)
    parser.add_argument('--jar', required=True)
    parser.add_argument('--source-root', help='Checkout used to build pack; required for packs outside a checkout')
    parser.add_argument('--java', default='java')
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--output')
    args = parser.parse_args(argv)
    args.version, args.op_player, args.client_command = '26.2', None, None
    harness = TrailforgeHarness(args)
    passed = False
    failure = None
    try:
        harness.load_fixture()
        harness.run()
        passed = True
    except ValidationFailure as error:
        failure = {'phase': error.phase, 'message': str(error), 'matches': error.matches}
        print(f'FAILED: {error.phase}: {error}; log: {harness.log_path}', file=sys.stderr)
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        (harness.output / 'evidence.json').write_text(json.dumps({
            'minecraft': '26.2', 'passed': passed, 'checks': harness.checks,
            'pack_sha256': harness.pack_sha256,
            'source_root': harness.source_root, 'source_tree': harness.source_tree,
            'source_manifest': harness.source_manifest,
            'source_revision': harness.revision, 'worktree_dirty': harness.worktree_dirty,
            'harness_revision': harness.harness_revision,
            'harness_sha256': harness.harness_sha256, 'harness_dirty': harness.harness_dirty,
            'java_version': harness.java_version, 'server_jar_sha256': harness.server_jar_sha256,
            'failure': failure,
            'observer': 'Separate observation-only load-tag marker pack; wait for fresh marker and completion of the stepped tick containing load-tag execution before state assertions.',
            'scope': 'Two real passive players; recipe-result items granted by commands; exported grapple menu trigger and scout function invoked from console. Frozen assertions with explicit real tick stepping for lifecycle, regeneration and cooldown; actual reload and reconnect. Separate observation-only load-marker pack. Does not prove client crafting/dialog input, death/respawn, unloaded/out-of-world rays or transactional effect success.',
        }, indent=2) + '\n')
    print(f'{"PASSED" if passed else "FAILED"}: Trailforge gameplay; {harness.log_path}')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
