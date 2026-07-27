//! #200: typed selector filter parity between `Selector` and the typed target
//! wrappers `EntityTarget<A>` / `PlayerTarget<A>`.
//!
//! Every wrapper helper must (a) render byte-identically to the equivalent
//! direct [`Selector`] call, (b) preserve the cardinality marker `A`, and
//! (c) route through `Selector`'s validation rather than reimplementing it.

use sand_commands::RenderCommand;
use sand_commands::selector::{
    EntityTag, EntityTargets, GameMode, PlayerTargets, PredicateId, ScoreRange, Selector,
    SelectorRange, SelectorScores, SingleEntity, SinglePlayer, TeamName,
};
use sand_commands::{ObjectiveName, Validate};

// ── Cardinality witnesses ────────────────────────────────────────────────────
//
// These take the *concrete* aliases, so passing a filtered value proves the
// PhantomData arity marker survived the filter call. A wrapper that returned
// `EntityTarget<Many>` from a `SingleEntity` filter would not compile here.

fn takes_single_entity(_: SingleEntity) {}
fn takes_entity_targets(_: EntityTargets) {}
fn takes_single_player(_: SinglePlayer) {}
fn takes_player_targets(_: PlayerTargets) {}

// ── Entity target rendering parity ───────────────────────────────────────────

#[test]
fn entity_target_filters_render_identically_to_selector() {
    let cases: Vec<(String, String)> = vec![
        (
            EntityTargets::all().tag("elite").to_string(),
            Selector::all_entities().tag("elite").to_string(),
        ),
        (
            EntityTargets::all().not_tag("done").to_string(),
            Selector::all_entities().not_tag("done").to_string(),
        ),
        (
            EntityTargets::all()
                .tag_typed(EntityTag::new("elite"))
                .to_string(),
            Selector::all_entities()
                .tag_typed(EntityTag::new("elite"))
                .to_string(),
        ),
        (
            EntityTargets::all().team("red").to_string(),
            Selector::all_entities().team("red").to_string(),
        ),
        (
            EntityTargets::all().not_team("blue").to_string(),
            Selector::all_entities().not_team("blue").to_string(),
        ),
        (
            EntityTargets::all()
                .team_typed(TeamName::new("red"))
                .to_string(),
            Selector::all_entities()
                .team_typed(TeamName::new("red"))
                .to_string(),
        ),
        (
            EntityTargets::all().name("Boss").to_string(),
            Selector::all_entities().name("Boss").to_string(),
        ),
        (
            EntityTargets::all().not_name("Boss").to_string(),
            Selector::all_entities().not_name("Boss").to_string(),
        ),
        (
            EntityTargets::all()
                .entity_type("minecraft:zombie")
                .to_string(),
            Selector::all_entities()
                .entity_type("minecraft:zombie")
                .to_string(),
        ),
        (
            EntityTargets::all().not_type("minecraft:cow").to_string(),
            Selector::all_entities()
                .not_type("minecraft:cow")
                .to_string(),
        ),
        (
            EntityTargets::all().excluding_players().to_string(),
            Selector::all_entities().not_player().to_string(),
        ),
        (
            EntityTargets::all().within_blocks(16.0).to_string(),
            Selector::all_entities().distance_max(16.0).to_string(),
        ),
        (
            EntityTargets::all().distance_min(2.0).to_string(),
            Selector::all_entities().distance_min(2.0).to_string(),
        ),
        (
            EntityTargets::all().distance_range(0.5, 10.0).to_string(),
            Selector::all_entities()
                .distance_range(0.5, 10.0)
                .to_string(),
        ),
        (
            EntityTargets::all().distance("1..4").to_string(),
            Selector::all_entities().distance("1..4").to_string(),
        ),
        (
            EntityTargets::all()
                .distance_typed(SelectorRange::at_most(16.0))
                .to_string(),
            Selector::all_entities()
                .distance_typed(SelectorRange::at_most(16.0))
                .to_string(),
        ),
        (
            EntityTargets::all().excluding_self().to_string(),
            Selector::all_entities().distance("0.1..").to_string(),
        ),
        (
            EntityTargets::all()
                .scores_typed(
                    SelectorScores::new()
                        .with("threat", ScoreRange::at_least(5))
                        .with("kills", ScoreRange::exact(0)),
                )
                .to_string(),
            Selector::all_entities()
                .scores_typed(
                    SelectorScores::new()
                        .with("threat", ScoreRange::at_least(5))
                        .with("kills", ScoreRange::exact(0)),
                )
                .to_string(),
        ),
        (
            EntityTargets::all()
                .score(ObjectiveName::new("threat"), ScoreRange::at_least(5))
                .unwrap()
                .to_string(),
            Selector::all_entities()
                .score_typed(ObjectiveName::new("threat"), ScoreRange::at_least(5))
                .unwrap()
                .to_string(),
        ),
        (
            EntityTargets::all()
                .predicate_id(PredicateId::new("pack:is_burning"))
                .to_string(),
            Selector::all_entities()
                .predicate_id(PredicateId::new("pack:is_burning"))
                .to_string(),
        ),
        (
            EntityTargets::all()
                .predicate_id(PredicateId::new("pack:is_burning").negated())
                .to_string(),
            Selector::all_entities()
                .predicate_id(PredicateId::new("pack:is_burning").negated())
                .to_string(),
        ),
        (
            EntityTargets::all().volume(3.0, 1.0, 3.0).to_string(),
            Selector::all_entities().volume(3.0, 1.0, 3.0).to_string(),
        ),
        (
            EntityTargets::all().at_pos(10.0, 64.0, -20.0).to_string(),
            Selector::all_entities()
                .at_pos(10.0, 64.0, -20.0)
                .to_string(),
        ),
        (
            EntityTargets::all().scores_raw("kills=1..10").to_string(),
            Selector::all_entities()
                .scores_raw("kills=1..10")
                .to_string(),
        ),
        (
            EntityTargets::all()
                .nbt_raw("{CustomName:\"Boss\"}")
                .to_string(),
            Selector::all_entities()
                .nbt_raw("{CustomName:\"Boss\"}")
                .to_string(),
        ),
        (
            EntityTargets::all().predicate_raw("pack:ready").to_string(),
            Selector::all_entities()
                .predicate_raw("pack:ready")
                .to_string(),
        ),
    ];
    for (index, (wrapper, direct)) in cases.iter().enumerate() {
        assert_eq!(wrapper, direct, "entity parity case {index}");
    }
}

