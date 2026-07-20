#!/usr/bin/env bash
set -euo pipefail

echo "=== Formatting ==="
cargo fmt --all -- --check

echo "=== Clippy ==="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "=== Workspace tests ==="
cargo test --workspace --all-features

echo "=== Macro trybuild tests ==="
cargo test -p sand-macros

echo "=== Canonical book_project (facade-only, MC 26.2) ==="
cargo build --manifest-path examples/book_project/Cargo.toml
(
    cd examples/book_project
    SAND_EXPORT_MC_VERSION=26.2 cargo run --bin sand_export |
        python3 -c 'import json, sys; json.load(sys.stdin)'
)

echo "=== Participant runtime-validation audit pack (facade-only, MC 26.2, #265) ==="
cargo test --manifest-path examples/participant_audit/Cargo.toml
(
    cd examples/participant_audit
    SAND_EXPORT_MC_VERSION=26.2 cargo run --bin sand_export |
        python3 -c 'import json, sys; json.load(sys.stdin)'
)

echo "=== Rustdoc ==="
cargo doc --workspace --all-features --no-deps

echo "=== mdBook ==="
scripts/build-book.sh

echo "=== Doc links ==="
python3 scripts/check-docs.py

echo "=== All checks passed ==="
