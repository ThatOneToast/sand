use sand::prelude::*;
use sand::systems::{Lifetime, Progression, Session};

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "progression", scope = entity)]
struct Round {
    #[state(min = 0, max = 2)]
    phase: Score,
    #[state(min = 0, max = 2)]
    quest: Score,
    points: Score,
    rewards: Score,
    stops: Score,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "progression", scope = entity)]
struct Temporary {
    value: Score,
}

#[datapack_component]
fn foreign_quest() -> Progression<AnyEntity> {
    Progression::new(
        "progression_other:shared_quest".parse().unwrap(),
        Round::quest,
    )
    .objective(
        |owner| owner.state(Round::points).matches(1..).unwrap(),
        |owner| owner.state(Round::rewards).add(100),
    )
    .objective(
        |owner| owner.state(Round::points).matches(2..).unwrap(),
        |owner| owner.state(Round::rewards).add(1000),
    )
}

fn quest() -> Progression<AnyEntity> {
    Progression::new("progression:quest".parse().unwrap(), Round::quest)
        .objective(
            |owner| owner.state(Round::points).matches(1..).unwrap(),
            |owner| {
                let mut actions = owner.state(Round::rewards).add(1);
                // Reentrant calls observe committed progress and must not recurse,
                // reset it, or consume the next objective in the same dispatch.
                actions.extend([
                    quest().advance(),
                    quest().reset(),
                    foreign_quest().advance(),
                    foreign_quest().reset(),
                ]);
                actions
            },
        )
        .objective(
            |owner| owner.state(Round::points).matches(2..).unwrap(),
            |owner| {
                let mut actions = owner.state(Round::rewards).add(10);
                actions.extend([cmd::return_cmd(7)]);
                actions
            },
        )
}
#[datapack_component]
fn registered_quest() -> Progression<AnyEntity> {
    quest()
}
fn phases() -> Progression<AnyEntity> {
    Progression::new("progression:phases".parse().unwrap(), Round::phase)
        .objective(
            |owner| owner.state(Round::points).matches(1..).unwrap(),
            |owner| {
                let mut actions = owner.state(Round::rewards).add(1);
                actions.extend([session().restart(), session().reset()]);
                actions
            },
        )
        .objective(
            |owner| owner.state(Round::points).matches(2..).unwrap(),
            |owner| owner.state(Round::rewards).add(10),
        )
}
#[datapack_component]
fn child() -> Lifetime<AnyEntity> {
    Lifetime::new("progression:child".parse().unwrap(), Ticks::new(100))
        .capacity(4)
        .owns::<Temporary>()
}
#[datapack_component]
fn session() -> Session<AnyEntity> {
    Session::new(
        Lifetime::new("progression:session".parse().unwrap(), Ticks::new(100))
            .capacity(2)
            .owns::<Temporary>()
            .on_start(|ctx| ctx.start_child(child()))
            .on_stop(|ctx| {
                ctx.owner(|owner| {
                    let mut actions = owner.state(Round::stops).add(1);
                    actions.extend([session().restart(), session().reset()]);
                    actions
                })
            }),
        phases(),
    )
}
#[function("progression:attach")]
fn attach() {
    Round::composition_attach("@s");
}
#[function("progression:detach")]
fn detach() {
    Round::composition_detach("@s");
}
#[function("progression:quest_advance")]
fn quest_advance() {
    quest().advance();
}
#[function("progression:quest_reset")]
fn quest_reset() {
    quest().reset();
}
#[function("progression:start")]
fn start() {
    session().start();
}
#[function("progression:restart")]
fn restart() {
    session().restart();
}
#[function("progression:reset")]
fn reset() {
    session().reset();
}
#[function("progression:cancel")]
fn cancel() {
    session().cancel();
}

