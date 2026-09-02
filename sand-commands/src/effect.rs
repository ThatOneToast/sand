//! Typed `effect give` command IR and whole-second duration semantics.

use std::collections::BTreeMap;
use std::fmt;

use crate::Build;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::EffectDuration",
    aliases = ["sand::cmd::EffectDuration", "sand::prelude::EffectDuration", "sand::prelude::cmd::EffectDuration"],
    module = "sand::command",
    summary = "Minecraft's effect duration representation.",
    context = "Minecraft's effect duration representation. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::EffectDuration;",
    variants(Infinite = "Persist until explicitly cleared. Supported by Java 1.19.4+.", Seconds = "An explicit whole-second duration.", Ticks = "Compatibility input expressed in ticks; validation requires exact divisibility by 20."),
    variant_fields(Seconds = ["An explicit whole-second duration."], Ticks = ["Compatibility input expressed in ticks; validation requires exact divisibility by 20."]),
)]
/// Minecraft's effect duration representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectDuration {
    /// Persist until explicitly cleared. Supported by Java 1.19.4+.
    Infinite,
    /// An explicit whole-second duration.
    Seconds(#[doc = "An explicit whole-second duration."] u32),
    /// Compatibility input expressed in ticks; validation requires exact
    /// divisibility by 20.
    Ticks(
        #[doc = "Compatibility input expressed in ticks; validation requires exact divisibility by 20."]
         u32,
    ),
}

impl EffectDuration {
    /// Creates an effect duration measured in whole seconds.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectDuration::seconds",
        aliases = ["sand::cmd::EffectDuration::seconds", "sand::prelude::EffectDuration::seconds", "sand::prelude::cmd::EffectDuration::seconds"],
        module = "sand::command",
        kind = "method",
        summary = "Creates an effect duration measured in whole seconds.",
        context = "Creates an effect duration measured in whole seconds. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(seconds = "`seconds` supplies the seconds value used to create an effect duration measured in whole seconds."),
        returns = "A newly constructed `EffectDuration` configured to create an effect duration measured in whole seconds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(seconds: u32)  {\n    let effect_duration = sand::command::EffectDuration::seconds(seconds);\n}",
    )]
    pub const fn seconds(seconds: u32) -> Self {
        Self::Seconds(seconds)
    }

    /// Creates an effect duration measured in game ticks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectDuration::ticks",
        aliases = ["sand::cmd::EffectDuration::ticks", "sand::prelude::EffectDuration::ticks", "sand::prelude::cmd::EffectDuration::ticks"],
        module = "sand::command",
        kind = "method",
        summary = "Creates an effect duration measured in game ticks.",
        context = "Creates an effect duration measured in game ticks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(ticks = "`ticks` provides the Minecraft tick duration used to create an effect duration measured in game ticks."),
        returns = "A newly constructed `EffectDuration` configured to create an effect duration measured in game ticks.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ticks: u32)  {\n    let effect_duration = sand::command::EffectDuration::ticks(ticks);\n}",
    )]
    pub const fn ticks(ticks: u32) -> Self {
        Self::Ticks(ticks)
    }

    fn render(self) -> String {
        match self {
            Self::Infinite => "infinite".to_string(),
            Self::Seconds(seconds) => seconds.to_string(),
            Self::Ticks(ticks) => (ticks / 20).to_string(),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::EffectCommand",
    aliases = ["sand::cmd::EffectCommand", "sand::prelude::cmd::EffectCommand"],
    module = "sand::command",
    summary = "Structured `effect give` terminal command.",
    context = "Structured `effect give` terminal command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::EffectCommand;",
)]
/// Structured `effect give` terminal command.
#[derive(Debug, Clone)]
pub struct EffectCommand {
    target: Selector,
    effect: String,
    raw_effect: bool,
    duration: Option<EffectDuration>,
    amplifier: Option<u8>,
    show_particles: bool,
}

