use sand::component::IntoCommands;
use sand::prelude::*;
use sand::systems::{Ability, Lifetime};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum Phase {
    Idle = 0,
    Active = 1,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "owned", scope = entity)]
struct Energy {
    #[state(default = 10)]
    mana: Score,
    #[state(auto_tick)]
    cooldown: Cooldown,
    #[state(default = "Phase::Idle")]
    phase: EntityEnum<Phase>,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "owned", scope = entity)]
struct Instance {
    count: Score,
}

#[datapack_component]
fn pulse() -> Ability<AnyEntity> {
    Ability::new(
        Lifetime::new("owned:pulse".parse().unwrap(), Ticks::new(4))
            .capacity(2)
            .owns::<Instance>()
            .on_start(|ctx| ctx.owner(|_| cmd::say("OWNED_START")))
            .on_tick(|ctx| ctx.owner(|_| cmd::say("OWNED_TICK")))
            .on_stop(|ctx| {
                let mut actions = ctx.owner(|_| cmd::say("OWNED_STOP"));
                actions.extend([ctx.stop()]); // Regression: nested cleanup runs once.
                actions
            }),
    )
    .cost(Energy::mana, 3)
    .cooldown(Energy::cooldown, Ticks::new(8))
}
#[function("owned:activate")]
fn activate() {
    pulse().activate();
}
#[function("owned:cancel")]
fn cancel() {
    pulse().cancel();
}
#[function("owned:attach")]
fn attach() {
    Energy::composition_attach("@s");
}

fn exported(version: &str) -> serde_json::Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json("owned", version).unwrap()).unwrap()
}
fn body<'a>(pack: &'a serde_json::Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}
#[test]
fn lifetime_uses_owner_identity_atomic_admission_and_idempotent_cleanup() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        assert_eq!(pack, exported(version));
        let start = body(&pack, "__sand_system/lifetime/pulse/start");
        assert!(start.contains("matches 3.."));
        assert!(start.contains("matches 0"));
        assert!(start.contains("/admit"));
        assert_eq!(
            body(&pack, "__sand_system/lifetime/pulse/tick")
                .matches("distance=0..")
                .count(),
            3
        );
        let admit = body(&pack, "__sand_system/lifetime/pulse/admit");
        assert!(admit.find("#live").unwrap() < admit.find("summon").unwrap());
        assert!(admit.contains("spawn.generation"));
        assert!(admit.contains("2147483645"));
        let init = body(&pack, "__sand_system/lifetime/pulse/init");
        assert!(init.find("/commit").unwrap() < init.find("/started").unwrap());
        let stop = body(&pack, "__sand_system/lifetime/pulse/stop_instance");
        assert!(stop.find(".stopping").unwrap() < stop.find("/stopped").unwrap());
        let stop = body(&pack, "__sand_system/lifetime/pulse/cleanup");
        assert!(stop.contains("scoreboard players reset"), "{stop}");
        assert!(stop.find("tag @s remove").unwrap() < stop.find("kill @s").unwrap());
        assert!(stop.ends_with("kill @s\n") || stop.ends_with("kill @s"));
        let validate = body(&pack, "__sand_system/lifetime/pulse/validate");
        assert!(validate.contains("$(owner)"));
        assert!(validate.contains("$(generation)"));
        assert!(validate.contains("Health:0.0f"));
        assert!(
            body(&pack, "__sand_system/lifetime/pulse/step")
                .contains("/validate with entity @s data")
        );
        if version == std::env::var("SAND_GAMEPLAY_PROOF_VERSION").unwrap_or_else(|_| "26.3".into())
            && let Some(root) = std::env::var_os("SAND_GAMEPLAY_PROOF_PACK")
        {
            let root = std::path::PathBuf::from(root);
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join("fixture.json"), serde_json::to_string_pretty(&serde_json::json!({"version":version,"mana":Energy::mana.objective(),"cooldown":Energy::cooldown.objective(),"phase":Energy::phase.objective(), "combat_fractional_health":FractionalFighter::health.objective(), "combat_fractional_shield":FractionalFighter::shield.objective(), "player_combat_health":PlayerFighter::health.objective(), "combat_health":Fighter::health.objective(), "combat_shield":Fighter::shield.objective(), "combat_power":Fighter::power.objective(), "combat_immunity":Fighter::invulnerability.objective(), "combat_hits":Fighter::hits.objective(), "combat_defeats":Fighter::defeats.objective(), "combat_heals":Fighter::heals.objective()})).unwrap()).unwrap();
            std::fs::write(root.join("pack.mcmeta"),serde_json::to_string(&serde_json::json!({"pack":{"description":"Sand owned gameplay proof","min_format": if version=="26.3" {121} else if version=="26.2" {107} else {101},"max_format":if version=="26.3" {121} else if version=="26.2" {107} else {101}}})).unwrap()).unwrap();
            for c in pack["components"].as_array().unwrap() {
                let directory = c["dir"].as_str().expect("component directory");
                let extension = if directory == "function" {
                    "mcfunction"
                } else {
                    "json"
                };
                let path = root
                    .join("data")
                    .join(c["namespace"].as_str().unwrap())
                    .join(directory)
                    .join(format!("{}.{}", c["path"].as_str().unwrap(), extension));
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let content = &c["content"];
                std::fs::write(
                    path,
                    content
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| serde_json::to_string(content).unwrap()),
                )
                .unwrap();
            }
        }
    }
}

