#!/usr/bin/env bash
set -euo pipefail

echo "=== Formatting ==="
cargo fmt --all -- --check

echo "=== Python harness regressions ==="
scripts/check-python.sh

echo "=== Clippy ==="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "=== Workspace tests ==="
cargo test --workspace --all-features --no-fail-fast

echo "=== Standalone example tests and datapack builds ==="
scripts/check-examples.sh

echo "=== External registration consumer (sand-only) ==="
cargo fmt --manifest-path sand/tests/fixtures/registration/Cargo.toml -- --check
CARGO_TARGET_DIR="$PWD/target" cargo test --locked --manifest-path sand/tests/fixtures/registration/Cargo.toml

echo "=== External ability linkage consumers (sand-only) ==="
scripts/check-ability-linking.sh

echo "=== Portable program frontends and schema drift ==="
scripts/check-program.sh

echo "=== Rustdoc ==="
cargo doc --workspace --all-features --no-deps

echo "=== mdBook ==="
scripts/build-book.sh

echo "=== Doc links ==="
python3 scripts/check-docs.py

echo "=== All checks passed ==="
