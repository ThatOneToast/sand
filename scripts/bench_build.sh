#!/usr/bin/env bash
# Multi-sample build-performance benchmark harness for BENCHMARKS.md.
#
# Runs each scenario N times, capturing wall-clock seconds per sample, and
# prints per-scenario median/min/max/p95 (Markdown-ready). See BENCHMARKS.md
# for the methodology this supports and the sample-count deviation (5
# hot/incremental samples, 2 clean-build samples) from the nominal spec.
#
# Usage: scripts/bench_build.sh <scenario-name> <samples> -- <command...>
#
# The command is run once per sample; wall time is measured with the shell's
# builtin `time` (TIMEFORMAT set to emit just real-seconds).

set -euo pipefail

if [[ $# -lt 4 || "$3" != "--" ]]; then
  echo "usage: $0 <scenario-name> <samples> -- <command...>" >&2
  exit 1
fi

name="$1"
samples="$2"
shift 3
cmd=("$@")

times=()
for ((i = 1; i <= samples; i++)); do
  TIMEFORMAT='%R'
  t=$( { time "${cmd[@]}" >/tmp/bench_build_out.log 2>&1; } 2>&1 )
  status=$?
  if [[ $status -ne 0 ]]; then
    echo "sample $i FAILED (exit $status); see /tmp/bench_build_out.log" >&2
    tail -n 40 /tmp/bench_build_out.log >&2
    exit $status
  fi
  times+=("$t")
  echo "  [$name] sample $i/$samples: ${t}s" >&2
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
  median=$(awk -v a="$a" -v b="$b" 'BEGIN { printf "%.3f", (a + b) / 2 }')
fi

p95_idx=$(awk -v n="$n" 'BEGIN { i = int(0.95 * (n - 1) + 0.5); print i }')
p95=${sorted[$p95_idx]}
if ((n < 5)); then
  p95_note=" (n=$n, p95 not meaningful — approx max)"
else
  p95_note=""
fi

echo ""
echo "### $name (n=$n)"
echo ""
echo "samples: ${times[*]}"
echo ""
echo "| median | min | max | p95 |"
echo "|---|---|---|---|"
echo "| ${median}s | ${min}s | ${max}s | ${p95}s${p95_note} |"
