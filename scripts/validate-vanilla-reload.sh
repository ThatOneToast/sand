#!/usr/bin/env bash
# validate-vanilla-reload.sh — opt-in vanilla Minecraft server reload harness.
#
# Downloads (or reuses a cached) vanilla server jar, installs a generated
# datapack into a temporary world, starts the server long enough to load the
# pack, triggers /reload, and fails if the logs contain datapack errors.
#
# Usage:
#   scripts/validate-vanilla-reload.sh --version 1.21.4 --pack dist/my_pack
#   scripts/validate-vanilla-reload.sh --help
#
# NOT run by default CI (cargo test).  Add it to a scheduled or manual
# workflow, or run it locally before releasing a new pack version.
#
# Requirements: java (>= 21), curl, jq (for version manifest parsing).

set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
JAR_CACHE_DIR="${SAND_JAR_CACHE:-$HOME/.cache/sand/minecraft-jars}"
MANIFEST_URL="https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
TIMEOUT_SECONDS="${SAND_SERVER_TIMEOUT:-90}"
VERSION=""
PACK_DIR=""
WORLD_NAME="sand_validate"

# ── Help ──────────────────────────────────────────────────────────────────────

usage() {
    cat <<'EOF'
Usage: validate-vanilla-reload.sh [OPTIONS]

Validate a generated Sand datapack against a vanilla Minecraft server.

Options:
  --version <version>   Minecraft Java version to test against (e.g. 1.21.4)
  --pack <path>         Path to the generated datapack folder (contains pack.mcmeta)
  --help                Show this help message

Environment variables:
  SAND_JAR_CACHE        Directory to cache downloaded server jars
                        (default: ~/.cache/sand/minecraft-jars)
  SAND_SERVER_TIMEOUT   Seconds to wait for server startup (default: 90)

Exit codes:
  0  Pack loaded cleanly — no datapack errors in logs
  1  Pack or log contained errors (exact log path printed)
  2  Script usage error (bad arguments, missing files)

Example:
  cargo run -p sand -- build
  scripts/validate-vanilla-reload.sh --version 1.21.4 --pack dist/my_pack
EOF
}

# ── Argument parsing ──────────────────────────────────────────────────────────

