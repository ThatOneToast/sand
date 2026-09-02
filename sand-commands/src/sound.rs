//! Builder for the `playsound` command.
//!
//! # Example
//! ```rust,ignore
//! let cmd = Sound::play("minecraft:entity.experience_orb.pickup")
//!     .to(Selector::self_())
//!     .source(SoundSource::Player)
//!     .volume(1.0)
//!     .pitch(1.2)
//!     .build();
//! // → "playsound minecraft:entity.experience_orb.pickup player @s ~ ~ ~ 1 1.2"
//!
//! let cmd = Sound::stop_all(Selector::all_players());
//! // → "stopsound @a"
//! ```

use std::collections::BTreeMap;
use std::fmt;

use crate::Build;
use crate::coord::Vec3;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;

// ── SoundSource ───────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::IntoSoundEvent",
    aliases = ["sand::cmd::IntoSoundEvent", "sand::prelude::cmd::IntoSoundEvent"],
    module = "sand::command",
    summary = "Conversion into a sound-event resource-location token.",
    context = "Conversion into a sound-event resource-location token. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::IntoSoundEvent;",
)]
/// Conversion into a sound-event resource-location token.
pub trait IntoSoundEvent {
    /// Converts a typed or validated value into a Minecraft sound-event identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::IntoSoundEvent::into_sound_event",
        aliases = ["sand::cmd::IntoSoundEvent::into_sound_event", "sand::prelude::cmd::IntoSoundEvent::into_sound_event"],
        module = "sand::command",
        summary = "Converts a typed or validated value into a Minecraft sound-event identifier.",
        context = "Converts a typed or validated value into a Minecraft sound-event identifier. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to convert a typed or validated value into a Minecraft sound-event identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::IntoSoundEvent>(into_sound_event_value: T)  {\n    let into_sound_event = into_sound_event_value.into_sound_event();\n}",
    )]
    fn into_sound_event(self) -> String;
}

impl IntoSoundEvent for String {
    fn into_sound_event(self) -> String {
        self
    }
}

impl IntoSoundEvent for &str {
    fn into_sound_event(self) -> String {
        self.to_string()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::SoundSource",
    aliases = ["sand::cmd::SoundSource", "sand::prelude::SoundSource", "sand::prelude::cmd::SoundSource"],
    module = "sand::command",
    summary = "Minecraft audio channel/category for sound playback.",
    context = "Minecraft audio channel/category for sound playback. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::SoundSource;",
    variants(Ambient = "Selects the ambient form of the sound source Minecraft command value.", Block = "Selects the block form of the sound source Minecraft command value.", Hostile = "Selects the hostile form of the sound source Minecraft command value.", Master = "Selects the master form of the sound source Minecraft command value.", Music = "Selects the music form of the sound source Minecraft command value.", Neutral = "Selects the neutral form of the sound source Minecraft command value.", Player = "Selects the player form of the sound source Minecraft command value.", Record = "Selects the record form of the sound source Minecraft command value.", Ui = "Selects the ui form of the sound source Minecraft command value.", Voice = "Selects the voice form of the sound source Minecraft command value.", Weather = "Selects the weather form of the sound source Minecraft command value."),
)]
/// Minecraft audio channel/category for sound playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    #[doc = "Selects the master form of the sound source Minecraft command value."]
    Master,
    #[doc = "Selects the music form of the sound source Minecraft command value."]
    Music,
    #[doc = "Selects the record form of the sound source Minecraft command value."]
    Record,
    #[doc = "Selects the weather form of the sound source Minecraft command value."]
    Weather,
    #[doc = "Selects the block form of the sound source Minecraft command value."]
    Block,
    #[doc = "Selects the hostile form of the sound source Minecraft command value."]
    Hostile,
    #[doc = "Selects the neutral form of the sound source Minecraft command value."]
    Neutral,
    #[doc = "Selects the player form of the sound source Minecraft command value."]
    Player,
    #[doc = "Selects the ui form of the sound source Minecraft command value."]
    Ui,
    #[doc = "Selects the ambient form of the sound source Minecraft command value."]
    Ambient,
    #[doc = "Selects the voice form of the sound source Minecraft command value."]
    Voice,
}

