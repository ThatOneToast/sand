# 13. Particles And Sounds

The grapple dash needs to *feel* like something happening, not just change
scoreboard values silently. Trailforge packages its particle and sound
feedback into one reusable sequence:

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:vfx}}
```

## `Vfx` as a named, reusable sequence

`Vfx::new("grapple_dash")` groups a `VfxParticle` and a `VfxSound` under one
named effect rather than issuing two independent `particle` and
`playsound` commands inline wherever the dash fires. The benefit shows up
at the call site (chapter 8's `trail:grapple/execute`):

```rust,ignore
grapple_vfx().play_at(Target::self_());
```

One call plays the whole sequence at the target selector's position. If
Trailforge later wants to add a third layer (say, a screen shake via
camera shake, or a second particle burst), it changes in exactly one place
— the `Vfx` definition — rather than every call site that plays the dash
effect.

Use `try_play()` when constructing a VFX plan outside a function body and you
want immediate feedback. Export validates the typed nodes again, so malformed
IDs, non-finite values, zero counts, and empty geometry cannot reach generated
`.mcfunction` output.

## Particle tuning

```rust,ignore
VfxParticle::raw_token("minecraft:cloud")
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

Reusing the vanilla ender pearl throw sound keeps Trailforge's audio grounded in a sound players
already associate with "launched through the air," which is exactly the
sensation the dash is going for. `.source(SoundSource::Player)` picks the
sound category the client's audio mixer uses (so players can turn dash
sounds down via their own "Players" volume slider without muting hostile
mobs); `.pitch(1.4)` shifts the sound higher than the pearl's default
throw pitch, differentiating "you dashed" from "you threw a pearl" even
though the underlying sample is identical.

## Instrument and jukebox sound definitions

`Instrument` and `JukeboxSong` share `SoundEventDefinition` from
`sand::component`. A reference names an existing sound-event registry entry;
an inline definition names a resource-pack sound directly. Neither form
packages audio assets. Custom registered IDs require the target event to exist
when Minecraft loads the pack.

```rust,ignore
use sand::component::{Instrument, JukeboxSong, SoundEventDefinition};
use sand::registry::SoundEventId;
use sand::text::TextComponent;
use sand::vanilla::SoundEvent;

let vanilla = SoundEventDefinition::reference(SoundEvent::BlockNoteBlockHarp);
let custom = SoundEventDefinition::reference(
    "example:registered_chime".parse::<SoundEventId>().unwrap(),
);
let inline = SoundEventDefinition::inline("example:chime".parse().unwrap(), Some(32.0));
let horn = Instrument::new("example:horn".parse().unwrap())
    .sound_event(inline.clone())
    .range(256.0)
    .description(TextComponent::literal("Chime horn"));
let song = JukeboxSong::new("example:song".parse().unwrap())
    .sound_event(inline)
    .song_length(180.0)
    .comparator_output(0)
    .description(TextComponent::literal("Chime song"));
```

Inline `Some(32.0)` becomes `{"sound_id":"example:chime","range":32.0}`;
`None` omits `range` for Minecraft's volume-dependent behavior. This nested
range is independent of the instrument's positive outer `range`. Minecraft
26.2 accepts finite negative and zero inline ranges; Sand rejects NaN and
infinity before export. The old string-only `.sound_event("…")` setter is
replaced by the shared typed definition.

`SoundEventDefinition::raw(RawJson)` preserves unknown extension fields for
unsupported forms. It still validates the reference or required inline
`sound_id`, and any known `range` must be a finite f32 number. Selecting a new
sound replaces the entire previous typed or raw value; fields are never merged.
