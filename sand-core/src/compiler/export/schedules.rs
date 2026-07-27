//! Schedule lowering phase of the export pipeline.
//!
//! Owns the collision-safe objective-key allocation for `#[schedule]`
//! descriptors, the record emission for the generated body / `_start` /
//! `_stop` functions, and the per-player tick maintenance commands for the
//! generated `__sand_sched_tick` function.
//!
//! # Key allocation
//! Each schedule owns two private scoreboard objectives, `__ss_<key>_t` and
//! `__ss_<key>_p`, where `<key>` is 8 hex chars so the objective name stays
//! inside Minecraft's 16-character limit (`__ss_` + 8 + `_t` = 15).
//!
//! The base key is a 32-bit FNV-1a hash of the schedule path, which — being
//! 32 bits — can collide between two distinct paths. A collision would make
//! two unrelated schedules silently share countdown/phase state, so keys are
//! not hashed independently per descriptor:
//! [`allocate_schedule_objective_keys`] walks
//! *all* schedule paths in sorted order and probes deterministically salted
//! rehashes until it finds an unclaimed key. Sorted iteration makes the
//! resulting mapping independent of inventory registration order, thread
//! scheduling, and hash-map iteration order, and the first probe is the plain
//! path hash so non-colliding packs keep byte-identical output.
//!
//! If every probe collides the export fails with a diagnostic naming both
//! paths, the key, and the affected objectives, rather than emitting a pack
//! with shared state.

use std::collections::{BTreeMap, BTreeSet};

use super::records::{ComponentRecord, ExportResult};
use crate::component::ComponentExportError;

/// Maximum number of deterministic rehash probes tried for a single schedule
/// path before allocation is declared impossible and the export fails.
pub(crate) const SCHEDULE_KEY_PROBE_LIMIT: u32 = 64;

/// Minecraft's hard limit on scoreboard objective name length.
pub(crate) const OBJECTIVE_NAME_LIMIT: usize = 16;

/// Injectable key generator: `(path, attempt) -> key`.
///
/// Attempt `0` must be the canonical key for the path; higher attempts are
/// alternative candidates tried in order when lower attempts are already
/// claimed. Production always uses [`schedule_key_attempt`]; tests inject a
/// controlled source to exercise collision handling without brute-forcing a
/// real FNV-1a collision.
pub(crate) type ScheduleKeySource = fn(&str, u32) -> String;

/// FNV-1a 32-bit hash rendered as 8 lowercase hex chars.
///
/// This is the canonical implementation for the export pipeline;
/// [`super::armor`] keeps a private mirror of the same algorithm for entity
/// tag names so both generated namespaces hash identically.
fn fnv1a_hex(value: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in value.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

/// Compute the canonical 8-hex-char key for a schedule path (FNV-1a 32-bit).
///
/// This is probe attempt `0` of [`schedule_key_attempt`]: whenever a path's
/// base key is unclaimed — the overwhelmingly common case — the allocated key
/// is exactly this value, so generated objective names are unchanged from
/// before collision handling existed.
pub(crate) fn schedule_key(path: &str) -> String {
    fnv1a_hex(path)
}

/// Production [`ScheduleKeySource`]: attempt `0` is [`schedule_key`], every
/// later attempt is the hash of a deterministically salted path (`path#n`).
pub(crate) fn schedule_key_attempt(path: &str, attempt: u32) -> String {
    if attempt == 0 {
        schedule_key(path)
    } else {
        fnv1a_hex(&format!("{path}#{attempt}"))
    }
}

/// The two generated objective names for an allocated schedule key.
pub(crate) fn schedule_objective_names(key: &str) -> (String, String) {
    (format!("__ss_{key}_t"), format!("__ss_{key}_p"))
}

/// Diagnostic error for the schedule phase of the export pipeline.
pub(crate) fn schedule_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "schedules")
            .expect("fixed schedule resource location is valid"),
        kind: "schedule".to_string(),
        field: "objectives".to_string(),
        message: message.into(),
    }
}

