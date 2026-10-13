#!/usr/bin/env bash
set -euo pipefail

echo "=== Formatting ==="
cargo fmt --all -- --check

echo "=== Minimal Minecraft client regressions ==="
python3 -m unittest scripts.mc_validation.test_minimal_join_client scripts.tests.test_player_validation_harness scripts.mc_validation.test_local_rcon_server

echo "=== Clippy ==="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "=== Workspace tests ==="
cargo test --workspace --all-features

echo "=== Standalone example tests and datapack builds ==="
scripts/check-examples.sh

echo "=== External registration consumer (sand-only) ==="
cargo fmt --manifest-path sand/tests/fixtures/registration/Cargo.toml -- --check
CARGO_TARGET_DIR="$PWD/target" cargo test --locked --manifest-path sand/tests/fixtures/registration/Cargo.toml

echo "=== Portable program frontends and schema drift ==="
scripts/check-program.sh

echo "=== Rustdoc ==="
cargo doc --workspace --all-features --no-deps

echo "=== mdBook ==="
scripts/build-book.sh

echo "=== Doc links ==="
python3 scripts/check-docs.py

echo "=== All checks passed ==="
