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

`CommandError` names the helper and the invalid field/reason. Prefer the
`try_*` path at command-generation boundaries where input isn't already
known-valid (e.g. values threaded from user-facing config or another
system), and the infallible helpers for literals you control directly.
