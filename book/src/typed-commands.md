# Typed Commands

Command builders return command-like values that attribute functions collect.

```rust
#[function]
pub fn reward() {
    cmd::tellraw(Selector::self_(), Text::new("Quest complete").green());
    cmd::give(Selector::self_(), "minecraft:diamond");
    cmd::tag_add(Selector::self_(), "quest_complete");
}
```

Prefer typed selectors, text, resources, items, and builders where Sand exposes
them. Use `cmd::raw(...)` only when the command is intentionally outside typed
coverage.

## Positions, summons, damage, and common commands

Use `Vec3` and `Rotation` rather than formatting coordinates yourself:

```rust
use sand_core::cmd::{Rotation, Vec3};
use sand_commands::{summon_at, summon_at_with_nbt, tp_vec3, tp_with_rotation};

tp_vec3(Selector::self_(), Vec3::absolute(10.0, 64.0, -5.0));
tp_with_rotation(Selector::self_(), Vec3::here(), Rotation::absolute(90.0, 0.0));
summon_at("minecraft:armor_stand", Vec3::here());
summon_at_with_nbt("minecraft:armor_stand", Vec3::here(), "{Invisible:1b}");
```

`DamageAmount::hearts(1.0)` lowers to 2 HP, while `points(1.0)` and `fixed(1.0)` mean one HP. The only supported variant is fixed damage; the old score/event variants were removed because they could panic while generating a command.

```rust
Damage::to(Selector::self_()).amount(DamageAmount::hearts(2.0)).run();
cmd::give(Selector::self_(), "minecraft:diamond");
cmd::effect_give(Selector::self_(), EffectId::Speed).duration(Ticks::seconds(5));
cmd::attribute_base_set(Selector::self_(), "minecraft:generic.max_health", 40.0);
```

Particles, sounds, effects, text, selectors, scoreboards, and execute chains also have typed builders. Use a raw command only for syntax Sand does not yet model, a modded command, or a Minecraft version change; label that choice in project code.