#[test]
fn player_target_filters_render_identically_to_selector() {
    let cases: Vec<(String, String)> = vec![
        (
            PlayerTargets::all().tag("ready").to_string(),
            Selector::all_players().tag("ready").to_string(),
        ),
        (
            PlayerTargets::all().not_tag("afk").to_string(),
            Selector::all_players().not_tag("afk").to_string(),
        ),
        (
            PlayerTargets::all()
                .tag_typed(EntityTag::new("ready"))
                .to_string(),
            Selector::all_players()
                .tag_typed(EntityTag::new("ready"))
                .to_string(),
        ),
        (
            PlayerTargets::all().team("red").to_string(),
            Selector::all_players().team("red").to_string(),
        ),
        (
            PlayerTargets::all().not_team("blue").to_string(),
            Selector::all_players().not_team("blue").to_string(),
        ),
        (
            PlayerTargets::all()
                .team_typed(TeamName::new("red"))
                .to_string(),
            Selector::all_players()
                .team_typed(TeamName::new("red"))
                .to_string(),
        ),
        (
            PlayerTargets::all().name("Steve").to_string(),
            Selector::all_players().name("Steve").to_string(),
        ),
        (
            PlayerTargets::all().not_name("Notch").to_string(),
            Selector::all_players().not_name("Notch").to_string(),
        ),
        (
            PlayerTargets::all().within_blocks(16.0).to_string(),
            Selector::all_players().distance_max(16.0).to_string(),
        ),
        (
            PlayerTargets::all().distance_min(2.0).to_string(),
            Selector::all_players().distance_min(2.0).to_string(),
        ),
        (
            PlayerTargets::all().distance_range(0.5, 10.0).to_string(),
            Selector::all_players()
                .distance_range(0.5, 10.0)
                .to_string(),
        ),
        (
            PlayerTargets::all().distance("1..4").to_string(),
            Selector::all_players().distance("1..4").to_string(),
        ),
        (
            PlayerTargets::all()
                .distance_typed(SelectorRange::between(0.5, 10.0))
                .to_string(),
            Selector::all_players()
                .distance_typed(SelectorRange::between(0.5, 10.0))
                .to_string(),
        ),
        (
            PlayerTargets::all().excluding_self().to_string(),
            Selector::all_players().distance("0.1..").to_string(),
        ),
        (
            PlayerTargets::all().level("10..30").to_string(),
            Selector::all_players().level("10..30").to_string(),
        ),
        (
            PlayerTargets::all()
                .level_typed(SelectorRange::between(10.0, 30.0))
                .to_string(),
            Selector::all_players()
                .level_typed(SelectorRange::between(10.0, 30.0))
                .to_string(),
        ),
        (
            PlayerTargets::all().gamemode("survival").to_string(),
            Selector::all_players().gamemode("survival").to_string(),
        ),
        (
            PlayerTargets::all()
                .gamemode_typed(GameMode::Adventure)
                .to_string(),
            Selector::all_players()
                .gamemode_typed(GameMode::Adventure)
                .to_string(),
        ),
        (
            PlayerTargets::all()
                .not_gamemode_typed(GameMode::Creative)
                .to_string(),
            Selector::all_players()
                .not_gamemode_typed(GameMode::Creative)
                .to_string(),
        ),
        (
            PlayerTargets::all()
                .scores_typed(
                    SelectorScores::new()
                        .with("kills", ScoreRange::between(1, 10))
                        .with("deaths", ScoreRange::exact(0)),
                )
                .to_string(),
            Selector::all_players()
                .scores_typed(
                    SelectorScores::new()
                        .with("kills", ScoreRange::between(1, 10))
                        .with("deaths", ScoreRange::exact(0)),
                )
                .to_string(),
        ),
        (
            PlayerTargets::all()
                .score(ObjectiveName::new("kills"), ScoreRange::at_least(1))
                .unwrap()
                .to_string(),
            Selector::all_players()
                .score_typed(ObjectiveName::new("kills"), ScoreRange::at_least(1))
                .unwrap()
                .to_string(),
        ),
        (
            PlayerTargets::all()
                .predicate_id(PredicateId::new("pack:is_sneaking"))
                .to_string(),
            Selector::all_players()
                .predicate_id(PredicateId::new("pack:is_sneaking"))
                .to_string(),
        ),
        (
            PlayerTargets::all().volume(3.0, 1.0, 3.0).to_string(),
            Selector::all_players().volume(3.0, 1.0, 3.0).to_string(),
        ),
        (
            PlayerTargets::all().at_pos(10.0, 64.0, -20.0).to_string(),
            Selector::all_players()
                .at_pos(10.0, 64.0, -20.0)
                .to_string(),
        ),
        (
            PlayerTargets::all().scores_raw("kills=1..10").to_string(),
            Selector::all_players()
                .scores_raw("kills=1..10")
                .to_string(),
        ),
        (
            PlayerTargets::all().nbt_raw("{Health:20.0f}").to_string(),
            Selector::all_players()
                .nbt_raw("{Health:20.0f}")
                .to_string(),
        ),
        (
            PlayerTargets::all().predicate_raw("pack:ready").to_string(),
            Selector::all_players()
                .predicate_raw("pack:ready")
                .to_string(),
        ),
    ];
    for (index, (wrapper, direct)) in cases.iter().enumerate() {
        assert_eq!(wrapper, direct, "player parity case {index}");
    }
}

