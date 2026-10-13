//! Count-only native predicates still require the item-component capability.
use sand::prelude::*;

fn queries() -> Actions {
    Target::players().each(|player| {
        let predicate = ItemPredicate::new().count_min(2);
        mcfunction![
            when(player.inventory().contains(&predicate)).then_all(["say present"]);
            when(player.inventory().offhand().matches(predicate).unwrap()).then_all(["say equipped"]);
        ]
    })
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "queries", make: queries,
    }
}

#[test]
fn count_only_queries_reject_unsupported_profiles_without_leaking_resources() {
    let caps = sand::version::VersionCaps::all_disabled();
    let error = sand_core::try_export_components_for_version("count_profile", &caps, "999.0", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("999.0"), "{error}");
    assert!(error.contains("item_components"), "{error}");
    let records = sand_core::try_export_components("count_profile").unwrap();
    let text = records
        .iter()
        .filter(|r| r.dir == "function")
        .map(|r| r.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("clear @s *[minecraft:count~{\"min\":2}] 0"),
        "{text}"
    );
    assert!(
        text.contains(
            "if items entity @s weapon.offhand *[minecraft:count~{\"min\":2}] run return 1"
        ),
        "{text}"
    );
    assert!(!text.contains("__sand_local:"));
}
