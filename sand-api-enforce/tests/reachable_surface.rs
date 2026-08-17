use std::collections::BTreeSet;
use std::fs;

use sand_api_enforce::{
    CfgSet, ContractIdentity, GeneratedApi, GeneratedProducer, InertItemMacroClassification,
    ReachabilityError, ReachableKind, SourceCrate, SurfaceGraph, audit_reachable_surface,
};

fn fixture(features: &[&str]) -> (tempfile::TempDir, Vec<sand_api_enforce::ReachableApi>) {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core_lib");
    let facade = directory.path().join("facade");
    fs::create_dir_all(&core).unwrap();
    fs::create_dir_all(&facade).unwrap();
    fs::write(
        core.join("lib.rs"),
        r#"
            pub mod model;
            pub type QualifiedAlias = model::Thing;
            pub mod aliases {
                use crate::model::Thing as Imported;
                use crate::model as domain;
                pub type ImportedAlias = Imported;
                pub type ModuleAlias = domain::Thing;
                pub type SuperAlias = super::model::Thing;
                pub type CrateAlias = crate::model::Thing;
            }
            mod extensions {
                use crate::model::Thing as Imported;
                type PrivateAlias = Imported;
                impl PrivateAlias {
                    pub fn from_private_alias() {}
                    #[cfg(feature = "extras")] pub fn feature_method() {}
                }
            }
            #[path = "alternate.rs"] pub mod path_module;
            #[cfg_attr(feature = "extras", path = "conditional_alt.rs")]
            pub mod conditional_path;
            #[cfg_attr(feature = "extras", doc(hidden))]
            pub mod conditional_visibility { pub struct ConditionallyVisible; }
            mod macros { #[macro_export] macro_rules! root_command { () => {} } }
            mod private_source { pub struct Forwarded; impl Forwarded { pub fn create() -> Self { Self } } }
            pub use private_source::Forwarded as RootForwarded;
            pub mod implementation_only { pub struct PublicButUnsupported; }
            #[doc(hidden)] pub mod __private { pub struct CompilerWire; }
        "#,
    )
    .unwrap();
    fs::write(
        core.join("model.rs"),
        r#"
            pub struct Thing {
                pub value: u32,
                #[cfg(feature = "extras")] pub extra: u32,
                hidden: u32,
            }
            impl Thing {
                pub fn new(value: u32) -> Self { Self { value, hidden: 0 } }
                pub const DEFAULT: u32 = 1;
                fn implementation_detail(&self) -> u32 { self.hidden }
            }
            pub enum Mode {
                Fast,
                Slow { code: u32, #[cfg(feature = "extras")] detail: u32 },
            }
            pub union Bits { pub integer: u32, pub float: f32 }
            pub static CURRENT: u32 = 1;
            pub trait ContractTrait {
                type Output;
                const VERSION: u32;
                fn render(&self) -> Self::Output;
                #[cfg(feature = "extras")] fn render_extra(&self);
            }
            pub type Alias = Thing;
            pub struct r#type;
            pub fn helper() {}
            #[cfg(feature = "extras")] pub fn feature_helper() {}
            #[cfg(unix)] pub fn unix_helper() {}
            #[cfg(target_os = "linux")] pub fn linux_helper() {}
            #[cfg(target_arch = "wasm32")] pub fn wasm_helper() {}
            #[doc(hidden)] pub struct GeneratedWire;
            pub(crate) struct CrateOnly;
        "#,
    )
    .unwrap();
    fs::write(
        core.join("alternate.rs"),
        "pub struct PathThing; impl PathThing { pub fn via_path() {} }",
    )
    .unwrap();
    fs::write(
        core.join("conditional_path.rs"),
        "pub struct DefaultPathThing;",
    )
    .unwrap();
    fs::write(
        core.join("conditional_alt.rs"),
        "pub struct AlternatePathThing;",
    )
    .unwrap();
    fs::write(
        facade.join("lib.rs"),
        r#"
            extern crate core_lib as implementation;
            pub type ExternAlias = implementation::model::Thing;
            pub type GeneratedAlias = core_lib::generated::GeneratedBuilder;
            pub use core_lib::model::{self, Thing as Builder, ContractTrait, Alias};
            pub use core_lib::{QualifiedAlias, aliases::*};
            pub use core_lib::model::Mode::*;
            pub use core_lib::model::r#type as KeywordType;
            pub use core_lib::RootForwarded;
            pub use implementation::path_module::PathThing;
            pub use implementation::conditional_path::*;
            pub use implementation::conditional_visibility::ConditionallyVisible;
            pub use core_lib::root_command;
            pub mod prelude { pub use core_lib::model::*; }
            pub use core_lib::generated::GeneratedBuilder;
            #[doc(hidden)] pub mod __private { pub struct FacadeWire; }
        "#,
    )
    .unwrap();

    let graph = SurfaceGraph::load_with_cfg(
        [
            SourceCrate {
                name: "core_lib".into(),
                root: core.join("lib.rs"),
            },
            SourceCrate {
                name: "facade".into(),
                root: facade.join("lib.rs"),
            },
        ],
        CfgSet {
            features: features
                .iter()
                .map(|feature| (*feature).to_owned())
                .collect(),
            flags: [("unix".to_owned(), true)].into_iter().collect(),
            key_values: [
                ("target_os".to_owned(), BTreeSet::from(["linux".to_owned()])),
                (
                    "target_arch".to_owned(),
                    BTreeSet::from(["x86_64".to_owned()]),
                ),
            ]
            .into_iter()
            .collect(),
        },
        [GeneratedApi {
            identity: "core_lib::generated::GeneratedBuilder".into(),
            provider: "generated_commands".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: vec![("build".into(), ReachableKind::Method)],
            excluded: false,
        }],
    )
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    (directory, reachable)
}

fn item<'a>(
    reachable: &'a [sand_api_enforce::ReachableApi],
    identity: &str,
) -> &'a sand_api_enforce::ReachableApi {
    reachable
        .iter()
        .find(|item| item.identity == identity)
        .unwrap_or_else(|| panic!("missing {identity}; found {reachable:#?}"))
}

fn contracts_for(reachable: &[sand_api_enforce::ReachableApi]) -> Vec<ContractIdentity> {
    reachable
        .iter()
        .map(|item| {
            let canonical_path = item.paths.iter().next().unwrap().clone();
            ContractIdentity {
                identity: item.identity.clone(),
                aliases: item
                    .paths
                    .iter()
                    .filter(|path| *path != &canonical_path)
                    .cloned()
                    .collect(),
                canonical_path,
            }
        })
        .collect()
}

fn item_macro_graph(
    source: &str,
    generated: impl IntoIterator<Item = GeneratedApi>,
) -> (tempfile::TempDir, SurfaceGraph) {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade");
    fs::create_dir_all(&facade).unwrap();
    fs::write(facade.join("lib.rs"), source).unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.join("lib.rs"),
        }],
        [],
        generated,
    )
    .unwrap();
    (directory, graph)
}

#[test]
fn reachable_item_macro_fails_closed_until_its_exact_provider_is_bound() {
    let (_directory, graph) = item_macro_graph(
        r#"
            macro_rules! family { ($name:ident) => { pub struct $name; }; }
            family!(Generated);
        "#,
        [GeneratedApi {
            identity: "facade::Generated".into(),
            provider: "fixture_family".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: Vec::new(),
            excluded: false,
        }],
    );

    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnboundItemMacro {
            module,
            macro_path,
            ..
        }) if module == "facade" && macro_path == "family"
    ));
}

#[test]
fn item_macro_binding_requires_an_invocation_and_provider_output_below_the_module() {
    let (_directory, graph) = item_macro_graph(
        r#"
            macro_rules! family { ($name:ident) => { pub struct $name; }; }
            family!(Generated);
        "#,
        [GeneratedApi {
            identity: "elsewhere::Generated".into(),
            provider: "fixture_family".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: Vec::new(),
            excluded: false,
        }],
    );

    assert!(matches!(
        graph.bind_item_macro_provider("facade", "family", "fixture_family"),
        Err(ReachabilityError::InvalidItemMacroProvider {
            module,
            macro_path,
            provider,
            invocations: 1,
        }) if module == "facade" && macro_path == "family" && provider == "fixture_family"
    ));

    let (_directory, graph) = item_macro_graph(
        "pub struct Handwritten;",
        [GeneratedApi {
            identity: "facade::Generated".into(),
            provider: "fixture_family".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: Vec::new(),
            excluded: false,
        }],
    );
    assert!(matches!(
        graph.bind_item_macro_provider("facade", "family", "fixture_family"),
        Err(ReachabilityError::InvalidItemMacroProvider { invocations: 0, .. })
    ));
}

#[test]
fn exact_item_macro_binding_exposes_provider_owned_output() {
    let (_directory, graph) = item_macro_graph(
        r#"
            macro_rules! family { ($name:ident) => { pub struct $name; }; }
            family!(Generated);
        "#,
        [GeneratedApi {
            identity: "facade::Generated".into(),
            provider: "fixture_family".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: vec![("create".into(), ReachableKind::Method)],
            excluded: false,
        }],
    );
    let reachable = graph
        .bind_item_macro_provider("facade", "family", "fixture_family")
        .unwrap()
        .reachable_from("facade")
        .unwrap();

    assert_eq!(
        item(&reachable, "facade::Generated").kind,
        ReachableKind::Struct
    );
    assert_eq!(
        item(&reachable, "facade::Generated::create").origin,
        sand_api_enforce::ReachableOrigin::Generator("fixture_family".into())
    );
}

#[test]
fn item_macro_binding_does_not_cover_the_same_spelling_in_another_module() {
    let (_directory, graph) = item_macro_graph(
        r#"
            macro_rules! family { ($name:ident) => { pub struct $name; }; }
            family!(RootGenerated);
            pub mod nested {
                macro_rules! family { ($name:ident) => { pub struct $name; }; }
                family!(NestedGenerated);
            }
        "#,
        [
            GeneratedApi {
                identity: "facade::RootGenerated".into(),
                provider: "fixture_family".into(),
                producer: None,
                kind: ReachableKind::Struct,
                members: Vec::new(),
                excluded: false,
            },
            GeneratedApi {
                identity: "facade::nested::NestedGenerated".into(),
                provider: "fixture_family".into(),
                producer: None,
                kind: ReachableKind::Struct,
                members: Vec::new(),
                excluded: false,
            },
        ],
    );
    let error = graph
        .bind_item_macro_provider("facade", "family", "fixture_family")
        .unwrap()
        .reachable_from("facade")
        .unwrap_err();

    assert!(matches!(
        error,
        ReachabilityError::UnboundItemMacro {
            module,
            macro_path,
            ..
        } if module == "facade::nested" && macro_path == "family"
    ));
}