#[datapack_component]
fn regeneration() -> sand::systems::StatusEffect<AnyEntity> {
    sand::systems::StatusEffect::new(
        Lifetime::new("owned:regen".parse().unwrap(), Ticks::new(4))
            .requires::<Energy>()
            .on_start(|ctx| ctx.owner(|_| cmd::say("STATUS_START")))
            .on_stop(|ctx| ctx.owner(|_| cmd::say("STATUS_STOP"))),
    )
}
#[datapack_component]
fn stacking() -> sand::systems::StatusEffect<AnyEntity> {
    sand::systems::StatusEffect::new(
        Lifetime::new("owned:stack".parse().unwrap(), Ticks::new(4))
            .on_start(|ctx| ctx.owner(|_| cmd::say("STACK_START")))
            .on_stop(|ctx| ctx.owner(|_| cmd::say("STACK_STOP"))),
    )
    .stacking(2)
}
#[datapack_component]
fn fireball() -> Ability<AnyEntity> {
    Ability::projectile(
        sand::systems::Projectile::new(
            Lifetime::new("owned:fireball".parse().unwrap(), Ticks::new(20)).capacity(1),
        )
        .speed(1.0)
        .targets(Target::entities().tag("hit_target"))
        .on_hit(|ctx| {
            let mut actions = ctx.target(|_| cmd::say("PROJECTILE_TARGET"));
            actions.extend([ctx.owner(|_| cmd::say("PROJECTILE_OWNER"))]);
            actions.extend([ctx.damage_target(2.0, sand::command::DamageKind::Magic)]);
            actions
        }),
    )
    .cost(Energy::mana, 2)
}
#[function("owned:regenerate")]
fn regenerate() {
    regeneration().apply();
}
#[function("owned:stack")]
fn stack() {
    stacking().apply();
}
#[function("owned:fireball")]
fn cast() {
    fireball().activate();
}
#[function("owned:detach")]
fn detach() {
    Energy::composition_detach("@s");
}
#[function("owned:numeric_step")]
fn numeric_step() {
    Energy::mana.bind().set(StatCurve::add([
        StatCurve::state(Energy::mana),
        StatCurve::constant(1.0),
    ]));
}

#[datapack_component]
fn scenario() -> sand::systems::GameplayTest {
    let actor = || Target::entities().tag("owner_a");
    let mana_is = |value| {
        Condition::entity(
            actor()
                .state(Energy::mana.matches(value..=value).unwrap())
                .unwrap(),
        )
    };
    let mut cleanup = actor().each(|entity| entity.state(Energy::mana).set(10));
    cleanup.extend([cmd::say("SCENARIO_CLEANUP")]);
    sand::systems::GameplayTest::new("owned:scenario".parse().unwrap())
        .setup(cmd::say("SCENARIO_SETUP"))
        .setup(actor().each(|entity| entity.state(Energy::mana).set(42)))
        .expect(mana_is(42), "setup initializes canonical State")
        .after(
            Ticks::new(2),
            actor().each(|entity| entity.state(Energy::mana).add(1)),
        )
        .expect(mana_is(43), "delayed gameplay changes canonical State")
        .cleanup(cleanup)
}

#[function("owned:test")]
fn run_scenario() {
    scenario().run();
}

