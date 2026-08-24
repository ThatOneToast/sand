use sand_api_contract::{ApiCatalog, ApiKind};

#[test]
fn lower_crate_registrations_are_installed_in_the_facade_binary() {
    assert_eq!(sand::Widget.value(), 7);
    let compiled_surface_items = sand_api_contract::inventory::iter::<
        sand_api_contract::ApiRegistration,
    >
    .into_iter()
    .count();
    let catalog = ApiCatalog::from_registrations(
        "fixture",
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
    let widget = catalog.find("sand::Widget").unwrap();
    assert_eq!(widget.kind, ApiKind::Struct);
    assert_eq!(widget.aliases, ["sand::prelude::Widget".to_owned()]);
    let method = catalog.find("sand::Widget::value").unwrap();
    assert_eq!(method.kind, ApiKind::Method);
    assert!(method.parameters.is_empty());
}
