use sand::component::WrittenBook;
use sand::prelude::*;
use sand::systems::{Hud, Lifetime, Scene, Transform};

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "presentation", scope = player)]
struct Status {
    progress: Score,
}
#[datapack_component]
fn hud() -> Hud {
    Hud::new(
        Lifetime::new("presentation:status".parse().unwrap(), Ticks::new(2000))
            .capacity(2)
            .requires::<Status>()
            .on_start(|ctx| ctx.owner(|_| cmd::say("HUD_STARTED")))
            .on_stop(|_| cmd::return_cmd(0)),
    )
    .actionbar(Text::new("Ready"))
    .bossbar(
        Text::new("Progress $(literal)"),
        StatCurve::state(Status::progress),
        100,
    )
}
#[datapack_component]
fn other_hud() -> Hud {
    Hud::new(Lifetime::new(
        "presentation:other".parse().unwrap(),
        Ticks::new(20),
    ))
    .actionbar(Text::new("Other"))
    .bossbar(Text::new("Other progress"), 75, 100)
}
#[datapack_component]
fn foreign_first() -> Hud {
    Hud::new(
        Lifetime::new(
            "hud_import_a:foreign_first".parse().unwrap(),
            Ticks::new(100),
        )
        .on_start(|_| cmd::say("HUD_FOREIGN_FIRST")),
    )
    .actionbar(Text::new("Foreign first"))
}
#[datapack_component]
fn foreign_second() -> Hud {
    Hud::new(Lifetime::new(
        "hud_import_b:foreign_second".parse().unwrap(),
        Ticks::new(100),
    ))
    .actionbar(Text::new("Foreign second"))
}
#[datapack_component]
fn mixed_scene() -> Scene {
    Scene::new(Lifetime::new(
        "presentation:mixed".parse().unwrap(),
        Ticks::new(40),
    ))
    .part(
        "block",
        BlockId::minecraft("stone").unwrap(),
        Transform::default(),
    )
    .item(
        "icon",
        ItemStack::new(ItemId::minecraft("diamond").unwrap())
            .count(3)
            .component(ItemComponent::custom_name(Text::new("Gem\nShard").gold()))
            .component(ItemComponent::custom_data(CustomData::marker("mark\nline"))),
        Transform::default(),
    )
    .text(
        "label",
        Text::new("Ready\nNow\r\nLater\t\"quote\"\\tail").bold(true),
        Transform::default(),
    )
    .item(
        "book",
        ItemStack::new(ItemId::minecraft("written_book").unwrap()).component(
            ItemComponent::WrittenBook(
                WrittenBook::new("Notes", "Sand").page(Text::new("Page one\nPage two")),
            ),
        ),
        Transform::default(),
    )
    .keyframe(
        "icon",
        Ticks::new(1),
        Ticks::new(5),
        Transform::default().yaw(90.0),
    )
    .keyframe(
        "label",
        Ticks::new(1),
        Ticks::new(5),
        Transform::default().translated([0.0, 1.0, 0.0]),
    )
}
#[function("presentation:start")]
fn start() {
    hud().start();
    mixed_scene().start();
}
#[function("presentation:attach")]
fn attach() {
    cmd::say("HUD_PLAYER_READY");
}
#[function("presentation:start_hud")]
fn start_hud() {
    hud().start();
}
#[function("presentation:start_other")]
fn start_other() {
    other_hud().start();
}
#[function("presentation:cancel_hud")]
fn cancel_hud() {
    hud().cancel();
}
#[function("presentation:cancel_other")]
fn cancel_other() {
    other_hud().cancel();
}
#[function("presentation:start_foreign_first")]
fn start_foreign_first() {
    foreign_first().start();
}
#[function("presentation:start_foreign_second")]
fn start_foreign_second() {
    foreign_second().start();
}
#[function("presentation:cancel_foreign_first")]
fn cancel_foreign_first() {
    foreign_first().cancel();
}
#[function("presentation:cancel_foreign_second")]
fn cancel_foreign_second() {
    foreign_second().cancel();
}
#[function("presentation:cancel")]
fn cancel() {
    hud().cancel();
    mixed_scene().cancel();
}

