//! Sneak to cast a fireball at another player. All gameplay uses Sand definitions;
//! the framework owns selection, timers, costs, cooldowns, hit identity and cleanup.
use sand::prelude::*;
use sand::systems::{Ability, Lifetime, Projectile, StatusEffect};

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "owned_combat", scope = player)]
pub struct Mage {
    #[state(default = 100, min = 0, max = 100)]
    mana: Score,
    #[state(auto_tick)]
    fireball_cooldown: Cooldown,
}

/// Configure the projectile and its admission policy as one definition.
#[datapack_component]
pub fn fireball() -> Ability {
    Ability::projectile(
        Projectile::new(
            Lifetime::new("owned_combat:fireball".parse().unwrap(), Ticks::seconds(2))
                .origin(sand::command::Anchor::Eyes)
                .capacity(64)
                .requires::<Mage>(),
        )
        .speed(1.5)
        .radius(1.75)
        .targets(Target::players())
        .on_hit(|hit| hit.damage_target(6.0, sand::command::DamageKind::Magic)),
    )
    .cost(Mage::mana, 20)
    .cooldown(Mage::fireball_cooldown, Ticks::seconds(3))
}

/// A five-second effect that restores mana once per second. Reapplying refreshes
/// its duration; the lifetime handles disconnect, death and explicit removal.
#[datapack_component]
pub fn regeneration() -> StatusEffect {
    StatusEffect::new(
        Lifetime::new(
            "owned_combat:regeneration".parse().unwrap(),
            Ticks::seconds(5),
        )
        .every(Ticks::seconds(1))
        .requires::<Mage>()
        .on_tick(|effect| effect.owner(|player| player.state(Mage::mana).add(5))),
    )
}

#[on_event]
fn sneak_to_cast(_: Event<sand::events::PlayerStartSneakingEvent>) {
    fireball().activate();
}

// Lifecycle events invalidate even unloaded instances when a player dies or
// reconnects before their task's chunk gets another tick.
#[on_event]
fn clear_on_death(_: Event<sand::events::OnDeathEvent>) {
    fireball().cancel();
    regeneration().remove();
}

#[on_event]
fn clear_on_join(_: Event<sand::events::OnJoinEvent>) {
    fireball().cancel();
    regeneration().remove();
}

#[system(tick, every = 20)]
fn recover_mana(mages: Mage) {
    mages.each(|mage| mage.mana.add(1));
}

/// Player command entry point; events can call the same definition directly.
#[function("owned_combat:cast", context = player)]
pub fn cast() {
    fireball().activate();
}

#[function("owned_combat:regenerate", context = player)]
pub fn regenerate() {
    regeneration().apply();
}

/// Explicit removal is useful when leaving a match; death/disconnect are automatic.
#[function("owned_combat:cancel", context = player)]
pub fn cancel() {
    fireball().cancel();
    regeneration().remove();
}

#[function("owned_combat:diagnose", context = player)]
pub fn diagnose() {
    fireball().diagnose();
}

#[doc(hidden)]
pub fn __sand_export(namespace: &str, version: &str) {
    println!(
        "{}",
        sand::advanced::try_export_pack_json(namespace, version).expect("owned combat must export")
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_exports_deterministically_for_current_and_older_targets() {
        for version in ["26.1", "26.2", "26.3"] {
            let export = sand::advanced::try_export_pack_json("owned_combat", version).unwrap();
            assert_eq!(
                export,
                sand::advanced::try_export_pack_json("owned_combat", version).unwrap()
            );
            assert!(export.contains("projectile_search"));
            assert!(export.contains("Cooldown is not ready"));
        }
    }
}