#[test]
fn profiles_keep_compute_isolated_and_projectiles_bound_their_work() {
    let old = exported("26.2");
    let new = exported("26.3");
    let text = |pack: &serde_json::Value| {
        pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["content"].as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(!text(&old).contains("run compute "));
    assert!(text(&new).contains("run compute entity @s integer"));
    assert_eq!(old, exported("26.2"));
    let movement = body(&new, "__sand_system/lifetime/fireball/projectile_move");
    assert_eq!(movement.matches("run tp @s").count(), 4);
    let search = body(&new, "__sand_system/lifetime/fireball/projectile_search");
    assert!(search.contains("limit=16"));
    assert!(search.contains("matches $(owner)"));
    assert!(body(&new, "__sand_system/lifetime/regen/status_admit").contains("/status_refresh"));
    assert!(
        body(&new, "__sand_system/lifetime/stack/status_admit")
            .contains("matches 2.. run return 0")
    );
    assert!(body(&new, "__sand_test/scenario/phase_1").contains("SAND_TEST_PASS owned:scenario"));
}

#[function("owned:transition")]
fn transition() {
    Energy::phase.bind().transition(
        Phase::Active,
        cmd::say("PHASE_EXIT"),
        cmd::say("PHASE_ENTER"),
    );
    Energy::phase.bind().transition(
        Phase::Active,
        cmd::say("PHASE_EXIT"),
        cmd::say("PHASE_ENTER"),
    );
}
#[function("owned:transition_nested")]
fn transition_nested() {
    Energy::phase.bind().transition(
        Phase::Active,
        Energy::phase.bind().transition(
            Phase::Idle,
            cmd::say("NESTED_EXIT"),
            cmd::say("NESTED_ENTER"),
        ),
        cmd::say("SUPERSEDED_ENTER"),
    );
}

#[test]
fn squared_distance_uses_the_canonical_numeric_model() {
    let curve = StatCurve::distance_squared(
        [0.0.into(), 0.0.into(), 0.0.into()],
        [3.0.into(), 4.0.into(), 0.0.into()],
    );
    let result = curve
        .evaluate(
            &sand::entity::CurveInputs::new(),
            FixedPoint::default(),
            "owned:test",
            "distance",
        )
        .unwrap();
    assert_eq!(result.as_f64(FixedPoint::default()), 25.0);
}

#[datapack_component]
fn player_task() -> Lifetime<PlayerKind> {
    Lifetime::new("owned:player_task".parse().unwrap(), Ticks::new(2000))
        .capacity(2)
        .requires::<Energy>()
        .on_start(|ctx| ctx.owner(|_| cmd::say("PLAYER_TASK_START")))
        .on_stop(|ctx| ctx.owner(|_| cmd::say("PLAYER_TASK_STOP")))
}
#[function("owned:player_start")]
fn player_start() {
    player_task().start();
}
#[function("owned:player_cancel")]
fn player_cancel() {
    player_task().cancel();
}

#[datapack_component]
fn self_stopping() -> Lifetime<AnyEntity> {
    Lifetime::new("owned:self_stopping".parse().unwrap(), Ticks::new(20)).on_tick(|ctx| ctx.stop())
}
#[function("owned:self_stopping")]
fn start_self_stopping() {
    self_stopping().start();
}

#[test]
fn stopping_from_tick_cannot_recreate_scores_after_cleanup() {
    let pack = exported("26.3");
    let step = body(&pack, "__sand_system/lifetime/self_stopping/step");
    assert!(step.find("/ticked").unwrap() < step.find("/resume_tick with entity @s data").unwrap());
    assert!(!step.contains("scoreboard players set"));
    let resume = body(&pack, "__sand_system/lifetime/self_stopping/resume_tick");
    assert!(resume.contains("=$(instance)"));
    let stop = body(&pack, "__sand_system/lifetime/self_stopping/cleanup");
    assert!(stop.find("tag @s remove").unwrap() < stop.find("scoreboard players reset").unwrap());
}

#[datapack_component]
fn budgeted() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new("owned:budgeted".parse().unwrap(), Ticks::new(100))
            .owns::<Instance>()
            .on_start(|_| cmd::say("JOB_START"))
            .on_stop(|_| cmd::return_cmd(0)),
        5,
        |ctx, index| {
            let mut actions = ctx.instance().state(Instance::count).set(index);
            actions.extend([cmd::say("JOB_STEP")]);
            actions
        },
    )
    .per_tick(2)
    .on_complete(|ctx| {
        let mut actions = cmd::say("JOB_COMPLETE").into_commands();
        actions.extend([ctx.stop()]);
        actions
    })
}
#[function("owned:job_start")]
fn job_start() {
    budgeted().start();
}
#[function("owned:job_cancel")]
fn job_cancel() {
    budgeted().cancel();
}

#[datapack_component]
fn stopped_job() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new("owned:stopped_job".parse().unwrap(), Ticks::new(100)),
        5,
        |ctx, _| ctx.stop(),
    )
    .per_tick(4)
    .on_complete(|_| cmd::say("UNEXPECTED_COMPLETE"))
}
#[function("owned:job_stop_inside")]
fn job_stop_inside() {
    stopped_job().start();
}

#[datapack_component]
fn animated() -> sand::systems::Scene<AnyEntity> {
    use sand::systems::{Scene, Transform};
    Scene::new(
        Lifetime::new("owned:animated".parse().unwrap(), Ticks::new(100))
            .on_start(|_| cmd::say("SCENE_START"))
            .on_stop(|_| cmd::return_cmd(0)),
    )
    .part(
        "body",
        BlockId::minecraft("stone").unwrap(),
        Transform::default(),
    )
    .part(
        "head",
        BlockId::minecraft("gold_block").unwrap(),
        Transform::default().translated([0.0, 2.0, 0.0]),
    )
    .keyframe(
        "head",
        Ticks::new(1),
        Ticks::new(4),
        Transform::default().translated([0.0, 3.0, 0.0]).yaw(90.0),
    )
    .follow_owner(true)
}
#[function("owned:scene_start")]
fn scene_start() {
    animated().start();
}
#[function("owned:scene_cancel")]
fn scene_cancel() {
    animated().cancel();
}

