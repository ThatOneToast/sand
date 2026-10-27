#!/usr/bin/env bash
set -euo pipefail

echo "=== Formatting ==="
cargo fmt --all -- --check

echo "=== Minimal Minecraft client regressions ==="
python3 -m py_compile scripts/validate_runtime_report.py scripts/validate_portable_global.py scripts/validate_state_flow.py scripts/validate_trailforge.py scripts/validate_portable_flows.py scripts/validate_furnace_gameplay.py scripts/tests/test_furnace_gameplay.py scripts/validate_live_item_copy.py
python3 -m unittest scripts.mc_validation.test_minimal_join_client scripts.tests.test_player_validation_harness scripts.mc_validation.test_local_rcon_server scripts.mc_validation.test_unified_state_audit scripts.tests.test_archetype_cost scripts.tests.test_dev_workflow scripts.tests.test_furnace_gameplay scripts.tests.test_live_item_copy scripts.tests.test_server_jar_consumers

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
