# 20. Command Arguments: Coordinates, Selectors, Scores, and Blocks

Sand's command-argument boundary follows the same pipeline as the validated
media builders in the previous chapter:

```text
canonical typed argument
        ↓
argument-level validation
        ↓
typed command node
        ↓
profile-aware command validation
        ↓
deterministic rendering
        ↓
pre-write exporter boundary
```

This chapter covers the primitives that sit underneath every higher-level
command builder: coordinates, selectors/targets, scoreboard objectives and
holders, and typed block states. Diagnostics in this layer use the
`SAND-COORD-*`, `SAND-SELECTOR-*`, `SAND-SCORE-*`, and `SAND-BLOCK-*`
families.

## Coordinates

`Coord` models one axis: absolute, relative (`~`), or local (`^`). Every
higher-level position type ([`Vec3`], [`Vec2`], [`Rotation`], [`BlockPos`])
is built from `Coord`s and shares one validator: non-finite values (`NaN`,
`±inf`) are rejected before they can reach generated command text.

```rust,ignore
use sand_commands::coord::{Coord, Vec3, BlockPos};
use sand_commands::RenderCommand;

// Generic position — accepts local (^) coordinates and fractional values.
let pos = Vec3::new(Coord::abs(1.5), Coord::abs(64.0), Coord::local_n(2.0));

// Block position — setblock/fill/clone/block-data grammar.
// Integer absolute/relative coordinates only; local (^) is rejected.
let block_pos = BlockPos::absolute(0, 64, 0);
assert!(block_pos.try_build().is_ok());

let invalid = BlockPos::new(Coord::abs(1.5), Coord::abs(2), Coord::abs(3));
assert!(invalid.try_build().is_err());
```

`BlockPos` and `Vec3` are deliberately different types with different
grammars — do not reuse `BlockPos` as a generic fractional/local position.
Use `Vec3` for commands with generic position grammar (`execute positioned`,
`particle`, `summon`, `tp`), and `BlockPos` only for commands that require
Minecraft's stricter block-position syntax.

## Selectors and typed targets

`Selector` is the canonical entity/player target builder. It validates
limits, distance/level ranges, score filters, tags, teams, and player-name
tokens, and keeps output deterministic (arguments render in a stable order).

```rust,ignore
use sand_commands::{Selector, RenderCommand};

let scan = Selector::all_entities()
    .limit(5)
    .distance_range(0.0, 16.0);

assert!(scan.try_build().is_ok());
assert!(Selector::all_entities().limit(0).try_build().is_err());
```

Opaque advanced syntax (arbitrary SNBT filters, modded selector arguments)
has an explicit escape hatch rather than a best-effort parser:

```rust,ignore
Selector::nbt_raw(r#"{CustomModTag:1b}"#);
Selector::argument_raw("my_mod:custom_arg=value");
```

Typed target wrappers — `ScoreHolder`, display/sound audiences, execute
targets — build on the same `Selector` validation path; none of them format
a selector to a string before validating it.

## Scoreboard objectives and holders

`ObjectiveName` and `ScoreHolder` are the canonical scoreboard primitives:

```rust,ignore
use sand_commands::scoreboard::{Objective, ObjectiveName, ScoreHolder};

static MANA: Objective = Objective::new("mana");

// Validated path: rejects malformed names/holders before rendering.
let cmd = MANA.try_set(ScoreHolder::self_(), 100)?;

// Fake players get their own validation (no leading `@`, ≤ 40 chars).
assert!(ScoreHolder::fake("#total_kills").try_build().is_ok());
assert!(ScoreHolder::fake("@a").try_build().is_err());
# Ok::<(), sand_commands::CommandError>(())
```

`sand_core::state::ScoreVar` is the ergonomic `static`-declared counterpart
used throughout gameplay code. Its infallible methods (`set`, `add`, `clamp`,
...) remain available for byte-identical compatibility with existing code,
but prefer the validated counterpart where the input is not already known to
be correct — for example, `ScoreVar::try_clamp` rejects `min > max` instead
of emitting two contradictory `execute if score ... matches` commands:

```rust,ignore
use sand_core::state::ScoreVar;

static HEALTH: ScoreVar<i32> = ScoreVar::new("health");

assert!(HEALTH.try_clamp("@s", 0, 100).is_ok());
assert!(HEALTH.try_clamp("@s", 100, 0).is_err());
```

