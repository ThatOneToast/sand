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
  strings to verify escape-hatch compatibility.

## B. Valid Explicit Escape Hatch

- `sand_components` custom loot, predicate, and item component APIs keep raw
  JSON/SNBT entry points for modded datapacks and features not yet modeled.
- `sand_commands::*_raw` helpers remain valid escape hatches for advanced
  debugging, interop, and future Minecraft syntax.

## C. Stale User-Facing Teaching

- `README.md` still leads with raw `mcfunction!` examples for `tellraw` and
  scoreboards.
- `examples/basic_functions.rs` teaches raw `.mcfunction` strings as the first
  beginner path.
- `examples/README.md` still mentions `cargo install sand` even though the CLI
  is not currently published that way.
- Several rustdoc examples in `sand-core/src/lib.rs`, `sand-core/src/events`,
  and `sand-macros/src/lib.rs` still show raw `tellraw` or scoreboard strings.

## D. Missing Typed API That Forced a Raw String

- Dialog examples currently use `DialogAction::run_command("/function ...")`.
  The command builder exists as `cmd::function(ResourceLocation)`, but dialog
  actions need a typed command-friendly path documented and hardened.
- Event rustdocs use raw score and message commands even though typed state,
  text, selectors, and command builders now cover the normal use cases.

## E. Interop/Modded Examples That Should Stay Raw

- A single interop example should remain under `examples/interop_escape_hatches.rs`
  and `docs/escape-hatches.md`, clearly labeled as an explicit escape hatch for
  another datapack's command/function contract.
