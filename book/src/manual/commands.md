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
execute arguments. Sand applies conservative validation to collected function
commands before export and enriches failures with the owning function, command
index, and Minecraft profile.

Prefer `try_*` wherever an older free function still returns `String`. Use
`cmd::raw(...)` only for syntax Sand does not model. Unknown advanced syntax
remains verbatim; export still applies single-line integrity checks and rejects
recognizably malformed foundational selectors, coordinates, score operations,
and function references.
