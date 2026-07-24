//! Unified gameplay-data and state-flow example.
//!
//! This module connects schema fields, live inventory locations, typed NBT
//! snapshots, guarded enum-state transitions, enter/exit hooks, and while-in
//! tick hooks through one exported datapack.

use sand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossPhase {
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
static HAS_WAND: FlagField = FlagField::new("boss_has_wand").default(false);
static CAST: CooldownField = CooldownField::new("boss_cast", Ticks::seconds(3));
static PHASE: GameStateField<BossPhase> =
    GameStateField::with_default_score("boss_phase", BossPhase::Idle as i32);

struct PlayerModel;
static PLAYER: PlayerModel = PlayerModel;

impl PlayerModel {
    fn schema(&self) -> PlayerDataSchema {
        PlayerDataSchema::new("boss")
            .score_field(&HEALTH_PERCENT)
            .flag_field(&HAS_WAND)
            .cooldown_field(&CAST)
            .game_state(&PHASE)
    }

    fn health_percent(&self) -> &'static ScoreField {
        &HEALTH_PERCENT
    }

    fn phase(&self) -> &'static GameStateField<BossPhase> {
        &PHASE
    }
}

#[function("hello_world:start_enrage")]
pub fn start_enrage() {
    cmd::tellraw(
        Selector::self_(),
        Text::new("The boss is enraged!").dark_red().bold(true),
    );
}

#[function("hello_world:stop_fighting")]
pub fn stop_fighting() {
    cmd::tellraw(
        Selector::self_(),
        Text::new("The fighting phase ended.").gray(),
    );
}

#[function("hello_world:enraged_tick")]
pub fn enraged_tick() {
    Actionbar::show(Selector::self_(), Text::new("[Enraged] berserk").dark_red());
}

#[component(Load)]
pub fn boss_load_and_flow() {
    // The flow owns the `boss_phase` objective lifecycle, so only the other
    // schema fields are defined here.
    PlayerDataSchema::new("boss")
        .score_field(&HEALTH_PERCENT)
        .flag_field(&HAS_WAND)
        .cooldown_field(&CAST)
        .define_all();
    PLAYER
        .schema()
        .init_player("@s")
        .into_iter()
        .map(|command| TypedExecute::as_players().run(command))
        .collect::<Vec<_>>();
    Nbt::storage("boss_phases:config").path("max_level").set(10);
    StateFlow::players(PHASE.value())
        .named("boss")
        .transition(BossPhase::Fighting, BossPhase::Defeated)
        .when(PLAYER.health_percent().of("@s").lte(0))
        .priority(200)
        .done()
        .transition(BossPhase::Fighting, BossPhase::Enraged)
        .when(PLAYER.health_percent().of("@s").lte(50))
        .priority(100)
        .done()
        .on_exit(BossPhase::Fighting, cmd::call(stop_fighting))
        .on_enter(BossPhase::Enraged, cmd::call(start_enrage))
        .on_tick(BossPhase::Enraged, cmd::call(enraged_tick))
        .register();
}

/// Snapshot the selected stack into global command storage. The item location
/// is live; storage contains a copy made when this function runs.
#[function("hello_world:cache_selected_item")]
pub fn cache_selected_item() {
    let selected = ItemLocation::entity(Selector::self_()).mainhand();
    let cache = Nbt::storage("boss_phases:cache").path("last_item");
    selected.copy_to(&cache);
}

/// A typed low-level escape hatch for starting the fight manually. Registered
/// flow hooks only observe writes performed by the flow itself.
#[function("hello_world:start_fight")]
pub fn start_fight() {
    PLAYER.phase().of("@s").set(BossPhase::Fighting);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_only_data_and_schema_access_are_typed() {
        assert_eq!(
            cache_selected_item(),
            vec!["data modify storage boss_phases:cache last_item set from entity @s SelectedItem"]
        );
        assert!(matches!(
            PLAYER.health_percent().of("@s").lte(50),
            Condition::Score { .. }
        ));
        assert_eq!(
            start_fight(),
            vec!["scoreboard players set @s boss_phase 1"]
        );
    }

    #[test]
    fn hook_functions_are_normal_typed_function_targets() {
        assert_eq!(cmd::call(start_enrage), "function hello_world:start_enrage");
        assert_eq!(
            cmd::call(stop_fighting),
            "function hello_world:stop_fighting"
        );
    }
}