Score-range helpers (`ScoreRef::gt`, `lt`, `between`, `matches`) have
validated counterparts (`try_gt`, `try_lt`, `try_between`, `try_matches`)
that reject ranges no `i32` score can satisfy, such as `Gt(i32::MAX)` or a
`between` call with `min > max`.

## Typed block states

`BlockState` models `namespace:path[key=value,...]` block-state strings.
Block IDs are validated as resource locations (syntax only — Sand does not
claim registry-aware membership checking), and property keys/values reject
whitespace, control characters, and the block-state delimiters (`[`, `]`,
`=`, `,`) that would corrupt the surrounding grammar:

```rust,ignore
use sand_commands::blocks::{BlockState, SetBlock};
use sand_commands::coord::BlockPos;
use sand_commands::RenderCommand;

let stairs = BlockState::of("minecraft:oak_stairs")
    .prop("facing", "east")
    .prop("half", "bottom");

let cmd = SetBlock::new(BlockPos::here(), stairs).try_build()?;
assert_eq!(cmd, "setblock ~ ~ ~ minecraft:oak_stairs[facing=east,half=bottom]");
# Ok::<(), sand_commands::CommandError>(())
```

`SetBlock`, `Fill`, and `CloneBlocks` all implement the shared
`Validate`/`RenderCommand` traits, so `try_build()` validates positions
(reusing the coordinate validator above), block state, and non-empty
replace/clone filters. `Build::build()`/`Display` remain available as an
explicit, infallible raw path for syntax Sand does not yet model. Fill/clone
region *volume* limits are intentionally **not** enforced: the effective
limit is server/gamerule-configurable, so Sand cannot claim build-time
correctness for it — that remains a runtime constraint.

## Raw command lines

`RawCommand` is the explicit escape hatch for command families Sand does not
model at all (modded commands, future syntax). It only validates the
container, not the command's semantics: a raw command must be exactly one
line, contain no NUL/newline/carriage-return or other control characters,
and must not start with `/` (a `.mcfunction` line never begins with a
leading slash). Unknown command names are never rejected — that would break
modded and future commands Sand has no way to know about.

## A combined example

```rust,ignore
use sand_commands::{Selector, RawCommand};
use sand_commands::blocks::{BlockState, SetBlock};
use sand_commands::coord::BlockPos;
use sand_commands::scoreboard::{Objective, ScoreHolder};
use sand_core::cmd;

static PROGRESS: Objective = Objective::new("trail_prog");

let nearby_runners = Selector::all_players()
    .distance_range(0.0, 24.0)
    .tag("trailforge_active");

let mark_checkpoint = SetBlock::new(
    BlockPos::absolute(120, 70, -32),
    BlockState::of("minecraft:beacon"),
)
.try_build()?;

let bump_score = PROGRESS.try_add(ScoreHolder::self_(), 1)?;
let teleport = cmd::try_tp(Selector::self_(), 120.5, 71.0, -31.5)?;
let tag_done = cmd::try_tag_add(Selector::self_(), "checkpoint_1")?;

// Escape hatch for a mod command Sand does not model.
let modded = RawCommand::new("mymod:pulse 5").to_string();

let _ = (nearby_runners, mark_checkpoint, bump_score, teleport, tag_done, modded);
# Ok::<(), sand_commands::CommandError>(())
```

## Migrating from string-first helpers

Most `sand_commands::builtins` free functions have a `try_` counterpart
(`try_summon`, `try_tp`, `try_tag_add`, `try_gamemode`, `try_damage`,
`try_team_add`, `try_schedule`, `try_gamerule`, ...) that validates its
arguments before returning command text. The plain, infallible functions
remain documented compatibility/raw paths — prefer the `try_` form for any
input that is not already known-valid at compile time.

`sand_core::cmd::Storage` (the `HashMap`-style NBT storage wrapper) follows
the same pattern: `try_remove`, `try_get`, `try_get_scaled`, `try_contains`,
`try_get_or_insert`, and `try_merge` route through the same
`DataTarget`/`NbtPath` validation as the typed `data`-command IR, while the
original infallible methods keep their existing (unvalidated) output for
compatibility.
