# Isolated build measurements

## 2026-11-04 baseline

The isolated suite passed on clean code revision `953edfdee5362be9b73ddb657369d4e7ab7087b0`,
using Minecraft 26.3 and `cargo check --workspace --locked` on an arm64 macOS
host with 10 logical CPUs. The full repository gate and native proofs finished
before measurement; no agent builds or test servers ran concurrently.

| Scenario | Samples | Median | Observed range |
| --- | ---: | ---: | ---: |
| Fresh target, warm external caches | 2 | 60.010 s | 59.728–60.293 s |
| Warm facade comment edit | 5 | 6.979 s | 6.906–7.064 s |
| Restored source, no-op | 5 | 0.730 s | 0.669–0.808 s |

The initial cache warmup and restored-source warmup are excluded from these
summaries. Fresh targets retain warm Cargo/Minecraft caches; this does not measure
first-time downloads. Warm edits touch only a comment in the copied facade.
These small samples do not establish p95, a speedup over historical workloads,
full test/link time, or the cost of changing sand-core. Profile those workloads
before consolidating crates or changing API-contract invalidation.

[Raw results](../../benchmarks/evidence/build-2026-11-04/results.json) retain
commands, versions, environment, timings and original temporary artifact paths.
The [source manifest](../../benchmarks/evidence/build-2026-11-04/tree-manifest.json.gz)
and compressed [sample logs](../../benchmarks/evidence/build-2026-11-04/logs/)
are bundled. The snapshot fingerprint is
`f76b9c8d22adbbcd0d3469ec61193caf9e77255e4399a5668ab515f672e21194`.

`scripts/bench_build_suite.sh` measures Cargo check performance from a copied
snapshot of the current worktree. It never edits the live checkout, invokes
`cargo clean`, or removes Cargo/Minecraft caches. Existing dirty tracked files,
nonignored untracked files and tracked deletions are included. `.git`, `target`
and `dist` directories are excluded. External, dangling and excluded-target
symlinks fail the run; internal links are rebased into the snapshot. Concurrent
source changes during snapshot creation fail rather than producing ambiguous
measurements.

Run after other builds have stopped, using a new output directory outside the
checkout:

```sh
scripts/bench_build_suite.sh --version 26.3 \
  --cold-samples 2 --warm-samples 5 --output /tmp/sand-build-bench-2026-11-03
```

Omitting `--output` allocates a unique temporary directory and prints its path.
Existing output directories are rejected; no previous evidence is overwritten.
`--source` can select another Git checkout root. The command can take several
minutes and retain multiple complete Cargo targets, so provision disk space.

## Scenarios

Every invocation runs `cargo check --workspace --locked` in the snapshot, with
an explicit Minecraft version and strict code generation. Placeholder codegen
is disabled. Each Cargo target belongs to this run; the live checkout's target
directory is never used.

1. **Unmeasured cache warmup:** populate one isolated warm target and prime
   external Cargo and Sand caches. Downloads/codegen misses during this stage
   remain visible in its log and duration, outside measured samples.
2. **Cold target, warm external caches:** use a separate newly created empty
   target for every sample, after warmup. No deletion substitutes for isolation.
3. **Warm facade rebuild:** reuse the warm target while appending a distinct
   comment to the snapshot's canonical `sand/src/lib.rs` for each sample. This
   measures facade source invalidation without depending on a private function,
   a historical line number, or the API-contract file layout.
4. **Unmeasured restore warmup:** restore the facade's original dirty-worktree
   bytes and build once, making the next scenario a true no-op.
5. **No-op:** repeat the identical command with the restored source and warm
   target, without intervening source edits.

Only the selected snapshot source changes (the facade by default), and its original bytes are restored even if a
sample fails. Other worktree edits made after snapshot creation cannot affect
its files. This is source/build-output isolation, not an operating-system sandbox
for arbitrary build scripts. External caches and configured compiler wrappers
remain shared and can affect results; their paths, selected environment settings
and Cargo-config hashes are recorded. Compare runs only when these conditions
and commands match. This suite measures Cargo checks, not `sand build`, release
builds, runtime MSPT, or a complete crate-boundary experiment.

## Artifacts and failures

The output directory contains:

- `results.json`: schema version, pass/failure status, exact command and target
  per sample, monotonic wall time, exit status and log path; measured summaries
  are separate from warmup. Failures retain completed samples and stop later
  scenarios. A missing executable is a recorded exit-127 failure.
- `tree-manifest.json`: sorted paths, file SHA-256, executable bits and normalized
  internal symlink targets. `tree_sha256` hashes this normalized manifest, proving
  the actual selected source bytes independently of the reported Git HEAD.
- `source/`: the retained input snapshot, including the current dirty changes.
- `targets/` and `logs/`: isolated build outputs and one combined stdout/stderr
  log for every invocation, including failed samples and warmup.

Metadata records the Git HEAD and dirty status, Minecraft version, Cargo/Rustc/
Java versions, Python/platform identity, relevant environment settings and
external Cargo-config hashes. Summaries report median/min/max. Nearest-rank p95
is emitted only for at least twenty successful samples; small runs make no
useful tail-latency claim. Historical benchmark tables remain dated evidence and
are not automatically updated by a run.

The compatible single-command entrypoint remains available:

```sh
scripts/bench_build.sh no-change-check 5 -- cargo check --workspace --locked
```

It uses the same timing engine and retains unique JSON/log artifacts, including
the original failing command exit code. It runs the supplied command in the
current directory; use the suite for automatic snapshot isolation.

Focused regression tests use small temporary Git fixtures and fake Cargo
commands, without compiling Sand:

```sh
python3 -m unittest discover -s scripts/tests -p test_bench_build.py
```

## Additional invalidation workloads

The suite accepts `--edit-scope core`, `contracts` or `codegen` to measure a
comment-only edit in sand-core/src/lib.rs, sand/src/api_contracts.rs or
sand-build/src/codegen/mod.rs respectively. The default remains `facade`. Each
scope gets a distinct scenario name and recorded edited path; source restoration,
cold-target isolation and failure artifacts follow the same contract. This measures
source invalidation, not the semantic cost of adding new declarations.

Use `--test-compile` to measure `cargo test --workspace --locked --all-features
--no-run` instead of Cargo check. This includes test compilation/linking without
executing tests. The command and relevant debug/optimization profile environment
overrides are retained in results, so these measurements cannot be confused with
the historical Cargo-check baseline. Run workloads serially after other builds
and native proofs stop; no new speedup or server-performance claim is implied.
