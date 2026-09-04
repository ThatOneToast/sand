//! Contract tests for the canonical `Target` API and its internal selector lowering.

use sand_commands::selector::{AnyTarget, Many, One, PlayersOnly};
use sand_commands::{
    GameMode, ObjectiveName, RenderCommand, ScoreRange, SortOrder, Target, Validate,
};

fn takes_single_entity(_: Target<AnyTarget, One>) {}
fn takes_many_entities(_: Target<AnyTarget, Many>) {}
fn takes_single_player(_: Target<PlayersOnly, One>) {}
fn takes_many_players(_: Target<PlayersOnly, Many>) {}

#[test]
fn constructors_encode_category_and_cardinality() {
    takes_single_entity(Target::self_());
    takes_many_entities(Target::entities());
    takes_single_player(Target::nearest_player());
    takes_single_player(Target::current_player());
    takes_single_player(Target::raw_single_player("@a[modded=true,limit=1]"));
    takes_many_players(Target::players());
}

#[test]
fn raw_single_player_preserves_both_type_assertions() {
    let target = Target::raw_single_player("@a[modded=true,limit=1]");
    assert_eq!(target.to_string(), "@a[modded=true,limit=1]");
    assert!(target.try_build().is_ok());
    takes_single_player(target);
}

#[test]
fn entity_filters_render_on_one_value() {
    let target = Target::entities()
        .tag("elite")
        .not_tag("friendly")
        .team("red")
        .not_team("blue")
        .name("Boss")
        .not_name("Decoy")
        .entity_type("minecraft:zombie")
        .not_type("minecraft:cow")
        .excluding_players()
        .distance_range(2.0, 16.0)
        .at_pos(0.0, 64.0, 0.0)
        .volume(8.0, 4.0, 8.0)
        .predicate("demo:is_hostile")
        .not_predicate("demo:is_friendly");

    assert_eq!(
        target.to_string(),
        "@e[tag=elite,tag=!friendly,team=red,team=!blue,name=Boss,name=!Decoy,type=minecraft:zombie,type=!minecraft:cow,type=!minecraft:player,distance=2..16,x=0,y=64,z=0,dx=8,dy=4,dz=8,predicate=demo:is_hostile,predicate=!demo:is_friendly]"
    );
}

#[test]
fn player_only_filters_stay_on_player_targets() {
    let target = Target::players()
        .gamemode(GameMode::Survival)
        .not_gamemode(GameMode::Spectator)
        .level_range(10.0, 30.0)
        .within_blocks(24.0)
        .tag("ready");

    assert_eq!(
        target.to_string(),
        "@a[gamemode=survival,gamemode=!spectator,level=10..30,distance=..24,tag=ready]"
    );
    takes_many_players(target);
}

#[test]
fn scalar_level_helpers_do_not_require_a_range_wrapper() {
    assert_eq!(
        Target::players().level_min(5.0).to_string(),
        "@a[level=5..]"
    );
    assert_eq!(
        Target::players().level_max(20.0).to_string(),
        "@a[level=..20]"
    );
}

#[test]
fn multiple_scores_do_not_require_a_selector_map_wrapper() {
    let target = Target::entities().scores([
        (ObjectiveName::new("threat"), ScoreRange::at_least(5)),
        (ObjectiveName::new("deaths"), ScoreRange::exact(0)),
    ]);

    assert_eq!(target.to_string(), "@e[scores={threat=5..,deaths=0}]");
}

#[test]
fn score_helper_is_fallible_at_the_immediate_boundary() {
    let target = Target::entities()
        .score(ObjectiveName::new("threat"), ScoreRange::between(1, 10))
        .unwrap();
    assert_eq!(target.to_string(), "@e[scores={threat=1..10}]");
}

#[test]
fn narrowing_changes_only_cardinality() {
    let entity = Target::entities().tag("elite").nearest();
    assert_eq!(entity.to_string(), "@e[tag=elite,sort=nearest,limit=1]");
    takes_single_entity(entity);

    let player = Target::players().tag("ready").limit(1).unwrap();
    assert_eq!(player.to_string(), "@a[tag=ready,limit=1]");
    takes_single_player(player);

    assert!(Target::entities().limit(2).is_err());
}

