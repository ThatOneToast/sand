# Sand Examples

These example files demonstrate common patterns for building Minecraft datapacks with Sand. They are standalone reference snippets (not runnable on their own) — to run them, scaffold a project with `sand new` and paste the code into your `src/lib.rs`.

## Files

- **`basic_functions.rs`** — `#[function]`, `mcfunction!`, and `#[component(Tick/Load)]` basics.
- **`advancements.rs`** — Custom advancements with triggers, criteria, displays, and rewards.
- **`recipes.rs`** — Shaped, shapeless, cooking, stonecutting, and smithing recipes.
- **`loot_tables.rs`** — Loot tables with pools, conditions, functions, and convenience constructors.
- **`custom_items.rs`** — 1.21+ custom item components (food, tools, equipment, enchantments).
- **`player_join.rs`** — Complete working example: detect player joins, send welcome messages, track visit counts.

## Quick start

```sh
cargo install sand
sand new my_pack
cd my_pack
# Edit src/lib.rs using the examples as reference
sand build
```
