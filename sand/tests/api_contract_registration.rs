use sand::api;
use sand_api_contract::{ApiCatalog, ApiKind};

#[api(
    path = "sand::testing::contract_fixture",
    aliases = ["sand::prelude::contract_fixture"],
    summary = "Builds a contract-registry fixture.",
    context = "The fixture proves static API metadata is linked without parsing Rust source.",
    minecraft = "It models metadata only and emits no datapack resource.",
    use_when = ["Testing installed API metadata"],
    avoid_when = ["Authoring a datapack"],
    params(value = "The fixture value returned to the caller."),
    returns = "The unchanged fixture value.",
    example = "contract_fixture(1)",
)]
fn contract_fixture(value: u32) -> u32 {
    value
}

struct Fixture;

impl Fixture {
    #[api(
        path = "sand::testing::Fixture::value",
        summary = "Reads the fixture value.",
        context = "The method contract proves inherent-method registrations are collected.",
        minecraft = "It models metadata only and emits no datapack resource.",
        use_when = ["Testing method metadata"],
        avoid_when = ["Authoring a datapack"],
        returns = "A stable fixture value.",
        example = "Fixture.value()",
    )]
    fn value(&self) -> u32 {
        1
    }
}

#[test]
fn generated_registrations_build_an_installed_catalog() {
    assert_eq!(contract_fixture(7), 7);
    assert_eq!(Fixture.value(), 1);

    let coverage = sand::__private::api_contract::installed_coverage();
    assert_eq!(coverage.static_surface_items, 11_736);
    assert_eq!(coverage.pending_item_ceiling, 11_613);
    assert_eq!(coverage.pending_scope_ceiling, 38);
    assert_eq!(coverage.pending_scopes.len(), 38);
    let catalog = ApiCatalog::installed_with_coverage(env!("CARGO_PKG_VERSION"), coverage).unwrap();
    let function = catalog.find("sand::prelude::contract_fixture").unwrap();
    assert_eq!(function.kind, ApiKind::Function);
    assert_eq!(function.parameters[0].name, "value");
    assert_eq!(
        catalog.find("sand::testing::Fixture::value").unwrap().kind,
        ApiKind::Method
    );

    let predicate = catalog
        .find("sand::prelude::Predicate::new")
        .expect("definition-owned predicate method contract");
    assert_eq!(predicate.canonical_path, "sand::predicate::Predicate::new");
    assert_eq!(predicate.kind, ApiKind::Method);
    assert_eq!(
        predicate
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        ["location", "root"]
    );

    let json_once = catalog.to_json_pretty().unwrap();
    let json_twice = ApiCatalog::installed_with_coverage(
        env!("CARGO_PKG_VERSION"),
        sand::__private::api_contract::installed_coverage(),
    )
    .unwrap()
    .to_json_pretty()
    .unwrap();
    assert_eq!(json_once.as_bytes(), json_twice.as_bytes());
}