// ── Single-cardinality parity (same helpers, `@s`-based bases) ───────────────

#[test]
fn single_entity_filters_render_identically_to_selector() {
    assert_eq!(
        SingleEntity::self_()
            .tag("elite")
            .team("red")
            .distance_typed(SelectorRange::at_most(8.0))
            .predicate_id(PredicateId::new("pack:ready"))
            .to_string(),
        Selector::self_()
            .tag("elite")
            .team("red")
            .distance_typed(SelectorRange::at_most(8.0))
            .predicate_id(PredicateId::new("pack:ready"))
            .to_string()
    );
}

#[test]
fn single_player_filters_render_identically_to_selector() {
    assert_eq!(
        SinglePlayer::nearest()
            .tag("ready")
            .level_typed(SelectorRange::at_least(30.0))
            .gamemode_typed(GameMode::Survival)
            .scores_typed(SelectorScores::new().with("kills", ScoreRange::at_least(1)))
            .to_string(),
        Selector::nearest_player()
            .tag("ready")
            .level_typed(SelectorRange::at_least(30.0))
            .gamemode_typed(GameMode::Survival)
            .scores_typed(SelectorScores::new().with("kills", ScoreRange::at_least(1)))
            .to_string()
    );
}

// ── Cardinality preservation ─────────────────────────────────────────────────

