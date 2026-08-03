use std::collections::BTreeSet;
use std::path::PathBuf;

use sand_api_enforce::{
    ContractIdentity, ReachableApi, ReachableKind, ScopeFailure, ScopeManifest,
};

fn api(identity: &str, paths: &[&str]) -> ReachableApi {
    ReachableApi {
        identity: identity.into(),
        kind: ReachableKind::Function,
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

#[test]
fn deterministic_report_counts_pending_and_ignores_out_of_scope_items() {
    let contracts = [contract(
        "sand_core::cmd::say",
        "sand::cmd::say",
        &["sand::command::say"],
    )];
    let report = manifest()
        .evaluate(&surface(), &contracts, &BTreeSet::new())
        .unwrap();
    assert_eq!(report.pending_items, 2);
    assert_eq!(report.enforced_items, 1);
    assert_eq!(
        report.to_string(),
        concat!(
            "sand::cmd state=enforced tier=author active=true items=1 contracted=1 aliases=sand::command features=- generator=vanilla-commands\n",
            "sand::state state=pending tier=advanced active=true items=2 contracted=0 aliases=- features=- generator=-\n",
            "sand::systems state=pending tier=author active=false items=0 contracted=0 aliases=- features=systems-all generator=-\n",
            "totals pending=2 enforced=1 pending_ceiling=2"
        )
    );
    assert_eq!(report.to_string(), report.to_string());
}

#[test]
fn feature_scopes_are_counted_only_when_active() {
    let enabled = BTreeSet::from(["systems-all".to_owned()]);
    let failures = manifest().evaluate(&surface(), &[], &enabled).unwrap_err();
    assert!(failures.contains(&ScopeFailure::MissingContracts {
        scope: "sand::cmd".into(),
        identities: vec!["sand_core::cmd::say".into()],
    }));
    assert!(failures.contains(&ScopeFailure::PendingCeilingExceeded {
        actual: 3,
        ceiling: 2,
    }));
}

#[test]
fn enforced_scope_rejects_existing_and_new_uncontracted_items() {
    let current = surface();
    let failures = manifest()
        .evaluate(&current, &[], &BTreeSet::new())
        .unwrap_err();
    assert!(matches!(
        &failures[0],
        ScopeFailure::MissingContracts { scope, identities }
            if scope == "sand::cmd" && identities == &["sand_core::cmd::say"]
    ));

    let contracts = [contract(
        "sand_core::cmd::say",
        "sand::cmd::say",
        &["sand::command::say"],
    )];
    let mut expanded = current;
    expanded.push(api("sand_core::cmd::forgotten", &["sand::cmd::forgotten"]));
    let failures = manifest()
        .evaluate(&expanded, &contracts, &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::MissingContracts {
        scope: "sand::cmd".into(),
        identities: vec!["sand_core::cmd::forgotten".into()],
    }));
}

#[test]
fn pending_scope_reports_items_without_requiring_contracts() {
    let contracts = [contract(
        "sand_core::cmd::say",
        "sand::cmd::say",
        &["sand::command::say"],
    )];
    let report = manifest()
        .evaluate(&surface(), &contracts, &BTreeSet::new())
        .unwrap();
    let state = report
        .entries
        .iter()
        .find(|entry| entry.canonical_module == "sand::state")
        .unwrap();
    assert_eq!(state.reachable_items, 2);
    assert_eq!(state.contracted_items, 0);
}

#[test]
fn enforced_to_pending_regression_exceeds_committed_baseline() {
    let source = include_str!("fixtures/scope-ratchet/api-scopes.toml");
    let regressed = source.replace("state = \"enforced\"", "state = \"pending\"");
    let manifest = ScopeManifest::from_toml(&regressed).unwrap();
    let failures = manifest
        .evaluate(&surface(), &[], &BTreeSet::new())
        .unwrap_err();
    assert!(failures.contains(&ScopeFailure::PendingCeilingExceeded {
        actual: 3,
        ceiling: 2,
    }));
}

#[test]
fn duplicate_overlapping_scopes_and_invalid_aliases_are_rejected() {
    let base = r#"
        schema_version = 1
        pending_item_ceiling = 0
        [[scope]]
        canonical_module = "sand::cmd"
        state = "pending"
        tier = "author"
    "#;
    let duplicate = format!(
        "{base}\n[[scope]]\ncanonical_module = \"sand::cmd\"\nstate = \"enforced\"\ntier = \"author\""
    );
    assert_eq!(
        ScopeManifest::from_toml(&duplicate).unwrap_err(),
        ScopeFailure::DuplicateCanonicalScope("sand::cmd".into())
    );

    let overlap = format!(
        "{base}\n[[scope]]\ncanonical_module = \"sand::cmd::execute\"\nstate = \"enforced\"\ntier = \"author\""
    );
    assert!(matches!(
        ScopeManifest::from_toml(&overlap),
        Err(ScopeFailure::OverlappingScopes { .. })
    ));

    let invalid_alias = base.replace(
        "tier = \"author\"",
        "tier = \"author\"\naliases = [\"sand::bad-alias\"]",
    );
    assert!(matches!(
        ScopeManifest::from_toml(&invalid_alias),
        Err(ScopeFailure::InvalidAlias { .. })
    ));

    let item_exemption = base.replace(
        "tier = \"author\"",
        "tier = \"author\"\nexemptions = [\"sand::cmd::legacy\"]",
    );
    assert!(matches!(
        ScopeManifest::from_toml(&item_exemption),
        Err(ScopeFailure::Toml(_))
    ));
}
