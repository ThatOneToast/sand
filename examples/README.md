# Sand Examples

These examples are copyable reference files for a Sand project. They are not
standalone crates by themselves; scaffold a project and paste the relevant code
into `src/lib.rs`.

## Typed Beginner Path

- `basic_typed.rs` - typed function/load/tick basics.
- `state_and_conditions.rs` - scoreboard state, flags, cooldowns, and nested conditions.
- `dialogs.rs` - typed dialog component with a typed function command action.
- `storage_nbt.rs` - typed storage paths and storage-backed conditions.
- `datapack_components.rs` - typed datapack JSON components.
- `spell_system.rs` - small typed gameplay flow tying state, execute, text, and storage together.

## Advanced And Interop

- `interop_escape_hatches.rs` - the only beginner-facing raw command example; it is intentionally labeled as interop.
- Existing legacy reference files cover advancements, recipes, loot tables, particles, custom items, and player join patterns. They are useful during migration, but new beginner docs should prefer the typed files above.

## Quick Start

```sh
cargo run -p sand -- new my_pack
cd my_pack
# edit src/lib.rs using one of these examples
cargo run -p sand -- build
```
