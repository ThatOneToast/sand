use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = player)]
struct PlayerState {
    #[state(
        default = 100,
        min = 0,
        max = 100,
        criterion = "dummy",
        display_name = "Player mana"
    )]
    mana: EntityScore<i32>,
    #[state(default = false)]
    enabled: EntityFlag,
    #[state(default = 0, auto_tick)]
    timer: EntityTimer,
    #[state(auto_tick)]
    dash: EntityCooldown,
    #[state(default_snbt = "{announcements:1b}")]
    preferences: Data<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(StateBundle)]
struct PlayerStateBundle {
    state: PlayerState,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = global)]
struct GlobalState {
    #[state(
        default = 1,
        criterion = "playerKillCount",
        display_name = "World wave"
    )]
    wave: EntityScore<i32>,
    #[state(default = 3)]
    settings: Data<i32>,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = global)]
struct GlobalOptions {
    enabled: Flag,
}

#[allow(dead_code)]
#[derive(StateBundle)]
struct GlobalResourcesBundle {
    state: GlobalState,
    options: GlobalOptions,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = entity)]
struct EntityRuntimeState {
    #[state(default = 0, min = 0, max = 10)]
    charge: EntityScore<i32>,
    cooldown: EntityCooldown,
}

#[state_lifecycle]
impl StateLifecycle for EntityRuntimeState {
    fn reconcile(_ctx: StateReconcile) -> Vec<String> {
        vec!["say runtime reconciled".into()]
    }
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "state_test", scope = living)]
struct LivingRuntimeState {
    timer: EntityTimer,
}

#[entity_archetype]
fn composed_state_archetype() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new(ResourceLocation::new("statepack", "composed_state_archetype").unwrap())
        .components::<LivingRuntimeState>()
        .components::<OptionalMarker>()
}

#[allow(dead_code)]
#[derive(State)]
#[state(
    namespace = "state_test",
    scope = entity,
    version = 3,
    migrate(from = 1, to = 2),
    migrate(from = 2, to = 3)
)]
struct OptionalMarker;

#[allow(dead_code)]
#[derive(StateBundle)]
struct RepeatedMarkerBundle {
    first: OptionalMarker,
    second: OptionalMarker,
}

#[allow(dead_code)]
#[derive(StateBundle)]
struct RuntimeBundle {
    runtime: EntityRuntimeState,
    marker: OptionalMarker,
}

#[allow(dead_code)]
#[derive(StateBundle)]
struct NestedRuntimeBundle {
    components: RuntimeBundle,
}

#[allow(dead_code)]
#[derive(StateQuery)]
#[query(scope = entity)]
struct RuntimeOnly {
    #[require]
    runtime: EntityRuntimeState,
}

#[state_lifecycle]
impl StateLifecycle for OptionalMarker {
    fn initialize(_ctx: StateInit) -> Vec<String> {
        vec!["say marker initialized".into()]
    }

    fn tick(_ctx: StateTick) -> Vec<String> {
        vec!["say marker tick".into()]
    }

    fn cleanup(_ctx: StateCleanup) -> Vec<String> {
        vec!["say marker cleanup".into()]
    }

    fn migrate(ctx: StateMigrate) -> Vec<String> {
        vec![format!("say migrate {} to {}", ctx.from(), ctx.to())]
    }
}

#[allow(dead_code)]
#[derive(StateQuery)]
struct RuntimeEntities {
    #[require]
    runtime: EntityRuntimeState,
    #[optional]
    marker: OptionalMarker,
    #[without]
    living_runtime: LivingRuntimeState,
}

#[system(tick, every = 20)]
fn recharge_runtime(query: RuntimeEntities) {
    query.each(|item| item.runtime.charge.add(1));
}

#[system(tick, every = 20)]
fn recharge_direct(query: EntityRuntimeState) {
    query.each(|runtime| runtime.charge.add(1));
}

fn records() -> Vec<serde_json::Value> {
    serde_json::from_str(&sand_core::try_export_components_json("statepack").unwrap()).unwrap()
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated function {path}"))
}

