//! Derived State and nested typed conditions.

use sand::prelude::*;

#[derive(State)]
#[state(namespace = "example", scope = player)]
struct AbilityState {
    #[state(default = 100, min = 0, max = 100)]
    mana: Score,
    #[state(default = false)]
    casting: Flag,
    #[state(auto_tick)]
    dash: Cooldown,
}

#[function]
pub fn try_dash() {
    let state = AbilityState::on(EntityContext::<PlayerKind>::default());
    TypedExecute::as_players_at_self()
        .when(all![
            state.mana.gte(25),
            state.casting.is_disabled(),
            any![
                state.dash.ready(),
                Condition::predicate(PredicateId::custom(
                    "example:dash_override".parse().unwrap(),
                )),
            ],
        ])
        .run(Actionbar::show(
            Target::self_(),
            Text::new("Dash ready").aqua(),
        ));
}
