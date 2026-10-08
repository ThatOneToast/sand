//! Concrete declarations reuse State and archetype lifecycle resources.
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "concrete", scope = living)]
pub struct Combat {
    #[state(default = 20)]
    pub health: Score,
}

#[derive(State)]
#[state(namespace = "concrete", scope = living)]
pub struct Progression {
    #[state(default = 1)]
    pub level: Score,
}

#[derive(StateBundle)]
pub struct Growth {
    pub progression: Progression,
}

#[derive(Archetype)]
#[archetype(id = "concrete:seeker", entity = Zombie, configure = Self::configure)]
pub struct Seeker {
    pub combat: Combat,
    pub growth: Growth,
}

impl Seeker {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype.derive(
            Combat::health,
            StatCurve::linear(StatCurve::state(Progression::level), 2.0, 18.0),
        )
    }
}

#[derive(Archetype)]
#[archetype(id = "concrete:guard", entity = Zombie)]
pub struct Guard {
    pub combat: Combat,
}

#[function("concrete:spawn")]
pub fn spawn() {
    Seeker::summon(Vec3::here());
}

#[system(tick, every = 20)]
fn seekers(query: Seeker) {
    query.each(|seeker| seeker.growth.progression.level.add(1));
}

#[test]
fn concrete_declarations_export_the_canonical_composition_and_queries() {
    let first = sand::advanced::try_export_components_json("concrete", "26.2").unwrap();
    let second = sand::advanced::try_export_components_json("concrete", "26.2").unwrap();
    assert_eq!(first, second);
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let spawn = records
        .iter()
        .find(|record| record["path"] == "spawn" && record["dir"] == "function")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    assert!(
        spawn
            .starts_with("execute positioned ~ ~ ~ summon minecraft:zombie run function concrete:")
    );
    assert!(spawn.ends_with("/initialize"));
    assert!(
        records
            .iter()
            .any(|record| record["content"].as_str().is_some_and(|body| {
                body.contains("type=minecraft:zombie,tag=__sand.a.") && body.contains("scores=")
            }))
    );
    let shared_objective = format!(
        "scoreboard objectives add {} dummy",
        Combat::health.objective()
    );
    assert_eq!(
        records
            .iter()
            .filter_map(|record| record["content"].as_str())
            .flat_map(str::lines)
            .filter(|line| *line == shared_objective)
            .count(),
        1
    );
    let seeker = Seeker::on(EntityContext::<ZombieKind>::default());
    assert!(seeker.combat.health.add(1)[0].contains("scoreboard players add @s"));
    assert!(seeker.growth.progression.level.add(1)[0].contains("scoreboard players add @s"));
    let _: EntityContext<ZombieKind> = seeker.entity();
    assert_ne!(
        Seeker::attach(Default::default()),
        Guard::attach(Default::default())
    );
    assert!(Seeker::detach(Default::default())[0].ends_with("/cleanup"));
}

#[test]
fn adoption_preserves_a_conflicting_selection_instead_of_retargeting_it() {
    let commands = Seeker::adopt(Target::entities().entity_type(vanilla::EntityType::Skeleton));
    assert!(commands[0].starts_with("execute as @e[type=minecraft:skeleton] at @s if entity @s[type=minecraft:zombie] run function concrete:"));
}
