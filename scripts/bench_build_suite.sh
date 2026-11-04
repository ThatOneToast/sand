#!/usr/bin/env bash
# Isolated dirty-worktree snapshot; no live source edits or cache deletion.
# Run --help for artifact, version and sample-count options.
set -euo pipefail
exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/bench_build.py" suite "$@"
