# Final Sync Audit

Audit of every user-facing surface for alignment with Sand's settled direction:
attribute-first typed Rust authoring with `#[function]`, `#[component(Load)]`,
`#[component(Tick)]` as the primary API. Raw strings only as explicit escape
hatches through `cmd::raw(...)`.

## A. Correct internal implementation/test output

All `sand-commands/`, `sand-core/src/state/`, `sand-core/src/condition.rs`,
`sand-core/src/execute_when.rs`, `sand-core/src/cmd/`, `sand-core/src/component.rs`,
`sand-macros/tests/cases/`, and most book/docs content correctly use raw MC function
strings in implementation code and tests. These are the *output* of typed builders,
not teaching raw syntax.

## B. Correct explicit escape hatch

Raw command strings appear in the following places as correctly labeled escape hatches:

- `README.md` lines 222-223: labeled interop example
- `docs/escape-hatches.md` lines 19-20: dedicated escape hatch docs
- `docs/authoring-model.md` lines 77-78: interop examples
- `examples/interop_escape_hatches.rs` entire file: intentionally labeled
- `book/src/examples/interop.md` lines 8, 12: interop examples
- `book/src/examples/spell-system.md` line 49: one escape hatch in spell system
- `book/src/advanced/escape-hatches.md` entire file: dedicated advanced docs
- `sand-core/src/cmd/mod.rs` line 89: `cmd::raw()` function docs
- `sand-components/src/item/mod.rs` lines 744, 969: raw component escape hatch
- `sand-commands/src/text.rs` line 171: raw JSON text escape hatch
- `sand-macros/src/lib.rs` lines 60-68: documentation of escape hatch usage

## C. Stale teaching (raw strings used as normal examples)

### `sand-core/src/events/mod.rs` (22 occurrences)

Event rustdoc examples use `mcfunction!` with raw command strings instead of
typed APIs. Lines: 27, 32, 38, 64, 118, 152, 168-169, 187-188, 206, 233, 239,
253, 276, 281, 304, 331, 352, 421, 567, 644, 677, 881.

These should use typed command builders (e.g. `cmd::tellraw(...)` instead of
`mcfunction! { r#"tellraw @s {"text":"..."}"# }`).

### `sand-core/src/lib.rs` line 28-30

Crate-level rustdoc shows `mcfunction!` with raw JSON tellraw as the primary
usage example. Should show attribute-first typed code.

### `sand-macros/src/lib.rs` event macro docs

Lines 621, 665, 1440, 2336, 2342: event/run_fn/armor_event rustdoc uses raw
scoreboard strings instead of typed state.

### `examples/arcane_arsenal.html`

Generated/static reference page with ~20+ occurrences of raw scoreboard/tellraw/
execute syntax. Should either be regenerated with typed examples or clearly
marked as a generated reference document.

## D. Missing typed API forcing raw syntax

### `examples/player_join.rs` lines 67, 69

Uses `cmd::raw("advancement revoke ...")` because typed advancement revoke
builder is not yet exposed. This is the only place a raw command is forced by
a missing typed API.

### `docs/raw-command-audit.md` line 53

Audit self-identifies this gap.

## E. Rustdoc that needs updating

### `sand-core/src/lib.rs` line 11

Crate-level doc lists `mcfunction!` as a top-level feature alongside typed APIs.
Should reposition as advanced tooling.

### `sand-macros/src/lib.rs` line 57

Proc-macro doc lists `mcfunction!` as a primary path. Should note it as advanced.

## F. Scaffold/template drift

### `sand/src/templates/default/src_lib_rs.hbs` lines 18-24

New-project scaffold teaches raw `tellraw @a` and `playsound minecraft:` inside
`mcfunction!` as the hello-world default. Should use attribute-first typed code:
`cmd::tellraw(Selector::all_players(), Text::new("...").gold().bold(true))`.

## G. Valid historical note

### `README.md` line 30

Documents that the CLI is not published to crates.io yet. This is accurate current
state, not stale content.

## Summary of findings

| Category | Count | Files affected |
|---|---|---|
| A. Correct implementation | ~100+ | commands, state, condition, tests |
| B. Correct escape hatch | ~20 | README, docs, examples, book, cmd/mod.rs |
| C. Stale teaching | ~35 | events/mod.rs, lib.rs, macros/lib.rs, arcane_arsenal.html |
| D. Missing typed API | 1 | player_join.rs (advancement revoke) |
| E. Rustdoc update needed | 2 | core/lib.rs, macros/lib.rs |
| F. Scaffold drift | 1 | src_lib_rs.hbs |
| G. Historical note | 1 | README.md (cargo install) |

## Synced surfaces (already correct)

- README.md main examples: typed-first with `#[function]`, `#[component(Load)]`, `#[component(Tick)]`
- `docs/authoring-model.md`: correctly positions `mcfunction!` as advanced
- `docs/getting-started.md`: correctly positions `mcfunction!` as advanced
- `book/src/getting-started.md`: correctly positions `mcfunction!` as advanced
- `book/src/authoring-model.md`: correctly positions `mcfunction!` as advanced
- `examples/basic_typed.rs`: fully typed
- `examples/spell_system.rs`: fully typed
- `examples/state_and_conditions.rs`: fully typed
- `examples/dialogs.rs`: fully typed
- `examples/storage_nbt.rs`: fully typed
- `examples/interop_escape_hatches.rs`: intentionally raw, labeled as interop

## Stale usages removed in this pass

- Scaffold template: rewritten to attribute-first typed code
- Event rustdocs: rewritten to use typed command builders
- Crate-level rustdoc: rewritten to show typed-first usage
- Proc-macro rustdocs: rewritten to show typed-first usage

## Missing typed APIs discovered

- `cmd::advancement(Selector).grant(...)` / `.revoke(...)` — needed by player_join
- (ItemStack builder exists but needs hardening — see Part 5)

## Docs/examples/templates synced

- `sand/src/templates/default/src_lib_rs.hbs` — rewritten
- `sand-core/src/events/mod.rs` — event examples rewritten
- `sand-core/src/lib.rs` — crate-level doc rewritten
- `sand-macros/src/lib.rs` — event/armor_event/run_fn examples rewritten
- `examples/README.md` — updated to reflect examples
- All book pages — already aligned

## Intentionally deferred

- `examples/arcane_arsenal.html` — generated/static reference page; left as-is since it documents the framework's full API surface and is not a beginner teaching surface
- `player_join.rs` advancement revoke — blocked on typed advancement builder API
