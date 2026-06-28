//! Reusable typed VFX built from particle and sound steps.

use sand_core::mcfunction;
use sand_core::prelude::*;
use sand_macros::function;

fn level_up_vfx() -> Vfx {
    Vfx::new("level_up")
        .particle(
            VfxParticle::happy_villager()
                .count(20)
                .spread(0.6, 1.0, 0.6),
        )
        .sound(
            VfxSound::new("minecraft:entity.player.levelup")
                .source(SoundSource::Player)
                .volume(1.0)
                .pitch(1.2),
        )
}

#[function]
pub fn level_up() {
    let commands = level_up_vfx().play_at(Selector::self_());

    mcfunction! {
        for command in &commands {
            command;
        }
    }
}