impl fmt::Display for SoundSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SoundSource::Master => "master",
            SoundSource::Music => "music",
            SoundSource::Record => "record",
            SoundSource::Weather => "weather",
            SoundSource::Block => "block",
            SoundSource::Hostile => "hostile",
            SoundSource::Neutral => "neutral",
            SoundSource::Player => "player",
            SoundSource::Ui => "ui",
            SoundSource::Ambient => "ambient",
            SoundSource::Voice => "voice",
        };
        f.write_str(s)
    }
}

// ── Sound ─────────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Sound",
    aliases = ["sand::cmd::Sound", "sand::prelude::Sound", "sand::prelude::cmd::Sound"],
    module = "sand::command",
    summary = "Builder for `playsound` commands.",
    context = "Builder for `playsound` commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Sound;",
)]
/// Builder for `playsound` commands.
#[derive(Debug, Clone)]
pub struct Sound {
    event: String,
    raw_event: bool,
    source: SoundSource,
    target: Option<Selector>,
    pos: Option<Vec3>,
    volume: f64,
    pitch: f64,
    min_volume: Option<f64>,
}

impl Sound {
    /// Begin building a `playsound` command for the given sound event ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::play",
        aliases = ["sand::cmd::Sound::play", "sand::prelude::Sound::play", "sand::prelude::cmd::Sound::play"],
        module = "sand::command",
        kind = "method",
        summary = "Begin building a `playsound` command for the given sound event ID.",
        context = "Begin building a `playsound` command for the given sound event ID. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(event = "`event` supplies the event value used to begin building a `playsound` command for the given sound event ID."),
        returns = "A newly constructed `Sound` configured to begin building a `playsound` command for the given sound event ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate(event: impl sand::command::IntoSoundEvent)  {\n    let sound = sand::command::Sound::play(event);\n}",
    )]
    pub fn play(event: impl IntoSoundEvent) -> Self {
        Self {
            event: event.into_sound_event(),
            raw_event: false,
            source: SoundSource::Master,
            target: None,
            pos: None,
            volume: 1.0,
            pitch: 1.0,
            min_volume: None,
        }
    }

    /// Begin building a sound command with an intentionally opaque event token.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::play_raw",
        aliases = ["sand::cmd::Sound::play_raw", "sand::prelude::Sound::play_raw", "sand::prelude::cmd::Sound::play_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Begin building a sound command with an intentionally opaque event token.",
        context = "Begin building a sound command with an intentionally opaque event token. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(event = "`event` supplies the event value used to begin building a sound command with an intentionally opaque event token."),
        returns = "A newly constructed `Sound` configured to begin building a sound command with an intentionally opaque event token.",
        example = "use sand::prelude::*;\n\nfn demonstrate(event: impl sand::command::IntoSoundEvent)  {\n    let sound = sand::command::Sound::play_raw(event);\n}",
    )]
    pub fn play_raw(event: impl IntoSoundEvent) -> Self {
        Self {
            raw_event: true,
            ..Self::play(event)
        }
    }

    /// Set the target entity/player who hears the sound (default: `@s`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::to",
        aliases = ["sand::cmd::Sound::to", "sand::prelude::Sound::to", "sand::prelude::cmd::Sound::to"],
        module = "sand::command",
        kind = "method",
        summary = "Set the target entity/player who hears the sound (default: `@s`).",
        context = "Set the target entity/player who hears the sound (default: `@s`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to set the target entity/player who hears the sound (default: `@s`)."),
        returns = "The `Sound` value with the documented change applied to set the target entity/player who hears the sound (default: `@s`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(sound_value: sand::command::Sound, selector: sand::command::Selector)  {\n    let updated_sound = sound_value.to(selector);\n}",
    )]
    pub fn to(mut self, selector: Selector) -> Self {
        self.target = Some(selector);
        self
    }

    /// Set the sound source/channel category (default: `master`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::source",
        aliases = ["sand::cmd::Sound::source", "sand::prelude::Sound::source", "sand::prelude::cmd::Sound::source"],
        module = "sand::command",
        kind = "method",
        summary = "Set the sound source/channel category (default: `master`).",
        context = "Set the sound source/channel category (default: `master`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(source = "`source` supplies the source value used to set the sound source/channel category (default: `master`)."),
        returns = "The `Sound` value with the documented change applied to set the sound source/channel category (default: `master`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(sound_value: sand::command::Sound, source: sand::command::SoundSource)  {\n    let updated_sound = sound_value.source(source);\n}",
    )]
    pub fn source(mut self, source: SoundSource) -> Self {
        self.source = source;
        self
    }

    /// Set the position in the world where the sound originates (default: `~ ~ ~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::at",
        aliases = ["sand::cmd::Sound::at", "sand::prelude::Sound::at", "sand::prelude::cmd::Sound::at"],
        module = "sand::command",
        kind = "method",
        summary = "Set the position in the world where the sound originates (default: `~ ~ ~`).",
        context = "Set the position in the world where the sound originates (default: `~ ~ ~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to set the position in the world where the sound originates (default: `~ ~ ~`)."),
        returns = "The `Sound` value with the documented change applied to set the position in the world where the sound originates (default: `~ ~ ~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(sound_value: sand::command::Sound, pos: sand::command::Vec3)  {\n    let updated_sound = sound_value.at(pos);\n}",
    )]
    pub fn at(mut self, pos: Vec3) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Set the volume multiplier (default: `1.0`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::volume",
        aliases = ["sand::cmd::Sound::volume", "sand::prelude::Sound::volume", "sand::prelude::cmd::Sound::volume"],
        module = "sand::command",
        kind = "method",
        summary = "Set the volume multiplier (default: `1.0`).",
        context = "Set the volume multiplier (default: `1.0`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(volume = "`volume` supplies the volume value used to set the volume multiplier (default: `1.0`)."),
        returns = "The `Sound` value with the documented change applied to set the volume multiplier (default: `1.0`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(sound_value: sand::command::Sound, volume: f64)  {\n    let updated_sound = sound_value.volume(volume);\n}",
    )]
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set the pitch multiplier (default: `1.0`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::pitch",
        aliases = ["sand::cmd::Sound::pitch", "sand::prelude::Sound::pitch", "sand::prelude::cmd::Sound::pitch"],
        module = "sand::command",
        kind = "method",
        summary = "Set the pitch multiplier (default: `1.0`).",
        context = "Set the pitch multiplier (default: `1.0`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pitch = "`pitch` supplies the pitch value used to set the pitch multiplier (default: `1.0`)."),
        returns = "The `Sound` value with the documented change applied to set the pitch multiplier (default: `1.0`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(sound_value: sand::command::Sound, pitch: f64)  {\n    let updated_sound = sound_value.pitch(pitch);\n}",
    )]
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set minimum volume for players far from the sound origin.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::min_volume",
        aliases = ["sand::cmd::Sound::min_volume", "sand::prelude::Sound::min_volume", "sand::prelude::cmd::Sound::min_volume"],
        module = "sand::command",
        kind = "method",
        summary = "Set minimum volume for players far from the sound origin.",
        context = "Set minimum volume for players far from the sound origin. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`min` provides the inclusive lower bound used to set minimum volume for players far from the sound origin."),
        returns = "The `Sound` value with the documented change applied to set minimum volume for players far from the sound origin.",
        example = "use sand::prelude::*;\n\nfn demonstrate(sound_value: sand::command::Sound, min: f64)  {\n    let updated_sound = sound_value.min_volume(min);\n}",
    )]
    pub fn min_volume(mut self, min: f64) -> Self {
        self.min_volume = Some(min);
        self
    }

    // ── stopsound helpers ─────────────────────────────────────────────────────

    /// `stopsound <selector>` — stop all sounds playing for the target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::stop_all",
        aliases = ["sand::cmd::Sound::stop_all", "sand::prelude::Sound::stop_all", "sand::prelude::cmd::Sound::stop_all"],
        module = "sand::command",
        kind = "method",
        summary = "`stopsound <selector>` — stop all sounds playing for the target.",
        context = "`stopsound <selector>` — stop all sounds playing for the target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to emit the documented `stopsound <selector>` — stop all sounds playing for the target form."),
        returns = "The string value produced to emit the documented `stopsound <selector>` — stop all sounds playing for the target form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector)  {\n    let stop_all = sand::command::Sound::stop_all(target);\n}",
    )]
    pub fn stop_all(target: Selector) -> String {
        StopSoundCommand::All { target }.build_registered()
    }

    /// `stopsound <selector> <source>` — stop all sounds in a specific category.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::stop_source",
        aliases = ["sand::cmd::Sound::stop_source", "sand::prelude::Sound::stop_source", "sand::prelude::cmd::Sound::stop_source"],
        module = "sand::command",
        kind = "method",
        summary = "`stopsound <selector> <source>` — stop all sounds in a specific category.",
        context = "`stopsound <selector> <source>` — stop all sounds in a specific category. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to emit the documented `stopsound <selector> <source>` — stop all sounds in a specific category form.", source = "`source` supplies the source value used to emit the documented `stopsound <selector> <source>` — stop all sounds in a specific category form."),
        returns = "The string value produced to emit the documented `stopsound <selector> <source>` — stop all sounds in a specific category form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, source: sand::command::SoundSource)  {\n    let stop_source = sand::command::Sound::stop_source(target, source);\n}",
    )]
    pub fn stop_source(target: Selector, source: SoundSource) -> String {
        StopSoundCommand::Source { target, source }.build_registered()
    }

    /// `stopsound <selector> <source> <event>` — stop a specific sound for the target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::stop_event",
        aliases = ["sand::cmd::Sound::stop_event", "sand::prelude::Sound::stop_event", "sand::prelude::cmd::Sound::stop_event"],
        module = "sand::command",
        kind = "method",
        summary = "`stopsound <selector> <source> <event>` — stop a specific sound for the target.",
        context = "`stopsound <selector> <source> <event>` — stop a specific sound for the target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to emit the documented `stopsound <selector> <source> <event>` — stop a specific sound for the target form.", source = "`source` supplies the source value used to emit the documented `stopsound <selector> <source> <event>` — stop a specific sound for the target form.", event = "`event` supplies the event value used to emit the documented `stopsound <selector> <source> <event>` — stop a specific sound for the target form."),
        returns = "The string value produced to emit the documented `stopsound <selector> <source> <event>` — stop a specific sound for the target form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, source: sand::command::SoundSource, event: impl Into < String >)  {\n    let stop_event = sand::command::Sound::stop_event(target, source, event);\n}",
    )]
    pub fn stop_event(target: Selector, source: SoundSource, event: impl Into<String>) -> String {
        StopSoundCommand::Event {
            target,
            source,
            event: event.into(),
            raw_event: false,
        }
        .build_registered()
    }

    /// Compatibility alias for [`Sound::stop_event`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::stop",
        aliases = ["sand::cmd::Sound::stop", "sand::prelude::Sound::stop", "sand::prelude::cmd::Sound::stop"],
        module = "sand::command",
        kind = "method",
        summary = "Compatibility alias for [`Sound::stop_event`].",
        context = "Compatibility alias for [`Sound::stop_event`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to use compatibility alias for [`Sound::stop_event`].", source = "`source` supplies the source value used to use compatibility alias for [`Sound::stop_event`].", event = "`event` supplies the event value used to use compatibility alias for [`Sound::stop_event`]."),
        returns = "The string value produced to use compatibility alias for [`Sound::stop_event`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, source: sand::command::SoundSource, event: impl Into < String >)  {\n    let stop = sand::command::Sound::stop(target, source, event);\n}",
    )]
    pub fn stop(target: Selector, source: SoundSource, event: impl Into<String>) -> String {
        Self::stop_event(target, source, event)
    }

    /// Stop a sound with an intentionally opaque event token.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Sound::stop_event_raw",
        aliases = ["sand::cmd::Sound::stop_event_raw", "sand::prelude::Sound::stop_event_raw", "sand::prelude::cmd::Sound::stop_event_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Stop a sound with an intentionally opaque event token.",
        context = "Stop a sound with an intentionally opaque event token. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to stop a sound with an intentionally opaque event token.", source = "`source` supplies the source value used to stop a sound with an intentionally opaque event token.", event = "`event` supplies the event value used to stop a sound with an intentionally opaque event token."),
        returns = "The string value produced to stop a sound with an intentionally opaque event token.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, source: sand::command::SoundSource, event: impl Into < String >)  {\n    let stop_event_raw = sand::command::Sound::stop_event_raw(target, source, event);\n}",
    )]
    pub fn stop_event_raw(
        target: Selector,
        source: SoundSource,
        event: impl Into<String>,
    ) -> String {
        StopSoundCommand::Event {
            target,
            source,
            event: event.into(),
            raw_event: true,
        }
        .build_registered()
    }
}

