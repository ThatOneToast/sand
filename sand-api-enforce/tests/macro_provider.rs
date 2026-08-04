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
                    ("fmt".into(), ReachableKind::TraitMethod),
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
                    ("Err".into(), ReachableKind::AssociatedType),
                    ("as_resource_location".into(), ReachableKind::Method),
                    ("custom".into(), ReachableKind::Method),
                    ("fmt".into(), ReachableKind::TraitMethod),
                    ("from".into(), ReachableKind::TraitMethod),
                    ("from_str".into(), ReachableKind::TraitMethod),
                    ("minecraft".into(), ReachableKind::Method),
                    ("serialize".into(), ReachableKind::TraitMethod),
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
    assert!(
        provider.iter().all(|item| {
            item.members
                == [
                    ("new".into(), ReachableKind::Method),
                    ("typed".into(), ReachableKind::Method),
                ]
        }),
        "{provider:#?}"
    );
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
            .contains(&("Custom::0".into(), ReachableKind::Field))
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
        item.provider == "generated_event_markers" && item.kind == ReachableKind::Struct
    }));
    let survival = provider
        .iter()
        .find(|item| item.identity.ends_with("PlayerEnteredSurvivalEvent"))
        .unwrap();
    assert_eq!(
        survival.members,
        [("dispatch".into(), ReachableKind::TraitMethod)]
    );
    let speed = provider
        .iter()
        .find(|item| item.identity.ends_with("::Speed"))
        .unwrap();
    assert_eq!(
        speed.members,
        [
            ("CONDITION".into(), ReachableKind::AssociatedConst),
            ("EFFECT_ID".into(), ReachableKind::AssociatedConst),
            ("TRACKER_ID".into(), ReachableKind::AssociatedConst),
        ]
    );
}

#[test]
fn type_family_models_public_fields_and_every_associated_item_shape() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("resource_ref.rs");
    fs::write(
        &source,
        r#"
macro_rules! resource_ref {
    ($name:ident) => {
        pub struct $name { pub raw: String }
        impl $name {
            pub const DEFAULT: usize = 0;
            pub type View = String;
            pub fn view(&self) {}
        }
        impl ExampleTrait for $name {
            type Output = String;
            const ENABLED: bool = true;
            fn execute(&self) {}
        }
    };
}
resource_ref!(FixtureRef);
"#,
    )
    .unwrap();
    let provider = resource_ref_provider(&source).unwrap();
    assert_eq!(
        provider[0].members,
        [
            ("DEFAULT".into(), ReachableKind::AssociatedConst),
            ("ENABLED".into(), ReachableKind::AssociatedConst),
            ("Output".into(), ReachableKind::AssociatedType),
            ("View".into(), ReachableKind::AssociatedType),
            ("execute".into(), ReachableKind::TraitMethod),
            ("raw".into(), ReachableKind::Field),
            ("view".into(), ReachableKind::Method),
        ]
    );
}

#[test]
fn type_family_rejects_an_unmodeled_public_top_level_item() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("resource_ref.rs");
    fs::write(
        &source,
        r#"
macro_rules! resource_ref {
    ($name:ident) => {
        pub struct $name;
        impl $name { pub fn new() {} }
        pub fn bypass() {}
    };
}

resource_ref!(FixtureRef);
"#,
    )
    .unwrap();
    let error = resource_ref_provider(&source).unwrap_err().to_string();
    assert!(
        error.contains("unsupported public top-level `fn`"),
        "{error}"
    );
}

#[test]
fn type_family_rejects_public_api_hidden_in_a_repetition() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("resource_ref.rs");
    fs::write(
        &source,
        r#"
macro_rules! resource_ref {
    ($name:ident, $($extra:ident),*) => {
        pub struct $name;
        impl $name { pub fn new() {} }
        $(pub struct $extra;)*
    };
}
resource_ref!(FixtureRef, HiddenOne, HiddenTwo);
"#,
    )
    .unwrap();
    let error = resource_ref_provider(&source).unwrap_err().to_string();
    assert!(error.contains("inside a repetition"), "{error}");
}

#[test]
fn event_family_growth_models_generated_inherent_members() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("events.rs");
    fs::write(
        &source,
        r#"
macro_rules! gamemode_transition {
    ($mode:literal, $tracker:literal, $enter:ident, $exit:ident) => {
        pub struct $enter;
        impl $enter { pub const MODE: &'static str = $mode; }
        pub struct $exit;
        impl $exit { pub fn tracker() {} }
    };
}
macro_rules! status_effect_marker {
    ($ty:ident, $id:literal, $tracker:literal, $condition:literal) => {
        pub struct $ty;
        impl $ty { pub type Marker = (); }
    };
}
gamemode_transition!("survival", "tracker", Enter, Exit);
status_effect_marker!(Speed, "speed", "tracker", "condition");
"#,
    )
    .unwrap();
    let provider = event_generated_type_provider(&source).unwrap();
    assert_eq!(
        provider
            .iter()
            .find(|item| item.identity.ends_with("::Enter"))
            .unwrap()
            .members,
        [("MODE".into(), ReachableKind::AssociatedConst)]
    );
    assert_eq!(
        provider
            .iter()
            .find(|item| item.identity.ends_with("::Exit"))
            .unwrap()
            .members,
        [("tracker".into(), ReachableKind::Method)]
    );
    assert_eq!(
        provider
            .iter()
            .find(|item| item.identity.ends_with("::Speed"))
            .unwrap()
            .members,
        [("Marker".into(), ReachableKind::AssociatedType)]
    );
}

#[test]
fn event_family_rejects_an_extra_public_generated_type() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("events.rs");
    fs::write(
        &source,
        r#"
macro_rules! gamemode_transition {
    ($mode:literal, $tracker:literal, $enter:ident, $exit:ident) => {
        pub struct $enter;
        pub struct $exit;
        pub struct Untracked;
    };
}
macro_rules! status_effect_marker {
    ($ty:ident, $id:literal, $tracker:literal, $condition:literal) => {
        pub struct $ty;
    };
}
gamemode_transition!("survival", "tracker", Enter, Exit);
status_effect_marker!(Speed, "speed", "tracker", "condition");
"#,
    )
    .unwrap();
    let error = event_generated_type_provider(&source)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unsupported public struct `Untracked`"),
        "{error}"
    );
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
        pub enum $name { $($variant,)+ Custom(String), FutureOwned = DISCRIMINANT }
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
    assert!(
        !provider[0]
            .members
            .iter()
            .any(|(name, _)| name == "DISCRIMINANT")
    );
}
use std::collections::BTreeSet;
use std::fs;
