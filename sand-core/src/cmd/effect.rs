use std::fmt;

use sand_commands::{Build, EffectCommand, EffectDuration, RenderCommand, Selector};
use sand_components::{EffectId, Ticks};

use super::Command;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::EffectGive",
    aliases = ["sand::cmd::EffectGive", "sand::prelude::cmd::EffectGive"],
    module = "sand::command",
    summary = "Builder for `effect give`.",
    context = "Builder for `effect give`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::EffectGive;",
)]
/// Builder for `effect give`.
#[derive(Debug, Clone)]
pub struct EffectGive {
    command: EffectCommand,
}

impl EffectGive {
    /// Creates a typed effect give command builder from the supplied command inputs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::new",
        aliases = ["sand::cmd::EffectGive::new", "sand::prelude::cmd::EffectGive::new"],
        module = "sand::command",
        kind = "method",
        summary = "Creates a typed effect give command builder from the supplied command inputs.",
        context = "Creates a typed effect give command builder from the supplied command inputs. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to create a typed effect give command builder from the supplied command inputs.", effect = "`effect` is used when creating a typed effect give command builder from the supplied command inputs."),
        returns = "An `EffectGive` representing a typed effect give command builder from the supplied command inputs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector, effect: impl Into < sand::registry::EffectId >)  {\n    let effect_give = sand::command::EffectGive::new(selector, effect);\n}",
    )]
    pub fn new(selector: Selector, effect: impl Into<EffectId>) -> Self {
        Self {
            command: EffectCommand::give(selector, effect.into().to_string()),
        }
    }

    /// Set command duration using ticks. Minecraft command syntax stores this as seconds.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::duration",
        aliases = ["sand::cmd::EffectGive::duration", "sand::prelude::cmd::EffectGive::duration"],
        module = "sand::command",
        kind = "method",
        summary = "Set command duration using ticks. Minecraft command syntax stores this as seconds.",
        context = "Set command duration using ticks. Minecraft command syntax stores this as seconds. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(duration = "`duration` provides the Minecraft tick duration used to set command duration using ticks. Minecraft command syntax stores this as seconds."),
        returns = "The `EffectGive` value with the documented change applied to set command duration using ticks. Minecraft command syntax stores this as seconds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: sand::command::EffectGive, duration: sand::state::Ticks)  {\n    let updated_effect_give = effect_give_value.duration(duration);\n}",
    )]
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.command = self.command.duration(EffectDuration::ticks(duration.get()));
        self
    }

    /// Set command duration in seconds.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::seconds",
        aliases = ["sand::cmd::EffectGive::seconds", "sand::prelude::cmd::EffectGive::seconds"],
        module = "sand::command",
        kind = "method",
        summary = "Set command duration in seconds.",
        context = "Set command duration in seconds. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(seconds = "`seconds` provides the seconds applied when setting command duration in seconds."),
        returns = "The `EffectGive` value with the documented change applied to set command duration in seconds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: sand::command::EffectGive, seconds: u32)  {\n    let updated_effect_give = effect_give_value.seconds(seconds);\n}",
    )]
    pub fn seconds(mut self, seconds: u32) -> Self {
        self.command = self.command.duration(EffectDuration::seconds(seconds));
        self
    }

    /// Persist until explicitly cleared (Minecraft 1.19.4+).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::infinite",
        aliases = ["sand::cmd::EffectGive::infinite", "sand::prelude::cmd::EffectGive::infinite"],
        module = "sand::command",
        kind = "method",
        summary = "Persist until explicitly cleared (Minecraft 1.19.4+).",
        context = "Persist until explicitly cleared (Minecraft 1.19.4+). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `EffectGive` value with the documented change applied to persist until explicitly cleared (Minecraft 1.19.4+).",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: sand::command::EffectGive)  {\n    let updated_effect_give = effect_give_value.infinite();\n}",
    )]
    pub fn infinite(mut self) -> Self {
        self.command = self.command.duration(EffectDuration::Infinite);
        self
    }

    /// Sets the zero-based amplifier on this typed effect command.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::amplifier",
        aliases = ["sand::cmd::EffectGive::amplifier", "sand::prelude::cmd::EffectGive::amplifier"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the zero-based amplifier on this typed effect command.",
        context = "Sets the zero-based amplifier on this typed effect command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(amplifier = "`amplifier` provides the amplifier applied when setting the zero-based amplifier on this typed effect command."),
        returns = "The `EffectGive` value with the documented change applied to set the zero-based amplifier on this typed effect command.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: sand::command::EffectGive, amplifier: u8)  {\n    let updated_effect_give = effect_give_value.amplifier(amplifier);\n}",
    )]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.command = self.command.amplifier(amplifier);
        self
    }

    /// Control visible particles. `false` serializes to Minecraft's `hideParticles=true`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::particles",
        aliases = ["sand::cmd::EffectGive::particles", "sand::prelude::cmd::EffectGive::particles"],
        module = "sand::command",
        kind = "method",
        summary = "Control visible particles. `false` serializes to Minecraft's `hideParticles=true`.",
        context = "Control visible particles. `false` serializes to Minecraft's `hideParticles=true`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(show_particles = "`show_particles` provides the switch that enables or disables the behavior used to control visible particles. `false` serializes to Minecraft's `hideParticles=true`."),
        returns = "The `EffectGive` value with the documented change applied to control visible particles. `false` serializes to Minecraft's `hideParticles=true`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: sand::command::EffectGive, show_particles: bool)  {\n    let updated_effect_give = effect_give_value.particles(show_particles);\n}",
    )]
    pub fn particles(mut self, show_particles: bool) -> Self {
        self.command = self.command.particles(show_particles);
        self
    }

    /// Renders the configured effect give as validated Minecraft command text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::build",
        aliases = ["sand::cmd::EffectGive::build", "sand::prelude::cmd::EffectGive::build"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the configured effect give as validated Minecraft command text.",
        context = "Renders the configured effect give as validated Minecraft command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The rendered Minecraft command text produced to render the configured effect give as validated Minecraft command text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: &sand::command::EffectGive)  {\n    let command = effect_give_value.build();\n}",
    )]
    pub fn build(&self) -> String {
        self.command.build()
    }

    /// Validate exact whole-second semantics before rendering.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectGive::try_build",
        aliases = ["sand::cmd::EffectGive::try_build", "sand::prelude::cmd::EffectGive::try_build"],
        module = "sand::command",
        kind = "method",
        summary = "Validate exact whole-second semantics before rendering.",
        context = "Validate exact whole-second semantics before rendering. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `sand :: command :: CommandResult < String >` value produced to validate exact whole-second semantics before rendering.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_give_value: &sand::command::EffectGive)  {\n    let try_build = effect_give_value.try_build();\n}",
    )]
    pub fn try_build(&self) -> sand_commands::CommandResult<String> {
        self.command.try_build()
    }
}

