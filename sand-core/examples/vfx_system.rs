//! Reusable typed VFX command generation.
//!
//! Run with:
//! ```sh
//! cargo run --example vfx_system -p sand-core
//! ```

use sand_core::prelude::*;

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

fn main() {
    for command in level_up_vfx().play_at(Selector::self_()) {
        println!("{command}");
    }
}
