use std::collections::BTreeSet;
use std::path::PathBuf;

use sand_api_enforce::{
    ContractIdentity, ReachableApi, ReachableKind, ReachableOrigin, ScopeFailure, ScopeManifest,
};

fn api(identity: &str, paths: &[&str]) -> ReachableApi {
    api_kind(identity, ReachableKind::Function, paths)
}

fn api_kind(identity: &str, kind: ReachableKind, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind,
        origin: ReachableOrigin::Source,
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
        definition: None,
    }
}

#[test]
fn nonrecursive_root_scope_uses_discovered_types_for_lowercase_members() {
    let source = r#"
        schema_version = 1
        static_surface_items = 3
        enforced_scope_baseline = []
        pending_scope_ceiling = 2
        pending_item_ceiling = 3

        [[scope]]
        id = "root"
        canonical_module = "sand"
        state = "pending"
        tier = "author"
        provider = "source"
        recursive = false

        [[scope]]
        id = "topic"
        canonical_module = "sand::topic"
        state = "pending"
        tier = "author"
        provider = "source"
    "#;
    let reachable = [
        api_kind("fixture::widget", ReachableKind::Struct, &["sand::widget"]),
        api_kind(
            "fixture::widget::BASE",
            ReachableKind::AssociatedConst,
            &["sand::widget::BASE"],
        ),
        api("fixture::topic::item", &["sand::topic::item"]),
    ];

    let report = ScopeManifest::from_toml(source)
        .unwrap()
        .evaluate(&reachable, &[], &BTreeSet::new())
        .unwrap();
    let root = report
        .entries
        .iter()
        .find(|entry| entry.id == "root")
        .unwrap();
    let topic = report
        .entries
        .iter()
        .find(|entry| entry.id == "topic")
        .unwrap();
    assert_eq!(root.reachable_items, 2);
    assert_eq!(topic.reachable_items, 1);
}

fn generated_api(identity: &str, provider: &str, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind: ReachableKind::Function,
        origin: ReachableOrigin::Generator(provider.into()),
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
        definition: None,
    }
}

fn contract(identity: &str, canonical_path: &str, aliases: &[&str]) -> ContractIdentity {
    ContractIdentity {
        identity: identity.into(),
        canonical_path: canonical_path.into(),
        aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
    }
}

fn manifest() -> ScopeManifest {
    ScopeManifest::from_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/scope-ratchet/api-scopes.toml"),
    )
    .unwrap()
}

fn surface() -> Vec<ReachableApi> {
    vec![
        api(
            "sand_core::cmd::say",
            &["sand::cmd::say", "sand::command::say"],
        ),
        api("sand_core::state::Flag", &["sand::state::Flag"]),
        api("sand_core::state::Timer", &["sand::state::Timer"]),
        api("sand_core::event::Event", &["sand::event::Event"]),
        api("sand_core::systems::Damage", &["sand::systems::Damage"]),
    ]
}

fn command_contract() -> ContractIdentity {
    contract(
        "sand_core::cmd::say",
        "sand::command::say",
        &["sand::cmd::say"],
    )
}

#[test]
fn deterministic_report_counts_pending_and_feature_scopes() {
    let report = manifest()
        .evaluate(&surface(), &[command_contract()], &BTreeSet::new())
        .unwrap();
    assert_eq!(report.pending_items, 3);
    assert_eq!(report.enforced_items, 1);
    assert_eq!(
        report.to_string(),
        concat!(
            "command-source module=sand::command state=enforced tier=author provider=source precedence=50 recursive=true active=true items=1 contracted=1 aliases=sand::cmd features=-\n",
            "event-source module=sand::event state=pending tier=author provider=source precedence=50 recursive=true active=true items=1 contracted=0 aliases=- features=-\n",
            "state-source module=sand::state state=pending tier=advanced provider=source precedence=50 recursive=true active=true items=2 contracted=0 aliases=- features=-\n",
            "systems-source module=sand::systems state=pending tier=author provider=source precedence=50 recursive=true active=false items=0 contracted=0 aliases=- features=systems-all\n",
            "totals pending_scopes=3 pending_items=3 enforced_items=1 pending_scope_ceiling=3 pending_item_ceiling=3"
        )
    );

    let enabled = BTreeSet::from(["systems-all".to_owned()]);
    let failures = manifest()
        .evaluate(&surface(), &[command_contract()], &enabled)
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::PendingCeilingExceeded {
        actual: 4,
        ceiling: 3,
    }));
}