fn all_function_content(records: &[serde_json::Value]) -> String {
    records
        .iter()
        .filter(|record| record["dir"] == "function")
        .filter_map(|record| record["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn derived_state_lifecycle_is_scoped_deterministic_and_deduplicated() {
    let first = sand_core::try_export_components_json("statepack").unwrap();
    let second = sand_core::try_export_components_json("statepack").unwrap();
    assert_eq!(first, second);

    let records = records();
    let load = function(&records, "__sand_lifecycle_load");
    let objectives: Vec<_> = load
        .lines()
        .filter(|line| line.starts_with("scoreboard objectives add "))
        .collect();
    let unique: std::collections::BTreeSet<_> = objectives.iter().copied().collect();
    // PlayerState, GlobalState, and EntityRuntimeState each own one numeric
    // scratch objective in addition to their persistent lifecycle objectives.
    assert_eq!(objectives.len(), 25);
    assert_eq!(unique.len(), objectives.len());
    assert!(load.contains("#sand_state_test_global_state"));
    assert!(load.contains("dummy \"Player mana\""));
    assert!(load.contains("playerKillCount \"World wave\""));
    assert!(load.contains(
        "execute unless data storage state_test:state components.\"global_state\".settings run data modify storage state_test:state components.\"global_state\".settings set value 3"
    ));

    let init = function(&records, "__sand_lifecycle_init");
    assert_eq!(init.lines().count(), 10);
    let suppression = PlayerState::detach(EntityContext::<PlayerKind>::default())
        .last()
        .and_then(|command| command.split_whitespace().nth(4))
        .expect("player detach publishes its suppression score")
        .to_owned();
    for line in init
        .lines()
        .filter(|line| line.contains("__sand_owner") || line.contains("sand/state_data/keyed/"))
    {
        assert!(
            line.contains(&format!("unless score @s {suppression} matches 1..")),
            "player data initialization must remain suppressed after detach: {line}"
        );
    }

    let tick = function(&records, "__sand_lifecycle_tick");
    let generated = all_function_content(&records);
    assert!(tick.contains("execute as @a run function statepack:__sand_lifecycle_init"));
    assert_eq!(generated.matches("scoreboard players remove @s").count(), 2);
    assert!(!tick.contains("execute as @e run"));
    assert!(tick.contains("execute as @e[scores={"));
    for dirty in [
        PlayerState::timer.dirty_objective(),
        PlayerState::dash.dirty_objective(),
    ] {
        assert!(!tick.contains(&dirty));
    }
}

#[test]
fn state_query_lowers_required_optional_and_forbidden_presence_at_runtime() {
    let charge = EntityRuntimeState::charge.objective();
    let invocation = RuntimeEntities::each(|item| {
        let mut commands = item.runtime.charge.add(1);
        commands.extend(item.marker(|_| vec!["say optional marker".into()]));
        commands
    });
    assert_eq!(invocation.len(), 1);
    let required =
        <EntityRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    assert!(invocation[0].contains(&format!("{}=1", required[0].0)));
    assert!(invocation[0].contains("execute as @e[scores={"));
    assert!(invocation[0].contains(" at @s run function "));

    let generated = sand_core::drain_dyn_fns()
        .into_iter()
        .map(|(_, commands)| commands.join("\n"))
        .find(|content| content.contains("say optional marker"))
        .expect("query callback should be emitted as a generated function");
    let forbidden =
        <LivingRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let optional = <OptionalMarker as sand::__private::StateBundleMember>::presence_requirements();
    assert!(generated.contains(&format!(
        "unless score @s {} matches {}",
        forbidden[0].0, forbidden[0].1
    )));
    assert!(generated.contains(&format!(
        "if score @s {} matches {} run say optional marker",
        optional[0].0, optional[0].1
    )));
    assert!(generated.contains(&charge));
}

#[test]
fn state_query_current_filters_an_event_executor_without_scanning() {
    let commands = RuntimeEntities::current(|item| item.runtime.charge.add(1));
    assert!(!commands.is_empty());
    let required =
        <EntityRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let forbidden =
        <LivingRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    assert!(commands.iter().all(|command| {
        command.starts_with("execute if score @s ")
            && command.contains(&format!("{} matches {}", required[0].0, required[0].1))
            && command.contains(&format!(
                "unless score @s {} matches {}",
                forbidden[0].0, forbidden[0].1
            ))
            && !command.contains("execute as @e")
            && !command.contains("execute as @a")
    }));
}

#[test]
fn scoped_states_query_directly_with_concrete_bound_views() {
    let entity_requirement =
        <EntityRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let entity = <EntityRuntimeState as sand::__private::StateQuerySpec>::each(|runtime| {
        runtime.charge.add(1)
    });
    assert_eq!(entity.len(), 1);
    assert!(entity[0].starts_with("execute as @e[scores={"));
    assert!(entity[0].contains(&format!("{}=1", entity_requirement[0].0)));

    let living_requirement =
        <LivingRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let living = <LivingRuntimeState as sand::__private::StateQuerySpec>::each(|runtime| {
        runtime.timer.tick()
    });
    assert!(living[0].starts_with("execute as @e[scores={"));
    assert!(living[0].contains(&format!("{}=1", living_requirement[0].0)));

    let player_requirement =
        <PlayerState as sand::__private::StateBundleMember>::presence_requirements();
    let player = <PlayerState as sand::__private::StateQuerySpec>::each(|state| state.mana.add(1));
    assert!(player[0].starts_with("execute as @a[scores={"));
    assert!(player[0].contains(&format!("{}=1", player_requirement[0].0)));

    let marker_requirement =
        <OptionalMarker as sand::__private::StateBundleMember>::presence_requirements();
    let marker = <OptionalMarker as sand::__private::StateQuerySpec>::each(|_marker| {
        vec!["say dead marker".into()]
    });
    assert!(marker[0].contains(&format!("{}=3", marker_requirement[0].0)));
}

#[test]
fn direct_state_current_guards_presence_without_a_scan() {
    let requirement =
        <EntityRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let commands = <EntityRuntimeState as sand::__private::StateQuerySpec>::current(|runtime| {
        runtime.charge.add(1)
    });
    assert!(!commands.is_empty());
    assert!(commands.iter().all(|command| {
        command.starts_with(&format!(
            "execute if score @s {} matches {} run ",
            requirement[0].0, requirement[0].1
        )) && !command.contains("execute as @e")
            && !command.contains("execute as @a")
    }));
}

#[test]
fn direct_and_one_field_queries_have_equivalent_presence_lowering() {
    let direct = <EntityRuntimeState as sand::__private::StateQuerySpec>::each(|runtime| {
        runtime.charge.add(1)
    });
    let wrapped = RuntimeOnly::each(|item| item.runtime.charge.add(1));
    assert_eq!(direct, wrapped);

    let direct_current =
        <EntityRuntimeState as sand::__private::StateQuerySpec>::current(|runtime| {
            runtime.charge.add(1)
        });
    let wrapped_current = RuntimeOnly::current(|item| item.runtime.charge.add(1));
    assert_eq!(direct_current, wrapped_current);
}

#[test]
fn direct_bundles_require_every_flattened_component() {
    let flat = <RuntimeBundle as sand::__private::StateQuerySpec>::each(|bundle| {
        bundle.runtime.charge.add(1)
    });
    let nested = <NestedRuntimeBundle as sand::__private::StateQuerySpec>::each(|bundle| {
        bundle.components.runtime.charge.add(1)
    });
    let requirements =
        <RuntimeBundle as sand::__private::StateBundleMember>::presence_requirements();
    assert_eq!(requirements.len(), 2);
    for (objective, version) in requirements {
        assert!(flat[0].contains(&format!("{objective}={version}")));
        assert!(nested[0].contains(&format!("{objective}={version}")));
    }

    let current = <NestedRuntimeBundle as sand::__private::StateQuerySpec>::current(|bundle| {
        bundle.components.runtime.charge.add(1)
    });
    assert!(
        current
            .iter()
            .all(|command| !command.contains("execute as @e"))
    );
}

#[test]
fn attachment_and_detachment_share_the_direct_query_presence_identity() {
    let requirement =
        <EntityRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let owner = EntityContext::<AnyEntity>::default();
    let attach = EntityRuntimeState::attach(owner);
    let detach = EntityRuntimeState::detach(owner);
    let query = <EntityRuntimeState as sand::__private::StateQuerySpec>::each(|runtime| {
        runtime.charge.add(1)
    });

    assert!(attach.iter().any(|command| command.contains(&format!(
        "scoreboard players set @s {} {}",
        requirement[0].0, requirement[0].1
    ))));
    assert!(detach.iter().any(|command| {
        command.contains(&format!("scoreboard players reset @s {}", requirement[0].0))
    }));
    assert!(query[0].contains(&format!("{}={}", requirement[0].0, requirement[0].1)));
    assert!(!query[0].contains(&EntityRuntimeState::charge.objective()));
}

#[test]
fn direct_and_composed_systems_share_the_compatible_outer_scan() {
    let records = records();
    let tick = function(&records, "__sand_system_tick");
    let requirement =
        <EntityRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let selector = format!("@e[scores={{{}={}}}]", requirement[0].0, requirement[0].1);
    let matching_scans = tick
        .lines()
        .filter(|line| line.contains(&format!("execute as {selector} at @s")))
        .collect::<Vec<_>>();
    assert_eq!(
        matching_scans.len(),
        1,
        "the direct State system and equivalent required StateQuery system should share one scan: {tick}"
    );
    let (_, grouped_path) = matching_scans[0]
        .rsplit_once("function statepack:")
        .expect("the shared scan invokes its generated system group");
    assert_eq!(
        function(&records, grouped_path)
            .lines()
            .filter(|line| line.starts_with("function "))
            .count(),
        2,
        "the shared scan group should invoke both compatible systems"
    );
}

#[test]
fn archetype_composition_uses_component_lifecycle_once() {
    let records = records();
    let requirement =
        <LivingRuntimeState as sand::__private::StateBundleMember>::presence_requirements();
    let provision = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .find(|record| {
            record["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/provision"))
                && record["content"].as_str().is_some_and(|content| {
                    content.contains(&format!(
                "scoreboard players set @s {} {}",
                requirement[0].0, requirement[0].1
                    ))
                })
        })
        .and_then(|record| record["content"].as_str())
        .expect("archetype initialize should attach the composed component before publishing completion");
    assert_eq!(
        provision
            .lines()
            .filter(|line| {
                line.ends_with(&format!(
                    "scoreboard players set @s {} {}",
                    requirement[0].0, requirement[0].1
                ))
            })
            .count(),
        1
    );
    assert!(
        provision
            .lines()
            .any(|line| line.contains("unless score @s"))
    );

    let migrated =
        <OptionalMarker as sand::__private::StateBundleMember>::presence_requirements()[0].clone();
    for (from, to) in [(1, 2), (2, 3)] {
        assert_eq!(
            provision
                .matches(&format!("if score @s {} matches {from}", migrated.0))
                .count(),
            2,
            "each component migration has one hook and one presence-version update",
        );
        assert!(provision.contains(&format!(
            "run scoreboard players set @s {} {to}",
            migrated.0
        )));
    }
    assert_eq!(
        provision
            .lines()
            .filter(|line| line.ends_with(&format!(
                "scoreboard players set @s {} {}",
                migrated.0, migrated.1
            )) && line.starts_with("execute unless score @s"))
            .count(),
        1,
        "archetype composition must publish one canonical component presence value",
    );

    let initialize = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .filter_map(|record| record["content"].as_str())
        .find(|content| {
            content.contains("/provision")
                && content
                    .lines()
                    .last()
                    .is_some_and(|line| line.starts_with("tag @s add __sand.a."))
        })
        .expect("archetype initialize should publish completion after provisioning");
    assert!(initialize.lines().next().unwrap().contains("/provision"));

    let cleanup = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .filter_map(|record| record["content"].as_str())
        .find(|content| {
            content.contains(&format!("scoreboard players reset @s {}", requirement[0].0))
                && content
                    .lines()
                    .last()
                    .is_some_and(|line| line.starts_with("tag @s remove __sand.external."))
        })
        .expect("archetype cleanup should detach the composed component");
    let unique = cleanup.lines().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), cleanup.lines().count());
}

