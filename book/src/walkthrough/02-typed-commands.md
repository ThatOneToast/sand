# 2. Typed Commands

## What you will build

Add a debug/reward function that demonstrates the normal typed command path and one explicit raw boundary.

## Concepts introduced

Typed text, give/effect, positions, summon, damage amounts, and `cmd::raw`.

## File changes

Add this to `arcane/src/lib.rs`:

```rust
use sand_commands::{summon_at, tp_vec3, Vec3};

#[function("arcane:command_demo")]
pub fn command_demo() {
    cmd::tellraw(Selector::self_(), Text::new("Arcane command demo").gold());
    cmd::give(Selector::self_(), "minecraft:amethyst_shard");
    cmd::effect_give(Selector::self_(), EffectId::Speed).seconds(5).amplifier(0);
    tp_vec3(Selector::self_(), Vec3::here());
    summon_at("minecraft:pig", Vec3::absolute(0.0, 64.0, 0.0));
    Damage::new().to(EntityTargets::nearby(3.0)).amount(DamageAmount::hearts(1.0)).run();
    // Unsupported external datapack API: intentionally raw.
    cmd::raw("function other_pack:api/optional_hook");
}
```

## How it works

Builders serialize to valid vanilla commands. `hearts(1.0)` is two `/damage` HP; use `points`/`fixed` for HP. Position helpers accept typed absolute/relative/local coordinates. Raw commands are only for external/modded/unsupported syntax.

## What Sand generates

The function contains `tellraw`, `give`, `effect give`, `tp`, `summon`, and safe multi-target damage lowering (`execute as ... run damage @s ...`).

## Try it in Minecraft

Reload then run `/function arcane:command_demo`. Confirm the message, item, effect, pig, and nearby damage. Remove the optional raw hook unless that datapack exists.

## Common mistakes

- Using `cmd::raw` for give/effect/teleport that already have typed APIs.
- Confusing hearts with HP.
- Summoning an entity at a constant world coordinate unintentionally.

## Deeper reading

[Typed Commands](../manual/commands.md), [Command Builders](../reference/command-builders.md), and [Raw Commands](../manual/raw-commands.md).