#[test]
fn enforced_scope_rejects_existing_and_new_uncontracted_items() {
    let failures = manifest()
        .evaluate(&surface(), &[], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::MissingContracts {
        scope: "sand::command".into(),
        identities: vec!["sand_core::cmd::say".into()],
    }));

    let mut expanded = surface();
    expanded.push(api(
        "sand_core::cmd::forgotten",
        &["sand::command::forgotten"],
    ));
    let failures = manifest()
        .evaluate(&expanded, &[command_contract()], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::MissingContracts {
        scope: "sand::command".into(),
        identities: vec!["sand_core::cmd::forgotten".into()],
    }));
}

#[test]
fn enforced_scope_rejects_incomplete_alias_metadata() {
    let incomplete = contract("sand_core::cmd::say", "sand::command::say", &[]);
    let failures = manifest()
        .evaluate(&surface(), &[incomplete], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.iter().any(|failure| matches!(
        failure,
        ScopeFailure::InvalidContracts { scope, diagnostics }
            if scope == "sand::command"
                && diagnostics.iter().any(|message| message.contains("aliases differ"))
    )));
}

#[test]
fn enforced_to_pending_regression_exceeds_committed_baseline() {
    let source = include_str!("fixtures/scope-ratchet/api-scopes.toml");
    // Exercise the aggregate ceilings independently of the stronger
    // per-scope ledger check below.
    let regressed = source
        .replace("state = \"enforced\"", "state = \"pending\"")
        .replace(
            "enforced_scope_baseline = [\"command-source\"]",
            "enforced_scope_baseline = []",
        );
    let failures = ScopeManifest::from_toml(&regressed)
        .unwrap()
        .evaluate(&surface(), &[], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::PendingCeilingExceeded {
        actual: 4,
        ceiling: 3,
    }));
    assert!(
        failures.contains(&ScopeFailure::PendingScopeCeilingExceeded {
            actual: 4,
            ceiling: 3,
        })
    );
}

#[test]
fn per_scope_baseline_rejects_an_aggregate_neutral_state_swap() {
    let source = include_str!("fixtures/scope-ratchet/api-scopes.toml");
    let swapped = source
        .replacen(
            "id = \"command-source\"\ncanonical_module = \"sand::command\"\nstate = \"enforced\"",
            "id = \"command-source\"\ncanonical_module = \"sand::command\"\nstate = \"pending\"",
            1,
        )
        .replacen(
            "id = \"event-source\"\ncanonical_module = \"sand::event\"\nstate = \"pending\"",
            "id = \"event-source\"\ncanonical_module = \"sand::event\"\nstate = \"enforced\"",
            1,
        );

    assert_eq!(
        ScopeManifest::from_toml(&swapped).unwrap_err(),
        ScopeFailure::RecordedScopeNotEnforced("command-source".into())
    );
}

#[test]
fn invalid_or_item_level_manifest_entries_are_rejected() {
    let base = r#"
        schema_version = 1
        static_surface_items = 0
        enforced_scope_baseline = []
        pending_scope_ceiling = 1
        pending_item_ceiling = 0
        [[scope]]
        id = "command"
        canonical_module = "sand::command"
        state = "pending"
        tier = "author"
        provider = "source"
    "#;
    let duplicate = format!(
        "{base}\n[[scope]]\nid = \"command-two\"\ncanonical_module = \"sand::command\"\nstate = \"enforced\"\ntier = \"author\"\nprovider = \"source\""
    );
    assert_eq!(
        ScopeManifest::from_toml(&duplicate).unwrap_err(),
        ScopeFailure::DuplicateCanonicalScope("sand::command".into())
    );

    let overlap = format!(
        "{base}\n[[scope]]\nid = \"execute\"\ncanonical_module = \"sand::command::execute\"\nstate = \"enforced\"\ntier = \"author\"\nprovider = \"source\""
    );
    assert!(matches!(
        ScopeManifest::from_toml(&overlap),
        Err(ScopeFailure::OverlappingScopes { .. })
    ));

    let invalid_alias = base.replace(
        "provider = \"source\"",
        "provider = \"source\"\naliases = [\"sand::bad-alias\"]",
    );
    assert!(matches!(
        ScopeManifest::from_toml(&invalid_alias),
        Err(ScopeFailure::InvalidAlias { .. })
    ));

    let item_exemption = base.replace(
        "provider = \"source\"",
        "provider = \"source\"\nexemptions = [\"sand::command::legacy\"]",
    );
    assert!(matches!(
        ScopeManifest::from_toml(&item_exemption),
        Err(ScopeFailure::Toml(_))
    ));
}

