use std::fmt;

use sand_commands::Selector;
use sand_components::{EffectId, Ticks};

use super::Command;

/// Builder for `effect give`.
#[derive(Debug, Clone)]
pub struct EffectGive {
    selector: Selector,
    effect: EffectId,
    duration: Option<Ticks>,
    amplifier: Option<u8>,
    show_particles: bool,
}

impl EffectGive {
    pub fn new(selector: Selector, effect: EffectId) -> Self {
        Self {
            selector,
            effect,
            duration: None,
            amplifier: None,
            show_particles: true,
        }
    }

    /// Set command duration using ticks. Minecraft command syntax stores this as seconds.
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set command duration in seconds.
    pub fn seconds(self, seconds: u32) -> Self {
        self.duration(Ticks::seconds(seconds))
    }

    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = Some(amplifier);
        self
    }

    /// Control visible particles. `false` serializes to Minecraft's `hideParticles=true`.
    pub fn particles(mut self, show_particles: bool) -> Self {
        self.show_particles = show_particles;
        self
    }

    pub fn build(&self) -> String {
        let mut out = format!("effect give {} {}", self.selector, self.effect);
        if let Some(duration) = self.duration {
            out.push_str(&format!(" {}", duration.as_seconds()));
        }
        if let Some(amplifier) = self.amplifier {
            if self.duration.is_none() {
                out.push_str(" 30");
            }
            out.push_str(&format!(" {amplifier}"));
        }
        if !self.show_particles {
            if self.duration.is_none() {
                out.push_str(" 30");
            }
            if self.amplifier.is_none() {
                out.push_str(" 0");
            }
            out.push_str(" true");
        }
        out
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
pub fn effect_give(selector: Selector, effect: EffectId) -> EffectGive {
    EffectGive::new(selector, effect)
}

/// `effect clear <selector>` — clear all status effects.
pub fn effect_clear(selector: Selector) -> String {
    format!("effect clear {selector}")
}

/// `effect clear <selector> <effect>` — clear one typed status effect.
pub fn effect_clear_effect(selector: Selector, effect: EffectId) -> String {
    format!("effect clear {selector} {effect}")
}

/// Explicit raw escape hatch for unsupported effect command syntax.
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
    use sand_components::{EffectId, Ticks};

    use super::*;

    #[test]
    fn effect_give_typed() {
        assert_eq!(
            effect_give(Selector::self_(), EffectId::Speed).to_string(),
            "effect give @s minecraft:speed"
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
