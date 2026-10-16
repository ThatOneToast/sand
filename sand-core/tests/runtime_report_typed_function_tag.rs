//! Typed lifecycle tags keep factory contributors distinct from function owners.
use sand::prelude::*;

/// The authored implementation keeps its own Rust definition as owner.
#[function]
fn ordinary() {
    cmd::say("typed tag member");
}

/// A standalone typed load tag contributes a membership, not the implementation.
#[datapack_component]
fn typed_load() -> sand::component::TypedTag<FunctionId> {
    sand::component::TypedTag::new("minecraft:load".parse().unwrap()).entry(ordinary.function_id())
}

/// The same implementation can have a different contributor in another phase.
#[datapack_component]
fn typed_tick() -> sand::component::TypedTag<FunctionId> {
    sand::component::TypedTag::new("minecraft:tick".parse().unwrap()).entry(ordinary.function_id())
}

/// No automatic lifecycle resources are declared, preserving output uniqueness.
#[test]
fn typed_lifecycle_tags_retain_their_factory_contributors() {
    for namespace in ["typed_owners", "other_typed"] {
        let json = sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
        let export: serde_json::Value = serde_json::from_str(&json).unwrap();
        let report = &export["runtime_report"];
        let function = format!("{namespace}:ordinary");
        for (phase, factory) in [("load", "typed_load"), ("tick", "typed_tick")] {
            let entry = report[phase]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["reference"] == function)
                .unwrap();
            assert_eq!(
                entry["contributors"],
                serde_json::json!([format!("{}::{factory}", module_path!())])
            );
        }
        assert_eq!(
            report["functions"][function]["owners"],
            serde_json::json!([format!("{}::ordinary", module_path!())])
        );
        assert_eq!(
            json,
            sand::advanced::try_export_pack_json(namespace, "26.2").unwrap()
        );
    }
}
