use sand::prelude::*;

#[derive(State)]
#[state(namespace = "direct", scope = entity)]
struct EntityHealth {
    #[state(default = 20)]
    health: Score,
}

#[derive(State)]
#[state(namespace = "direct", scope = living)]
struct LivingHealth {
    #[state(default = 20)]
    health: Score,
}

#[derive(State)]
#[state(namespace = "direct", scope = player)]
struct PlayerHealth {
    #[state(default = 20)]
    health: Score,
}

#[derive(State)]
#[state(namespace = "direct", scope = entity)]
struct Dead;

#[derive(StateBundle)]
struct EntityCombat {
    health: EntityHealth,
    dead: Dead,
}

#[derive(StateBundle)]
struct NestedEntityCombat {
    combat: EntityCombat,
}

#[system(tick, every = 10)]
fn free_tick(query: EntityHealth) {
    query.each(|health| health.health.add(1));
}

#[system(tick, every = 10)]
fn bundle_tick(query: NestedEntityCombat) {
    query.each(|nested| nested.combat.health.health.add(1));
}

struct DirectSystems;

#[system]
impl DirectSystems {
    #[tick(every = 10)]
    fn grouped_tick(query: LivingHealth) {
        query.each(|health| health.health.add(1));
    }

    #[event(DirectPulse)]
    fn current(_event: DirectPulse, query: PlayerHealth) {
        query.current(|health| health.health.add(1));
    }
}

struct DirectPulse;

impl SandEvent for DirectPulse {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

fn main() {
    let _: Vec<String> = <EntityCombat as sand::__private::StateQuerySpec>::each(|combat| {
        combat.health.health.add(1)
    });
    let _: Vec<String> =
        <NestedEntityCombat as sand::__private::StateQuerySpec>::current(|nested| {
            nested.combat.health.health.add(1)
    });
}

#[allow(dead_code)]
fn source_level_methods(
    entity: EntityHealth,
    living: LivingHealth,
    player: PlayerHealth,
    marker: Dead,
) {
    let _: Vec<String> = entity.each(|health| health.health.add(1));
    let _: Vec<String> = living.current(|health| health.health.add(1));
    let _: Vec<String> = player.each(|health| health.health.add(1));
    let _: Vec<String> = marker.each(|_dead| vec!["say dead".into()]);
}

#[allow(dead_code)]
fn source_level_bundle_methods(bundle: NestedEntityCombat) {
    let _: Vec<String> = bundle.each(|nested| nested.combat.health.health.add(1));
}