#[test]
fn spatial_operations_use_checked_numeric_semantics() {
    let vector = |x: f64, y: f64, z: f64| [x.into(), y.into(), z.into()];
    let eval = |curve: StatCurve| {
        curve
            .evaluate(
                &sand::entity::CurveInputs::new(),
                FixedPoint::default(),
                "owned:test",
                "math",
            )
            .unwrap()
            .as_f64(FixedPoint::default())
    };
    assert_eq!(
        eval(StatCurve::dot(
            vector(1.0, 2.0, 3.0),
            vector(-1.0, 2.0, 0.0)
        )),
        3.0
    );
    assert_eq!(
        StatCurve::cross(vector(1.0, 0.0, 0.0), vector(0.0, 1.0, 0.0)).map(eval),
        [0.0, 0.0, 1.0]
    );
    assert_eq!(
        StatCurve::cross(vector(0.0, 1.0, 0.0), vector(1.0, 0.0, 0.0)).map(eval),
        [0.0, 0.0, -1.0]
    );
    assert_eq!(
        eval(StatCurve::lerp(10.0.into(), 20.0.into(), 0.25.into())),
        12.5
    );
    assert_eq!(
        eval(StatCurve::lerp(10.0.into(), 20.0.into(), 2.0.into())),
        30.0
    );
}

#[test]
fn jobs_and_scenes_export_portably_with_instance_scoped_cleanup() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        assert_eq!(pack, exported(version));
        let batch = body(&pack, "__sand_system/lifetime/budgeted/job_batch");
        assert_eq!(batch.matches("/job_dispatch").count(), 2);
        let step = body(&pack, "__sand_system/lifetime/budgeted/job_step");
        assert!(step.find("/job_body").unwrap() < step.find("/job_resume").unwrap());
        assert!(!step.contains("scoreboard players add"));
        let resume = body(&pack, "__sand_system/lifetime/budgeted/job_resume");
        assert!(resume.contains("=$(instance)"));
        let advance = body(&pack, "__sand_system/lifetime/budgeted/job_advance");
        assert!(
            advance.find("scoreboard players add").unwrap()
                < advance.find("/job_complete").unwrap()
        );
        let node = body(&pack, "__sand_system/lifetime/animated/scene_part_1");
        assert!(node.contains(if version == "26.3" {
            "block_state:{id:"
        } else {
            "block_state:{Name:"
        }));
        let spawn = body(&pack, "__sand_system/lifetime/animated/scene_spawn");
        assert_eq!(
            spawn
                .matches("execute summon minecraft:block_display")
                .count(),
            2
        );
        assert!(spawn.contains("{instance:$(instance)}"));
        let cleanup = body(&pack, "__sand_system/lifetime/animated/scene_remove");
        assert!(cleanup.contains("=$(instance)"));
        let update = body(&pack, "__sand_system/lifetime/animated/scene_update");
        assert!(update.contains("interpolation_duration:4"));
        assert!(update.contains("start_interpolation:0"));
    }
}

#[datapack_component]
fn one_tick_job() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new("owned:one_tick_job".parse().unwrap(), Ticks::new(1)),
        1,
        |_, _| cmd::say("ONE_TICK_STEP"),
    )
    .on_complete(|_| cmd::say("ONE_TICK_COMPLETE"))
}
#[function("owned:one_tick_job")]
fn start_one_tick_job() {
    one_tick_job().start();
}

#[test]
fn extensions_share_validation_and_run_before_lifetime_expiry() {
    let pack = exported("26.3");
    let step = body(&pack, "__sand_system/lifetime/one_tick_job/step");
    assert!(step.find("/validate").unwrap() < step.find("/resume_tick").unwrap());
    let after = body(&pack, "__sand_system/lifetime/one_tick_job/after_tick");
    assert!(after.find("/job_batch").unwrap() < after.find("/resume_step").unwrap());
    assert!(
        body(&pack, "__sand_system/lifetime/one_tick_job/finish_step")
            .contains("scoreboard players remove")
    );
    assert!(
        !pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["path"].as_str().unwrap_or("").ends_with("/job_tick"))
    );
}

#[datapack_component]
fn crossing_job() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new("owned:crossing_job".parse().unwrap(), Ticks::new(20)),
        2,
        |ctx, _| {
            // Harness-only motion arranges a dimension crossing during one tick.
            let mut actions =
                ctx.owner(|_| "execute in minecraft:overworld run tp @s 10.5 200 2.5");
            actions
                .extend(["execute in minecraft:overworld run tp @s 10.5 200 2.5".into_commands()]);
            actions.extend([cmd::say("CROSSING_STEP")]);
            actions
        },
    )
    .on_complete(|_| cmd::say("CROSSING_COMPLETE"))
}
#[function("owned:crossing_job")]
fn start_crossing_job() {
    crossing_job().start();
}

#[test]
fn dimension_scans_share_a_once_per_game_tick_guard() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        assert!(
            body(&pack, "__sand_system/lifetime/crossing_job/tick")
                .starts_with("execute store result score #now ")
        );
        assert!(
            body(&pack, "__sand_system/lifetime/crossing_job/tick")
                .contains("run time query gametime")
        );
        let step = body(&pack, "__sand_system/lifetime/crossing_job/step");
        assert!(step.find("= #now").unwrap() < step.find("/resume_tick").unwrap());
    }
}

