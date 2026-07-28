use sand::prelude::*;

#[derive(State)]
#[state(namespace = "test", scope = player)]
struct PlayerState {
    #[state(default = 20, min = 0, max = 20)]
    health: EntityScore<i32>,
    #[state(default = false)]
    poisoned: EntityFlag,
    #[state(default = 0, auto_tick)]
    dash: EntityCooldown,
}

#[derive(State)]
#[state(namespace = "test", scope = entity)]
struct GenericState {
    value: EntityScore<i32>,
}

#[derive(State)]
#[state(namespace = "test", scope = living)]
struct LivingState {
    #[state(default = 0, auto_tick)]
    age: EntityTimer,
}

#[derive(State)]
#[state(namespace = "test", scope = global, name = "world")]
struct GlobalState {
    value: EntityScore<i32>,
}

fn main() {
    let player = PlayerState::on(EntityContext::<PlayerKind>::default());
    let _: Vec<String> = player.health.add(1);
    let _: Vec<String> = player.poisoned.enable();
    let _: Vec<String> = player.dash.start(Ticks::new(20));

    let entity = GenericState::on(EntityContext::<AnyEntity>::default());
    let _: Vec<String> = entity.value.set(1);

    let living = LivingState::on(EntityContext::<ZombieKind>::default());
    let _: Vec<String> = living.age.tick();

    let global = GlobalState::global();
    let _: Vec<String> = global.value.set(2);
}
