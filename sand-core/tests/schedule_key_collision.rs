//! Export-level coverage for collision-safe `#[schedule]` objective keys.
//!
//! The unit tests in `compiler::export::schedules` inject a controlled key
//! source to exercise forced collisions and probe exhaustion. These tests
//! cover the real, production key source end-to-end through the public
//! fallible export API: distinct schedules must never share generated
//! objectives, names must stay within Minecraft's 16-character limit, and the
//! output must stay byte-stable across repeated exports.

use std::collections::BTreeSet;

use sand_core::ScheduleDescriptor;

fn body() -> Vec<String> {
    vec!["say tick".to_string()]
}

macro_rules! schedule {
    ($path:literal, $total:expr, $every:expr) => {
        sand_core::inventory::submit! {
            ScheduleDescriptor {
                path: $path,
                total_ticks: $total,
                every: $every,
                make: body,
            }
        }
    };
}

schedule!("collision_probe_a", 20, 1);
schedule!("collision_probe_b", 40, 3);
schedule!("collision_probe_c", 60, 5);
schedule!("zzz_late_registration", 80, 7);
schedule!("aaa_early_registration", 100, 2);

const PATHS: [&str; 5] = [
    "collision_probe_a",
    "collision_probe_b",
    "collision_probe_c",
    "zzz_late_registration",
    "aaa_early_registration",
];

/// Mirror of the exporter's canonical key hash (FNV-1a 32-bit, 8 hex chars).
/// Duplicated here on purpose: it pins the *observable* naming contract from
/// outside the crate, so an accidental change to the internal helper cannot
/// silently rewrite every generated objective name.
fn expected_key(path: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in path.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

fn export() -> Vec<serde_json::Value> {
    let json =
        sand_core::try_export_components_json("schedkeys").expect("schedule export must succeed");
    serde_json::from_str(&json).expect("export is valid JSON")
}

fn function_content(records: &[serde_json::Value], path: &str) -> String {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated function {path}"))
        .to_string()
}

/// Every `scoreboard objectives add __ss_..._x dummy` line from the generated
/// load function.
fn declared_schedule_objectives(records: &[serde_json::Value]) -> Vec<String> {
    function_content(records, "__sand_sched_init")
        .lines()
        .filter_map(|line| {
            line.strip_prefix("scoreboard objectives add ")
                .and_then(|rest| rest.strip_suffix(" dummy"))
        })
        .filter(|name| name.starts_with("__ss_"))
        .map(str::to_string)
        .collect()
}

#[test]
fn distinct_schedules_never_share_generated_objectives() {
    let records = export();
    let objectives = declared_schedule_objectives(&records);
    assert!(!objectives.is_empty(), "schedules must declare objectives");

    let unique: BTreeSet<&String> = objectives.iter().collect();
    assert_eq!(
        unique.len(),
        objectives.len(),
        "generated schedule objectives must be unique: {objectives:?}"
    );
}

#[test]
fn generated_objectives_fit_the_minecraft_length_limit() {
    let records = export();
    for name in declared_schedule_objectives(&records) {
        assert!(
            name.len() <= 16,
            "objective `{name}` ({} chars) exceeds Minecraft's 16-char limit",
            name.len()
        );
    }
}

#[test]
fn non_colliding_schedules_keep_the_plain_path_hash() {
    let records = export();
    let tick = function_content(&records, "__sand_sched_tick");
    let init = function_content(&records, "__sand_sched_init");

    for path in PATHS {
        let key = expected_key(path);
        let obj_t = format!("__ss_{key}_t");
        assert!(
            init.contains(&format!("scoreboard objectives add {obj_t} dummy")),
            "expected `{obj_t}` for schedule `{path}` in:\n{init}"
        );
        assert!(
            tick.contains(&format!("run function schedkeys:{path}")),
            "expected tick dispatch for schedule `{path}`"
        );

        let start = function_content(&records, &format!("{path}_start"));
        assert!(start.contains(&obj_t), "start function must use `{obj_t}`");
        let stop = function_content(&records, &format!("{path}_stop"));
        assert_eq!(stop, format!("scoreboard players set @s {obj_t} 0"));
    }
}

#[test]
fn allocation_is_stable_across_repeated_exports() {
    let first = sand_core::try_export_components_json("schedkeys").unwrap();
    for _ in 0..8 {
        let again = sand_core::try_export_components_json("schedkeys").unwrap();
        assert_eq!(first, again, "repeated exports must be byte-identical");
    }
}

#[test]
fn record_order_follows_registration_not_key_order() {
    let records = export();
    let schedule_bodies: Vec<&str> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .filter_map(|record| record["path"].as_str())
        .filter(|path| PATHS.contains(path))
        .collect();

    let inventory_order: Vec<&str> = sand_core::inventory::iter::<ScheduleDescriptor>()
        .map(|desc| desc.path)
        .filter(|path| PATHS.contains(path))
        .collect();

    assert_eq!(
        schedule_bodies, inventory_order,
        "records must be emitted in inventory order, not sorted key-allocation order"
    );
}