fn exported(version: &str) -> serde_json::Value {
    serde_json::from_str(&sand::advanced::try_export_pack_json("presentation", version).unwrap())
        .unwrap()
}
fn body<'a>(pack: &'a serde_json::Value, definition: &str, part: &str) -> &'a str {
    let path = format!("__sand_system/lifetime/{definition}/{part}");
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == path)
        .unwrap()["content"]
        .as_str()
        .unwrap()
}
#[test]
fn mixed_displays_share_keyframes_and_instance_cleanup_on_all_targets() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        assert_eq!(pack, exported(version));
        let spawn = body(&pack, "mixed", "scene_spawn");
        for kind in ["block_display", "item_display", "text_display"] {
            assert!(
                spawn.contains(&format!("execute summon minecraft:{kind}")),
                "{spawn}"
            );
        }
        let item = body(&pack, "mixed", "scene_part_1");
        assert!(item.contains("id:\"minecraft:diamond\""), "{item}");
        assert!(item.contains("count:3"), "{item}");
        assert!(
            item.contains("minecraft:custom_name") && item.contains("Gem") && item.contains("gold"),
            "{item}"
        );
        let text = body(&pack, "mixed", "scene_part_2");
        assert!(
            text.contains("text:{") && text.contains("Ready") && text.contains("bold:1b"),
            "{text}"
        );
        let update = body(&pack, "mixed", "scene_update");
        assert_eq!(update.matches("interpolation_duration:5").count(), 2);
        assert!(!update.contains("type=minecraft:block_display"), "{update}");
        for part in ["scene_update", "scene_remove"] {
            let commands = body(&pack, "mixed", part);
            assert!(commands.contains("$(instance)"), "{commands}");
            assert!(commands.contains("tag=sand.task."), "{commands}");
        }
        assert!(
            body(&pack, "mixed", "scene_remove_scores").contains("scoreboard players reset @s")
        );
        assert!(body(&pack, "mixed", "scene_remove_scores").contains("kill @s"));
    }
}
#[test]
fn multiline_scene_text_item_names_and_books_stay_on_native_command_lines() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        for (part, expected) in [
            ("scene_part_1", r#"text:"Gem\nShard""#),
            (
                "scene_part_2",
                r#"text:"Ready\nNow\r\nLater\t\"quote\"\\tail""#,
            ),
            ("scene_part_3", r#"text:"Page one\nPage two""#),
        ] {
            let commands = body(&pack, "mixed", part);
            assert!(
                commands.contains(expected),
                "{version} {part}: {commands:?}"
            );
            assert!(
                !commands.contains(['\r', '\t', '\0']),
                "{version} {part}: {commands:?}"
            );
            assert_eq!(
                commands.lines().count(),
                4,
                "{version} {part}: {commands:?}"
            );
            assert_eq!(
                commands
                    .lines()
                    .filter(|line| line.starts_with("data merge entity @s "))
                    .count(),
                1
            );
        }
        assert!(body(&pack, "mixed", "scene_part_1").contains(r#""mark\nline":1b"#));
        assert!(body(&pack, "mixed", "scene_part_3").contains("minecraft:written_book_content"));
    }
}
#[test]
fn hud_exports_private_bars_bounded_reload_cleanup_and_cross_definition_leases() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        if version == "26.2"
            && let Some(root) = std::env::var_os("SAND_PRESENTATION_PROOF_PACK")
        {
            let root = std::path::PathBuf::from(root);
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(
                root.join("fixture.json"),
                serde_json::to_string(
                    &serde_json::json!({"progress":Status::progress.objective()}),
                )
                .unwrap(),
            )
            .unwrap();
            std::fs::write(root.join("pack.mcmeta"), r#"{"pack":{"description":"Sand presentation proof","min_format":107,"max_format":107}}"#).unwrap();
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
                std::fs::write(
                    path,
                    component["content"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| serde_json::to_string(&component["content"]).unwrap()),
                )
                .unwrap();
            }
        }
        let create = body(&pack, "status", "hud_create");
        assert!(create.contains("hud_bar/$(instance)"), "{create}");
        assert!(
            create.contains("active append value {instance:$(instance)}"),
            "{create}"
        );
        let init = body(&pack, "status", "hud_init");
        assert!(init.contains("hud_create with entity @s data"), "{init}");
        assert!(body(&pack, "status", "hud_step").contains("hud_owner with entity @s data"));
        let render = body(&pack, "status", "hud_render");
        assert!(
            render.contains("hud_bar/$(instance) players @s"),
            "{render}"
        );
        assert!(
            render.contains("store result bossbar") && render.contains("matches 0..100"),
            "{render}"
        );
        let release = body(&pack, "status", "hud_release");
        assert!(
            release.contains("$(owner)") && release.contains("$(hud_lease)"),
            "{release}"
        );
        let lease = |definition| {
            body(&pack, definition, "hud_claim")
                .split_whitespace()
                .nth(4)
                .unwrap()
        };
        assert_eq!(lease("status"), lease("other"));
        assert!(!body(&pack, "status", "hud_forget").contains("@a"));
        assert!(body(&pack, "status", "hud_forget").contains("active[{instance:$(instance)}]"));
        let load = body(&pack, "status", "hud_load");
        assert!(
            load.contains("active[0]") && load.contains("/hud_drain"),
            "{load}"
        );
        assert!(body(&pack, "status", "hud_drain").contains("hud_remove_bar with storage"));
        let numeric = body(&pack, "status", "hud_value");
        assert!(
            numeric.starts_with("scoreboard players reset #hud"),
            "{numeric}"
        );
        let all = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|record| record["content"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains(&Status::progress.objective()));
        assert!(all.contains("HUD_STARTED"));
        // Generated framework cleanup follows an isolated user stop callback.
        assert!(body(&pack, "status", "stopped").contains("/hud_cleanup"));
    }
}
#[test]
fn actionbar_claims_share_pack_identity_across_definition_namespaces() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = exported(version);
        let lease = |pack: &serde_json::Value, definition| {
            body(pack, definition, "hud_claim")
                .split_whitespace()
                .nth(4)
                .unwrap()
                .to_owned()
        };
        assert_eq!(lease(&pack, "status"), lease(&pack, "foreign_first"));
        assert_eq!(
            lease(&pack, "foreign_first"),
            lease(&pack, "foreign_second")
        );
        for definition in ["status", "foreign_first", "foreign_second"] {
            let allocate = body(&pack, definition, "hud_allocate");
            assert!(
                allocate.contains("matches 0..2147483646 run return 0"),
                "{allocate}"
            );
            assert!(
                allocate.contains("store result entity @s data.hud_lease"),
                "{allocate}"
            );
            assert!(
                allocate.contains("scoreboard players add #next"),
                "{allocate}"
            );
            let claim = body(&pack, definition, "hud_claim");
            assert!(
                claim.contains("$(hud_lease)") && !claim.contains("$(instance)"),
                "{claim}"
            );
            let render = body(&pack, definition, "hud_render");
            assert!(
                render.contains("matches $(hud_lease) run function"),
                "{render}"
            );
            let load = body(&pack, definition, "hud_load");
            assert!(
                load.contains(&format!(
                    "scoreboard players add #next {} 0",
                    lease(&pack, definition)
                )),
                "{load}"
            );
            assert!(!load.contains("scoreboard players set #next"), "{load}");
            assert!(body(&pack, definition, "start").contains("/hud_available"));
        }
        let alternate: serde_json::Value = serde_json::from_str(
            &sand::advanced::try_export_pack_json("other_pack", version).unwrap(),
        )
        .unwrap();
        assert_ne!(
            lease(&pack, "foreign_first"),
            lease(&alternate, "foreign_first")
        );
    }
}
#[test]
fn transform_composition_rejects_shear_and_nonfinite_inputs() {
    let local = Transform::default().yaw(45.0);
    assert!(
        local
            .then(Transform::default().scaled([2.0, 1.0, 1.0]))
            .is_err()
    );
    assert!(local.then(Transform::default().scaled([2.0; 3])).is_ok());
    assert!(
        Transform::default()
            .yaw(90.0)
            .then(Transform::default().scaled([2.0, 1.0, 1.0]))
            .is_ok()
    );
    assert!(
        local
            .then(Transform::default().translated([f32::INFINITY, 0.0, 0.0]))
            .is_err()
    );
}