/// Reject any key whose generated objective names would exceed Minecraft's
/// 16-character limit.
///
/// Keys are 8 hex chars by construction, so `__ss_` (5) + 8 + `_t` (2) = 15
/// always fits; this is an explicit guard so a future change to the key
/// format cannot silently emit an objective Minecraft will refuse.
fn ensure_objective_names_fit(path: &str, key: &str) -> ExportResult<()> {
    let (obj_t, obj_p) = schedule_objective_names(key);
    if obj_t.len() > OBJECTIVE_NAME_LIMIT || obj_p.len() > OBJECTIVE_NAME_LIMIT {
        return Err(schedule_export_error(format!(
            "schedule `{path}` generated key `{key}`, whose objectives `{obj_t}` / `{obj_p}` \
             exceed Minecraft's {OBJECTIVE_NAME_LIMIT}-character scoreboard objective limit"
        )));
    }
    Ok(())
}

/// Deterministically allocate a unique objective key for every schedule path.
///
/// Paths are de-duplicated and visited in lexicographic order via a
/// [`BTreeSet`], so the returned mapping depends only on the *set* of paths —
/// never on registration, thread, or hash-map order. Each path claims the
/// first `source(path, n)` (for `n = 0, 1, 2, …`) not already claimed by an
/// earlier path. Repeating the same path is not a collision: it collapses to
/// a single entry with a single key. Production always passes
/// [`schedule_key_attempt`]; `source` is a seam so tests can force a
/// collision without brute-forcing a real FNV-1a one.
///
/// This is the schedule-namespace sibling of
/// [`super::armor::allocate_armor_tag_keys`], which resolves collisions by
/// appending a stable `_<n>` suffix. That approach cannot be reused here:
/// entity tag names are effectively unbounded, but a scoreboard objective is
/// capped at 16 chars and `__ss_` (5) + 8 hex + `_t` (2) already spends 15 of
/// them, leaving no room for a suffix. Schedules therefore disambiguate by
/// deterministically salted rehash, which keeps the key width fixed at 8.
pub(crate) fn allocate_schedule_objective_keys<'a, I>(
    paths: I,
    source: ScheduleKeySource,
) -> ExportResult<BTreeMap<String, String>>
where
    I: IntoIterator<Item = &'a str>,
{
    let unique: BTreeSet<&str> = paths.into_iter().collect();
    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    let mut allocated: BTreeMap<String, String> = BTreeMap::new();

    for path in unique {
        let mut chosen: Option<String> = None;
        let mut first_owner: Option<(&str, String)> = None;

        for attempt in 0..SCHEDULE_KEY_PROBE_LIMIT {
            let key = source(path, attempt);
            ensure_objective_names_fit(path, &key)?;
            match claimed.get(&key) {
                Some(owner) => {
                    if first_owner.is_none() {
                        first_owner = Some((owner, key));
                    }
                }
                None => {
                    chosen = Some(key);
                    break;
                }
            }
        }

        let key = match chosen {
            Some(key) => key,
            None => {
                let (owner, key) = first_owner.expect("probe limit is non-zero");
                let (obj_t, obj_p) = schedule_objective_names(&key);
                return Err(schedule_export_error(format!(
                    "schedule path `{path}` collides with schedule path `{owner}` on generated \
                     key `{key}` (objectives `{obj_t}` / `{obj_p}`); {SCHEDULE_KEY_PROBE_LIMIT} \
                     deterministic rehash attempts all collided, so the two schedules cannot be \
                     given independent scoreboard state — rename one of the schedule functions"
                )));
            }
        };

        claimed.insert(key.clone(), path);
        allocated.insert(path.to_string(), key);
    }

    Ok(allocated)
}