#[test]
fn local_inert_item_macro_requires_an_exact_invocation_and_audited_definition() {
    let (_directory, graph) = item_macro_graph(
        r#"
            trait Marker {}
            macro_rules! marker_family {
                ($name:ident) => {
                    struct PrivateHelper;
                    impl Marker for $name {}
                };
            }
            struct Subject;
            marker_family!(Subject);
        "#,
        [],
    );
    let reachable = graph
        .bind_inert_item_macro(
            "facade",
            "marker_family",
            InertItemMacroClassification::LocalTraitImplOnly,
        )
        .unwrap()
        .reachable_from("facade")
        .unwrap();
    assert!(reachable.is_empty());

    let (_directory, graph) = item_macro_graph("macro_rules! marker_family { () => {} }", []);
    assert!(matches!(
        graph.bind_inert_item_macro(
            "facade",
            "marker_family",
            InertItemMacroClassification::LocalTraitImplOnly,
        ),
        Err(ReachabilityError::InvalidInertItemMacro { invocations: 0, .. })
    ));
}

#[test]
fn inert_item_macro_binding_is_not_another_modules_exemption() {
    let (_directory, graph) = item_macro_graph(
        r#"
            macro_rules! marker_family { () => { struct RootPrivate; }; }
            marker_family!();
            pub mod nested {
                macro_rules! marker_family { () => { struct NestedPrivate; }; }
                marker_family!();
            }
        "#,
        [],
    );
    let error = graph
        .bind_inert_item_macro(
            "facade",
            "marker_family",
            InertItemMacroClassification::LocalTraitImplOnly,
        )
        .unwrap()
        .reachable_from("facade")
        .unwrap_err();
    assert!(matches!(
        error,
        ReachabilityError::UnboundItemMacro { module, macro_path, .. }
            if module == "facade::nested" && macro_path == "marker_family"
    ));
}

#[test]
fn inert_transcriber_rejects_public_items_and_inherent_members() {
    for (source, expected) in [
        (
            "macro_rules! family { () => { pub struct Escaped; }; } family!();",
            "public declaration",
        ),
        (
            "struct Subject; macro_rules! family { () => { impl Subject { pub fn escaped() {} } }; } family!();",
            "inherent impl",
        ),
    ] {
        let (_directory, graph) = item_macro_graph(source, []);
        let error = graph
            .bind_inert_item_macro(
                "facade",
                "family",
                InertItemMacroClassification::LocalTraitImplOnly,
            )
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn inert_transcriber_rejects_nested_item_macros_and_repetition() {
    for (source, expected) in [
        (
            "trait Marker {} struct Subject; macro_rules! family { () => { impl Marker for Subject { helper!(); } }; } family!();",
            "unmodeled `helper!`",
        ),
        (
            "macro_rules! family { ($($name:ident),*) => { $(struct $name;)* }; } family!(One, Two);",
            "unaudited repetition",
        ),
    ] {
        let (_directory, graph) = item_macro_graph(source, []);
        let error = graph
            .bind_inert_item_macro(
                "facade",
                "family",
                InertItemMacroClassification::LocalTraitImplOnly,
            )
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn external_compiler_wiring_classifications_are_path_specific() {
    let (_directory, graph) = item_macro_graph("inventory::collect!(Descriptor);", []);
    graph
        .bind_inert_item_macro(
            "facade",
            "inventory::collect",
            InertItemMacroClassification::InventoryCollectionWiring,
        )
        .unwrap()
        .reachable_from("facade")
        .unwrap();

    let (_directory, graph) = item_macro_graph("other::collect!(Descriptor);", []);
    let error = graph
        .bind_inert_item_macro(
            "facade",
            "other::collect",
            InertItemMacroClassification::InventoryCollectionWiring,
        )
        .err()
        .unwrap()
        .to_string();
    assert!(
        error.contains("valid only for `inventory::collect!`"),
        "{error}"
    );

    let (_directory, graph) = item_macro_graph("thread_local! { static VALUE: u8 = 0; }", []);
    graph
        .bind_inert_item_macro(
            "facade",
            "thread_local",
            InertItemMacroClassification::ThreadLocalStorageWiring,
        )
        .unwrap()
        .reachable_from("facade")
        .unwrap();
}

#[test]
fn external_inert_bindings_structurally_audit_every_invocation_payload() {
    for source in [
        "thread_local! { pub static ESCAPED: u8 = 0; }",
        "thread_local! { static VALUE: u8 = 0 }",
    ] {
        let (_directory, graph) = item_macro_graph(source, []);
        let error = graph
            .bind_inert_item_macro(
                "facade",
                "thread_local",
                InertItemMacroClassification::ThreadLocalStorageWiring,
            )
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("thread_local!"), "{error}");
    }

    let (_directory, graph) =
        item_macro_graph("inventory::collect!(Descriptor; pub struct Escaped;);", []);
    let error = graph
        .bind_inert_item_macro(
            "facade",
            "inventory::collect",
            InertItemMacroClassification::InventoryCollectionWiring,
        )
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("exactly one type"), "{error}");
}

#[test]
fn texture_only_resourcepack_bindings_are_audited_without_generated_api() {
    let (_directory, graph) = item_macro_graph(
        r#"texture!(id = "fixture:item/icon", path = "assets/icon.png");"#,
        [],
    );
    let reachable = graph
        .bind_inert_item_macro(
            "facade",
            "texture",
            InertItemMacroClassification::ResourcepackTextureRegistration,
        )
        .unwrap()
        .reachable_from("facade")
        .unwrap();
    assert!(reachable.is_empty());

    for source in [
        r#"texture!(id = "missing-namespace", path = "assets/icon.png");"#,
        r#"texture!(id = "fixture:item/icon");"#,
    ] {
        let (_directory, graph) = item_macro_graph(source, []);
        let error = graph
            .bind_inert_item_macro(
                "facade",
                "texture",
                InertItemMacroClassification::ResourcepackTextureRegistration,
            )
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("texture!"), "{error}");
    }
}

#[test]
fn associated_item_macros_fail_closed_and_require_exact_owner_output() {
    for source in [
        "pub struct Subject; impl Subject { generated_members!(); }",
        "pub trait Subject { generated_members!(); }",
    ] {
        let (_directory, graph) = item_macro_graph(
            source,
            [GeneratedApi {
                identity: "facade::Subject::generated".into(),
                provider: "associated_fixture".into(),
                producer: None,
                kind: ReachableKind::Method,
                members: Vec::new(),
                excluded: false,
            }],
        );
        assert!(matches!(
            graph.reachable_from("facade"),
            Err(ReachabilityError::UnboundAssociatedItemMacro {
                owner,
                macro_path,
                ..
            }) if owner == "facade::Subject" && macro_path == "generated_members"
        ));
    }

    let (_directory, graph) = item_macro_graph(
        "pub struct Subject; impl Subject { generated_members!(); }",
        [GeneratedApi {
            identity: "facade::Other::generated".into(),
            provider: "associated_fixture".into(),
            producer: None,
            kind: ReachableKind::Method,
            members: Vec::new(),
            excluded: false,
        }],
    );
    assert!(matches!(
        graph.bind_associated_item_macro_provider(
            "facade::Subject",
            "generated_members",
            "associated_fixture",
        ),
        Err(ReachabilityError::InvalidAssociatedItemMacroProvider {
            owner,
            invocations: 1,
            ..
        }) if owner == "facade::Subject"
    ));

    let (_directory, graph) = item_macro_graph(
        "pub struct Subject; impl Subject { generated_members!(); }",
        [GeneratedApi {
            identity: "facade::Subject::generated".into(),
            provider: "associated_fixture".into(),
            producer: None,
            kind: ReachableKind::Method,
            members: Vec::new(),
            excluded: false,
        }],
    );
    let reachable = graph
        .bind_associated_item_macro_provider(
            "facade::Subject",
            "generated_members",
            "associated_fixture",
        )
        .unwrap()
        .reachable_from("facade")
        .unwrap();
    assert_eq!(
        item(&reachable, "facade::Subject::generated").kind,
        ReachableKind::Method
    );
}

#[test]
fn duplicate_local_macro_definitions_are_rejected_instead_of_overwritten() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("lib.rs");
    fs::write(
        &root,
        "macro_rules! family { () => {}; } macro_rules! family { ($x:ident) => {}; }",
    )
    .unwrap();
    let error = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root,
        }],
        [],
        [],
    )
    .err()
    .unwrap()
    .to_string();
    assert!(
        error.contains("duplicate or shadowed macro_rules"),
        "{error}"
    );
}

#[test]
fn reachable_foreign_modules_fail_closed_but_unexposed_ones_do_not() {
    let (_directory, graph) = item_macro_graph(
        "pub mod ffi { unsafe extern \"C\" { pub fn escaped(); } }",
        [],
    );
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnsupportedReachableSyntax {
            module,
            syntax: "extern block",
            ..
        }) if module == "facade::ffi"
    ));

    let (_directory, graph) = item_macro_graph(
        "mod implementation { unsafe extern \"C\" { pub fn internal_only(); } } pub struct Supported;",
        [],
    );
    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "facade::Supported").kind,
        ReachableKind::Struct
    );
}

#[test]
fn exported_macro_declaration_attributes_are_classified_and_cannot_be_spoofed() {
    let (_directory, graph) = item_macro_graph(
        r#"
            #[unknown_contract_attribute]
            #[macro_export]
            macro_rules! exported { () => {}; }
        "#,
        [],
    );
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnclassifiedApiMacro {
            owner,
            name,
            form: "attribute",
            ..
        }) if owner == "facade::exported" && name == "unknown_contract_attribute"
    ));

    let (_directory, graph) = item_macro_graph(
        r#"
            use impostor::api;
            #[api]
            #[macro_export]
            macro_rules! exported { () => {}; }
        "#,
        [],
    );
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnclassifiedApiMacro {
            owner,
            name,
            form: "macro imported under an audited bare name",
            ..
        }) if owner == "facade::exported" && name == "api"
    ));
}

#[test]
fn extracts_explicit_and_glob_reexports_with_associated_surface() {
    let (_directory, reachable) = fixture(&[]);

    let thing = item(&reachable, "core_lib::model::Thing");
    assert_eq!(
        thing.paths,
        BTreeSet::from([
            "facade::Builder".into(),
            "facade::model::Thing".into(),
            "facade::prelude::Thing".into(),
        ])
    );
    assert_eq!(
        item(&reachable, "core_lib::model::Thing::value").kind,
        ReachableKind::Field
    );
    assert_eq!(
        item(&reachable, "core_lib::model::Mode::Fast").kind,
        ReachableKind::Variant
    );
    assert_eq!(
        item(&reachable, "core_lib::model::ContractTrait::render").kind,
        ReachableKind::TraitMethod
    );
    assert_eq!(
        item(&reachable, "core_lib::model::ContractTrait::Output").kind,
        ReachableKind::AssociatedType
    );
    assert_eq!(
        item(&reachable, "core_lib::model::Alias").kind,
        ReachableKind::TypeAlias
    );
    assert_eq!(
        item(&reachable, "core_lib::model::Thing::new").paths,
        BTreeSet::from([
            "facade::Alias::new".into(),
            "facade::Builder::new".into(),
            "facade::CrateAlias::new".into(),
            "facade::ExternAlias::new".into(),
            "facade::ImportedAlias::new".into(),
            "facade::ModuleAlias::new".into(),
            "facade::QualifiedAlias::new".into(),
            "facade::SuperAlias::new".into(),
            "facade::model::Alias::new".into(),
            "facade::model::Thing::new".into(),
            "facade::prelude::Alias::new".into(),
            "facade::prelude::Thing::new".into(),
        ])
    );
    assert_eq!(
        item(&reachable, "core_lib::private_source::Forwarded::create").paths,
        BTreeSet::from(["facade::RootForwarded::create".into()])
    );
}