#[test]
fn root_direct_and_generator_scopes_partition_the_same_module() {
    let source = r#"
        schema_version = 1
        static_surface_items = 2
        enforced_scope_baseline = []
        pending_scope_ceiling = 3
        pending_item_ceiling = 2

        [[scope]]
        id = "root-source"
        canonical_module = "sand"
        state = "pending"
        tier = "author"
        provider = "source"
        recursive = false

        [[scope]]
        id = "command-source"
        canonical_module = "sand::command"
        state = "pending"
        tier = "author"
        provider = "source"

        [[scope]]
        id = "command-generated"
        canonical_module = "sand::command"
        state = "pending"
        tier = "author"
        provider = "generator:commands"
    "#;
    let surface = vec![
        api("sand_macros::function", &["sand::function"]),
        generated_api(
            "sand_commands::generated::Say",
            "commands",
            &["sand::command::say"],
        ),
    ];
    let report = ScopeManifest::from_toml(source)
        .unwrap()
        .evaluate(&surface, &[], &BTreeSet::new())
        .unwrap();
    assert_eq!(report.pending_items, 2);
    assert_eq!(report.entries[0].id, "command-generated");
    assert_eq!(report.entries[1].id, "command-source");
    assert_eq!(report.entries[2].id, "root-source");
}

#[test]
fn parametric_scope_cannot_be_enforced_without_connected_consumer_audit() {
    let pending = r#"
        schema_version = 1
        static_surface_items = 0
        enforced_scope_baseline = []
        pending_scope_ceiling = 1
        pending_item_ceiling = 0

        [[scope]]
        id = "generated-state-derive"
        canonical_module = "sand"
        state = "pending"
        tier = "ordinary"
        provider = "generator:state_derive"
        enforcement = "consumer_build"
        recursive = false
    "#;
    let manifest = ScopeManifest::from_toml(pending).unwrap();
    assert!(manifest.evaluate(&[], &[], &BTreeSet::new()).is_ok());

    let enforced = pending
        .replace("state = \"pending\"", "state = \"enforced\"")
        .replace(
            "enforced_scope_baseline = []",
            "enforced_scope_baseline = [\"generated-state-derive\"]",
        )
        .replace("pending_scope_ceiling = 1", "pending_scope_ceiling = 0");
    let manifest = ScopeManifest::from_toml(&enforced).unwrap();
    let failures = manifest.evaluate(&[], &[], &BTreeSet::new()).unwrap_err();
    assert!(
        failures.contains(&ScopeFailure::DisconnectedEnforcedProvider(
            "generated-state-derive".into()
        ))
    );
    assert!(failures.contains(&ScopeFailure::EmptyEnforcedScope(
        "generated-state-derive".into()
    )));

    let disguised = enforced.replace(
        "enforcement = \"consumer_build\"",
        "enforcement = \"facade_graph\"",
    );
    let failures = ScopeManifest::from_toml(&disguised)
        .unwrap()
        .evaluate(&[], &[], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::EmptyEnforcedScope(
        "generated-state-derive".into()
    )));

    let connected = BTreeSet::from(["generated-state-derive".to_owned()]);
    assert!(
        manifest
            .evaluate_with_provider_audits(&[], &[], &BTreeSet::new(), &connected)
            .is_ok()
    );
}

#[test]
fn unscoped_reachable_items_fail_the_ratchet() {
    let mut surface = surface();
    surface.push(api("sand_core::vfx::Vfx", &["sand::vfx::Vfx"]));
    let failures = manifest()
        .evaluate(&surface, &[command_contract()], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::UnscopedItems(vec![
        "sand_core::vfx::Vfx".into()
    ])));
}

