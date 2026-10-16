//! Reporting keeps semantic owners across nested resources and tag descriptors.
use sand::prelude::*;
use sand::registration::DatapackComponent;

struct Bundle(ResourceLocation);

impl DatapackComponent for Bundle {
    fn resource_location(&self) -> &ResourceLocation {
        &self.0
    }
    fn component_dir(&self) -> &'static str {
        "predicate"
    }
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({"condition": "minecraft:random_chance", "chance": 1.0})
    }
    fn nested_components(&self) -> Vec<Box<dyn DatapackComponent>> {
        vec![Box::new(
            sand::component::McFunction::new("__sand_local:nested_body".parse().unwrap())
                .command("say nested"),
        )]
    }
}

#[datapack_component]
fn bundle() -> Bundle {
    Bundle("__sand_local:bundle".parse().unwrap())
}

#[function]
fn ordinary_entry() {
    cmd::say("ordinary");
}

#[on_event]
fn joined(_event: Event<sand::events::OnJoinEvent>) {
    cmd::say("joined");
}

#[sand::schedule(ticks = 60, every = 5)]
fn scheduled() {
    cmd::say("scheduled");
}

#[sand::armor_event(Equip, slot = Head, item = "minecraft:diamond_helmet")]
fn equipped() {
    cmd::say("equipped");
}

/// A second armor subscription must share coordinator ownership.
#[sand::armor_event(Unequip, slot = Head, item = "minecraft:diamond_helmet")]
fn unequipped() {
    cmd::say("unequipped");
}

#[datapack_component(Tick)]
fn tick_entry() {
    cmd::say("tick");
}

#[datapack_component(Load)]
fn load_entry() {
    cmd::say("load");
}

#[datapack_component(Tag = "owners:custom")]
fn custom_entry() {
    cmd::say("custom");
}

// Preconstruct and replay identical nested IR from two distinct authored owners.
static SHARED: std::sync::OnceLock<Actions> = std::sync::OnceLock::new();
/// Reuse nested helpers before and during repeated export registration scopes.
fn shared_actions() -> Actions {
    SHARED
        .get_or_init(|| {
            let condition = sand_core::state::Flag::new("report_ready")
                .of("@s")
                .is_true();
            let leaf = when(condition.clone()).then_all(["say shared report leaf"]);
            mcfunction![
                leaf.clone();
                unless(condition).then_all(leaf);
            ]
        })
        .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        owner: concat!(module_path!(), "::shared_first"),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "shared_first",
        make: shared_actions,
    }
}
sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        owner: concat!(module_path!(), "::shared_second"),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "shared_second",
        make: shared_actions,
    }
}

/// Contribute one owner to the shared tick-poll coordinator.
#[on_event(item = "minecraft:stone")]
fn holding_stone(_event: Event<sand::events::HoldingItemEvent>) {
    cmd::say("holding stone");
}

/// Contribute a distinct owner to the same tick-poll coordinator.
#[on_event(item = "minecraft:diamond")]
fn holding_diamond(_event: Event<sand::events::HoldingItemEvent>) {
    cmd::say("holding diamond");
}

/// Nested functions inherit their factory owner and descriptors retain Rust owners.
#[test]
fn expanded_resources_and_attribute_tags_keep_semantic_owners() {
    let _prebuilt = shared_actions();
    for namespace in ["owners", "other_owners"] {
        let json = sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
        let export: serde_json::Value = serde_json::from_str(&json).unwrap();
        let report = &export["runtime_report"];
        let owner = |name: &str| format!("{}::{name}", module_path!());
        assert_eq!(
            report["functions"][format!("{namespace}:nested_body")]["owners"],
            serde_json::json!([owner("bundle")])
        );
        assert_eq!(
            report["functions"][format!("{namespace}:ordinary_entry")]["owners"],
            serde_json::json!([owner("ordinary_entry")])
        );
        for (path, name) in [
            ("joined", "joined"),
            ("equipped", "equipped"),
            ("scheduled", "scheduled"),
            ("scheduled_start", "scheduled"),
            ("scheduled_stop", "scheduled"),
            ("__sand_sched_init", "scheduled"),
            ("__sand_sched_tick", "scheduled"),
        ] {
            assert_eq!(
                report["functions"][format!("{namespace}:{path}")]["owners"],
                serde_json::json!([owner(name)]),
                "missing owner for {path}"
            );
        }
        let shared_owners = serde_json::json!([owner("shared_first"), owner("shared_second")]);
        let records = export["components"].as_array().unwrap();
        let helpers: Vec<_> = records
            .iter()
            .filter(|record| {
                record["dir"] == "function"
                    && record["content"].as_str().is_some_and(|body| {
                        body.contains("shared report leaf")
                            || body.contains("function ") && body.contains("report_ready")
                    })
                    && record["path"] != "shared_first"
                    && record["path"] != "shared_second"
            })
            .collect();
        assert!(
            helpers.len() >= 2,
            "nested shared helper fixture must emit multiple bodies"
        );
        for helper in helpers {
            let id = format!(
                "{}:{}",
                helper["namespace"].as_str().unwrap(),
                helper["path"].as_str().unwrap()
            );
            assert_eq!(report["functions"][&id]["owners"], shared_owners, "{id}");
        }
        let armor = format!("{namespace}:__sand_armor_check");
        let armor_owners = serde_json::json!([owner("equipped"), owner("unequipped")]);
        assert_eq!(report["functions"][&armor]["owners"], armor_owners);
        let armor_tick = report["tick"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["reference"] == armor)
            .unwrap();
        assert_eq!(armor_tick["contributors"], armor_owners);
        let coordinator = format!("{namespace}:__sand_tick_check");
        let event_owners = serde_json::json!([owner("holding_diamond"), owner("holding_stone")]);
        assert_eq!(report["functions"][&coordinator]["owners"], event_owners);
        let tick = report["tick"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["reference"] == coordinator)
            .unwrap();
        assert_eq!(tick["contributors"], event_owners);
        for (tag, name) in [
            ("minecraft:tick", "tick_entry"),
            ("minecraft:load", "load_entry"),
            ("owners:custom", "custom_entry"),
        ] {
            let member = format!("{namespace}:{name}");
            let contributions = report["provenance"]["tag_contributors"].as_array().unwrap();
            let entry = contributions
                .iter()
                .find(|entry| entry["tag"] == tag && entry["member"] == member)
                .unwrap();
            assert_eq!(entry["owners"], serde_json::json!([owner(name)]));
            assert_eq!(report["functions"][&member]["owners"], entry["owners"]);
            if let Some(phase) = tag.strip_prefix("minecraft:") {
                let lifecycle = report[phase]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|entry| entry["reference"] == member)
                    .unwrap();
                assert_eq!(lifecycle["contributors"], entry["owners"]);
            }
        }
        assert_eq!(
            json,
            sand::advanced::try_export_pack_json(namespace, "26.2").unwrap()
        );
    }
}
