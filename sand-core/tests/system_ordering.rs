use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "ordered", scope = entity)]
struct Health {
    #[state(default = 20)]
    value: Score,
}

#[system(tick, before = a_check_death)]
fn z_damage(query: Health) {
    query.each(|health| health.value.add(-1));
}

#[system(tick, after = z_damage, before = Finish::cleanup)]
fn a_check_death(query: Health) {
    query.each(|health| health.value.add(2));
}

struct Finish;
#[system]
impl Finish {
    #[tick(after = a_check_death, before = Self::done, every = 2)]
    fn cleanup(query: Health) {
        query.each(|health| health.value.add(3));
    }
    #[tick(after = Self::cleanup)]
    fn done(query: Health) {
        query.each(|health| health.value.add(4));
    }
}

#[test]
fn dependencies_emit_separate_ordered_scans_and_keep_cadences() {
    let first = sand::advanced::try_export_pack_json("ordered", "26.3").unwrap();
    let second = sand::advanced::try_export_pack_json("ordered", "26.3").unwrap();
    assert_eq!(first, second);
    let pack: serde_json::Value = serde_json::from_str(&first).unwrap();
    let systems = pack["runtime_report"]["provenance"]["systems"]
        .as_array()
        .unwrap();
    assert_eq!(systems.len(), 4, "dependencies are whole-scan barriers");
    for (system, expected) in systems.iter().zip([
        "z_damage",
        "a_check_death",
        "Finish::cleanup",
        "Finish::done",
    ]) {
        assert_eq!(
            system["owners"],
            serde_json::json!([format!("{}::{expected}", module_path!())])
        );
    }
    assert_eq!(systems[2]["every"], 2);
    let tick = pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == "__sand_system_tick")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let calls = systems
        .iter()
        .map(|system| tick.find(system["function"].as_str().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert!(calls.windows(2).all(|pair| pair[0] < pair[1]));
}
