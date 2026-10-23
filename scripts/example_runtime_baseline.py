#!/usr/bin/env python3
"""Compare canonical RuntimeReport data; never estimate work from pack text."""

import argparse
import copy
import difflib
import json
from pathlib import Path
import sys


def canonical_json(value):
    return json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def fields(value, expected, location):
    if not isinstance(value, dict) or set(value) != set(expected.split()):
        raise ValueError(f"{location}: unexpected fields; review the report schema")


def ordered_set(values):
    # Sort without deduplicating: a newly repeated contribution must remain visible.
    return sorted(values, key=canonical_json)


def normalize(build):
    """Retain report semantics, excluding the build's paths/timings/change summary.

    Lifecycle entries and fused-system owners retain execution order. Only known
    unordered collections are sorted. Lexical selector line numbers and all
    unknown reasons stay visible, including the report's truncation markers.
    """
    if build.get("schema_version") != 1 or build.get("success") is not True:
        raise ValueError("expected successful schema-version-1 sand build JSON")
    report = copy.deepcopy(build.get("runtime_report"))
    fields(report, "schema_version provenance tick load functions limitations", "runtime_report")
    if report["schema_version"] != 1:
        raise ValueError("unsupported RuntimeReport schema version")

    def work(value):
        fields(value, "minimum maximum unknown", "evaluations")
        minimum, maximum = value["minimum"], value["maximum"]
        if type(minimum) is not int or minimum < 0 or (
            maximum is not None and (type(maximum) is not int or maximum < minimum)
        ):
            raise ValueError("invalid evaluation range")
        value["unknown"] = ordered_set(value["unknown"])

    for phase in ("tick", "load"):
        for entry in report[phase]:
            fields(entry, "reference required contributors evaluations", phase)
            entry["contributors"] = ordered_set(entry["contributors"])
            work(entry["evaluations"])
    for function in report["functions"].values():
        fields(function, "owners evaluations selector_sites", "function")
        function["owners"] = ordered_set(function["owners"])
        work(function["evaluations"])
        for site in function["selector_sites"]:
            fields(site, "line operation selector", "selector site")
        function["selector_sites"] = ordered_set(function["selector_sites"])
    report["limitations"] = ordered_set(report["limitations"])
    provenance = report["provenance"]
    fields(provenance, "function_owners tag_contributors systems archetypes lifecycle_owners", "provenance")
    for key, owners in provenance["function_owners"].items():
        provenance["function_owners"][key] = ordered_set(owners)
    for entry in provenance["tag_contributors"]:
        entry["owners"] = ordered_set(entry["owners"])
    provenance["tag_contributors"] = ordered_set(provenance["tag_contributors"])
    # systems (and their owners) are ordered by the canonical lowering. Do not sort.
    for archetype in provenance["archetypes"]:
        for key in ("objectives", "tags", "functions"):
            archetype[key] = ordered_set(archetype[key])
    provenance["archetypes"] = ordered_set(provenance["archetypes"])
    for key, spans in provenance["lifecycle_owners"].items():
        provenance["lifecycle_owners"][key] = ordered_set(spans)
    return {
        "baseline_schema_version": 1,
        "namespace": build["namespace"],
        "minecraft_version": build["minecraft_version"],
        "profile": build["profile"],
        "runtime_report": report,
    }


def compare(build_path, baseline_path, update=False):
    actual = canonical_json(normalize(json.loads(build_path.read_text())))
    if update:
        baseline_path.parent.mkdir(parents=True, exist_ok=True)
        baseline_path.write_text(actual)
        print(f"Regenerated {baseline_path}")
        return 0
    if not baseline_path.is_file():
        raise ValueError(f"missing baseline: {baseline_path}")
    # Canonicalize object keys/whitespace in the committed artifact as well.
    expected = canonical_json(json.loads(baseline_path.read_text()))
    if expected == actual:
        print(f"Runtime baseline matches: {baseline_path.stem}")
        return 0
    print("".join(difflib.unified_diff(
        expected.splitlines(keepends=True), actual.splitlines(keepends=True),
        fromfile=str(baseline_path), tofile="current RuntimeReport",
    )), end="", file=sys.stderr)
    print("Runtime baseline drift: review the delta, then run "
          "scripts/check-examples.sh --update-runtime-baselines", file=sys.stderr)
    return 1


def inventory(examples, baselines):
    expected = {path.parent.name for path in examples.glob("*/Cargo.toml")}
    actual = {path.stem for path in baselines.glob("*.json")}
    if expected != actual:
        raise ValueError(f"baseline inventory mismatch: missing={sorted(expected - actual)}, "
                         f"stale={sorted(actual - expected)}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("check", "update"):
        command = commands.add_parser(name)
        command.add_argument("build_json", type=Path)
        command.add_argument("baseline", type=Path)
    command = commands.add_parser("inventory")
    command.add_argument("examples", type=Path)
    command.add_argument("baselines", type=Path)
    args = parser.parse_args(argv)
    try:
        if args.command == "inventory":
            inventory(args.examples, args.baselines)
            return 0
        return compare(args.build_json, args.baseline, args.command == "update")
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Runtime baseline error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
