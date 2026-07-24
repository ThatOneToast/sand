//! Unified gameplay-data and state-flow example.
//!
//! This is the source-level companion to
//! `sand-example/src/gameplay_state_example.rs`. It demonstrates the canonical
//! façade import and can be copied into a Sand project.

use sand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BossPhase {
    Idle = 0,
    Fighting = 1,
    Enraged = 2,
    Defeated = 3,
}

impl TypedGameState for BossPhase {
    fn to_score(self) -> i32 {
        self as i32
    }

    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Fighting),
            2 => Some(Self::Enraged),
            3 => Some(Self::Defeated),
            _ => None,
        }
    }
}

static HEALTH_PERCENT: ScoreField = ScoreField::new("boss_health_pct").default(100);
static PHASE: GameStateField<BossPhase> =
    GameStateField::with_default_score("boss_phase", BossPhase::Idle as i32);

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
    Bossbar::set_players(bar, Selector::all_players());
    Title::of(Selector::all_players())
        .title(Text::new("ENRAGED").dark_red().bold(true))
        .subtitle(Text::new("The guardian breaks its chains").gold())
        .times(10, 50, 20)
        .build();
    cmd::tellraw(
        Selector::self_(),
        Text::new("The boss is enraged!").dark_red().bold(true),
    );
    cmd::effect_give(Selector::self_(), EffectId::Strength)
        .seconds(10)
        .amplifier(1);
}

#[function("boss_phases:phase/stop_fighting")]
fn stop_fighting() {
    cmd::tellraw(Selector::self_(), Text::new("Fight ended.").gray());
}

#[function("boss_phases:phase/enraged_tick")]
fn enraged_tick() {
    Actionbar::show(Selector::self_(), Text::new("ENRAGED").dark_red());
    for command in ParticleBuilder::new(Particle::dust_hex(0xCC2200, 1.2))
        .try_circle(2.0, 1.0, 16)
        .unwrap()
    {
        command;
    }
    Sound::play("minecraft:entity.warden.heartbeat")
        .source(SoundSource::Hostile)
        .to(Selector::all_players())
        .volume(0.7)
        .pitch(0.8);
}

#[component(Load)]
fn boss_flow() {
    PlayerDataSchema::new("boss")
        .score_field(&HEALTH_PERCENT)
        .define_all();
    Nbt::storage("boss_phases:config")
        .path("max_level")
        .set(10);
    StateFlow::players(PHASE.value())
        .transition(BossPhase::Fighting, BossPhase::Defeated)
        .when(HEALTH_PERCENT.of("@s").lte(0))
        .priority(200)
        .done()
        .transition(BossPhase::Fighting, BossPhase::Enraged)
        .when(HEALTH_PERCENT.of("@s").lte(50))
        .priority(100)
        .done()
        .on_exit(BossPhase::Fighting, cmd::call(stop_fighting))
        .on_enter(BossPhase::Enraged, cmd::call(start_enrage))
        .on_tick(BossPhase::Enraged, cmd::call(enraged_tick))
        .register();
}

#[function("boss_phases:inventory/cache_selected")]
fn cache_selected_item() {
    ItemLocation::entity(Selector::self_())
        .mainhand()
        .copy_to(&Nbt::storage("boss_phases:cache").path("last_item"));
}