#[test]
fn entity_filters_preserve_cardinality_marker() {
    takes_entity_targets(
        EntityTargets::all()
            .tag("a")
            .not_tag("b")
            .tag_typed(EntityTag::new("c"))
            .team("red")
            .not_team("blue")
            .team_typed(TeamName::new("green"))
            .name("Boss")
            .not_name("Mini")
            .entity_type("minecraft:zombie")
            .not_type("minecraft:cow")
            .excluding_players()
            .within_blocks(16.0)
            .distance_min(1.0)
            .distance_range(1.0, 2.0)
            .distance("1..2")
            .distance_typed(SelectorRange::at_most(4.0))
            .excluding_self()
            .scores_typed(SelectorScores::new().with("k", ScoreRange::exact(1)))
            .predicate_id(PredicateId::new("pack:a"))
            .volume(1.0, 1.0, 1.0)
            .at_pos(0.0, 0.0, 0.0)
            .scores_raw("k=1")
            .nbt_raw("{a:1}")
            .predicate_raw("pack:b"),
    );

    let single = SingleEntity::self_()
        .tag("a")
        .not_tag("b")
        .tag_typed(EntityTag::new("c"))
        .team("red")
        .not_team("blue")
        .team_typed(TeamName::new("green"))
        .name("Boss")
        .not_name("Mini")
        .entity_type("minecraft:zombie")
        .not_type("minecraft:cow")
        .excluding_players()
        .within_blocks(16.0)
        .distance_min(1.0)
        .distance_range(1.0, 2.0)
        .distance("1..2")
        .distance_typed(SelectorRange::at_most(4.0))
        .excluding_self()
        .scores_typed(SelectorScores::new().with("k", ScoreRange::exact(1)))
        .predicate_id(PredicateId::new("pack:a"))
        .volume(1.0, 1.0, 1.0)
        .at_pos(0.0, 0.0, 0.0)
        .scores_raw("k=1")
        .nbt_raw("{a:1}")
        .predicate_raw("pack:b");
    takes_single_entity(single.clone());
    takes_single_entity(
        single
            .score(ObjectiveName::new("threat"), ScoreRange::at_least(1))
            .unwrap(),
    );
}