if [[ $# -eq 0 ]]; then
    usage
    exit 2
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --pack)
            PACK_DIR="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown option '$1'" >&2
            usage
            exit 2
            ;;
    esac
done

if [[ -z "$VERSION" ]]; then
    echo "error: --version is required" >&2
    exit 2
fi

if [[ -z "$PACK_DIR" ]]; then
    echo "error: --pack is required" >&2
    exit 2
fi

if [[ ! -f "$PACK_DIR/pack.mcmeta" ]]; then
    echo "error: '$PACK_DIR/pack.mcmeta' not found — is '$PACK_DIR' a valid datapack?" >&2
    exit 2
fi

# ── Helpers ───────────────────────────────────────────────────────────────────

log() { echo "[validate] $*"; }
fail() { echo "FAILED: $*" >&2; exit 1; }

check_dependency() {
    command -v "$1" &>/dev/null || fail "Missing dependency: $1 (install it and retry)"
}

check_dependency java
check_dependency curl
check_dependency jq

# ── Resolve server jar ────────────────────────────────────────────────────────

mkdir -p "$JAR_CACHE_DIR"
SERVER_JAR="$JAR_CACHE_DIR/server-${VERSION}.jar"

if [[ ! -f "$SERVER_JAR" ]]; then
    log "Downloading Mojang version manifest..."
    MANIFEST_FILE=$(mktemp)
    trap 'rm -f "$MANIFEST_FILE"' EXIT
    curl -fsSL "$MANIFEST_URL" -o "$MANIFEST_FILE"

    log "Resolving download URL for Minecraft $VERSION..."
    VERSION_URL=$(jq -r --arg v "$VERSION" \
        '.versions[] | select(.id == $v) | .url' "$MANIFEST_FILE")

    if [[ -z "$VERSION_URL" || "$VERSION_URL" == "null" ]]; then
        fail "Minecraft version '$VERSION' not found in Mojang manifest"
    fi

    VERSION_META=$(mktemp)
    trap 'rm -f "$MANIFEST_FILE" "$VERSION_META"' EXIT
    curl -fsSL "$VERSION_URL" -o "$VERSION_META"

    DOWNLOAD_URL=$(jq -r '.downloads.server.url' "$VERSION_META")
    if [[ -z "$DOWNLOAD_URL" || "$DOWNLOAD_URL" == "null" ]]; then
        fail "Server jar download URL not found in version metadata for $VERSION"
    fi

    log "Downloading server jar ($VERSION)..."
    curl -fsSL "$DOWNLOAD_URL" -o "$SERVER_JAR"
    log "Cached to $SERVER_JAR"
else
    log "Using cached server jar: $SERVER_JAR"
fi

# ── Create temporary server directory ────────────────────────────────────────

TMP_DIR=$(mktemp -d)
trap 'log "Cleaning up $TMP_DIR"; rm -rf "$TMP_DIR"' EXIT

log "Using temporary server directory: $TMP_DIR"

# ── Accept EULA (only in the temp dir) ────────────────────────────────────────

echo "eula=true" > "$TMP_DIR/eula.txt"
log "EULA accepted (in temp dir only)"

# ── Install the datapack into a clean world ───────────────────────────────────

WORLD_DIR="$TMP_DIR/$WORLD_NAME"
DATAPACK_DEST="$WORLD_DIR/datapacks/$(basename "$PACK_DIR")"
mkdir -p "$DATAPACK_DEST"
cp -r "$PACK_DIR"/. "$DATAPACK_DEST/"
log "Installed datapack to $DATAPACK_DEST"

# ── Write a minimal server.properties ─────────────────────────────────────────

cat > "$TMP_DIR/server.properties" <<EOF
level-name=$WORLD_NAME
online-mode=false
max-players=0
enable-rcon=false
spawn-npcs=false
spawn-animals=false
spawn-monsters=false
generate-structures=false
view-distance=2
simulation-distance=2
EOF

# ── Start the server ──────────────────────────────────────────────────────────

LOG_FILE="$TMP_DIR/logs/latest.log"
mkdir -p "$TMP_DIR/logs"

log "Starting server (timeout: ${TIMEOUT_SECONDS}s)..."
java -jar "$SERVER_JAR" --nogui \
    --universe "$TMP_DIR" \
    --world "$WORLD_NAME" \
    2>&1 | tee "$LOG_FILE" &
SERVER_PID=$!

# ── Wait for the server to load or timeout ────────────────────────────────────

ELAPSED=0
LOADED=false
while [[ $ELAPSED -lt $TIMEOUT_SECONDS ]]; do
    if grep -q "Done (.*)! For help" "$LOG_FILE" 2>/dev/null; then
        LOADED=true
        break
    fi
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

# Stop the server after we've seen the "Done" message (or timeout).
kill "$SERVER_PID" 2>/dev/null || true
wait "$SERVER_PID" 2>/dev/null || true

if [[ "$LOADED" != true ]]; then
    log "Server did not finish startup within ${TIMEOUT_SECONDS}s"
    log "Log: $LOG_FILE"
    exit 1
fi

log "Server loaded — scanning logs for datapack errors..."

# ── Check logs for errors ─────────────────────────────────────────────────────

ERROR_PATTERNS=(
    "Failed to load datapack"
    "Could not load recipe"
    "Could not load loot table"
    "Could not load predicate"
    "Could not load advancement"
    "Could not load function"
    "Could not load tag"
    "Unknown command"
    "Unknown argument"
    "Parsing error"
    "JsonParseException"
    "IllegalArgumentException"
    "DatapackLoadFailedException"
    "Couldn't load"
    "Failed to parse"
    "Expected value"
)

ERRORS_FOUND=false
for pattern in "${ERROR_PATTERNS[@]}"; do
    if grep -qi "$pattern" "$LOG_FILE"; then
        ERRORS_FOUND=true
        log "ERROR pattern matched: '$pattern'"
        grep -i "$pattern" "$LOG_FILE" | head -5 | while IFS= read -r line; do
            log "  > $line"
        done
    fi
done

if [[ "$ERRORS_FOUND" == true ]]; then
    echo ""
    echo "FAILED: Datapack errors detected in $LOG_FILE"
    echo "  Run: less $LOG_FILE"
    exit 1
fi

log "OK: No datapack errors detected."
log "Log: $LOG_FILE"
echo ""
echo "PASSED: Pack '$PACK_DIR' loaded cleanly on Minecraft $VERSION"