#[test]
fn feature_selection_and_narrow_exclusions_are_machine_visible() {
    let (_directory, without_feature) = fixture(&[]);
    assert!(
        !without_feature
            .iter()
            .any(|item| item.identity.ends_with("feature_helper"))
    );
    // Documentation visibility is not an enforcement exclusion. A reachable
    // hidden item remains part of the supported surface unless it lives under
    // the explicit `__private` boundary.
    assert!(
        without_feature
            .iter()
            .any(|item| item.identity.contains("GeneratedWire"))
    );
    assert!(
        !without_feature
            .iter()
            .any(|item| item.identity.contains("CompilerWire"))
    );
    assert!(
        !without_feature
            .iter()
            .any(|item| item.identity.contains("CrateOnly"))
    );

    let (_directory, with_feature) = fixture(&["extras"]);
    assert!(
        with_feature
            .iter()
            .any(|item| item.identity == "core_lib::model::feature_helper")
    );
}

#[test]
fn cfg_attr_doc_hidden_cannot_remove_surface_and_path_still_uses_cfg_set() {
    let (_directory, without_feature) = fixture(&[]);
    assert!(
        without_feature.iter().any(|item| {
            item.identity == "core_lib::conditional_visibility::ConditionallyVisible"
        })
    );
    assert!(
        without_feature
            .iter()
            .any(|item| item.identity == "core_lib::conditional_path::DefaultPathThing")
    );
    assert!(
        !without_feature
            .iter()
            .any(|item| { item.identity == "core_lib::conditional_path::AlternatePathThing" })
    );

    let (_directory, with_feature) = fixture(&["extras"]);
    assert!(
        with_feature.iter().any(|item| {
            item.identity == "core_lib::conditional_visibility::ConditionallyVisible"
        })
    );
    assert!(
        with_feature
            .iter()
            .any(|item| item.identity == "core_lib::conditional_path::AlternatePathThing")
    );
    assert!(
        !with_feature
            .iter()
            .any(|item| item.identity == "core_lib::conditional_path::DefaultPathThing")
    );
}

#[test]
fn controlled_generator_provider_participates_in_the_same_audit() {
    let (_directory, reachable) = fixture(&[]);
    assert_eq!(
        item(&reachable, "core_lib::generated::GeneratedBuilder").paths,
        BTreeSet::from(["facade::GeneratedBuilder".into()])
    );
    assert_eq!(
        item(&reachable, "core_lib::generated::GeneratedBuilder::build").kind,
        ReachableKind::Method
    );
    assert_eq!(
        item(&reachable, "core_lib::generated::GeneratedBuilder").origin,
        sand_api_enforce::ReachableOrigin::Generator("generated_commands".into())
    );
}

#[test]
fn generated_children_of_empty_include_module_flow_through_glob_reexports() {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(
        &core,
        r#"
            pub mod cmd {
                mod _generated { include!(concat!(env!("OUT_DIR"), "/generated_commands.rs")); }
                pub use _generated::*;
            }
        "#,
    )
    .unwrap();
    fs::write(
        &facade,
        r#"
            pub use sand_core::cmd::*;
            pub use sand_core::cmd::Teleport as TeleportCommand;
        "#,
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "sand_core".into(),
                root: core,
            },
            SourceCrate {
                name: "sand".into(),
                root: facade,
            },
        ],
        [],
        [GeneratedApi {
            identity: "sand_core::cmd::_generated::Teleport".into(),
            provider: "generated_commands".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: vec![("to".into(), ReachableKind::Method)],
            excluded: false,
        }],
    )
    .unwrap()
    .bind_generated_include("sand_core::cmd::_generated", "generated_commands")
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();
    assert_eq!(
        item(&reachable, "sand_core::cmd::_generated::Teleport").paths,
        BTreeSet::from(["sand::Teleport".into(), "sand::TeleportCommand".into()])
    );
    assert_eq!(
        item(&reachable, "sand_core::cmd::_generated::Teleport::to").paths,
        BTreeSet::from([
            "sand::Teleport::to".into(),
            "sand::TeleportCommand::to".into(),
        ])
    );
}

#[test]
fn reachable_nonliteral_include_without_a_provider_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            pub mod generated {
                include!(concat!(env!("OUT_DIR"), "/untracked.rs"));
            }
        "#,
    )
    .unwrap();

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnboundInclude { module, expression, .. })
            if module == "facade::generated" && expression.contains("OUT_DIR")
    ));
}

#[test]
fn literal_include_is_parsed_as_source_without_a_provider() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(&facade, r#"pub mod included { include!("included.rs"); }"#).unwrap();
    fs::write(
        directory.path().join("included.rs"),
        "pub struct LiteralItem; impl LiteralItem { pub fn create() -> Self { Self } }",
    )
    .unwrap();

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "facade::included::LiteralItem").paths,
        BTreeSet::from(["facade::included::LiteralItem".into()])
    );
    assert_eq!(
        item(&reachable, "facade::included::LiteralItem::create").kind,
        ReachableKind::Method
    );
}

#[test]
fn generated_include_binding_must_name_a_provider_that_owns_the_module() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"pub mod generated { include!(concat!(env!("OUT_DIR"), "/generated.rs")); }"#,
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [GeneratedApi {
            identity: "facade::other::Generated".into(),
            provider: "named_provider".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: vec![],
            excluded: false,
        }],
    )
    .unwrap();
    assert!(matches!(
        graph.bind_generated_include("facade::generated", "named_provider"),
        Err(ReachabilityError::InvalidIncludeProvider {
            module,
            provider,
            dynamic_includes: 1,
        })
            if module == "facade::generated" && provider == "named_provider"
    ));
}

#[test]
fn placeholder_include_binding_requires_one_include_and_zero_declarations() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"pub mod generated { include!(concat!(env!("OUT_DIR"), "/generated.rs")); }"#,
    )
    .unwrap();
    let empty = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        [],
        [],
    )
    .unwrap()
    .bind_placeholder_generated_include("facade::generated", "named_provider")
    .unwrap();
    empty.reachable_from("facade").unwrap();

    let populated = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [GeneratedApi {
            identity: "facade::generated::Generated".into(),
            provider: "named_provider".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: vec![],
            excluded: false,
        }],
    )
    .unwrap();
    assert!(matches!(
        populated.bind_placeholder_generated_include("facade::generated", "named_provider"),
        Err(ReachabilityError::InvalidPlaceholderIncludeProvider {
            dynamic_includes: 1,
            generated_declarations: 1,
            ..
        })
    ));
}

#[test]
fn missing_contract_alias_drift_and_duplicate_canonical_paths_fail() {
    let (_directory, reachable) = fixture(&[]);
    let mut contracts = contracts_for(&reachable);
    audit_reachable_surface(&reachable, &contracts).unwrap();

    let missing = contracts.pop().unwrap();
    let errors = audit_reachable_surface(&reachable, &contracts).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        ReachabilityError::MissingContract { identity, .. } if identity == &missing.identity
    )));

    let mut contracts = contracts_for(&reachable);
    let aliased = contracts
        .iter_mut()
        .find(|contract| !contract.aliases.is_empty())
        .unwrap();
    aliased.aliases.clear();
    assert!(
        audit_reachable_surface(&reachable, &contracts)
            .unwrap_err()
            .iter()
            .any(|error| { matches!(error, ReachabilityError::AliasSetMismatch { .. }) })
    );

    let mut contracts = contracts_for(&reachable);
    let duplicate = contracts[0].canonical_path.clone();
    contracts[1].canonical_path = duplicate;
    assert!(
        audit_reachable_surface(&reachable, &contracts)
            .unwrap_err()
            .iter()
            .any(|error| { matches!(error, ReachabilityError::DuplicateCanonicalPath(_)) })
    );

    let mut contracts = contracts_for(&reachable);
    contracts.push(contracts[0].clone());
    assert!(
        audit_reachable_surface(&reachable, &contracts)
            .unwrap_err()
            .iter()
            .any(|error| matches!(error, ReachabilityError::DuplicateContractIdentity(_)))
    );
}

#[test]
fn member_cfg_static_union_variant_fields_external_impls_and_grouped_exports_are_complete() {
    let (_, without) = fixture(&[]);
    assert_eq!(
        item(&without, "core_lib::model::CURRENT").kind,
        ReachableKind::Static
    );
    assert_eq!(
        item(&without, "core_lib::model::Bits").kind,
        ReachableKind::Union
    );
    assert_eq!(
        item(&without, "core_lib::model::Mode::Slow::code").kind,
        ReachableKind::Field
    );
    assert!(
        item(&without, "core_lib::model::Mode::Fast")
            .paths
            .contains("facade::Fast")
    );
    assert!(
        item(&without, "core_lib::model")
            .paths
            .contains("facade::model")
    );
    assert!(
        item(&without, "core_lib::model::Thing::from_private_alias")
            .paths
            .contains("facade::Builder::from_private_alias")
    );
    assert!(!without.iter().any(|api| api.identity.ends_with("::extra")
        || api.identity.ends_with("::detail")
        || api.identity.ends_with("::render_extra")
        || api.identity.ends_with("::feature_method")));

    let (_, with) = fixture(&["extras"]);
    for identity in [
        "core_lib::model::Thing::extra",
        "core_lib::model::Mode::Slow::detail",
        "core_lib::model::ContractTrait::render_extra",
        "core_lib::model::Thing::feature_method",
    ] {
        item(&with, identity);
    }
}

