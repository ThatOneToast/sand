# Conditions

Conditions model `execute if` and `execute unless` without writing command
syntax by hand.

```rust
let ready = all![
    MANA.of("@s").gte(25),
    CASTING.of("@s").is_false(),
    any![DASH.ready("@s"), Condition::predicate("example:dash_override")],
];
```

`all!` chains clauses together. `any!` expands into multiple legal execute
plans, including nested `any!` inside `all!`.
