#!/usr/bin/env bash
set -euo pipefail

if ! command -v mdbook >/dev/null 2>&1; then
  echo "mdbook is not installed. Install with: cargo install mdbook" >&2
  exit 127
fi

mdbook build
python3 scripts/check-docs.py
