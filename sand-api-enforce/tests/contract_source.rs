use std::collections::BTreeSet;

use sand_api_enforce::{
    ContractSourceError, ReachableApi, ReachableKind, ReachableOrigin, SourceCrate, SurfaceGraph,
    contract_declarations_from_files, resolve_contract_identities,
    validate_contract_lookup_namespace,
};
use tempfile::tempdir;

fn api(identity: &str, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind: ReachableKind::Struct,
        origin: ReachableOrigin::Source,
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
        definition: None,
    }
}

#[test]
fn canonical_and_alias_paths_share_one_collision_checked_namespace() {
    let contracts = vec![
        sand_api_enforce::ContractIdentity {
            identity: "lower::First".into(),
            canonical_path: "sand::first".into(),
            aliases: BTreeSet::from(["sand::shared".into()]),
        },
        sand_api_enforce::ContractIdentity {
            identity: "lower::Second".into(),
            canonical_path: "sand::shared".into(),
            aliases: BTreeSet::new(),
        },
    ];
    assert!(matches!(
        validate_contract_lookup_namespace(&contracts),
        Err(ContractSourceError::DuplicateLookupPath { path, .. }) if path == "sand::shared"
    ));
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

        register_component_api! {
            path: "sand::predicate::check",
            aliases: ["sand::prelude::check"],
            kind: Function,
            summary: "Checks a predicate value."
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
fn inherent_contracts_inherit_their_type_aliases_unless_authored_explicitly() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"
        #[api(
            path = "sand::topic::Thing",
            aliases = ["sand::prelude::Thing"],
            summary = "Names a topic value.",
            context = "The type is re-exported through the prelude.",
            minecraft = "Represents a checked Minecraft value.",
            use_when = ["A topic value is required"],
            avoid_when = ["Compiler wiring is required"],
            example = "let value = Thing;"
        )]
        pub struct Thing;

        impl Thing {
            #[api(
                path = "sand::topic::Thing::build",
                summary = "Builds a topic value.",
                context = "The constructor is available through every type alias.",
                minecraft = "Creates the checked Minecraft value.",
                use_when = ["A new value is needed"],
                avoid_when = ["An existing value is available"],
                returns = "A topic value.",
                example = "Thing::build()"
            )]
            pub fn build() -> Self { Self }

            #[api(
                path = "sand::topic::Thing::explicit",
                aliases = ["sand::advanced::Thing::explicit"],
                summary = "Uses an intentionally different alias.",
                context = "This proves authored aliases are never overwritten.",
                minecraft = "Reads the checked Minecraft value.",
                use_when = ["The advanced route is required"],
                avoid_when = ["The ordinary alias should be used"],
                returns = "A topic value.",
                example = "Thing::explicit()"
            )]
            pub fn explicit() -> Self { Self }
        }

        #[api(
            path = "sand::topic::Describe",
            aliases = ["sand::prelude::Describe"],
            summary = "Defines a fixture trait.",
            context = "The trait is re-exported through the prelude.",
            minecraft = "Describes a checked Minecraft fixture value.",
            use_when = ["A fixture behavior is required"],
            avoid_when = ["A concrete value is sufficient"],
            example = "Describe::describe()"
        )]
        pub trait Describe {
            #[api(
                path = "sand::topic::Describe::describe",
                summary = "Describes a fixture value.",
                context = "The associated method inherits every trait alias.",
                minecraft = "Describes the checked Minecraft fixture value.",
                use_when = ["Testing associated aliases"],
                avoid_when = ["Authoring a production value"],
                returns = "A fixture description.",
                example = "Describe::describe()"
            )]
            fn describe() -> &'static str;
        }
        "#,
    )
    .unwrap();

    let declarations = contract_declarations_from_files([source]).unwrap();
    let build = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::topic::Thing::build")
        .unwrap();
    assert_eq!(
        build.aliases,
        BTreeSet::from(["sand::prelude::Thing::build".to_owned()])
    );
    let explicit = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::topic::Thing::explicit")
        .unwrap();
    assert_eq!(
        explicit.aliases,
        BTreeSet::from(["sand::advanced::Thing::explicit".to_owned()])
    );
    let trait_method = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::topic::Describe::describe")
        .unwrap();
    assert_eq!(
        trait_method.aliases,
        BTreeSet::from(["sand::prelude::Describe::describe".to_owned()])
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
        register_component_api! {
            path: "sand::predicate::Predicate",
            aliases: ["sand::prelude::Predicate"],
            kind: Struct,
            summary: "Builds a predicate."
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
        register_component_api! { path: "sand::predicate::Predicate", aliases: [], kind: Struct, summary: "Builds a predicate." }
        register_component_api! { path: "sand::prelude::Predicate", aliases: [], kind: Struct, summary: "Builds the same predicate." }
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
fn facade_contracts_use_the_shared_semantic_validator() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"register_entity_api! {
            path: "sand::entity::EntityArchetype::new",
            aliases: [],
            kind: Method,
            summary: ""
        }"#,
    )
    .unwrap();
    let error = contract_declarations_from_files([source]).unwrap_err();
    assert!(error.to_string().contains("summary") && error.to_string().contains("cannot be empty"));
}

