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

echo "=== Rustdoc ==="
cargo doc --workspace --all-features --no-deps

echo "=== mdBook ==="
scripts/build-book.sh

echo "=== AI resources ==="
python3 scripts/check-ai-resources.py

echo "=== Doc links ==="
python3 scripts/check-docs.py

echo "=== All checks passed ==="
