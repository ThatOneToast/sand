# Selector Arity

Raw `Selector` remains available for compatibility, but new typed APIs use
arity wrappers:

- `SingleEntity` — exactly one entity, such as `@s`
- `EntityTargets` — many entities, such as `@e[...]`
- `SinglePlayer` — exactly one player
- `PlayerTargets` — many players, such as `@a`

Examples:

```rust
let self_entity = SingleEntity::self_();
let nearby = EntityTargets::nearby(5.0)
    .excluding_players()
    .excluding_self();
let nearest = EntityTargets::all().entity_type("minecraft:zombie").nearest();
```

Commands that require one entity can require `SingleEntity`. Commands that
support many targets can accept `EntityTargets` or `PlayerTargets`. This moves
vanilla parser arity rules into Rust types.