// Raw commands here arrange vanilla entity-copy boundaries for runtime proofs.
fn transfer(ctx: &sand::systems::TaskContext<AnyEntity>, dimension: &'static str) -> Actions {
    let command = format!("execute in {dimension} run tp @s 10.5 200 2.5");
    let mut actions = ctx.owner(|_| command.clone());
    actions.extend([command.into_commands()]);
    actions
}
#[datapack_component]
fn crossing_expiry() -> Lifetime<AnyEntity> {
    Lifetime::new("owned:crossing_expiry".parse().unwrap(), Ticks::new(1))
        .on_tick(|ctx| transfer(&ctx, "minecraft:overworld"))
        .on_stop(|ctx| {
            let mut actions = transfer(&ctx, "minecraft:the_nether");
            actions.extend([cmd::say("CROSS_EXPIRY_STOP")]);
            actions
        })
}
#[function("owned:crossing_expiry")]
fn start_crossing_expiry() {
    crossing_expiry().start();
}
#[datapack_component]
fn crossing_stopped_job() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new(
            "owned:crossing_stopped_job".parse().unwrap(),
            Ticks::new(20),
        )
        .on_stop(|ctx| {
            let mut actions = transfer(&ctx, "minecraft:the_nether");
            actions.extend([cmd::say("CROSS_JOB_STOP")]);
            actions
        }),
        5,
        |ctx, _| {
            let mut actions = transfer(&ctx, "minecraft:overworld");
            actions.extend([ctx.stop()]);
            actions
        },
    )
    .per_tick(4)
    .on_complete(|_| cmd::say("UNEXPECTED_COMPLETE"))
}
#[function("owned:crossing_stopped_job")]
fn start_crossing_stopped_job() {
    crossing_stopped_job().start();
}
#[datapack_component]
fn crossing_scene() -> sand::systems::Scene<AnyEntity> {
    sand::systems::Scene::new(
        Lifetime::new("owned:crossing_scene".parse().unwrap(), Ticks::new(1))
            .on_tick(|ctx| transfer(&ctx, "minecraft:overworld"))
            .on_stop(|ctx| transfer(&ctx, "minecraft:the_nether")),
    )
    .part(
        "body",
        BlockId::minecraft("gold_block").unwrap(),
        sand::systems::Transform::default(),
    )
}
#[function("owned:crossing_scene")]
fn start_crossing_scene() {
    crossing_scene().start();
}

#[datapack_component]
fn timeline() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(
        Lifetime::new("owned:timeline".parse().unwrap(), Ticks::new(10))
            .every(Ticks::new(4))
            .on_stop(|_| cmd::say("SEQUENCE_STOP")),
    )
    .then(|_| vec!["say SEQUENCE_FIRST", "return 0"])
    .then(|_| cmd::say("SEQUENCE_SECOND"))
    .wait(Ticks::new(2))
    .then(|_| cmd::say("SEQUENCE_THIRD"))
    .wait(Ticks::new(1))
    .on_complete(|_| cmd::say("SEQUENCE_COMPLETE"))
}
#[function("owned:timeline")]
fn start_timeline() {
    timeline().start();
}
#[function("owned:timeline_cancel")]
fn cancel_timeline() {
    timeline().cancel();
}

#[datapack_component]
fn stopped_timeline() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(Lifetime::new(
        "owned:stopped_timeline".parse().unwrap(),
        Ticks::new(10),
    ))
    .then(|ctx| ctx.stop())
    .then(|_| cmd::say("UNEXPECTED_SEQUENCE_ACTION"))
    .on_complete(|_| cmd::say("UNEXPECTED_COMPLETE"))
}
#[function("owned:stopped_timeline")]
fn start_stopped_timeline() {
    stopped_timeline().start();
}
#[datapack_component]
fn crossing_timeline() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(Lifetime::new(
        "owned:crossing_timeline".parse().unwrap(),
        Ticks::new(4),
    ))
    .then(|ctx| transfer(&ctx, "minecraft:overworld"))
    .then(|_| cmd::say("SEQUENCE_CROSSED"))
    .wait(Ticks::new(1))
    .then(|ctx| transfer(&ctx, "minecraft:the_nether"))
    .on_complete(|_| cmd::say("SEQUENCE_CROSS_COMPLETE"))
}
#[function("owned:crossing_timeline")]
fn start_crossing_timeline() {
    crossing_timeline().start();
}
#[datapack_component]
fn expired_timeline() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(Lifetime::new(
        "owned:expired_timeline".parse().unwrap(),
        Ticks::new(1),
    ))
    .then(|_| cmd::say("SEQUENCE_EXPIRING"))
    .wait(Ticks::new(1))
    .then(|_| cmd::say("UNEXPECTED_SEQUENCE_ACTION"))
    .on_complete(|_| cmd::say("UNEXPECTED_COMPLETE"))
}
#[function("owned:expired_timeline")]
fn start_expired_timeline() {
    expired_timeline().start();
}

