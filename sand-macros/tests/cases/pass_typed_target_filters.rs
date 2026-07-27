//! #200: typed selector filters forwarded onto `EntityTarget<A>` /
//! `PlayerTarget<A>` preserve the cardinality marker `A` by construction, and
//! player-only filters are reachable from player targets of both arities.

use sand::prelude::*;

fn needs_one_entity(_target: SingleEntity) {}
fn needs_many_entities(_target: EntityTargets) {}
fn needs_one_player(_target: SinglePlayer) {}
fn needs_many_players(_target: PlayerTargets) {}

fn main() {
    needs_many_entities(
        EntityTargets::all()
            .entity_type("minecraft:zombie")
            .not_type("minecraft:cow")
            .tag("elite")
            .not_tag("done")
            .team("red")
            .not_team("blue")
            .name("Boss")
            .not_name("Mini")
            .within_blocks(16.0)
            .distance_min(1.0)
            .excluding_self()
            .volume(3.0, 1.0, 3.0)
            .at_pos(0.0, 64.0, 0.0)
            .scores_raw("threat=5..")
            .nbt_raw("{Silent:1b}")
            .predicate_raw("pack:is_burning"),
    );

    needs_one_entity(
        SingleEntity::self_()
            .tag("elite")
            .team("red")
            .within_blocks(16.0),
    );

    // Narrowing stays explicit and cardinality-changing.
    needs_one_entity(EntityTargets::all().tag("elite").nearest());

    needs_many_players(
        PlayerTargets::all()
            .tag("ready")
            .team("red")
            .name("Steve")
            .level("10..30")
            .gamemode("survival")
            .gamemode_typed(GameMode::Adventure)
            .not_gamemode_typed(GameMode::Spectator)
            .within_blocks(16.0)
            .excluding_self()
            .scores_raw("kills=1..")
            .nbt_raw("{Health:20.0f}")
            .predicate_raw("pack:is_sneaking"),
    );

    needs_one_player(
        SinglePlayer::nearest()
            .tag("ready")
            .gamemode_typed(GameMode::Survival)
            .level("30.."),
    );

    needs_one_player(PlayerTargets::all().tag("ready").nearest());
}
