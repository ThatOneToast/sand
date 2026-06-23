# 2. Typed Commands

Use a builder whenever Sand models the command; it emits the same vanilla command you would write by hand.

```rust
#[function]
pub fn reward() {
    cmd::give(Selector::self_(), "minecraft:diamond");
    cmd::effect_give(Selector::self_(), EffectId::Speed).seconds(5).amplifier(1);
}
```

For positions, use `sand_commands::{tp_vec3, Vec3}`. For the complete command map and raw fallback rules, see [Typed Commands](../manual/commands.md).