#[function("owned:sphere_hits")]
fn sphere_hits() {
    Target::entities()
        .tag("hit_candidate")
        .sort(sand::command::SortOrder::Nearest)
        .each_in(sand::entity::HitVolume::sphere(2.0).unwrap(), |hit| {
            hit.identity()
                .add_tag(&EntityTag::new("sphere_hit").unwrap())
                .unwrap()
        });
}
#[function("owned:box_hits")]
fn box_hits() {
    Target::entities().tag("hit_candidate").each_in(
        sand::entity::HitVolume::cuboid([2.0, 2.0, 2.0])
            .unwrap()
            .translated([0.0, 1.0, 0.0])
            .unwrap(),
        |hit| {
            hit.identity()
                .add_tag(&EntityTag::new("box_hit").unwrap())
                .unwrap()
        },
    );
}
#[function("owned:nested_hits")]
fn nested_hits() {
    Target::entities().tag("hit_candidate").each_in(
        sand::entity::HitVolume::sphere(2.0).unwrap(),
        |hit| {
            let mut commands = Target::entities().tag("hit_candidate").each_in(
                sand::entity::HitVolume::sphere(0.01).unwrap(),
                |nested| {
                    nested
                        .identity()
                        .add_tag(&EntityTag::new("nested_hit").unwrap())
                        .unwrap()
                },
            );
            commands.extend([hit
                .identity()
                .add_tag(&EntityTag::new("outer_hit").unwrap())
                .unwrap()]);
            commands
        },
    );
}
#[function("owned:budgeted_hits")]
fn budgeted_hits() {
    Target::entities()
        .tag("hit_candidate")
        .sort(sand::command::SortOrder::Nearest)
        .limit(1)
        .unwrap()
        .each_in(
            sand::entity::HitVolume::sphere(0.5)
                .unwrap()
                .translated([2.0, 0.0, 0.0])
                .unwrap(),
            |hit| {
                hit.identity()
                    .add_tag(&EntityTag::new("budget_hit").unwrap())
                    .unwrap()
            },
        );
}
#[test]
fn timeline_and_hit_queries_export_portably_without_parallel_runtime_state() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        let tick = body(&pack, "__sand_system/lifetime/timeline/sequence_tick");
        assert!(tick.contains(
            "matches 0 run function owned:__sand_system/lifetime/timeline/sequence_dispatch_0"
        ));
        assert!(tick.contains(
            "matches 2 run function owned:__sand_system/lifetime/timeline/sequence_dispatch_2"
        ));
        assert!(tick.find("dispatch_0").unwrap() < tick.find("dispatch_1").unwrap());
        let dispatch = body(&pack, "__sand_system/lifetime/timeline/sequence_dispatch_1");
        assert!(dispatch.contains("$(instance)"));
        assert!(dispatch.contains("sequence_action_1"));
        assert!(body(&pack, "__sand_system/lifetime/timeline/job_step").contains("matches 0..3"));
        let sphere = body(&pack, "sphere_hits");
        assert!(
            sphere.contains("if entity @s[distance=..2] at @s"),
            "{sphere}"
        );
        assert!(!sphere.contains("summon"));
        let cuboid = body(&pack, "box_hits");
        assert!(
            cuboid.contains("positioned ~-1 ~ ~-1 if entity @s[dx=1,dy=1,dz=1] at @s"),
            "{cuboid}"
        );
    }
}

#[datapack_component]
fn owned_child() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new("owned:owned_child".parse().unwrap(), Ticks::new(20))
            .on_start(|ctx| {
                let mut actions = ctx.owner(|_| cmd::say("CHILD_STARTED"));
                actions.extend([ctx.start_child(grandchild())]);
                actions
            })
            .on_stop(|_| cmd::say("CHILD_STOPPED")),
        15,
        |ctx, _| ctx.owner(|_| cmd::say("CHILD_STEP")),
    )
}
#[datapack_component]
fn parent_a() -> Lifetime<AnyEntity> {
    Lifetime::new("owned:parent_a".parse().unwrap(), Ticks::new(20))
        .on_start(|ctx| ctx.start_child(owned_child()))
        .on_stop(|ctx| ctx.start_child(owned_child())) // Stopping parents cannot spawn children.
}
#[datapack_component]
fn parent_b() -> Lifetime<AnyEntity> {
    Lifetime::new("owned:parent_b".parse().unwrap(), Ticks::new(20))
        .on_start(|ctx| ctx.start_child(owned_child()))
}
#[function("owned:parents_start")]
fn parents_start() {
    parent_a().start();
    parent_b().start();
}
#[function("owned:parent_a_start")]
fn parent_a_start() {
    parent_a().start();
}
#[function("owned:parent_a_cancel")]
fn parent_a_cancel() {
    parent_a().cancel();
}
#[function("owned:parent_b_cancel")]
fn parent_b_cancel() {
    parent_b().cancel();
}
#[test]
fn child_tasks_capture_ancestors_and_invalidate_unloaded_parent_cancellation() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        let admit = body(&pack, "__sand_system/lifetime/owned_child/admit_child");
        assert!(admit.contains("spawn.ancestors append value"));
        assert!(admit.contains("init_child"));
        assert!(
            body(&pack, "__sand_system/lifetime/parent_a/cancel_owner")
                .contains("active[{owner:$(owner),generation:$(generation)}]")
        );
        assert!(body(&pack, "__sand_system/lifetime/parent_a/stop_instance").contains("/retire"));
        assert!(
            body(&pack, "__sand_system/lifetime/parent_a/cancel_descendants")
                .contains("ancestors:[{ledger:")
        );
    }
}

