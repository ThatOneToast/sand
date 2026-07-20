//! Schedule lowering phase of the export pipeline.
//!
//! Owns the stable objective-name hashing for `#[schedule]` descriptors and
//! the per-player tick maintenance commands for the generated
//! `__sand_sched_tick` function.

/// Compute a stable 8-hex-char key for a schedule path (FNV-1a 32-bit).
/// Keeps scoreboard objective names within Minecraft's 16-char limit:
/// `__ss_` (5) + 8 hex + `_t`/`_p` (2) = 15 chars.
pub(crate) fn schedule_key(path: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in path.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
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
