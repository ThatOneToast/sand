use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = entity)]
struct EntityData {
    #[state(default = 0)]
    payload: Data<i32>,
}

#[function]
fn write_data() {
    EntityData::on(EntityContext::<AnyEntity>::default()).payload.set(7);
}

fn main() {
    let records = sand_core::try_export_components("demo").unwrap();
    let body = |path: &str| records.iter()
        .find(|record| record.dir == "function" && record.path == path)
        .unwrap().content.as_str();
    let call = body("write_data");
    let write = body(call.strip_prefix("function demo:").unwrap());
    assert_eq!(write.matches("set from entity @s UUID").count(), 1);
    assert!(!write.contains("UUID["));
    assert!(write.contains("execute unless entity @s run data remove storage demo:state __sand_owner.uuid"));
    assert!(write.contains(" with storage demo:state __sand_owner"));
    assert!(write.contains("data modify storage demo:state __sand_owner.value set value 7"));
    assert!(write.ends_with("data remove storage demo:state __sand_owner.value"));
    let state = EntityData::on(EntityContext::<AnyEntity>::default());
    let guarded = state.payload.if_present(|| vec!["say present".into()]);
    assert_eq!(guarded.len(), 3);
}
