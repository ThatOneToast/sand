use sand::prelude::*;

#[derive(State)]
#[state(namespace = "cf", scope = living)]
#[allow(dead_code)]
struct Progression {
    #[state(default = 1, min = 1, max = 100)]
    level: Score,
}

#[derive(State)]
#[state(namespace = "cf", scope = living)]
#[allow(dead_code)]
struct Combat {
    #[state(default = 20, min = 1, max = 2000)]
    maximum: Score,
    #[state(default = 20, min = 0, max = 2000)]
    current: Score,
}

#[derive(State)]
#[state(namespace = "cf", scope = living)]
#[allow(dead_code)]
struct Scaling {
    #[state(default = 1)]
    power: Score,
}

#[derive(State)]
#[state(namespace = "cf", scope = living)]
#[allow(dead_code)]
struct Conditions {
    #[state(default = false)]
    weakened: Flag,
}

#[derive(StateBundle)]
#[allow(dead_code)]
struct CoreStats {
    progression: Progression,
    combat: Combat,
}

#[derive(StateBundle)]
#[allow(dead_code)]
struct NestedStats {
    core: CoreStats,
    scaling: Scaling,
}

#[entity_archetype]
fn valid_component_first() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_valid:seeker".parse().unwrap())
        .components::<NestedStats>()
        .components::<Conditions>()
        .components::<Progression>()
        .derive(
            Combat::maximum,
            StatCurve::linear(StatCurve::state(Progression::level), 2.0, 18.0),
        )
        .derive(
            Scaling::power,
            StatCurve::linear(StatCurve::state(Combat::maximum), 1.0, 0.0),
        )
        .health(
            HealthBinding::new(Combat::maximum)
                .current_health(Combat::current, CurrentHealthSync::Bidirectional),
        )
        .effect_when(
            Conditions::weakened,
            EffectBinding::new(
                StatusEffectId::minecraft("weakness").unwrap(),
                Ticks::seconds(5),
            ),
        )
        .name(
            EntityName::new()
                .text(Text::new("Lv. ").gold())
                .state(Progression::level, ChatColor::Yellow)
                .text(Text::new(" [").gray())
                .state(Combat::current, ChatColor::Red)
                .text(Text::new("]").gray())
                .refresh_every(Ticks::new(5)),
        )
}

#[entity_archetype]
fn shared_progression_user() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_valid:progression_observer".parse().unwrap())
        .components::<Progression>()
}

#[test]
fn flattened_components_drive_cross_component_properties_deterministically() {
    let first = sand_core::try_export_components_json("cf_valid")
        .expect("component-first archetype should export");
    let second =
        sand_core::try_export_components_json("cf_valid").expect("second export should succeed");
    assert_eq!(first, second);

    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions = records
        .iter()
        .filter_map(|record| record["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(functions.contains(&Progression::level.objective()));
    assert!(functions.contains(&Combat::maximum.objective()));
    assert!(functions.contains(&Scaling::power.objective()));
    assert!(functions.contains(&Conditions::weakened.objective()));
    assert!(functions.contains(&Progression::level.dirty_objective()));
    assert!(functions.contains(&Combat::maximum.dirty_objective()));
    assert!(functions.contains("CustomName"));
    assert!(functions.contains("weakness"));
    assert!(functions.contains("\"color\":\"gold\""));
    assert!(functions.contains("\"color\":\"gray\""));
    assert!(functions.contains("color:\"yellow\""));
    assert!(functions.contains("color:\"red\""));
    assert!(functions.contains("matches 5.."));

    let provision = records
        .iter()
        .find(|record| {
            record["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/provision"))
                && record["content"]
                    .as_str()
                    .is_some_and(|content| content.contains("scoreboard players"))
        })
        .and_then(|record| record["content"].as_str())
        .expect("archetype component provisioning function");
    assert_eq!(
        provision
            .matches(&format!(
                "{} matches",
                Progression::composition_identities()[0].0
            ))
            .count(),
        1,
        "duplicate direct/bundle composition must use one canonical attach path",
    );
    assert!(
        provision.contains("execute unless score @s"),
        "adoption must preserve already-attached component values"
    );

    let initialize_record = records
        .iter()
        .find(|record| {
            record["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/initialize"))
                && record["content"]
                    .as_str()
                    .is_some_and(|content| content.contains("/derive/0"))
        })
        .expect("seeker initialization function");
    let initialize = initialize_record["content"].as_str().unwrap();
    let first_derivation = initialize
        .lines()
        .position(|line| line.contains("/derive/0"))
        .expect("first derivation must be seeded during adoption");
    let second_derivation = initialize
        .lines()
        .position(|line| line.contains("/derive/1"))
        .expect("chained derivation must be seeded during adoption");
    assert!(first_derivation < second_derivation);

    let cleanup_path = format!(
        "{}/cleanup",
        initialize_record["path"]
            .as_str()
            .unwrap()
            .trim_end_matches("/initialize")
    );
    let cleanup = records
        .iter()
        .find(|record| record["path"] == cleanup_path)
        .and_then(|record| record["content"].as_str())
        .expect("seeker cleanup function");
    let shared_presence = &Progression::composition_identities()[0].0;
    let exclusive_presence = &Combat::composition_identities()[0].0;
    assert!(
        cleanup
            .lines()
            .filter(|line| line.contains(shared_presence))
            .all(|line| line.starts_with("execute unless entity @s[tag=")),
        "cleanup must preserve a component still claimed by another archetype"
    );
    assert!(
        cleanup
            .lines()
            .any(|line| line.contains(exclusive_presence)
                && !line.starts_with("execute unless entity")),
        "cleanup must still detach components exclusive to this archetype"
    );
    assert!(
        !cleanup.lines().any(|line| {
            line == format!(
                "scoreboard players reset @s {}",
                Progression::level.objective()
            )
        }),
        "archetype cleanup must not bypass component ownership with an unconditional field reset"
    );
}
