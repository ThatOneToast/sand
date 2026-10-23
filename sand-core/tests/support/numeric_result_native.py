#!/usr/bin/env python3
"""Compare exported numeric result fixtures on vanilla using the maintained harness.

Export both JSON fixtures on the pre-elision base and candidate first. This
runner never builds Rust or substitutes handwritten numeric assignments.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scripts"))
from vanilla_reload_harness import ServerHarness, ValidationFailure  # noqa: E402


class Probe(ServerHarness):
    def __init__(self, args, numeric, author):
        super().__init__(args)
        self.numeric = numeric
        self.author = author
        self.observations = {}

    def prepare(self):
        super().prepare()
        with (self.server / "server.properties").open("a") as stream:
            stream.write("\nserver-ip=127.0.0.1\n")

    def batch(self, commands):
        start = len(self.lines)
        for command in commands:
            self.send(command)
        self.send("say NUMERIC_RESULT_BATCH_DONE")
        end = self.wait_for("NUMERIC_RESULT_BATCH_DONE", "numeric results", start)
        return self.lines[start:end]

    def observe(self, commands, names):
        lines = self.batch(commands + [f"scoreboard players get {name} probe_result" for name in names])
        result = {}
        for name in names:
            pattern = re.compile(re.escape(name) + r" has (-?\d+) \[probe_result\]")
            matches = [int(match[1]) for line in lines if (match := pattern.search(line))]
            if len(matches) != 1:
                raise ValidationFailure("numeric results", f"missing/duplicate observation {name}", lines)
            result[name] = matches[0]
        return result

    def run_client(self):
        fixture = self.numeric
        self.batch([
            "function result_probe:setup",
            "scoreboard objectives add observed dummy",
            f"scoreboard objectives add {self.author['gate']} dummy",
            "forceload add 0 0",
        ])
        # forceload returns before a newly generated chunk is entity-ticking.
        # Do not summon fixtures into a chunk still being asynchronously loaded.
        deadline = time.monotonic() + self.args.timeout
        while True:
            lines = self.batch(["execute if loaded 0 100 0 run say NUMERIC_CHUNK_LOADED"])
            if any("NUMERIC_CHUNK_LOADED" in line for line in lines):
                break
            if time.monotonic() >= deadline:
                raise ValidationFailure("numeric setup", "probe chunk did not load")
            time.sleep(0.1)
        self.batch([
            "kill @e[tag=numeric_probe]",
            'summon minecraft:armor_stand 0 100 0 {Tags:["numeric_probe","probe_first"],NoGravity:1b}',
            'summon minecraft:armor_stand 1 100 0 {Tags:["numeric_probe","probe_second"],NoGravity:1b}',
            "scoreboard players set #gate probe_gate 1",
        ])
        first = "@e[tag=probe_first,limit=1]"
        second = "@e[tag=probe_second,limit=1]"
        for case in fixture["cases"]:
            commands = ["scoreboard players set #observed probe_result -999"]
            fields = [fixture[key] for key in ["value", "field_dirty", "component_dirty"]]
            for field in fields:
                commands += [f"scoreboard players set #target {field} 0", f"scoreboard players set @e[tag=numeric_probe] {field} 0"]
            commands.append(f"execute as {first} run function result_probe:{case['name']}")
            names = ["#observed"]
            for owner, holder in [("fake", "#target"), ("first", first), ("second", second)]:
                for index, field in enumerate(fields):
                    name = f"#{owner}_{index}"
                    names.append(name)
                    commands.append(f"scoreboard players operation {name} probe_result = {holder} {field}")
            result = self.observe(commands, names)
            fake = [result[f"#fake_{i}"] for i in range(3)]
            entities = sorted([result[f"#{owner}_{i}"] for i in range(3)] for owner in ["first", "second"])
            written = [7, int(case["dirty"]), int(case["dirty"])]
            expected_fake = written if case["holder"] in ["fake", "wildcard"] else [0, 0, 0]
            expected_entities = sorted([written, written] if case["holder"] == "wildcard" else [[0, 0, 0], written] if case["holder"] in ["self", "random"] else [[0, 0, 0], [0, 0, 0]])
            if fake != expected_fake or entities != expected_entities:
                raise ValidationFailure("numeric ownership", f"{case['name']}: {result}")
            # Random ownership is compared as paired value/dirty tuples, not
            # entity identity: exactly one of two eligible entities must commit.
            normalized = {"result": result["#observed"], "fake": fake, "entities": entities}
            self.observations[case["name"]] = normalized
            if case["holder"] == "fake":
                sequence = case["dirty"] or case["bound"] in ["bounded", "lower"]
                expected = (0 if case["context"].startswith("return") else -999) if sequence else (1 if case["success"] else 7)
                if result["#observed"] != expected:
                    raise ValidationFailure("numeric callback", f"{case['name']}: expected {expected}, got {result['#observed']}")

        author = self.author
        for path, expected in [("bounded_result", -999), ("bounded_success", -999), ("unbounded_result", 7), ("unbounded_success", 1), ("operand_wrapper", -999), ("condition_wrapper", -999), ("return_result", -999), ("return_success", -999)]:
            for mode in ["result", "success"]:
                commands = [
                    "scoreboard players set #observed observed -999",
                    "scoreboard players set #outer probe_result -999",
                    f"scoreboard players set #gate {author['gate']} 1",
                    f"scoreboard players set {author['holder']} {author['bounded']} 0",
                    f"scoreboard players set {author['holder']} {author['unbounded']} 1",
                    f"execute store {mode} score #outer probe_result run function author_probe:{path}",
                    "scoreboard players operation #observed probe_result = #observed observed",
                    f"scoreboard players operation #value probe_result = {author['holder']} {author['unbounded'] if path.startswith('unbounded') else author['bounded']}",
                ]
                result = self.observe(commands, ["#observed", "#outer", "#value"])
                expected_outer = 0 if path.startswith("return") else -999
                if result != {"#observed": expected, "#outer": expected_outer, "#value": 7}:
                    raise ValidationFailure("public authoring callback", f"{path}/{mode}: {result}")
                self.observations[f"public/{path}/{mode}"] = result


def run(args, label):
    numeric_path = getattr(args, f"{label}_numeric")
    author_path = getattr(args, f"{label}_author")
    numeric = json.loads(numeric_path.read_text())
    author = json.loads(author_path.read_text())
    pack = args.output / f"{label}-pack"
    pack.mkdir(parents=True)
    (pack / "pack.mcmeta").write_text(json.dumps({"pack": {"pack_format": 107, "min_format": 107, "max_format": 107, "description": "Exported Sand numeric result regression"}}))
    for record in numeric["components"] + author["components"]:
        path = pack / "data" / record["namespace"] / record["dir"] / f"{record['path']}.{record['ext']}"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(record["content"])
    harness_args = argparse.Namespace(version="26.2", pack=str(pack), jar=str(args.jar), java=args.java, output=str(args.output / label), timeout=args.timeout, op_player=None, client_command=None)
    probe = Probe(harness_args, numeric, author)
    probe.run()
    pack_hash = hashlib.sha256()
    for file in sorted(pack.rglob("*")):
        if file.is_file():
            pack_hash.update(file.relative_to(pack).as_posix().encode() + b"\0" + file.read_bytes())
    evidence = {"candidate_commit": args.commit, "candidate_source_tree": args.source_tree,
                "fixture_production_commit": args.baseline_commit if label == "baseline" else args.commit,
                "candidate_source_sha256": args.source_hashes, "port": probe.port,
                "pack_sha256": pack_hash.hexdigest(), "observations": probe.observations, "fixture_sha256": {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in [numeric_path, author_path]}, "server_jar_sha256": hashlib.sha256(args.jar.read_bytes()).hexdigest(), "log": str(probe.log_path)}
    (args.output / f"{label}-results.json").write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    return probe.observations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for label in ["baseline", "candidate"]:
        for kind in ["numeric", "author"]:
            parser.add_argument(f"--{label}-{kind}", type=Path, required=True)
    parser.add_argument("--commit", required=True, help="frozen clean candidate commit")
    parser.add_argument("--baseline-commit", default="655e4f0cd2f1cc828eac724aabe045e3c47ffe0f")
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True, help="new directory for packs, server logs, and observations")
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True)
    if head != args.commit or status:
        raise ValidationFailure("source provenance", "candidate must match --commit and have a clean worktree")
    args.source_tree = subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip()
    sources = ["sand-core/src/ir/mod.rs", "sand-core/src/ir/numeric.rs",
               "sand-core/src/ir/numeric/result_tests.rs", "sand-core/tests/numeric_constant_results.rs",
               "sand-core/tests/support/numeric_result_native.py"]
    args.source_hashes = {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in sources}
    args.output.mkdir(parents=True, exist_ok=False)
    baseline = run(args, "baseline")
    candidate = run(args, "candidate")
    if baseline != candidate:
        differences = [key for key in baseline.keys() | candidate.keys() if baseline.get(key) != candidate.get(key)]
        raise ValidationFailure("numeric differential", f"before/after mismatches: {differences}")
    print(f"PASS: {len(candidate)} exported numeric/public authoring observations match; no MSPT claim")


if __name__ == "__main__":
    main()
