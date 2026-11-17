mod support;
use support::*;

#[test]
fn retained_references_are_values_in_state_records_calls_and_events() {
    compile(&analyze_one(
        r#"#![pack(namespace = "refs", minecraft = "26.2")]
struct Ownership { owner: EntityRef }
entity state Projectile { ownership: Ownership }
global state Saved { owner: EntityRef }
event Hit(owner: EntityRef);
#[on(Hit)] fn announce(owner: EntityRef) { for player in resolve_player(owner) { chat!("Hit!"); } }
fn identity(owner: EntityRef) -> EntityRef { return owner; }
#[on(first_join)] fn capture() { Saved.owner = identity(entity_ref()); }
fn hit() {
    for mob in entities(type = Entity::Pig) {
        attach_state(Projectile);
        Projectile.ownership = Ownership { owner: Saved.owner };
        damage(2.5, by = Projectile.ownership.owner);
        emit Hit(Projectile.ownership.owner);
    }
    for owner in resolve_entity(Saved.owner) { add_tag("resolved"); }
}
"#,
    ));
}

#[test]
fn references_reject_numeric_conversions_and_wrong_execution_contexts() {
    for code in [
        "#[load] fn f() { let r = entity_ref(); }",
        "fn f(r: EntityRef) { let n = r + r; }",
        "fn f(r: EntityRef) { let n = r as number; }",
        "fn f() { for e in resolve_entity(1) {} }",
        "fn f(r: EntityRef) { for e in resolve_entity(r) { give_item(Item::Bread); } }",
    ] {
        let source = format!("#![pack(namespace = \"refs\")] {code}");
        assert!(analyze_one(&source).has_errors(), "accepted {code}");
    }
}

#[test]
fn borrowed_receiver_native_fixture_compiles() {
    compile(&analyze_one(include_str!("fixtures/borrows.sand")));
}
