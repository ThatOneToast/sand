# 8. Commands And Execution Contexts

This chapter walks through Trailforge's `#[function]`-backed commands, which
together demonstrate typed commands, `execute as/at` context, and grouped
conditional branching.

## `trail:grapple` — gating with `TypedExecute`

{{#include ../../examples/book_project/src/lib.rs:fn_grapple}}

`TypedExecute::as_players_at_self()` builds an `execute as @a at @s run
...`-shaped command: **as** every player (so the check runs once per
player, in that player's own scoreboard context), **at self** (so any
subsequent positional commands run at that player's current position,
useful once you add particles/sounds relative to the player). `.when(...)`
takes the same `all![...]` condition list chapter 3's `tick` uses for
readiness, and `.run(...)` dispatches to `trail:grapple/execute` by
`ResourceLocation` (chapter 4) rather than inlining the dash logic here.
Splitting "is the dash allowed" from "what the dash does" means the
gating conditions stay visible and reusable (the `tick` actionbar reuses
the identical condition list) without duplicating the effects/VFX code.

## `trail:grapple/execute` — paying costs and playing feedback

{{#include ../../examples/book_project/src/lib.rs:fn_grapple_execute}}

This function assumes its caller already validated the gate — it's an
"apply" function, not a "check" function, so it goes straight to paying the
stamina cost, starting the cooldown, applying vanilla effects
(`Speed`/`SlowFalling` standing in for a grapple's momentum, since Sand
commands can't grant arbitrary physics velocity — see
[Vanilla Limitations](reference/vanilla-limitations.md)), and playing VFX
(chapter 13). `Selector::self_()` (`@s`) is correct here because this
function is always reached from inside the `as @a` context `trail:grapple`
already established — `@s` means "the player the outer execute is
currently iterating," not "whoever ran this command directly."

## `trail:recover` — a plain reset function

{{#include ../../examples/book_project/src/lib.rs:fn_recover}}

Called from `tick`'s exhaustion-clearing branch (chapter 3) via
`cmd::function(ResourceLocation::new("trail", "recover").unwrap())`. No
gating logic lives here — the caller already checked the condition — so the
function body is just the state change and the player-facing message.

## `trail:claim_striders` — grouped branching with `if_()`

{{#include ../../examples/book_project/src/lib.rs:fn_claim_striders}}

`if_(condition).then_all(mcfunction![...]).else_all(mcfunction![...])` is
Sand's grouped-branch API: it compiles to a pair of `execute if`/`unless`
guarded command blocks sharing one condition, rather than you writing that
condition (and its negation) out twice by hand. The `then` arm handles the
rejection path — a player who already owns Trail Striders is told so and
the function returns a failure signal via `cmd::return_fail()`, which
matters if this function is ever chained from another `execute
... run function` that checks success. The `else` arm is the happy path:
grant the item (reusing `trail_striders()` from chapter 5, so the given
item can never drift from its definition), set the `HAS_STRIDERS` flag, and
`cmd::return_cmd(1)` to signal success explicitly.

`mcfunction![...]` is the declarative macro (re-exported through the
prelude) for "a block of commands treated as one execute branch" — the
semicolon-separated list inside becomes the ordered command sequence for
that arm.

## `trail:menu` — a one-line dispatch to a dialog

{{#include ../../examples/book_project/src/lib.rs:fn_open_menu}}

`cmd::show_dialog` opens a dialog by reference (`DialogRef::local(...)`)
rather than needing the full `Dialog` value in scope — see chapter 12 for
`trailhead_dialog`, the definition this reference points at.

## Execution context, summarized

Every command above runs inside *some* execution context — `as @a`,
`at @s`, the default "whoever/wherever this function was called from." Sand
makes that context an explicit, typed part of the builder chain
(`TypedExecute::as_players_at_self()`) rather than a string you assemble by
hand, so `@s` inside a called function means exactly what the caller's
`as`/`at` clause set it to, and a missing `as` (which would make `@s`
resolve to the command block or server console instead of a player) is a
visible gap in the builder chain rather than a silent runtime bug.
