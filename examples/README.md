# Sand Examples

These examples are copyable reference files for a Sand project. They are not
standalone crates; scaffold a project and paste the relevant code into
`src/lib.rs`.

## Typed Beginner Path

- `basic_typed.rs` — typed function, load, and tick basics with `ScoreVar`, `#[component(Load)]`, `#[component(Tick)]`.
- `state_and_conditions.rs` — scoreboard state, flags, cooldowns, and nested `all!`/`any!` conditions.
- `dialogs.rs` — typed dialog component with a typed function command action.
- `storage_nbt.rs` — typed `StorageVar<T>`, storage paths, and storage-backed conditions.
- `datapack_components.rs` — typed datapack JSON components (dialogs, custom items).
- `spell_system.rs` — small typed gameplay flow: load, tick, named function, cooldown, conditions, actionbar, storage.

## Advanced And Interop

- `arcane_starter.rs` — complete dogfood datapack: mana, dash cooldown, actionbar, typed execute, storage, dialog, and interop escape hatch. Includes golden tests.
- `interop_escape_hatches.rs` — the only beginner-facing raw command example; intentionally labeled as interop escape hatch.
- `advancements.rs`, `recipes.rs`, `loot_tables.rs`, `custom_items.rs`, `particle_effects.rs`, `player_join.rs` — legacy reference files for specific component types. Useful during migration, but new beginner docs should prefer the typed files above.

## Quick Start

```sh
cargo run -p sand -- new my_pack
cd my_pack
# edit src/lib.rs using one of these examples
cargo run -p sand -- build
```

## Testing

Golden tests for the attribute-first authoring patterns live in
`sand-example/src/attribute_golden.rs`. They assert exact command strings and
JSON output for typed state, conditions, execute chains, text, storage, raw
interop, and dialog components.
