use std::collections::BTreeSet;
use std::fs;

use sand_api_enforce::{
    CfgSet, ContractIdentity, GeneratedApi, ReachabilityError, ReachableKind, SourceCrate,
    SurfaceGraph, audit_reachable_surface,
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
            #[path = "alternate.rs"] pub mod path_module;
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
            pub struct Thing { pub value: u32, hidden: u32 }
            impl Thing {
                pub fn new(value: u32) -> Self { Self { value, hidden: 0 } }
                pub const DEFAULT: u32 = 1;
                fn implementation_detail(&self) -> u32 { self.hidden }
            }
            pub enum Mode { Fast, Slow }
            pub trait ContractTrait {
                type Output;
                const VERSION: u32;
                fn render(&self) -> Self::Output;
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
        facade.join("lib.rs"),
        r#"
            extern crate core_lib as implementation;
            pub use core_lib::model::{Thing as Builder, ContractTrait, Alias};
            pub use core_lib::model::r#type as KeywordType;
            pub use core_lib::RootForwarded;
            pub use implementation::path_module::PathThing;
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

#[test]
fn extracts_explicit_and_glob_reexports_with_associated_surface() {
    let (_directory, reachable) = fixture(&[]);

    let thing = item(&reachable, "core_lib::model::Thing");
    assert_eq!(
        thing.paths,
        BTreeSet::from(["facade::Builder".into(), "facade::prelude::Thing".into()])
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
    assert!(
        !without_feature
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
fn architecture_09_narrow_internal_exclusions_are_absent() {
    let (_, api) = fixture(&[]);
    assert!(
        !api.iter()
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
