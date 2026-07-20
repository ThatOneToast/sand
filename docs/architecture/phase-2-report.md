# Phase 2 report — export pipeline split into phase modules

Date: 2026-07-19
Branch: `codex/sand-api-compiler-reorganization`
Scope: sand-core only. Behavior-preserving refactor — no generated output,
command string, JSON, ordering, or naming changed.

## What moved

`sand-core/src/component.rs` (4406 lines, 6 public items plus a 1900-line
export driver and ~1450 lines of tests) is now a 21-line public facade over a
new `sand-core/src/compiler/export/` module tree. All public paths
(`sand_core::component::*`, root re-exports, `advanced`) are unchanged.

New module tree (each file carries a `//!` doc naming its pipeline phase):

| File | Lines | Owns |
|---|---|---|
| `compiler/mod.rs` | 8 | compiler root; declares `export` |
| `compiler/export/mod.rs` | 120 | public entry points (`try_export_components*`, `export_components_json`) and `ExportCtx` |
| `compiler/export/pipeline.rs` | 1946 | `try_export_components_impl` — the collection → aggregation → validation → assembly driver |
| `compiler/export/records.rs` | 788 | `ComponentRecord`, `ExportResult`, `component_to_record` (validate-once + version gating) + its contract tests |
| `compiler/export/events.rs` | 733 | event-graph command builders (dispatch/edge/staged/observe functions), custom `SandEvent` backend resolution (#121), trigger validation, XP observation commands |
| `compiler/export/lifecycle.rs` | 191 | private lifecycle/transition path-collision checks and their error constructors |
| `compiler/export/diagnostics.rs` | 186 | final `.mcfunction` string-boundary validation (`validate_function_records`) |
| `compiler/export/dialogs.rs` | 165 | dialog callback drain → `__sand_dialog_init/tick` generation |
| `compiler/export/predicates.rs` | 143 | Sand-owned player-state predicate recognition + JSON |
| `compiler/export/tags.rs` | 99 | function-tag ordering rules (sort user entries, dedupe preserving order) |
| `compiler/export/schedules.rs` | 61 | schedule objective hashing + tick maintenance commands |
| `compiler/export/armor.rs` | 48 | armor watch-map keying + `if items` condition rendering |
| `compiler/export/functions.rs` | 40 | dynamic anonymous function drain + `__sand_local` sentinel resolution |
| `compiler/export/testing.rs` | 35 | test-only record/tag JSON helpers shared by phase tests |

Every `#[cfg(test)]` test moved with the code it tests (record boundary tests
to `records.rs`, command-validation tests to `diagnostics.rs`, dialog/lifecycle/
tag tests to their phases, etc.). The `inventory::submit!` test fixture for
`__test_user_load_after_setup` moved to `tags.rs` with the tag-ordering test
that asserts on it.

Only visibility (`pub(crate)` on formerly file-private helpers) and import
paths changed; function bodies, doc comments, and emission order are verbatim.

## Deliberately not moved

- `try_export_components_impl` (pipeline.rs) remains one large function. Its
  aggregation passes share ~15 accumulators whose interleaving defines output
  order; splitting the function itself risks reordering records, so Phase 2
  moved it whole. A future phase can extract per-dispatch stages behind an
  explicit ordered record sink.
- `function.rs` (817 lines): descriptor/registry types (`FunctionDescriptor`,
  `EventDescriptor`, `EventDispatch`, dyn-fn registry) are macro-facing wiring
  reached via `::sand::__private`; they are declarations consumed by the
  compiler, not compiler internals, and their paths must stay stable.
- `event/` (typed authoring model) and `events/` (graph/dispatch + vanilla
  markers): both paths are public and macro-referenced. `events/graph.rs`
  (1975 lines) is already a single-purpose compiler-internal module (graph
  discovery/normalization/validation); relocating or splitting it would churn
  `crate::events::graph::*` call sites for no ownership gain. Folding the two
  trees into one authoring module is ADR-001 work for a later phase.

## Sizes

Before: `component.rs` 4406 (largest file in sand-core).
After: facade 21; largest new files `pipeline.rs` 1946, `records.rs` 788,
`events.rs` 733. Net +178 lines from module docs/imports/test-module headers.
`function.rs`, `event/`, and `events/` are unchanged.

## Verification

All run at HEAD of this change, all passing:

- `cargo fmt --all` (and `--check` clean)
- `cargo build --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` — 1889 passed, 0 failed
