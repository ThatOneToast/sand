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
        archetype.adopt(Adoption::external()).derive(
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
    let seeker = Seeker::on(EntityContext::<ZombieKind>::default());
    let guard = Guard::on(EntityContext::<ZombieKind>::default());
    // Both archetypes expose the same physical component objective and dirty
    // marker; provisioning the same objective from several lifecycle helpers
    // does not create separate State storage.
    assert_eq!(seeker.combat.health.set(7), guard.combat.health.set(7));
    assert!(seeker.combat.health.set(7)[0].contains(&Combat::health.objective()));
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
    assert!(commands[0].starts_with("execute as @e[type=minecraft:skeleton] at @s if entity @s[type=minecraft:zombie] unless entity @s[tag=__sand.a."));
}

#[test]
fn lifecycle_calls_guard_membership_before_invoking_callbacks_or_cleanup() {
    let attach = Seeker::attach(Default::default());
    let detach = Seeker::detach(Default::default());
    let adopt = Seeker::adopt(Target::entities());
    assert_eq!(attach.len(), 1);
    assert_eq!(detach.len(), 1);
    assert_eq!(adopt.len(), 1);
    let marker = attach[0]
        .split("unless entity @s[tag=")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap();
    assert!(
        attach[0].starts_with("execute if entity @s[type=minecraft:zombie] unless entity @s[tag=")
    );
    assert!(attach[0].ends_with("/initialize"));
    assert!(adopt[0].contains(&format!("unless entity @s[tag={marker}] run function")));
    assert!(detach[0].starts_with(&format!(
        "execute if entity @s[type=minecraft:zombie,tag={marker}] run function"
    )));
    assert!(detach[0].ends_with("/cleanup"));
    // The same membership bit gates both entry and exit; independently attached
    // State cannot enter cleanup without membership, and repeat attachment cannot
    // rerun the native initializer or callback.
}

#[test]
fn concrete_external_adoption_tag_matches_exported_scan_and_cleanup() {
    let tag = Seeker::external_adoption_tag();
    assert_ne!(tag, Guard::external_adoption_tag());
    assert_eq!(tag, Seeker::external_adoption_tag());
    let export = sand::advanced::try_export_components_json("concrete", "26.2").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&export).unwrap();
    let scan = format!(",tag={}]", tag.as_str());
    assert!(
        records
            .iter()
            .any(|record| record["content"].as_str().is_some_and(|body| body
                .contains("execute as @e[type=minecraft:zombie,tag=!")
                && body.contains(&scan)
                && body.contains("/initialize")))
    );
    let cleanup = format!("tag @s remove {}", tag.as_str());
    assert!(records.iter().any(|record| {
        record["content"]
            .as_str()
            .is_some_and(|body| body.lines().any(|line| line == cleanup))
    }));
}
