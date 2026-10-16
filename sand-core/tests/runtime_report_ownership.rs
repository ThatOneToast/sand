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

/// Nested functions inherit their factory owner and descriptors retain Rust owners.
#[test]
fn expanded_resources_and_attribute_tags_keep_semantic_owners() {
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
