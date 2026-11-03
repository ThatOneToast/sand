//! A custom-health boss with a shield, two native player attacks, phase rules,
//! telegraphs and owned cleanup. Invoke `animated_boss:start` as its actor.
use sand::component::IntoCommands;
use sand::entity::HitVolume;
use sand::prelude::*;
use sand::systems::{Combat, Encounter, Job, Lifetime, Scene, Sequence, Transform};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
pub enum Phase {
    Idle = 0,
    Charging = 1,
    Strike = 2,
    Recovering = 3,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "animated_boss", scope = entity)]
pub struct Boss {
    #[state(default = "Phase::Idle")]
    phase: EntityEnum<Phase>,
    #[state(default = 100, min = 0, max = 100)]
    health: Score,
    #[state(default = 20, min = 0, max = 20)]
    shield: Score,
    #[state(auto_tick)]
    invulnerability: Cooldown,
    attack_turn: Score,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "animated_boss", scope = entity)]
pub struct Charge {
    percent: Score,
}

/// Native client interpolation needs no per-frame transformation commands.
#[datapack_component]
pub fn telegraph() -> Scene<AnyEntity> {
    let small = Transform::default().translated([-0.5, 2.5, -0.5]);
    let charged = Transform::default()
        .translated([-1.0, 2.0, -1.0])
        .scaled([2.0; 3])
        .yaw(90.0);
    Scene::new(
        Lifetime::new("animated_boss:telegraph".parse().unwrap(), Ticks::new(60))
            .capacity(16)
            .on_start(|ctx| ctx.owner(|owner| owner.state(Boss::phase).set(Phase::Charging))),
    )
    .part("core", BlockId::minecraft("amethyst_block").unwrap(), small)
    .part(
        "crown",
        BlockId::minecraft("gold_block").unwrap(),
        Transform::default()
            .translated([-0.25, 4.0, -0.25])
            .scaled([0.5; 3]),
    )
    .keyframe("core", Ticks::new(1), Ticks::new(39), charged)
    .keyframe("core", Ticks::new(40), Ticks::new(10), small)
    .follow_owner(true)
}

/// One indexed planning step per tick. The lifetime owns local progress State;
/// completion performs a bounded nearest-player query in the owner's context.
#[datapack_component]
pub fn attack() -> Job<AnyEntity> {
    Job::new(
        Lifetime::new("animated_boss:attack".parse().unwrap(), Ticks::new(60))
            .capacity(16)
            .requires::<Boss>()
            .owns::<Charge>()
            .on_start(|ctx| ctx.owner(|owner| owner.state(Boss::phase).set(Phase::Charging)))
            .on_stop(|ctx| ctx.owner(|owner| owner.state(Boss::phase).set(Phase::Idle))),
        40,
        |ctx, index| {
            ctx.instance().state(Charge::percent).set(StatCurve::lerp(
                0.0.into(),
                100.0.into(),
                StatCurve::linear(index, 1.0 / 40.0, 1.0 / 40.0),
            ))
        },
    )
    .per_tick(1)
    .on_complete(|ctx| {
        ctx.owner(|owner| {
            let mut actions = owner.state(Boss::phase).set(Phase::Strike).into_commands();
            actions.extend([EntityScope::bind(&owner, |source| {
                Target::players()
                    .within_blocks(6.0)
                    .sort(sand::command::SortOrder::Nearest)
                    .limit(1)
                    .unwrap()
                    .each_in(HitVolume::sphere(6.0).unwrap(), |player| {
                        player
                            .living()
                            .damage_by(4.0, sand::command::DamageKind::Magic, source)
                    })
            })]);
            actions
        })
    })
}

/// Recovery belongs to the child timeline, so the controller cannot overlap it.
#[datapack_component]
pub fn strike() -> Sequence<AnyEntity> {
    Sequence::new(
        Lifetime::new("animated_boss:strike".parse().unwrap(), Ticks::new(100))
            .capacity(16)
            .requires::<Boss>()
            .on_stop(|ctx| ctx.owner(|owner| owner.state(Boss::attack_turn).set(1))),
    )
    .then(start_children)
    .wait_until(|ctx| ctx.children_finished(), Ticks::new(65))
    .then(|ctx| ctx.owner(|owner| owner.state(Boss::phase).set(Phase::Recovering)))
    .wait(Ticks::new(20))
}

