# Selector Arity

New typed command APIs distinguish selector arity:

- `SingleEntity`
- `EntityTargets`
- `SinglePlayer`
- `PlayerTargets`

```rust
let target = SingleEntity::self_();
let nearby = EntityTargets::nearby(5.0)
    .excluding_players()
    .excluding_self();
let nearest = EntityTargets::all().entity_type("minecraft:zombie").nearest();
```

Use typed wrappers when a command has vanilla target rules. `Selector` remains
available for older APIs and lower-level builders.
