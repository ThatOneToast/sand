#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/sand/tests/fixtures/ability-linking/Cargo.toml"
reports="$(mktemp -d)"
trap 'rm -rf -- "$reports"' EXIT
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"
cargo fmt --manifest-path "$manifest" -- --check
for mode in baseline discarded registered active; do
    cargo run --locked --quiet --manifest-path "$manifest" --bin "$mode" > "$reports/$mode.json"
done
python3 - "$reports" <<'PY'
import json, sys
from pathlib import Path
root = Path(sys.argv[1])
reports = {mode: (root / (mode + '.json')).read_bytes()
           for mode in ('baseline', 'discarded', 'registered', 'active')}
assert reports['baseline'] == reports['discarded'], 'construct/discard changes complete export'
assert reports['baseline'] != reports['registered'], 'registration must add maintenance'
assert reports['registered'] != reports['active'], 'activation must add its function'
for mode in ('registered', 'active'):
    pack = json.loads(reports[mode])
    ticks = [c['content'] for c in pack['components']
             if c['dir'] == 'function' and c['path'] == '__sand_lifecycle_tick']
    assert len(ticks) == 1 and ticks[0].count('function linking:recharge') == 1
print('Independent sand-only linkage consumers: complete export equality and registration deltas passed')
PY