#[test]
fn raw_many_narrowing_is_rendered_instead_of_only_changing_the_type() {
    let limited = Target::raw_many("@e[modded=true]").limit(1).unwrap();
    assert_eq!(limited.to_string(), "@e[modded=true,limit=1]");
    assert_eq!(limited.try_build().unwrap(), "@e[modded=true,limit=1]");
    takes_single_entity(limited);

    let nearest = Target::raw_many("@e[modded=true]").nearest();
    assert_eq!(nearest.to_string(), "@e[modded=true,sort=nearest,limit=1]");
    takes_single_entity(nearest);

    assert_eq!(
        Target::raw_many("@e").limit(1).unwrap().to_string(),
        "@e[limit=1]"
    );
}

#[test]
fn sorting_stays_on_the_canonical_many_target_api() {
    assert_eq!(
        Target::entities().sort(SortOrder::Furthest).to_string(),
        "@e[sort=furthest]"
    );
    assert_eq!(
        Target::players()
            .sort(SortOrder::Random)
            .limit(1)
            .unwrap()
            .to_string(),
        "@a[sort=random,limit=1]"
    );
}

#[test]
fn named_target_filters_are_rendered_and_remain_single() {
    let player = Target::named_player("Steve")
        .tag("ready")
        .gamemode(GameMode::Survival);
    assert_eq!(
        player.to_string(),
        "@a[name=Steve,tag=ready,gamemode=survival,limit=1]"
    );
    assert_eq!(
        player.try_build().unwrap(),
        "@a[name=Steve,tag=ready,gamemode=survival,limit=1]"
    );
    takes_single_player(player);

    let entity = Target::named("Alex").tag("builder");
    assert_eq!(entity.to_string(), "@a[name=Alex,tag=builder,limit=1]");
    takes_single_entity(entity);

    assert_eq!(Target::named_player("Steve").to_string(), "Steve");
}

#[test]
fn explicit_raw_escape_hatches_are_named_raw() {
    let target = Target::players()
        .gamemode_raw("survival")
        .level_raw("1..5")
        .scores_raw("kills=1..")
        .nbt_raw("{Health:20.0f}")
        .predicate_raw("demo:ready");

    assert_eq!(
        target.to_string(),
        "@a[gamemode=survival,level=1..5,scores={kills=1..},nbt={Health:20.0f},predicate=demo:ready]"
    );
}

#[test]
fn invalid_filters_share_the_target_validation_boundary() {
    assert!(Target::entities().tag("has space").try_build().is_err());
    assert!(
        Target::entities()
            .distance_range(10.0, 1.0)
            .try_build()
            .is_err()
    );
    assert!(
        Target::players()
            .level_range(30.0, 1.0)
            .try_build()
            .is_err()
    );
    assert!(
        Target::players()
            .gamemode_raw("hardcore")
            .try_build()
            .is_err()
    );
    assert!(
        Target::entities()
            .predicate("NOT A LOCATION")
            .try_build()
            .is_err()
    );
}

#[test]
fn duplicate_score_objectives_are_rejected() {
    let target = Target::players().scores([
        (ObjectiveName::new("kills"), ScoreRange::exact(1)),
        (ObjectiveName::new("kills"), ScoreRange::exact(2)),
    ]);
    assert!(target.try_build().is_err());
}

#[test]
fn validation_is_deterministic() {
    let target = Target::entities()
        .scores([
            (ObjectiveName::new("z"), ScoreRange::at_least(1)),
            (ObjectiveName::new("a"), ScoreRange::at_most(2)),
        ])
        .within_blocks(20.0);
    let profile = sand_commands::CommandProfile::unprofiled();

    assert!(target.validate(&profile).is_ok());
    assert_eq!(target.to_string(), target.clone().to_string());
}