#[test]
fn player_filters_preserve_cardinality_marker() {
    takes_player_targets(
        PlayerTargets::all()
            .tag("a")
            .not_tag("b")
            .tag_typed(EntityTag::new("c"))
            .team("red")
            .not_team("blue")
            .team_typed(TeamName::new("green"))
            .name("Steve")
            .not_name("Notch")
            .within_blocks(16.0)
            .distance_min(1.0)
            .distance_range(1.0, 2.0)
            .distance("1..2")
            .distance_typed(SelectorRange::at_most(4.0))
            .excluding_self()
            .level("1..2")
            .level_typed(SelectorRange::at_least(3.0))
            .gamemode("survival")
            .gamemode_typed(GameMode::Creative)
            .not_gamemode_typed(GameMode::Spectator)
            .scores_typed(SelectorScores::new().with("k", ScoreRange::exact(1)))
            .predicate_id(PredicateId::new("pack:a"))
            .volume(1.0, 1.0, 1.0)
            .at_pos(0.0, 0.0, 0.0)
            .scores_raw("k=1")
            .nbt_raw("{a:1}")
            .predicate_raw("pack:b"),
    );

    let single = SinglePlayer::self_()
        .tag("a")
        .not_tag("b")
        .tag_typed(EntityTag::new("c"))
        .team("red")
        .not_team("blue")
        .team_typed(TeamName::new("green"))
        .name("Steve")
        .not_name("Notch")
        .within_blocks(16.0)
        .distance_min(1.0)
        .distance_range(1.0, 2.0)
        .distance("1..2")
        .distance_typed(SelectorRange::at_most(4.0))
        .excluding_self()
        .level("1..2")
        .level_typed(SelectorRange::at_least(3.0))
        .gamemode("survival")
        .gamemode_typed(GameMode::Creative)
        .not_gamemode_typed(GameMode::Spectator)
        .scores_typed(SelectorScores::new().with("k", ScoreRange::exact(1)))
        .predicate_id(PredicateId::new("pack:a"))
        .volume(1.0, 1.0, 1.0)
        .at_pos(0.0, 0.0, 0.0)
        .scores_raw("k=1")
        .nbt_raw("{a:1}")
        .predicate_raw("pack:b");
    takes_single_player(single.clone());
    takes_single_player(
        single
            .score(ObjectiveName::new("kills"), ScoreRange::at_least(1))
            .unwrap(),
    );
}

#[test]
fn cardinality_narrowing_still_flows_through_dedicated_methods() {
    // `limit`/`sort` are deliberately NOT forwarded as generic filters: the
    // only ways to narrow are the cardinality-changing helpers, which return
    // the single-target types.
    takes_single_entity(EntityTargets::all().tag("elite").nearest());
    takes_single_entity(EntityTargets::all().tag("elite").limit(1).unwrap());
    takes_single_player(PlayerTargets::all().tag("ready").nearest());
    takes_single_player(PlayerTargets::all().tag("ready").limit(1).unwrap());
    assert!(EntityTargets::all().tag("elite").limit(2).is_err());
    assert!(PlayerTargets::all().tag("ready").limit(0).is_err());
}

// ── Validation is delegated, never reimplemented ─────────────────────────────

fn assert_same_error(
    wrapper: sand_commands::CommandResult<String>,
    direct: Result<String, String>,
) {
    let wrapper = wrapper.map_err(|e| e.to_string());
    assert!(wrapper.is_err(), "expected wrapper error, got {wrapper:?}");
    assert_eq!(wrapper.err(), direct.err());
}

#[test]
fn invalid_predicate_id_produces_the_same_error_as_selector() {
    assert_same_error(
        EntityTargets::all()
            .predicate_id(PredicateId::new("NOT A LOCATION"))
            .try_build(),
        Selector::all_entities()
            .predicate_id(PredicateId::new("NOT A LOCATION"))
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        PlayerTargets::all().predicate_raw("Bad Id").try_build(),
        Selector::all_players()
            .predicate_raw("Bad Id")
            .try_build()
            .map_err(|e| e.to_string()),
    );
}

#[test]
fn impossible_integer_score_range_produces_the_same_error_as_selector() {
    let scores = || SelectorScores::new().with("kills", ScoreRange::between(10, 1));
    assert_same_error(
        PlayerTargets::all().scores_typed(scores()).try_build(),
        Selector::all_players()
            .scores_typed(scores())
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        EntityTargets::all().scores_typed(scores()).try_build(),
        Selector::all_entities()
            .scores_typed(scores())
            .try_build()
            .map_err(|e| e.to_string()),
    );
}

#[test]
fn non_finite_float_ranges_produce_the_same_error_as_selector() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_same_error(
            EntityTargets::all()
                .distance_typed(SelectorRange::at_most(bad))
                .try_build(),
            Selector::all_entities()
                .distance_typed(SelectorRange::at_most(bad))
                .try_build()
                .map_err(|e| e.to_string()),
        );
        assert_same_error(
            PlayerTargets::all()
                .level_typed(SelectorRange::at_least(bad))
                .try_build(),
            Selector::all_players()
                .level_typed(SelectorRange::at_least(bad))
                .try_build()
                .map_err(|e| e.to_string()),
        );
        assert_same_error(
            EntityTargets::all().volume(bad, 1.0, 1.0).try_build(),
            Selector::all_entities()
                .volume(bad, 1.0, 1.0)
                .try_build()
                .map_err(|e| e.to_string()),
        );
        assert_same_error(
            PlayerTargets::all().at_pos(1.0, bad, 1.0).try_build(),
            Selector::all_players()
                .at_pos(1.0, bad, 1.0)
                .try_build()
                .map_err(|e| e.to_string()),
        );
    }
}