#[test]
fn conflicting_source_and_generator_or_generator_providers_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(&core, "pub struct Thing;").unwrap();
    fs::write(&facade, "pub use core_lib::Thing;").unwrap();
    let crates = [
        SourceCrate {
            name: "core_lib".into(),
            root: core,
        },
        SourceCrate {
            name: "facade".into(),
            root: facade,
        },
    ];

    let graph = SurfaceGraph::load(
        crates.clone(),
        [],
        [GeneratedApi {
            identity: "core_lib::Thing".into(),
            provider: "bad_generator".into(),
            producer: None,
            kind: ReachableKind::Enum,
            members: vec![],
            excluded: false,
        }],
    )
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::ConflictingReachableDefinition { .. })
    ));

    let error = SurfaceGraph::load(
        crates,
        [],
        [
            GeneratedApi {
                identity: "core_lib::generated::Only".into(),
                provider: "first".into(),
                producer: None,
                kind: ReachableKind::Struct,
                members: vec![],
                excluded: false,
            },
            GeneratedApi {
                identity: "core_lib::generated::Only".into(),
                provider: "second".into(),
                producer: None,
                kind: ReachableKind::Struct,
                members: vec![],
                excluded: false,
            },
        ],
    )
    .err()
    .expect("duplicate generator identity must fail while constructing the graph");
    assert!(matches!(
        error,
        ReachabilityError::ConflictingReachableDefinition { .. }
    ));
}

#[test]
fn field_and_method_name_collision_has_distinct_identities_and_ambiguous_lookup_fails() {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(
        &core,
        r#"
            pub struct BlockPredicate { pub blocks: Vec<String> }
            impl BlockPredicate { pub fn blocks(&self) -> &[String] { &self.blocks } }
        "#,
    )
    .unwrap();
    fs::write(&facade, "pub use core_lib::BlockPredicate;").unwrap();
    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "core_lib".into(),
                root: core,
            },
            SourceCrate {
                name: "facade".into(),
                root: facade,
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    let field = item(&reachable, "core_lib::BlockPredicate::blocks#field");
    let method = item(&reachable, "core_lib::BlockPredicate::blocks#method");
    assert_eq!(field.kind, ReachableKind::Field);
    assert_eq!(method.kind, ReachableKind::Method);
    assert_eq!(field.paths, method.paths);
    assert_eq!(
        field.paths,
        BTreeSet::from(["facade::BlockPredicate::blocks".into()])
    );

    let errors = audit_reachable_surface(&reachable, &contracts_for(&reachable)).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        ReachabilityError::DuplicateCanonicalPath(path)
            if path == "facade::BlockPredicate::blocks"
    )));
}

#[test]
fn architecture_01_explicit_reexport_is_reachable() {
    let (_, api) = fixture(&[]);
    assert!(
        item(&api, "core_lib::model::Thing")
            .paths
            .contains("facade::Builder")
    );
}

#[test]
fn architecture_02_glob_reexport_is_reachable() {
    let (_, api) = fixture(&[]);
    assert!(
        item(&api, "core_lib::model::helper")
            .paths
            .contains("facade::prelude::helper")
    );
}

#[test]
fn architecture_03_inherent_method_is_reachable() {
    let (_, api) = fixture(&[]);
    assert_eq!(
        item(&api, "core_lib::model::Thing::new").kind,
        ReachableKind::Method
    );
}

#[test]
fn architecture_04_trait_associated_items_are_reachable() {
    let (_, api) = fixture(&[]);
    assert_eq!(
        item(&api, "core_lib::model::ContractTrait::render").kind,
        ReachableKind::TraitMethod
    );
    assert_eq!(
        item(&api, "core_lib::model::ContractTrait::Output").kind,
        ReachableKind::AssociatedType
    );
}

#[test]
fn architecture_05_fields_and_variants_are_reachable() {
    let (_, api) = fixture(&[]);
    assert_eq!(
        item(&api, "core_lib::model::Thing::value").kind,
        ReachableKind::Field
    );
    assert_eq!(
        item(&api, "core_lib::model::Mode::Fast").kind,
        ReachableKind::Variant
    );
}

#[test]
fn architecture_06_type_alias_members_share_underlying_identity() {
    let (_, api) = fixture(&[]);
    let method = item(&api, "core_lib::model::Thing::new");
    assert!(method.paths.contains("facade::Alias::new"));
    assert!(
        !api.iter()
            .any(|item| item.identity == "core_lib::model::Alias::new")
    );
}

#[test]
fn relative_imported_and_qualified_aliases_cannot_hide_inherent_members() {
    let (_, api) = fixture(&[]);
    let method = item(&api, "core_lib::model::Thing::new");
    for alias_path in [
        "facade::QualifiedAlias::new",
        "facade::ImportedAlias::new",
        "facade::ModuleAlias::new",
        "facade::SuperAlias::new",
        "facade::CrateAlias::new",
        "facade::ExternAlias::new",
    ] {
        assert!(
            method.paths.contains(alias_path),
            "missing alias member path {alias_path}; found {:?}",
            method.paths
        );
    }

    let contracts = contracts_for(&api)
        .into_iter()
        .filter(|contract| contract.identity != "core_lib::model::Thing::new")
        .collect::<Vec<_>>();
    let errors = audit_reachable_surface(&api, &contracts).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        ReachabilityError::MissingContract { identity, paths, .. }
            if identity == "core_lib::model::Thing::new"
                && paths.contains(&"facade::QualifiedAlias::new".into())
    )));
}

#[test]
fn alias_to_generated_type_preserves_generated_member_identity() {
    let (_, api) = fixture(&[]);
    let member = item(&api, "core_lib::generated::GeneratedBuilder::build");
    assert_eq!(
        member.origin,
        sand_api_enforce::ReachableOrigin::Generator("generated_commands".into())
    );
    assert!(member.paths.contains("facade::GeneratedAlias::build"));
    assert!(
        !api.iter()
            .any(|item| item.identity == "facade::GeneratedAlias::build")
    );
}

#[test]
fn alias_to_unmodeled_external_type_fails_closed() {
    let (_directory, graph) = item_macro_graph("pub type Escape = third_party::Thing;", []);
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnresolvedTypeAliasTarget { identity, target })
            if identity == "facade::Escape" && target == "third_party::Thing"
    ));
}

#[test]
fn standard_library_names_shadowed_by_local_imports_keep_underlying_members() {
    for shadow in ["std", "core", "alloc"] {
        let source = format!(
            r#"
                mod third_party {{
                    pub struct Thing;
                    impl Thing {{ pub fn undocumented() {{}} }}
                }}
                use crate::third_party as {shadow};
                pub type Escape = {shadow}::Thing;
            "#
        );
        let (_directory, graph) = item_macro_graph(&source, []);
        let api = graph.reachable_from("facade").unwrap();
        let method = item(&api, "facade::third_party::Thing::undocumented");
        assert!(method.paths.contains("facade::Escape::undocumented"));

        let contracts = contracts_for(&api)
            .into_iter()
            .filter(|contract| contract.identity != "facade::third_party::Thing::undocumented")
            .collect::<Vec<_>>();
        assert!(
            audit_reachable_surface(&api, &contracts)
                .unwrap_err()
                .iter()
                .any(|error| matches!(
                    error,
                    ReachabilityError::MissingContract { identity, .. }
                        if identity == "facade::third_party::Thing::undocumented"
                ))
        );
    }
}

#[test]
fn standard_library_names_shadowed_by_external_aliases_fail_closed() {
    for shadow in ["std", "core", "alloc"] {
        for declaration in [
            format!("extern crate third_party as {shadow};"),
            format!("use third_party as {shadow};"),
        ] {
            let source = format!("{declaration} pub type Escape = {shadow}::Thing;");
            let (_directory, graph) = item_macro_graph(&source, []);
            assert!(matches!(
                graph.reachable_from("facade"),
                Err(ReachabilityError::UnresolvedTypeAliasTarget { identity, target })
                    if identity == "facade::Escape" && target == format!("{shadow}::Thing")
            ));
        }
    }
}

#[test]
fn standard_library_names_exported_by_local_globs_keep_underlying_members() {
    for shadow in ["std", "core", "alloc"] {
        let source = format!(
            r#"
                mod source {{
                    pub mod {shadow} {{
                        pub struct Thing;
                        impl Thing {{ pub fn undocumented() {{}} }}
                    }}
                }}
                use crate::source::*;
                pub type Escape = {shadow}::Thing;
            "#
        );
        let (_directory, graph) = item_macro_graph(&source, []);
        let api = graph.reachable_from("facade").unwrap();
        let method = item(
            &api,
            &format!("facade::source::{shadow}::Thing::undocumented"),
        );
        assert!(method.paths.contains("facade::Escape::undocumented"));
    }
}

#[test]
fn unresolved_external_glob_cannot_spoof_the_standard_library_boundary() {
    let (_directory, graph) =
        item_macro_graph("use third_party::*; pub type Escape = std::Thing;", []);
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnresolvedTypeAliasTarget { identity, target })
            if identity == "facade::Escape" && target == "std::Thing"
    ));
}

#[test]
fn one_super_segment_resolves_exactly_one_lexical_parent() {
    let (_directory, graph) = item_macro_graph(
        r#"
            pub mod parent {
                pub struct Thing;
                impl Thing { pub fn new() -> Self { Self } }
                pub mod child { pub type Alias = super::Thing; }
            }
            pub use parent::child::Alias;
        "#,
        [],
    );
    let api = graph.reachable_from("facade").unwrap();
    assert!(
        item(&api, "facade::parent::Thing::new")
            .paths
            .contains("facade::Alias::new")
    );
}

#[test]
fn architecture_07_features_and_target_cfgs_select_exact_surface() {
    let (_, api) = fixture(&[]);
    assert!(
        api.iter()
            .any(|item| item.identity.ends_with("unix_helper"))
    );
    assert!(
        api.iter()
            .any(|item| item.identity.ends_with("linux_helper"))
    );
    assert!(
        !api.iter()
            .any(|item| item.identity.ends_with("wasm_helper"))
    );
    assert!(
        !api.iter()
            .any(|item| item.identity.ends_with("feature_helper"))
    );
    let (_, extras) = fixture(&["extras"]);
    assert!(
        extras
            .iter()
            .any(|item| item.identity.ends_with("feature_helper"))
    );
}

#[test]
fn architecture_08_generated_command_provider_is_reachable() {
    let (_, api) = fixture(&[]);
    assert_eq!(
        item(&api, "core_lib::generated::GeneratedBuilder").origin,
        sand_api_enforce::ReachableOrigin::Generator("generated_commands".into())
    );
}

#[test]
fn architecture_09_only_explicit_private_boundaries_are_absent() {
    let (_, api) = fixture(&[]);
    assert!(
        api.iter()
            .any(|item| item.identity.contains("GeneratedWire"))
    );
    assert!(
        !api.iter()
            .any(|item| item.identity.contains("CompilerWire"))
    );
    assert!(!api.iter().any(|item| item.identity.contains("CrateOnly")));
}

#[test]
fn architecture_10_public_implementation_item_outside_facade_is_absent() {
    let (_, api) = fixture(&[]);
    assert!(
        !api.iter()
            .any(|item| item.identity.contains("PublicButUnsupported"))
    );
}

