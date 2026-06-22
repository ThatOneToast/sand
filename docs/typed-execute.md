# Typed Execute

`TypedExecute` provides common execute chains. `ExecuteExt` adds `.when(...)`
and `.unless(...)` to the lower-level execute builder.

```rust
use sand_core::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

let commands = TypedExecute::as_players_at_self()
    .when(MANA.of("@s").gte(25))
    .run(Actionbar::show(Selector::self_(), Text::new("Ready").aqua()));
```

For custom chains, start from `cmd::Execute`:

```rust
let commands = Execute::new()
    .as_(Selector::all_players())
    .at(Selector::self_())
    .unless(MANA.of("@s").lt(25))
    .run(cmd::function(ResourceLocation::new("example", "cast").unwrap()));
```