#[test]
fn systems_family_requires_a_known_feature_mapping() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"register_systems_api! {
            path: "sand::systems::future_system::Tracker",
            aliases: [],
            kind: Struct,
            summary: "Tracks a future gameplay system."
        }"#,
    )
    .unwrap();
    let error = contract_declarations_from_files([source]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not map to a known Cargo feature"),
        "{error}"
    );
}

#[test]
fn facade_contract_kind_must_match_the_reachable_definition() {
    let reachable = vec![api("lower::Thing", &["sand::topic::Thing"])];
    let temp = tempdir().unwrap();
    let source = temp.path().join("contracts.rs");
    std::fs::write(
        &source,
        r#"register_component_api! {
            path: "sand::topic::Thing",
            aliases: [],
            kind: Enum,
            summary: "Names a typed thing."
        }"#,
    )
    .unwrap();
    let declarations = contract_declarations_from_files([source]).unwrap();
    let error = resolve_contract_identities(&reachable, &declarations).unwrap_err();
    assert!(matches!(
        error.as_slice(),
        [ContractSourceError::InvalidFacadeContract { .. }]
    ));
}

#[test]
fn facade_callable_shape_is_validated_against_the_reachable_definition() {
    let temp = tempdir().unwrap();
    let facade = temp.path().join("sand.rs");
    let implementation = temp.path().join("lower.rs");
    std::fs::write(&facade, "pub use lower::build;\n").unwrap();
    std::fs::write(
        &implementation,
        "pub fn build(value: u32) -> String { value.to_string() }\n",
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
                root: implementation,
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();

    for (signature, params, returns, expected) in [
        (
            "fn build(value: u64) -> String",
            r#"["value" => "Input."]"#,
            "Some(\"Text.\")",
            "stale signature",
        ),
        (
            "fn build(value: u32) -> String",
            "[]",
            "Some(\"Text.\")",
            "parameter metadata",
        ),
        (
            "fn build(value: u32) -> String",
            r#"["value" => "Input.", "extra" => "Missing."]"#,
            "Some(\"Text.\")",
            "parameter metadata",
        ),
        (
            "fn build(value: u32) -> String",
            r#"["value" => "Input."]"#,
            "None",
            "return metadata",
        ),
    ] {
        let contracts = temp.path().join("contracts.rs");
        std::fs::write(
            &contracts,
            format!(
                r#"register! {{
                path: "sand::build",
                aliases: [],
                module: "sand",
                kind: Function,
                signature: "{signature}",
                summary: "Builds text from a value.",
                context: "This fixture validates facade structure.",
                minecraft: "It emits no Minecraft data.",
                use_when: ["Testing facade validation"],
                avoid_when: ["Authoring a datapack"],
                params: {params},
                returns: {returns},
                example: "let text = build(1);"
            }}"#
            ),
        )
        .unwrap();
        let declarations = contract_declarations_from_files([contracts]).unwrap();
        let errors = resolve_contract_identities(&reachable, &declarations).unwrap_err();
        assert!(errors[0].to_string().contains(expected), "{errors:?}");
    }
}

#[test]
fn concrete_family_signatures_validate_receiver_mutability_and_typed_self() {
    let temp = tempdir().unwrap();
    let facade = temp.path().join("sand.rs");
    let implementation = temp.path().join("lower.rs");
    std::fs::write(&facade, "pub use lower::Thing;\n").unwrap();
    std::fs::write(
        &implementation,
        r#"pub struct Thing;
impl Thing {
    pub fn view(&self) {}
    pub fn boxed(self: Box<Self>) {}
}
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
                root: implementation,
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();

    for (member, signature) in [("view", "fn view(&mut self)"), ("boxed", "fn boxed(self)")] {
        let contracts = temp.path().join(format!("{member}.rs"));
        std::fs::write(
            &contracts,
            format!(
                r#"register_event_api! {{
                    path: "sand::Thing::{member}",
                    aliases: [],
                    module: "sand",
                    kind: Method,
                    signature: "{signature}",
                    summary: "Exercises a concrete family receiver.",
                    minecraft: "This test has no Minecraft runtime behavior."
                }}"#
            ),
        )
        .unwrap();
        let declarations = contract_declarations_from_files([contracts]).unwrap();
        let error = resolve_contract_identities(&reachable, &declarations).unwrap_err();
        assert!(
            error
                .iter()
                .any(|error| error.to_string().contains("stale signature")),
            "receiver drift unexpectedly passed: {error:?}"
        );
    }

    let contracts = temp.path().join("exact.rs");
    std::fs::write(
        &contracts,
        r#"register_event_api! {
            path: "sand::Thing::view",
            aliases: [],
            module: "sand",
            kind: Method,
            signature: "fn view(&self)",
            summary: "Exercises the exact concrete family receiver.",
            minecraft: "This test has no Minecraft runtime behavior."
        }"#,
    )
    .unwrap();
    let declarations = contract_declarations_from_files([contracts]).unwrap();
    resolve_contract_identities(&reachable, &declarations).unwrap();
}