#[test]
fn path_modules_raw_identifiers_extern_aliases_and_macro_root_are_resolved() {
    let (_, api) = fixture(&[]);
    assert!(
        item(&api, "core_lib::path_module::PathThing")
            .paths
            .contains("facade::PathThing")
    );
    assert!(
        item(&api, "core_lib::path_module::PathThing::via_path")
            .paths
            .contains("facade::PathThing::via_path")
    );
    assert!(
        item(&api, "core_lib::model::type")
            .paths
            .contains("facade::KeywordType")
    );
    assert!(
        item(&api, "core_lib::root_command")
            .paths
            .contains("facade::root_command")
    );
}

#[test]
fn proc_macro_exports_use_macro_namespace_names_kinds_and_aliases() {
    let directory = tempfile::tempdir().unwrap();
    let macros = directory.path().join("macros.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(
        &macros,
        r#"
            extern crate proc_macro;
            use proc_macro::TokenStream;
            #[proc_macro]
            pub fn make_command(input: TokenStream) -> TokenStream { input }
            #[proc_macro_attribute]
            pub fn tracked(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
            #[proc_macro_derive(EncodedState, attributes(state))]
            pub fn derive_encoded_state(input: TokenStream) -> TokenStream { input }
        "#,
    )
    .unwrap();
    fs::write(
        &facade,
        r#"
            pub use macro_crate::{make_command as command, tracked};
            pub use macro_crate::EncodedState;
            pub mod prelude {
                pub use macro_crate::EncodedState as StateEnum;
            }
        "#,
    )
    .unwrap();

    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "macro_crate".into(),
                root: macros,
            },
            SourceCrate {
                name: "facade".into(),
                root: facade,
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "macro_crate::make_command").kind,
        ReachableKind::FunctionLikeMacro
    );
    assert_eq!(
        item(&reachable, "macro_crate::tracked").kind,
        ReachableKind::AttributeMacro
    );
    let derive = item(&reachable, "macro_crate::EncodedState");
    assert_eq!(derive.kind, ReachableKind::DeriveMacro);
    assert_eq!(
        derive.paths,
        BTreeSet::from([
            "facade::EncodedState".into(),
            "facade::prelude::StateEnum".into(),
        ])
    );
    assert!(
        !reachable
            .iter()
            .any(|api| api.identity == "macro_crate::derive_encoded_state")
    );
}

#[test]
fn crate_paths_inside_reexported_external_modules_use_the_defining_crate() {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core");
    let facade = directory.path().join("facade.rs");
    fs::create_dir_all(&core).unwrap();
    fs::write(
        core.join("lib.rs"),
        "pub mod cmd; pub mod function { pub struct Function; }",
    )
    .unwrap();
    fs::write(core.join("cmd.rs"), "pub use crate::function::Function;").unwrap();
    fs::write(&facade, "pub use core_lib::cmd as command;").unwrap();

    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "core_lib".into(),
                root: core.join("lib.rs"),
            },
            SourceCrate {
                name: "facade".into(),
                root: facade,
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "core_lib::function::Function").paths,
        BTreeSet::from(["facade::command::Function".into()])
    );
    assert!(
        !reachable
            .iter()
            .any(|api| api.identity.starts_with("facade::function"))
    );
}

#[test]
fn loaded_external_crate_root_can_be_reexported_without_extern_crate_declaration() {
    let directory = tempfile::tempdir().unwrap();
    let resourcepack = directory.path().join("resourcepack.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(&resourcepack, "pub struct ResourcePack;").unwrap();
    fs::write(
        &facade,
        "pub use sand_resourcepack as resourcepack; pub use ::sand_resourcepack as absolute_resourcepack;",
    )
    .unwrap();

    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "sand_resourcepack".into(),
                root: resourcepack,
            },
            SourceCrate {
                name: "sand".into(),
                root: facade,
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();
    assert_eq!(
        item(&reachable, "sand_resourcepack").paths,
        BTreeSet::from([
            "sand::absolute_resourcepack".into(),
            "sand::resourcepack".into(),
        ])
    );
    assert_eq!(
        item(&reachable, "sand_resourcepack::ResourcePack").paths,
        BTreeSet::from([
            "sand::absolute_resourcepack::ResourcePack".into(),
            "sand::resourcepack::ResourcePack".into(),
        ])
    );
    assert!(!reachable.iter().any(|api| api.identity.contains("::::")));
}

#[test]
fn unresolved_reexport_inside_mapped_workspace_is_a_hard_error_with_edge_context() {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(&core, "pub struct Present;").unwrap();
    fs::write(&facade, "pub use core_lib::Missing;").unwrap();
    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "core_lib".into(),
                root: core,
            },
            SourceCrate {
                name: "facade".into(),
                root: facade.clone(),
            },
        ],
        [],
        [],
    )
    .unwrap();
    assert_eq!(
        graph.reachable_from("facade").unwrap_err(),
        ReachabilityError::UnresolvedReexport {
            source: facade,
            line: 1,
            facade_path: "facade::Missing".into(),
            target: "core_lib::Missing".into(),
        }
    );
}

#[test]
fn named_and_glob_reexports_from_mapped_source_remain_reachable() {
    let directory = tempfile::tempdir().unwrap();
    let dependency = directory.path().join("dependency.rs");
    let facade = directory.path().join("facade.rs");
    fs::write(
        &dependency,
        "pub struct Named; pub mod group { pub struct ThroughGlob; }",
    )
    .unwrap();
    fs::write(
        &facade,
        "pub use mapped_dependency::Named; pub use mapped_dependency::group::*;",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "mapped_dependency".into(),
                root: dependency,
            },
            SourceCrate {
                name: "facade".into(),
                root: facade,
            },
        ],
        [],
        [],
    )
    .unwrap();

    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "mapped_dependency::Named").paths,
        BTreeSet::from(["facade::Named".into()])
    );
    assert_eq!(
        item(&reachable, "mapped_dependency::group::ThroughGlob").paths,
        BTreeSet::from(["facade::ThroughGlob".into()])
    );
}

#[test]
fn named_reexport_from_unmapped_external_crate_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(&facade, "pub use external_dependency::Thing;").unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        [],
        [],
    )
    .unwrap();

    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnresolvedReexport {
            source,
            line: 1,
            facade_path,
            target,
        }) if source == facade
            && facade_path == "facade::Thing"
            && target == "facade::external_dependency::Thing"
    ));
}

#[test]
fn glob_reexport_from_unmapped_external_crate_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(&facade, "pub use external_dependency::*;").unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        [],
        [],
    )
    .unwrap();

    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnresolvedReexport {
            source,
            line: 1,
            facade_path,
            target,
        }) if source == facade
            && facade_path == "facade"
            && target == "facade::external_dependency::*"
    ));
}

#[test]
fn chained_reexport_from_unmapped_external_crate_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "mod bridge { pub use external_dependency::Thing; } pub use bridge::Thing;",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        [],
        [],
    )
    .unwrap();

    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnresolvedReexport {
            source,
            line: 1,
            facade_path,
            target,
        }) if source == facade
            && facade_path == "facade::Thing"
            && target == "facade::bridge::Thing"
    ));
}

#[test]
fn unknown_cfg_is_a_hard_error_instead_of_enabling_the_item() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(&facade, "#[cfg(unknown_platform)] pub fn accidental() {}").unwrap();
    assert!(matches!(
        SurfaceGraph::load_with_cfg(
            [SourceCrate { name: "facade".into(), root: facade }],
            CfgSet::default(),
            [],
        ),
        Err(ReachabilityError::UnknownCfg { predicate, .. }) if predicate == "unknown_platform"
    ));
}

#[test]
fn unknown_cfg_attr_predicate_is_a_hard_error_even_for_exclusion_attributes() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "#[cfg_attr(unknown_platform, doc(hidden))] pub struct Accidental;",
    )
    .unwrap();
    assert!(matches!(
        SurfaceGraph::load_with_cfg(
            [SourceCrate {
                name: "facade".into(),
                root: facade
            }],
            CfgSet::default(),
            [],
        ),
        Err(ReachabilityError::UnknownCfg { predicate, .. }) if predicate == "unknown_platform"
    ));
}

#[test]
fn public_impl_for_unmodeled_generated_owner_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "some_generator!(Generated); impl Generated { pub fn exposed() {} } pub use crate::Generated as Exported;",
    )
    .unwrap();
    assert!(matches!(
        SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        ),
        Err(ReachabilityError::UnresolvedImplOwner { module, self_type })
            if module == "facade" && self_type == "Generated"
    ));

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: directory.path().join("facade.rs"),
        }],
        [],
        [GeneratedApi {
            identity: "facade::Generated".into(),
            provider: "fixture_generator".into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: vec![],
            excluded: false,
        }],
    )
    .expect("a controlled provider models the generated owner");
    let reachable = graph
        .bind_item_macro_provider("facade", "some_generator", "fixture_generator")
        .unwrap()
        .reachable_from("facade")
        .unwrap();
    assert_eq!(
        item(&reachable, "facade::Generated::exposed").origin,
        sand_api_enforce::ReachableOrigin::Generator("fixture_generator".into())
    );
}

#[test]
fn reachable_api_producing_derive_requires_connected_provider() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "#[derive(Debug, SandStorage)] pub struct Storage { value: i32 }",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        [],
        [],
    )
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnboundApiProducer { producer, owner, .. })
            if producer == "SandStorage" && owner == "facade::Storage"
    ));

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [
            GeneratedApi {
                identity: "facade::Storage::SCHEMA".into(),
                provider: "storage_provider".into(),
                producer: Some(GeneratedProducer {
                    owner: "facade::Storage".into(),
                    name: "SandStorage".into(),
                }),
                kind: ReachableKind::AssociatedConst,
                members: vec![],
                excluded: false,
            },
            GeneratedApi {
                identity: "facade::Storage::value".into(),
                provider: "storage_provider".into(),
                producer: Some(GeneratedProducer {
                    owner: "facade::Storage".into(),
                    name: "SandStorage".into(),
                }),
                kind: ReachableKind::Method,
                members: vec![],
                excluded: false,
            },
        ],
    )
    .unwrap()
    .bind_api_producer("facade::Storage", "SandStorage", "storage_provider")
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "facade::Storage::SCHEMA").origin,
        sand_api_enforce::ReachableOrigin::Generator("storage_provider".into())
    );
}

#[test]
fn producer_binding_requires_the_exact_owner_output_set() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "#[derive(SandStorage)] pub struct Storage { value: i32 }",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [GeneratedApi {
            identity: "facade::Storage::SCHEMA".into(),
            provider: "partial_provider".into(),
            producer: Some(GeneratedProducer {
                owner: "facade::Storage".into(),
                name: "SandStorage".into(),
            }),
            kind: ReachableKind::AssociatedConst,
            members: vec![],
            excluded: false,
        }],
    )
    .unwrap();
    assert!(matches!(
        graph.bind_api_producer("facade::Storage", "SandStorage", "partial_provider"),
        Err(ReachabilityError::InvalidApiProducerProvider { expected, actual, .. })
            if expected == ["facade::Storage::SCHEMA [AssociatedConst]", "facade::Storage::value [Method]"]
                && actual == ["facade::Storage::SCHEMA [AssociatedConst]"]
    ));
}

