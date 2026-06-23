# 4. Conditions And Branching

Conditions lower into legal `execute if` plans. Branch helpers generate private helper functions automatically.

```rust
when(all![MANA.of("@s").gte(20), CASTING.of("@s").is_false()]).then_all([
    MANA.remove("@s", 20), cmd::say("cast"),
]);
```

Keep branch bodies small; Sand registers and exports their helper functions. See [Conditions](../manual/conditions.md) and [Function References](../manual/function-refs.md).
