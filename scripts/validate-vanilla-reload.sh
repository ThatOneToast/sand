#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CACHE_ROOT="${SAND_JAR_CACHE:-$HOME/.sand/cache}"

usage() {
    cat <<'EOF'
Usage: validate-vanilla-reload.sh --version <version> --pack <path> [OPTIONS]

Options:
  --jar <path>       SHA-verified server jar (default: Sand cache)
  --output <path>    Persistent diagnostics directory
  --timeout <secs>   Per-phase timeout (default: 120)
  --help             Show this help

The default jar is ~/.sand/cache/<version>/server.jar. Set SAND_JAR_CACHE to
override the Sand cache root. Populate it with:
  cargo run -p sand-build --bin ensure-server-jar -- <version>
EOF
}

VERSION=""
PACK=""
JAR=""
OUTPUT=""
TIMEOUT="${SAND_SERVER_TIMEOUT:-120}"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="${2:?}"; shift 2 ;;
        --pack) PACK="${2:?}"; shift 2 ;;
        --jar) JAR="${2:?}"; shift 2 ;;
        --output) OUTPUT="${2:?}"; shift 2 ;;
        --timeout) TIMEOUT="${2:?}"; shift 2 ;;
        --help|-h) usage; exit 0 ;;
        *) echo "error: unknown option '$1'" >&2; usage; exit 2 ;;
    esac
done

[[ -n "$VERSION" ]] || { echo "error: --version is required" >&2; exit 2; }
[[ -n "$PACK" ]] || { echo "error: --pack is required" >&2; exit 2; }
[[ -f "$PACK/pack.mcmeta" ]] || { echo "error: missing $PACK/pack.mcmeta" >&2; exit 2; }

if [[ -z "$JAR" ]]; then
    JAR="$CACHE_ROOT/$VERSION/server.jar"
    legacy="$CACHE_ROOT/server-$VERSION.jar"
    [[ -f "$JAR" || ! -f "$legacy" ]] || JAR="$legacy"
fi
[[ -f "$JAR" ]] || {
    echo "error: no cached server jar for $VERSION at $JAR" >&2
    echo "run: cargo run -p sand-build --bin ensure-server-jar -- $VERSION" >&2
    exit 2
}

args=(
    "$REPO_ROOT/scripts/vanilla_reload_harness.py"
    --version "$VERSION"
    --pack "$PACK"
    --jar "$JAR"
    --timeout "$TIMEOUT"
)
[[ -z "$OUTPUT" ]] || args+=(--output "$OUTPUT")
exec python3 "${args[@]}"
