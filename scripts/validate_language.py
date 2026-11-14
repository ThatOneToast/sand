#!/usr/bin/env python3
"""Compile the language control-flow fixture and execute it on vanilla 26.2.

Requires a built Sand CLI, a matching server JAR, and Java. Keeps compilation,
startup, reload, and assertion evidence in the requested output directory.
"""
import argparse
from pathlib import Path
import subprocess

from vanilla_reload_harness import ValidationFailure
from validate_world_gameplay import WorldGameplayHarness


class LanguageHarness(WorldGameplayHarness):
    def run_client(self):
        marker = "__SAND_LANGUAGE_OK__"
        if not self.passed("LANGUAGE", "function language_probe:run", marker):
            raise ValidationFailure("language", "control-flow fixture failed its assertions")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    root = options.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    fixture = Path(__file__).resolve().parents[1] / "sand-lang/tests/fixtures/control_flow.sand"
    subprocess.run([str(options.binary.resolve()), "lang", "build", str(fixture),
                    "--output", str(root / "pack")], check=True, timeout=60)
    args = argparse.Namespace(version="26.2", pack=str(root / "pack"), jar=options.jar,
                              java=options.java, timeout=120, output=str(root / "logs"),
                              op_player=None, client_command=None)
    harness = LanguageHarness(args)
    harness.run()
    print(f"PASSED: language control flow on vanilla 26.2; log: {harness.log_path}")


if __name__ == "__main__":
    main()
