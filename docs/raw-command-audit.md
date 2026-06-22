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
- `cmd::raw(...)` remains the explicit escape hatch for interop, modded
  commands, snapshot syntax, future features, and debugging.

## C. Cleaned Up In Attribute-First Pass

- `README.md` starts with `#[component(Load)]`, `#[component(Tick)]`, and
  `#[function]` bodies containing typed command expressions directly.
- `docs/getting-started.md`, `docs/typed-state.md`,
  `docs/typed-commands.md`, and `docs/storage-nbt.md` teach attribute
  functions before `mcfunction!`.
- `examples/basic_typed.rs`, `examples/state_and_conditions.rs`,
  `examples/spell_system.rs`, `examples/storage_nbt.rs`, and
  `examples/player_join.rs` use typed command builders throughout.
- Event rustdocs (`sand-core/src/events/mod.rs`) use typed command builders.
- Macro event docs (`sand-macros/src/lib.rs`) use typed command builders.
- Crate-level rustdoc (`sand-core/src/lib.rs`) shows typed-first usage.
- Scaffold template generates attribute-first typed code.

## D. Remaining Stale Or Legacy Teaching

- `examples/arcane_arsenal.html` is a generated/static long-form reference
  page that still contains raw command strings. It should either be regenerated
  from typed examples or moved under a legacy/migration label.

## E. Typed APIs Now Available

- `cmd::advancement_grant_only(targets, advancement)` — generated typed builder
- `cmd::advancement_revoke_only(targets, advancement)` — generated typed builder
- `cmd::advancement_grant_everything(targets)` — generated typed builder
- `cmd::advancement_revoke_everything(targets)` — generated typed builder
- `cmd::recipe_give(targets)`, `cmd::recipe_give_2(targets, recipe)` — generated
- `cmd::recipe_take(targets)`, `cmd::recipe_take_2(targets, recipe)` — generated
- `cmd::function(id)` — free function in builtins
- `cmd::give(selector, item)` — free function accepting typed items via `Into<String>`
- `Sound::play(event).to(selector)` — typed sound builder
- `cmd::tag_add(selector, tag)`, `cmd::tag_remove(selector, tag)` — builtins
- `cmd::effect_give(selector, effect, duration, amplifier)` — builtin

## F. Interop/Modded Examples That Should Stay Raw

- A single interop example remains under `examples/interop_escape_hatches.rs`
  and `docs/escape-hatches.md`, clearly labeled as an explicit escape hatch for
  another datapack's command/function contract.
