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
    cmd::tellraw(
        Selector::self_(),
        Text::new("The boss is enraged!").dark_red().bold(true),
    );
}

#[function("boss_phases:phase/stop_fighting")]
fn stop_fighting() {
    cmd::tellraw(Selector::self_(), Text::new("Fight ended.").gray());
}

#[function("boss_phases:phase/enraged_tick")]
fn enraged_tick() {
    Actionbar::show(Selector::self_(), Text::new("ENRAGED").dark_red());
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
