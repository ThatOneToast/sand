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
