# Sand Examples

The `.rs` files in this directory are copyable reference snippets. The project
subdirectories below are standalone crates that build against this checkout.

## Complete projects

| Project | Purpose |
| --- | --- |
| [book_project](book_project/) | Canonical typed gameplay example used throughout the book. |
| [unified_state](unified_state/) | State components, bundles, queries, systems, migrations, and archetype attachment. |
| [rpg_entity](rpg_entity/) | Typed entity archetypes and deterministic export coverage. |
| [participant_audit](participant_audit/) | Live event participants, equipment snapshots, and multiplayer regressions. |
| [portable_counter](portable_counter/) | Rust frontend for the portable-program acceptance counter. |
| [arcane_pack](arcane_pack/) | Advanced integration fixture with primitive scoreboard/storage controls and golden tests. |

Start with `book_project` for new gameplay. `arcane_pack` deliberately retains
lower-level interop coverage; it is not the canonical State authoring model.

## Typed Beginner Path

- `basic_typed.rs` — typed functions and a minimal global `#[derive(State)]` schema.
- `state_and_conditions.rs` — derived player State, flags, cooldowns, and nested `all!`/`any!` conditions.
- `gameplay_state.rs` — boss encounter using enum-backed derived State with a typed bossbar, function handles, and status effects. See the [player state and storage chapter](../book/src/07-state-and-storage.md) and [validated command-media chapter](../book/src/19-validated-command-media.md).
- `dialogs.rs` — typed dialog component with a typed function command action.
- `storage_nbt.rs` — canonical `Nbt` roots, typed paths, and storage-backed conditions.
- `datapack_components.rs` — typed datapack JSON components (dialogs, custom items).
- `spell_system.rs` — small typed gameplay flow using derived State and canonical function handles.

## Advanced And Interop

- `arcane_starter.rs` — complete dogfood datapack: mana, dash cooldown, actionbar, typed execute, storage, dialog, and interop escape hatch. Includes golden tests.
- `interop_escape_hatches.rs` — the only beginner-facing raw command example; intentionally labeled as interop escape hatch.
- `advancements.rs`, `recipes.rs`, `loot_tables.rs`, `predicates.rs`, `custom_items.rs`, `particle_effects.rs`, `player_join.rs` — legacy reference files for specific component types. Useful during migration, but new beginner docs should prefer the typed files above.

## Build a project

From the repository root:

```sh
cd examples/book_project
cargo test
cargo run --manifest-path ../../sand-cli/Cargo.toml --bin sand -- build
```

For a new project, follow [installing and creating a datapack](../book/src/01-installing-and-creating.md).

## Testing

`scripts/check-examples.sh` discovers every standalone project under `examples/`,
runs its tests, and builds a server-ready datapack through the Sand CLI. Both
`scripts/check.sh` and the Rust CI workflow invoke it, so a new example project
receives the same coverage automatically.

The unified-State real-server audit runs in the latest-version vanilla workflow.
Run it locally with Minecraft 26.2 and Java 25:

```sh
python3 scripts/mc_validation/run_unified_state_audit.py --build \
  --jar "$HOME/.sand/cache/26.2/server.jar"
```

Additional attribute and component golden tests live in `sand-example/src/`.
They assert emitted commands and JSON for typed state, conditions, execution,
text, storage, interop, and dialogs.
