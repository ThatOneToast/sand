# Representing Locations

Minecraft has no single universal “location variable.” Choose a representation based on what you need to do next.

| Representation | Best for | Tradeoff |
|---|---|---|
| Three scores `x/y/z` | integer math, per-player values | converting to coordinates needs score/data plumbing |
| Marker entity | `execute at`, visual/runtime anchor | must summon, tag, and clean up it |
| Storage `{x,y,z}` | global records/config | direct teleport needs extraction/command work |
| Entity tag | find an existing runtime target | not a coordinate snapshot |
| Rust constant | fixed known locations | cannot change at runtime |

## Marker entity: practical default for runtime anchors

```rust
// Raw NBT is limited to marker setup; subsequent targeting is typed selector work.
cmd::raw("summon minecraft:marker ~ ~ ~ {Tags:[\"arcane_last_cast\"]}");
TypedExecute::as_players().at(Selector::all_entities().tag("arcane_last_cast"))
    .run(cmd::say("at marker"));
```

Use a tag unique enough for your feature and remove obsolete markers. Marker entities are particularly good when later commands need `execute at`.

## Storage record

```rust
static SPAWN: StorageVar<SnbtCompound> = StorageVar::new("arcane:data", "spawn");
SPAWN.set_value(SnbtCompound::new().field("x", 0).field("y", 64).field("z", 0));
```

This is ideal for global configuration, not automatic direct teleport. For per-player casting history, prefer a marker per player or score triples unless you deliberately build runtime NBT keying.
