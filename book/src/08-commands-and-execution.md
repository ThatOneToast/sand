# 8. Commands And Execution Contexts

This chapter walks through Trailforge's `#[function]`-backed commands, which
together demonstrate typed commands, `execute as/at` context, and grouped
conditional branching.

## `trail:grapple` — gating with `TypedExecute`

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:fn_grapple}}
```

`#[function(..., context = player)]` declares a player entry point and ensures
State initialization before the body runs. Invoke it as a player (the dialog
already does this). `TypedExecute::as_self_at_self()` keeps that player as the
executor and positions the effects at their location. It never selects other
players: one menu click pays only the clicking player's stamina and cooldown.

`.when(...)` takes the same `all![...]` condition list chapter 3's `tick` uses
for readiness, and `.run(cmd::function(grapple_execute))` dispatches through the
canonical function handle rather than inlining the dash logic here.
Splitting "is the dash allowed" from "what the dash does" means the
gating conditions stay visible and reusable (the `tick` actionbar reuses
the identical condition list) without duplicating the effects/VFX code.

## `trail:grapple/execute` — paying costs and playing feedback

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:fn_grapple_execute}}
```

This function assumes its caller already validated the gate — it's an
"apply" function, not a "check" function, so it goes straight to paying the
stamina cost, starting the cooldown, applying vanilla effects
(`Speed`/`SlowFalling` standing in for a grapple's momentum, since Sand
commands can't grant arbitrary physics velocity — see
[Vanilla Limitations](reference/vanilla-limitations.md)), and playing VFX
(chapter 13). `Target::self_()` (`@s`) is correct here because this
function is always reached from inside the `as @a` context `trail:grapple`
already established — `@s` means "the player the outer execute is
currently iterating," not "whoever ran this command directly."

## `trail:recover` — a plain reset function

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:fn_recover}}
```

Called from `tick`'s exhaustion-clearing branch (chapter 3) via
`cmd::function(FunctionId::custom(ResourceLocation::new("trail", "recover").unwrap()))`. No
gating logic lives here — the caller already checked the condition — so the
function body is just the state change and the player-facing message.

## `trail:claim_striders` — grouped branching with `if_()`

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:fn_claim_striders}}
```

`if_(condition).then_all(mcfunction![...]).else_all(mcfunction![...])` is
Sand's grouped-branch API evaluates a condition once and runs only its selected
body. The outer branch rejects players who already unlocked Trail Striders.
The inner branch requires a matching Grapple Core in the current player's
inventory. A plain Heart of the Sea does not satisfy the custom-item predicate.

On success, the function removes one core, gives the component-bearing boots
through `cmd::give`, and sets the unlock flag. Rejected requests leave inventory
and progression unchanged. The boots are granted to inventory rather than
replacing an occupied armor slot; the player equips them normally. As with
vanilla `give`, a full inventory can cause the reward to drop beside the player.
This synchronous example checks presence immediately before removal, but does
not promise a general atomic inventory transaction API.

`mcfunction![...]` is the declarative macro (re-exported through the
prelude) for "a block of commands treated as one execute branch" — the
semicolon-separated list inside becomes the ordered command sequence for
that arm.

## `trail:menu` — a one-line dispatch to a dialog

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:fn_open_menu}}
```

`cmd::show_dialog` opens a dialog by reference (`DialogId::local(...)`)
rather than needing the full `Dialog` value in scope — see chapter 12 for
`trailhead_dialog`, the definition this reference points at.

## Execution context, summarized

Every command above runs inside *some* execution context — `as @a`,
`at @s`, the default "whoever/wherever this function was called from." Sand
makes that context an explicit, typed part of the builder chain
(`TypedExecute::as_self_at_self()`) rather than a string you assemble by
hand, so `@s` inside a called function means exactly what the caller's
`as`/`at` clause set it to, and a missing `as` (which would make `@s`
resolve to the command block or server console instead of a player) is a
visible gap in the builder chain rather than a silent runtime bug.
