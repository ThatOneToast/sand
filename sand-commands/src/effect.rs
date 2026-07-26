//! Typed `effect give` command IR and whole-second duration semantics.

use std::collections::BTreeMap;
use std::fmt;

use crate::Build;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;

/// Minecraft's effect duration representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectDuration {
    /// Persist until explicitly cleared. Supported by Java 1.19.4+.
    Infinite,
    /// An explicit whole-second duration.
    Seconds(u32),
    /// Compatibility input expressed in ticks; validation requires exact
    /// divisibility by 20.
    Ticks(u32),
}

impl EffectDuration {
    pub const fn seconds(seconds: u32) -> Self {
        Self::Seconds(seconds)
    }

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

    pub fn give_raw(target: Selector, effect: impl Into<String>) -> Self {
        Self {
            raw_effect: true,
            ..Self::give(target, effect)
        }
    }

    pub fn duration(mut self, duration: EffectDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = Some(amplifier);
        self
    }

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
