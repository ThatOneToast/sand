use std::collections::BTreeSet;
use std::fs;

use sand_api_enforce::{
    ContractIdentity, GeneratedApi, ReachabilityError, ReachableKind, SourceCrate, SurfaceGraph,
    audit_reachable_surface,
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
            mod private_source { pub struct Forwarded; impl Forwarded { pub fn create() -> Self { Self } } }
            pub use private_source::Forwarded as RootForwarded;
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
            pub fn helper() {}
            #[cfg(feature = "extras")] pub fn feature_helper() {}
            #[doc(hidden)] pub struct GeneratedWire;
            pub(crate) struct CrateOnly;
        "#,
    )
    .unwrap();
    fs::write(
        facade.join("lib.rs"),
        r#"
            pub use core_lib::model::{Thing as Builder, ContractTrait, Alias};
            pub use core_lib::RootForwarded;
            pub mod prelude { pub use core_lib::model::*; }
            pub use core_lib::generated::GeneratedBuilder;
            #[doc(hidden)] pub mod __private { pub struct FacadeWire; }
        "#,
    )
    .unwrap();

    let graph = SurfaceGraph::load(
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
        features.iter().map(|feature| (*feature).to_owned()),
        [GeneratedApi {
            identity: "core_lib::generated::GeneratedBuilder".into(),
            provider: "fixture_generator".into(),
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
        item(&reachable, "core_lib::model::Thing::new").paths,
        BTreeSet::from([
            "facade::Builder::new".into(),
            "facade::prelude::Thing::new".into()
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
        sand_api_enforce::ReachableOrigin::Generator("fixture_generator".into())
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