/// Lower every registered [`crate::function::ScheduleDescriptor`] into records
/// and load/tick tag entries, using the production key source.
pub(crate) fn emit_schedule_records(
    namespace: &str,
    schedules: &[&crate::function::ScheduleDescriptor],
    records: &mut Vec<ComponentRecord>,
    tag_map: &mut BTreeMap<String, Vec<String>>,
) -> ExportResult<()> {
    emit_schedule_records_with(namespace, schedules, records, tag_map, schedule_key_attempt)
}

/// Lower every registered schedule descriptor with an injectable key source.
///
/// Keys are allocated up front, in sorted path order, for the whole schedule
/// set; records are then emitted in the caller's original (inventory)
/// iteration order so record ordering is unaffected by the allocation pass.
/// If allocation fails, this returns `Err` *before* pushing any record, so no
/// partial schedule output can reach the caller.
pub(crate) fn emit_schedule_records_with(
    namespace: &str,
    schedules: &[&crate::function::ScheduleDescriptor],
    records: &mut Vec<ComponentRecord>,
    tag_map: &mut BTreeMap<String, Vec<String>>,
    source: ScheduleKeySource,
) -> ExportResult<()> {
    if schedules.is_empty() {
        return Ok(());
    }

    let keys = allocate_schedule_objective_keys(schedules.iter().map(|desc| desc.path), source)?;

    let mut init_cmds: Vec<String> = Vec::new();
    let mut tick_cmds: Vec<String> = Vec::new();

    for desc in schedules {
        let hash = keys
            .get(desc.path)
            .expect("every schedule path receives an allocated key");
        let (obj_t, obj_p) = schedule_objective_names(hash);

        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: desc.path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: (desc.make)().join("\n"),
        });

        let mut start_cmds = vec![format!(
            "scoreboard players set @s {obj_t} {}",
            desc.total_ticks
        )];
        if desc.every > 1 {
            start_cmds.push(format!("scoreboard players set @s {obj_p} 1"));
        }
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: format!("{}_start", desc.path),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: start_cmds.join("\n"),
        });
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: format!("{}_stop", desc.path),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: format!("scoreboard players set @s {obj_t} 0"),
        });

        init_cmds.push(format!("scoreboard objectives add {obj_t} dummy"));
        if desc.every > 1 {
            init_cmds.push(format!("scoreboard objectives add {obj_p} dummy"));
        }

        tick_cmds.extend(schedule_tick_commands(namespace, desc, &obj_t, &obj_p));
    }

    let init_path = "__sand_sched_init";
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: init_path.to_string(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: init_cmds.join("\n"),
    });
    tag_map
        .entry("minecraft:load".to_string())
        .or_default()
        .push(format!("{namespace}:{init_path}"));

    let tick_path = "__sand_sched_tick";
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: tick_path.to_string(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: tick_cmds.join("\n"),
    });
    tag_map
        .entry("minecraft:tick".to_string())
        .or_default()
        .push(format!("{namespace}:{tick_path}"));

    Ok(())
}

