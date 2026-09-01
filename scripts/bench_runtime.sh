#!/usr/bin/env bash
# Real-server runtime benchmark for BuildProfile::Bench worlds (issue #357).
#
# This is a v1, honestly-scoped runtime metric: wall-clock time from
# launching a real Minecraft server (via `sand run`) against a
# `bench`-profile world to that server reporting ready (`sand run`'s own
# "Minecraft <version> ready" console-health classification, see
# sand-cli/src/console/health.rs and render.rs). The `bench` profile
# (examples/book_project/sand.build.rs) uses full vanilla noise generation
# with a fixed seed, so this captures a real, reproducible cost: server
# boot + spawn-chunk generation for a genuine (non-flat) world.
#
# What this does NOT measure (left for a future follow-up, not claimed
# here): steady-state TPS, chunk-generation throughput away from spawn, or
# any other in-game tick-rate/runtime-performance metric. Those need a
# scripted in-game workload (e.g. teleporting/exploring to force chunk
# generation and sampling `/forge tps`-equivalent or debug output) that
# this v1 harness does not attempt.
#
# Requires: a real JDK on PATH and network access on first run (to
# download the Minecraft server jar into ~/.sand/cache/<version>/, unless
# already cached).
#
# Usage: scripts/bench_runtime.sh [samples] [project-dir]

set -uo pipefail

samples="${1:-1}"
project_dir="${2:-examples/book_project}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sand_bin="$repo_root/target/debug/sand"

if ! command -v java >/dev/null 2>&1; then
  echo "error: no java on PATH — cannot run a real server for this benchmark" >&2
  exit 1
fi

if [[ ! -x "$sand_bin" ]]; then
  echo "building sand-cli (debug)..." >&2
  (cd "$repo_root" && cargo build -p sand-cli) >&2 || exit 1
fi

cd "$repo_root/$project_dir" || exit 1

echo "building bench-profile world..." >&2
"$sand_bin" build --profile bench >&2 || exit 1

times=()
for ((i = 1; i <= samples; i++)); do
  rm -rf dist/server
  log="/tmp/bench_runtime_sample_${i}.log"
  start=$(date +%s.%N)
  "$sand_bin" run --no-build --offline --profile bench >"$log" 2>&1 &
  pid=$!

  ready=""
  # Poll for the ready line rather than trusting a fixed sleep; give a
  # generous ceiling since first-run jar download/unpack can be slow.
  for _ in $(seq 1 240); do
    if grep -qE "Minecraft .* ready" "$log" 2>/dev/null; then
      ready="1"
      break
    fi
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    sleep 0.5
  done
  end=$(date +%s.%N)

  # Stop the server cleanly, then make sure nothing lingers.
  if kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null
  fi
  pkill -9 -f "java -Xmx.*-jar .*server\.jar nogui" 2>/dev/null
  wait "$pid" 2>/dev/null

  if [[ -z "$ready" ]]; then
    echo "sample $i FAILED to reach ready state; see $log" >&2
    tail -n 40 "$log" >&2
    exit 1
  fi

  elapsed=$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.2f", e - s }')
  times+=("$elapsed")
  echo "  [bench-runtime] sample $i/$samples: ${elapsed}s (log: $log)" >&2
done

sorted=($(printf '%s\n' "${times[@]}" | sort -n))
n=${#sorted[@]}
min=${sorted[0]}
max=${sorted[$((n - 1))]}
if ((n % 2 == 1)); then
  median=${sorted[$((n / 2))]}
else
  a=${sorted[$((n / 2 - 1))]}
  b=${sorted[$((n / 2))]}
  median=$(awk -v a="$a" -v b="$b" 'BEGIN { printf "%.2f", (a + b) / 2 }')
fi

echo ""
echo "### bench-profile server-ready wall time (n=$n)"
echo ""
echo "samples: ${times[*]}"
echo ""
echo "| median | min | max |"
echo "|---|---|---|"
echo "| ${median}s | ${min}s | ${max}s |"
