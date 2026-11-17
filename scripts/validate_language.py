#!/usr/bin/env python3
"""Compile a language assertion fixture and execute it on vanilla 26.2.

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
        # Entity fixtures need a loaded origin before a server-context summon.
        self.load_test_chunk("minecraft:overworld")
        marker = "__SAND_LANGUAGE_OK__"
        for invocation in range(2):
            command = "execute positioned 0 80 0 run function language_probe:run"
            if not self.passed(f"LANGUAGE_{invocation}", command, marker):
                raise ValidationFailure("language", "language fixture failed its assertions")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, help="Source with language_probe:run and the standard assertion marker")
    options = parser.parse_args()
    root = options.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    fixture = options.fixture or Path(__file__).resolve().parents[1] / "sand-lang/tests/fixtures/control_flow.sand"
    subprocess.run([str(options.binary.resolve()), "lang", "build", str(fixture),
                    "--output", str(root / "pack")], check=True, timeout=60)
    args = argparse.Namespace(version="26.2", pack=str(root / "pack"), jar=options.jar,
                              java=options.java, timeout=120, output=str(root / "logs"),
                              op_player=None, client_command=None)
    harness = LanguageHarness(args)
    harness.run()
    print(f"PASSED: language fixture on vanilla 26.2; log: {harness.log_path}")


if __name__ == "__main__":
    main()
