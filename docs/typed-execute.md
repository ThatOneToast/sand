# Typed Execute

`TypedExecute` provides common execute chains. `ExecuteExt` adds `.when(...)`
and `.unless(...)` to the lower-level execute builder.

`when` / `unless` / `if_` from the prelude create execute-conditional commands
for use inside `#[function]` or `#[component]` bodies.

## TypedExecute (single-command execute chains)

```rust
use sand_core::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

let commands = TypedExecute::as_players_at_self()
    .when(MANA.of("@s").gte(25))
    .run(Actionbar::show(Selector::self_(), Text::new("Ready").aqua()));
```

## when / unless (function-body conditional branches)

These are for inside `#[function]` bodies to emit conditional logic:

```rust
// Single command — no branch function generated
when(MANA.of("@s").gte(25)).then_one("say enough mana");

// Grouped branch — all commands run in order under one condition check
when(HAS_CELLS.of("@s").is_true()).then_all(mcfunction![
    cmd::tellraw(Selector::self_(), Text::new("Already granted").red());
    cmd::return_fail();
]);

// Per-command wrapping (explicit opt-in — use only when re-checking is intended)
when(MANA.of("@s").gte(25)).then_each(["say a", "say b"]);
```

## if_ (if/else branches)

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

Both arms generate stable branch helper functions. The `else` arm uses `unless`
polarity — no manual polarity management needed.

## Custom execute chains

For custom execute chains, start from `cmd::Execute`:

```rust
let commands = Execute::new()
    .as_(Selector::all_players())
    .at(Selector::self_())
    .unless(MANA.of("@s").lt(25))
    .run(cmd::function(ResourceLocation::new("example", "cast").unwrap()));
```

## Return behavior in branches

`cmd::return_fail()` → `return fail`
`cmd::return_cmd(0)` → `return 0`

Inside a branch function (from `then_all` / `else_all`), `return` stops the branch
function and returns control to the parent. The parent continues executing commands
after the `execute … run function` line that called the branch.
