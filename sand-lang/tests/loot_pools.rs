mod support;
use support::*;

#[test]
fn conditional_loot_pools_render_in_authored_order() {
    let analysis = analyze_one(include_str!("fixtures/loot_pools.sand"));
    let pack = compile(&analysis);
    let loot: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/language_probe/loot_table/supply.json"])
            .unwrap();
    let pools = loot["pools"].as_array().unwrap();
    assert_eq!(pools.len(), 5);
    assert_eq!(pools[0]["entries"][0]["name"], "minecraft:bread");
    assert_eq!(pools[1]["conditions"][0]["chance"], 0.0);
    assert_eq!(pools[2]["conditions"][1]["raining"], false);
    assert_eq!(pools[2]["entries"][0]["functions"][0]["count"], 3.0);
    assert_eq!(pools[3]["conditions"][0]["raining"], true);
    assert_eq!(pools[3]["conditions"][0]["thundering"], false);
    assert_eq!(pools[4]["conditions"][0]["thundering"], true);
}

#[test]
fn loot_pool_errors_point_to_the_invalid_value() {
    for (declaration, expected) in [
        (
            "items: [Item::Bread], pools: []",
            "use either pools or top-level items and rolls",
        ),
        ("pools: []", "a loot table needs at least one pool"),
        (
            "pools: [{ rolls: 2 }]",
            "a loot table needs at least one item",
        ),
        (
            "pools: [{ items: [Item::Bread], chance: 2 }]",
            "chance must be between zero and one",
        ),
        (
            "pools: [{ items: [Item::Bread], weather: Weather::Snow }]",
            "no weather named",
        ),
        (
            "pools: [{ items: [Item::Bread], rolls: 2, rolls: 3 }]",
            "duplicate loot pool field `rolls`",
        ),
    ] {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"loot_example\")]\nloot Crate {{ {declaration} }}"
        ));
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains(expected)),
            "{:?}",
            errors(&analysis)
        );
    }
}