/// Lower scheduler maintenance through a per-player execution context.
///
/// Schedule counters belong to the player that called the generated `_start`
/// function. Keeping every generated mutation on `@s` under `execute as`
/// makes that ownership explicit and prevents future source-bearing
/// scoreboard operations from accidentally receiving a multi-holder selector.
pub(crate) fn schedule_tick_commands(
    namespace: &str,
    desc: &crate::function::ScheduleDescriptor,
    obj_t: &str,
    obj_p: &str,
) -> Vec<String> {
    let active = format!("{obj_t}=1..");
    if desc.every <= 1 {
        vec![
            format!(
                "execute as @a[scores={{{active}}}] at @s run function {namespace}:{}",
                desc.path
            ),
            format!(
                "execute as @a[scores={{{active}}}] run scoreboard players remove @s {obj_t} 1"
            ),
        ]
    } else {
        let fire = format!("{obj_t}=1..,{obj_p}=..0");
        vec![
            format!(
                "execute as @a[scores={{{active}}}] run scoreboard players remove @s {obj_p} 1"
            ),
            format!(
                "execute as @a[scores={{{fire}}}] at @s run function {namespace}:{}",
                desc.path
            ),
            format!(
                "execute as @a[scores={{{fire}}}] run scoreboard players set @s {obj_p} {}",
                desc.every
            ),
            format!(
                "execute as @a[scores={{{active}}}] run scoreboard players remove @s {obj_t} 1"
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Key source that maps *every* path to the same base key on attempt 0,
    /// then to distinct salted keys — the controlled equivalent of a real
    /// FNV-1a collision between two paths.
    fn colliding_source(path: &str, attempt: u32) -> String {
        if attempt == 0 {
            "deadbeef".to_string()
        } else {
            fnv1a_hex(&format!("{path}#{attempt}"))
        }
    }

    /// Key source that never yields anything but one key, so every probe
    /// collides and allocation must fail.
    fn always_colliding_source(_path: &str, _attempt: u32) -> String {
        "deadbeef".to_string()
    }

    /// Key source whose keys are too long for a valid objective name.
    fn oversized_source(_path: &str, _attempt: u32) -> String {
        "0123456789abcdef".to_string()
    }

    fn objectives(keys: &BTreeMap<String, String>) -> Vec<String> {
        keys.values()
            .flat_map(|key| {
                let (t, p) = schedule_objective_names(key);
                [t, p]
            })
            .collect()
    }

    #[test]
    fn non_colliding_paths_keep_the_plain_path_hash() {
        let keys = allocate_schedule_objective_keys(
            ["every_tick_schedule", "interval_schedule"],
            schedule_key_attempt,
        )
        .unwrap();
        assert_eq!(
            keys["every_tick_schedule"],
            schedule_key("every_tick_schedule")
        );
        assert_eq!(keys["interval_schedule"], schedule_key("interval_schedule"));
        // Golden values also asserted by tests/schedule_multiplayer_safety.rs.
        assert_eq!(keys["every_tick_schedule"], "8863de6c");
        assert_eq!(keys["interval_schedule"], "5607096c");
    }

    #[test]
    fn colliding_paths_get_distinct_keys() {
        let keys = allocate_schedule_objective_keys(["alpha", "beta"], colliding_source).unwrap();
        assert_eq!(keys.len(), 2);
        assert_ne!(keys["alpha"], keys["beta"]);
        // Sorted order: "alpha" claims the contested base key first.
        assert_eq!(keys["alpha"], "deadbeef");
        assert_ne!(keys["beta"], "deadbeef");
    }

    #[test]
    fn allocation_is_independent_of_input_order() {
        let forward =
            allocate_schedule_objective_keys(["alpha", "beta"], colliding_source).unwrap();
        let reversed =
            allocate_schedule_objective_keys(["beta", "alpha"], colliding_source).unwrap();
        assert_eq!(forward, reversed);

        let forward_real =
            allocate_schedule_objective_keys(["alpha", "beta", "gamma"], schedule_key_attempt)
                .unwrap();
        let reversed_real =
            allocate_schedule_objective_keys(["gamma", "beta", "alpha"], schedule_key_attempt)
                .unwrap();
        assert_eq!(forward_real, reversed_real);
    }

    #[test]
    fn allocation_is_deterministic_across_repeats() {
        let expected =
            allocate_schedule_objective_keys(["a", "b", "c", "d"], colliding_source).unwrap();
        for _ in 0..256 {
            let again =
                allocate_schedule_objective_keys(["d", "c", "b", "a"], colliding_source).unwrap();
            assert_eq!(again, expected);
        }
    }

    #[test]
    fn duplicate_paths_collapse_without_collision_handling() {
        let keys =
            allocate_schedule_objective_keys(["dup", "dup", "dup"], schedule_key_attempt).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys["dup"], schedule_key("dup"));
    }

    #[test]
    fn generated_objectives_stay_within_the_length_limit() {
        let plain = allocate_schedule_objective_keys(
            ["alpha", "beta", "a_very_long_schedule_path"],
            schedule_key_attempt,
        )
        .unwrap();
        let collided =
            allocate_schedule_objective_keys(["alpha", "beta"], colliding_source).unwrap();
        for name in objectives(&plain).into_iter().chain(objectives(&collided)) {
            assert!(
                name.len() <= OBJECTIVE_NAME_LIMIT,
                "objective `{name}` exceeds the {OBJECTIVE_NAME_LIMIT}-char limit"
            );
        }
    }

    #[test]
    fn probe_exhaustion_reports_both_paths_key_and_objectives() {
        let err = allocate_schedule_objective_keys(["alpha", "beta"], always_colliding_source)
            .expect_err("exhausted probes must fail export");
        let message = err.to_string();
        assert!(message.contains("alpha"), "{message}");
        assert!(message.contains("beta"), "{message}");
        assert!(message.contains("deadbeef"), "{message}");
        assert!(message.contains("__ss_deadbeef_t"), "{message}");
        assert!(message.contains("__ss_deadbeef_p"), "{message}");
        assert!(message.contains("schedule"), "{message}");
    }

    #[test]
    fn oversized_keys_are_rejected() {
        let err = allocate_schedule_objective_keys(["alpha"], oversized_source)
            .expect_err("oversized objective names must fail export");
        assert!(err.to_string().contains("16-character"), "{err}");
    }

    fn body() -> Vec<String> {
        vec!["say hi".to_string()]
    }

    #[test]
    fn failed_allocation_emits_no_records() {
        let a = crate::function::ScheduleDescriptor {
            path: "alpha",
            total_ticks: 20,
            every: 1,
            make: body,
        };
        let b = crate::function::ScheduleDescriptor {
            path: "beta",
            total_ticks: 20,
            every: 1,
            make: body,
        };
        let mut records: Vec<ComponentRecord> = Vec::new();
        let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

        let err = emit_schedule_records_with(
            "pack",
            &[&a, &b],
            &mut records,
            &mut tag_map,
            always_colliding_source,
        )
        .expect_err("failed allocation must abort the schedule phase");

        assert!(records.is_empty(), "no records may be emitted on failure");
        assert!(
            tag_map.is_empty(),
            "no tag entries may be emitted on failure"
        );
        assert!(err.to_string().contains("__ss_deadbeef_t"), "{err}");
    }

    #[test]
    fn colliding_emission_uses_distinct_objectives() {
        let a = crate::function::ScheduleDescriptor {
            path: "alpha",
            total_ticks: 20,
            every: 1,
            make: body,
        };
        let b = crate::function::ScheduleDescriptor {
            path: "beta",
            total_ticks: 20,
            every: 1,
            make: body,
        };
        let mut records: Vec<ComponentRecord> = Vec::new();
        let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

        emit_schedule_records_with(
            "pack",
            &[&a, &b],
            &mut records,
            &mut tag_map,
            colliding_source,
        )
        .expect("collision must be resolved, not fatal");

        let init = records
            .iter()
            .find(|record| record.path == "__sand_sched_init")
            .expect("init function emitted");
        let objectives: Vec<&str> = init
            .content
            .lines()
            .map(|line| line.trim_start_matches("scoreboard objectives add "))
            .map(|rest| rest.trim_end_matches(" dummy"))
            .collect();
        assert_eq!(objectives.len(), 2);
        assert_ne!(objectives[0], objectives[1]);
        for name in objectives {
            assert!(name.len() <= OBJECTIVE_NAME_LIMIT);
        }

        // Record ordering follows the caller's (inventory) order, not sorted
        // key-allocation order.
        assert_eq!(records[0].path, "alpha");
        assert_eq!(records[3].path, "beta");
    }
}
