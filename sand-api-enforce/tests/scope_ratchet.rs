use std::collections::BTreeSet;
use std::path::PathBuf;

use sand_api_enforce::{
    ContractIdentity, ReachableApi, ReachableKind, ReachableOrigin, ScopeFailure, ScopeManifest,
};

fn api(identity: &str, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind: ReachableKind::Function,
        origin: ReachableOrigin::Source,
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
    }
}

fn generated_api(identity: &str, provider: &str, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind: ReachableKind::Function,
        origin: ReachableOrigin::Generator(provider.into()),
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
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
            "command-source module=sand::command state=enforced tier=author provider=source recursive=true active=true items=1 contracted=1 aliases=sand::cmd features=-\n",
            "event-source module=sand::event state=pending tier=author provider=source recursive=true active=true items=1 contracted=0 aliases=- features=-\n",
            "state-source module=sand::state state=pending tier=advanced provider=source recursive=true active=true items=2 contracted=0 aliases=- features=-\n",
            "systems-source module=sand::systems state=pending tier=author provider=source recursive=true active=false items=0 contracted=0 aliases=- features=systems-all\n",
            "totals pending=3 enforced=1 pending_ceiling=3"
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
fn enforced_to_pending_regression_exceeds_committed_baseline() {
    let source = include_str!("fixtures/scope-ratchet/api-scopes.toml");
    let regressed = source.replace("state = \"enforced\"", "state = \"pending\"");
    let failures = ScopeManifest::from_toml(&regressed)
        .unwrap()
        .evaluate(&surface(), &[], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::PendingCeilingExceeded {
        actual: 4,
        ceiling: 3,
    }));
}

#[test]
fn invalid_or_item_level_manifest_entries_are_rejected() {
    let base = r#"
        schema_version = 1
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
