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
            pub use core_lib::model::{self, Thing as Builder, ContractTrait, Alias};
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
                mod _generated { include!("generated_commands.rs"); }
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
            kind: ReachableKind::Struct,
            members: vec![("to".into(), ReachableKind::Method)],
            excluded: false,
        }],
    )
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
                kind: ReachableKind::Struct,
                members: vec![],
                excluded: false,
            },
            GeneratedApi {
                identity: "core_lib::generated::Only".into(),
                provider: "second".into(),
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
            kind: ReachableKind::Struct,
            members: vec![],
            excluded: false,
        }],
    )
    .expect("a controlled provider models the generated owner");
    let reachable = graph.reachable_from("facade").unwrap();
    assert_eq!(
        item(&reachable, "facade::Generated::exposed").origin,
        sand_api_enforce::ReachableOrigin::Generator("fixture_generator".into())
    );
}
