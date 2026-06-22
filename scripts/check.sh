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
if command -v mdbook >/dev/null 2>&1; then
  mdbook build
else
  echo "mdbook not installed, skipping"
fi

echo "=== All checks passed ==="