/// Below half health, alternate a bounded area attack with the normal strike.
#[datapack_component]
pub fn burst() -> Sequence<AnyEntity> {
    Sequence::new(
        Lifetime::new("animated_boss:burst".parse().unwrap(), Ticks::new(100))
            .capacity(16)
            .requires::<Boss>()
            .on_stop(|ctx| ctx.owner(|owner| owner.state(Boss::attack_turn).set(0))),
    )
    .then(|ctx| ctx.start_child(telegraph()))
    .wait(Ticks::new(40))
    .then(|ctx| {
        ctx.owner(|owner| {
            EntityScope::bind(&owner, |source| {
                Target::players()
                    .within_blocks(10.0)
                    .sort(sand::command::SortOrder::Nearest)
                    .take(8)
                    .unwrap()
                    .each_in(HitVolume::sphere(10.0).unwrap(), |player| {
                        player
                            .living()
                            .damage_by(3.0, sand::command::DamageKind::Magic, source)
                    })
            })
        })
    })
    .wait_until(|ctx| ctx.children_finished(), Ticks::new(25))
    .wait(Ticks::new(20))
}

/// An encounter remains active while alive and a player is within 24 blocks.
/// Priority rules read the same health State as combat and presentation.
#[datapack_component]
pub fn encounter() -> Encounter<AnyEntity> {
    Encounter::new(
        Lifetime::new(
            "animated_boss:encounter".parse().unwrap(),
            Ticks::seconds(300),
        )
        .capacity(16)
        .requires::<Boss>()
        .on_stop(|ctx| {
            ctx.owner(|owner| {
                let mut actions = owner.state(Boss::phase).set(Phase::Idle);
                actions.extend([owner.identity().remove_tag(&boss_tag()).unwrap()]);
                actions
            })
        }),
    )
    .while_active(|owner| {
        Condition::all([
            owner.state(Boss::health).matches(1..).unwrap(),
            Condition::entity(Target::players().within_blocks(24.0)),
        ])
    })
    .attack(
        |owner| {
            Condition::all([
                owner.state(Boss::health).matches(..=50).unwrap(),
                owner.state(Boss::attack_turn).matches(1..=1).unwrap(),
            ])
        },
        burst(),
    )
    .attack(|_| Condition::all([]), strike())
}

fn boss_tag() -> EntityTag {
    EntityTag::new("animated_boss.active").unwrap()
}

/// The boss uses custom State health, so a marker or display actor can fight.
/// Player victims retain native Minecraft health and native damage attribution.
fn defenses() -> Combat {
    Combat::new(Boss::health, 100)
        .shield(Boss::shield)
        .resistance(0.2)
        .invulnerability(Boss::invulnerability, Ticks::new(5))
        .on_defeat(encounter().cancel())
}

/// Call as a player from an item event or input binding. The nearest active boss
/// within six blocks receives a hit; other encounters and players are untouched.
#[function("animated_boss:hit", context = player)]
pub fn hit() {
    Target::entities()
        .tag("animated_boss.active")
        .within_blocks(6.0)
        .sort(sand::command::SortOrder::Nearest)
        .limit(1)
        .unwrap()
        .each(|_| defenses().damage(10));
}

fn start_children(ctx: sand::systems::TaskContext<AnyEntity>) -> Actions {
    let mut actions = ctx.owner(|owner| owner.state(Boss::phase).set(Phase::Idle));
    actions.extend([ctx.start_child(telegraph())]);
    actions.extend([
        when(ctx.owner_matches(|owner| owner.state(Boss::phase).is(Phase::Charging)))
            .then(ctx.start_child(attack())),
    ]);
    actions
}

/// Restart this owner's encounter; other entities remain independent.
#[function("animated_boss:start")]
pub fn start() {
    encounter().cancel();
    Boss::composition_attach("@s");
    Boss::health.bind().set(100);
    Boss::shield.bind().set(20);
    Boss::invulnerability.bind().start(Ticks::new(0));
    Boss::attack_turn.bind().set(0);
    encounter().start();
    EntityContext::<AnyEntity>::default()
        .identity()
        .add_tag(&boss_tag())
        .unwrap();
}

#[function("animated_boss:cancel")]
pub fn cancel() {
    encounter().cancel();
}

#[doc(hidden)]
pub fn __sand_export(namespace: &str, version: &str) {
    println!(
        "{}",
        sand::advanced::try_export_pack_json(namespace, version).expect("boss must export")
    );
}
#[cfg(test)]
mod tests {
    #[test]
    fn exports_for_all_supported_targets() {
        for version in ["26.1", "26.2", "26.3"] {
            let pack = sand::advanced::try_export_pack_json("animated_boss", version).unwrap();
            assert_eq!(
                pack,
                sand::advanced::try_export_pack_json("animated_boss", version).unwrap()
            );
            assert!(pack.contains("interpolation_duration:39"));
            assert!(pack.contains("limit=1"));
            assert!(pack.contains("job_complete"));
            assert!(pack.contains("encounter_select"));
            assert!(pack.contains("sequence_gate_0"));
            assert!(pack.contains("sand/combat/defeated"));
            assert!(pack.contains(" by @e[tag=__sand_scope_"));
            assert!(pack.contains("if entity @s[distance=..6]"));
        }
    }
}
