//! Sand-only authoring example: define each ID once and reuse the component.
use sand::prelude::*;

#[datapack_component]
fn equipped() -> Predicate {
    let item = || ItemPredicate::new().item(ItemId::minecraft("diamond").unwrap());
    Predicate::new(
        PredicateId::custom(ResourceLocation::new("__sand_local", "equipped").unwrap()),
        PredicateRoot::entity_properties(
            EntityPredicateTarget::This,
            EntityPredicate::new().equipment(
                EntityEquipment::new()
                    .head(item())
                    .chest(item())
                    .legs(item())
                    .feet(item())
                    .mainhand(item())
                    .offhand(item()),
            ),
        ),
    )
}

#[datapack_component]
fn delegated() -> Predicate {
    Predicate::new(
        PredicateId::custom(ResourceLocation::new("__sand_local", "delegated").unwrap()),
        PredicateRoot::reference(equipped()),
    )
}

struct Equipped;
impl SandEvent for Equipped {
    fn dispatch() -> impl Into<SandEventDispatch> {
        let retained = equipped();
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::predicate(&retained))
            .when(Condition::predicate(delegated()))
            .when(Condition::entity(Target::self_().predicate(&retained)))
    }
}

#[on_event]
fn on_equipped(_event: Equipped) {
    cmd::say("equipped");
}

#[function]
fn check_predicates() {
    let retained = equipped();
    when(Condition::predicate(&retained)).then_one(cmd::say("borrowed"));
    when(!Condition::predicate(equipped())).then_one(cmd::say("negated"));
    when(Condition::entity(
        Target::entities().not_predicate(&retained),
    ))
    .then_one(cmd::say("excluded"));
    when(Condition::entity(
        Target::entities()
            .predicate(equipped())
            .not_predicate(delegated()),
    ))
    .then_one(cmd::say("selected"));
}

fn snapshot(namespace: &str) -> Vec<(String, String, String, String)> {
    sand_core::try_export_components(namespace)
        .unwrap()
        .into_iter()
        .map(|r| (r.namespace, r.dir, r.path, r.content))
        .collect()
}

#[test]
fn references_export_the_registered_identity_and_preserve_context_and_negation() {
    let records = snapshot("bridge");
    let content = |dir: &str, path: &str| {
        &records
            .iter()
            .find(|r| r.0 == "bridge" && r.1 == dir && r.2 == path)
            .unwrap()
            .3
    };
    let equipment: serde_json::Value =
        serde_json::from_str(content("predicate", "equipped")).unwrap();
    for slot in ["head", "chest", "legs", "feet", "mainhand", "offhand"] {
        assert_eq!(
            equipment["predicate"]["equipment"][slot]["items"],
            serde_json::json!("minecraft:diamond")
        );
    }
    let reference: serde_json::Value =
        serde_json::from_str(content("predicate", "delegated")).unwrap();
    assert_eq!(
        reference,
        serde_json::json!({"condition":"minecraft:reference", "name":"bridge:equipped"})
    );
    let commands = content("function", "check_predicates");
    for expected in [
        "execute if predicate bridge:equipped run say borrowed",
        "execute unless predicate bridge:equipped run say negated",
        "execute if entity @e[predicate=!bridge:equipped] run say excluded",
        "execute if entity @e[predicate=bridge:equipped,predicate=!bridge:delegated] run say selected",
    ] {
        assert!(commands.contains(expected), "{commands}");
    }
    let all = records
        .iter()
        .map(|r| r.3.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("as @a"), "{all}");
    assert!(all.contains("if predicate bridge:equipped"), "{all}");
    assert!(
        all.contains("if entity @s[predicate=bridge:equipped]"),
        "{all}"
    );
    assert!(!all.contains("__sand_local"));
    assert_eq!(records.iter().filter(|r| r.1 == "predicate").count(), 2);
    assert_eq!(records, snapshot("bridge"));
    let (same, other) = std::thread::scope(|scope| {
        let same = scope.spawn(|| snapshot("bridge"));
        let other = scope.spawn(|| snapshot("other"));
        (same.join().unwrap(), other.join().unwrap())
    });
    assert_eq!(records, same);
    assert!(other.iter().all(|r| !r.3.contains("bridge:equipped")));
    assert!(
        other
            .iter()
            .any(|r| r.3.contains("if predicate other:equipped"))
    );
    assert_eq!(records, snapshot("bridge"));
}

#[test]
fn borrowed_identity_retains_definition_without_registering_or_evaluating_it() {
    let retained = Predicate::new(
        PredicateId::minecraft("unregistered").unwrap(),
        PredicateRoot::random_chance(2.0),
    );
    let borrowed = PredicateId::from(&retained);
    assert_eq!(
        Target::entities().predicate(&retained).to_string(),
        "@e[predicate=minecraft:unregistered]"
    );
    // An invalid tree still supplies an ID; validation belongs to registration.
    assert_eq!(borrowed, PredicateId::from(retained));
    assert!(!snapshot("bridge").iter().any(|r| r.2 == "unregistered"));
}
