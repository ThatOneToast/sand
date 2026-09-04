use sand::prelude::*;

#[derive(State)]
#[state(namespace = "cf_missing_name", scope = living)]
#[allow(dead_code)]
struct Progression {
    level: Score,
}

#[derive(State)]
#[state(namespace = "cf_missing_name", scope = living)]
#[allow(dead_code)]
struct Unrelated {
    value: Score,
}

#[entity_archetype]
fn missing_name_field() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("cf_missing_name:seeker".parse().unwrap())
        .components::<Progression>()
        .name(EntityName::new().state(Unrelated::value, ChatColor::Red))
}

#[test]
fn unattached_name_field_uses_the_same_membership_diagnostic() {
    let message = sand_core::try_export_components_json("cf_missing_name")
        .unwrap_err()
        .to_string();
    assert!(message.contains("cf_missing_name:seeker"));
    assert!(message.contains("name"));
    assert!(message.contains("cf_missing_name:unrelated"));
    assert!(message.contains("value"));
}
