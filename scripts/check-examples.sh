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

for manifest in "$repo_root"/examples/*/Cargo.toml; do
    project_dir="$(dirname -- "$manifest")"
    example="$(basename -- "$project_dir")"
    echo "=== Example: $example ==="
    cargo test --manifest-path "$manifest"
    if ! (
        cd -- "$project_dir" || exit 1
        cargo run --manifest-path "$repo_root/sand-cli/Cargo.toml" --bin sand -- build --format json > "$reports/$example.json"
    ); then
        cat -- "$reports/$example.json" >&2
        exit 1
    fi
    python3 "$repo_root/scripts/example_runtime_baseline.py" "$mode" \
        "$reports/$example.json" "$baseline_dir/$example.json"
done
python3 "$repo_root/scripts/example_runtime_baseline.py" inventory "$repo_root/examples" "$baseline_dir"