#[datapack_component]
fn conditional_timeline() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(
        Lifetime::new(
            "owned:conditional_timeline".parse().unwrap(),
            Ticks::new(20),
        )
        .on_stop(|_| cmd::say("GATE_STOP")),
    )
    .then(|_| cmd::say("GATE_START"))
    .wait_until(
        |ctx| ctx.owner_matches(|owner| owner.state(Energy::mana).matches(9..).unwrap()),
        Ticks::new(3),
    )
    .then(|_| cmd::say("GATE_READY"))
    .wait(Ticks::new(1))
    .on_complete(|_| cmd::say("GATE_COMPLETE"))
}
#[function("owned:conditional_timeline")]
fn start_conditional_timeline() {
    conditional_timeline().start();
}
#[function("owned:first_hit")]
fn first_hit() {
    Target::entities()
        .tag("hit_candidate")
        .sort(sand::command::SortOrder::Nearest)
        .take(16)
        .unwrap()
        .first_in(
            sand::entity::HitVolume::sphere(0.5)
                .unwrap()
                .translated([2.0, 0.0, 0.0])
                .unwrap(),
            |hit| {
                vec![
                    hit.identity()
                        .add_tag(&EntityTag::new("first_hit").unwrap())
                        .unwrap(),
                    "return 0".to_owned(),
                ]
            },
        );
    cmd::say("AFTER_FIRST_HIT");
}

#[datapack_component]
fn grandchild() -> Lifetime<AnyEntity> {
    Lifetime::new("owned:grandchild".parse().unwrap(), Ticks::new(40))
        .on_stop(|_| cmd::say("GRANDCHILD_STOP"))
}
#[datapack_component]
fn joined_child() -> sand::systems::Job<AnyEntity> {
    sand::systems::Job::new(
        Lifetime::new("owned:joined_child".parse().unwrap(), Ticks::new(10)),
        2,
        |_, _| Actions::default(),
    )
    .on_complete(|_| cmd::say("JOIN_CHILD_DONE"))
}
#[datapack_component]
fn join_parent() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(Lifetime::new(
        "owned:join_parent".parse().unwrap(),
        Ticks::new(20),
    ))
    .then(|ctx| {
        let mut actions = ctx.start_child(joined_child());
        actions.extend([ctx.start_child(joined_child())]);
        actions
    })
    .wait_until(|ctx| ctx.children_finished(), Ticks::new(10))
    .then(|_| cmd::say("JOIN_PARENT_DONE"))
}
#[function("owned:join_parent")]
fn start_join_parent() {
    join_parent().start();
}

