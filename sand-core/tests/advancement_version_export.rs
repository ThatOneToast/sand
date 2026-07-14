//! Integration coverage for #231/#232: proves the *real* datapack export
//! pipeline (not just direct `AdvancementTrigger::render_for` calls) threads
//! the selected `VersionProfile` into advancement rendering, and that
//! unsupported conversions fail the whole export with an actionable
//! diagnostic instead of emitting weakened JSON.

use sand_core::advanced::ComponentFactory;
use sand_core::prelude::*;

fn filtered_placed_block() -> Advancement {
    Advancement::new(
        "advancement_export_test:filtered_placed_block"
            .parse()
            .unwrap(),
    )
    .criterion(
        "event",
        Criterion::new(AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(ItemPredicate::id("minecraft:white_wool").custom_data_key("elevator")),
            None,
            None,
        )),
    )
    .rewards(AdvancementRewards::new().function("advancement_export_test:on_placed"))
}

fn multi_criteria_advancement() -> Advancement {
    Advancement::new("advancement_export_test:multi_criteria".parse().unwrap())
        .criterion("a", Criterion::new(AdvancementTrigger::Tick))
        .criterion("b", Criterion::new(AdvancementTrigger::Impossible))
}

inventory::submit! {
    ComponentFactory { make: || Box::new(filtered_placed_block()) }
}
inventory::submit! {
    ComponentFactory { make: || Box::new(multi_criteria_advancement()) }
}

fn find_record<'a>(
    records: &'a [sand_core::ComponentRecord],
    path: &str,
) -> &'a sand_core::ComponentRecord {
    records
        .iter()
        .find(|r| r.dir == "advancement" && r.path == path)
        .unwrap_or_else(|| panic!("no advancement record for path {path}"))
}

#[test]
fn modern_profile_export_renders_location_check_and_match_tool() {
    let profile = VersionProfile::resolve(&MinecraftVersion::parse("26.2").unwrap()).unwrap();
    let records = sand_core::try_export_components_for_version(
        "advancement_export_test",
        &profile.caps(),
        &profile.resolved_name,
        profile.is_fallback,
    )
    .expect("modern-profile export must succeed");

    let record = find_record(&records, "filtered_placed_block");
    let json: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    let location = &json["criteria"]["event"]["conditions"]["location"];
    assert!(location.is_array(), "expected conditions.location array");
    let conditions: Vec<&str> = location
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["condition"].as_str().unwrap())
        .collect();
    assert!(conditions.contains(&"minecraft:location_check"));
    assert!(conditions.contains(&"minecraft:match_tool"));
    assert_eq!(json["requirements"], serde_json::json!([["event"]]));
}

#[test]
fn legacy_profile_export_fails_with_actionable_diagnostic_instead_of_weakened_json() {
    // 1.19.0 predates the 1.20.5 item-component system — Sand has no verified
    // representation for an item filter on this profile family, so the whole
    // export must fail loudly rather than silently emit an incorrect or
    // weakened `item` condition.
    let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.19.0").unwrap()).unwrap();
    assert!(!profile.supports_item_components);

    let error = sand_core::try_export_components_for_version(
        "advancement_export_test",
        &profile.caps(),
        &profile.resolved_name,
        profile.is_fallback,
    )
    .expect_err("legacy-profile export with an item filter must fail");

    let message = error.to_string();
    assert!(message.contains("minecraft:placed_block"));
    assert!(message.contains("pre-item-component"));
}

#[test]
fn multi_criterion_advancement_export_derives_requirements() {
    let profile = VersionProfile::resolve(&MinecraftVersion::parse("26.2").unwrap()).unwrap();

    // This profile also exercises `filtered_placed_block`, which would fail
    // export on a legacy profile — use the modern profile so both
    // process-global components (this test binary registers both via
    // `#[component]`) export successfully together.
    let records = sand_core::try_export_components_for_version(
        "advancement_export_test",
        &profile.caps(),
        &profile.resolved_name,
        profile.is_fallback,
    )
    .expect("modern-profile export must succeed");

    let record = find_record(&records, "multi_criteria");
    let json: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    let mut requirements: Vec<Vec<String>> =
        serde_json::from_value(json["requirements"].clone()).unwrap();
    for group in &mut requirements {
        group.sort();
    }
    assert_eq!(requirements, vec![vec!["a".to_string(), "b".to_string()]]);
}
