use sand_core::prelude::*;

fn main() {
    let profile = sand_core::cmd::CommandProfile::unprofiled();
    let selector = Selector::all_players().limit(1);
    let player = SinglePlayer::try_from(selector).expect("limit=1 is a single player");
    let entity: SingleEntity = player.into();

    let holder = ScoreHolder::from(entity.selector().clone());
    let objective = ObjectiveName::try_dynamic("mana").expect("valid objective");
    holder.validate(&profile).expect("valid holder");
    objective.validate(&profile).expect("valid objective");

    let operation = sand_core::cmd::scoreboard_players_operation(
        holder,
        objective.clone(),
        sand_core::cmd::ScoreOp::Set,
        ScoreHolder::fake("#default"),
        objective,
    );
    let rendered = Execute::new()
        .as_(entity.selector().clone())
        .at(Selector::self_())
        .try_run(&operation)
        .expect("typed execute chain is valid");
    assert_eq!(
        rendered,
        "execute as @a[limit=1] at @s run scoreboard players operation @a[limit=1] mana = #default mana"
    );

    let raw: RawCommand = sand_core::cmd::raw("modded command syntax");
    assert_eq!(raw.as_str(), "modded command syntax");
}
