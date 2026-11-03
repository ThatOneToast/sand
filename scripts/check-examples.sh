#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
mode=check
if [[ "${1:-}" == --update-runtime-baselines && $# == 1 ]]; then
    mode=update
elif [[ $# != 0 ]]; then
    echo "Usage: $0 [--update-runtime-baselines]" >&2
    exit 2
fi
baseline_dir="$repo_root/benchmarks/example-runtime-baselines"
reports="$(mktemp -d)"
trap 'rm -rf -- "$reports"' EXIT
(cd -- "$repo_root"; python3 -m unittest scripts.tests.test_example_runtime_baseline)

# Share compiled dependencies across isolated example workspaces and CLI builds.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"
if [[ "$CARGO_TARGET_DIR" != /* ]]; then
    export CARGO_TARGET_DIR="$PWD/$CARGO_TARGET_DIR"
fi

# Build the CLI once before switching to each example's code-generation target.
# Re-running Cargo for the CLI inside the loop otherwise switches the shared
# Sand dependency graph back to the default Minecraft version on every pack.
cargo build --manifest-path "$repo_root/sand-cli/Cargo.toml" --bin sand
sand_binary="$CARGO_TARGET_DIR/debug/sand"

for manifest in "$repo_root"/examples/*/Cargo.toml; do
    project_dir="$(dirname -- "$manifest")"
    example="$(basename -- "$project_dir")"
    echo "=== Example: $example ==="
    example_version="${SAND_MC_VERSION:-$(python3 - "$project_dir/sand.toml" <<'PYTHON'
import sys
import tomllib
from pathlib import Path
print(tomllib.loads(Path(sys.argv[1]).read_text())["pack"]["mc_version"])
PYTHON
)}"
    SAND_MC_VERSION="$example_version" cargo test --manifest-path "$manifest"
    if ! (
        cd -- "$project_dir" || exit 1
        SAND_MC_VERSION="$example_version" "$sand_binary" build --format json > "$reports/$example.json"
    ); then
        cat -- "$reports/$example.json" >&2
        exit 1
    fi
    python3 "$repo_root/scripts/example_runtime_baseline.py" "$mode" \
        "$reports/$example.json" "$baseline_dir/$example.json"
done
python3 "$repo_root/scripts/example_runtime_baseline.py" inventory "$repo_root/examples" "$baseline_dir"
