#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

# Compile every harness; discover regressions so new test modules cannot silently
# miss either the local or CI gate. Native server proofs remain separate.
python3 -m compileall -q scripts
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 -m unittest discover -s scripts/mc_validation -p 'test_*.py'
