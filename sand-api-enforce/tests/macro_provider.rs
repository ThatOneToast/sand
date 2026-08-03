use sand_api_enforce::{
    ReachableKind, event_generated_type_provider, registry_id_provider, resource_ref_provider,
    vanilla_registry_enum_provider,
};
use tempfile::tempdir;

#[test]
fn repository_resource_ref_family_comes_from_generator_and_invocations() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let provider = resource_ref_provider(&workspace.join("sand-core/src/resource_ref.rs")).unwrap();
    assert_eq!(provider.len(), 6);
    assert!(provider.iter().all(|item| {
        item.provider == "generated_resource_refs"
            && item.kind == ReachableKind::Struct
            && item.members
                == [
                    ("external".into(), ReachableKind::Method),
                    ("location".into(), ReachableKind::Method),
                    ("new".into(), ReachableKind::Method),
                ]
    }));
}

#[test]
fn repository_registry_id_family_tracks_every_invocation() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let provider =
        registry_id_provider(&workspace.join("sand-components/src/registry.rs")).unwrap();
    assert_eq!(provider.len(), 34);
    assert!(provider.iter().all(|item| {
        item.provider == "generated_registry_ids"
            && item.members
                == [
                    ("as_resource_location".into(), ReachableKind::Method),
                    ("custom".into(), ReachableKind::Method),
                    ("minecraft".into(), ReachableKind::Method),
                ]
    }));
}

#[test]
fn generator_method_and_invocation_growth_changes_provider_without_a_list_edit() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("resource_ref.rs");
    std::fs::write(
        &source,
        r#"
        macro_rules! resource_ref {
            ($name:ident) => {
                pub struct $name;
                impl $name {
                    pub fn new() {}
                    pub fn typed() {}
                }
            };
        }
        resource_ref!(FirstRef);
        resource_ref!(SecondRef);
        "#,
    )
    .unwrap();
    let provider = resource_ref_provider(&source).unwrap();
    assert_eq!(provider.len(), 2);
    assert!(provider.iter().all(|item| {
        item.members
            == [
                ("new".into(), ReachableKind::Method),
                ("typed".into(), ReachableKind::Method),
            ]
    }));
}

#[test]
fn effect_registry_provider_reads_types_variants_and_methods_from_the_declaration() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let provider =
        vanilla_registry_enum_provider(&workspace.join("sand-components/src/effect.rs")).unwrap();
    assert_eq!(provider.len(), 2);
    let status_effect = provider
        .iter()
        .find(|item| item.identity.ends_with("::EffectId"))
        .unwrap();
    assert_eq!(status_effect.kind, ReachableKind::Enum);
    assert!(
        status_effect
            .members
            .contains(&("Speed".into(), ReachableKind::Variant))
    );
    assert!(
        status_effect
            .members
            .contains(&("Custom".into(), ReachableKind::Variant))
    );
    assert!(
        status_effect
            .members
            .contains(&("custom".into(), ReachableKind::Method))
    );
    assert!(
        status_effect
            .members
            .contains(&("as_resource_location".into(), ReachableKind::Method))
    );
}

#[test]
fn event_provider_covers_exact_checked_in_generated_marker_types() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let provider =
        event_generated_type_provider(&workspace.join("sand-core/src/events/mod.rs")).unwrap();
    assert_eq!(provider.len(), 25);
    let identities = provider
        .iter()
        .map(|item| item.identity.as_str())
        .collect::<BTreeSet<_>>();
    assert!(identities.contains("sand_core::events::PlayerEnteredSurvivalEvent"));
    assert!(identities.contains("sand_core::events::PlayerExitedSpectatorEvent"));
    assert!(identities.contains("sand_core::events::Speed"));
    assert!(identities.contains("sand_core::events::Absorption"));
    assert!(provider.iter().all(|item| {
        item.provider == "generated_event_markers"
            && item.kind == ReachableKind::Struct
            && item.members.is_empty()
    }));
}

#[test]
fn effect_provider_growth_follows_macro_body_and_invocation() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("effect-family.rs");
    fs::write(
        &source,
        r#"
macro_rules! vanilla_registry_enum {
    ($name:ident { $($variant:ident => $path:literal),+ }) => {
        pub enum $name { $($variant,)+ Custom(String), FutureOwned }
        impl $name {
            pub fn custom() -> Self { todo!() }
            pub fn newly_added() {}
        }
    };
}
vanilla_registry_enum! { Effect { Speed => "speed", Future => "future" } }
"#,
    )
    .unwrap();
    let provider = vanilla_registry_enum_provider(&source).unwrap();
    assert_eq!(provider.len(), 1);
    for member in [
        "Speed",
        "Future",
        "Custom",
        "FutureOwned",
        "custom",
        "newly_added",
    ] {
        assert!(
            provider[0].members.iter().any(|(name, _)| name == member),
            "missing {member}"
        );
    }
}
use std::collections::BTreeSet;
use std::fs;