#[test]
fn custom_lifecycle_hooks_are_version_gated_and_cleanup_runs_first() {
    let entity = EntityContext::<AnyEntity>::default();
    let requirements =
        <OptionalMarker as sand::__private::StateBundleMember>::presence_requirements();
    let (presence, version) = &requirements[0];
    let attach = OptionalMarker::attach(entity);
    let initialize = attach
        .iter()
        .position(|command| command.contains("say marker initialized"))
        .expect("attach should invoke initialization hook");
    let publish = attach
        .iter()
        .position(|command| {
            command.ends_with(&format!("scoreboard players set @s {presence} {version}"))
        })
        .expect("attach should publish presence");
    assert!(initialize < publish);
    assert!(attach[initialize].contains(&format!("unless score @s {presence} matches 1..")));
    assert!(attach.iter().any(|command| command.contains(&format!(
        "if score @s {presence} matches 1 run say migrate 1 to 2"
    ))));
    assert!(
        attach
            .iter()
            .any(|command| command.ends_with(&format!("scoreboard players set @s {presence} 2")))
    );
    assert!(attach.iter().any(|command| command.contains(&format!(
        "if score @s {presence} matches 2 run say migrate 2 to 3"
    ))));

    let detach = OptionalMarker::detach(entity);
    assert!(detach[0].contains("say marker cleanup"));
    assert!(detach[0].contains(&format!("if score @s {presence} matches 1..")));

    let records = records();
    let tick = function(&records, "__sand_lifecycle_tick").to_owned();
    let generated = all_function_content(&records);
    assert!(tick.contains(&format!("@e[scores={{{presence}={version}}}]")));
    assert!(generated.contains("say marker tick"));
}