impl Build for Sound {
    /// Build the complete `playsound` command string.
    ///
    /// Defaults: target=`@s`, position=`~ ~ ~`.
    fn build(&self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, SoundCommand::Play(self.clone()));
        line
    }
}

impl Validate for Sound {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        if !self.raw_event {
            crate::validate::resource_location_shape(&self.event, "SoundCommand", "event")
                .map_err(|error| sound_error("SAND-SOUND-ID", error.field, error.message))?;
        }
        self.target
            .as_ref()
            .unwrap_or(&Selector::self_())
            .validate(profile)
            .map_err(|error| sound_error("SAND-SOUND-TARGET", "target", error.to_string()))?;
        validate_non_negative(self.volume, "volume")?;
        validate_positive(self.pitch, "pitch")?;
        if let Some(minimum) = self.min_volume {
            validate_non_negative(minimum, "minimum_volume")?;
        }
        if let Some(position) = &self.pos {
            for (field, value) in [
                ("position.x", &position.x),
                ("position.y", &position.y),
                ("position.z", &position.z),
            ] {
                let text = value.to_string();
                if text.contains("NaN") || text.contains("inf") {
                    return Err(sound_error(
                        "SAND-SOUND-NUMERIC",
                        field,
                        format!("coordinates must be finite, got `{text}`"),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl RenderCommand for Sound {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        let target = self.target.clone().unwrap_or_else(Selector::self_);
        let pos = self.pos.clone().unwrap_or_else(Vec3::here);

        let mut s = format!(
            "playsound {} {} {} {} {} {}",
            self.event, self.source, target, pos, self.volume, self.pitch
        );

        if let Some(mv) = self.min_volume {
            s.push(' ');
            s.push_str(&format_float(mv));
        }

        s
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::StopSoundCommand",
    aliases = ["sand::cmd::StopSoundCommand", "sand::prelude::cmd::StopSoundCommand"],
    module = "sand::command",
    summary = "Structured forms of `stopsound`.",
    context = "Structured forms of `stopsound`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::StopSoundCommand;",
    variants(All = "Selects the all form of the stop sound command Minecraft command value.", Event = "Selects the event form of the stop sound command Minecraft command value.", Source = "Selects the source form of the stop sound command Minecraft command value."),
    variant_fields(All(target = "`target` provides the command target when the variant selects the all form of the stop sound command Minecraft command value."), Event(event = "`event` provides the event when the variant selects the event form of the stop sound command Minecraft command value.", raw_event = "`raw_event` provides the raw event when the variant selects the event form of the stop sound command Minecraft command value.", source = "`source` provides the source when the variant selects the event form of the stop sound command Minecraft command value.", target = "`target` provides the command target when the variant selects the event form of the stop sound command Minecraft command value."), Source(source = "`source` provides the source when the variant selects the source form of the stop sound command Minecraft command value.", target = "`target` provides the command target when the variant selects the source form of the stop sound command Minecraft command value.")),
)]
/// Structured forms of `stopsound`.
#[derive(Debug, Clone)]
pub enum StopSoundCommand {
    #[doc = "Selects the all form of the stop sound command Minecraft command value."]
    All {
        /// `target` provides the command target when the variant selects the all form of the stop sound command Minecraft command value.
        target: Selector,
    },
    #[doc = "Selects the source form of the stop sound command Minecraft command value."]
    Source {
        /// `target` provides the command target when the variant selects the source form of the stop sound command Minecraft command value.
        target: Selector,
        /// `source` provides the source when the variant selects the source form of the stop sound command Minecraft command value.
        source: SoundSource,
    },
    #[doc = "Selects the event form of the stop sound command Minecraft command value."]
    Event {
        /// `target` provides the command target when the variant selects the event form of the stop sound command Minecraft command value.
        target: Selector,
        /// `source` provides the source when the variant selects the event form of the stop sound command Minecraft command value.
        source: SoundSource,
        /// `event` provides the event when the variant selects the event form of the stop sound command Minecraft command value.
        event: String,
        /// `raw_event` provides the raw event when the variant selects the event form of the stop sound command Minecraft command value.
        raw_event: bool,
    },
}

impl StopSoundCommand {
    fn build_registered(self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, SoundCommand::Stop(self));
        line
    }
}

impl Validate for StopSoundCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        let target = match self {
            Self::All { target } | Self::Source { target, .. } | Self::Event { target, .. } => {
                target
            }
        };
        target
            .validate(profile)
            .map_err(|error| sound_error("SAND-SOUND-TARGET", "target", error.to_string()))?;
        if let Self::Event {
            event,
            raw_event: false,
            ..
        } = self
        {
            crate::validate::resource_location_shape(event, "SoundCommand", "event")
                .map_err(|error| sound_error("SAND-SOUND-ID", error.field, error.message))?;
        }
        Ok(())
    }
}

impl RenderCommand for StopSoundCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        match self {
            Self::All { target } => format!("stopsound {target}"),
            Self::Source { target, source } => format!("stopsound {target} {source}"),
            Self::Event {
                target,
                source,
                event,
                ..
            } => format!("stopsound {target} {source} {event}"),
        }
    }
}

