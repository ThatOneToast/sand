#!/usr/bin/env python3
"""Generate or check portable schema artifacts using an absolute prebuilt Sand binary."""
import argparse
import json
from pathlib import Path
import subprocess


def artifacts(binary):
    def schema(module=False):
        args = [str(binary), "program", "schema", "--format", "json"]
        if module:
            args.append("--module")
        return json.loads(subprocess.check_output(args, timeout=30))["schema"]
    bundled, module = schema(), schema(True)
    result = {"bundled.schema.json": bundled, "module.schema.json": module}
    definitions = bundled.get("$defs", {})
    def offline(value):
        if isinstance(value, list):
            return [offline(child) for child in value]
        if isinstance(value, dict):
            return {key: ("defs-" + child.removeprefix("#/$defs/") + ".schema.json"
                          if key == "$ref" and child.startswith("#/$defs/") else offline(child))
                    for key, child in value.items()}
        return value
    for name, definition in definitions.items():
        result["defs-" + name + ".schema.json"] = {"$schema": bundled["$schema"], **offline(definition)}
    root = offline({key: value for key, value in bundled.items() if key != "$defs"})
    result["program.schema.json"] = root
    return {name: json.dumps(value, indent=2, sort_keys=True) + "\n" for name, value in result.items()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1] / "schemas" / "program"
    expected = artifacts(args.binary.resolve())
    if args.check:
        actual = {path.name: path.read_text() for path in root.glob("*.json")}
        if actual != expected:
            raise SystemExit("portable schema drift: regenerate with scripts/generate-program-schemas.py")
    else:
        root.mkdir(parents=True, exist_ok=True)
        for path in root.glob("*.json"):
            if path.name not in expected:
                path.unlink()
        for name, body in expected.items():
            (root / name).write_text(body)


if __name__ == "__main__":
    main()