#[test]
fn repeated_bundle_members_reuse_one_component_lifecycle() {
    let entity = EntityContext::<AnyEntity>::default();
    assert_eq!(
        RepeatedMarkerBundle::attach(entity),
        OptionalMarker::attach(entity)
    );
    assert_eq!(
        RepeatedMarkerBundle::detach(entity),
        OptionalMarker::detach(entity)
    );
    let bound = RepeatedMarkerBundle::on(entity);
    let _: OptionalMarkerBound = bound.first;
    let _: OptionalMarkerBound = bound.second;
}

#[test]
fn player_bundle_lifecycle_preserves_explicit_observation_suppression() {
    let player = EntityContext::<PlayerKind>::default();
    assert_eq!(
        PlayerStateBundle::attach(player),
        PlayerState::attach(player)
    );
    assert_eq!(
        PlayerStateBundle::detach(player),
        PlayerState::detach(player)
    );
}

#[test]
fn global_bundle_uses_each_components_deterministic_singleton_holder() {
    let bundle = GlobalResourcesBundle::global();
    assert_eq!(bundle.state.wave.set(7), GlobalState::global().wave.set(7));
    assert_eq!(
        bundle.options.enabled.enable(),
        GlobalOptions::global().enabled.enable()
    );

    let mut attach = GlobalState::attach();
    attach.extend(GlobalOptions::attach());
    let mut seen = std::collections::BTreeSet::new();
    attach.retain(|command| seen.insert(command.clone()));
    assert_eq!(GlobalResourcesBundle::attach_global(), attach);

    let mut detach = GlobalOptions::detach();
    detach.extend(GlobalState::detach());
    let mut seen = std::collections::BTreeSet::new();
    detach.retain(|command| seen.insert(command.clone()));
    assert_eq!(GlobalResourcesBundle::detach_global(), detach);
}

