#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CLIENT_ROOT="$SCRIPT_DIR/vanilla-semantic-client"

if [[ $# -lt 1 ]]; then
    echo "usage: validate-vanilla-semantics.sh <generated-pack-path> [output-path]" >&2
    exit 2
fi

PACK="$1"
OUTPUT="${2:-$REPO_ROOT/target/vanilla-reload/semantic-26.2}"

npm ci --prefix "$CLIENT_ROOT" --no-audit --no-fund

exec "$SCRIPT_DIR/validate-vanilla-reload.sh" \
    --version 26.2 \
    --pack "$PACK" \
    --output "$OUTPUT" \
    --op-player SandAuditBot \
    --client-command node "$CLIENT_ROOT/client.cjs"
