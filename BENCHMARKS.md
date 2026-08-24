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

## Deferred phases and why (see PR description for full detail)

Phases 2 (build.rs shrink beyond the doc-edit guarantee already measured),
3 (generated-Rust content-addressed cache), 4 (moving codegen prep into the
`sand` binary), 5 (per-crate incremental API-contract manifests), 6
(sccache support), 7 (incremental datapack/resource-pack output writing),
and 8 (`--explain-rebuild`) are not implemented in this change. They are
substantial, mostly-greenfield subsystems (a ~10k-line existing contract
enforcement engine in `sand-api-enforce`, and no existing fingerprint-cache
infrastructure in `sand-build` to build Phase 3/5 on top of) that the issue
itself asks to be delivered as "small measured phases" rather than one
pass. Attempting them within this change's time budget without dedicated
review/testing would risk exactly what #347 explicitly forbids: silently
weakening API-contract enforcement or introducing a cache that can go
stale. See the PR description for a concrete, scoped follow-up plan per
phase.
