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
higher-level position type (`Vec3`, `Vec2`, `Rotation`, `BlockPos`) is built
from `Coord`s and shares one validator: non-finite values (`NaN`, `±inf`)
are rejected before they can reach generated command text.

`BlockPos` and `Vec3` are deliberately different types with different
grammars — `BlockPos` (used by `setblock`/`fill`/`clone`/block-targeted
`data`) requires integer absolute or relative coordinates and rejects local
(`^`) coordinates, while `Vec3` (used by commands with generic position
grammar, such as `execute positioned`, `particle`, `summon`, `tp`) accepts
fractional and local coordinates. Do not reuse `BlockPos` as a generic
fractional/local position, and do not reuse `Vec3` where a command's grammar
actually requires integer block coordinates.

## Selectors and typed targets

`Target` is the canonical entity/player target builder. It validates
limits, distance/level ranges, score filters, tags, teams, and player-name
tokens, and keeps output deterministic (arguments render in a stable order).

```rust,ignore
use sand::prelude::*;

let scan = Target::entities()
    .distance_range(0.0, 16.0)
    .nearest();

assert_eq!(scan.to_string(), "@e[distance=0..16,sort=nearest,limit=1]");
assert!(Target::entities().limit(2).is_err());
```

Literal named targets can still be refined, but their literal already counts
as the selector's one positive `name` filter. Validation rejects a second
positive name while allowing multiple `.not_name(...)` exclusions.

Opaque advanced syntax (arbitrary SNBT filters, modded selector arguments)
has an explicit escape hatch rather than a best-effort parser:
`Target::nbt_raw(...)`, `Target::predicate_raw(...)`, and the explicitly named
`Target::raw_many(...)` / `Target::raw_single(...)` constructors. A raw target
is emitted verbatim until typed refinements are added; refinements such as
`.sort(...)` and `.limit(1)` are incorporated into the selector text so the
rendered command continues to match its typed cardinality.

Reusable predicate filters use `.predicate(PredicateId)` and
`.not_predicate(PredicateId)`. Other registry ID kinds do not compile at that
boundary; arbitrary predicate text stays explicit through `.predicate_raw(...)`.

`ScoreHolder` remains a distinct, broader scoreboard domain because it also
models fake players and wildcards. Entity/player targets, display and sound
audiences, and execute targets all consume `Target` directly.

## Scoreboard objectives and holders

`ObjectiveName` and `ScoreHolder` are the canonical scoreboard primitives:

```rust,ignore
use sand::prelude::*;

// Fake players get their own validation (no leading `@`, ≤ 40 chars).
assert!(ScoreHolder::fake("#total_kills").try_build().is_ok());
assert!(ScoreHolder::fake("@a").try_build().is_err());
```

Gameplay scores belong in a derived State schema. Its generated bound accessor
provides arithmetic and comparisons without exposing an objective name:

```rust,ignore
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = player)]
struct Combat {
    #[state(default = 100, min = 0, max = 100)]
    health: Score,
}

let combat = Combat::on(EntityContext::<PlayerKind>::default());
combat.health.set(80);
combat.health.gte(1);
```

Low-level `Objective` and scoreboard primitives are available only from the
explicit advanced/command boundary for framework integrations.

## Typed block states

`BlockState` models `namespace:path[key=value,...]` block-state strings used
by `setblock`, `fill`, and `clone`. Block IDs are validated as resource
locations (syntax only — Sand does not claim registry-aware membership
checking), and property keys/values reject whitespace, control characters,
and the block-state delimiters (`[`, `]`, `=`, `,`) that would corrupt the
surrounding grammar. `SetBlock`, `Fill`, and `CloneBlocks` share this
validation through `try_build()`, which also validates positions (reusing
the coordinate validator above) and non-empty replace/clone filters.
`Build::build()`/`Display` remain available as an explicit, infallible raw
path for syntax Sand does not yet model.

Fill/clone region *volume* limits are intentionally **not** enforced: the
effective limit is server/gamerule-configurable, so Sand cannot claim
build-time correctness for it — that remains a runtime constraint.

> **Facade note.** `BlockState`/`SetBlock`/`Fill`/`CloneBlocks`/`Coord`/
> `Vec3`/`BlockPos` currently live in `sand_commands` and are not yet
> re-exported through `sand::prelude`. Datapack authors reach them today via
> the lower-level `sand_commands` crate directly, or by depending on future
> `sand::block`/`sand::coord` facade modules. Exposing them through the
> prelude is tracked as follow-up work, not part of this change.

## Raw command lines

`RawCommand` is the explicit escape hatch for command families Sand does not
model at all (modded commands, future syntax). It only validates the
container, not the command's semantics: a raw command must be exactly one
line, contain no NUL/newline/carriage-return or other control characters,
and must not start with `/` (a `.mcfunction` line never begins with a
leading slash). Unknown command names are never rejected — that would break
modded and future commands Sand has no way to know about.

```rust,ignore
use sand::prelude::*;

let modded = RawCommand::new("mymod:pulse 5").to_string();
```

## A combined example

```rust,ignore
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "trail", scope = player)]
struct Progress { #[state(default = 0)] value: Score }

let nearby_runners = Target::players()
    .distance_range(0.0, 24.0)
    .tag("trailforge_active");

let progress = Progress::on(EntityContext::<PlayerKind>::default());
let bump_score = progress.value.add(1);
let teleport = cmd::try_tp(Target::self_(), 120.5, 71.0, -31.5)?;
let tag_done = cmd::try_tag_add(Target::self_(), "checkpoint_1")?;

// Escape hatch for a mod command Sand does not model.
let modded = RawCommand::new("mymod:pulse 5").to_string();

let _ = (nearby_runners, bump_score, teleport, tag_done, modded);
# Ok::<(), sand::command::CommandError>(())
```

Block placement (`setblock`, `fill`) is left out of this facade-only example
per the note above — see `sand_commands::blocks` directly until it is
re-exported.

## Migrating from string-first helpers

Normal resource, function, target, and NBT paths accept canonical typed values.
Unsupported future or modded syntax is deliberately secondary and uses an API
whose name ends in `_raw`.
