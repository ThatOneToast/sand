# Command Builders

Sand command values serialize to normal Minecraft commands. Use the typed API when it exists; generated command builders live under `cmd`, while position helpers are re-exported by `sand_commands`.

| Purpose | Rust API | Generated shape | Common mistake |
|---|---|---|---|
| Server message | `cmd::say("hi")` | `say hi` | expecting JSON text |
| Rich message | `cmd::tellraw(target, Text::new(...))` | `tellraw <target> <json>` | passing a plain selector string where typed text is needed |
| Give | `cmd::give(target, item)` | `give <target> <item>` | losing custom-data identity |
| Effect | `cmd::effect_give(target, EffectId::Speed)` | `effect give ...` | amplifier is zero-based |
| Attribute | `cmd::attribute_base_set(...)` | `attribute ... base set` | using an invalid vanilla attribute id |
| Damage | `Damage::new()...` | `damage` / execute-as lowering | `hearts` are two HP each |
| Teleport | `tp_vec3(...)` | `tp <target> <pos>` | mixing absolute/relative coords |
| Summon | `summon_at(...)` | `summon <entity> <pos>` | using raw NBT unnecessarily |
| Raw boundary | `cmd::raw(...)` | supplied command | using it for a covered command |

## Minimal examples

```rust
use sand_core::prelude::*;
use sand_commands::{summon_at, tp_vec3, tp_with_rotation, Vec3};

cmd::say("Arcane ready");
cmd::tellraw(Selector::self_(), Text::new("Mana restored").aqua());
cmd::give(Selector::self_(), "minecraft:diamond");
cmd::effect_give(Selector::self_(), EffectId::Speed).seconds(5).amplifier(1);
cmd::attribute_base_set(Selector::self_(), "minecraft:generic.max_health", 40.0);
tp_vec3(Selector::self_(), Vec3::here());
tp_with_rotation(Selector::self_(), Vec3::here(), Rotation::absolute(90.0, 0.0));
summon_at("minecraft:pig", Vec3::absolute(0.0, 64.0, 0.0));
```

`summon_at_with_nbt` accepts a typed position plus explicit NBT for data not otherwise modeled. Read [Entity Commands](commands/entity.md) and [Scoreboard And Storage Commands](commands/data.md) for lower-level examples.