#[test]
fn tick_system_uses_global_cadence_and_query_iteration() {
    let records = records();
    let load = function(&records, "__sand_system_load");
    let tick = function(&records, "__sand_system_tick");
    assert!(load.contains("scoreboard objectives add "));
    assert!(load.contains("scoreboard players set #sand_system"));
    assert!(tick.contains("scoreboard players add #sand_system"));
    assert!(tick.contains("matches 20.. run execute as @e[scores={"));
    assert!(tick.contains(" at @s run function statepack:__sand_system/"));
    let system = records
        .iter()
        .find(|record| {
            record["dir"] == "function"
                && record["path"]
                    .as_str()
                    .is_some_and(|path| path.starts_with("__sand_system/"))
        })
        .and_then(|record| record["content"].as_str())
        .expect("system body should be exported");
    assert!(system.contains("function statepack:sand/entity_query/"));
}

#[test]
fn bound_views_emit_complete_scope_aware_command_vectors() {
    let player = PlayerState::on(EntityContext::<PlayerKind>::default());
    let mana = PlayerState::mana.objective();
    assert_eq!(
        player.mana.set(25),
        vec![
            format!("scoreboard players set @s {mana} 25"),
            format!(
                "execute if score @s {mana} matches ..-1 run scoreboard players set @s {mana} 0"
            ),
            format!(
                "execute if score @s {mana} matches 101.. run scoreboard players set @s {mana} 100"
            ),
        ]
    );
    let enabled = PlayerState::enabled.objective();
    assert_eq!(
        player.enabled.enable(),
        vec![
            format!("scoreboard players set @s {enabled} 1"),
            format!(
                "execute if score @s {enabled} matches ..-1 run scoreboard players set @s {enabled} 0"
            ),
            format!(
                "execute if score @s {enabled} matches 2.. run scoreboard players set @s {enabled} 1"
            ),
        ]
    );
    assert_eq!(
        player.enabled.disable(),
        vec![
            format!("scoreboard players set @s {enabled} 0"),
            format!(
                "execute if score @s {enabled} matches ..-1 run scoreboard players set @s {enabled} 0"
            ),
            format!(
                "execute if score @s {enabled} matches 2.. run scoreboard players set @s {enabled} 1"
            ),
        ]
    );
    let timer = PlayerState::timer.objective();
    assert_eq!(
        player.timer.start(Ticks::new(5)),
        vec![
            format!("scoreboard players set @s {timer} 5"),
            format!(
                "execute if score @s {timer} matches ..-1 run scoreboard players set @s {timer} 0"
            ),
        ]
    );
    assert_eq!(
        player.timer.tick(),
        vec![format!(
            "execute if score @s {timer} matches 1.. run scoreboard players remove @s {timer} 1"
        )]
    );
    let dash = PlayerState::dash.objective();
    assert_eq!(
        player.dash.start(Ticks::new(10)),
        vec![
            format!("scoreboard players set @s {dash} 10"),
            format!(
                "execute if score @s {dash} matches ..-1 run scoreboard players set @s {dash} 0"
            ),
        ]
    );

    let global = GlobalState::global();
    let wave = GlobalState::wave.objective();
    let holder = "#sand_state_test_global_state_e53d9a32";
    assert_eq!(
        global.wave.add(1),
        vec![format!("scoreboard players add {holder} {wave} 1")]
    );
    assert_eq!(
        global.wave.get().command(),
        format!("scoreboard players get {holder} {wave}")
    );
    assert_ne!(PlayerState::mana.objective(), GlobalState::wave.objective());
    assert_eq!(
        global.settings.set(4),
        vec![
            "data modify storage state_test:state components.\"global_state\".settings set value 4"
        ]
    );
    assert_eq!(
        global.settings.get(),
        "data get storage state_test:state components.\"global_state\".settings"
    );
    assert!(GlobalState::detach().iter().any(|command| {
        command == "data remove storage state_test:state components.\"global_state\".settings"
    }));
}

