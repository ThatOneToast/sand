# Conditions

Conditions are typed values that lower into valid execute plans. Use them with
`TypedExecute`, `when`, or `unless`.

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

Nested `any!` inside `all!` expands to multiple complete execute commands, so
callers do not need to flatten boolean logic by hand.

```rust
let commands = TypedExecute::as_players()
    .when(ready)
    .run(cmd::function(ResourceLocation::new("example", "dash").unwrap()));
```

Typed condition sources currently include scores, flags, cooldowns, predicates,
entities, and storage paths.
