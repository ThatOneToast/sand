mod support;
use support::*;

#[test]
fn entity_filters_compose_with_each_other_and_native_selector_options() {
    let analysis = analyze_one(include_str!("fixtures/entity_filters.sand"));
    let pack = compile(&analysis);
    let commands = function(&pack, "language_probe/function/run");
    assert!(commands.contains("=-3..4"), "{commands}");
    assert!(
        commands.contains("nbt={data:{quest:{stage:2,active:1b}}}"),
        "{commands}"
    );
    assert!(
        commands.lines().any(|line| {
            line.contains("scores={")
                && line.contains("=1,")
                && line.contains("=-3..}")
                && line.contains("nbt={Tags:[\"low\"]}")
        }),
        "{commands}"
    );
}

#[test]
fn entity_filters_require_score_fields_ordered_bounds_and_typed_data() {
    for (query, expected) in [
        ("score = Threat.level", "supply both score and score_range"),
        (
            "score = Shared.level, score_range = 1",
            "query scores require a number field",
        ),
        (
            "score = Threat.name, score_range = 1",
            "query scores require a number field",
        ),
        (
            "score = Threat.level, score_range = 2..2",
            "query score bounds must be ordered",
        ),
        (
            "score = Threat.level, score_range = ..-2147483648",
            "exclusive score bound",
        ),
        ("nbt = \"{Silent:1b}\"", "expected a typed compound"),
        (
            "nbt = Nbt { Silent: true, Silent: false }",
            "duplicate NBT field",
        ),
    ] {
        let source = format!(
            r#"
#![pack(namespace = "entities_example")]
global state Shared {{ level: number }}
entity state Threat {{ level: number, name: string }}
fn run() {{ if any_entities({query}) {{ return; }} }}
"#
        );
        let analysis = analyze_one(&source);
        assert!(
            errors(&analysis)
                .iter()
                .any(|message| message.contains(expected)),
            "{query}: {:?}",
            errors(&analysis)
        );
    }
}

#[test]
fn player_score_filter_checks_each_candidate_without_a_player_executor() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "entities_example")]
player state Rank { level: number }
#[load]
fn run() {
    if any_entities(type = Entity::Player, score = Rank.level, score_range = 3..) { return; }
}
"#,
    );
    compile(&analysis);
}
