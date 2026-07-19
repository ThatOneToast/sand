# ADR 001 — Public façade, compiler boundary, and crate graph

Date: 2026-07-19
Status: accepted
Scope: the Sand API/compiler reorganization (branch `codex/sand-api-compiler-reorganization`)

## Context

Audits of the current workspace (2026-07-19) found:

- The `sand` package name is currently the **CLI**, while the authoring API lives
  in `sand-core`, which users must import alongside `sand-macros`. A datapack
  author therefore needs two to three internal crates and the book/examples
  import `sand_core`, `sand_macros`, and `sand_commands` directly.
- `sand-core`'s root re-exports mix authoring vocabulary with compiler
  machinery: exporter entry points (`try_export_components*`), drain functions
  (`drain_dyn_fns`, `drain_dialog_callbacks`, `drain_load_commands`,
  `drain_tick_commands`, `define_registered_state`), inventory descriptor
  records (`FunctionDescriptor`, `EventDescriptor`, `ComponentFactory`,
  `StateDescriptor`, …), and `#[doc(hidden)]` re-exports of `inventory` and
  `serde_json`. The `prelude`/`advanced`/`compat` tiers are documentary, not
  enforced, and `sand-core` carries a crate-level `#![allow(deprecated)]`.
- The proc macros in `sand-macros` hardcode `::sand_core::…` (and
  `::sand_resourcepack::…`) expansion paths for ~30 symbols.
- Ten `#[deprecated]` items exist; all but `InventorySlot`/`SlotPattern` are
  unused outside their own defensive tests.
- Fixtures canonically target Minecraft 1.21.4 (CI codegen pin) while
  `LATEST_KNOWN` is 26.2. Golden tests largely assert substrings, not full
  output.
- Documentation exists in three overlapping trees (`docs/`, SUMMARY-linked book
  chapters, orphan book chapters) plus an `ai/` agent-bookkeeping layer enforced
  only by local scripts, not CI.

## Decision

### Target crate graph

```text
sand              NEW: public façade library. The only crate users depend on.
                  Re-exports the authoring API as curated modules + prelude,
                  re-exports the proc macros, and exposes `sand::__private`
                  (doc(hidden)) for macro expansion and compiler wiring.

sand-macros       proc-macro crate (Rust requires a separate crate). Emits
                  `::sand::__private::…` paths. Re-exported by `sand`; users
                  never add it to Cargo.toml.

sand-cli          RENAMED from the current `sand` package. Installable binary
                  is still `sand` ([[bin]] name = "sand"). Owns clap, zip,
                  handlebars, server management. publish-optional; authors'
                  library builds never compile these dependencies.

sand-core         internal implementation (authoring types + compiler/export).
sand-commands     internal implementation (command builders, text, selectors).
sand-components   internal implementation (JSON component builders).
sand-version      internal implementation (version model/profiles).
sand-build        internal implementation (codegen, server jar management).
sand-resourcepack internal implementation (resource pack support; surfaced
                  through `sand` behind a feature if/when stabilized).

sand-vanilla-audit  internal real-server validation fixture (not published).
examples/book_project  NEW: compile-tested example depending only on `sand`;
                       canonical source for book snippets.
```

Dependency direction: `sand-cli → {sand-core, sand-build, sand-components,
sand-resourcepack}`; `sand → {sand-core, sand-macros}` (transitively commands/
components/version); `sand-macros` depends on no workspace crate (path-only
token emission), so there is no cycle. Internal crates set `publish = false`
until a deliberate publishing decision is made; only `sand`, `sand-macros`, and
`sand-cli` are publishable.

### Public API tiers

1. **`sand::prelude`** — common authoring vocabulary (derived from today's
   `sand_core::prelude` minus compiler leaks: `define_registered_state` and the
   deprecated `InventorySlot`/`SlotPattern` are removed; `recently_damaged`
   moves behind `sand::state`/systems modules).
2. **Named modules** — `sand::{event, item, state, command, component, entity,
   data, text, version, vfx}` for the full supported surface.
3. **`sand::advanced`** — deliberate, documented low-level hooks (export entry
   points, raw escape hatches) for framework integrators.
4. **`sand::__private`** — `#[doc(hidden)]`, macro/compiler wiring only
   (descriptor records, `inventory`, drain/register functions). Explicitly not
   a compatibility promise.

Compiler machinery disappears from the crate root: authoring users cannot reach
descriptor records or drain functions without opting into `advanced` or
`__private`.

### Compiler boundary and pipeline

`sand-core`'s export machinery is reorganized (Phase 2) into an explicit
pipeline with per-phase modules under `sand-core/src/compiler/` (collection →
semantic model → lowering/IR → version-aware validation → rendering → datapack
assembly), splitting `component.rs` (4.4k lines) and the `events` modules along
those phases. Behavior is preserved; ownership and phase boundaries become
visible. The two event module trees (`event/` typed model vs `events/`
graph/dispatch) become one authoring-facing module plus compiler-internal graph
code.

### Deprecations

Crate-level `#![allow(deprecated)]` is removed. The eight unused `#[deprecated]`
items are deleted. `InventorySlot`/`SlotPattern` (and
`if_items_pattern`/`unless_items_pattern`) are removed in favor of `ItemSlot`;
`compat::TypedEvent` is removed. No historical aliases are carried into the new
`sand` façade — it starts clean.

### Test target

Canonical fixtures, examples, and the book project target **Minecraft Java
26.2**. 1.21.4 remains only as an explicit oldest-profile/compatibility
boundary where a rendering branch exists. Golden tests compare full ordered
output, not substrings.

### Documentation

`book/` (SUMMARY-linked) is the single user documentation tree; `docs/` keeps
only internal architecture/compiler/testing docs (this ADR's directory). The
`ai/` layer, `AGENTS.md`, `llms.txt`, orphan book chapters, and finished audit
reports are removed after migrating real limitations into rustdoc/book/tests;
`scripts/check-ai-resources.py` is deleted and `check-docs.py` pruned.

## Consequences

- Users add one dependency (`sand`) and one import (`use sand::prelude::*`).
- All macro users must go through `sand` (expansion paths change to
  `::sand::__private`). Internal fixtures (`sand-example` replacement,
  `sand-vanilla-audit`, book project) migrate accordingly.
- Breaking change for anyone importing `sand_core`/`sand_macros` directly.
  Sand is pre-1.0 and volatile; no migration shims are kept (per project
  policy).
- Architecture tests (workspace `tests/`) guard the boundary: façade-only
  compile tests, no internal-crate imports in examples/book, no CLI deps in
  author builds, deterministic 26.2 export.