#[test]
fn duplicate_alias_in_one_scope_is_rejected_but_shared_prelude_alias_is_allowed() {
    let duplicate = r#"
        schema_version = 1
        static_surface_items = 0
        enforced_scope_baseline = []
        pending_scope_ceiling = 1
        pending_item_ceiling = 0
        [[scope]]
        id = "predicate"
        canonical_module = "sand::predicate"
        state = "pending"
        tier = "author"
        provider = "source"
        aliases = ["sand::prelude", "sand::prelude"]
    "#;
    assert_eq!(
        ScopeManifest::from_toml(duplicate).unwrap_err(),
        ScopeFailure::DuplicateAlias("sand::prelude".into())
    );

    let shared = duplicate
        .replace(", \"sand::prelude\"", "")
        .replace(
            "aliases = [\"sand::prelude\"]",
            "aliases = [\"sand::prelude\"]\n[[scope]]\nid = \"event\"\ncanonical_module = \"sand::event\"\nstate = \"pending\"\ntier = \"author\"\nprovider = \"source\"\naliases = [\"sand::prelude\"]",
        );
    ScopeManifest::from_toml(&shared).unwrap();
}

#[test]
fn canonical_precedence_assigns_aliases_to_one_topic_scope() {
    let source = r#"
        schema_version = 1
        static_surface_items = 1
        enforced_scope_baseline = []
        pending_scope_ceiling = 3
        pending_item_ceiling = 1

        [[scope]]
        id = "prelude"
        canonical_module = "sand::prelude"
        state = "pending"
        tier = "author"
        provider = "source"
        precedence = 0

        [[scope]]
        id = "component"
        canonical_module = "sand::component"
        state = "pending"
        tier = "author"
        provider = "source"

        [[scope]]
        id = "predicate"
        canonical_module = "sand::predicate"
        state = "pending"
        tier = "author"
        provider = "source"
        precedence = 100
    "#;
    let reachable = [api(
        "sand_components::predicate::Predicate",
        &[
            "sand::component::Predicate",
            "sand::predicate::Predicate",
            "sand::prelude::Predicate",
        ],
    )];
    let report = ScopeManifest::from_toml(source)
        .unwrap()
        .evaluate(&reachable, &[], &BTreeSet::new())
        .unwrap();
    assert_eq!(report.pending_items, 1);
    assert_eq!(
        report
            .entries
            .iter()
            .find(|entry| entry.id == "predicate")
            .unwrap()
            .reachable_items,
        1
    );
    assert_eq!(
        report
            .entries
            .iter()
            .find(|entry| entry.id == "component")
            .unwrap()
            .reachable_items,
        0
    );
}

#[test]
fn topic_precedence_rejects_selecting_the_prelude_alias_as_canonical() {
    let source = r#"
        schema_version = 1
        static_surface_items = 1
        enforced_scope_baseline = []
        pending_scope_ceiling = 2
        pending_item_ceiling = 1

        [[scope]]
        id = "prelude"
        canonical_module = "sand::prelude"
        state = "pending"
        tier = "author"
        provider = "source"
        precedence = 0

        [[scope]]
        id = "predicate"
        canonical_module = "sand::predicate"
        state = "pending"
        tier = "author"
        provider = "source"
        precedence = 100
    "#;
    let reachable = [api(
        "sand_components::predicate::Predicate",
        &["sand::predicate::Predicate", "sand::prelude::Predicate"],
    )];
    let prelude_canonical = contract(
        "sand_components::predicate::Predicate",
        "sand::prelude::Predicate",
        &["sand::predicate::Predicate"],
    );
    let failures = ScopeManifest::from_toml(source)
        .unwrap()
        .evaluate(&reachable, &[prelude_canonical], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::NonCanonicalContractPath {
        identity: "sand_components::predicate::Predicate".into(),
        selected: "sand::prelude::Predicate".into(),
        required_scope: "sand::predicate".into(),
    }));
}

#[test]
fn nonrecursive_scope_rejects_descendant_alias_as_canonical() {
    let source = r#"
        schema_version = 1
        static_surface_items = 1
        enforced_scope_baseline = []
        pending_scope_ceiling = 1
        pending_item_ceiling = 1

        [[scope]]
        id = "root"
        canonical_module = "sand"
        state = "pending"
        tier = "author"
        provider = "source"
        recursive = false
    "#;
    let reachable = [api(
        "sand_components::Predicate",
        &["sand::Predicate", "sand::prelude::Predicate"],
    )];
    let descendant_canonical = contract(
        "sand_components::Predicate",
        "sand::prelude::Predicate",
        &["sand::Predicate"],
    );

    let failures = ScopeManifest::from_toml(source)
        .unwrap()
        .evaluate(&reachable, &[descendant_canonical], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::NonCanonicalContractPath {
        identity: "sand_components::Predicate".into(),
        selected: "sand::prelude::Predicate".into(),
        required_scope: "sand".into(),
    }));
}
