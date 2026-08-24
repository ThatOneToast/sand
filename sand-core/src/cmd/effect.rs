use std::fmt;

use sand_commands::{Build, EffectCommand, EffectDuration, RenderCommand, Selector};
use sand_components::{EffectId, Ticks};

use super::Command;

#[doc = "**API Contract:** Run `sand api show sand::command::EffectGive` for the canonical contract."]
/// Builder for `effect give`.
#[derive(Debug, Clone)]
pub struct EffectGive {
    command: EffectCommand,
}

impl EffectGive {
    /// Creates a typed effect give command builder from the supplied command inputs.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::new` for the canonical contract."]
    pub fn new(selector: Selector, effect: impl Into<EffectId>) -> Self {
        Self {
            command: EffectCommand::give(selector, effect.into().to_string()),
        }
    }

    /// Set command duration using ticks. Minecraft command syntax stores this as seconds.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::duration` for the canonical contract."]
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.command = self.command.duration(EffectDuration::ticks(duration.get()));
        self
    }

    /// Set command duration in seconds.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::seconds` for the canonical contract."]
    pub fn seconds(mut self, seconds: u32) -> Self {
        self.command = self.command.duration(EffectDuration::seconds(seconds));
        self
    }

    /// Persist until explicitly cleared (Minecraft 1.19.4+).
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::infinite` for the canonical contract."]
    pub fn infinite(mut self) -> Self {
        self.command = self.command.duration(EffectDuration::Infinite);
        self
    }

    /// Sets the zero-based amplifier on this typed effect command.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::amplifier` for the canonical contract."]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.command = self.command.amplifier(amplifier);
        self
    }

    /// Control visible particles. `false` serializes to Minecraft's `hideParticles=true`.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::particles` for the canonical contract."]
    pub fn particles(mut self, show_particles: bool) -> Self {
        self.command = self.command.particles(show_particles);
        self
    }

    /// Renders the configured effect give as validated Minecraft command text.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::build` for the canonical contract."]
    pub fn build(&self) -> String {
        self.command.build()
    }

    /// Validate exact whole-second semantics before rendering.
    #[doc = "**API Contract:** Run `sand api show sand::command::EffectGive::try_build` for the canonical contract."]
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
#[doc = "**API Contract:** Run `sand api show sand::command::effect_give` for the canonical contract."]
pub fn effect_give(selector: Selector, effect: impl Into<EffectId>) -> EffectGive {
    EffectGive::new(selector, effect)
}

/// `effect clear <selector>` — clear all status effects.
#[doc = "**API Contract:** Run `sand api show sand::command::effect_clear` for the canonical contract."]
pub fn effect_clear(selector: Selector) -> String {
    format!("effect clear {selector}")
}

/// `effect clear <selector> <effect>` — clear one typed status effect.
#[doc = "**API Contract:** Run `sand api show sand::command::effect_clear_effect` for the canonical contract."]
pub fn effect_clear_effect(selector: Selector, effect: impl Into<EffectId>) -> String {
    format!("effect clear {selector} {}", effect.into())
}

/// Explicit raw escape hatch for unsupported effect command syntax.
#[doc = "**API Contract:** Run `sand api show sand::command::effect_give_raw` for the canonical contract."]
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