#[test]
fn impossible_float_range_produces_the_same_error_as_selector() {
    assert_same_error(
        EntityTargets::all()
            .distance_typed(SelectorRange::between(10.0, 1.0))
            .try_build(),
        Selector::all_entities()
            .distance_typed(SelectorRange::between(10.0, 1.0))
            .try_build()
            .map_err(|e| e.to_string()),
    );
}

#[test]
fn invalid_score_objective_produces_the_same_error_as_selector() {
    let wrapper = EntityTargets::all()
        .score(ObjectiveName::new("has space"), ScoreRange::exact(1))
        .map(|t| t.to_string())
        .map_err(|e| e.to_string());
    let direct = Selector::all_entities()
        .score_typed(ObjectiveName::new("has space"), ScoreRange::exact(1))
        .map(|s| s.to_string())
        .map_err(|e| e.to_string());
    assert!(wrapper.is_err(), "expected wrapper error, got {wrapper:?}");
    assert_eq!(wrapper.err(), direct.err());

    let wrapper = PlayerTargets::all()
        .score(ObjectiveName::new(""), ScoreRange::exact(1))
        .map(|t| t.to_string())
        .map_err(|e| e.to_string());
    let direct = Selector::all_players()
        .score_typed(ObjectiveName::new(""), ScoreRange::exact(1))
        .map(|s| s.to_string())
        .map_err(|e| e.to_string());
    assert!(wrapper.is_err(), "expected wrapper error, got {wrapper:?}");
    assert_eq!(wrapper.err(), direct.err());
}

#[test]
fn duplicate_score_objective_produces_the_same_error_as_selector() {
    let wrapper = PlayerTargets::all()
        .score(ObjectiveName::new("kills"), ScoreRange::exact(1))
        .unwrap()
        .score(ObjectiveName::new("kills"), ScoreRange::exact(2))
        .map(|t| t.to_string())
        .map_err(|e| e.to_string());
    let direct = Selector::all_players()
        .score_typed(ObjectiveName::new("kills"), ScoreRange::exact(1))
        .unwrap()
        .score_typed(ObjectiveName::new("kills"), ScoreRange::exact(2))
        .map(|s| s.to_string())
        .map_err(|e| e.to_string());
    assert!(wrapper.is_err(), "expected wrapper error, got {wrapper:?}");
    assert_eq!(wrapper.err(), direct.err());
}

#[test]
fn invalid_tag_and_team_produce_the_same_error_as_selector() {
    assert_same_error(
        EntityTargets::all()
            .tag_typed(EntityTag::new("has space"))
            .try_build(),
        Selector::all_entities()
            .tag_typed(EntityTag::new("has space"))
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        PlayerTargets::all().not_tag("has space").try_build(),
        Selector::all_players()
            .not_tag("has space")
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        EntityTargets::all()
            .team_typed(TeamName::new("has space"))
            .try_build(),
        Selector::all_entities()
            .team_typed(TeamName::new("has space"))
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        PlayerTargets::all().not_team("bad\tteam").try_build(),
        Selector::all_players()
            .not_team("bad\tteam")
            .try_build()
            .map_err(|e| e.to_string()),
    );
}

#[test]
fn invalid_gamemode_level_and_type_produce_the_same_error_as_selector() {
    assert_same_error(
        PlayerTargets::all().gamemode("hardcore").try_build(),
        Selector::all_players()
            .gamemode("hardcore")
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        PlayerTargets::all().level("-1..").try_build(),
        Selector::all_players()
            .level("-1..")
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        EntityTargets::all().entity_type("not a type").try_build(),
        Selector::all_entities()
            .entity_type("not a type")
            .try_build()
            .map_err(|e| e.to_string()),
    );
    assert_same_error(
        EntityTargets::all().nbt_raw("{broken:[1,2}").try_build(),
        Selector::all_entities()
            .nbt_raw("{broken:[1,2}")
            .try_build()
            .map_err(|e| e.to_string()),
    );
}

