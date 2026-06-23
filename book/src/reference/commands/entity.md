# Entity And Player Commands

## Teleport and summon

```rust
use sand_commands::{summon_at, summon_at_with_nbt, tp_vec3, tp_with_rotation, Vec3};

tp_vec3(Selector::self_(), Vec3::absolute(10.0, 64.0, -5.0));
tp_with_rotation(Selector::self_(), Vec3::here(), Rotation::absolute(180.0, 0.0));
summon_at("minecraft:armor_stand", Vec3::here());
summon_at_with_nbt("minecraft:armor_stand", Vec3::here(), "{Invisible:1b}");
```

The first three are fully typed around target/position. The final NBT is intentionally raw because arbitrary summon NBT is version- and entity-specific.

## Damage, effects, sound, and particles

```rust
Damage::new().to(EntityTargets::nearby(5.0)).amount(DamageAmount::hearts(1.0)).run();
cmd::effect_give(Selector::self_(), EffectId::Regeneration).seconds(3).amplifier(0);
```

`DamageAmount::points(1.0)` and `.fixed(1.0)` mean one HP; `.hearts(1.0)` means two HP. Sound and particle builders generate the corresponding vanilla commands; choose a typed resource id where Sand exposes one. For multi-target damage Sand lowers through `execute as ... run damage @s`, because vanilla direct damage has a single-target rule.

<div class="sand-generated"><strong>Generated model.</strong> Command builders do not require a runtime library in Minecraft. They become text in a generated `.mcfunction`.</div>