#[test]
fn repository_contract_sources_are_the_actual_authored_declarations() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let mut sources = Vec::new();
    for package in [
        "sand",
        "sand-commands",
        "sand-components",
        "sand-core",
        "sand-resourcepack",
        "sand-version",
    ] {
        collect_rust_sources(&workspace.join(package).join("src"), &mut sources);
    }
    sources.sort();
    let declarations = contract_declarations_from_files(&sources).unwrap();
    assert_eq!(declarations.len(), 4_802);
    assert_eq!(
        declarations.first().unwrap().canonical_path,
        "sand::EntityStateEnum"
    );
    assert_eq!(
        declarations.last().unwrap().canonical_path,
        "sand::vfx::VfxStep::Sound::0"
    );
    let predicate_new = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::predicate::Predicate::new")
        .unwrap();
    assert!(
        predicate_new
            .source
            .ends_with("sand-components/src/predicate/mod.rs")
    );
    assert!(predicate_new.definition.is_some());
    let branch = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::execute_when::if_")
        .unwrap();
    assert!(branch.source.ends_with("sand-core/src/execute_when.rs"));
    assert!(branch.definition.is_some());
    let condition = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::condition::Condition::entity")
        .unwrap();
    assert!(condition.source.ends_with("sand-core/src/condition.rs"));
    assert!(condition.definition.is_some());
    let resource_module = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::resource_ref")
        .unwrap();
    assert!(resource_module.source.ends_with("sand/src/lib.rs"));
    assert!(resource_module.definition.is_some());
    let vfx_play = declarations
        .iter()
        .find(|declaration| declaration.canonical_path == "sand::vfx::Vfx::play")
        .unwrap();
    assert_eq!(
        vfx_play.aliases,
        BTreeSet::from([
            "sand::cmd::Vfx::play".to_owned(),
            "sand::command::Vfx::play".to_owned(),
            "sand::prelude::Vfx::play".to_owned(),
            "sand::prelude::cmd::Vfx::play".to_owned(),
        ])
    );
}

fn collect_rust_sources(directory: &std::path::Path, sources: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

fn contract_binding_fixture(dummy_attributes: &str, features: &[&str]) -> ContractSourceError {
    let temp = tempdir().unwrap();
    let facade = temp.path().join("sand.rs");
    let implementation = temp.path().join("lower.rs");
    std::fs::write(&facade, "pub use lower::Supported;\n").unwrap();
    std::fs::write(
        &implementation,
        format!(
            r#"
            pub struct Supported;

            {dummy_attributes}
            #[api(
                path = "sand::Supported",
                summary = "Attempts to impersonate the supported item.",
                context = "This contract is deliberately attached to the wrong declaration.",
                minecraft = "It must not describe the reachable Minecraft API.",
                use_when = ["Never"],
                avoid_when = ["Always"],
                example = "compile_error!(\"unreachable example\");"
            )]
            struct Dummy;
            "#
        ),
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
        features.iter().map(|feature| (*feature).to_owned()),
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();
    let declarations = contract_declarations_from_files([implementation]).unwrap();
    resolve_contract_identities(&reachable, &declarations)
        .unwrap_err()
        .into_iter()
        .next()
        .unwrap()
}

#[test]
fn private_dummy_contract_cannot_satisfy_a_reachable_item() {
    assert!(matches!(
        contract_binding_fixture("", &[]),
        ContractSourceError::ContractAttachedToDifferentItem { canonical_path, .. }
            if canonical_path == "sand::Supported"
    ));
}

#[test]
fn cfg_disabled_dummy_contract_cannot_satisfy_a_reachable_item() {
    assert!(matches!(
        contract_binding_fixture("#[cfg(any())]", &[]),
        ContractSourceError::ContractAttachedToDifferentItem { canonical_path, .. }
            if canonical_path == "sand::Supported"
    ));
}

#[test]
fn cfg_attr_disabled_dummy_contract_cannot_satisfy_a_reachable_item() {
    assert!(matches!(
        contract_binding_fixture(
            "#[cfg_attr(feature = \"disable_dummy\", cfg(any()))]",
            &["disable_dummy"],
        ),
        ContractSourceError::ContractAttachedToDifferentItem { canonical_path, .. }
            if canonical_path == "sand::Supported"
    ));
}
