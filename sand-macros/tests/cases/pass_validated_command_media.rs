use sand_core::prelude::*;
use sand_macros::function;

#[function]
fn boss_warning() {
    let id = BossbarId::parse("example:boss").unwrap();
    Bossbar::add(id.clone(), Text::new("Boss").red());
    Bossbar::set_players(id, Target::all_players());
    Actionbar::show(Target::all_players(), Text::new("Incoming").gold());
    Sound::play("example:boss.roar")
        .source(SoundSource::Hostile)
        .to(Target::all_players())
        .build();
    cmd::effect_give(Target::all_players(), EffectId::Strength).seconds(10);
}

fn main() {
    assert!(boss_warning().iter().any(|line| line.starts_with("bossbar")));
}
