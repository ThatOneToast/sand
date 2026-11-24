#!/usr/bin/env bash
set -euo pipefail
program_repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$program_repo_root"
cargo build -p sand-cli --bin sand
program_binary="$program_repo_root/target/debug/sand"
program_examples="$(mktemp -d)"
trap 'rm -rf "$program_examples"' EXIT
for program_source in "$program_repo_root"/examples/*/pack.sand; do
    program_example_dir="$(dirname "$program_source")"
    program_example_name="$(basename "$program_example_dir")"
    "$program_binary" lang build "$program_example_dir" --output "$program_examples/$program_example_name"
done
program_python="$program_repo_root/target/program-python/bin/python"
if [[ ! -x "$program_python" ]]; then
    python3 -m venv "$program_repo_root/target/program-python"
fi
"$program_python" -m pip install --disable-pip-version-check -r scripts/portable_program/requirements.txt
"$program_python" scripts/generate-program-schemas.py --binary "$program_binary" --check
(
    cd examples/portable_counter
    CARGO_TARGET_DIR="$program_repo_root/target" "$program_binary" build --format json
)
SAND_PROGRAM_BIN="$program_binary" \
SAND_RUST_PACK="$program_repo_root/examples/portable_counter/dist/demo" \
    "$program_python" -m unittest discover -s scripts/portable_program -v
