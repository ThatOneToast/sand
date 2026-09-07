//! A compact gameplay example using derived State and typed function handles.

use sand::prelude::*;

#[derive(State)]
#[state(namespace = "spells", scope = player)]
struct Spells {
    #[state(default = 100, min = 0, max = 100)]
    mana: Score,
    #[state(auto_tick)]
    fireball: Cooldown,
    #[state(default_snbt = "{}")]
    settings: Data<serde_json::Value>,
}

#[function]
pub fn cast_fireball() {
    let spells = Spells::on(EntityContext::<PlayerKind>::default());
    when(all![spells.mana.gte(20), spells.fireball.ready()]).then_all([
        spells.mana.remove(20),
        spells.fireball.start(Ticks::seconds(5)),
        cmd::function(do_cast),
    ]);
}

#[function("spells:fireball/do_cast")]
pub fn do_cast() {
    cmd::tellraw(Target::self_(), Text::new("Fireball!").gold());
}