#[datapack_component]
fn narrow_projectile() -> sand::systems::Projectile<AnyEntity> {
    sand::systems::Projectile::new(Lifetime::new(
        "owned:narrow_projectile".parse().unwrap(),
        Ticks::new(4),
    ))
    .speed(1.0)
    .radius(0.01)
    .targets(Target::entities().tag("thin_hit"))
    .on_hit(|ctx| ctx.target(|_| cmd::say("NARROW_HIT")))
}
#[function("owned:narrow_projectile")]
fn launch_narrow_projectile() {
    narrow_projectile().launch();
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "owned", scope = entity)]
struct Fighter {
    #[state(default = 20, min = 0, max = 100)]
    health: Score,
    #[state(default = 5, min = 0, max = 100)]
    shield: Score,
    #[state(default = 8)]
    power: Score,
    #[state(auto_tick)]
    invulnerability: Cooldown,
    hits: Score,
    defeats: Score,
    heals: Score,
}
fn combat() -> sand::systems::Combat {
    sand::systems::Combat::new(Fighter::health, 20)
        .shield(Fighter::shield)
        .resistance(0.25)
        .invulnerability(Fighter::invulnerability, Ticks::new(3))
        .on_hit(Fighter::hits.bind().add(1))
        .on_defeat(Fighter::defeats.bind().add(1))
        .on_heal(Fighter::heals.bind().add(1))
}
#[function("owned:combat_attach")]
fn combat_attach() {
    Fighter::composition_attach("@s");
}
#[function("owned:combat_hit")]
fn combat_hit() {
    combat().damage(Fighter::power.bind());
}
#[function("owned:combat_critical")]
fn combat_critical() {
    // Overlapping alternatives must select one critical multiplier, not two.
    combat()
        .critical(
            Condition::any([Condition::all([]), Condition::all([])]),
            2.0,
        )
        .damage(8);
}
#[function("owned:combat_heal")]
fn combat_heal() {
    combat().heal(8);
}
#[function("owned:combat_missing")]
fn combat_missing() {
    combat().damage(StatCurve::ratio(8.0.into(), 0.0.into()));
}
#[function("owned:combat_nested")]
fn combat_nested() {
    sand::systems::Combat::new(Fighter::health, 20)
        .on_hit(combat().damage(8))
        .damage(2);
}
#[function("owned:combat_defeat")]
fn combat_defeat() {
    combat().damage(100);
}
#[function("owned:native_dynamic")]
fn native_dynamic() {
    EntityContext::<sand::entity::ZombieKind>::default()
        .living()
        .damage(Fighter::power.bind(), sand::command::DamageKind::Magic);
}
#[function("owned:native_negative")]
fn native_negative() {
    EntityContext::<sand::entity::ZombieKind>::default()
        .living()
        .damage(
            StatCurve::linear(StatCurve::state(Fighter::power), -1.0, 0.0),
            sand::command::DamageKind::Magic,
        );
}
#[function("owned:native_attributed")]
fn native_attributed() {
    EntityScope::bind(&EntityContext::<AnyEntity>::default(), |source| {
        Target::entities().tag("dynamic_victim").each(|_| {
            EntityContext::<sand::entity::ZombieKind>::default()
                .living()
                .damage_by(
                    StatCurve::linear(StatCurve::state(Fighter::power), 0.25, 0.5),
                    sand::command::DamageKind::Magic,
                    source,
                )
        })
    });
}
#[datapack_component]
fn encounter_attack() -> sand::systems::Sequence<AnyEntity> {
    sand::systems::Sequence::new(
        Lifetime::new("owned:encounter_attack".parse().unwrap(), Ticks::new(20))
            .on_start(|ctx| ctx.owner(|_| cmd::say("ENCOUNTER_ATTACK")))
            .on_stop(|ctx| ctx.owner(|_| cmd::say("ENCOUNTER_CHILD_STOP"))),
    )
    .then(|_| Actions::default())
    .wait(Ticks::new(8))
}
#[datapack_component]
fn encounter_controller() -> sand::systems::Encounter<AnyEntity> {
    sand::systems::Encounter::new(
        Lifetime::new("owned:controller".parse().unwrap(), Ticks::new(60))
            .requires::<Fighter>()
            .on_stop(|ctx| ctx.owner(|_| cmd::say("ENCOUNTER_STOP"))),
    )
    .while_active(|owner| owner.state(Fighter::health).matches(1..).unwrap())
    .attack(
        |_| Condition::any([Condition::all([]), Condition::all([])]),
        encounter_attack(),
    )
    .attack(|_| Condition::all([]), encounter_attack())
}
#[function("owned:encounter_start")]
fn encounter_start() {
    encounter_controller().start();
}
#[function("owned:encounter_cancel")]
fn encounter_cancel() {
    encounter_controller().cancel();
}

#[test]
fn combat_exports_checked_runtime_damage_and_isolated_encounters() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        assert_eq!(pack, exported(version));
        let text = pack.to_string();
        assert!(text.contains("$(amount) minecraft:magic"));
        assert!(text.contains(" by @e[tag=__sand_scope_"));
        assert!(text.contains("/encounter_select"));
        assert!(text.contains("scoreboard players operation #absorbed"));
        assert!(text.contains("sand/combat/defeated"));
    }
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "owned", scope = entity)]
struct FractionalFighter {
    #[state(default = 20, min = 0, max = 20, scale = 1000)]
    health: FixedScore,
    #[state(default = 0.5, min = 0, max = 20, scale = 1000)]
    shield: FixedScore,
}
#[function("owned:combat_fractional")]
fn combat_fractional() {
    FractionalFighter::composition_attach("@s");
    sand::systems::Combat::new(FractionalFighter::health, 20)
        .shield(FractionalFighter::shield)
        .resistance(0.2)
        .damage(1.25);
}
#[function("owned:combat_return")]
fn combat_return() {
    sand::systems::Combat::new(Fighter::health, 20)
        .on_hit(cmd::return_fail())
        .damage(1);
    Fighter::hits.bind().add(1);
}

#[function("owned:player_native_attributed", context = player)]
fn player_native_attributed() {
    EntityScope::bind(&EntityContext::<PlayerKind>::default(), |source| {
        Target::players().tag("owner_b").each(|victim| {
            victim.living().damage_by(
                StatCurve::linear(StatCurve::state(Fighter::power), 0.25, 0.5),
                sand::command::DamageKind::Magic,
                source,
            )
        })
    });
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "owned", scope = player)]
struct PlayerFighter {
    #[state(default = 20, min = 0, max = 20)]
    health: Score,
    #[state(default = 5, min = 0, max = 5)]
    shield: Score,
    #[state(auto_tick)]
    immunity: Cooldown,
}
#[function("owned:player_combat", context = player)]
fn player_combat() {
    PlayerFighter::attach(EntityContext::<PlayerKind>::default());
    sand::systems::Combat::new(PlayerFighter::health, 20)
        .shield(PlayerFighter::shield)
        .invulnerability(PlayerFighter::immunity, Ticks::new(4))
        .damage(8);
}
