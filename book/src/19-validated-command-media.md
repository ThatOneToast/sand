# 19. Validated Command Media

Sand's text, display, particle, sound, effect, and VFX helpers share one
profile-aware validation boundary. Ordinary builders remain ergonomic, but
their typed state is retained until export:

```text
typed builder → command/media node → semantic validation
              → version validation → deterministic rendering
```

An invalid node stops export before any datapack files are written. Diagnostics
name the function/component and structured field path, and use stable families
such as `SAND-TEXT-*`, `SAND-PARTICLE-*`, and `SAND-BOSSBAR-*`.

## Validated text

Use `Text`/`TextComponent` for normal content:

```rust,ignore
cmd::tellraw(
    Target::self_(),
    Text::new("Inspect")
        .color_hex("#12ff00")
        .hover_item("minecraft:diamond"),
);
```

Sand recursively checks selector and score fields, objectives, translation
keys, keybinds, named/hex colors, fonts, click values, URLs, hover payloads,
entity UUIDs, and nested `extra`/translation arguments. The same validator is
used by tellraw, titles, actionbars, bossbar names, dialogs, advancements,
item names/lore, and chat decoration.

Opaque compatibility is explicit: `Text::raw_json`,
`TextComponent::selector_raw`, `hover_item_raw`, and `hover_entity_raw` render
unchanged. Sand still validates typed children surrounding an opaque field.

## Titles, actionbars, and bossbars

`Title` is payload-oriented and rejects an empty payload through `try_build`.
Use `TitleTimes` when timing is the whole command:

```rust,ignore
TitleTimes::new(Target::players(), 10, 70, 20).build();
Actionbar::show(Target::self_(), Text::new("Dash ready").aqua());
```

Bossbars use `BossbarId`; IDs are resource locations and player audiences are
typed selectors:

```rust,ignore
let id = BossbarId::parse("trail:guardian")?;
Bossbar::add(id.clone(), Text::new("Guardian").red());
Bossbar::set_max(id.clone(), 100);
Bossbar::set_value(id.clone(), 40);
Bossbar::set_players(id, Target::players());
# Ok::<(), sand_commands::CommandError>(())
```

Standalone `set_value` cannot know the live maximum and therefore validates
only its own argument. `set_max(0)` is rejected. Use `BossbarId::raw` and
`Actionbar::show_raw` only for deliberately opaque syntax.

## Particles and geometry

Named/custom particle IDs are checked as resource locations without assuming a
vanilla-only registry. Dust channels must be in `0.0..=1.0`; scale must be
positive; spread and speed must be finite and non-negative; count must be
non-zero.

Fallible geometry helpers (`try_circle`, `try_sphere`, `try_helix`,
`try_torus`, `try_grid`, and the corresponding `try_*` family) reject empty,
non-finite, negative, or overflowing plans instead of silently returning an
empty command list. `Particle::raw_token` is the opaque escape hatch.

## Sounds and VFX

`Sound` validates custom/vanilla event IDs by resource-location shape, typed
selectors and sources, finite non-negative volume/minimum volume, positive
pitch, and coordinates. Sand intentionally does not impose an undocumented
maximum volume or pitch:

```rust,ignore
Sound::play("my_pack:boss.roar")
    .source(SoundSource::Hostile)
    .to(Target::players())
    .volume(1.5)
    .pitch(0.8);

Sound::stop_event(
    Target::players(),
    SoundSource::Hostile,
    "my_pack:boss.roar",
);
```

`Vfx::try_play` delegates to the same particle and sound nodes; it does not
maintain separate numeric rules. Raw VFX command steps remain user-owned.

## Effect durations

Minecraft's `effect give` command stores duration in whole seconds. Prefer
`.seconds(n)` or `.infinite()`. Compatibility `.duration(Ticks)` is accepted
only when the tick count is a positive multiple of 20; Sand never truncates:

```rust,ignore
cmd::effect_give(Target::self_(), EffectId::Speed).seconds(10);
cmd::effect_give(Target::self_(), EffectId::Strength).infinite();
```

Typed seconds are limited to Minecraft's `1..=1_000_000` command domain.
`infinite` is version-gated to 1.19.4+. Positional defaults remain stable:
amplifier or hidden-particle options insert Minecraft's 30-second default only
when required by the command grammar. `effect_give_raw` remains the opt-out.

## Migrating broad string calls

Prefer `Target`, `BossbarId`, typed enums, `TextComponent`, and validated
resource IDs. Existing bossbar string IDs are checked at export for
compatibility; new code should parse once into `BossbarId`. Replace raw
actionbar selectors with `Target`, and use explicit `_raw` constructors only
where Sand cannot model the syntax.
