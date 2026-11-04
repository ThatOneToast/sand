#!/usr/bin/env bash
# Compatible timing entrypoint: <scenario-name> <samples> -- <command...>.
# Unique per-run logs and results.json replace shared /tmp files.
set -euo pipefail
exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/bench_build.py" single "$@"
