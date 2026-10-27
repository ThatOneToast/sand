#!/usr/bin/env python3
"""Validate Trailforge player initialization semantics and native evaluations.

Supply an unchanged base Trailforge pack and a JSON provenance record containing
its pack_sha256. Run once per before/after pack; no generated gameplay is edited.
The observer pack contributes two explicitly excluded trace-wrapper evaluations.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

from measure_archetype_idle import TRACE_FILE, trace_counts, verified_runtime_version
from player_validation_harness import PlayerHarness
from validate_trailforge import pack_digest
from vanilla_reload_harness import ValidationFailure

# Contract for the unchanged book_project fixture at 6f77dcb6. A differently
# authored fixture needs its own contract; never infer state from hash spelling.
FIELDS = {
    'stamina': 'sfa49ff66aebe47c', 'grapple': 'scf21a43c2b92f8a',
    'has_striders': 's9b03f7001255338', 'exhausted': 'saafb19392893dee',
    'regen': 's8520e22d64708b4', 'presence': 'sa9c5ae898ac8f8e',
    'suppression': 's5d6868466971a59',
}
INITIALIZER = 'trail:__sand_lifecycle_init'
WRAPPER = 'init_audit:probe'
COMPLETE = '__INIT_TRACE_COMPLETE__'


def completed_trace(content):
    """Reject trace errors, truncated output, or unexpected observer overhead."""
    counts = trace_counts(content)
    if counts.get(WRAPPER) != 2:
        raise ValueError('expected exactly two observer wrapper evaluations')
    # A command merely mentioning the sentinel does not prove it completed.
    if not re.search(r'^\s*\[C\] say ' + COMPLETE + r' -> 1$', content, re.MULTILINE):
        raise ValueError('trace lacks successful completion command')
    return counts


def trace_filename(lines):
    names = [match[1] for line in lines if (match := TRACE_FILE.search(line))]
    if len(names) != 1:
        raise ValueError(f'expected exactly one completed trace file, got {names}')
    return names[0]


class InitializationHarness(PlayerHarness):
    def __init__(self, args):
        super().__init__(args)
        self.checks = []
        self.samples = []
        self.provenance = {}

    def prepare(self):
        self.provenance = json.loads(Path(self.args.provenance).read_text())
        digest = pack_digest(Path(self.args.pack))
        if self.provenance.get('pack_sha256') != digest:
            raise ValidationFailure('provenance', 'pack does not match provenance digest')
        self.provenance = {
            **self.provenance,
            'harness_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            'server_jar_sha256': hashlib.sha256(Path(self.args.jar).read_bytes()).hexdigest(),
            'java_version': subprocess.run([self.args.java, '-version'], check=True,
                                           text=True, capture_output=True).stderr.strip(),
        }
        functions = '\n'.join(path.read_text() for path in
                              (Path(self.args.pack) / 'data/trail/function').rglob('*.mcfunction'))
        if not all(objective in functions for objective in FIELDS.values()):
            raise ValidationFailure('fixture', 'pack does not implement the base Trailforge State contract')
        super().prepare()
        root = self.server / 'world/datapacks/init_audit'
        (root / 'data/init_audit/function').mkdir(parents=True)
        (root / 'pack.mcmeta').write_text((Path(self.args.pack) / 'pack.mcmeta').read_text())
        (root / 'data/init_audit/function/probe.mcfunction').write_text(
            f'execute as @a at @s run function {INITIALIZER}\nsay {COMPLETE}\n')

    def check(self, name, condition):
        super().check(name, condition)
        self.checks.append({'name': name, 'condition': condition})

    def initialize(self, player):
        self.send(f'execute as {player} at @s run function {INITIALIZER}')

    def field(self, name, player, field, expected):
        self.check(name, f'if score {player} {FIELDS[field]} matches {expected}')

    def trace(self, label, population):
        self.send('scoreboard players set #players init_audit 0')
        self.send('execute as @a run scoreboard players add #players init_audit 1')
        self.check(label + '_population', f'if score #players init_audit matches {population}')
        start = len(self.lines)
        self.send(f'debug function {WRAPPER}')
        self.send('say __INIT_TRACE_DONE__')
        self.wait_for(self.marker_pattern('__INIT_TRACE_DONE__'), 'trace', start)
        name = trace_filename(self.lines[start:])
        content = (self.server / 'debug' / name).read_text()
        counts = completed_trace(content)
        destination = f'{label}.trace.txt'
        (self.output / destination).write_text(content)
        self.samples.append({
            'label': label, 'players': population,
            'function_evaluations': counts,
            'initializer_evaluations': sum(counts.values()) - 2,
            'probe_wrapper_evaluations': 2, 'trace': destination,
            'trace_sha256': hashlib.sha256(content.encode()).hexdigest(),
        })

    def boundary(self, label):
        # Ordinary functions without explicit return do not publish a return
        # callback. Sentinel preservation differs from result=0/success=0 or 1.
        for key in ('result', 'success'):
            self.send(f'scoreboard players set #{key} init_audit -99')
        self.send('execute as SandAlpha at @s store result score #result init_audit '
                  f'store success score #success init_audit run function {INITIALIZER}')
        for key in ('result', 'success'):
            self.check(label + '_' + key, f'if score #{key} init_audit matches -99')

    def boundaries(self):
        a, b = self.players
        for objective in FIELDS.values():
            self.send(f'scoreboard players reset {a} {objective}')
        self.boundary('fresh')
        self.field('fresh_default', a, 'stamina', 100)
        self.send(f'scoreboard players set {a} {FIELDS["stamina"]} 61')
        self.boundary('current')
        self.field('current_preserved', a, 'stamina', 61)
        self.send(f'scoreboard players reset {a} {FIELDS["stamina"]}')
        self.boundary('missing')
        self.field('missing_repaired', a, 'stamina', 100)
        self.send(f'scoreboard players reset {a} {FIELDS["stamina"]}')
        self.send(f'scoreboard players set {a} {FIELDS["suppression"]} 1')
        self.boundary('suppressed')
        self.check('suppressed_absent', f'unless score {a} {FIELDS["stamina"]} matches -2147483648..')
        self.field('boundary_other_player_preserved', b, 'stamina', 61)

    def run_client(self):
        verified_runtime_version(self.lines)
        self.send('tick freeze')
        self.send('scoreboard objectives add init_audit dummy')
        try:
            self.trace('healthy_0', 0)
            with self.clients(players=('SandAlpha',)):
                self.initialize('SandAlpha')
                self.trace('healthy_1', 1)
                with self.clients(players=('SandBeta',)):
                    self.initialize('SandBeta')
                    self.trace('healthy_2', 2)
                    self.send(f'scoreboard players set SandBeta {FIELDS["stamina"]} 61')
                    self.send(f'scoreboard players reset SandAlpha {FIELDS["stamina"]}')
                    self.trace('missing_2', 2)
                    self.field('trace_missing_repaired', 'SandAlpha', 'stamina', 100)
                    self.field('trace_other_player_preserved', 'SandBeta', 'stamina', 61)
                    self.send(f'scoreboard players reset SandAlpha {FIELDS["stamina"]}')
                    self.send(f'scoreboard players set SandAlpha {FIELDS["suppression"]} 1')
                    self.trace('suppressed_2', 2)
                    self.check('trace_suppressed_absent',
                               f'unless score SandAlpha {FIELDS["stamina"]} matches -2147483648..')
                    self.boundaries()
        finally:
            self.send('tick unfreeze')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True)
    parser.add_argument('--provenance', required=True, help='JSON source description with matching pack_sha256')
    parser.add_argument('--jar', required=True)
    parser.add_argument('--java', default='java')
    parser.add_argument('--output', required=True, help='Disposable directory; replaced when server prepares')
    parser.add_argument('--timeout', type=float, default=120)
    args = parser.parse_args(argv)
    args.version, args.op_player, args.client_command = '26.2', None, None
    harness = InitializationHarness(args)
    passed, failure = False, None
    try:
        harness.run()
        passed = True
    except (ValidationFailure, ValueError, OSError, subprocess.CalledProcessError) as error:
        failure = {'phase': getattr(error, 'phase', 'fixture-or-trace'), 'message': str(error)}
        print(f'FAILED: {failure}; log: {harness.log_path}', file=sys.stderr)
    finally:
        harness.output.mkdir(parents=True, exist_ok=True)
        (harness.output / 'evidence.json').write_text(json.dumps({
            'schema_version': 1, 'minecraft': '26.2', 'passed': passed, 'failure': failure,
            'provenance': harness.provenance, 'checks': harness.checks, 'samples': harness.samples,
            'scope': 'Two real passive players, frozen game ticks. Initializer result/success callbacks, '
                     'missing-score repair, suppression and isolation. Completed debug [C] evaluations '
                     'exclude two observer commands; these are not mutation counts, command-budget '
                     'units or MSPT. Does not validate reload, reconnect or whole-tick scheduling.',
        }, indent=2) + '\n')
    print(f'{"PASSED" if passed else "FAILED"}: player initialization; {harness.log_path}')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
