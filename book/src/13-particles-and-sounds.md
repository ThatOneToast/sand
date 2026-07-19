# 13. Particles And Sounds

The grapple dash needs to *feel* like something happening, not just change
scoreboard values silently. Trailforge packages its particle and sound
feedback into one reusable sequence:

{{#include ../../examples/book_project/src/lib.rs:vfx}}

## `Vfx` as a named, reusable sequence

`Vfx::new("grapple_dash")` groups a `VfxParticle` and a `VfxSound` under one
named effect rather than issuing two independent `particle` and
`playsound` commands inline wherever the dash fires. The benefit shows up
at the call site (chapter 8's `trail:grapple/execute`):

```rust,ignore
grapple_vfx().play_at(Selector::self_());
```

One call plays the whole sequence at the target selector's position. If
Trailforge later wants to add a third layer (say, a screen shake via
camera shake, or a second particle burst), it changes in exactly one place
— the `Vfx` definition — rather than every call site that plays the dash
effect.

## Particle tuning

```rust,ignore
VfxParticle::named("minecraft:cloud")
    .count(24)
    .spread(0.4, 0.2, 0.4)
```

`"minecraft:cloud"` is a vanilla particle ID (the same registry
`/particle` uses); `.count(24)` and `.spread(0.4, 0.2, 0.4)` map directly
to the `/particle` command's count and delta arguments — a modest burst
centered on the player, wider on the horizontal axes than vertically, to
read as a ground-level dash puff rather than a vertical fountain.

## Sound tuning

```rust,ignore
VfxSound::new("minecraft:entity.ender_pearl.throw")
    .source(SoundSource::Player)
    .volume(0.8)
    .pitch(1.4)
```

Reusing the vanilla ender pearl throw sound (rather than needing a custom
resource-pack sound) keeps Trailforge's audio grounded in a sound players
already associate with "launched through the air," which is exactly the
sensation the dash is going for. `.source(SoundSource::Player)` picks the
sound category the client's audio mixer uses (so players can turn dash
sounds down via their own "Players" volume slider without muting hostile
mobs); `.pitch(1.4)` shifts the sound higher than the pearl's default
throw pitch, differentiating "you dashed" from "you threw a pearl" even
though the underlying sample is identical.

## Resource packs are a separate, opt-in system

Everything in this chapter uses vanilla particle and sound IDs — Trailforge
never needs a custom resource pack for its VFX. Sand does support authoring
resource-pack assets (HUD bars, custom textures) behind the `resourcepack`
feature and `sand add resourcepack`, but that's a separate system from the
datapack `Vfx` sequencing shown here, and Trailforge doesn't use it; if your
own pack needs custom particle textures or sounds, that's where you'd start
instead.
