use sand::api;
use sand_api_contract::{ApiCatalog, ApiKind};

#[api(
    path = "sand::testing::Mode",
    aliases = ["sand::prelude::Mode"],
    summary = "Selects how the fixture emits a generated resource.",
    context = "The fixture exercises nested variant registrations from one authoritative parent contract.",
    minecraft = "Each mode selects a distinct generated-resource behavior.",
    use_when = ["Testing enum member metadata"],
    avoid_when = ["Authoring production datapacks"],
    example = "Mode::Enabled",
    variants(
        Enabled = "Emits the fixture resource into the datapack.",
        Disabled = "Suppresses the fixture resource during export."
    )
)]
enum Mode {
    Enabled,
    Disabled,
}

#[api(
    path = "sand::testing::Settings",
    aliases = ["sand::prelude::Settings"],
    summary = "Stores the fixture's public resource settings.",
    context = "The fixture exercises nested field registrations from one authoritative parent contract.",
    minecraft = "The values select a resource identity and whether export emits it.",
    use_when = ["Testing public field metadata"],
    avoid_when = ["Authoring production datapacks"],
    example = "Settings { id: 7, enabled: true }",
    fields(
        id = "Identifies the fixture resource deterministically.",
        enabled = "Controls whether the fixture resource is emitted."
    )
)]
struct Settings {
    pub id: u32,
    pub enabled: bool,
}

impl Settings {
    #[api(
        kind = "associated_const",
        path = "sand::testing::Settings::DEFAULT_ENABLED",
        summary = "Defines the fixture's default export state.",
        context = "Constructors use one shared default when author code omits an explicit choice.",
        minecraft = "Enabled resources are eligible for datapack emission.",
        use_when = ["Implementing fixture constructors"],
        avoid_when = ["Inspecting an existing fixture value"],
        example = "Settings::DEFAULT_ENABLED",
    )]
    const DEFAULT_ENABLED: bool = true;
}

trait Schema {
    #[api(
        path = "sand::testing::Schema::Id",
        summary = "Names the identifier type carried by a fixture schema.",
        context = "The associated type preserves identifier typing through generic schema code.",
        minecraft = "The identifier ultimately selects a generated resource path.",
        use_when = ["Implementing a fixture schema"],
        avoid_when = ["Passing an unvalidated resource string"],
        example = "type Id = u32;",
    )]
    type Id;
}

struct TestSchema;

impl Schema for TestSchema {
    type Id = u32;
}

#[test]
fn parent_contracts_register_nested_members_with_derived_shapes() {
    let _: <TestSchema as Schema>::Id = 7;
    let _ = [Mode::Enabled, Mode::Disabled];
    let settings = Settings {
        id: 7,
        enabled: true,
    };
    assert_eq!(settings.id, 7);
    assert!(settings.enabled);

    let compiled_surface_items =
        sand_api_contract::inventory::iter::<sand_api_contract::ApiRegistration>
            .into_iter()
            .count();
    let catalog = ApiCatalog::from_registrations(
        env!("CARGO_PKG_VERSION"),
        sand_api_contract::ApiConfiguration {
            surface_profile: "test".into(),
            minecraft_version: "test".into(),
            cargo_features: Vec::new(),
            placeholder_codegen: false,
            compiled_surface_items,
        },
        sand_api_contract::inventory::iter::<sand_api_contract::ApiRegistration>,
    )
    .unwrap();
    let enabled = catalog.find("sand::prelude::Mode::Enabled").unwrap();
    assert_eq!(enabled.canonical_path, "sand::testing::Mode::Enabled");
    assert_eq!(enabled.canonical_module, "sand::testing::Mode");
    assert_eq!(enabled.kind, ApiKind::Variant);
    assert_eq!(
        enabled.summary,
        "Emits the fixture resource into the datapack."
    );
    assert!(enabled.signature.contains("Enabled"));

    let field = catalog.find("sand::prelude::Settings::enabled").unwrap();
    assert_eq!(field.canonical_path, "sand::testing::Settings::enabled");
    assert_eq!(field.kind, ApiKind::Field);
    assert_eq!(
        field.signature.split_whitespace().collect::<String>(),
        "pubenabled:bool"
    );

    let constant = catalog
        .find("sand::testing::Settings::DEFAULT_ENABLED")
        .unwrap();
    assert_eq!(constant.kind, ApiKind::AssociatedConst);
    assert!(std::hint::black_box(Settings::DEFAULT_ENABLED));

    let associated_type = catalog.find("sand::testing::Schema::Id").unwrap();
    assert_eq!(associated_type.kind, ApiKind::AssociatedType);
}
