# 3. Selectors And Execute

Selectors identify vanilla targets; execute changes the context in which a command runs.

```rust
TypedExecute::as_players_at_self()
    .run(cmd::tellraw(Selector::self_(), Text::new("Nearby caster").aqua()));
```

Use narrowly scoped selectors for multiplayer safety. [Selectors](../manual/selectors.md) and [Execute](../manual/execute.md) explain target cardinality and chaining.
