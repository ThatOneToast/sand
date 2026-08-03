use std::collections::BTreeSet;

use sand_api_enforce::{
    ContractSourceError, ReachableApi, ReachableKind, ReachableOrigin, SourceCrate, SurfaceGraph,
    contract_declarations_from_files, resolve_contract_identities,
};
use tempfile::tempdir;

fn api(identity: &str, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind: ReachableKind::Struct,
        origin: ReachableOrigin::Source,
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
    }
}

#[test]
fn lower_crate_attribute_resolves_through_the_facade_graph() {
    let temp = tempdir().unwrap();
    let facade = temp.path().join("sand.rs");
    let implementation = temp.path().join("lower.rs");
    std::fs::write(&facade, "pub use lower::Thing;\n").unwrap();
    std::fs::write(
        &implementation,
        r#"
        #[api(
            path = "sand::Thing",
            summary = "Names a supported lower-crate value.",
            context = "The facade deliberately re-exports this implementation type.",
            minecraft = "Represents a checked Minecraft domain value.",
            use_when = ["The typed value is required"],
            avoid_when = ["The value is only compiler wiring"],
            example = "let value = sand::Thing;"
        )]
        pub struct Thing;
        "#,
    )
    .unwrap();
    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "sand".into(),
                root: facade,
            },
            SourceCrate {
                name: "lower".into(),
                root: implementation.clone(),
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();
    let declarations = contract_declarations_from_files([implementation]).unwrap();
    let contracts = resolve_contract_identities(&reachable, &declarations).unwrap();
    assert_eq!(contracts[0].identity, "lower::Thing");
    assert_eq!(contracts[0].canonical_path, "sand::Thing");
}

#[test]
fn reads_attribute_members_and_facade_provider_contracts() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"
        #[api(
            path = "sand::predicate::Mode",
            aliases = ["sand::prelude::Mode"],
            summary = "Chooses the predicate mode.",
            context = "The mode controls composition.",
            minecraft = "Serializes as a condition mode.",
            use_when = ["Composing predicates"],
            avoid_when = ["No condition is needed"],
            variants(All = "Requires all children."),
            example = "Mode::All"
        )]
        pub enum Mode { All }

        register! {
            path: "sand::predicate::check",
            aliases: ["sand::prelude::check"],
            params: ["value" => "The value to check."],
            returns: Some("A condition."),
        }
        "#,
    )
    .unwrap();

    let declarations = contract_declarations_from_files([&source]).unwrap();
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| declaration.canonical_path.as_str())
            .collect::<Vec<_>>(),
        [
            "sand::predicate::Mode",
            "sand::predicate::Mode::All",
            "sand::predicate::check"
        ]
    );
}

#[test]
fn resolves_paths_to_one_underlying_identity_and_rejects_bogus_aliases() {
    let reachable = vec![api(
        "sand_components::predicate::Predicate",
        &["sand::predicate::Predicate", "sand::prelude::Predicate"],
    )];
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"
        register! {
            path: "sand::predicate::Predicate",
            aliases: ["sand::prelude::Predicate"],
        }
        "#,
    )
    .unwrap();
    let declarations = contract_declarations_from_files([&source]).unwrap();
    let contracts = resolve_contract_identities(&reachable, &declarations).unwrap();
    assert_eq!(
        contracts[0].identity,
        "sand_components::predicate::Predicate"
    );
    assert_eq!(
        contracts[0].aliases,
        BTreeSet::from(["sand::prelude::Predicate".to_owned()])
    );

    let mut bogus = declarations;
    bogus[0].aliases.insert("sand::missing::Predicate".into());
    assert_eq!(
        resolve_contract_identities(&reachable, &bogus).unwrap_err(),
        [ContractSourceError::UnreachablePath(
            "sand::missing::Predicate".into()
        )]
    );
}

#[test]
fn duplicate_contracts_for_aliases_of_one_item_fail() {
    let reachable = vec![api(
        "sand_components::predicate::Predicate",
        &["sand::predicate::Predicate", "sand::prelude::Predicate"],
    )];
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"
        register! { path: "sand::predicate::Predicate", aliases: [] }
        register! { path: "sand::prelude::Predicate", aliases: [] }
        "#,
    )
    .unwrap();
    let declarations = contract_declarations_from_files([&source]).unwrap();
    let failures = resolve_contract_identities(&reachable, &declarations).unwrap_err();
    assert!(matches!(
        failures.as_slice(),
        [ContractSourceError::DuplicateIdentity { .. }]
    ));
}

#[test]
fn repository_contract_sources_are_the_actual_authored_declarations() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let declarations = contract_declarations_from_files([
        workspace.join("sand/src/lib.rs"),
        workspace.join("sand/src/api_contracts.rs"),
    ])
    .unwrap();
    assert_eq!(declarations.len(), 14);
    assert_eq!(declarations.first().unwrap().canonical_path, "sand::data");
    assert_eq!(declarations.last().unwrap().canonical_path, "sand::vanilla");
}
