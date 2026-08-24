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

## Phase 5 (incremental API-contract manifests): measured, designed, deliberately not implemented in this PR

Given the root-cause measurement above, Phase 5 is the single highest-value
remaining optimization — but `sand/build.rs`'s facade analysis is also the
part of the build pipeline with the strongest correctness requirement in
the entire issue ("no exemptions/allowlists, no silent stale-manifest
surface drift," "a stale cache must never let a genuinely new uncontracted
API through"). Implementing a cache/short-circuit around it requires
*exhaustively* cataloguing every input that can change its output — a
partial catalogue that misses one input class is exactly the failure mode
#347 explicitly forbids, and is worse than not caching at all.

Auditing `sand/build.rs` (1164 lines) for this PR found the analysis is
sensitive to at least four independent input classes, only some of which
are already declared as `rerun-if-*` triggers:

1. **Source content** of every file already covered by the existing
   `rerun-if-changed` directory watches (`sand`, `sand-core`,
   `sand-commands`, `sand-components`, `sand-macros`, `sand-resourcepack`,
   `sand-version` — all `src/`), plus `api-scopes.toml`,
   `api-surface-profiles.toml`, and the selected baseline file
   (`api-surface-baseline*.txt`).
2. **Generated API provider directory** content
   (`DEP_SAND_CORE_API_PROVIDER_DIR`) — currently only its *presence* is a
   `rerun-if-env-changed` trigger (the env var pointing at the directory),
   not its content; sand-core's own codegen fingerprint (Phase 3) already
   captures this on sand-core's side, but sand's facade build script reads
   the directory contents directly and would need its own content hash.
3. **`CARGO_CFG_*`/`CARGO_FEATURE_*` environment variables** — the facade
   build script reads *all* `CARGO_CFG_*` vars (`cargo_cfg()`, line ~334)
   to reconstruct the effective `cfg(...)` set for the analysis, and
   `CARGO_FEATURE_*` vars (`enabled_cargo_features()`, line ~287) for the
   installed feature configuration. Cargo already reruns the build script
   when these change as part of its own fingerprinting, but a *cache*
   keyed globally (not per-`OUT_DIR`) needs them folded into the
   fingerprint explicitly to avoid conflating two different
   target/feature configurations under one cache entry.
4. **Enforcement engine logic itself** (`sand-api-enforce`,
   `sand-api-contract`) — needs an explicit schema/impl-version salt (the
   same pattern as `codegen::CODEGEN_SCHEMA_VERSION` added for Phase 3 in
   this PR) so a bug fix or new check in the enforcement crates invalidates
   every cached facade result, not just entries whose watched source also
   happened to change.

**Recommended design** (not implemented here, left for a dedicated
follow-up PR with its own focused review): wrap the existing analysis
(`source_crates` discovery through the three `fs::write` calls around line
1070-1104) in a fingerprint-addressed cache using the exact same pattern
already proven in this PR for the codegen cache
(`sand-build/src/codegen/cache.rs`): compute a fingerprint from all four
input classes above using `sand_build::fingerprint::{hash_bytes,
combine}`, check a cache entry under (e.g.)
`~/.sand/cache/api-facade/<fingerprint>/` containing the three generated
output files, copy-and-return on a validated hit, else run the existing
analysis unchanged and publish its output. This is a **deliberate,
documented simplification** of the issue's proposed *per-crate* manifest
architecture (six independent `sand-core.json`/`sand-components.json`/...
manifests consumed by the facade) in favor of one whole-input fingerprint
around the single existing consolidated analysis: it does not achieve the
issue's stated "editing an unrelated file does not cause a full
source-tree contract parse" for edits *within* the seven watched crates
(any content change there still invalidates the one cache entry and
triggers a full reparse, same cost as today), but it directly and safely
addresses the scenarios Phase 0's own benchmark list emphasizes most —
repeated/no-change builds, clean `target/` with a warm cache, and a second
worktree checkout with byte-identical source at different mtimes — without
the substantially larger design and review surface of a true per-crate
manifest split. Because the fingerprint is a strict superset of every
`rerun-if-*` trigger already declared plus the additional inputs identified
above, it is fail-closed by construction: it can only ever return a result
previously produced by a full, real validation of byte-identical inputs,
so it cannot let a newly uncontracted API through.

This was deliberately **not implemented in this PR** because verifying the
four-input-class catalogue above is exhaustive — with no fifth input
missed — needs dedicated review time this PR's scope didn't allow, and an
incomplete catalogue here is strictly worse than the current
always-correct-but-slow behavior. Landing it requires, at minimum: a second
pass specifically hunting for any remaining input to `sand/build.rs`'s
analysis (ideally via a change-detection fuzzer that mutates one input
class at a time and confirms the fingerprint always changes), and the
concurrency/corruption tests already established as the pattern in
`sand-build/src/codegen/cache.rs`.

## Deferred phases and why (see PR description for full detail)

Phase 2 (further build.rs shrinking) is measured and documented above but
not acted on beyond what already existed. Phase 4 (moving codegen/report
prep into the `sand` binary ahead of Cargo invocation) is deferred — Phase
3's cache already removes the redundant-regeneration cost Phase 4 was
chiefly motivated by, and Phase 4's own requirement that `cargo check`
alone keep working without any `sand`-binary bootstrap step means
build.rs must retain a complete fallback path regardless, so its
incremental value on top of Phase 3 is smaller than originally scoped.
Phase 5 (per-crate incremental API-contract manifests) is measured,
designed, and deliberately not implemented — see above. See the PR
description for the full phase-by-phase status and follow-up plan.
