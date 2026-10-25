# Biome ambient audio: Minecraft 26.2

The bounded #182 slice replaces the ignored `BiomeEffects::ambient_sound`,
`raw_ambient_sound`, `mood_sound`, and `additions_sound` APIs with
`Biome::ambient_sounds(BiomeAmbientSounds)` and `Biome::raw_ambient_sounds`.
It writes `attributes.minecraft:audio/ambient_sounds`, matching the bundled
26.2 `soul_sand_valley` biome. This is not complete biome effects parity.
Legacy `music` and `particle` setters still write ignored effects fields;
background music and particle environment attributes remain future #182 work.
Other visual attributes and noise settings also remain outside this slice.

```rust
use sand::component::{Biome, BiomeEffects, BiomeAmbientSounds,
    BiomeMoodSound, BiomeAdditionsSound, SoundEventDefinition};
let cave = SoundEventDefinition::reference(
    sand::registry::SoundEventId::minecraft("ambient.cave").unwrap());
let biome = Biome::new("example:cave".parse().unwrap(),
    BiomeEffects::new(0, 0, 0, 0))
    .ambient_sounds(BiomeAmbientSounds::new()
        .mood_sound(BiomeMoodSound::new(cave.clone(), 6000, 8, 2.0))
        .additions_sound([BiomeAdditionsSound::new(cave, 0.01)]));
```

## Native contract

Inspection of the cached 26.2 server's actual bytecode establishes:

- `world.attribute.AmbientMoodSettings.CODEC`: required `sound` uses
  `SoundEvent.CODEC`; `tick_delay` and `block_search_extent` use `Codec.INT`;
  `offset` uses `Codec.DOUBLE`. No field has a default or sign bound.
- `AmbientAdditionsSettings.CODEC`: required sound holder and unrestricted
  `Codec.DOUBLE` `tick_chance`. Sand deliberately does not impose `0..1`.
- `AmbientSounds.CODEC`: optional `loop` and `mood`; `additions` defaults to an
  empty list and accepts either an object or list via `compactListCodec`.
- `BiomeSpecialEffects.CODEC` no longer contains ambient audio fields.
  A server accepting old JSON would not prove those fields had an effect.

Sand rejects non-finite floats and raw integer fields outside signed 32-bit
integer representation before export. Inline ranges retain the shared sound
model's finite-f32 validation. Sound references require an existing registry
entry at load time; inline definitions name resource-pack audio. No audio assets
are packaged, and neither the tests nor probe establish audible playback.

There is no general biome attribute-map API in the current builder. This slice
owns only the ambient audio attribute and adds no generic attribute framework.
Typed and raw assignments replace that same attribute. Raw audio preserves
unknown extension members, rejects malformed known members, and rejects legacy
field names or enclosing `attributes`/`effects` maps rather than silently
accepting misplaced settings.

## Maintained validation

`biome_ambient_sounds` component tests cover exact output, negative and above-one
chances, integer extremes, non-finite numbers, shared reference/inline holders,
raw compact additions, extension preservation, nested owner diagnostics, and
replacement semantics. The facade test covers namespace A/B/A export isolation
and deterministic attribute output. Existing compiler collision validation
continues to own resource identity.

The `audit_biome_ambient_sounds` registration in `sand-vanilla-audit` exports five
biomes: a registry reference and inline definitions with omitted, 32, zero, and
negative ranges. Each has a loop, mood, and two additions with chances -1 and 2.
`scripts/mc_validation/BiomeAmbientSoundProbe.java` decodes those exported full
biomes, reads the actual environment attribute, verifies every holder and
numeric setting, and requires the attribute to survive re-encoding. It also
checks native defaults, signed codec boundaries, compact additions, and missing
required mood fields. Merely loading a datapack is insufficient evidence.

After building the maintained audit pack from a clean commit, run the maintained
harness with Java 25 and the cached 26.2 bootstrap JAR. It binds to loopback on an
allocated port, resolves all five biomes with `fillbiome`, reloads, shuts down,
and runs the getter/re-encode probe against the unchanged exported pack. The
output includes source/fixture/harness/JAR/pack hashes and retained probe output.
Use an unused output directory:

```sh
python3 scripts/validate_biome_ambient_sounds.py --version 26.2 \
  --pack sand-vanilla-audit/dist/sand_audit \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --output /tmp/biome-audio-proof --timeout 180
```

Native exported-fixture validation passed on clean source commit
`042a7f8b438b5fb5f9dd7228278d3cf52f877ac4`. The verbatim
[machine evidence](biome-ambient-sound-evidence.json) records all five in-world
biome assertions, startup/reload/shutdown, and successful native getters and
re-encoding for every fixture, plus codec boundary/default checks. It includes
source, fixture, harness, bootstrap/extracted JAR, and pack hashes. The evidence
was committed afterward; its source commit identifies the tested code.

The harness fills a region within loaded chunk 0,0 and checks its interior.
Native `BiomeManager` subtracts two from block coordinates and selects among
eight neighboring quart cells using jitter; filling only one cell does not
necessarily cover the queried block's sampled biome. The original failed run
is preserved at `/tmp/sand182-native-bc4277b2`. A second incomplete run at
`/tmp/sand182-native-8d188664` passed all server checks but exposed the probe's
bootstrap registry-owner mismatch during reference encoding. The probe now
uses actual built-in registry owners alongside vanilla dynamic lookups. Neither
correction removes an assertion or changes exported resources. The complete run
is at `/tmp/sand182-native-042a7f8b`.

Focused component Clippy with warnings denied and all 29 ambient/canonical
integration tests passed after the equivalent integer-validation lint fix.
Facade, API discovery, CLI catalog, normal profile, and true invalid-version
placeholder checks passed earlier. Interim profile counts are 11143 normal and
5043 placeholder. Final integration must regenerate affected canonical profiles
after the pending living API changes and run the coordinated full workspace
gate; that full gate is not claimed here.
