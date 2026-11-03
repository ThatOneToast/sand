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

The public `Ability::attempt` checks the configured readiness before casting. It requires the unlock, the actual custom Trail Striders in the feet
slot, sufficient stamina, a ready cooldown, and no exhaustion. Owning the
boots in inventory or wearing plain leather boots is insufficient.

## Bounded anchor interaction

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:anchor_workflow}}
```

The eye-origin ray samples every 0.25 blocks, up to 8 blocks for grapple or
4 for scout. It stops at the first sampled non-air block. Grapple accepts a
gold block; scout accepts an iron block. A wall before the anchor rejects the
interaction. Missing, unloaded, out-of-world, and invalid hits spend nothing
and start no cooldown. Sampling can skip thin obstacles; it is not exact
voxel traversal. Only ordinary air is passable in this example.

`trail:scout` demonstrates reuse with a different range, accepted block, cost
(10), and cooldown (20 ticks). It shares stamina and cooldown with grapple;
using either prevents immediately using the other. Both import the same
`sand::systems::ability` composition used by the separate `aether_tools` example.

## `trail:grapple/execute` — accepted-hit effects

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:fn_grapple_execute}}
```

The internal hit callback calls `Ability::accept`, which checks the sampled
gold block and readiness together before any writes. One grouped branch spends
30 stamina, starts an 80-tick cooldown, then invokes this effect-only function
for Speed/Slow Falling plus feedback. Spending the last 30 stamina cannot
suppress cooldown or effects. These effects stand in
for momentum; there is no teleport or arbitrary physics velocity. Success
means accepting the anchor and issuing the effects, not proving that every
Minecraft effect command changed the player. There is no rollback promise.

The ray preserves the caster as executor and runs the callback at the hit
sample. Thus effects and State apply only to that player; VFX explicitly reposition at
the player. The internal `/execute` functions contain effects only and are not safe activation
entrypoints. Hit callbacks recheck eligibility but require the ray's target position. Use `trail:grapple` or `trail:scout` as gameplay entrypoints.

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