#[test]
fn producer_binding_requires_exact_output_kinds() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "#[derive(SandStorage)] pub struct Storage { value: i32 }",
    )
    .unwrap();
    let producer = Some(GeneratedProducer {
        owner: "facade::Storage".into(),
        name: "SandStorage".into(),
    });
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [
            GeneratedApi {
                identity: "facade::Storage::SCHEMA".into(),
                provider: "wrong_kinds".into(),
                producer: producer.clone(),
                kind: ReachableKind::Method,
                members: vec![],
                excluded: false,
            },
            GeneratedApi {
                identity: "facade::Storage::value".into(),
                provider: "wrong_kinds".into(),
                producer,
                kind: ReachableKind::AssociatedConst,
                members: vec![],
                excluded: false,
            },
        ],
    )
    .unwrap();
    assert!(matches!(
        graph.bind_api_producer("facade::Storage", "SandStorage", "wrong_kinds"),
        Err(ReachabilityError::InvalidApiProducerProvider { .. })
    ));
}

#[test]
fn unknown_custom_derives_and_attributes_fail_closed() {
    for source in [
        "#[derive(UnknownExpansion)] pub struct Thing;",
        "#[unknown_expansion] pub struct Thing;",
        "use some_crate::UnknownExpansion as Alias; #[derive(Alias)] pub struct Thing;",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        let result = graph.reachable_from("facade");
        assert!(
            matches!(result, Err(ReachabilityError::UnclassifiedApiMacro { .. })),
            "qualified or aliased macro escaped: {source}: {result:?}"
        );
    }

    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "mod implementation { #[derive(InternalMacro)] pub struct Internal; } pub struct Public;",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    assert!(graph.reachable_from("facade").is_ok());
}

#[test]
fn bare_custom_attributes_are_not_trusted_without_their_derive_context() {
    for source in [
        "#[serde(rename_all = \"snake_case\")] pub struct Thing;",
        "#[derive(Debug)] #[command(name = \"spoofed\")] pub struct Thing;",
        "#[derive(Debug)] pub enum Thing { #[value(name = \"spoofed\")] Value }",
        "#[derive(Debug)] pub struct Thing { #[state(default = 1)] pub value: i32 }",
        "#[derive(Serialize)] pub struct Thing { #[error(transparent)] pub value: i32 }",
        "#[derive(Error)] pub enum Thing { Bad(#[serde(skip)] i32) }",
        "use evil::serde; #[derive(Serialize)] #[serde(rename_all = \"snake_case\")] pub struct Thing;",
        "use evil::error; #[derive(Error)] pub enum Thing { #[error(\"bad\")] Bad }",
        "#[tick] pub fn thing() {}",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        let result = graph.reachable_from("facade");
        assert!(
            matches!(result, Err(ReachabilityError::UnclassifiedApiMacro { .. })),
            "bare custom attribute escaped without its helper derive: {source}: {result:?}"
        );
    }
}

#[test]
fn real_derive_helpers_are_accepted_only_on_the_forms_the_derive_owns() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "snake_case")]
            pub struct Wire {
                #[serde(default)]
                pub value: i32,
            }

            pub struct Cause;
            #[derive(Error)]
            pub enum Problem {
                #[error("bad input")]
                Bad(#[from] Cause),
            }

            #[derive(Parser)]
            #[command(name = "sand")]
            pub struct Cli {
                #[arg(long)]
                pub verbose: bool,
            }

            #[derive(ValueEnum)]
            pub enum Mode {
                #[value(name = "fast")]
                Fast,
            }
        "#,
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    graph.reachable_from("facade").unwrap();
}

#[test]
fn qualified_and_aliased_macros_cannot_impersonate_audited_names() {
    for source in [
        "#[derive(evil::Debug)] pub struct Thing;",
        "#[evil::api] pub struct Thing;",
        "#[derive(evil::SandStorage)] pub struct Thing { value: i32 }",
        "use evil::ApiMaker as Debug; #[derive(Debug)] pub struct Thing;",
        "use evil::ApiMaker as api; #[api] pub struct Thing;",
        "use sand::SandStorage as StorageDerive; #[derive(StorageDerive)] pub struct Thing { value: i32 }",
        "use evil as serde; #[derive(serde::Serialize)] pub struct Thing;",
        "extern crate evil as serde; #[derive(serde::Serialize)] pub struct Thing;",
        "mod serde { pub use evil::ApiMaker as Serialize; } #[derive(serde::Serialize)] pub struct Thing;",
        "use evil as sand; #[derive(sand::SandStorage)] pub struct Thing { value: i32 }",
        "use evil as sand_macros; #[sand_macros::api] pub struct Thing;",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        let result = graph.reachable_from("facade");
        assert!(
            matches!(result, Err(ReachabilityError::UnclassifiedApiMacro { .. })),
            "qualified or aliased macro escaped: {source}: {result:?}"
        );
    }
}

#[test]
fn reexported_items_audit_every_private_defining_ancestor() {
    for source in [
        r#"
            mod implementation {
                #[unknown_expansion]
                mod nested { pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
        r#"
            mod implementation {
                unknown_items!();
                pub mod nested { pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
        r#"
            mod implementation {
                include!(env!("GENERATED_API"));
                pub mod nested { pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
        r#"
            mod implementation {
                extern "C" {}
                pub mod nested { pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        assert!(
            matches!(
                graph.reachable_from("facade"),
                Err(ReachabilityError::UnclassifiedApiMacro { .. })
                    | Err(ReachabilityError::UnboundItemMacro { .. })
                    | Err(ReachabilityError::UnboundInclude { .. })
                    | Err(ReachabilityError::UnsupportedReachableSyntax { .. })
            ),
            "private defining ancestor escaped the audit: {source}"
        );
    }
}

#[test]
fn resolved_impl_owners_keep_the_impls_lexical_macro_namespace() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            pub struct Exposed;
            mod implementations {
                use evil::api;
                #[api]
                impl crate::Exposed { pub fn method() {} }
            }
        "#,
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnclassifiedApiMacro { owner, .. }) if owner == "facade::Exposed"
    ));
}

#[test]
fn macro_namespace_shadowing_in_lexical_ancestors_fails_closed() {
    for source in [
        r#"
            use evil::Debug;
            mod implementation {
                pub mod nested { #[derive(Debug)] pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
        r#"
            use evil as serde;
            mod implementation {
                pub mod nested { #[derive(serde::Serialize)] pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        assert!(
            matches!(
                graph.reachable_from("facade"),
                Err(ReachabilityError::UnclassifiedApiMacro { .. })
            ),
            "ancestor macro namespace spoof escaped: {source}"
        );
    }
}

#[test]
fn ordinary_nested_definitions_and_trusted_ancestor_imports_remain_supported() {
    for source in [
        r#"
            mod implementation {
                pub mod nested { #[derive(Debug)] pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
        r#"
            use serde::Serialize;
            mod implementation {
                pub mod nested { #[derive(Serialize)] pub struct Exposed; }
            }
            pub use implementation::nested::Exposed;
        "#,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        let reachable = graph.reachable_from("facade").unwrap();
        assert!(
            reachable
                .iter()
                .any(|api| api.identity.ends_with("::Exposed"))
        );
    }
}

#[test]
fn attributes_on_inherent_impls_and_members_fail_closed() {
    for source in [
        "pub struct Thing; #[unknown_expansion] impl Thing {}",
        "pub struct Thing; impl Thing { #[unknown_expansion] pub fn existing() {} }",
        "pub struct Thing; trait Marker {} #[unknown_expansion] impl Marker for Thing {}",
        "pub struct Thing; trait Marker { fn run(); } impl Marker for Thing { #[unknown_expansion] fn run() {} }",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        assert!(matches!(
            graph.reachable_from("facade"),
            Err(ReachabilityError::UnclassifiedApiMacro { owner, .. }) if owner == "facade::Thing"
        ));
    }
}

#[test]
fn attributes_on_modules_reexports_and_trait_members_fail_closed() {
    for source in [
        "#[unknown_expansion] pub mod topic {}",
        "mod inner { pub struct Thing; } #[unknown_expansion] pub use inner::Thing;",
        "pub trait Trait { #[unknown_expansion] fn method(); }",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        assert!(matches!(
            graph.reachable_from("facade"),
            Err(ReachabilityError::UnclassifiedApiMacro { .. })
        ));
    }
}

#[test]
fn trait_only_and_builtin_derives_do_not_require_providers() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "#[derive(Debug, Clone, EntityStateEnum)] pub enum Mode { Active }",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    assert!(graph.reachable_from("facade").is_ok());
}

#[test]
fn renamed_public_attributes_are_trusted_and_custom_item_remains_fail_closed() {
    for source in [
        "#[datapack_component] pub fn component() {} #[on_event] pub fn event() {}",
        "#[sand::datapack_component] pub fn component() {} #[sand::on_event] pub fn event() {}",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        assert!(graph.reachable_from("facade").is_ok(), "{source}");
    }

    for source in [
        "#[custom_item] pub fn item() {}",
        "#[sand::custom_item] pub fn item() {}",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let facade = directory.path().join("facade.rs");
        fs::write(&facade, source).unwrap();
        let graph = SurfaceGraph::load(
            [SourceCrate {
                name: "facade".into(),
                root: facade,
            }],
            [],
            [],
        )
        .unwrap();
        assert!(matches!(
            graph.reachable_from("facade"),
            Err(ReachabilityError::UnboundApiProducer { producer, .. }) if producer == "custom_item"
        ));
    }

    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(&facade, "#[sand::item] pub fn obsolete_name() {}").unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [],
    )
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnclassifiedApiMacro { name, .. }) if name == "item"
    ));
}

#[test]
fn state_derive_provider_claims_the_exact_bound_view_and_inherent_surface() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            use sand::State;
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct PlayerState {
                mana: EntityScore<i32>,
                ready: EntityFlag,
            }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::state_derive_provider(&facade, "facade", &CfgSet::default()).unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        generated,
    )
    .unwrap()
    .bind_api_producer("facade::PlayerState", "State", "state_derive")
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    for identity in [
        "facade::PlayerStateBound",
        "facade::PlayerStateBound::mana",
        "facade::PlayerState::FIELDS",
        "facade::PlayerState::mana",
        "facade::PlayerState::on",
    ] {
        assert!(
            reachable.iter().any(|api| api.identity == identity),
            "missing {identity}"
        );
    }
}

#[test]
fn state_derive_provider_normalizes_raw_field_identifiers() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct PlayerState { r#type: EntityScore<i32> }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::state_derive_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert!(
        generated
            .iter()
            .any(|api| api.identity == "facade::PlayerState::type")
    );
    assert!(generated.iter().any(|api| {
        api.identity == "facade::PlayerStateBound"
            && api.members.iter().any(|(name, _)| name == "type")
    }));
    assert!(!generated.iter().any(|api| {
        api.identity.contains("r#type") || api.members.iter().any(|(name, _)| name == "r#type")
    }));
}

#[test]
fn state_derive_provider_normalizes_raw_owner_identifiers() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct r#type { mana: EntityScore<i32> }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::state_derive_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert!(
        generated
            .iter()
            .any(|api| api.identity == "facade::typeBound")
    );
    assert!(generated.iter().any(|api| {
        api.producer
            .as_ref()
            .is_some_and(|producer| producer.owner == "facade::type")
    }));
    assert!(!generated.iter().any(|api| api.identity.contains("r#")));
}

