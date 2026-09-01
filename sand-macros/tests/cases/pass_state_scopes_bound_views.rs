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
    #[state(default = 1.255, min = -2, max = 2, scale = 100)]
    speed: FixedScore,
}

#[derive(State)]
#[state(namespace = "test", scope = entity)]
struct GenericState {
    value: EntityScore<i32>,
}

#[derive(State)]
#[state(namespace = "test", scope = living)]
struct LivingState {
    #[state(default = 0)]
    age: EntityTimer,
}

#[derive(State)]
#[state(namespace = "test", scope = global, name = "world")]
struct GlobalState {
    #[state(criterion = "playerKillCount", display_name = "World value")]
    value: EntityScore<i32>,
}

#[derive(StateBundle)]
struct PlayerBundle {
    state: PlayerState,
}

#[derive(StateBundle)]
struct LivingBundle {
    state: LivingState,
}

#[derive(StateBundle)]
struct GlobalStateBundle {
    state: GlobalState,
}

#[derive(StateQuery)]
#[query(scope = player)]
struct PlayersWithState {
    state: PlayerState,
}

fn main() {
    let player = PlayerState::on(EntityContext::<PlayerKind>::default());
    let _: Vec<String> = player.health.add(1);
    let _: Vec<String> = player.poisoned.enable();
    let _: Vec<String> = player.dash.start(Ticks::new(20));
    let _: Vec<String> = player.speed.add(0.25);
    assert_eq!(PlayerState::speed.scale(), 100);
    assert_eq!(PlayerState::speed.descriptor().default, 125);

    let entity = GenericState::on(EntityContext::<AnyEntity>::default());
    let _: Vec<String> = entity.value.set(1);

    let living = LivingState::on(EntityContext::<ZombieKind>::default());
    let _: Vec<String> = living.age.tick();

    let global = GlobalState::global();
    let _: Vec<String> = global.value.set(2);

    let player_context = EntityContext::<PlayerKind>::default();
    let _: PlayerBundleBound = PlayerBundle::on(player_context);
    let _: Vec<String> = PlayerBundle::attach(player_context);
    let _: Vec<String> = PlayerBundle::detach(player_context);
    let living_context = EntityContext::<ZombieKind>::default();
    let _: LivingBundleBound = LivingBundle::on(living_context);
    let _: Vec<String> = LivingBundle::attach(living_context);
    let _: Vec<String> = LivingBundle::detach(living_context);
    let _: GlobalStateBundleBound = GlobalStateBundle::global();
    let _: Vec<String> = GlobalStateBundle::attach_global();
    let _: Vec<String> = GlobalStateBundle::detach_global();

    let query = PlayersWithState::each(|item| item.state.health.add(1));
    assert!(query[0].starts_with("execute as @a["));
}
