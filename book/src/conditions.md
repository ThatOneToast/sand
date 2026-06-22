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

## Branch semantics

`when(cond)` and `unless(cond)` return builders for conditional branches.

### Single-command — `then_one`

```rust
when(MANA.of("@s").gte(25)).then_one("say enough mana");
// → execute if score @s mana matches 25.. run say enough mana
```

### Grouped branch — `then_all` (safe, recommended)

All commands run inside a generated helper function under one condition check.
Mutations inside the branch do not affect later branch commands.

```rust
when(HAS_CELLS.of("@s").is_true()).then_all(mcfunction![
    cmd::tellraw(Selector::self_(), Text::new("Already granted").red());
    cmd::return_fail();
]);
```

### If/else — `if_`

```rust
if_(HAS_CELLS.of("@s").is_true())
    .then_all(mcfunction![
        cmd::tellraw(Selector::self_(), Text::new("Already have it").red());
        cmd::return_fail();
    ])
    .else_all(mcfunction![
        cmd::attribute_base_set(Selector::self_(), "minecraft:max_health", 40.0);
        HAS_CELLS.enable("@s");
        cmd::return_cmd(0);
    ]);
```

### Per-command wrapping — `then_each` (explicit opt-in)

```rust
when(MANA.of("@s").gte(25)).then_each(["say a", "say b"]);
// Wraps each command separately — NOT safe when commands mutate the condition.
```

## Flag semantics

| Method | Meaning |
|---|---|
| `is_true()` | score = 1 exactly |
| `is_false()` | score = 0 exactly (requires score to exist) |
| `is_not_true()` | score ≠ 1 (matches 0 **and** missing) — recommended for absence |
| `is_set()` | alias for `is_true()` |
| `is_unset()` | alias for `is_false()` |

Use `is_not_true()` for "player does not have this yet" checks, since
scoreboard scores are missing by default.