#[test]
fn custom_item_provider_claims_the_exact_typed_reference_surface() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            #[sand::custom_item(name = "ShardBlade", data = [DAMAGE: i32 = 7])]
            pub fn shard_blade() -> CustomItem {
                CustomItem::new("minecraft:diamond_sword")
                    .custom_data("shard_blade")
            }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::custom_item_provider(&facade, "facade", &CfgSet::default()).unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        generated,
    )
    .unwrap()
    .bind_api_producer("facade::shard_blade", "custom_item", "item_macro")
    .unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    for identity in [
        "facade::ShardBlade",
        "facade::ShardBlade::BASE",
        "facade::ShardBlade::CUSTOM_DATA_KEY",
        "facade::ShardBlade::DAMAGE",
        "facade::ShardBlade::if_wearing",
        "facade::ShardBlade::unless_wearing",
        "facade::ShardBlade::item",
    ] {
        assert!(
            reachable.iter().any(|api| api.identity == identity),
            "missing {identity}"
        );
    }
}

#[test]
fn consumer_macro_providers_traverse_inline_modules() {
    let directory = tempfile::tempdir().unwrap();
    let custom_item = directory.path().join("custom_item.rs");
    fs::write(
        &custom_item,
        r#"
            pub mod nested {
                #[sand::custom_item(name = "ShardBlade")]
                pub fn shard_blade() -> CustomItem { todo!() }
            }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::custom_item_provider(&custom_item, "facade", &CfgSet::default()).unwrap();
    assert_eq!(generated[0].identity, "facade::nested::ShardBlade");
    assert_eq!(
        generated[0].producer.as_ref().unwrap().owner,
        "facade::nested::shard_blade"
    );

    let resourcepack = directory.path().join("resourcepack.rs");
    fs::write(
        &resourcepack,
        r#"
            pub mod nested {
                hud_bar!(
                    name = "health",
                    texture = "health.png",
                    steps = 10,
                    height = 8,
                    ascent = 8,
                );
            }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::resourcepack_macro_provider(&resourcepack, "facade", &CfgSet::default())
            .unwrap();
    assert_eq!(generated[0].identity, "facade::nested::HEALTH");

    let derives = directory.path().join("derives.rs");
    fs::write(
        &derives,
        r#"
            pub mod nested {
                #[derive(State)]
                #[state(namespace = "fixture", scope = player)]
                pub struct PlayerState { mana: EntityScore<i32> }

                #[derive(SandStorage)]
                pub struct PlayerStorage { value: i32 }
            }
        "#,
    )
    .unwrap();
    let state =
        sand_api_enforce::state_derive_provider(&derives, "facade", &CfgSet::default()).unwrap();
    assert!(
        state
            .iter()
            .any(|api| api.identity == "facade::nested::PlayerStateBound")
    );
    assert!(state.iter().any(|api| {
        api.producer.as_ref().is_some_and(|producer| {
            producer.owner == "facade::nested::PlayerState" && producer.name == "State"
        })
    }));
    let storage =
        sand_api_enforce::sand_storage_derive_provider(&derives, "facade", &CfgSet::default())
            .unwrap();
    assert!(
        storage
            .iter()
            .any(|api| api.identity == "facade::nested::PlayerStorage::value")
    );

    let shape_preserving = directory.path().join("shape_preserving.rs");
    fs::write(
        &shape_preserving,
        r#"
            pub mod nested {
                #[sand::function]
                pub fn tick() {}
            }
        "#,
    )
    .unwrap();
    sand_api_enforce::shape_preserving_consumer_provider(
        &shape_preserving,
        "function",
        &CfgSet::default(),
    )
    .unwrap();
}

#[test]
fn resourcepack_provider_accepts_unicode_rust_handle_identifiers() {
    let directory = tempfile::tempdir().unwrap();
    let resourcepack = directory.path().join("resourcepack.rs");
    fs::write(
        &resourcepack,
        r#"
            hud_bar!(
                name = "énergie",
                texture = "energy.png",
                steps = 10,
                height = 8,
                ascent = 8,
            );
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::resourcepack_macro_provider(&resourcepack, "facade", &CfgSet::default())
            .unwrap();
    assert_eq!(generated[0].identity, "facade::ÉNERGIE");
}

#[test]
fn resourcepack_provider_uses_the_last_duplicate_name_like_the_macro() {
    let directory = tempfile::tempdir().unwrap();
    let resourcepack = directory.path().join("resourcepack.rs");
    fs::write(
        &resourcepack,
        r#"hud_element!(name = "old", name = "current", texture = "status.png", height = 8, ascent = 8);"#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::resourcepack_macro_provider(&resourcepack, "facade", &CfgSet::default())
            .unwrap();
    assert_eq!(generated[0].identity, "facade::CURRENT");
}

#[test]
fn custom_item_provider_normalizes_raw_data_constant_identifiers() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
            #[sand::custom_item(name = "TypedItem", data = [r#type: i32 = 1])]
            pub fn typed_item() -> CustomItem { todo!() }
        "#,
    )
    .unwrap();
    let generated =
        sand_api_enforce::custom_item_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert!(generated[0].members.iter().any(|(name, _)| name == "type"));
    assert!(
        !generated[0]
            .members
            .iter()
            .any(|(name, _)| name == "r#type")
    );
}

#[test]
fn consumer_macro_providers_traverse_out_of_line_and_path_modules() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("lib.rs");
    fs::write(
        &facade,
        r#"
            pub mod schemas;
            #[path = "generated/items.rs"]
            pub mod items;
        "#,
    )
    .unwrap();
    fs::write(
        directory.path().join("schemas.rs"),
        r#"
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct PlayerState { mana: EntityScore<i32> }

            #[sand::function]
            pub fn tick() {}

            hud_element!(
                name = "status",
                texture = "status.png",
                height = 8,
                ascent = 8,
            );
        "#,
    )
    .unwrap();
    fs::create_dir(directory.path().join("generated")).unwrap();
    fs::write(
        directory.path().join("generated/items.rs"),
        r#"
            #[sand::custom_item(name = "TypedItem")]
            pub fn typed_item() -> CustomItem { todo!() }

            #[derive(SandStorage)]
            pub struct PlayerStorage { value: i32 }
        "#,
    )
    .unwrap();

    let state =
        sand_api_enforce::state_derive_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert!(
        state
            .iter()
            .any(|api| api.identity == "facade::schemas::PlayerStateBound")
    );
    sand_api_enforce::shape_preserving_consumer_provider(&facade, "function", &CfgSet::default())
        .unwrap();
    let resourcepack =
        sand_api_enforce::resourcepack_macro_provider(&facade, "facade", &CfgSet::default())
            .unwrap();
    assert_eq!(resourcepack[0].identity, "facade::schemas::STATUS");
    let items =
        sand_api_enforce::custom_item_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert_eq!(items[0].identity, "facade::items::TypedItem");
    let storage =
        sand_api_enforce::sand_storage_derive_provider(&facade, "facade", &CfgSet::default())
            .unwrap();
    assert!(
        storage
            .iter()
            .any(|api| api.identity == "facade::items::PlayerStorage::value")
    );
}

#[test]
fn nested_path_modules_resolve_from_the_containing_file_directory() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("lib.rs");
    fs::write(&facade, "pub mod outer;").unwrap();
    fs::write(
        directory.path().join("outer.rs"),
        r#"#[path = "schema.rs"] pub mod schema;"#,
    )
    .unwrap();
    fs::write(
        directory.path().join("schema.rs"),
        r#"
            pub struct Widget;

            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct PlayerState { mana: EntityScore<i32> }

            #[derive(SandStorage)]
            pub struct PlayerStorage { value: i32 }

            #[sand::custom_item(name = "TypedItem")]
            pub fn typed_item() -> CustomItem { todo!() }

            #[sand::function]
            pub fn tick() {}

            hud_element!(
                name = "status",
                texture = "status.png",
                height = 8,
                ascent = 8,
            );
        "#,
    )
    .unwrap();

    let surface_directory = directory.path().join("surface");
    fs::create_dir(&surface_directory).unwrap();
    let surface = surface_directory.join("lib.rs");
    fs::write(&surface, "pub mod outer;").unwrap();
    fs::write(
        surface_directory.join("outer.rs"),
        r#"#[path = "schema.rs"] pub mod schema;"#,
    )
    .unwrap();
    fs::write(surface_directory.join("schema.rs"), "pub struct Widget;").unwrap();
    let reachable = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: surface,
        }],
        [],
        [],
    )
    .unwrap()
    .reachable_from("facade")
    .unwrap();
    assert!(
        reachable
            .iter()
            .any(|api| api.identity == "facade::outer::schema::Widget")
    );

    let state =
        sand_api_enforce::state_derive_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert!(
        state
            .iter()
            .any(|api| api.identity == "facade::outer::schema::PlayerStateBound")
    );
    let storage =
        sand_api_enforce::sand_storage_derive_provider(&facade, "facade", &CfgSet::default())
            .unwrap();
    assert!(
        storage
            .iter()
            .any(|api| api.identity == "facade::outer::schema::PlayerStorage::value")
    );
    let items =
        sand_api_enforce::custom_item_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert_eq!(items[0].identity, "facade::outer::schema::TypedItem");
    sand_api_enforce::shape_preserving_consumer_provider(&facade, "function", &CfgSet::default())
        .unwrap();
    let resourcepack =
        sand_api_enforce::resourcepack_macro_provider(&facade, "facade", &CfgSet::default())
            .unwrap();
    assert_eq!(resourcepack[0].identity, "facade::outer::schema::STATUS");
}

