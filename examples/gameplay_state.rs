//! Canonical scoped State and gameplay-data example.
//!
//! `#[derive(State)]` owns schema registration and lifecycle wiring. Authors
//! interact with generated bound fields; the scoreboard and NBT lowering stays
//! behind those handles.

use sand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, StateEnum)]
enum BossPhase {
    Idle,
    Fighting,
    Enraged,
    Defeated,
}

#[derive(State)]
#[state(namespace = "boss_phases", scope = player)]
struct Combat {
    #[state(default = 100, min = 0, max = 100)]
    health_percent: Score,
    #[state(default = "BossPhase::Idle")]
    phase: EntityEnum<BossPhase>,
    #[state(auto_tick)]
    special_attack: Cooldown,
}

#[function("boss_phases:phase/start_enrage")]
fn start_enrage() {
    let bar = BossbarId::parse("boss_phases:guardian").unwrap();
    Bossbar::add(
        bar.clone(),
        Text::new("Ancient Guardian").dark_red().bold(true),
    );
    Bossbar::set_max(bar.clone(), 100);
    Bossbar::set_value(bar.clone(), 50);
    Bossbar::set_color(bar.clone(), BossbarColor::Red);
    Bossbar::set_players(bar, Target::players());
    cmd::tellraw(
        Target::self_(),
        Text::new("The boss is enraged!").dark_red().bold(true),
    );
    cmd::effect_give(Target::self_(), EffectId::Strength)
        .seconds(10)
        .amplifier(1);
}

#[function("boss_phases:phase/tick")]
fn boss_tick() {
    let combat = Combat::on(EntityContext::<PlayerKind>::default());
    when(combat.health_percent.lte(50)).then_all([
        combat.phase.set(BossPhase::Enraged),
        cmd::function(start_enrage),
    ]);
    combat.special_attack.start(Ticks::seconds(5));
}

#[function("boss_phases:inventory/cache_selected")]
fn cache_selected_item() {
    ItemLocation::entity(Target::self_())
        .mainhand()
        .copy_to(&Nbt::storage(ResourceLocation::new("boss_phases", "cache").unwrap()).path("last_item"));
}
