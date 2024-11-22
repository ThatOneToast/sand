use crate::{selector::TargetSelector, status_effects::StatusEffects};



#[derive(Debug, Clone)]
pub enum EffectCondition {
    Clear,
    Give
}

// /effect give TheOneTrueToast minecraft:absorption 30 0

pub type Duration = i32;
pub type Amplifier = i32;

#[derive(Debug, Clone)]
pub enum Effect {
    Give(TargetSelector, StatusEffects, Duration, Amplifier),
    Clear(TargetSelector)
}

impl ToString for Effect {
    fn to_string(&self) -> String {
        match self {
            Effect::Give(target, effect, duration, amplifier) => {
                let target = target.to_string();
                let effect = effect.to_string();
                let duration = duration.to_string();
                let amplifier = amplifier.to_string();
                format!("/effect give {target} {effect} {duration} {amplifier}")
            }
            Effect::Clear(target) => {
                let target = target.to_string();
                format!("/effect clear {target}")
            }
        }
    }
}