#!/usr/bin/env bash
set -euo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "${1:-}" == "--archetype-idle" ]]; then
    shift
    exec python3 "$script_dir/measure_archetype_idle.py" "$@"
fi
exec python3 "$script_dir/bench_runtime.py" "$@"