fn export(version: &str) -> serde_json::Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json("progression", version).unwrap())
        .unwrap()
}
fn body<'a>(pack: &'a serde_json::Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["path"] == path)
        .unwrap_or_else(|| panic!("missing function {path}"))["content"]
        .as_str()
        .unwrap()
}
fn all_functions(pack: &serde_json::Value) -> String {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|component| component["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
#[test]
fn ordered_objectives_commit_before_isolated_rewards_and_release_reentry_locks() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = export(version);
        assert_eq!(pack, export(version));
        let advance = body(&pack, "__sand_system/lifetime/quest/progression_advance");
        assert!(advance.contains("matches 1 run return 0"));
        let lock = advance
            .lines()
            .find(|line| line.starts_with("execute if score #busy "))
            .unwrap()
            .split_whitespace()
            .nth(4)
            .unwrap();
        let foreign = body(
            &pack,
            "__sand_system/lifetime/shared_quest/progression_advance",
        );
        assert!(foreign.contains(&format!("execute if score #busy {lock} matches 1")));
        assert!(advance.contains("matches 0"));
        assert!(advance.contains("matches 1"));
        assert_eq!(advance.matches("run return run function").count(), 2);
        assert!(advance.trim_end().ends_with("return 0"));
        let full = all_functions(&pack);
        let terminal = body(&pack, "__sand_system/lifetime/quest/progression_completed");
        assert!(terminal.contains("matches 2 run return 1"));
        let objective = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["content"].as_str())
            .find(|text| {
                text.contains("scoreboard players set #busy")
                    && text.contains("return 1")
                    && text.contains("function")
                    && text.contains(&Round::quest.objective())
            })
            .unwrap();
        assert!(
            objective.find(&Round::quest.objective()).unwrap()
                < objective.find("function").unwrap(),
            "{objective}"
        );
        assert!(
            objective.rfind("#busy").unwrap() > objective.find("function").unwrap(),
            "{objective}"
        );
        assert!(full.contains("return 7"));
        assert!(!advance.contains("scoreboard players set @s"));
    }
}
#[test]
fn sessions_capture_instances_and_use_owner_scoped_cleanup_and_explicit_reset() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = export(version);
        let step = body(&pack, "__sand_system/lifetime/session/session_step");
        assert!(step.contains("/session_advance with entity @s data"));
        let resume = body(&pack, "__sand_system/lifetime/session/session_advance");
        assert!(resume.contains("$(instance)"));
        assert!(resume.contains("/session_after"));
        let stop = body(&pack, "__sand_system/lifetime/session/session_stopped");
        assert!(stop.contains("matches 1 run return run function"));
        let reset = body(&pack, "__sand_system/lifetime/session/session_reset");
        assert!(reset.find("#busy").unwrap() < reset.find("function").unwrap());
        assert!(!reset.contains("/start"));
        let start = body(&pack, "__sand_system/lifetime/session/session_start");
        assert!(start.contains("/progression_completed"));
        let start_calls = start
            .lines()
            .filter_map(|line| line.strip_prefix("function progression:"))
            .map(|path| body(&pack, path))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(start_calls.find("/cancel").unwrap() < start_calls.find("/start").unwrap());
        let cancel = body(&pack, "__sand_system/lifetime/session/session_cancel");
        assert!(!cancel.contains("matches 1 run return 0\nexecute unless score @s"));
        let cleanup = body(&pack, "__sand_system/lifetime/session/cleanup");
        assert!(cleanup.contains("scoreboard players reset @s"));
        assert!(cleanup.contains("kill @s"));
        assert!(body(&pack, "__sand_system/lifetime/session/cancel_owner").contains("$(owner)"));
        assert!(
            body(&pack, "__sand_system/lifetime/session/cancel_descendants")
                .contains("$(instance)")
        );
    }
}
#[test]
fn optional_native_fixture() {
    let Some(root) = std::env::var_os("SAND_PROGRESSION_PROOF_PACK") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let pack = export("26.3");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("pack.mcmeta"), serde_json::to_vec(&serde_json::json!({"pack":{"description":"Sand progression native proof","min_format":121,"max_format":121}})).unwrap()).unwrap();
    std::fs::write(root.join("fixture.json"), serde_json::to_vec_pretty(&serde_json::json!({"phase":Round::phase.objective(), "quest":Round::quest.objective(), "points":Round::points.objective(), "rewards":Round::rewards.objective(), "stops":Round::stops.objective()})).unwrap()).unwrap();
    for component in pack["components"].as_array().unwrap() {
        let directory = component["dir"].as_str().unwrap();
        let extension = if directory == "function" {
            "mcfunction"
        } else {
            "json"
        };
        let path = root
            .join("data")
            .join(component["namespace"].as_str().unwrap())
            .join(directory)
            .join(format!(
                "{}.{}",
                component["path"].as_str().unwrap(),
                extension
            ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let content = component["content"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(&component["content"]).unwrap());
        std::fs::write(path, content).unwrap();
    }
}