impl EffectCommand {
    /// Creates a typed effect command builder from the supplied command inputs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectCommand::give",
        aliases = ["sand::cmd::EffectCommand::give", "sand::prelude::cmd::EffectCommand::give"],
        module = "sand::command",
        kind = "method",
        summary = "Creates a typed effect command builder from the supplied command inputs.",
        context = "Creates a typed effect command builder from the supplied command inputs. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to create a typed effect command builder from the supplied command inputs.", effect = "`effect` supplies the effect value used to create a typed effect command builder from the supplied command inputs."),
        returns = "A newly constructed `EffectCommand` configured to create a typed effect command builder from the supplied command inputs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, effect: impl Into < String >)  {\n    let effect_command = sand::command::EffectCommand::give(target, effect);\n}",
    )]
    pub fn give(target: Selector, effect: impl Into<String>) -> Self {
        Self {
            target,
            effect: effect.into(),
            raw_effect: false,
            duration: None,
            amplifier: None,
            show_particles: true,
        }
    }

    /// Uses the explicit raw give escape hatch on the effect command builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectCommand::give_raw",
        aliases = ["sand::cmd::EffectCommand::give_raw", "sand::prelude::cmd::EffectCommand::give_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Uses the explicit raw give escape hatch on the effect command builder.",
        context = "Uses the explicit raw give escape hatch on the effect command builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to use the explicit raw give escape hatch on the effect command builder.", effect = "`effect` supplies the effect value used to use the explicit raw give escape hatch on the effect command builder."),
        returns = "A newly constructed `EffectCommand` configured to use the explicit raw give escape hatch on the effect command builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, effect: impl Into < String >)  {\n    let effect_command = sand::command::EffectCommand::give_raw(target, effect);\n}",
    )]
    pub fn give_raw(target: Selector, effect: impl Into<String>) -> Self {
        Self {
            raw_effect: true,
            ..Self::give(target, effect)
        }
    }

    /// Sets the effect duration on this command builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectCommand::duration",
        aliases = ["sand::cmd::EffectCommand::duration", "sand::prelude::cmd::EffectCommand::duration"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the effect duration on this command builder.",
        context = "Sets the effect duration on this command builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(duration = "`duration` provides the Minecraft tick duration used to set the effect duration on this command builder."),
        returns = "The `EffectCommand` value with the documented change applied to set the effect duration on this command builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_command_value: sand::command::EffectCommand, duration: sand::command::EffectDuration)  {\n    let updated_effect_command = effect_command_value.duration(duration);\n}",
    )]
    pub fn duration(mut self, duration: EffectDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Sets the zero-based effect amplifier on this command builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectCommand::amplifier",
        aliases = ["sand::cmd::EffectCommand::amplifier", "sand::prelude::cmd::EffectCommand::amplifier"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the zero-based effect amplifier on this command builder.",
        context = "Sets the zero-based effect amplifier on this command builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(amplifier = "`amplifier` supplies the amplifier value used to set the zero-based effect amplifier on this command builder."),
        returns = "The `EffectCommand` value with the documented change applied to set the zero-based effect amplifier on this command builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_command_value: sand::command::EffectCommand, amplifier: u8)  {\n    let updated_effect_command = effect_command_value.amplifier(amplifier);\n}",
    )]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = Some(amplifier);
        self
    }

    /// Controls whether applying the effect displays particles.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EffectCommand::particles",
        aliases = ["sand::cmd::EffectCommand::particles", "sand::prelude::cmd::EffectCommand::particles"],
        module = "sand::command",
        kind = "method",
        summary = "Controls whether applying the effect displays particles.",
        context = "Controls whether applying the effect displays particles. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(show_particles = "`show_particles` provides the switch that enables or disables the behavior used to control whether applying the effect displays particles."),
        returns = "The `EffectCommand` value with the documented change applied to control whether applying the effect displays particles.",
        example = "use sand::prelude::*;\n\nfn demonstrate(effect_command_value: sand::command::EffectCommand, show_particles: bool)  {\n    let updated_effect_command = effect_command_value.particles(show_particles);\n}",
    )]
    pub fn particles(mut self, show_particles: bool) -> Self {
        self.show_particles = show_particles;
        self
    }
}