#[test]
fn entity_and_living_bound_views_retain_archetype_dirty_semantics() {
    let entity = EntityRuntimeState::on(EntityContext::<AnyEntity>::default());
    let runtime_schema = EntityRuntimeState::schema();
    let runtime_reconcile_dirty = ObjectiveName::logical(format!(
        "{}:{}.reconcile_dirty",
        runtime_schema.namespace, runtime_schema.name
    ))
    .as_str()
    .to_owned();
    let charge = EntityRuntimeState::charge.objective();
    let charge_dirty = EntityRuntimeState::charge.dirty_objective();
    assert_eq!(
        entity.charge.add(2),
        vec![
            format!("scoreboard players add @s {charge} 2"),
            format!(
                "execute if score @s {charge} matches ..-1 run scoreboard players set @s {charge} 0"
            ),
            format!(
                "execute if score @s {charge} matches 11.. run scoreboard players set @s {charge} 10"
            ),
            format!("scoreboard players set @s {charge_dirty} 1"),
            format!("scoreboard players set @s {runtime_reconcile_dirty} 1"),
        ]
    );
    let cooldown = EntityRuntimeState::cooldown.objective();
    let cooldown_dirty = EntityRuntimeState::cooldown.dirty_objective();
    assert_eq!(
        entity.cooldown.start(Ticks::new(8)),
        vec![
            format!("scoreboard players set @s {cooldown} 8"),
            format!(
                "execute if score @s {cooldown} matches ..-1 run scoreboard players set @s {cooldown} 0"
            ),
            format!("scoreboard players set @s {cooldown_dirty} 1"),
            format!("scoreboard players set @s {runtime_reconcile_dirty} 1"),
        ]
    );

    let living = LivingRuntimeState::on(EntityContext::<ZombieKind>::default());
    let living_schema = LivingRuntimeState::schema();
    let living_reconcile_dirty = ObjectiveName::logical(format!(
        "{}:{}.reconcile_dirty",
        living_schema.namespace, living_schema.name
    ))
    .as_str()
    .to_owned();
    let timer = LivingRuntimeState::timer.objective();
    let timer_dirty = LivingRuntimeState::timer.dirty_objective();
    assert_eq!(
        living.timer.start(Ticks::new(4)),
        vec![
            format!("scoreboard players set @s {timer} 4"),
            format!(
                "execute if score @s {timer} matches ..-1 run scoreboard players set @s {timer} 0"
            ),
            format!("scoreboard players set @s {timer_dirty} 1"),
            format!("scoreboard players set @s {living_reconcile_dirty} 1"),
        ]
    );
    assert_eq!(
        living.timer.tick(),
        vec![
            format!(
                "execute if score @s {timer} matches 1.. run scoreboard players set @s {timer_dirty} 1"
            ),
            format!(
                "execute if score @s {timer} matches 1.. run scoreboard players set @s {living_reconcile_dirty} 1"
            ),
            format!(
                "execute if score @s {timer} matches 1.. run scoreboard players remove @s {timer} 1"
            ),
        ]
    );

    let generated = all_function_content(&records());
    assert!(generated.contains(&format!(
        "execute if score @s {runtime_reconcile_dirty} matches 1.. run function statepack:sand/state_reconcile/"
    )));
    assert!(generated.contains("say runtime reconciled"));
    assert!(generated.contains(&format!(
        "scoreboard players reset @s {runtime_reconcile_dirty}"
    )));
}

