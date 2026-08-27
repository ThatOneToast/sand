#!/usr/bin/env bash
# Driver for the post-#350 multi-sample benchmark re-baseline (Phase 1).
# Runs each scenario via scripts/bench_build.sh and appends Markdown results
# to /tmp/bench_results.md. See BENCHMARKS.md for narrative + methodology.
#
# Sample counts: 5 for hot/incremental scenarios, 2 for clean-build
# scenarios (reduced from the nominal 7-10/3+ spec; see BENCHMARKS.md).

set -euo pipefail
cd "$(dirname "$0")/.."
BENCH="scripts/bench_build.sh"
OUT=/tmp/bench_results.md
: > "$OUT"

HOT=5
CLEAN=2

log() { echo "== $1 ==" | tee -a "$OUT" >&2; }

# --- 1. No-change cargo check (warm) ---
log "1. no-change cargo check --workspace"
cargo check --workspace >/tmp/bench_warm.log 2>&1
"$BENCH" "no-change-check" "$HOT" -- cargo check --workspace >>"$OUT"

# --- 2. Edit private impl body in sand-core ---
log "2. edit private impl body (sand-core)"
FILE=sand-core/src/lib.rs
LINE=$(grep -n 'self.resource_location().to_owned()' "$FILE" | head -1 | cut -d: -f1)
ORIG=$(sed -n "${LINE}p" "$FILE")
for i in $(seq 1 $HOT); do
  sed -i '' "${LINE}s#.*#        self.resource_location().to_owned() // bench-${i}#" "$FILE"
  "$BENCH" "edit-private-impl-sand-core-s${i}" 1 -- cargo check --workspace >>"$OUT"
done
sed -i '' "${LINE}s#.*#${ORIG}#" "$FILE"

# --- 3. Edit public/API-contract declaration in sand facade ---
log "3. edit api-contract declaration (sand/src/api_contracts.rs)"
FILE=sand/src/api_contracts.rs
LINE=1
ORIG=$(sed -n "${LINE}p" "$FILE")
for i in $(seq 1 $HOT); do
  sed -i '' "${LINE}s#.*#${ORIG} // bench-${i}#" "$FILE"
  "$BENCH" "edit-api-contract-decl-s${i}" 1 -- cargo check --workspace >>"$OUT"
  sed -i '' "${LINE}s#.*#${ORIG}#" "$FILE"
done

# --- 4. Edit implementation code in sand-components ---
log "4. edit impl (sand-components)"
FILE=sand-components/src/animal_variant.rs
LINE=$(grep -n 'pub fn biomes_raw' "$FILE" | head -1 | cut -d: -f1)
LINE=$((LINE + 1))
ORIG=$(sed -n "${LINE}p" "$FILE")
for i in $(seq 1 $HOT); do
  sed -i '' "${LINE}s#\$# // bench-${i}#" "$FILE"
  "$BENCH" "edit-impl-sand-components-s${i}" 1 -- cargo check --workspace >>"$OUT"
  sed -i '' "${LINE}s#.*#${ORIG}#" "$FILE"
done

# --- 5. Edit generated-provider-affecting source (sand-components registry.rs) ---
log "5. edit generated-provider-affecting source"
FILE=sand-components/src/registry.rs
LINE=1
ORIG=$(sed -n "${LINE}p" "$FILE")
for i in $(seq 1 $HOT); do
  sed -i '' "${LINE}s#.*#${ORIG} // bench-${i}#" "$FILE"
  "$BENCH" "edit-generated-provider-source-s${i}" 1 -- cargo check --workspace >>"$OUT"
  sed -i '' "${LINE}s#.*#${ORIG}#" "$FILE"
done

# --- 6. Edit documentation outside crate source trees ---
log "6. edit docs outside crate src trees"
FILE=README.md
LINE=1
ORIG=$(sed -n "${LINE}p" "$FILE")
for i in $(seq 1 $HOT); do
  sed -i '' "${LINE}s#\$# <!-- bench-${i} -->#" "$FILE"
  "$BENCH" "edit-docs-s${i}" 1 -- cargo check --workspace >>"$OUT"
  sed -i '' "${LINE}s#.*#${ORIG}#" "$FILE"
done

# --- 7. Clean cargo check --workspace ---
log "7. clean cargo check --workspace"
"$BENCH" "clean-check" "$CLEAN" -- bash -c '
  cargo clean -p sand -p sand-cli -p sand-core -p sand-components -p sand-commands \
    -p sand-macros -p sand-resourcepack -p sand-version -p sand-api-contract \
    -p sand-api-enforce -p sand-build -p sand-example >/dev/null 2>&1
  cargo check --workspace
' >>"$OUT"

# Build a fresh `sand` binary once so scenarios 8-10 exercise current code,
# not a stale ~/.cargo/bin/sand install.
log "building fresh sand binary"
cargo build -q -p sand-cli --bin sand
SAND_BIN="$(pwd)/target/debug/sand"

# --- 8. Real `sand build` on sand-example after a source edit ---
log "8. sand build after editing sand-core source (real project)"
FILE=sand-core/src/lib.rs
LINE=$(grep -n 'self.resource_location().to_owned()' "$FILE" | head -1 | cut -d: -f1)
ORIG=$(sed -n "${LINE}p" "$FILE")
for i in $(seq 1 $HOT); do
  sed -i '' "${LINE}s#.*#        self.resource_location().to_owned() // bench-${i}#" "$FILE"
  "$BENCH" "sand-build-after-edit-s${i}" 1 -- bash -c "cd sand-example && '$SAND_BIN' build" >>"$OUT"
done
sed -i '' "${LINE}s#.*#${ORIG}#" "$FILE"

# --- 9. Immediate no-change `sand build` ---
log "9. no-change sand build"
(cd sand-example && "$SAND_BIN" build >/dev/null 2>&1) || true
"$BENCH" "no-change-sand-build" "$HOT" -- bash -c "cd sand-example && '$SAND_BIN' build" >>"$OUT"

# --- 10. `sand build` then `sand build --release` ---
log "10. sand build then sand build --release"
"$BENCH" "sand-build-then-release" 1 -- bash -c "cd sand-example && '$SAND_BIN' build && '$SAND_BIN' build --release" >>"$OUT"

# --- 11. Warm ~/.sand/cache, cold Cargo target ---
log "11. warm sand cache, cold cargo target"
"$BENCH" "warm-sandcache-cold-cargo-target" 1 -- bash -c "
  cargo clean -p sand -p sand-cli -p sand-core -p sand-components -p sand-commands \
    -p sand-macros -p sand-resourcepack -p sand-version -p sand-api-contract \
    -p sand-api-enforce -p sand-build -p sand-example >/dev/null 2>&1
  cargo build -q -p sand-cli --bin sand
  cd sand-example && '$SAND_BIN' build
" >>"$OUT"

echo "done" >&2
