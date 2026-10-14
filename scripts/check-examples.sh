#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
# Share compiled dependencies across isolated example workspaces and CLI builds.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"
if [[ "$CARGO_TARGET_DIR" != /* ]]; then
    export CARGO_TARGET_DIR="$PWD/$CARGO_TARGET_DIR"
fi

for manifest in "$repo_root"/examples/*/Cargo.toml; do
    project_dir="$(dirname -- "$manifest")"
    echo "=== Example: $(basename -- "$project_dir") ==="
    cargo test --manifest-path "$manifest"
    (
        cd -- "$project_dir"
        cargo run --manifest-path "$repo_root/sand-cli/Cargo.toml" --bin sand -- build
    )
done
