#!/usr/bin/env python3
"""Compare static evaluation estimates with completed Minecraft 26.2 traces.

The fixture deliberately uses the portable raw-command escape hatch to cover
Minecraft control flow. It is compiled by Sand; no generated function is edited.
The metric is debug [C] evaluations, not MSPT or the internal command budget.
"""
import argparse
import json
from pathlib import Path
import subprocess

from validate_world_gameplay import WorldGameplayHarness
from measure_archetype_idle import TRACE_FILE, trace_counts, verified_runtime_version

CASES = {
    'ordinary': ['scoreboard players add #result cost_probe 1'],
    'false_guard': ['execute if score #false cost_probe matches 1 run scoreboard players add #result cost_probe 1'],
    'true_guard': ['execute if score #true cost_probe matches 1 run scoreboard players add #result cost_probe 1'],
    'fork_command': ['execute as @e[tag=cost_subject] run scoreboard players add #result cost_probe 1'],
    'fork_function': ['execute as @e[tag=cost_subject] run function cost_semantics:leaf'],
    'double_call': ['function cost_semantics:leaf', 'function cost_semantics:leaf'],
    'early_return': ['return 1', 'scoreboard players add #result cost_probe 100'],
    'return_run': ['return run function cost_semantics:leaf', 'scoreboard players add #result cost_probe 100'],
    'function_condition': ['execute if function cost_semantics:truth run scoreboard players add #result cost_probe 1'],
    'nested_forks': ['execute as @e[tag=cost_subject] at @e[tag=cost_subject] run function cost_semantics:leaf'],
}

class Probe(WorldGameplayHarness):
    def run_client(self):
        """Verify populations and completed traces before comparing report ranges."""
        verified_runtime_version(self.lines)
        self.load_test_chunk('minecraft:overworld')
        self.send('tick freeze')
        self.send('scoreboard objectives add cost_probe dummy')
        self.send('scoreboard players set #false cost_probe 0')
        self.send('scoreboard players set #true cost_probe 1')
        for count in (0, 3):
            if count:
                for _ in range(count):
                    self.send('summon minecraft:armor_stand 0 80 0 {Tags:["cost_subject"],NoGravity:1b,Invulnerable:1b}')
            self.send('scoreboard players set #count cost_probe 0')
            self.send('execute as @e[tag=cost_subject] run scoreboard players add #count cost_probe 1')
            self.check(f'POPULATION_{count}', f'if score #count cost_probe matches {count}')
            for name in CASES:
                self.send('scoreboard players set #result cost_probe 0')
                start = len(self.lines)
                self.send(f'debug function cost_semantics:probe_{name}')
                done = f'__COST_TRACE_{count}_{name}_DONE__'
                self.send(f'say {done}')
                self.wait_for(self.marker_pattern(done), 'trace', start)
                names = [m[1] for line in self.lines[start:] if (m := TRACE_FILE.search(line))]
                assert len(names) == 1, names
                content = (self.server / 'debug' / names[0]).read_text()
                counts = trace_counts(content)
                assert counts[f'cost_semantics:probe_{name}'] == 2
                assert f'__COST_COMPLETE_{name}__' in content
                path = self.evidence_root / f'{count}-{name}.txt'
                path.write_text(content)
                observed = sum(counts.values()) - 2
                estimate = self.report['functions'][f'cost_semantics:{name}']['evaluations']
                bounds = {
                    'ordinary': (1, 1), 'false_guard': (1, 1), 'true_guard': (1, 1),
                    'fork_command': (1, 1), 'fork_function': (1, None),
                    'double_call': (4, 4), 'early_return': (1, 1), 'return_run': (2, 2),
                    'function_condition': (3, 3), 'nested_forks': (1, None),
                }[name]
                assert (estimate['minimum'], estimate['maximum']) == bounds, (name, estimate, bounds)
                assert observed >= estimate['minimum'], (name, observed, estimate)
                assert estimate['maximum'] is None or observed <= estimate['maximum'], (name, observed, estimate)
                expected = {'ordinary': 1, 'false_guard': 1, 'true_guard': 1,
                    'fork_command': 1, 'fork_function': 1 + count, 'double_call': 4,
                    'early_return': 1, 'return_run': 2, 'function_condition': 3,
                    'nested_forks': 1 + count * count}[name]
                assert observed == expected, (name, observed, expected)
                self.samples.append({'subjects': count, 'case': name, 'evaluations': observed,
                    'estimate': estimate, 'trace': str(path)})
                print('TRACE', json.dumps(self.samples[-1]), flush=True)
        self.send('tick unfreeze')


def main():
    """Compile the fixture, validate it on a disposable server, and retain evidence."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--jar', type=Path, required=True)
    parser.add_argument('--java', default='java')
    parser.add_argument('--output', type=Path, required=True)
    options = parser.parse_args()
    root = options.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    bodies = dict(CASES)
    bodies['leaf'] = ['scoreboard players add #result cost_probe 1']
    bodies['truth'] = ['scoreboard players add #condition cost_probe 1', 'return 1']
    for name in CASES:
        bodies['probe_' + name] = [f'function cost_semantics:{name}', f'say __COST_COMPLETE_{name}__']
    program = {'format': 'sand.program', 'format_version': 1, 'target': {'minecraft': '26.2'},
        'pack': {'namespace': 'cost_semantics', 'description': 'Runtime report regression'},
        'requires': ['raw_commands'], 'modules': [{'id': 'cost_semantics:fixture',
        'functions': [{'id': f'cost_semantics:{name}', 'context': 'server',
            'body': [{'action': {'op': 'raw', 'command': command}} for command in body]}
            for name, body in bodies.items()]}]}
    (root / 'fixture.sand.json').write_text(json.dumps(program, indent=2) + '\n')
    compiled = subprocess.run([str(options.binary.resolve()), 'program', 'compile', '--input',
        str(root / 'fixture.sand.json'), '--output', str(root / 'pack')],
        check=True, capture_output=True, text=True, timeout=60)
    response = json.loads(compiled.stdout)
    assert response['success']
    report = response['runtime_report']
    (root / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    args = argparse.Namespace(version='26.2', pack=str(root / 'pack'), jar=str(options.jar.resolve()),
        java=options.java, timeout=60, output=str(root / 'logs'), op_player=None, client_command=None)
    harness = Probe(args)
    harness.evidence_root, harness.report, harness.samples = root, report, []
    harness.run()
    (root / 'evidence.json').write_text(json.dumps({'complete': True, 'minecraft': '26.2',
        'metric': 'vanilla_debug_C_entries', 'samples': harness.samples}, indent=2) + '\n')
    print('EVIDENCE', root / 'evidence.json')


if __name__ == '__main__':
    main()
