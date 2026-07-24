use sand_core::prelude::*;
use sand_macros::function;

#[function]
fn boss_warning() {
    let id = BossbarId::parse("example:boss").unwrap();
    Bossbar::add(id.clone(), Text::new("Boss").red());
    Bossbar::set_players(id, Selector::all_players());
    Actionbar::show(Selector::all_players(), Text::new("Incoming").gold());
    Sound::play("example:boss.roar")
        .source(SoundSource::Hostile)
        .to(Selector::all_players())
        .build();
    cmd::effect_give(Selector::all_players(), EffectId::Strength).seconds(10);
}

fn main() {
    assert!(boss_warning().iter().any(|line| line.starts_with("bossbar")));
}
