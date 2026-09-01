use sand::prelude::*;

#[derive(State)]
#[state(namespace = "demo", scope = player)]
struct PlayerOnly;

#[derive(State)]
#[state(namespace = "demo", scope = living)]
struct LivingOnly;

#[derive(State)]
#[state(namespace = "demo", scope = global)]
struct GlobalOnly;

#[derive(StateBundle)]
struct PlayerBundle {
    state: PlayerOnly,
}

#[derive(StateBundle)]
struct LivingBundle {
    state: LivingOnly,
}

#[derive(StateBundle)]
struct GlobalBundle {
    state: GlobalOnly,
}

fn main() {
    let entity = EntityContext::<ZombieKind>::default();
    let non_living = EntityContext::<MarkerKind>::default();
    let _ = PlayerBundle::attach(entity);
    let _ = LivingBundle::on(non_living);
    let _ = GlobalBundle::on(entity);
}
