use sand::component::IntoCommands;
use sand::prelude::*;
use sand::systems::Roster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum Role {
    Spectator = 0,
    Fighter = 1,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "roster_proof", scope = player)]
struct Member {
    arena: Score,
    role: EntityEnum<Role>,
    visits: Score,
}
#[datapack_component]
fn arena_a() -> Roster<Role> {
    Roster::new(
        "roster_proof:a".parse().unwrap(),
        Member::arena,
        Member::role,
        1,
    )
}
#[datapack_component]
fn arena_b() -> Roster<Role> {
    Roster::new("foreign:b".parse().unwrap(), Member::arena, Member::role, 2)
}
#[function]
fn open_a() {
    arena_a().open();
}
#[function]
fn open_b() {
    arena_b().open();
}
#[function]
fn close_a() {
    arena_a().close();
}
#[function]
fn join_a() {
    arena_a().join(Role::Fighter);
}
#[function]
fn spectate_a() {
    arena_a().join(Role::Spectator);
}
#[function]
fn join_b() {
    arena_b().join(Role::Fighter);
}
#[function]
fn leave_a() {
    arena_a().leave();
}
#[function]
fn leave_b() {
    arena_b().leave();
}
#[function]
fn visit_a() {
    arena_a().each(None, |player| player.state(Member::visits).add(1));
}
#[function]
fn fighters_a() {
    arena_a().each(Some(Role::Fighter), |player| {
        player.state(Member::visits).add(1)
    });
}
#[function]
fn reenter_a() {
    arena_a().each(None, |player| {
        let mut actions = player.state(Member::visits).add(1);
        actions.extend([
            arena_a().close(),
            arena_a().open(),
            arena_b().join(Role::Fighter),
            arena_a().leave(),
            arena_b().each(None, |_| cmd::say("NESTED_ROSTER_BAD").into_commands()),
            cmd::return_cmd(7).into_commands(),
        ]);
        actions
    });
}
fn export(namespace: &str, version: &str) -> serde_json::Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json(namespace, version).unwrap())
        .unwrap()
}
fn body<'a>(pack: &'a serde_json::Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}
#[test]
fn roster_export_preserves_generation_capacity_and_domain_guards() {
    let mut pack_locks = vec![];
    for namespace in ["roster_proof", "other_export"] {
        for version in ["26.1", "26.2", "26.3"] {
            let pack = export(namespace, version);
            assert_eq!(pack, export(namespace, version));
            let open = body(&pack, "__sand_system/lifetime/a/roster_open");
            assert!(open.find("#generation").unwrap() < open.find("2147483646").unwrap());
            let join = body(&pack, "__sand_system/lifetime/a/roster_join_1");
            assert!(
                join.find("matches 1.. run return 0").unwrap()
                    < join
                        .find(&format!("operation @s {}", Member::arena.objective()))
                        .unwrap()
            );
            let contains = body(&pack, "__sand_system/lifetime/a/roster_contains");
            assert!(contains.contains(&format!(
                "unless score @s {} = #generation",
                Member::arena.objective()
            )));
            assert!(contains.contains("run return 1"));
            let b = body(&pack, "__sand_system/lifetime/b/roster_join_1");
            assert_eq!(
                join.lines().next(),
                b.lines().next(),
                "imported definitions share their field-domain lock"
            );
            if version == "26.3" {
                pack_locks.push(join.lines().next().unwrap().to_owned());
            }
            let any = body(&pack, "__sand_system/lifetime/a/roster_any_1");
            assert!(any.contains(&format!(
                "if score @s {} matches 1 run return 1",
                Member::role.objective()
            )));
            let load = body(&pack, "__sand_system/lifetime/a/roster_load");
            assert!(load.contains("minecraft.custom:minecraft.leave_game"));
            assert!(load.contains("execute unless score #next"));
        }
    }
    assert_ne!(pack_locks[0], pack_locks[1]);
}
#[test]
fn export_native_roster_fixture_when_requested() {
    let Some(path) = std::env::var_os("SAND_ROSTER_PROOF_PACK") else {
        return;
    };
    let root = std::path::PathBuf::from(path);
    std::fs::create_dir_all(&root).unwrap();
    let pack = export("roster_proof", "26.3");
    std::fs::write(
        root.join("pack.mcmeta"),
        r#"{"pack":{"description":"Sand roster proof","min_format":121,"max_format":121}}"#,
    )
    .unwrap();
    for record in pack["components"].as_array().unwrap() {
        let dir = record["dir"].as_str().unwrap();
        let ext = if dir == "function" {
            "mcfunction"
        } else {
            "json"
        };
        let path = root
            .join("data")
            .join(record["namespace"].as_str().unwrap())
            .join(dir)
            .join(format!("{}.{}", record["path"].as_str().unwrap(), ext));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let content = &record["content"];
        std::fs::write(
            path,
            content
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| content.to_string()),
        )
        .unwrap();
    }
    std::fs::write(root.join("fixture.json"),serde_json::json!({"arena":Member::arena.objective(),"role":Member::role.objective(),"visits":Member::visits.objective()}).to_string()).unwrap();
}
