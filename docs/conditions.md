# Conditions

Conditions are typed values that lower into valid execute plans. Use them with
`TypedExecute`, `when`, `unless`, or `if_`.

```rust
use sand_core::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

let ready = all![
    MANA.of("@s").between(25, 100),
    CASTING.of("@s").is_false(),
    any![DASH.ready("@s"), Condition::predicate("example:can_dash")],
];
```

Nested `any!` inside `all!` expands to multiple complete execute commands.

## Flag semantics

| Method | Lowering | Meaning |
|---|---|---|
| `is_true()` | `if score … matches 1` | score is exactly 1 |
| `is_false()` | `if score … matches 0` | score is exactly 0 |
| `is_not_true()` | `unless score … matches 1` | score is not 1 (missing or 0) |
| `is_set()` | alias for `is_true()` | — |
| `is_unset()` | alias for `is_false()` | requires exact 0, not missing |

**Use `is_not_true()` for absence checks.** Scoreboard scores are missing by
default. `is_false()` requires the score to exist and equal 0, so it does not
match players who have never been assigned the flag.

```rust
// Matches both "flag = 0" and "flag not set" (recommended):
when(HAS_CELLS.of("@s").is_not_true()).then_one("say needs setup");

// Matches only "flag = 0" exactly:
when(HAS_CELLS.of("@s").is_false()).then_one("say explicitly false");
```

## Single-command branches — `then_one`

```rust
when(MANA.of("@s").gte(25)).then_one("say enough mana");
// → execute if score @s mana matches 25.. run say enough mana

unless(CASTING.of("@s").is_true()).then_one("say not casting");
// → execute unless score @s casting matches 1 run say not casting
```

## Grouped branches — `then_all` (safe multi-command)

`then_all` generates a helper function that runs all commands once under the
condition. The condition is evaluated **once** — mutations inside the branch
cannot prevent later branch commands from running.

```rust
when(HAS_CELLS.of("@s").is_true()).then_all(mcfunction![
    cmd::tellraw(Selector::self_(), Text::new("Already have enhanced cells").red());
    cmd::return_fail();
]);
// → execute if score @s … matches 1 run function <ns>:sand/branches/N
//
// Branch function:
//   tellraw @s {"text":"Already have enhanced cells","color":"red"}
//   return fail
```

This is the correct pattern for any branch with more than one command,
especially when the branch modifies its own condition.

## Per-command wrapping — `then_each` (explicit opt-in, legacy behavior)

```rust
when(MANA.of("@s").gte(25)).then_each(["say a", "say b"]);
// → execute if score @s mana matches 25.. run say a
//   execute if score @s mana matches 25.. run say b
```

**Warning:** if a command in `then_each` modifies the condition, later commands
may not run. Prefer `then_all` for correctness.

## Chained branches — `and_then(...).then(...)`

Equivalent to `then_all` — creates one grouped branch function:

```rust
when(MANA.of("@s").gte(25))
    .and_then("say first")
    .and_then("say second")
    .then("say third");
// → one execute line calling a branch function with all 3 commands
```

## If/else branches — `if_`

```rust
if_(HAS_CELLS.of("@s").is_true())
    .then_all(mcfunction![
        cmd::tellraw(Selector::self_(), Text::new("Already granted").red());
        cmd::return_fail();
    ])
    .else_all(mcfunction![
        cmd::attribute_base_set(Selector::self_(), "minecraft:max_health", 40.0);
        HAS_CELLS.enable("@s");
        cmd::return_cmd(0);
    ]);
// → execute if   score @s … matches 1 run function <ns>:sand/branches/N
//   execute unless score @s … matches 1 run function <ns>:sand/branches/N+1
```

## Return behavior

`cmd::return_fail()` and `cmd::return_cmd(n)` emit `return fail` / `return <n>`.

Inside a **branch function** (from `then_all`, `else_all`, `and_then().then()`):
- `return fail` / `return 0` stops **the branch function** and returns to the parent.
- The parent function continues after the execute line that called the branch.

Inside a **direct execute** (`then_one`):
- `return fail` / `return 0` is part of the inline command chain.

If you need the parent to also exit after the branch, call the branch with
`execute … run function …` and use `execute store result score … run function …`
to detect the return value, or structure your logic so the parent checks the flag
after both branches have run.

## Condition sources

| Source | Method | Lowers to |
|---|---|---|
| `ScoreVar<i32>` | `.of(sel).gte(n)`, `.lte(n)`, `.eq(n)` | `if score … matches …` |
| `Flag` | `.of(sel).is_true()`, `.is_false()`, `.is_not_true()` | `if/unless score … matches 1` |
| `Cooldown` | `.ready(sel)` | `if score … matches 0` |
| `Condition::predicate` | — | `if predicate …` |
| `Condition::entity` | — | `if entity …` |
| `Condition::storage_exists` | — | `if data storage … …` |
| `all![...]` | — | chained `if … if …` |
| `any![...]` | — | one execute command per sub-condition |
| `!condition` / `Condition::Not` | — | flips `if` ↔ `unless` |
