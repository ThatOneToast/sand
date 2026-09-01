use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = entity)]
struct EntityData {
    #[state(default = 0)]
    payload: Data<i32>,
}

fn main() {
    let state = EntityData::on(EntityContext::<AnyEntity>::default());
    let commands = state.payload.set(7);
    assert!(commands.iter().any(|command| command.contains("UUID[0]")));
    assert!(commands.last().unwrap().contains(" with storage demo:state __sand_owner"));
    let guarded = state.payload.if_present(|| vec!["say present".into()]);
    assert_eq!(guarded.len(), 5);
}
