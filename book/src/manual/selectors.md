# Selectors

`Selector` and target wrappers generate vanilla selectors without hand-formatting brackets.

```rust
let me = Selector::self_();
let players = Selector::all_players();
let nearby = EntityTargets::nearby(6.0).excluding_players();
```

Use `Selector` for a direct command target and `EntityTargets` where an API needs a potentially-many entity set. Avoid `@e` without filters in multiplayer. Selectors choose targets; [Predicates](predicates.md) describe JSON matching conditions.
