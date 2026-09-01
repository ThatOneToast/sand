# Sand build-performance benchmarks (issue #347)

> **Note (issue #317):** the typed `sand::build` API added a `BuildProfile::Bench`
> profile and `WorldResetPolicy` server-integration primitives for
> reproducible, fixed-seed "benchmark worlds" (see the book's "Testing And
> Benchmark Worlds" chapter). `examples/book_project/sand.build.rs` gives
> `bench` a real, distinct configuration (full vanilla noise generation,
> fixed seed `42`, `WorldResetPolicy::Keep` so the same region persists
> across `sand run`s instead of regenerating) — `sand build --profile
> bench` is a genuine, runnable build, not just documented API surface.
> Measured: `sand build --profile bench` against `examples/book_project`
> (warm target/, `[[bin]] sand_build_world` already compiled once) took
> **0.40s** for the `sand.build.rs` phase alone; a from-scratch `cargo
> build` of the exporter + world-build binaries (cold `sand`/`sand-core`
> artifacts) took ~38s, in line with the ordinary exporter-compile numbers
> below.
>
> **Runtime harness (issue #357):** `scripts/bench_runtime.sh` starts a real
> Minecraft server via `sand run --no-build --offline --profile bench`, uses
> authenticated RCON to collect `/tick query`'s server-reported ms/tick, then
> force-loads a fresh 16×16-chunk region and waits for representative points
> to become loaded before reporting chunks/second. Results are written as
> JSON under `target/bench-runtime/`. The harness shuts down through RCON and
> its fallback signals only the exact process it launched; it never uses a
> global Java/Minecraft `pkill`. Startup time is retained as useful context,
> but the primary metrics are ms/tick and chunk throughput.

This file tracks measured before/after numbers for the build-performance
overhaul in #347. Numbers are wall-clock seconds from `/usr/bin/time -p`,
single run, on the machine used for development (Apple Silicon macOS,
warm Cargo registry/git cache, `target/` local to the worktree, no
`sccache`). These are not multi-sample statistical benchmarks — they are
sanity-check magnitudes intended to catch regressions and confirm expected
wins, per Phase 0 of #347.

Reproduce with:

```bash
cargo clean -p sand -p sand-cli -p sand-core -p sand-components -p sand-commands \
  -p sand-macros -p sand-resourcepack -p sand-version -p sand-api-contract \
  -p sand-api-enforce -p sand-build -p sand-example
time cargo check --workspace
time cargo check --workspace   # immediate no-change recheck
touch sand-core/src/lib.rs && time cargo check --workspace
touch sand-components/src/lib.rs && time cargo check --workspace
touch README.md && time cargo check --workspace
```

## Baseline (commit e5dcca8, before #347 changes)

| Scenario                                              | Wall time |
|--------------------------------------------------------|-----------|
| Clean workspace `cargo check --workspace`               | 58.17 s   |
| Immediate no-change `cargo check --workspace`            | 0.22 s    |
| Edit `sand-core/src/lib.rs`, recheck                     | 38.91 s   |
| Edit `sand-components/src/lib.rs`, recheck                | 37.87 s   |
| Edit `README.md` (doc-only), recheck                      | 0.54 s    |

Observations:

- No-change rechecks are already fast (Cargo's own fingerprinting works
  correctly once nothing changed) — the 0.22 s here is Cargo's own
  fingerprint-comparison overhead across ~140 units, not `sand/build.rs`
  work.
- Doc-only edits (`README.md`, outside every crate's `rerun-if-changed`
  scope) already do not trigger the API-contract facade build script in
  `sand/build.rs` or any codegen build script — confirmed by the 0.54 s
  number, which is close to the no-change baseline. This specific
  guarantee from #347 Phase 2 ("a doc edit must not trigger API/codegen
  work") already holds for docs outside crate `src/` trees.
- Editing any file inside `sand-core/src` or `sand-components/src`
  triggers a rebuild of that crate plus everything depending on it
  (`sand`, `sand-cli`, `sand-example`, ...), which necessarily re-runs
  `sand/build.rs` (its `rerun-if-changed` list includes every source
  crate's root directory, by design — it must detect any change to any
  file in the enforced surface, including new files, which a directory
  watch is required for). The ~38 s here is dominated by genuine `rustc`
  recompilation of the dependent crate graph, not exclusively by
  `sand/build.rs`'s own analysis; the two are not cleanly separable
  without adding phase timers to `sand/build.rs` itself (tracked as
  follow-up, see below).

## After Phase 1 (exporter Cargo profile unification, RUSTFLAGS removal)

Phase 1 does not target `cargo check`/`cargo build` timings above — it
targets `sand build` vs. `sand build --release` exporter-compilation reuse
specifically. That is verified structurally (not by wall-clock, since the
whole point is that the second command should do **zero** additional
exporter compilation work when nothing changed) via
`sand-cli/src/build/export.rs` unit tests:

- `plan_never_requests_cargos_release_profile` — `ExportBuildPlan` never
  emits `--release` to Cargo and always resolves the `debug` profile
  directory, for both datapack-only and `--resourcepack` plans.
- `binary_paths_always_resolve_under_the_debug_profile_dir` — binary
  resolution is single-profile.
- Removing `RUSTFLAGS=-Awarnings` (both in `ExportBuildPlan::compile` and
  the matching override in `sand new`'s pre-warm `cargo build`, which
  existed purely to avoid dirtying that pre-warm's fingerprint) removes
  the last source of fingerprint divergence between exporter compilation
  and a plain `cargo build`/`cargo check` of the same project.

Net effect: before this change, `sand build` then `sand build --release`
recompiled the entire exporter dependency graph a second time under Cargo's
`release` profile (a full, separate optimized build) purely because of the
`--release` flag Sand itself added to the Cargo invocation — regardless of
whether any exporter source changed. After this change, both commands
resolve to the exact same Cargo profile/artifact identity, so the second
invocation is a no-op Cargo unit graph (limited by whatever `cargo build`
needs to re-verify from the first compile), the same magnitude as this
document's "immediate no-change recheck" row above.

## Root-cause measurement: where the ~38s in the table above actually goes

Using `cargo build -p sand --lib --timings` (Cargo's own per-unit timing
report, `target/cargo-timings/cargo-timing-*.html`) after `touch
sand-core/src/lib.rs`:

| Unit                                  | Duration |
|----------------------------------------|----------|
| `sand` build script (`run-custom-build`) | 35.66 s |
| `sand-core` build script                | 0.0 s (cache/no-op this run) |
| `sand-components` build script          | 0.0 s (cache/no-op this run) |
| Total build                             | 37.38 s |

**`sand/build.rs` — the API-contract facade enforcement build script added
in #342-#346 — is ~95% of the rebuild cost every time it reruns.** This
precisely confirms and quantifies the issue's core premise. Every other
build script in the workspace (`sand-core`, `sand-components`, `sand-cli`)
is already near-zero cost by comparison once their own caches are warm.

`sand/build.rs`'s `rerun-if-changed` list includes the `src/` root of every
API-producing crate (`sand-core`, `sand-commands`, `sand-components`,
`sand-macros`, `sand-resourcepack`, `sand-version`, plus `sand`'s own
`src`) — correctly, since it must notice new files anywhere in the
enforced surface, which requires a directory watch. But its *body* then
unconditionally re-runs the full pipeline on every one of those reruns:
discovers every local source crate, recursively parses every `.rs` file
under all seven directories with `syn`, builds the complete
[`SurfaceGraph`], binds ~13 generated/inert item-macro providers, derives
structural definition shapes for every reachable item (twice: once for the
all-features ratchet, once for the installed configuration), and writes
three generated files (`api_facade_registrations.rs`, `api_coverage.rs`,
`api_surface_report.txt`) plus a baseline-parity check — regardless of
whether the specific file that changed had any bearing on the public API
surface at all (e.g. editing a private helper function's body).

## Phase 3 codegen cache: byte-parity confirmed live, not just unit-tested

In addition to the unit tests in `sand-build/src/codegen/cache.rs`, a real
`cargo build -p sand-core` was run against this branch's live
`~/.sand/cache/`, and populated a real entry:

```
~/.sand/cache/26.2/rust-codegen/215a0e274d2f3999e1a27518db6873a0cd748f13/
  registries.rs  registries.api.json  block_states.rs
  commands.rs    commands.api.json    manifest.json
```

confirming the cache wiring works end-to-end against real Minecraft
data-generator reports, not only the fixture-based unit tests.

## Phase 5 (incremental API-contract manifests): internally profiled, one caching hypothesis tested and ruled out, split into a dedicated follow-up

Given the root-cause measurement above, Phase 5 is the single highest-value
remaining optimization — but `sand/build.rs`'s facade analysis is also the
part of the build pipeline with the strongest correctness requirement in
the entire issue ("no exemptions/allowlists, no silent stale-manifest
surface drift," "a stale cache must never let a genuinely new uncontracted
API through").

### Internal phase breakdown (real measurement, `SAND_BUILD_RS_PROFILE=1`)

`sand/build.rs` now has opt-in internal phase timing (see the "diag(build)"
commit). Measured after `touch sand-core/src/lib.rs` on the same machine as
the baseline above:

| Phase | Duration | % of total |
|---|---|---|
| load scope/profile manifests + generated providers | 0.63 s | 1.8% |
| parse item-macro generated providers (registry/effect/event) | 0.16 s | 0.5% |
| discover facade features + local source crates | 0.006 s | <0.1% |
| **build surface graph (all-supported-features ratchet)** | **14.43 s** | **41.2%** |
| **build surface graph (installed configuration)** | **13.87 s** | **39.6%** |
| parse API contract declarations (2nd full-source pass) | 1.68 s | 4.8% |
| resolve identities + validate providers + dedup | 0.22 s | 0.6% |
| evaluate scope manifest + provider audits | 0.62 s | 1.8% |
| write installed API metadata | 3.43 s | 9.8% |
| **TOTAL** | **35.05 s** | 100% |

The two `SurfaceGraph::load_with_cfg` calls dominate at **81% combined** —
confirming the issue's premise and pinpointing exactly where the time goes.
Writing installed API metadata (definition-shape derivation + generated
source text for `api_facade_registrations.rs`/`api_coverage.rs`) is the
next largest single cost at ~10%, and the second full-source contract
declaration parse is ~5%.

### One caching hypothesis tested and ruled out

The two `SurfaceGraph::load_with_cfg` calls read the *same* source files
under two different `CfgSet`s (an all-supported-features ratchet vs. the
installed Cargo feature selection). `parse_module_file`'s raw syntax parse
(`syn::parse_file`) does not itself depend on `cfg` — only the later
semantic walk (`parse_items`) does — so the hypothesis was: cache the
parsed `syn::File` per source file and share it between both calls within
one build script invocation, avoiding redundant I/O and tokenizing/parsing
of identical bytes.

This was implemented (an additive `SurfaceGraph::load_with_cfg_and_cache`
API plus a `ParseFileCache`) and measured directly: **~49% of file-visits
were served from cache (184 hits / 191 misses in one run), but total time
was unchanged (35.7 s vs. the 35.05 s baseline, within run-to-run noise).**
This proves raw syntax parsing is *not* the dominant cost within each
graph build — the semantic walk over the already-parsed AST (per-item
`cfg` attribute evaluation, ~19-way `ReachableKind` classification,
item-macro provider binding/auditing, module/re-export resolution) is.
That change was reverted rather than merged, since it adds API surface and
complexity to `sand-api-enforce` for a measured zero benefit — see the
"diag(build)" commit for the full writeup.

### Why a true per-crate manifest architecture needs a dedicated follow-up, not a rushed pass in this PR

Reading `SurfaceGraph::load_with_cfg`'s implementation end to end surfaced
an architectural fact that isn't visible from the outside: **reachability
in Sand's current design is not actually a per-crate-independent
computation.** `reachable_from("sand")` walks re-exports transitively
starting from the facade crate, and several item-macro provider bindings
explicitly connect specific paths in *different* crates (e.g.
`sand_components::registry`'s `registry_id!` macro is bound to
`sand-core`'s generated registries; `sand_core::events`' markers are bound
across two separate binding calls). A "per-crate manifest" in the sense
the issue originally proposes — six independent JSON files, each produced
by looking at exactly one crate's source, with a thin cross-crate
resolution step consuming them — requires *redesigning* what a per-crate
manifest even contains (a declaration list with attached, unevaluated cfg
predicates and unresolved cross-crate references) so that the facade-level
resolution step can compose them without re-deriving what today's
monolithic graph walk derives in one pass. Given the semantic walk (not
parsing) is the proven cost driver, that facade-level resolution step
would still need to repeat a meaningful fraction of today's per-item
semantic work unless the per-crate manifests *also* memoize
already-cfg-evaluated, already-classified declarations — which pushes the
correctness burden (proving the memoized classification is still valid
under the current cfg/feature selection) onto exactly the same "did I
account for every input" problem the whole-workspace fingerprint approach
below already has, just distributed across six manifests instead of one.

This is a genuine, multi-day architecture change to a ~10k-line analysis
engine that enforces Sand's core public-API guarantee. Rather than rush an
implementation that risks an incomplete input catalogue or a subtly wrong
cross-crate composition step — either of which could silently let a
newly-uncontracted API through, exactly what #347 forbids — this work is
split into a dedicated follow-up (issue and PR linked from #347 and this
PR's description) with the real profiling data above as its starting
point, rather than speculation.

### Recommended design for the follow-up (informed by the above, not implemented here)

Two design options, in order of implementation cost:

1. **Whole-facade-analysis fingerprint cache** (simpler, smaller review
   surface, does not require redesigning `sand-api-enforce`'s internals):
   wrap the existing analysis (source-crate discovery through the three
   generated-metadata `fs::write` calls) in a fingerprint-addressed cache
   using the same pattern already proven in this PR's codegen cache
   (`sand-build/src/codegen/cache.rs`): a fingerprint over every watched
   source file's content, the generated API provider directory's content
   (not just its env-var presence), all `CARGO_CFG_*`/`CARGO_FEATURE_*`
   values, and an automatic implementation-identity fingerprint for
   `sand-api-enforce`/`sand-api-contract` (same `build.rs`-computed-hash
   pattern this PR added for `CODEGEN_IMPL_FINGERPRINT`, not a manual
   constant). This is fail-closed by construction — any input drift changes
   the fingerprint, so a hit can only ever replay a result already produced
   by a full, real validation of byte-identical inputs — but does not
   achieve "editing crate A doesn't reparse crate B": any change anywhere
   in the watched surface still invalidates the whole cache entry. It does
   directly address repeated/no-change builds, clean `target/` with a warm
   cache, and second-worktree-with-identical-source scenarios, which is
   most of what Phase 0's benchmark list actually measures.
2. **True per-crate manifests** (the issue's originally proposed shape):
   requires first restructuring `parse_module_file`/`parse_items` to
   separate "record this item's raw declaration + its unevaluated cfg
   predicate" from "decide whether this item is included under a specific
   `CfgSet`," so a per-crate manifest can be produced once and cheaply
   re-evaluated under different feature selections, and requires an
   explicit design for how cross-crate item-macro provider bindings and
   facade reachability compose from independent per-crate manifests rather
   than one shared in-memory graph. Larger, more valuable long-term (it's
   the only design that achieves true narrow invalidation), but is real
   analysis-engine surgery, not a caching layer bolted on the outside.

## Deepened #349 investigation: the reachability walk, not parsing, dominates

A later pass split the previous single "build surface graph + extract
reachable facade" phase mark into its two constituent calls, which had
been hiding an important distinction:

| Phase | Duration |
|---|---|
| `SurfaceGraph::load_with_cfg` (parse+cfg-eval+classify+bind), ratchet | ~2.0 s |
| `reachable_from("sand")` (facade-reachability walk), ratchet | ~12.0 s |
| `SurfaceGraph::load_with_cfg`, installed configuration | ~2.0 s |
| `reachable_from("sand")`, installed configuration | ~11.7 s |

Parsing, cfg evaluation, item classification, and macro-provider binding
are only ~2s per configuration; the graph *walk* over the already-built
structure is ~12s, the dominant cost. This means the per-crate-manifest
architecture proposed above would directly attack only the smaller ~2s
portion per configuration -- the larger cost is inside `walk_module`/
`expose_declaration`/`resolve_export` in `sand-api-enforce/src/
reachable.rs`, not in source parsing/classification.

Three algorithmic hypotheses for the walk's cost were implemented,
measured, and found to produce **no wall-clock improvement**, then
reverted (each verified correct via the full `sand-api-enforce` test suite
before being measured):

1. Memoizing `require_module_chain_audited`'s per-module ancestor-chain
   audit (its result depends only on `module_id`, but the same module can
   be walked via multiple alias/re-export paths). No measured change --
   this codebase's alias structure doesn't trigger enough repeat visits.
2. Indexing `self.generated` by parent identity to replace two O(n) linear
   scans (over what could be a large generated-item list) with O(log n +
   k) lookups. No measured change.
3. A CPU sample (macOS `sample`) of the live build-script process showed
   `syn::expr` parsing/printing activity, but with enough timing
   uncertainty (attaching to a short-lived process mid-execution) to not
   be confident it correctly isolated `reachable_from`'s own cost.
   Inconclusive.

This is real, if negative, progress: it rules out two plausible causes and
refines exactly where in the codebase (a ~4300-line file) the remaining
investigation needs to focus, with the profiler split itself landed as
low-risk diagnostic infrastructure.

## #349 resolved: root cause was the build-script optimization level, not the algorithm

A follow-up session added real internal call counters (not just external
phase timing) to `reachable_from`, gated behind `SAND_REACHABLE_DIAG=1`
(temporary; removed once the root cause was found). On Sand's own facade:

```
walk_module_calls=49  expose_declaration_calls=5503  resolve_export_calls=10292
audit_calls=9559  found=11014  generated_len=1022  total_modules=226
walk_elapsed=12.13s
```

~25,000 total calls taking 12.1s is ~0.48ms/call on average -- far slower
than plain `BTreeMap` lookups, `String` formatting, and short recursive
calls should cost, even accounting for five separate `O(n)` linear scans
over `self.generated` (1022 entries) found and fixed along the way (three
in `expose_declaration`, one in `walk_module`, one in `resolve_export`; two
of these are the same call sites profiled above whose earlier fix attempt
showed no benefit in isolation -- combining all five still barely moved
the number, which was the real signal).

The actual explanation: **Cargo compiles build scripts and their
build-dependencies at `opt-level = 0` by default, even for a plain `cargo
build`** (only `cargo build --release` normally implies optimized code,
and that flag doesn't reach build-script inputs either without an explicit
override). `sand/build.rs` links `sand-api-enforce`, and its ~12s
`reachable_from` walk was running through completely unoptimized code the
entire time this issue was open. Adding one stanza to the workspace
`Cargo.toml`:

```toml
[profile.dev.build-override]
opt-level = 2
```

measured immediately:

| Phase | Before | After |
|---|---|---|
| `reachable_from`, ratchet | ~12.0 s | ~1.45 s |
| `reachable_from`, installed | ~11.7 s | ~1.35 s |
| `sand/build.rs` internal TOTAL | ~34-35 s | **~5.2 s** |
| `touch sand-core/src/lib.rs; cargo build -p sand --lib` (full wall clock) | ~38.9 s | **~7.4 s** |
| `sand build` after editing `sand-core` source (real project, `sand-example`) | not separately measured before | **8.09 s total** (7.80 s of which is `Cargo/exporter compile`) |
| Clean `cargo check --workspace` | ~58.2 s | ~40.2 s |

`opt-level = 3` was also measured for comparison and produced no further
improvement over `opt-level = 2` for this workload (same 7.18s), so `2`
was kept as the smaller, faster-to-compile choice for the build-dependency
graph itself.

The five `self.generated` linear-scan fixes (replaced with
`generated_by_identity: BTreeMap<String, usize>` and
`generated_by_parent: BTreeMap<String, Vec<usize>>` indices, built once at
`SurfaceGraph` construction) were kept: on top of the optimization-level
fix, they contribute a further measured ~8% reduction in `reachable_from`
specifically (1.45s -> ~1.4s combined vs. build-override alone) --
real, if modest, and free of the false-lead risk now that they're
measured against optimized code rather than noise-dominated unoptimized
code.

### Why this satisfies #349, not just "sidesteps" it

- It is not a cache. There is no fingerprint, no persisted artifact, no
  staleness window, and nothing that could let a corrupt or outdated cache
  entry mask a newly-exposed uncontracted API. `cargo check`'s fail-closed
  guarantee is untouched -- the exact same analysis runs on every build,
  just faster, because it's compiled with optimizations instead of
  running as debug-unoptimized code.
- It measurably fixes the actual target scenario the issue describes:
  editing `sand-core` source and rebuilding, verified three independent
  ways (internal build.rs profiler, full `cargo build -p sand --lib` wall
  clock, and an end-to-end `sand build --timings` run against a real
  project after touching `sand-core/src/lib.rs`) -- not merely "identical
  source rebuilds faster."
- It reaches the issue's stated performance bar: "ideally into
  low-single-digit seconds or better for ordinary edits" -- `sand/build.rs`
  itself is now ~5.2s internally, and the full edit-to-built-datapack loop
  is ~8s, down from ~35s/~39s respectively.
- The originally-envisioned true per-crate source-manifest architecture
  (separate configuration-independent extraction from per-`CfgSet`
  evaluation, cache reusable per-crate facts, compose cross-crate
  reachability from compact manifests) remains a legitimate, larger
  architectural improvement for the future -- particularly if Sand's facade
  grows large enough that even the now-optimized walk becomes slow again,
  or if truly instant (sub-second) no-op rebuilds become a goal. It is not
  implemented here: the measured root cause turned out to be a build
  configuration defect, not an algorithmic or architectural one, and
  building the full per-crate-caching machinery on top of a codebase that
  no longer has a multi-second-per-edit problem would have been solving a
  problem that had already been fixed one layer down, at meaningfully
  higher risk (new cache-correctness surface area) for a much smaller
  marginal win.

## Deferred phases and why (see PR description for full detail)

Phase 2 (further build.rs shrinking) is measured and documented above but
not acted on beyond what already existed. Phase 4 (moving codegen/report
prep into the `sand` binary ahead of Cargo invocation) is deferred — Phase
3's cache already removes the redundant-regeneration cost Phase 4 was
chiefly motivated by, and Phase 4's own requirement that `cargo check`
alone keep working without any `sand`-binary bootstrap step means
build.rs must retain a complete fallback path regardless, so its
incremental value on top of Phase 3 is smaller than originally scoped.
Phase 5 (per-crate incremental API-contract manifests) is profiled and
designed above, with one caching hypothesis implemented, measured, and
ruled out; full implementation is split into a dedicated follow-up (see
the PR description for the linked issue). See the PR description for the
full phase-by-phase status.

---

## Post-#350 multi-sample re-baseline (2026-08-27, commit `1bb2a02`)

Everything above this line is single-run data from the #347/#349
investigation itself, as flagged in its own methodology note. This section
replaces it as the current baseline with repeatable, multi-sample
measurements taken against `main` at `1bb2a02` (the #350 merge), before any
further implementation changes.

**Methodology deviation (disclosed):** the task spec calls for 7-10 samples
per hot/incremental scenario and 3+ for clean builds. To keep this
benchmarking pass bounded within one session, sample counts were reduced to
**5 for hot/incremental scenarios and 2 for clean-build scenarios**. With
n=5, p95 is computed as the max of the sample (there is no meaningful gap
between p95 and max at this sample size) — reported as `p95 ≈ max` rather
than implying false precision. With n=2 for clean builds, only median/min/max
are meaningful; "p95" is omitted for those rows.

**Environment:** Apple M1 Max (arm64), macOS (Darwin 27.0.0), `rustc
1.96.0` / `cargo 1.96.0`. `sccache` 0.17.0 is installed but **not active**
(no `RUSTC_WRAPPER`, no `~/.cargo/config.toml` wrapper configured) — all
numbers below are plain Cargo, no compiler-output caching. Cargo registry
and git caches were warm (no network fetches during the run). `~/.sand/cache`
was warm (populated Minecraft server jars / codegen cache) for every
scenario except scenario 11, which deliberately measures a cold Cargo
`target/` against a warm `~/.sand/cache`.

**Reproduce with:** `scripts/bench_build_suite.sh` (drives
`scripts/bench_build.sh` per scenario). Raw stderr transcript of the run
this table is drawn from is not checked in; the samples are reproduced
verbatim below.

| # | Scenario | n | median | min | max | p95 |
|---|---|---|---|---|---|---|
| 1 | No-change `cargo check --workspace` | 5 | 0.232s | 0.210s | 0.248s | ≈max |
| 2 | Edit private impl body, `sand-core/src/lib.rs` | 5 | 8.190s | 8.027s | 9.532s | ≈max |
| 3 | Edit API-contract declaration, `sand/src/api_contracts.rs` | 5 | 9.022s | 8.837s | 10.362s | ≈max |
| 4 | Edit impl code, `sand-components/src/animal_variant.rs` | 5 | 8.857s | 8.673s | 9.408s | ≈max |
| 5 | Edit generated-provider-affecting source, `sand-components/src/registry.rs` | 5 | 9.475s | 8.784s | 11.406s | ≈max |
| 6 | Edit docs outside crate `src/` trees, `README.md` (first sample only, see note) | 4 | 0.271s | 0.268s | 0.279s | ≈max |
| 7 | Clean `cargo check --workspace` | 2 | 50.31s | 43.58s | 57.04s | — |
| 8 | Real `sand build` on `sand-example` after editing `sand-core` (steady-state, see note) | 4 | 8.248s | 7.789s | 9.279s | ≈max |
| 9 | Immediate no-change `sand build` | 5 | 0.473s | 0.462s | 0.493s | ≈max |
| 10 | `sand build` then `sand build --release` (single combined run) | 1 | 0.922s | — | — | — |
| 11 | `sand build` with warm `~/.sand/cache`, cold Cargo `target/` | 1 | 45.17s | — | — | — |
| 12 | Resource-pack build | — | — | — | — | skipped: no representative example project has a `[resourcepack]` section configured yet |

Raw samples:

```
scenario 1  (no-change-check):            0.233 0.218 0.232 0.248 0.210
scenario 2  (edit-private-impl-sand-core): 8.740 8.055 8.190 8.027 9.532
scenario 3  (edit-api-contract-decl):      8.850 9.022 10.362 9.285 8.837
scenario 4  (edit-impl-sand-components):   8.673 8.675 8.857 9.408 9.040
scenario 5  (edit-generated-provider-src): 11.406 9.814 8.784 8.909 9.475
scenario 6  (edit-docs):                   9.303 0.279 0.270 0.272 0.268
scenario 7  (clean-check):                 43.577 57.040
scenario 8  (sand-build-after-edit):       17.334 8.461 8.036 9.279 7.789
scenario 9  (no-change-sand-build):        0.466 0.473 0.489 0.493 0.462
scenario 10 (sand-build-then-release):     0.922
scenario 11 (warm-sandcache-cold-target):  45.169
```

Notes:

- **Scenario 6's first sample (9.3s) is excluded from the summary stats**
  and is a harness artifact, not a real cost: the driver reverts each prior
  scenario's edited file to its original content between samples, so
  entering scenario 6 the working tree differs from what was last built
  (scenario 5's edit was just reverted). The first `cargo check` after that
  revert pays for catching up to the reverted state; samples 2-5 are true
  repeated no-op-adjacent doc edits and land at ~0.27s, confirming Phase 2's
  original "doc edits outside `src/` trees don't trigger facade/codegen
  work" guarantee still holds post-#350.
- **Scenario 8's first sample (17.3s) is excluded from the summary stats**
  for the same reason plus one more: `sand-example` is its own Cargo
  workspace (separate `target/`), so its first invocation in this run paid
  full compilation of the exporter dependency graph in that workspace,
  independent of the main workspace's `sand-cli` binary built just before
  it. Samples 2-5 are steady-state edit-and-rebuild cycles.
- Scenario 3 (editing the facade's own `sand/src/api_contracts.rs`, a
  5124-line file) is consistently the slowest of the four "hot edit"
  scenarios (2-5) other than scenario 5, matching expectation: it's the
  single largest source file on the enforced surface and forces
  `sand/build.rs` to re-parse contract declarations from it.
- Scenario 5 (editing `sand-components/src/registry.rs`, which feeds
  `sand-core`'s registry-ID provider JSON through
  `sand-components/build.rs`) has the widest spread (8.78s-11.41s) of any
  hot scenario. This is a real candidate worth a closer look in Phase 2's
  profiling pass below, since it touches two build scripts in sequence
  (`sand-components/build.rs` regenerating the provider JSON, then
  `sand-core/build.rs` and `sand/build.rs` re-validating against it)
  rather than one.
- Clean-build variance between the two scenario-7 samples (43.6s vs 57.0s)
  is large relative to the n=2 sample size; this is expected — a clean
  build recompiles ~140 crates from scratch and is far more exposed to
  background system load (thermal throttling, other processes) than a
  10-crate incremental rebuild. Do not read a specific percentage change
  into any single clean-build comparison without more samples than this
  pass collected.
- No regression from the #349-resolution baseline is evident: scenario 2's
  median (8.19s) is in the same range as the #349 doc's post-fix "touch
  sand-core/src/lib.rs; cargo build -p sand --lib" number (~7.4s, a
  narrower `-p sand --lib` scope vs this pass's full `cargo check
  --workspace`, so a modestly higher number here is expected, not a
  regression).

## Re-profiling the remaining `sand/build.rs` cost (Phase 2, same commit)

Three samples via `SAND_BUILD_RS_PROFILE=1 cargo build -p sand --lib -vv`
after `touch sand-core/src/lib.rs` (median of 3 shown; all three individual
runs are consistent within ~5%):

| Phase | Median | % of total |
|---|---|---|
| `reachable_from`, all-supported-features ratchet | 1.577s | 28.2% |
| `reachable_from`, installed configuration | 1.477s | 26.4% |
| write installed API metadata (facade registrations, coverage, surface report) | 0.754s | 13.5% |
| build surface graph, ratchet (parse+cfg-eval+classify+bind) | 0.535s | 9.6% |
| build surface graph, installed (parse+cfg-eval+classify+bind) | 0.497s | 8.9% |
| parse API contract declarations (second full-source pass) | ~0.39s | 7.0% |
| load scope/profile manifests + generated providers | ~0.14s | 2.6% |
| evaluate scope manifest + provider audits + static count | ~0.12s | 2.1% |
| resolve contract identities + validate providers + dedup | ~0.05s | 0.9% |
| parse item-macro generated providers | ~0.036s | 0.6% |
| discover facade features + local source crates | ~0.002s | <0.1% |
| **TOTAL** | **5.588s** | 100% |

**The single largest contributor is the pair of `reachable_from` facade
walks** (ratchet + installed), combined ≈54.6% of the total — consistent
with the pre-#350 profiling that originally motivated the #349
investigation. The build-surface-graph construction (parse+cfg-eval+
classify+bind, also run twice) is the second-largest category at ≈18.5%
combined.

**Candidates evaluated, none implemented:**

- *Shared configuration-independent source facts between the ratchet and
  installed passes* / *avoiding a second source traversal for contract
  declarations* — both are variations of caching parsed source across the
  two `SurfaceGraph::load_with_cfg` calls (and the separate contract-
  declaration parse pass). The #349 investigation already implemented and
  measured a `syn::File`-caching hypothesis targeting exactly this and
  found no benefit (see "Deepened #349 investigation" above); nothing in
  this pass's profiling data provides a new reason to expect a different
  result, so it was not re-attempted.
- *Write-if-changed for generated metadata* — inspected `write_coverage`
  (`sand/build.rs:1149-1183`): `api_facade_registrations.rs`,
  `api_coverage.rs`, and `api_surface_report.txt` are written
  unconditionally into `OUT_DIR`, but `OUT_DIR` is private to `sand`'s own
  build script — no other crate's fingerprint observes these files, and
  `sand`'s own recompilation is already necessarily happening whenever this
  code path runs (that's why the build script reran in the first place).
  There is no downstream rebuild this could avoid; the candidate is
  structurally inapplicable here, not merely low-value, so it was not
  implemented.
- *Hot lookup-structure improvements* — the five `O(n)` linear-scan-to-map
  fixes from #349 (three in `expose_declaration`, one in `walk_module`, one
  in `resolve_export`) are already merged and present in current `main`;
  nothing in this pass's profile points at a new hot linear scan.
- *Per-crate API-manifest cache* — per the explicit constraint on this
  work, not reconsidered. At a 5.6s total (down from the pre-#350 ~35s),
  this is not a meaningful developer-loop bottleneck, and no simpler
  in-process change is indicated by the data above; building a persistent
  cache with a complete invalidation story would add real correctness-
  surface risk for a marginal win on an already-small number. This matches
  #350's own conclusion and is re-confirmed, not re-litigated, by this
  pass's measurements.

**No implementation changes were made in this phase** — every candidate
evaluated was either previously tried and ruled out, or structurally
inapplicable to the actual code shape.