impl fmt::Display for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<Sound> for String {
    fn from(v: Sound) -> Self {
        v.build()
    }
}

fn format_float(v: f64) -> String {
    if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SoundCommand {
    Play(Sound),
    Stop(StopSoundCommand),
}

impl Validate for SoundCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Play(command) => command.validate(profile),
            Self::Stop(command) => command.validate(profile),
        }
    }
}

fn sound_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("SoundCommand", field, message).with_code(code)
}

fn validate_non_negative(value: f64, field: &'static str) -> CommandResult<()> {
    if !value.is_finite() || value < 0.0 {
        Err(sound_error(
            "SAND-SOUND-NUMERIC",
            field,
            format!("must be finite and non-negative, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_positive(value: f64, field: &'static str) -> CommandResult<()> {
    if !value.is_finite() || value <= 0.0 {
        Err(sound_error(
            "SAND-SOUND-NUMERIC",
            field,
            format!("must be finite and greater than zero, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

/// Export-scoped registry family holding this module's rendered
/// sound command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct SoundLines;

impl crate::export_registry::RegistryFamily for SoundLines {
    type State = BTreeMap<String, SoundCommand>;
}

fn register_line(line: &str, command: SoundCommand) {
    crate::export_registry::register_line::<SoundLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<SoundLines, _>(
        line,
        profile,
        |command, profile| command.validate(profile),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_playsound() {
        let cmd = Sound::play("minecraft:entity.experience_orb.pickup")
            .to(Selector::self_())
            .source(SoundSource::Player)
            .build();
        assert_eq!(
            cmd,
            "playsound minecraft:entity.experience_orb.pickup player @s ~ ~ ~ 1 1"
        );
    }

    #[test]
    fn custom_volume_pitch() {
        let cmd = Sound::play("minecraft:block.note_block.bell")
            .to(Selector::all_players())
            .volume(2.0)
            .pitch(0.5)
            .build();
        assert!(cmd.contains("2 0.5"), "{}", cmd);
    }

    #[test]
    fn min_volume() {
        let cmd = Sound::play("minecraft:ambient.cave")
            .to(Selector::self_())
            .min_volume(0.3)
            .build();
        assert!(cmd.ends_with("0.3"), "{}", cmd);
    }

    #[test]
    fn stopsound() {
        assert_eq!(Sound::stop_all(Selector::all_players()), "stopsound @a");
        assert_eq!(
            Sound::stop_source(Selector::all_players(), SoundSource::Music),
            "stopsound @a music"
        );
        assert_eq!(
            Sound::stop(
                Selector::all_players(),
                SoundSource::Block,
                "minecraft:block.stone.hit"
            ),
            "stopsound @a block minecraft:block.stone.hit"
        );
    }

    #[test]
    fn validates_ids_and_numeric_domains() {
        assert!(Sound::play("modded:custom.event").try_build().is_ok());
        assert_eq!(
            Sound::play("Bad Event").try_build().unwrap_err().code,
            "SAND-SOUND-ID"
        );
        for sound in [
            Sound::play("minecraft:test").volume(f64::NAN),
            Sound::play("minecraft:test").volume(-1.0),
            Sound::play("minecraft:test").pitch(0.0),
            Sound::play("minecraft:test").min_volume(-0.1),
        ] {
            assert_eq!(sound.try_build().unwrap_err().code, "SAND-SOUND-NUMERIC");
        }
    }

    #[test]
    fn raw_sound_event_is_opaque() {
        assert!(Sound::play_raw("modded payload").try_build().is_ok());
    }
}