impl fmt::Display for EffectGive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl Command for EffectGive {}

impl From<EffectGive> for String {
    fn from(value: EffectGive) -> Self {
        value.build()
    }
}

/// `effect give <selector> <effect>` with typed effect IDs.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::effect_give",
    aliases = ["sand::cmd::effect_give", "sand::prelude::cmd::effect_give"],
    module = "sand::command",
    summary = "`effect give <selector> <effect>` with typed effect IDs.",
    context = "`effect give <selector> <effect>` with typed effect IDs. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "`selector` provides the Minecraft target selection used to emit the documented `effect give <selector> <effect>` with typed effect IDs form.", effect = "`effect` supplies the documented `effect give <selector> <effect>` with typed effect IDs form."),
    returns = "The `EffectGive` value produced to emit the documented `effect give <selector> <effect>` with typed effect IDs form.",
    example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector, effect: impl Into < sand::registry::EffectId >)  {\n    let effect_give = sand::command::effect_give(selector, effect);\n}",
)]
pub fn effect_give(selector: Selector, effect: impl Into<EffectId>) -> EffectGive {
    EffectGive::new(selector, effect)
}

/// `effect clear <selector>` — clear all status effects.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::effect_clear",
    aliases = ["sand::cmd::effect_clear", "sand::prelude::cmd::effect_clear"],
    module = "sand::command",
    summary = "`effect clear <selector>` — clear all status effects.",
    context = "`effect clear <selector>` — clear all status effects. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "`selector` provides the Minecraft target selection used to emit the documented `effect clear <selector>` — clear all status effects form."),
    returns = "The string value produced to emit the documented `effect clear <selector>` — clear all status effects form.",
    example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector)  {\n    let effect_clear = sand::command::effect_clear(selector);\n}",
)]
pub fn effect_clear(selector: Selector) -> String {
    format!("effect clear {selector}")
}