fn referenced_objectives(
    commands: impl IntoIterator<Item = String>,
) -> std::collections::BTreeSet<String> {
    let mut objectives = std::collections::BTreeSet::new();
    for command in commands {
        let tokens: Vec<_> = command.split_whitespace().collect();
        for window in tokens.windows(5) {
            if window[0] == "scoreboard" && window[1] == "players" {
                objectives.insert(window[4].to_string());
            }
            if window[0] == "score" {
                objectives.insert(window[2].to_string());
            }
        }
    }
    objectives
}

#[test]
fn player_and_global_commands_reference_only_provisioned_objectives() {
    let records = records();
    let provisioned: std::collections::BTreeSet<_> = function(&records, "__sand_lifecycle_load")
        .lines()
        .filter_map(|line| {
            line.strip_prefix("scoreboard objectives add ")
                .and_then(|rest| rest.split_whitespace().next())
                .map(str::to_owned)
        })
        .collect();

    let player = PlayerState::on(EntityContext::<PlayerKind>::default());
    let global = GlobalState::global();
    let commands = [
        player.mana.set(25),
        player.enabled.enable(),
        player.enabled.disable(),
        player.timer.start(Ticks::new(5)),
        player.timer.tick(),
        player.dash.start(Ticks::new(10)),
        global.wave.add(1),
        vec![global.wave.get().command()],
    ]
    .concat();
    let referenced = referenced_objectives(commands.clone());
    assert!(
        referenced.is_subset(&provisioned),
        "unprovisioned objectives: {:?}",
        referenced.difference(&provisioned).collect::<Vec<_>>()
    );
    for dirty in [
        PlayerState::mana.dirty_objective(),
        PlayerState::enabled.dirty_objective(),
        PlayerState::timer.dirty_objective(),
        PlayerState::dash.dirty_objective(),
        GlobalState::wave.dirty_objective(),
    ] {
        assert!(
            commands.iter().all(|command| !command.contains(&dirty)),
            "player/global commands referenced dirty objective {dirty}"
        );
    }
}
