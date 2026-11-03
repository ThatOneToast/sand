use sand::component::McFunction;
use sand::prelude::*;
use sand::registration::{DatapackRegistration, IntoDatapack};
use sand::systems::ability::{Ability, ResourceMaintenance, ResourcePool};
use sand_core::serde_json;
use std::cell::Cell;

thread_local! { static MODE: Cell<u8> = const { Cell::new(0) }; }

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "composition", scope = player)]
struct Energy {
    #[state(default = 60, min = 0, max = 60)]
    value: Score,
    #[state(auto_tick)]
    cooldown: Cooldown,
    #[state(auto_tick)]
    regen: Timer,
}

fn policy(cost: i32, ticks: Ticks) -> sand::command::CommandResult<Ability> {
    let player = EntityContext::<PlayerKind>::default();
    let state = Energy::on(player);
    Ability::new(
        ResourcePool::new(state.value, 60),
        state.cooldown,
        cost,
        ticks,
        player
            .equipment()
            .helmet()
            .matches(vanilla::Item::IronHelmet)
            .unwrap(),
    )
}

thread_local! {
    static CACHED: Actions = policy(13, Ticks::new(29)).unwrap().accept(
        Condition::entity(Target::self_()), mcfunction![cmd::say("accepted");]);
}

fn maintenance(id: &str) -> ResourceMaintenance {
    ResourceMaintenance::new(
        ResourceLocation::local(id).unwrap(),
        Target::players().each(|player| {
            let state = Energy::on(*player);
            ResourcePool::new(state.value, 60)
                .regenerate(state.regen, 4, Ticks::new(17))
                .unwrap()
        }),
    )
}

#[datapack_component]
fn installation() -> DatapackRegistration {
    let mode = MODE.get();
    if mode == 0 {
        return DatapackRegistration::new();
    }
    // Genuinely construct/link the same public implementation, including actions,
    // then discard it. The State schema above remains identical in every mode.
    let ability = policy(13, Ticks::new(29)).unwrap();
    let maintain = maintenance("maintain");
    let activation =
        McFunction::new("composition:activate".parse().unwrap()).command(ability.accept(
            Condition::entity(Target::self_()),
            mcfunction![cmd::say("accepted");],
        ));
    if mode == 1 {
        return DatapackRegistration::new();
    }
    let activation = if mode == 5 {
        McFunction::new(ResourceLocation::local("activate").unwrap())
            .command(CACHED.with(Clone::clone))
    } else {
        activation
    };
    let mut pack = maintain.into_datapack().component(activation);
    if mode == 3 {
        pack = pack.merge(maintenance("maintain"));
    }
    if mode == 4 {
        pack = pack.component(
            McFunction::new("composition:other".parse().unwrap()).command(
                policy(19, Ticks::new(31))
                    .unwrap()
                    .accept(Condition::all([]), cmd::say("other")),
            ),
        );
    }
    pack
}

fn export(mode: u8) -> Result<String, String> {
    MODE.set(mode);
    sand::advanced::try_export_pack_json("composition", "26.2").map_err(|e| e.to_string())
}

#[test]
fn unused_construction_has_no_resources_or_runtime_delta_on_same_state_baseline() {
    let baseline = export(0).unwrap();
    assert_eq!(baseline, export(1).unwrap());
    let used = export(2).unwrap();
    assert_ne!(baseline, used);
    assert_eq!(used, export(2).unwrap());
    let other = sand::advanced::try_export_pack_json("alternate", "26.2").unwrap();
    assert_ne!(used, other);
    assert_eq!(used, export(2).unwrap());
    assert_eq!(
        baseline,
        export(1).unwrap(),
        "export registration must stay isolated"
    );
}

#[test]
fn repeated_installation_fails_and_distinct_abilities_share_one_maintenance() {
    let error = export(3).unwrap_err();
    assert!(
        error.contains("collid") || error.contains("duplicate"),
        "{error}"
    );
    let pack: serde_json::Value = serde_json::from_str(&export(4).unwrap()).unwrap();
    let components = pack["components"].as_array().unwrap();
    let functions: Vec<_> = components
        .iter()
        .filter(|c| c["dir"] == "function")
        .collect();
    assert_eq!(
        functions.iter().filter(|c| c["path"] == "maintain").count(),
        1
    );
    let tick = functions
        .iter()
        .find(|c| c["path"] == "__sand_lifecycle_tick")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    assert_eq!(tick.matches("function composition:maintain").count(), 1);
    let all = functions
        .iter()
        .map(|c| c["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for (cost, duration) in [(13, 29), (19, 31)] {
        assert!(
            all.contains(&format!(
                "scoreboard players remove @s {} {cost}",
                Energy::value.objective()
            )),
            "{all}"
        );
        assert!(
            all.contains(&format!(
                "scoreboard players set @s {} {duration}",
                Energy::cooldown.objective()
            )),
            "{all}"
        );
    }
}

#[test]
fn invalid_cost_and_cadence_fail_before_registration() {
    assert!(policy(-1, Ticks::new(1)).is_err());
    assert!(policy(0, Ticks::new(0)).is_err());
    assert!(policy(0, Ticks::new(u32::MAX)).is_err());
    assert!(policy(0, Ticks::new(1)).is_ok());
    let state = Energy::on(EntityContext::<PlayerKind>::default());
    let pool = ResourcePool::new(state.value, 60);
    assert!(pool.regenerate(state.regen, 0, Ticks::new(1)).is_err());
    assert!(pool.regenerate(state.regen, -1, Ticks::new(1)).is_err());
    assert!(pool.regenerate(state.regen, 1, Ticks::new(0)).is_err());
    assert!(
        pool.regenerate(state.regen, 1, Ticks::new(u32::MAX))
            .is_err()
    );
}

#[test]
fn cached_actions_reexport_under_two_namespaces_without_leaking_helpers() {
    let first = export(5).unwrap();
    let alternate = sand::advanced::try_export_pack_json("alternate", "26.2").unwrap();
    assert!(alternate.contains("alternate:maintain"));
    assert!(!alternate.contains("function composition:maintain"));
    assert_eq!(first, export(5).unwrap());
    assert_eq!(export(0).unwrap(), export(1).unwrap());
}
