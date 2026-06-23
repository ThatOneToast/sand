# Runtime Entities And Tags

Entities are runtime Minecraft objects. Selectors find them; tags provide a stable grouping/identity convention; `execute as` changes who `@s` is and `execute at` changes coordinates.

```rust
let altar = Selector::all_entities().tag("arcane_altar");
TypedExecute::as_players().at(altar).run(cmd::say("altar context"));
```

Use tags for marker entities, interactables, summoned mobs, and cleanup groups. Item `custom_data` identifies an item stack; it is not an entity tag. Entity NBT can be read/written with data commands, but fields are version-sensitive and player NBT is not a general-purpose safe database. Prefer scoreboards, tags, and dedicated marker entities for gameplay state.

<div class="sand-warning"><strong>Selector safety.</strong> A tag may match zero, one, or many entities. Add distance/type/limit constraints when a command expects one source or one destination.</div>
