# Typed Commands

Command builders return a command-like value collected by Sand's macros. Prefer typed selectors, positions, effects, text, scoreboards, sound, particles, and damage.

```rust
use sand_commands::{summon_at, tp_vec3, Vec3};
cmd::give(Selector::self_(), "minecraft:diamond");
tp_vec3(Selector::self_(), Vec3::here());
summon_at("minecraft:pig", Vec3::absolute(0.0, 64.0, 0.0));
cmd::effect_give(Selector::self_(), EffectId::Speed).seconds(5);
Damage::new().to(EntityTargets::nearby(4.0)).amount(DamageAmount::hearts(1.0)).run();
```

`DamageAmount::hearts(1.0)` means two HP; `points` and `fixed` are HP. Safe constructors are fixed-only and do not panic. Attribute, particle, sound, display, and scoreboard builders follow the same model. Use [Raw Commands](raw-commands.md) only where an API is unmodeled.

## Validated free functions

`sand_commands::builtins`'s free functions (`summon`, `tp`, `tag_add`,
`gamemode`, `effect_give`, `function`, `schedule*`, `damage`,
`attribute_base_set`, `give`/`give_count`, `kick`, and more) remain
infallible `String`-returning helpers for backward compatibility — they are
documented raw/unchecked escape hatches that accept non-finite coordinates,
empty tags, or an unrecognized `gamemode`/`difficulty` string without
complaint.

Each has a `try_*` counterpart returning `CommandResult<String>`
(`sand_commands::CommandError`) that validates first:

```rust
use sand_commands::{try_tp, try_tag_add, Selector};

// Rejected before it becomes command text — NaN coordinates would otherwise
// silently produce `tp @s NaN NaN NaN`.
assert!(try_tp(Selector::self_(), f64::NAN, 0.0, 0.0).is_err());

assert!(try_tag_add(Selector::self_(), "").is_err());
assert!(try_tag_add(Selector::self_(), "ready").is_ok());
```

`CommandError` names the helper and invalid field/reason. The shared
`RenderCommand` boundary additionally covers selectors, coordinates, item
slots, score holders/objectives, scoreboard operations, and foundational
execute arguments. Sand validates those typed values structurally. At final
export, every collected command receives single-line integrity validation;
only confidently recognized top-level commands receive additional
argument-position-aware fallback checks. Failures include the owning function,
command index, and Minecraft profile.

Prefer `try_*` wherever an older free function still returns `String`. Use
`cmd::raw(...)` only for syntax Sand does not model. Unknown advanced, macro,
and modded syntax remains verbatim after single-line integrity checks. The
fallback validator does not search raw JSON, SNBT, or literal text for
command-shaped substrings.

## Text click and hover events

`TextComponent` exposes typed builders for book page links, item stacks, and
entity tooltips as well as the usual command, URL, and text-hover actions:

```rust
use sand_core::prelude::*;

let next = Text::new("Next page").click_change_page(2);

let zombie = EntityTypeId::minecraft("zombie")?;
let uuid = EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000")?;
let entity = Text::new("Inspect")
    .hover_entity_with_id(zombie, uuid, Text::new("Undead").red());

let stack = Text::new("Reward")
    .hover_item_with_count("minecraft:diamond", 3);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`change_page` is only meaningful inside book text; Sand serializes page `0`
without rejecting it, but written-book pages are normally one-indexed. Item
hover counts are emitted only when supplied, so the existing count-free shape
is unchanged. Entity hover names are full styled text components, entity types
use typed registry IDs, and supplied entity UUIDs must first be parsed as an
`EntityHoverId`.

`hover_entity_raw` is the explicit compatibility escape hatch for legacy,
version-specific, or otherwise unmodeled entity type and UUID values. It does
not validate those raw strings; prefer `hover_entity` or
`hover_entity_with_id` for normal authoring. The final command export boundary
still validates line integrity and recognized command arguments for text
commands, just as it does for other collected commands.