#[test]
fn wrapper_validate_matches_selector_validate() {
    let profile = sand_commands::CommandProfile::unprofiled();
    let wrapper = PlayerTargets::all().level_typed(SelectorRange::between(30.0, 1.0));
    let direct = Selector::all_players().level_typed(SelectorRange::between(30.0, 1.0));
    assert_eq!(
        wrapper.validate(&profile).map_err(|e| e.to_string()),
        direct.validate(&profile).map_err(|e| e.to_string())
    );
}

// ── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn typed_target_score_map_ordering_is_insertion_ordered() {
    let forward = EntityTargets::all().scores_typed(
        SelectorScores::new()
            .with("threat", ScoreRange::at_least(5))
            .with("armor", ScoreRange::between(0, 3))
            .with("kills", ScoreRange::exact(0)),
    );
    assert_eq!(
        forward.to_string(),
        "@e[scores={threat=5..,armor=0..3,kills=0}]"
    );

    let reversed = EntityTargets::all().scores_typed(
        SelectorScores::new()
            .with("kills", ScoreRange::exact(0))
            .with("armor", ScoreRange::between(0, 3))
            .with("threat", ScoreRange::at_least(5)),
    );
    assert_eq!(
        reversed.to_string(),
        "@e[scores={kills=0,armor=0..3,threat=5..}]"
    );
    assert_ne!(forward.to_string(), reversed.to_string());

    // Chained `score` calls merge into one `scores={...}` argument, also in
    // insertion order.
    let chained = PlayerTargets::all()
        .score(ObjectiveName::new("kills"), ScoreRange::at_least(1))
        .unwrap()
        .score(ObjectiveName::new("deaths"), ScoreRange::exact(0))
        .unwrap();
    assert_eq!(chained.to_string(), "@a[scores={kills=1..,deaths=0}]");
}

#[test]
fn typed_target_rendering_is_repeatable() {
    // Each iteration rebuilds from scratch, so a hasher-seeded map anywhere in
    // the filter pipeline would surface as flaky inequality across runs.
    let build_entity = || {
        EntityTargets::all()
            .entity_type("minecraft:zombie")
            .tag("elite")
            .team_typed(TeamName::new("red"))
            .distance_typed(SelectorRange::at_most(20.0))
            .scores_typed(
                SelectorScores::new()
                    .with("threat", ScoreRange::at_least(5))
                    .with("armor", ScoreRange::between(0, 3)),
            )
            .predicate_id(PredicateId::new("pack:is_burning"))
            .to_string()
    };
    let build_player = || {
        PlayerTargets::all()
            .tag("ready")
            .gamemode_typed(GameMode::Adventure)
            .level_typed(SelectorRange::at_least(30.0))
            .scores_typed(SelectorScores::new().with("kills", ScoreRange::between(1, 10)))
            .to_string()
    };

    let entity_expected = "@e[type=minecraft:zombie,tag=elite,team=red,distance=..20,scores={threat=5..,armor=0..3},predicate=pack:is_burning]";
    let player_expected = "@a[tag=ready,gamemode=adventure,level=30..,scores={kills=1..10}]";
    assert_eq!(build_entity(), entity_expected);
    assert_eq!(build_player(), player_expected);
    for _ in 0..128 {
        assert_eq!(build_entity(), entity_expected);
        assert_eq!(build_player(), player_expected);
    }
}

// ── Widening conversions keep forwarded filters intact ───────────────────────

#[test]
fn player_targets_widen_to_entity_targets_with_filters_preserved() {
    let players = PlayerTargets::all()
        .tag("ready")
        .gamemode_typed(GameMode::Survival);
    let rendered = players.to_string();
    let entities: EntityTargets = players.into();
    assert_eq!(entities.to_string(), rendered);

    let player = SinglePlayer::self_().level_typed(SelectorRange::at_least(5.0));
    let rendered = player.to_string();
    let entity: SingleEntity = player.into();
    assert_eq!(entity.to_string(), rendered);
}
