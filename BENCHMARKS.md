# Sand build-performance benchmarks (issue #347)

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
