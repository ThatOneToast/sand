//! A separate sand-only consumer: offhand prism tools share a battery, while a
//! second battery powers an independent pulse. All mechanics use public APIs.
use sand::prelude::*;
use sand::systems::ability::{Ability, ResourceMaintenance, ResourcePool};

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "aether", scope = player)]
struct Battery {
    #[state(default = 48, min = 0, max = 48)]
    charge: Score,
    #[state(auto_tick)]
    recharge: Timer,
    #[state(auto_tick)]
    pulse: Cooldown,
    #[state(default = 21, min = 0, max = 21)]
    reserve: Score,
    #[state(auto_tick)]
    reserve_pulse: Cooldown,
    activations: Score,
}

fn prism() -> CustomItem {
    CustomItem::new(vanilla::Item::AmethystShard).custom_data("aether_prism")
}

fn pulse(player: EntityContext<PlayerKind>, cost: i32) -> Ability {
    let state = Battery::on(player);
    Ability::new(
        ResourcePool::new(state.charge, 48),
        state.pulse,
        cost,
        Ticks::new(12),
        player
            .inventory()
            .offhand()
            .matches(prism().item_predicate().unwrap())
            .unwrap(),
    )
    .unwrap()
}

#[datapack_component]
fn recharge() -> ResourceMaintenance {
    ResourceMaintenance::new(
        ResourceLocation::new("aether", "recharge").unwrap(),
        Target::players().each(|player| {
            let state = Battery::on(*player);
            ResourcePool::new(state.charge, 48)
                .regenerate(state.recharge, 3, Ticks::new(7))
                .unwrap()
        }),
    )
}

#[function("aether:grant", context = player)]
fn grant(player: EntityContext<PlayerKind>) {
    player.inventory().offhand().set(prism()).unwrap();
}

/// Accept only a diamond block directly below the caster.
#[function("aether:pulse", context = player)]
fn activate(player: EntityContext<PlayerKind>) {
    pulse(player, 7).accept(
        WorldContext::current()
            .block(BlockPos::below(1))
            .matches(vanilla::Block::DiamondBlock),
        Battery::on(player).activations.add(1),
    );
}

/// A second configuration shares the battery and maintenance, at a different cost.
#[function("aether:flare", context = player)]
fn flare(player: EntityContext<PlayerKind>) {
    pulse(player, 11).accept(Condition::all([]), Battery::on(player).activations.add(1));
}

/// Another instance binds independent fields; activation does not share cooldown.
#[function("aether:reserve", context = player)]
fn reserve(player: EntityContext<PlayerKind>) {
    let state = Battery::on(player);
    Ability::new(
        ResourcePool::new(state.reserve, 21),
        state.reserve_pulse,
        5,
        Ticks::new(9),
        Condition::all([]),
    )
    .unwrap()
    .accept(Condition::all([]), state.activations.add(1));
}

/// Native block replacement of an unloaded position fails after acceptance.
/// A following counter still runs; cost and cooldown are intentionally retained.
#[function("aether:failure", context = player)]
fn later_failure(player: EntityContext<PlayerKind>) {
    pulse(player, 7).accept(Condition::all([]), mcfunction![
        WorldContext::current().block(BlockPos::absolute(1000000, 80, 1000000)).replace(vanilla::Block::GoldBlock);
        Battery::on(player).activations.add(1);
    ]);
}

/// Early return from an effect callback does not undo accepted payment.
#[function("aether:early_return", context = player)]
fn early_return(player: EntityContext<PlayerKind>) {
    pulse(player, 7).accept(Condition::all([]), cmd::function(returning_effect));
}

#[function("aether:returning_effect", context = player)]
fn returning_effect(player: EntityContext<PlayerKind>) {
    cmd::return_fail();
    Battery::on(player).activations.add(1);
}

#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    match sand::advanced::try_export_pack_json(namespace, mc_version) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("sand export failed: {error}");
            std::process::exit(1);
        }
    }
}