/// `effect clear <selector> <effect>` — clear one typed status effect.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::effect_clear_effect",
    aliases = ["sand::cmd::effect_clear_effect", "sand::prelude::cmd::effect_clear_effect"],
    module = "sand::command",
    summary = "`effect clear <selector> <effect>` — clear one typed status effect.",
    context = "`effect clear <selector> <effect>` — clear one typed status effect. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "`selector` provides the Minecraft target selection used to emit the documented `effect clear <selector> <effect>` — clear one typed status effect form.", effect = "`effect` supplies the documented `effect clear <selector> <effect>` — clear one typed status effect form."),
    returns = "The string value produced to emit the documented `effect clear <selector> <effect>` — clear one typed status effect form.",
    example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector, effect: impl Into < sand::registry::EffectId >)  {\n    let effect_clear_effect = sand::command::effect_clear_effect(selector, effect);\n}",
)]
pub fn effect_clear_effect(selector: Selector, effect: impl Into<EffectId>) -> String {
    format!("effect clear {selector} {}", effect.into())
}

/// Explicit raw escape hatch for unsupported effect command syntax.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::effect_give_raw",
    aliases = ["sand::cmd::effect_give_raw", "sand::prelude::cmd::effect_give_raw"],
    module = "sand::command",
    summary = "Explicit raw escape hatch for unsupported effect command syntax.",
    context = "Explicit raw escape hatch for unsupported effect command syntax. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(selector = "`selector` provides the Minecraft target selection used to use explicit raw escape hatch for unsupported effect command syntax.", effect = "`effect` sets the effect for explicit raw escape hatch for unsupported effect command syntax.", duration_seconds = "`duration_seconds` sets the duration seconds for explicit raw escape hatch for unsupported effect command syntax.", amplifier = "`amplifier` sets the amplifier for explicit raw escape hatch for unsupported effect command syntax.", hide_particles = "`hide_particles` provides the switch that enables or disables the behavior used to use explicit raw escape hatch for unsupported effect command syntax."),
    returns = "The rendered Minecraft command text produced to use explicit raw escape hatch for unsupported effect command syntax.",
    example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector, effect: impl fmt::Display, duration_seconds: u32, amplifier: u8, hide_particles: bool)  {\n    let command = sand::command::effect_give_raw(selector, effect, duration_seconds, amplifier, hide_particles);\n}",
)]
pub fn effect_give_raw(
    selector: Selector,
    effect: impl fmt::Display,
    duration_seconds: u32,
    amplifier: u8,
    hide_particles: bool,
) -> String {
    let suffix = if hide_particles { " true" } else { "" };
    format!("effect give {selector} {effect} {duration_seconds} {amplifier}{suffix}")
}

#[cfg(test)]
mod tests {
    use sand_commands::Selector;
    use sand_components::{EffectId, StatusEffectId, Ticks};

    use super::*;

    #[test]
    fn effect_give_typed() {
        assert_eq!(
            effect_give(Selector::self_(), EffectId::Speed).to_string(),
            "effect give @s minecraft:speed"
        );
    }

    #[test]
    fn shared_registry_effect_id_uses_the_typed_command_path() {
        let id = StatusEffectId::minecraft("speed").unwrap();
        assert_eq!(
            effect_give(Selector::self_(), id).seconds(5).to_string(),
            "effect give @s minecraft:speed 5"
        );
        assert_eq!(
            effect_clear_effect(
                Selector::self_(),
                StatusEffectId::minecraft("speed").unwrap()
            ),
            "effect clear @s minecraft:speed"
        );
    }

    #[test]
    fn effect_give_duration_amplifier_hidden_particles() {
        assert_eq!(
            effect_give(Selector::self_(), EffectId::Speed)
                .duration(Ticks::seconds(10))
                .amplifier(1)
                .particles(false)
                .to_string(),
            "effect give @s minecraft:speed 10 1 true"
        );
    }

    #[test]
    fn non_aligned_ticks_do_not_truncate() {
        let error = effect_give(Selector::self_(), EffectId::Speed)
            .duration(Ticks::new(15))
            .try_build()
            .unwrap_err();
        assert_eq!(error.code, "SAND-EFFECT-DURATION");
    }

    #[test]
    fn effect_clear_all() {
        assert_eq!(effect_clear(Selector::self_()), "effect clear @s");
    }

    #[test]
    fn effect_clear_specific() {
        assert_eq!(
            effect_clear_effect(Selector::self_(), EffectId::Regeneration),
            "effect clear @s minecraft:regeneration"
        );
    }
}
