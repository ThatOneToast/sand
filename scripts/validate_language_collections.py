#!/usr/bin/env python3
"""Install the bundled collections library and prove its behavior on vanilla 26.2."""
import argparse
from pathlib import Path
import shutil
import subprocess

from validate_language import LanguageHarness
from vanilla_reload_harness import ValidationFailure


class CollectionsHarness(LanguageHarness):
    def run_client(self):
        self.load_test_chunk("minecraft:overworld")
        for invocation in range(2):
            for phase in ("run", "check_capacity", "check_update"):
                command = f"execute positioned 0 80 0 run function language_probe:{phase}"
                if not self.passed(f"COLLECTIONS_{invocation}_{phase}", command, "__SAND_LANGUAGE_OK__"):
                    raise ValidationFailure("language collections", f"{phase} failed its assertions")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--java", default="java")
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    root = options.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    source = root / "source"
    source.mkdir()
    fixture = Path(__file__).resolve().parents[1] / "sand-lang/tests/fixtures/maps.sand"
    shutil.copyfile(fixture, source / "pack.sand")
    binary = str(options.binary.resolve())
    subprocess.run([binary, "lang", "add", "collections", str(source)], check=True, timeout=60)
    # A child test module can access its parent's private fields. Seed a valid
    # full map in State to prove capacity without a quadratic setup workload.
    with (source / "collections.sand").open("a") as module:
        module.write("\npub mod test_support;\n")
    support = source / "collections"
    support.mkdir()
    entries = ",".join(f"Entry {{ key: {n}, value: {n} }}" for n in range(1024))
    (support / "test_support.sand").write_text(
        "use super::{Map, Entry};\n"
        f"global state Full {{ values: Map<number, number> = Map {{ items: [{entries}] }} }}\n"
        "pub fn full_map() -> Map<number, number> { return Full.values; }\n"
    )
    subprocess.run([binary, "lang", "build", str(source), "--output", str(root / "pack")], check=True, timeout=60)
    args = argparse.Namespace(version="26.2", pack=str(root / "pack"), jar=options.jar,
                              java=options.java, timeout=120, output=str(root / "logs"),
                              op_player=None, client_command=None)
    harness = CollectionsHarness(args)
    harness.run()
    print(f"PASSED: bundled collections on vanilla 26.2; log: {harness.log_path}")


if __name__ == "__main__":
    main()