impl Validate for EffectCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.target
            .validate(profile)
            .map_err(|error| effect_error("SAND-EFFECT-TARGET", "target", error.to_string()))?;
        if !self.raw_effect {
            crate::validate::resource_location_shape(&self.effect, "EffectCommand", "effect")
                .map_err(|error| effect_error("SAND-EFFECT-ID", "effect", error.message))?;
        }
        if let Some(duration) = self.duration {
            match duration {
                EffectDuration::Infinite if !profile.is_at_least(1, 19, 4) => {
                    return Err(effect_error(
                        "SAND-EFFECT-VERSION",
                        "duration",
                        format!(
                            "`infinite` effect duration requires Minecraft 1.19.4+; selected {}",
                            profile.requested_version()
                        ),
                    ));
                }
                EffectDuration::Seconds(seconds) => validate_seconds(seconds)?,
                EffectDuration::Ticks(ticks) => {
                    if ticks == 0 || ticks % 20 != 0 {
                        return Err(effect_error(
                            "SAND-EFFECT-DURATION",
                            "duration",
                            format!(
                                "{ticks} ticks cannot be represented exactly by the whole-second effect command; use a positive multiple of 20 or `EffectDuration::seconds(...)`"
                            ),
                        ));
                    }
                    validate_seconds(ticks / 20)?;
                }
                EffectDuration::Infinite => {}
            }
        }
        Ok(())
    }
}

impl RenderCommand for EffectCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        let mut output = format!("effect give {} {}", self.target, self.effect);
        if let Some(duration) = self.duration {
            output.push(' ');
            output.push_str(&duration.render());
        }
        if let Some(amplifier) = self.amplifier {
            if self.duration.is_none() {
                output.push_str(" 30");
            }
            output.push_str(&format!(" {amplifier}"));
        }
        if !self.show_particles {
            if self.duration.is_none() {
                output.push_str(" 30");
            }
            if self.amplifier.is_none() {
                output.push_str(" 0");
            }
            output.push_str(" true");
        }
        output
    }
}

impl Build for EffectCommand {
    fn build(&self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, self.clone());
        line
    }
}

impl fmt::Display for EffectCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<EffectCommand> for String {
    fn from(command: EffectCommand) -> Self {
        command.build()
    }
}

fn validate_seconds(seconds: u32) -> CommandResult<()> {
    if !(1..=1_000_000).contains(&seconds) {
        Err(effect_error(
            "SAND-EFFECT-DURATION",
            "duration",
            format!("effect duration must be within 1..=1,000,000 seconds, got `{seconds}`"),
        ))
    } else {
        Ok(())
    }
}

fn effect_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("EffectCommand", field, message).with_code(code)
}

/// Export-scoped registry family holding this module's rendered
/// `effect` command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct EffectLines;

impl crate::export_registry::RegistryFamily for EffectLines {
    type State = BTreeMap<String, EffectCommand>;
}

fn register_line(line: &str, command: EffectCommand) {
    crate::export_registry::register_line::<EffectLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<EffectLines, _>(
        line,
        profile,
        |command, profile| command.validate(profile),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_duration_rendering_and_defaults() {
        assert_eq!(
            EffectCommand::give(Selector::self_(), "minecraft:speed").build(),
            "effect give @s minecraft:speed"
        );
        assert_eq!(
            EffectCommand::give(Selector::self_(), "minecraft:speed")
                .duration(EffectDuration::seconds(10))
                .amplifier(1)
                .particles(false)
                .try_build()
                .unwrap(),
            "effect give @s minecraft:speed 10 1 true"
        );
    }

    #[test]
    fn ticks_must_align_and_seconds_are_bounded() {
        let bad = EffectCommand::give(Selector::self_(), "minecraft:speed")
            .duration(EffectDuration::ticks(15));
        assert_eq!(bad.try_build().unwrap_err().code, "SAND-EFFECT-DURATION");
        assert!(
            EffectCommand::give(Selector::self_(), "minecraft:speed")
                .duration(EffectDuration::seconds(0))
                .try_build()
                .is_err()
        );
        assert!(
            EffectCommand::give(Selector::self_(), "minecraft:speed")
                .duration(EffectDuration::seconds(1_000_001))
                .try_build()
                .is_err()
        );
    }
}