#[test]
fn inline_modules_preserve_their_child_file_search_directory() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("lib.rs");
    fs::write(
        &facade,
        r#"pub mod outer { pub mod schema; #[path = "alternate.rs"] pub mod alternate; }"#,
    )
    .unwrap();
    fs::create_dir(directory.path().join("outer")).unwrap();
    fs::write(
        directory.path().join("outer/schema.rs"),
        r#"
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct PlayerState { mana: EntityScore<i32> }

            #[derive(SandStorage)]
            pub struct PlayerStorage { value: i32 }

            #[sand::custom_item(name = "TypedItem")]
            pub fn typed_item() -> CustomItem { todo!() }

            #[sand::function]
            pub fn tick() {}

            hud_element!(name = "status", texture = "status.png", height = 8, ascent = 8);
        "#,
    )
    .unwrap();
    fs::write(
        directory.path().join("outer/alternate.rs"),
        "pub struct Alternate;",
    )
    .unwrap();

    let state =
        sand_api_enforce::state_derive_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert!(
        state
            .iter()
            .any(|api| api.identity == "facade::outer::schema::PlayerStateBound")
    );
    let storage =
        sand_api_enforce::sand_storage_derive_provider(&facade, "facade", &CfgSet::default())
            .unwrap();
    assert!(
        storage
            .iter()
            .any(|api| api.identity == "facade::outer::schema::PlayerStorage::value")
    );
    let items =
        sand_api_enforce::custom_item_provider(&facade, "facade", &CfgSet::default()).unwrap();
    assert_eq!(items[0].identity, "facade::outer::schema::TypedItem");
    sand_api_enforce::shape_preserving_consumer_provider(&facade, "function", &CfgSet::default())
        .unwrap();
    let resourcepack =
        sand_api_enforce::resourcepack_macro_provider(&facade, "facade", &CfgSet::default())
            .unwrap();
    assert_eq!(resourcepack[0].identity, "facade::outer::schema::STATUS");

    let surface_directory = directory.path().join("surface");
    fs::create_dir(&surface_directory).unwrap();
    fs::write(
        surface_directory.join("lib.rs"),
        r#"pub mod outer { pub mod schema; #[path = "alternate.rs"] pub mod alternate; }"#,
    )
    .unwrap();
    fs::create_dir(surface_directory.join("outer")).unwrap();
    fs::write(
        surface_directory.join("outer/schema.rs"),
        "pub struct Widget;",
    )
    .unwrap();
    fs::write(
        surface_directory.join("outer/alternate.rs"),
        "pub struct Alternate;",
    )
    .unwrap();
    let reachable = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: surface_directory.join("lib.rs"),
        }],
        [],
        [],
    )
    .unwrap()
    .reachable_from("facade")
    .unwrap();
    assert!(
        reachable
            .iter()
            .any(|api| api.identity == "facade::outer::schema::Widget")
    );
    assert!(
        reachable
            .iter()
            .any(|api| api.identity == "facade::outer::alternate::Alternate")
    );
}

#[test]
fn consumer_macro_providers_follow_literal_includes() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("lib.rs");
    fs::write(&facade, r#"include!("schemas.rs");"#).unwrap();
    fs::write(
        directory.path().join("schemas.rs"),
        r#"
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct PlayerState { mana: EntityScore<i32> }

            #[derive(SandStorage)]
            pub struct PlayerStorage { value: i32 }

            #[sand::custom_item(name = "TypedItem")]
            pub fn typed_item() -> CustomItem { todo!() }

            #[sand::function]
            pub fn tick() {}

            hud_element!(name = "status", texture = "status.png", height = 8, ascent = 8);
        "#,
    )
    .unwrap();
    let cfg = CfgSet::default();
    let state = sand_api_enforce::state_derive_provider(&facade, "facade", &cfg).unwrap();
    let storage = sand_api_enforce::sand_storage_derive_provider(&facade, "facade", &cfg).unwrap();
    let items = sand_api_enforce::custom_item_provider(&facade, "facade", &cfg).unwrap();
    let resourcepack =
        sand_api_enforce::resourcepack_macro_provider(&facade, "facade", &cfg).unwrap();
    sand_api_enforce::shape_preserving_consumer_provider(&facade, "function", &cfg).unwrap();
    assert!(
        state
            .iter()
            .any(|api| api.identity == "facade::PlayerStateBound")
    );
    assert!(
        storage
            .iter()
            .any(|api| api.identity == "facade::PlayerStorage::value")
    );
    assert_eq!(items[0].identity, "facade::TypedItem");
    assert_eq!(resourcepack[0].identity, "facade::STATUS");

    SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        state
            .into_iter()
            .chain(storage)
            .chain(items)
            .chain(resourcepack),
    )
    .unwrap()
    .bind_api_producer("facade::PlayerState", "State", "state_derive")
    .unwrap()
    .bind_api_producer("facade::PlayerStorage", "SandStorage", "storage_derive")
    .unwrap()
    .bind_api_producer("facade::typed_item", "custom_item", "item_macro")
    .unwrap();
}

#[test]
fn consumer_macro_providers_use_the_surface_cfg_set() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("lib.rs");
    fs::write(
        &facade,
        r#"
            #[cfg(feature = "active")]
            #[cfg_attr(feature = "active", derive(State))]
            #[cfg_attr(feature = "active", state(namespace = "fixture", scope = player))]
            pub struct ActiveState {
                value: EntityScore<i32>,
                #[cfg(feature = "inactive")]
                hidden: EntityScore<i32>,
            }

            #[cfg(feature = "inactive")]
            #[derive(State)]
            #[state(namespace = "fixture", scope = player)]
            pub struct InactiveState { value: EntityScore<i32> }

            #[cfg(feature = "active")]
            #[cfg_attr(feature = "active", derive(SandStorage))]
            pub struct ActiveStorage {
                value: i32,
                #[cfg(feature = "inactive")]
                hidden: i32,
            }

            #[cfg(feature = "inactive")]
            #[derive(SandStorage)]
            pub struct InactiveStorage { value: i32 }

            #[cfg(feature = "active")]
            #[cfg_attr(feature = "active", sand::custom_item(name = "ActiveItem"))]
            pub fn active_item() -> CustomItem { todo!() }

            #[cfg(feature = "inactive")]
            #[sand::custom_item(name = "InactiveItem")]
            pub fn inactive_item() -> CustomItem { todo!() }

            #[cfg(feature = "active")]
            hud_bar!(name = "active", texture = "active.png", steps = 1, height = 1, ascent = 1);
            #[cfg(feature = "inactive")]
            hud_bar!(name = "inactive", texture = "inactive.png", steps = 1, height = 1, ascent = 1);

            #[cfg(feature = "disabled")]
            pub mod absent;
        "#,
    )
    .unwrap();
    let cfg = CfgSet {
        features: ["active".to_owned()].into_iter().collect(),
        ..CfgSet::default()
    };
    let state = sand_api_enforce::state_derive_provider(&facade, "facade", &cfg).unwrap();
    assert!(
        state
            .iter()
            .any(|api| api.identity == "facade::ActiveStateBound")
    );
    assert!(!state.iter().any(|api| api.identity.contains("Inactive")));
    assert!(!state.iter().any(|api| {
        api.identity.ends_with("::hidden") || api.members.iter().any(|(name, _)| name == "hidden")
    }));
    let items = sand_api_enforce::custom_item_provider(&facade, "facade", &cfg).unwrap();
    assert_eq!(items[0].identity, "facade::ActiveItem");
    let storage = sand_api_enforce::sand_storage_derive_provider(&facade, "facade", &cfg).unwrap();
    assert!(
        storage
            .iter()
            .any(|api| api.identity == "facade::ActiveStorage::value")
    );
    assert!(!storage.iter().any(|api| api.identity.contains("Inactive")));
    assert!(!storage.iter().any(|api| api.identity.ends_with("::hidden")));
    let generated = state
        .iter()
        .chain(storage.iter())
        .chain(items.iter())
        .cloned()
        .collect::<Vec<_>>();
    SurfaceGraph::load_with_cfg(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        cfg.clone(),
        generated,
    )
    .unwrap()
    .bind_api_producer("facade::ActiveState", "State", "state_derive")
    .unwrap()
    .bind_api_producer("facade::ActiveStorage", "SandStorage", "storage_derive")
    .unwrap()
    .bind_api_producer("facade::active_item", "custom_item", "item_macro")
    .unwrap();
    let resourcepack =
        sand_api_enforce::resourcepack_macro_provider(&facade, "facade", &cfg).unwrap();
    assert_eq!(resourcepack[0].identity, "facade::ACTIVE");

    let shape = directory.path().join("shape.rs");
    fs::write(
        &shape,
        r#"
            #[cfg(feature = "inactive")]
            #[sand::function]
            pub fn inactive() {}

            #[cfg_attr(feature = "active", sand::function)]
            pub fn active() {}
        "#,
    )
    .unwrap();
    sand_api_enforce::shape_preserving_consumer_provider(&shape, "function", &cfg).unwrap();
}

#[test]
fn producer_binding_is_per_declaration_and_cannot_exempt_a_new_derive() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        "#[derive(SandStorage)] pub struct Covered { value: i32 } #[derive(SandStorage)] pub struct Added { value: i32 }",
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [
            GeneratedApi {
                identity: "facade::Covered::SCHEMA".into(),
                provider: "storage_provider".into(),
                producer: Some(GeneratedProducer {
                    owner: "facade::Covered".into(),
                    name: "SandStorage".into(),
                }),
                kind: ReachableKind::AssociatedConst,
                members: vec![],
                excluded: false,
            },
            GeneratedApi {
                identity: "facade::Covered::value".into(),
                provider: "storage_provider".into(),
                producer: Some(GeneratedProducer {
                    owner: "facade::Covered".into(),
                    name: "SandStorage".into(),
                }),
                kind: ReachableKind::Method,
                members: vec![],
                excluded: false,
            },
        ],
    )
    .unwrap()
    .bind_api_producer("facade::Covered", "SandStorage", "storage_provider")
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnboundApiProducer { owner, .. }) if owner == "facade::Added"
    ));
}

#[test]
fn api_producing_derive_inside_generated_transcriber_requires_provider() {
    let directory = tempfile::tempdir().unwrap();
    let facade = directory.path().join("facade.rs");
    fs::write(
        &facade,
        r#"
        #[macro_export]
        macro_rules! storage_family {
            ($name:ident) => {
                #[derive(Debug, SandStorage)]
                pub struct $name { value: i32 }
            };
        }
        "#,
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade.clone(),
        }],
        [],
        [],
    )
    .unwrap();
    assert!(matches!(
        graph.reachable_from("facade"),
        Err(ReachabilityError::UnboundApiProducer { producer, owner, .. })
            if producer == "SandStorage" && owner == "facade::storage_family"
    ));

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "facade".into(),
            root: facade,
        }],
        [],
        [GeneratedApi {
            identity: "facade::GeneratedStorage".into(),
            provider: "storage_family_provider".into(),
            producer: Some(GeneratedProducer {
                owner: "facade::storage_family".into(),
                name: "SandStorage".into(),
            }),
            kind: ReachableKind::Struct,
            members: vec![("SCHEMA".into(), ReachableKind::AssociatedConst)],
            excluded: false,
        }],
    )
    .unwrap();
    assert!(matches!(
        graph.bind_api_producer(
        "facade::storage_family",
        "SandStorage",
        "storage_family_provider",
        ),
        Err(ReachabilityError::InvalidApiProducerProvider { expected, .. }) if expected.is_empty()
    ));
}
