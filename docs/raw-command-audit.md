# Raw Command Teaching Audit

This audit classifies current uses of raw command strings while Sand moves to a
typed Rust-first datapack framework.

## A. Valid Internal Implementation Detail

- `sand-commands/src/**`: command builders and tests assert exact Minecraft
  command serialization.
- `sand-core/src/component.rs`: generated load/tick/event helper functions emit
  concrete command strings as compiler output.
- `sand-core/src/condition.rs` and `sand-core/src/execute_when.rs`: tests assert
  exact `execute` lowering, including nested `any!` expansion.
- `sand-core/src/state/**`: typed state APIs document and test their generated
  scoreboard/storage commands.
- `sand-macros/tests/cases/**`: macro compile tests intentionally use raw
  strings to verify diagnostics and escape-hatch compatibility.

## B. Valid Explicit Escape Hatch

- `sand_components` custom loot, predicate, and item component APIs keep raw
  JSON/SNBT entry points for modded datapacks and features not yet modeled.
- `sand_commands::*_raw` helpers remain valid escape hatches for advanced
  debugging, interop, and future Minecraft syntax.

## C. Cleaned Up In Attribute-First Pass

- `README.md` now starts with `#[component(Load)]`, `#[component(Tick)]`, and
  `#[function]` bodies containing typed command expressions directly.
- `docs/getting-started.md`, `docs/typed-state.md`,
  `docs/typed-commands.md`, and `docs/storage-nbt.md` now teach attribute
  functions before `mcfunction!`.
- `examples/basic_typed.rs`, `examples/state_and_conditions.rs`,
  `examples/spell_system.rs`, `examples/storage_nbt.rs`, and
  `examples/player_join.rs` no longer use raw command strings for normal logic.

## D. Remaining Stale Or Legacy Teaching

- Several rustdoc examples in `sand-core/src/lib.rs`, `sand-core/src/events`,
  and deeper `sand-macros/src/lib.rs` event sections still show legacy raw
  command strings. These should be converted in a focused event-doc pass.
- `examples/arcane_arsenal.html` is a generated/static long-form reference
  page that still contains raw command strings. It should either be regenerated
  from typed examples or moved under a legacy/migration label.

## E. Missing Typed API That Forced a Raw String

- Dialog examples currently use `DialogAction::run_command("/function ...")`.
  The command builder exists as `cmd::function(ResourceLocation)`, but dialog
  actions need a typed command-friendly path documented and hardened.
- Event rustdocs use raw score and message commands even though typed state,
  text, selectors, and command builders now cover the normal use cases.
- `examples/player_join.rs` still uses `cmd::raw("advancement revoke ...")`
  because the beginner prelude does not yet expose a typed advancement
  grant/revoke builder.

## F. Interop/Modded Examples That Should Stay Raw

- A single interop example should remain under `examples/interop_escape_hatches.rs`
  and `docs/escape-hatches.md`, clearly labeled as an explicit escape hatch for
  another datapack's command/function contract.
