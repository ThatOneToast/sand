#![allow(clippy::result_large_err)]

use serde::Serialize;

// ── Unified traits ────────────────────────────────────────────────────────────
// Re-export the canonical definitions from sand-components so the entire
// workspace shares ONE set of traits.  All builders in sand-components already
// implement these; McFunction (below) does too via crate::resource_location
// which now resolves to sand_components::ResourceLocation.

pub use sand_components::component::{ComponentContent, DatapackComponent, IntoDatapack};
pub use sand_components::error::SandError as ComponentExportError;
pub use sand_version::{ComponentFeature, VersionCaps};

// ── sand-core-specific types ──────────────────────────────────────────────────

/// A serializable record of a datapack component for output during the build
/// process.  Consumed by `sand-build` / the generated `sand_export` binary.
#[derive(Serialize, Debug)]
pub struct ComponentRecord {
    /// The namespace (e.g. `"my_pack"`).
    pub namespace: String,
    /// The component type directory (e.g. `"function"`, `"advancement"`).
    pub dir: String,
    /// The resource location path (e.g. `"my_tick"`, `"utils/helper"`).
    pub path: String,
    /// The file extension without the dot (e.g. `"mcfunction"`, `"json"`).
    pub ext: String,
    /// `"text"` writes `content` directly; `"copy"` copies the source path in `content`.
    pub content_type: String,
    /// The serialized content of the component.
    pub content: String,
}

/// Error returned by [`try_export_components`] when a registered component fails
/// validation or serialization.
pub type ExportResult<T> = std::result::Result<T, ComponentExportError>;

/// Version-aware export context — carries the resolved capability set and the
/// requested version string for diagnostics.
struct ExportCtx<'a> {
    caps: &'a VersionCaps,
    requested_version: &'a str,
    is_fallback: bool,
}

/// Convert a single [`DatapackComponent`] into a [`ComponentRecord`],
/// validating exactly once before any content is accepted, and checking
/// version-gated features against the export context.
fn component_to_record(
    comp: &dyn DatapackComponent,
    ctx: Option<&ExportCtx>,
) -> ExportResult<ComponentRecord> {
    let rl = comp.resource_location().clone();
    let kind = comp.component_dir().to_string();

    // Version-gate check: reject if a required feature is not supported.
    if let Some(ctx) = ctx {
        for feature in comp.required_features() {
            if !ctx.caps.supports(*feature) {
                return Err(sand_components::error::version_gating_error(
                    &rl.to_string(),
                    &kind,
                    *feature,
                    ctx.requested_version,
                    ctx.is_fallback,
                ));
            }
        }
    }

    let (content_type, content) = if let Some(path) = comp.copy_source_path() {
        comp.validate().map_err(|e| enrich_error(e, &rl, &kind))?;
        ("copy", path.to_string())
    } else {
        let content = comp
            .try_content()
            .map_err(|e| enrich_error(e, &rl, &kind))?;
        match content {
            ComponentContent::Json(v) => {
                let text = serde_json::to_string_pretty(&v).map_err(|serde_err| {
                    sand_components::error::SandError::ComponentValidation {
                        location: rl.clone(),
                        kind: kind.clone(),
                        field: "<serialization>".to_string(),
                        message: serde_err.to_string(),
                    }
                })?;
                ("text", text)
            }
            ComponentContent::Text(t) => ("text", t),
        }
    };

    Ok(ComponentRecord {
        namespace: rl.namespace().to_string(),
        dir: kind,
        path: rl.path().to_string(),
        ext: comp.file_extension().to_string(),
        content_type: content_type.to_string(),
        content,
    })
}

fn enrich_error(
    e: sand_components::error::SandError,
    rl: &crate::resource_location::ResourceLocation,
    kind: &str,
) -> ComponentExportError {
    match e {
        sand_components::error::SandError::Serialization(serde_err) => {
            sand_components::error::SandError::ComponentValidation {
                location: rl.clone(),
                kind: kind.to_string(),
                field: "<serialization>".to_string(),
                message: serde_err.to_string(),
            }
        }
        other => other,
    }
}

/// Collect all inventory-registered components into records, routing every
/// `ComponentFactory` through the fallible [`component_to_record`] helper
/// which validates all components (JSON, text, and copy-backed) before
/// accepting their content.
///
/// This is the **unprofiled** compatibility path — no version-gating is
/// performed. Use [`try_export_components_for_version`] when the target
/// `VersionProfile` is known so that version-gated components are rejected
/// before any pack output is written.
pub fn try_export_components(namespace: &str) -> ExportResult<Vec<ComponentRecord>> {
    try_export_components_impl(namespace, None)
}

/// Version-aware fallible export: collect all inventory-registered components,
/// rejecting components and advancement-backed events that require features not
/// available in the target [`VersionCaps`].
///
/// The `requested_version` string and `is_fallback` flag are used for
/// diagnostics only. Invalid components and unsupported version-gated
/// components are rejected **before** any pack output is written.
pub fn try_export_components_for_version(
    namespace: &str,
    caps: &VersionCaps,
    requested_version: &str,
    is_fallback: bool,
) -> ExportResult<Vec<ComponentRecord>> {
    let ctx = ExportCtx {
        caps,
        requested_version,
        is_fallback,
    };
    try_export_components_impl(namespace, Some(&ctx))
}

fn try_export_components_impl(
    namespace: &str,
    ctx: Option<&ExportCtx>,
) -> ExportResult<Vec<ComponentRecord>> {
    use crate::function::{
        ArmorEventDescriptor, ArmorEventKind, ArmorSlot, ComponentFactory, EventDescriptor,
        FunctionDescriptor, FunctionTagDescriptor,
    };
    use crate::inventory;
    use std::collections::BTreeMap;

    let mut records: Vec<ComponentRecord> = Vec::new();
    let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // ── FunctionDescriptors ───────────────────────────────────────────────────
    for desc in inventory::iter::<FunctionDescriptor>() {
        let commands = (desc.make)();
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: desc.path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: commands.join("\n"),
        });
    }

    // ── ComponentFactories (fallible boundary) ────────────────────────────────
    for factory in inventory::iter::<ComponentFactory>() {
        let comp = (factory.make)();
        let record = component_to_record(comp.as_ref(), ctx)?;
        records.push(record);
    }

    // ── EventDescriptors + ArmorEventDescriptors ─────────────────────────────
    use crate::function::EventDispatch;

    // Categorise events by dispatch type so we can batch-generate aggregators.
    let mut join_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut death_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut respawn_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut xp_level_up_events: Vec<&EventDescriptor> = Vec::new();
    // (descriptor, condition_string)
    let mut tick_poll_events: Vec<(&EventDescriptor, String)> = Vec::new();
    // Shared armor watch map — populated by both EventDescriptor ArmorEquip/
    // ArmorUnequip dispatch and the legacy ArmorEventDescriptor entries.
    // (slot, item_id, custom_data_snbt, Vec<(is_equip, path)>)
    type ArmorWatchEntry = (
        ArmorSlot,
        Option<&'static str>,
        Option<&'static str>,
        Vec<(bool, &'static str)>,
    );
    let mut armor_watch_map: BTreeMap<String, ArmorWatchEntry> = BTreeMap::new();

    for desc in inventory::iter::<EventDescriptor>() {
        // Always emit the handler function body first.
        let commands = (desc.make)();

        match &desc.dispatch {
            // ── Advancement-backed ────────────────────────────────────────────
            //
            // Generates three resources per event:
            //   function/<path>/body    — user-authored handler commands (pure)
            //   function/<path>         — entry: revoke → guard → call body
            //   advancement/<path>      — fires the trigger; reward calls entry
            //
            // Separation rationale:
            //   • body is testable independently (pure user commands, no plumbing)
            //   • entry's revoke always runs before guard, so AfterFire events
            //     re-arm even when the guard rejects execution on a given tick
            //   • Any-condition guards correctly expand into multiple guard lines
            EventDispatch::Advancement {
                make_trigger,
                revoke,
                guard,
            } => {
                let advancement_id = desc
                    .id_override
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}:{}", namespace, desc.path));

                let body_path = format!("{}/body", desc.path);
                let body_fn_ref = format!("{namespace}:{body_path}");

                // ── Body function: pure user commands ─────────────────────────
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: body_path,
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });

                // ── Entry function: revoke → guard(s) → call body ─────────────
                let mut entry: Vec<String> = Vec::new();
                if revoke() {
                    entry.push(format!("advancement revoke @s only {advancement_id}"));
                }
                if let Some(make_guard) = guard
                    && let Some(cond) = make_guard()
                {
                    // execute_commands(true, "return 0") correctly expands
                    // Any/OR guards into multiple guard lines and All/AND into
                    // one compound line — no more "unless if" syntax errors.
                    entry.extend(cond.execute_commands(true, "return 0"));
                }
                entry.push(format!("function {body_fn_ref}"));

                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: entry.join("\n"),
                });

                // ── Advancement: trigger fires entry ──────────────────────────
                let trigger = make_trigger();
                check_event_trigger(&trigger, &advancement_id, desc.path, ctx)?;
                let advancement = sand_components::Advancement::new(
                    advancement_id
                        .parse()
                        .expect("invalid advancement ID in #[event]"),
                )
                .criterion("event", sand_components::Criterion::new(trigger))
                .rewards(
                    sand_components::AdvancementRewards::new()
                        .function(format!("{namespace}:{}", desc.path)),
                );

                let content = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
                let rl = advancement.resource_location();
                records.push(ComponentRecord {
                    namespace: rl.namespace().to_string(),
                    dir: advancement.component_dir().to_string(),
                    path: rl.path().to_string(),
                    ext: advancement.file_extension().to_string(),
                    content_type: "text".to_string(),
                    content,
                });
            }

            // ── JoinTick ─────────────────────────────────────────────────────
            EventDispatch::JoinTick => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                join_tick_events.push(desc);
            }

            // ── DeathTick ────────────────────────────────────────────────────
            EventDispatch::DeathTick => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                death_tick_events.push(desc);
            }

            // ── RespawnTick ──────────────────────────────────────────────────
            EventDispatch::RespawnTick => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                respawn_tick_events.push(desc);
            }

            // ── TickPoll ─────────────────────────────────────────────────────
            EventDispatch::TickPoll { make_condition } => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                tick_poll_events.push((desc, make_condition()));
            }

            // ── XpLevelUp ────────────────────────────────────────────────────
            EventDispatch::XpLevelUp => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                xp_level_up_events.push(desc);
            }

            // ── ArmorEquip ───────────────────────────────────────────────────
            EventDispatch::ArmorEquip {
                slot,
                item_id,
                custom_data_snbt,
            } => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                let key = armor_watch_key(*slot, *item_id, *custom_data_snbt);
                let entry = armor_watch_map.entry(key).or_insert((
                    *slot,
                    *item_id,
                    *custom_data_snbt,
                    Vec::new(),
                ));
                entry.3.push((true, desc.path));
            }

            // ── ArmorUnequip ─────────────────────────────────────────────────
            EventDispatch::ArmorUnequip {
                slot,
                item_id,
                custom_data_snbt,
            } => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                let key = armor_watch_key(*slot, *item_id, *custom_data_snbt);
                let entry = armor_watch_map.entry(key).or_insert((
                    *slot,
                    *item_id,
                    *custom_data_snbt,
                    Vec::new(),
                ));
                entry.3.push((false, desc.path));
            }

            // ── Custom SandEvent ─────────────────────────────────────────────
            EventDispatch::Custom {
                make_trigger,
                make_condition,
                revoke,
            } => {
                if let Some(trigger) = make_trigger() {
                    // Advancement-backed custom (SandEvent) event.
                    // Same entry/body split as the typed Advancement arm.
                    let advancement_id = desc
                        .id_override
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("{}:{}", namespace, desc.path));
                    check_event_trigger(&trigger, &advancement_id, desc.path, ctx)?;

                    let body_path = format!("{}/body", desc.path);
                    let body_fn_ref = format!("{namespace}:{body_path}");

                    records.push(ComponentRecord {
                        namespace: namespace.to_string(),
                        dir: "function".to_string(),
                        path: body_path,
                        ext: "mcfunction".to_string(),
                        content_type: "text".to_string(),
                        content: commands.join("\n"),
                    });

                    let mut entry: Vec<String> = Vec::new();
                    if (revoke)() {
                        entry.push(format!("advancement revoke @s only {advancement_id}"));
                    }
                    entry.push(format!("function {body_fn_ref}"));
                    records.push(ComponentRecord {
                        namespace: namespace.to_string(),
                        dir: "function".to_string(),
                        path: desc.path.to_string(),
                        ext: "mcfunction".to_string(),
                        content_type: "text".to_string(),
                        content: entry.join("\n"),
                    });

                    let advancement = sand_components::Advancement::new(
                        advancement_id
                            .parse()
                            .expect("invalid advancement ID in custom #[event]"),
                    )
                    .criterion("event", sand_components::Criterion::new(trigger))
                    .rewards(
                        sand_components::AdvancementRewards::new()
                            .function(format!("{namespace}:{}", desc.path)),
                    );

                    let content = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
                    let rl = advancement.resource_location();
                    records.push(ComponentRecord {
                        namespace: rl.namespace().to_string(),
                        dir: advancement.component_dir().to_string(),
                        path: rl.path().to_string(),
                        ext: advancement.file_extension().to_string(),
                        content_type: "text".to_string(),
                        content,
                    });
                } else if let Some(condition) = make_condition() {
                    // Tick-poll custom event — same as TickPoll.
                    records.push(ComponentRecord {
                        namespace: namespace.to_string(),
                        dir: "function".to_string(),
                        path: desc.path.to_string(),
                        ext: "mcfunction".to_string(),
                        content_type: "text".to_string(),
                        content: commands.join("\n"),
                    });
                    tick_poll_events.push((desc, condition));
                } else {
                    panic!(
                        "Custom SandEvent for handler `{}` returned None from both \
                         make_trigger() and make_condition() — implement exactly one",
                        desc.path
                    );
                }
            }
        }
    }

    // ── ArmorEventDescriptors (legacy #[armor_event]) ─────────────────────────
    for desc in inventory::iter::<ArmorEventDescriptor>() {
        let commands = (desc.make)();
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: desc.path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: commands.join("\n"),
        });
        let key = armor_watch_key(desc.slot, desc.item_id, desc.custom_data_snbt);
        let is_equip = matches!(desc.kind, ArmorEventKind::Equip);
        let entry = armor_watch_map.entry(key).or_insert((
            desc.slot,
            desc.item_id,
            desc.custom_data_snbt,
            Vec::new(),
        ));
        entry.3.push((is_equip, desc.path));
    }

    // ── JoinTick aggregation ──────────────────────────────────────────────────
    // Detection: players whose `__sand_join` scoreboard score is unset (never
    // seen) or was cleared by the load-time reset have (re)joined since the
    // last server start/reload.
    //
    // WHY scoreboard instead of entity tag:
    //   Entity tags are saved to each player's playerdata/<uuid>.dat, so they
    //   persist across disconnects. A player who logs out still has the tag
    //   when they log back in, so tag-based detection fires only once ever.
    //   Scoreboard scores live in the world's scoreboard.dat.
    //   `scoreboard players reset * __sand_join` (run on minecraft:load)
    //   clears ALL entries — including offline players — so the next time
    //   any player joins after a server start or /reload, they fire OnJoin.
    //
    // KNOWN LIMITATION:
    //   Mid-session disconnect → reconnect WITHOUT a /reload does NOT re-fire
    //   OnJoin. The score for the player is still 1 in scoreboard.dat.
    //   True per-login detection for mid-session reconnects requires a mod or
    //   plugin; it is not achievable in vanilla datapacks.
    if !join_tick_events.is_empty() {
        // ── Init (minecraft:load) ──────────────────────────────────────────
        let join_init_path = "__sand_join_init";
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: join_init_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            // `reset *` clears ALL tracked entries, including offline players.
            content: "scoreboard objectives add __sand_join dummy\nscoreboard players reset * __sand_join".to_string(),
        });
        tag_map
            .entry("minecraft:load".to_string())
            .or_default()
            .push(format!("{namespace}:{join_init_path}"));

        // ── Tick check ────────────────────────────────────────────────────
        let join_path = "__sand_join_check";
        let mut join_cmds: Vec<String> = Vec::new();
        for desc in &join_tick_events {
            join_cmds.push(format!(
                "execute as @a unless score @s __sand_join matches 1 at @s run function {namespace}:{}",
                desc.path
            ));
        }
        // Set score for all online players AFTER all handlers have run, so
        // every handler fires for a newly-joined player on the same tick.
        join_cmds.push("scoreboard players set @a __sand_join 1".to_string());

        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: join_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: join_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{join_path}"));
    }

    // ── DeathTick + RespawnTick aggregation ───────────────────────────────────
    let needs_death_check = !death_tick_events.is_empty() || !respawn_tick_events.is_empty();
    if needs_death_check {
        let init_path = "__sand_death_init";
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: init_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: "scoreboard objectives add __sand_dc deathCount".to_string(),
        });
        tag_map
            .entry("minecraft:load".to_string())
            .or_default()
            .push(format!("{namespace}:{init_path}"));

        let check_path = "__sand_death_check";
        let mut check_cmds: Vec<String> = Vec::new();
        check_cmds.push(
            "execute as @a[scores={__sand_dc=1..}] run tag @s add __sand_just_died".to_string(),
        );
        check_cmds.push("scoreboard players set @a __sand_dc 0".to_string());
        // Tag dying players so the respawn check can detect them later.
        if !respawn_tick_events.is_empty() {
            check_cmds.push("tag @a[tag=__sand_just_died] add __sand_was_dead".to_string());
        }
        for desc in &death_tick_events {
            check_cmds.push(format!(
                "execute as @a[tag=__sand_just_died] run function {namespace}:{}",
                desc.path
            ));
        }
        check_cmds.push("tag @a remove __sand_just_died".to_string());

        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: check_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: check_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{check_path}"));
    }

    // ── RespawnTick check ─────────────────────────────────────────────────────
    if !respawn_tick_events.is_empty() {
        let respawn_path = "__sand_respawn_check";
        let mut respawn_cmds: Vec<String> = Vec::new();
        for desc in &respawn_tick_events {
            respawn_cmds.push(format!(
                "execute as @a[tag=__sand_was_dead,gamemode=!spectator] \
                 run function {namespace}:{}",
                desc.path
            ));
        }
        // Remove the tag once the player has respawned (i.e. exited spectator).
        respawn_cmds.push("tag @a[gamemode=!spectator] remove __sand_was_dead".to_string());

        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: respawn_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: respawn_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{respawn_path}"));
    }

    // ── XpLevelUp aggregation ─────────────────────────────────────────────────
    // Objectives (all ≤16 chars):
    //   __sand_xp_lvl   — current XP level this tick
    //   __sand_xp_prev  — XP level last tick
    //   __sand_xp_delta — current − previous
    //   __sand_xp_seen  — 0 until first tick (prevents spurious fire on join)
    if !xp_level_up_events.is_empty() {
        let xp_init_path = "__sand_xp_init";
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: xp_init_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: [
                "scoreboard objectives add __sand_xp_lvl dummy",
                "scoreboard objectives add __sand_xp_prev dummy",
                "scoreboard objectives add __sand_xp_delta dummy",
                "scoreboard objectives add __sand_xp_seen dummy",
            ]
            .join("\n"),
        });
        tag_map
            .entry("minecraft:load".to_string())
            .or_default()
            .push(format!("{namespace}:{xp_init_path}"));

        let xp_check_path = "__sand_xp_check";
        // Tick flow:
        // 1. Snapshot current XP level for all online players.
        // 2. On first tick (seen=0), copy current to prev as a baseline;
        //    then mark seen=1. Do NOT fire handlers yet.
        // 3. Compute delta = current − prev.
        // 4. Fire each handler for players whose delta ≥ 1 (level increased).
        // 5. Update prev to current (covers both increases and decreases).
        let handler_cmds: Vec<String> = xp_level_up_events
            .iter()
            .map(|desc| {
                format!(
                    "execute as @a[scores={{__sand_xp_delta=1..}}] at @s run function {namespace}:{}",
                    desc.path
                )
            })
            .collect();
        let mut xp_cmds = vec![
            // Step 1 — snapshot
            "execute as @a store result score @s __sand_xp_lvl run experience query @s levels"
                .to_string(),
            // Step 2 — first-tick baseline
            "execute as @a unless score @s __sand_xp_seen matches 1 \
             run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
                .to_string(),
            "scoreboard players set @a __sand_xp_seen 1".to_string(),
            // Step 3 — delta = current − prev
            "scoreboard players operation @a __sand_xp_delta = @a __sand_xp_lvl".to_string(),
            "scoreboard players operation @a __sand_xp_delta -= @a __sand_xp_prev".to_string(),
        ];
        // Step 4 — fire handlers
        xp_cmds.extend(handler_cmds);
        // Step 5 — advance prev
        xp_cmds
            .push("scoreboard players operation @a __sand_xp_prev = @a __sand_xp_lvl".to_string());

        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: xp_check_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: xp_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{xp_check_path}"));
    }

    // ── TickPoll aggregation ──────────────────────────────────────────────────
    if !tick_poll_events.is_empty() {
        // Built-in player-state events use generated entity predicates rather
        // than selector NBT. Emit only the predicates actually referenced by
        // this pack so a pack with no state events gets no internal files.
        let mut state_predicates = BTreeMap::new();
        for (_, condition) in &tick_poll_events {
            if let Some((path, flag)) = sand_player_state_predicate(condition) {
                state_predicates.insert(path, flag);
            }
        }
        for (path, flag) in state_predicates {
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "predicate".to_string(),
                path: path.to_string(),
                ext: "json".to_string(),
                content_type: "text".to_string(),
                content: serde_json::to_string_pretty(&player_state_predicate_json(flag)).unwrap(),
            });
        }
        let tick_path = "__sand_tick_check";
        let tick_cmds: Vec<String> = tick_poll_events
            .iter()
            .map(|(desc, condition)| {
                format!(
                    "execute as @a if {condition} at @s run function {namespace}:{}",
                    desc.path
                )
            })
            .collect();
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
    }

    // ── Armor check aggregation ───────────────────────────────────────────────
    if !armor_watch_map.is_empty() {
        let armor_path = "__sand_armor_check";
        let mut armor_cmds: Vec<String> = Vec::new();

        for (key, (slot, item_id, custom_data_snbt, handlers)) in &armor_watch_map {
            let tag_now = format!("__armor_{key}_now");
            let tag_had = format!("__armor_{key}_had");
            let cond = build_item_cond(*slot, *item_id, *custom_data_snbt);

            // Tag players currently wearing/holding the item.
            armor_cmds.push(format!("execute as @a if {cond} run tag @s add {tag_now}"));

            // Fire equip handlers (now present, wasn't before).
            for (is_equip, path) in handlers {
                if *is_equip {
                    armor_cmds.push(format!(
                        "execute as @a[tag={tag_now},tag=!{tag_had}] at @s run function {namespace}:{path}"
                    ));
                }
            }

            // Fire unequip handlers (was present, now gone).
            for (is_equip, path) in handlers {
                if !is_equip {
                    armor_cmds.push(format!(
                        "execute as @a[tag=!{tag_now},tag={tag_had}] at @s run function {namespace}:{path}"
                    ));
                }
            }

            // Advance state: sync `_had` to match current `_now`.
            armor_cmds.push(format!("tag @a[tag={tag_now}] add {tag_had}"));
            armor_cmds.push(format!("tag @a[tag=!{tag_now}] remove {tag_had}"));
            armor_cmds.push(format!("tag @a remove {tag_now}"));
        }

        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: armor_path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: armor_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{armor_path}"));
    }

    // ── ScheduleDescriptors ───────────────────────────────────────────────────
    use crate::function::ScheduleDescriptor;
    let schedules: Vec<&ScheduleDescriptor> = inventory::iter::<ScheduleDescriptor>().collect();
    if !schedules.is_empty() {
        let mut init_cmds: Vec<String> = Vec::new();
        let mut tick_cmds: Vec<String> = Vec::new();

        for desc in &schedules {
            let hash = schedule_key(desc.path);
            let obj_t = format!("__ss_{hash}_t");
            let obj_p = format!("__ss_{hash}_p");

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

            let active = format!("{obj_t}=1..");
            if desc.every <= 1 {
                tick_cmds.push(format!(
                    "execute as @a[scores={{{active}}}] at @s run function {namespace}:{}",
                    desc.path
                ));
                tick_cmds.push(format!(
                    "scoreboard players remove @a[scores={{{active}}}] {obj_t} 1"
                ));
            } else {
                tick_cmds.push(format!(
                    "scoreboard players remove @a[scores={{{active}}}] {obj_p} 1"
                ));
                let fire = format!("{obj_t}=1..,{obj_p}=..0");
                tick_cmds.push(format!(
                    "execute as @a[scores={{{fire}}}] at @s run function {namespace}:{}",
                    desc.path
                ));
                tick_cmds.push(format!(
                    "execute as @a[scores={{{fire}}}] run scoreboard players set @s {obj_p} {}",
                    desc.every
                ));
                tick_cmds.push(format!(
                    "scoreboard players remove @a[scores={{{active}}}] {obj_t} 1"
                ));
            }
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
    }

    // ── TempScoreboard → __sand_temp_scores (minecraft:load) ─────────────────
    {
        use std::collections::BTreeSet;
        let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
        let mut ts_cmds: Vec<String> = Vec::new();
        for ts in inventory::iter::<crate::TempScoreboard>() {
            if seen.insert((ts.name, ts.criteria)) {
                match ts.display_name {
                    Some(dn) => ts_cmds.push(format!(
                        "scoreboard objectives add {} {} {}",
                        ts.name, ts.criteria, dn
                    )),
                    None => ts_cmds.push(format!(
                        "scoreboard objectives add {} {}",
                        ts.name, ts.criteria
                    )),
                }
            }
        }
        if !ts_cmds.is_empty() {
            let ts_path = "__sand_temp_scores";
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: ts_path.to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: ts_cmds.join("\n"),
            });
            tag_map
                .entry("minecraft:load".to_string())
                .or_default()
                .push(format!("{namespace}:{ts_path}"));
        }
    }

    // ── Dynamic anonymous functions (branches from all make() calls above) ───
    // ── Compiler-managed score constants / expression temporaries ───────────
    // Score operands are registered while user factories execute, so this must
    // run after all factory/event processing and before tags are finalized.
    let score_setup = crate::state::score::drain_internal_score_setup();
    if !score_setup.is_empty() {
        let path = "__sand_score_init";
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: path.to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: score_setup.join("\n"),
        });
        tag_map
            .entry("minecraft:load".to_string())
            .or_default()
            .push(format!("{namespace}:{path}"));
    }

    // ── Lifecycle registry (register_load_objective / register_tick_handler) ──
    // Must run after all factories so all explicit registrations are captured.
    {
        let load_cmds = crate::state::drain_load_commands();
        if !load_cmds.is_empty() {
            let path = "__sand_lifecycle_load";
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: path.to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: load_cmds.join("\n"),
            });
            tag_map
                .entry("minecraft:load".to_string())
                .or_default()
                .push(format!("{namespace}:{path}"));
        }

        let tick_cmds = crate::state::drain_tick_commands();
        if !tick_cmds.is_empty() {
            let path = "__sand_lifecycle_tick";
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: path.to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: tick_cmds.join("\n"),
            });
            tag_map
                .entry("minecraft:tick".to_string())
                .or_default()
                .push(format!("{namespace}:{path}"));
        }
    }

    // ── Dynamic anonymous functions (branches from all make() calls above) ───
    // Must run AFTER every desc.make() call so branches registered by event
    // bodies, schedule bodies, armor handlers, etc. are all captured.
    // The loop handles chains: draining can trigger further registrations.
    drain_dynamic_functions_into(&mut records, namespace);

    // ── Dialog callback dispatcher ────────────────────────────────────────────
    // Must run after ComponentFactory and other make() calls so callbacks
    // registered while constructing dialog components are not missed.
    let _dialog_callback_lock = dialog_callback_export_lock();
    drain_dialog_callbacks_into(&mut records, &mut tag_map, namespace);

    // ── FunctionTagDescriptors ────────────────────────────────────────────────
    // Append user-declared function tag entries after Sand-owned setup/dispatcher
    // entries so load/tick infrastructure exists before user functions run.
    for desc in inventory::iter::<FunctionTagDescriptor>() {
        let fn_ref = format!("{}:{}", namespace, desc.function_path);
        tag_map
            .entry(desc.tag.to_string())
            .or_default()
            .push(fn_ref);
    }

    // ── Finalize tag_map → records ────────────────────────────────────────────
    for (tag_rl, values) in tag_map {
        let (tag_ns, tag_path) = match tag_rl.split_once(':') {
            Some((ns, path)) => (ns.to_string(), path.to_string()),
            None => (namespace.to_string(), tag_rl.clone()),
        };
        // Registration can reach the same lifecycle tag through multiple
        // framework paths. Preserve first-seen execution order while emitting
        // each function reference only once.
        let values = dedupe_preserve_order(values);
        let json = serde_json::json!({ "values": values });
        records.push(ComponentRecord {
            namespace: tag_ns,
            dir: "tags/function".to_string(),
            path: tag_path,
            ext: "json".to_string(),
            content_type: "text".to_string(),
            content: serde_json::to_string_pretty(&json).unwrap(),
        });
    }

    // ── Resolve local sentinels → real namespace ──────────────────────────────
    // Sentinel patterns written by Sand-generated code:
    //   `function __sand_local:<path>`         — from cmd::call(fn_ptr) for bare functions
    //   `... only __sand_local:<path>`         — from EventHandle::revoke/grant
    //   `__sand_local:<path>` in JSON          — from local component refs
    // They are resolved to the pack namespace here, after all records are collected.
    for rec in &mut records {
        if rec.namespace == crate::function::SAND_LOCAL_NS {
            rec.namespace = namespace.to_string();
        }
        rec.content = resolve_local_refs(&rec.content, namespace);
    }

    // Sanity: no sentinel should survive into the final output.
    debug_assert!(
        !records
            .iter()
            .any(|r| r.content.contains(crate::function::SAND_LOCAL_NS)),
        "BUG: unresolved __sand_local sentinel found in exported records"
    );

    Ok(records)
}

/// Fallibly collect all inventory-registered components and return them as a
/// JSON string for consumption by `sand build`.
///
/// This is the function the generated `sand_export` binary should call. On
/// success it returns the JSON array of [`ComponentRecord`] objects as a
/// `String`. On failure it returns a [`ComponentExportError`] carrying the
/// resource location, component kind, and validation field — **no panic, no
/// backtrace**. The caller is responsible for printing a diagnostic to stderr
/// and exiting non-zero.
///
/// See [`try_export_components`] for the record-level fallible API.
pub fn try_export_components_json(namespace: &str) -> ExportResult<String> {
    let records = try_export_components(namespace)?;
    serde_json::to_string_pretty(&records).map_err(sand_components::error::SandError::Serialization)
}

/// Version-aware fallible JSON export: collect all components and advancement-backed
/// events, rejecting any that require features not available in the target version.
///
/// This is the function the generated `sand_export` binary should call when a
/// target version is known. On failure it returns a [`ComponentExportError`]
/// carrying the resource location, component kind, and version-gating context
/// — no panic, no backtrace.
pub fn try_export_components_json_for_version(
    namespace: &str,
    caps: &VersionCaps,
    requested_version: &str,
    is_fallback: bool,
) -> ExportResult<String> {
    let records =
        try_export_components_for_version(namespace, caps, requested_version, is_fallback)?;
    serde_json::to_string_pretty(&records).map_err(sand_components::error::SandError::Serialization)
}

/// Collect all inventory-registered components and return them as a JSON string
/// for consumption by `sand build`.
///
/// **Compatibility wrapper** — this function **panics** on component validation
/// or serialization failure. It is retained for backward compatibility with
/// direct callers that expect an infallible `String` return. The generated
/// `sand_export` binary and all scaffold templates use
/// [`try_export_components_json`] or [`try_export_components_json_for_version`]
/// instead.
pub fn export_components_json(namespace: &str) -> String {
    match try_export_components_json(namespace) {
        Ok(s) => s,
        Err(e) => panic!("sand component export failed: {e}"),
    }
}

fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut deduped = Vec::with_capacity(values.len());

    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }

    deduped
}

/// Returns the output path and entity-predicate flag for a Sand-owned player
/// state predicate. Custom `TickCondition`s are deliberately left untouched.
fn sand_player_state_predicate(condition: &str) -> Option<(&'static str, &'static str)> {
    match condition {
        "predicate __sand_local:__sand/player_sneaking" => {
            Some(("__sand/player_sneaking", "is_sneaking"))
        }
        "predicate __sand_local:__sand/player_sprinting" => {
            Some(("__sand/player_sprinting", "is_sprinting"))
        }
        "predicate __sand_local:__sand/player_swimming" => {
            Some(("__sand/player_swimming", "is_swimming"))
        }
        "predicate __sand_local:__sand/player_on_fire" => {
            Some(("__sand/player_on_fire", "is_on_fire"))
        }
        _ => None,
    }
}

fn player_state_predicate_json(flag: &str) -> serde_json::Value {
    serde_json::json!({
        "condition": "minecraft:entity_properties",
        "entity": "this",
        "predicate": { "flags": { flag: true } },
    })
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Drain all dynamically-registered branch/anonymous functions into `records`.
///
/// Loops until the registry is empty so that branches registered *by* other
/// branches (nested mcfunction! blocks) are also captured.
fn drain_dynamic_functions_into(records: &mut Vec<ComponentRecord>, namespace: &str) {
    loop {
        let drained = crate::drain_dyn_fns();
        if drained.is_empty() {
            break;
        }
        for (path, commands) in drained {
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path,
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: commands.join("\n"),
            });
        }
    }
}

/// Drain dialog callbacks into generated trigger/load/tick infrastructure.
fn drain_dialog_callbacks_into(
    records: &mut Vec<ComponentRecord>,
    tag_map: &mut std::collections::BTreeMap<String, Vec<String>>,
    namespace: &str,
) {
    let callbacks = sand_components::dialog::drain_dialog_callbacks();
    if callbacks.is_empty() {
        return;
    }

    let trigger = sand_components::dialog::SAND_DIALOG_TRIGGER;

    let init_cmds = [
        format!("scoreboard objectives add {trigger} trigger"),
        format!("scoreboard players enable @a {trigger}"),
    ];
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: "__sand_dialog_init".to_string(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: init_cmds.join("\n"),
    });
    tag_map
        .entry("minecraft:load".to_string())
        .or_default()
        .push(format!("{namespace}:__sand_dialog_init"));

    let mut tick_cmds: Vec<String> = Vec::new();
    tick_cmds.push(format!("scoreboard players enable @a {trigger}"));
    for (id, path) in &callbacks {
        tick_cmds.push(format!(
            "execute as @a[scores={{{trigger}={id}}}] at @s run function {path}"
        ));
        tick_cmds.push(format!(
            "scoreboard players set @a[scores={{{trigger}={id}}}] {trigger} 0"
        ));
        tick_cmds.push(format!("scoreboard players enable @a {trigger}"));
    }
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: "__sand_dialog_tick".to_string(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: tick_cmds.join("\n"),
    });
    tag_map
        .entry("minecraft:tick".to_string())
        .or_default()
        .push(format!("{namespace}:__sand_dialog_tick"));
}

fn dialog_callback_export_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap()
}

/// Replace every `__sand_local:<path>` sentinel in an mcfunction content string
/// with `<namespace>:<path>`.
///
/// Handles both patterns:
/// - `function __sand_local:path` — bare function pointer calls
/// - `... only __sand_local:path` — advancement revoke/grant from EventHandle
fn resolve_local_refs(content: &str, namespace: &str) -> String {
    let sentinel = crate::function::SAND_LOCAL_NS;
    content.replace(&format!("{sentinel}:"), &format!("{namespace}:"))
}

/// Compute a stable 8-hex-char key for a schedule path (FNV-1a 32-bit).
/// Keeps scoreboard objective names within Minecraft's 16-char limit:
/// `__ss_` (5) + 8 hex + `_t`/`_p` (2) = 15 chars.
fn schedule_key(path: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in path.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

/// Build the unique key used to group armor watch entries by slot + filters.
fn armor_watch_key(
    slot: crate::function::ArmorSlot,
    item_id: Option<&str>,
    custom_data: Option<&str>,
) -> String {
    let mut parts = vec![slot.tag_name_segment().to_string()];
    if let Some(id) = item_id {
        parts.push(sanitize_armor_tag(id));
    }
    if let Some(cd) = custom_data {
        parts.push(sanitize_armor_tag(cd));
    }
    parts.join("_")
}

fn sanitize_armor_tag(s: &str) -> String {
    let raw: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    raw.trim_matches('_').to_string()
}

/// Validate an advancement trigger for the target version, returning a
/// fallible error instead of panicking.
///
/// Checks both the existing `validate_for_target` (trigger ID availability)
/// and the trigger coverage table's `since`/`removed_in` fields against the
/// target version when a version context is provided.
fn check_event_trigger(
    trigger: &crate::AdvancementTrigger,
    advancement_id: &str,
    handler_path: &str,
    ctx: Option<&ExportCtx>,
) -> ExportResult<()> {
    // Existing trigger-level validation (ID availability).
    trigger.validate_for_target().map_err(|diagnostic| {
        sand_components::error::SandError::ComponentValidation {
            location: advancement_id.parse().unwrap_or_else(|_| {
                sand_components::ResourceLocation::new("sand", "error").unwrap()
            }),
            kind: "advancement_event".to_string(),
            field: "trigger".to_string(),
            message: format!("cannot export advancement event `{handler_path}`: {diagnostic}"),
        }
    })?;

    // Version-aware trigger availability check using TRIGGER_COVERAGE.
    if let Some(ctx) = ctx {
        let trigger_id = trigger.trigger_id();
        if let Some(coverage) =
            sand_components::advancement::trigger_coverage::find_coverage(trigger_id)
        {
            if !coverage.since.is_empty() {
                let req = parse_version_components(coverage.since);
                let target = parse_version_components(ctx.requested_version);
                if let (Some(req), Some(target)) = (req, target)
                    && !is_version_gte(&target, &req)
                {
                    let fallback_note = if ctx.is_fallback {
                        " (fallback profile: all features disabled; use an exact known \
                         version or `mc_version = \"latest\"` to enable version-gated \
                         features)"
                    } else {
                        ""
                    };
                    return Err(sand_components::error::SandError::VersionGating {
                        location: advancement_id.to_string(),
                        kind: format!(
                            "trigger `{trigger_id}` (available since {since})",
                            trigger_id = trigger_id,
                            since = coverage.since
                        ),
                        requested_version: ctx.requested_version.to_string(),
                        is_fallback: ctx.is_fallback,
                        feature_name: format!("trigger since {since}", since = coverage.since),
                        fallback_note: fallback_note.to_string(),
                    });
                }
            }
            if let Some(removed_in) = coverage.removed_in {
                let removed = parse_version_components(removed_in);
                let target = parse_version_components(ctx.requested_version);
                if let (Some(removed), Some(target)) = (removed, target)
                    && is_version_gte(&target, &removed)
                {
                    let fallback_note = if ctx.is_fallback {
                        " (fallback profile)"
                    } else {
                        ""
                    };
                    return Err(sand_components::error::SandError::VersionGating {
                        location: advancement_id.to_string(),
                        kind: format!("trigger `{trigger_id}` (removed in {removed_in})"),
                        requested_version: ctx.requested_version.to_string(),
                        is_fallback: ctx.is_fallback,
                        feature_name: format!(
                            "trigger before {removed_in}",
                            removed_in = removed_in
                        ),
                        fallback_note: fallback_note.to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

fn parse_version_components(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split('.').collect();
    let major = parts.first()?.parse::<u32>().ok()?;
    let minor = parts
        .get(1)
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(0);
    Some((major, minor, patch))
}

fn is_version_gte(target: &(u32, u32, u32), required: &(u32, u32, u32)) -> bool {
    if target.0 != required.0 {
        return target.0 > required.0;
    }
    if target.1 != required.1 {
        return target.1 > required.1;
    }
    target.2 >= required.2
}

fn build_item_cond(
    slot: crate::function::ArmorSlot,
    item_id: Option<&str>,
    custom_data: Option<&str>,
) -> String {
    let predicate = match (item_id, custom_data) {
        (None, _) => "*".to_string(),
        (Some(id), None) => id.to_string(),
        (Some(id), Some(cd)) => format!("{}[minecraft:custom_data~{}]", id, cd),
    };
    format!("items entity @s {} {}", slot.slot_name(), predicate)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{player_state_predicate_json, sand_player_state_predicate};
    use crate::AdvancementTrigger;
    use crate::events::{PlayerSwimmingEvent, SandEvent};

    inventory::submit! {
        crate::function::FunctionTagDescriptor {
            tag: "minecraft:load",
            function_path: "__test_user_load_after_setup",
        }
    }

    #[test]
    fn player_state_events_use_predicate_flags() {
        let condition = match PlayerSwimmingEvent::dispatch() {
            crate::events::SandEventDispatch::TickCondition(condition) => condition,
            _ => panic!("player swimming must be tick-polled"),
        };
        assert_eq!(
            sand_player_state_predicate(&condition),
            Some(("__sand/player_swimming", "is_swimming"))
        );
        assert!(!condition.contains("nbt={"));
        assert_eq!(
            player_state_predicate_json("is_swimming"),
            json!({
                "condition": "minecraft:entity_properties",
                "entity": "this",
                "predicate": { "flags": { "is_swimming": true } },
            })
        );
        assert_eq!(
            format!(
                "execute as @a if {} at @s run function audit:while_swimming",
                condition.replace("__sand_local:", "audit:")
            ),
            "execute as @a if predicate audit:__sand/player_swimming at @s run function audit:while_swimming"
        );
    }

    #[test]
    fn all_owned_state_predicates_have_expected_flags() {
        assert_eq!(
            sand_player_state_predicate("predicate __sand_local:__sand/player_sneaking"),
            Some(("__sand/player_sneaking", "is_sneaking"))
        );
        assert_eq!(
            sand_player_state_predicate("predicate __sand_local:__sand/player_sprinting"),
            Some(("__sand/player_sprinting", "is_sprinting"))
        );
        assert_eq!(
            sand_player_state_predicate("predicate __sand_local:__sand/player_on_fire"),
            Some(("__sand/player_on_fire", "is_on_fire"))
        );
    }

    #[test]
    fn custom_sand_event_advancement_trigger_validation_accepts_supported_trigger() {
        let result = super::check_event_trigger(
            &AdvancementTrigger::Tick,
            "test:legacy_valid_event",
            "legacy_valid_event",
            None,
        );
        assert!(
            result.is_ok(),
            "supported trigger should pass: {:?}",
            result.err()
        );
    }

    #[test]
    fn custom_sand_event_advancement_trigger_validation_rejects_invalid_trigger() {
        let result = super::check_event_trigger(
            &AdvancementTrigger::LeveledUp { level: None },
            "test:legacy_level_up",
            "legacy_level_up",
            None,
        );
        let err = result.expect_err("invalid trigger should be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("cannot export advancement event `legacy_level_up`"),
            "error should name the handler path, got: {msg}"
        );
        assert!(
            msg.contains("minecraft:leveled_up"),
            "error should include the invalid trigger ID, got: {msg}"
        );
        assert!(
            msg.contains("experience query"),
            "error should include the migration diagnostic, got: {msg}"
        );
    }

    #[test]
    fn function_tag_values_dedupe_without_sorting() {
        let values = vec![
            "pack:z".to_string(),
            "pack:a".to_string(),
            "pack:z".to_string(),
            "pack:m".to_string(),
        ];

        assert_eq!(
            super::dedupe_preserve_order(values),
            vec![
                "pack:z".to_string(),
                "pack:a".to_string(),
                "pack:m".to_string()
            ]
        );
    }

    #[test]
    fn exported_load_tag_preserves_generated_insertion_order() {
        let _lock = crate::state::registry::registry_test_lock();
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();
        let _ = crate::state::score::drain_internal_score_setup();

        let _ = crate::state::ScoreConst::<i32>::new("tag order setup", 7).ref_();
        crate::state::register_load_objective("tag_order_lifecycle", "dummy");

        let json_str = super::export_components_json("orderpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(
            tag_values(&records, "minecraft:load"),
            vec![
                "orderpack:__sand_score_init".to_string(),
                "orderpack:__sand_lifecycle_load".to_string(),
                "orderpack:__test_user_load_after_setup".to_string(),
            ]
        );
    }

    // ── Lifecycle registry wiring ─────────────────────────────────────────────

    /// Parse the JSON output of `export_components_json` and return the subset of
    /// records matching the given `path`.
    fn records_with_path<'a>(
        records: &'a [serde_json::Value],
        path: &str,
    ) -> Vec<&'a serde_json::Value> {
        records
            .iter()
            .filter(|r| r["path"].as_str() == Some(path))
            .collect()
    }

    /// Return the "values" array from the tag record for `tag_rl` (e.g.
    /// `"minecraft:load"`), or an empty vec if no such record exists.
    fn tag_values(records: &[serde_json::Value], tag_rl: &str) -> Vec<String> {
        // Tag records: dir = "tags/function", path = everything after the colon.
        let tag_path = tag_rl.split_once(':').map(|(_, p)| p).unwrap_or(tag_rl);
        for r in records {
            if r["dir"].as_str() == Some("tags/function")
                && r["path"].as_str() == Some(tag_path)
                && let Some(arr) = r["content"]
                    .as_str()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .and_then(|v| v["values"].as_array().cloned())
            {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect();
            }
        }
        Vec::new()
    }

    #[test]
    fn late_dialog_callback_drain_emits_dispatcher_after_component_construction() {
        let _lock = super::dialog_callback_export_lock();
        let _ = sand_components::dialog::drain_dialog_callbacks();

        assert!(
            sand_components::dialog::drain_dialog_callbacks().is_empty(),
            "test starts from the old early-drain state"
        );

        let dialog = sand_components::dialog::Dialog::multi_action_local("welcome").button(
            sand_components::dialog::DialogButton::new("Grant").action(
                sand_components::dialog::DialogAction::callback("__sand_local:grant_reward"),
            ),
        );
        let dialog_json = dialog.to_json();
        let command = dialog_json["actions"][0]["action"]["command"]
            .as_str()
            .expect("dialog callback button should emit a command action");
        let callback_id = command
            .strip_prefix("/trigger sand.dialog set ")
            .expect("callback action should use the Sand dialog trigger");

        let mut records = Vec::new();
        let mut tag_map = std::collections::BTreeMap::new();
        super::drain_dialog_callbacks_into(&mut records, &mut tag_map, "dialogpack");

        let init_recs: Vec<_> = records
            .iter()
            .filter(|r| r.path == "__sand_dialog_init")
            .collect();
        assert_eq!(init_recs.len(), 1, "dialog init function should be emitted");
        assert!(
            init_recs[0]
                .content
                .contains("scoreboard objectives add sand.dialog trigger"),
            "dialog init function must create the trigger objective"
        );

        let tick_recs: Vec<_> = records
            .iter()
            .filter(|r| r.path == "__sand_dialog_tick")
            .collect();
        assert_eq!(tick_recs.len(), 1, "dialog tick function should be emitted");
        let tick_content = &tick_recs[0].content;
        assert!(
            tick_content.contains(&format!(
                "execute as @a[scores={{sand.dialog={callback_id}}}] at @s run function __sand_local:grant_reward"
            )),
            "dialog tick function must dispatch the registered callback, got: {tick_content}"
        );
        assert!(
            tick_content.contains(&format!(
                "scoreboard players set @a[scores={{sand.dialog={callback_id}}}] sand.dialog 0"
            )),
            "dialog tick function must reset the callback score, got: {tick_content}"
        );

        assert_eq!(
            tag_map.get("minecraft:load").cloned().unwrap_or_default(),
            vec!["dialogpack:__sand_dialog_init".to_string()]
        );
        assert_eq!(
            tag_map.get("minecraft:tick").cloned().unwrap_or_default(),
            vec!["dialogpack:__sand_dialog_tick".to_string()]
        );

        let _ = sand_components::dialog::drain_dialog_callbacks();
    }

    #[test]
    fn empty_dialog_callback_registry_emits_no_dispatcher() {
        let _lock = super::dialog_callback_export_lock();
        let _ = sand_components::dialog::drain_dialog_callbacks();

        let mut records = Vec::new();
        let mut tag_map = std::collections::BTreeMap::new();
        super::drain_dialog_callbacks_into(&mut records, &mut tag_map, "dialogpack");

        assert!(
            records.iter().all(|r| r.path != "__sand_dialog_init"),
            "no dialog init function should appear with no callbacks"
        );
        assert!(
            !tag_map.contains_key("minecraft:load"),
            "no load tag entry should appear with no callbacks"
        );
        assert!(
            !tag_map.contains_key("minecraft:tick"),
            "no tick tag entry should appear with no callbacks"
        );
    }

    #[test]
    fn lifecycle_load_objective_appears_in_export() {
        let _lock = crate::state::registry::registry_test_lock();
        // Drain any residual state from prior tests.
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        crate::state::register_load_objective("lc_test_mana", "dummy");

        let json_str = super::export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        // A __sand_lifecycle_load function must exist.
        let load_recs = records_with_path(&records, "__sand_lifecycle_load");
        assert_eq!(
            load_recs.len(),
            1,
            "__sand_lifecycle_load record must appear exactly once"
        );
        assert!(
            load_recs[0]["content"]
                .as_str()
                .unwrap_or("")
                .contains("scoreboard objectives add lc_test_mana dummy"),
            "load function must contain the registered objective command"
        );

        // The minecraft:load tag must reference it.
        let load_tag = tag_values(&records, "minecraft:load");
        assert!(
            load_tag.contains(&"lcpack:__sand_lifecycle_load".to_string()),
            "minecraft:load tag must contain lcpack:__sand_lifecycle_load, got: {load_tag:?}"
        );
    }

    #[test]
    fn lifecycle_tick_handler_appears_in_export() {
        let _lock = crate::state::registry::registry_test_lock();
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        crate::state::register_tick_handler(
            "lc_test/my_handler",
            vec!["scoreboard players remove @a lc_test_cd 1".to_string()],
        );

        let json_str = super::export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        let tick_recs = records_with_path(&records, "__sand_lifecycle_tick");
        assert_eq!(
            tick_recs.len(),
            1,
            "__sand_lifecycle_tick record must appear exactly once"
        );
        assert!(
            tick_recs[0]["content"]
                .as_str()
                .unwrap_or("")
                .contains("scoreboard players remove @a lc_test_cd 1"),
            "tick function must contain the registered handler commands"
        );

        let tick_tag = tag_values(&records, "minecraft:tick");
        assert!(
            tick_tag.contains(&"lcpack:__sand_lifecycle_tick".to_string()),
            "minecraft:tick tag must contain lcpack:__sand_lifecycle_tick, got: {tick_tag:?}"
        );
    }

    #[test]
    fn empty_lifecycle_registry_produces_no_spurious_records() {
        let _lock = crate::state::registry::registry_test_lock();
        // Ensure both registries are empty before the export.
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        let json_str = super::export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert!(
            records_with_path(&records, "__sand_lifecycle_load").is_empty(),
            "no __sand_lifecycle_load record should appear with empty registry"
        );
        assert!(
            records_with_path(&records, "__sand_lifecycle_tick").is_empty(),
            "no __sand_lifecycle_tick record should appear with empty registry"
        );
    }

    #[test]
    fn lifecycle_load_ordering_is_deterministic() {
        let _lock = crate::state::registry::registry_test_lock();
        let _ = crate::state::drain_load_commands();
        let _ = crate::state::drain_tick_commands();

        // Register in reverse alphabetical order.
        crate::state::register_load_objective("lc_zeta", "dummy");
        crate::state::register_load_objective("lc_alpha", "dummy");
        crate::state::register_load_objective("lc_mana", "dummy");

        let json_str = super::export_components_json("lcpack");
        let records: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        let load_recs = records_with_path(&records, "__sand_lifecycle_load");
        assert_eq!(load_recs.len(), 1);
        let content = load_recs[0]["content"].as_str().unwrap_or("");
        let lines: Vec<&str> = content.lines().collect();

        // BTreeMap guarantees alphabetical order.
        assert!(
            lines[0].contains("lc_alpha"),
            "first command must be lc_alpha (alphabetical), got: {lines:?}"
        );
        assert!(
            lines[1].contains("lc_mana"),
            "second command must be lc_mana, got: {lines:?}"
        );
        assert!(
            lines[2].contains("lc_zeta"),
            "third command must be lc_zeta, got: {lines:?}"
        );
    }

    // ── Fallible component-to-record contract tests (#145) ─────────────────────
    //
    // All tests use `component_to_record` with local component values — no
    // global `inventory::submit!` and no process-global atomic toggles that
    // could affect other export tests running concurrently.

    use super::component_to_record;
    use sand_components::component::ComponentContent;
    use sand_components::error::SandError;

    fn test_rl(ns: &str, path: &str) -> crate::resource_location::ResourceLocation {
        crate::resource_location::ResourceLocation::new(ns, path).unwrap()
    }

    // ── Validation call counter ─────────────────────────────────────────────────

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A component that counts how many times `validate()` is called.
    /// Shared counter lets tests assert exactly-once validation.
    struct CountingJsonComponent {
        loc: crate::resource_location::ResourceLocation,
        counter: &'static AtomicUsize,
    }
    impl super::DatapackComponent for CountingJsonComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({"ok": true})
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn component_dir(&self) -> &'static str {
            "test_count_json"
        }
    }

    struct CountingCopyComponent {
        loc: crate::resource_location::ResourceLocation,
        source_path: String,
        counter: &'static AtomicUsize,
    }
    impl super::DatapackComponent for CountingCopyComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn copy_source_path(&self) -> Option<&str> {
            Some(&self.source_path)
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn component_dir(&self) -> &'static str {
            "test_count_copy"
        }
    }

    #[test]
    fn json_component_validates_exactly_once() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let comp = CountingJsonComponent {
            loc: test_rl("test", "count_json"),
            counter: &COUNT,
        };
        COUNT.store(0, Ordering::SeqCst);
        component_to_record(&comp, None).expect("valid JSON component should succeed");
        assert_eq!(
            COUNT.load(Ordering::SeqCst),
            1,
            "JSON/text components must validate exactly once \
             (try_content includes validation)"
        );
    }

    #[test]
    fn copy_component_validates_exactly_once() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let comp = CountingCopyComponent {
            loc: test_rl("test", "count_copy"),
            source_path: "structures/x.nbt".to_string(),
            counter: &COUNT,
        };
        COUNT.store(0, Ordering::SeqCst);
        component_to_record(&comp, None).expect("valid copy component should succeed");
        assert_eq!(
            COUNT.load(Ordering::SeqCst),
            1,
            "copy-backed components must validate exactly once"
        );
    }

    // ── Test fixture components ─────────────────────────────────────────────────

    struct ValidJsonComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for ValidJsonComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({"hello": "world"})
        }
        fn component_dir(&self) -> &'static str {
            "test_json"
        }
    }

    struct ValidTextComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for ValidTextComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn content(&self) -> ComponentContent {
            ComponentContent::Text("say hello from text component".to_string())
        }
        fn component_dir(&self) -> &'static str {
            "function"
        }
        fn file_extension(&self) -> &'static str {
            "mcfunction"
        }
    }

    struct ValidCopyComponent {
        loc: crate::resource_location::ResourceLocation,
        source_path: String,
    }
    impl super::DatapackComponent for ValidCopyComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn copy_source_path(&self) -> Option<&str> {
            Some(&self.source_path)
        }
        fn component_dir(&self) -> &'static str {
            "structure"
        }
        fn file_extension(&self) -> &'static str {
            "nbt"
        }
    }

    struct InvalidJsonComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for InvalidJsonComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            Err(SandError::ComponentValidation {
                location: self.loc.clone(),
                kind: "test_invalid_json".to_string(),
                field: "test_field".to_string(),
                message: "intentional JSON validation failure".to_string(),
            })
        }
        fn component_dir(&self) -> &'static str {
            "test_invalid_json"
        }
    }

    struct InvalidCopyComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for InvalidCopyComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn copy_source_path(&self) -> Option<&str> {
            Some("structures/should_not_be_accepted.nbt")
        }
        fn validate(&self) -> sand_components::error::Result<()> {
            Err(SandError::ComponentValidation {
                location: self.loc.clone(),
                kind: "test_invalid_copy".to_string(),
                field: "source_check".to_string(),
                message: "intentional copy-backed validation failure".to_string(),
            })
        }
        fn component_dir(&self) -> &'static str {
            "test_invalid_copy"
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────────────

    #[test]
    fn component_to_record_valid_json_preserves_output() {
        let comp = ValidJsonComponent {
            loc: test_rl("test", "valid_json"),
        };
        let record = component_to_record(&comp, None).expect("valid JSON component should succeed");
        assert_eq!(record.namespace, "test");
        assert_eq!(record.dir, "test_json");
        assert_eq!(record.path, "valid_json");
        assert_eq!(record.ext, "json");
        assert_eq!(record.content_type, "text");
        assert!(record.content.contains("hello"));
    }

    #[test]
    fn component_to_record_text_content_exports_correctly() {
        let comp = ValidTextComponent {
            loc: test_rl("test", "valid_text"),
        };
        let record = component_to_record(&comp, None).expect("valid text component should succeed");
        assert_eq!(record.content_type, "text");
        assert_eq!(record.content, "say hello from text component");
    }

    #[test]
    fn component_to_record_valid_copy_exports_correctly() {
        let comp = ValidCopyComponent {
            loc: test_rl("test", "valid_copy"),
            source_path: "structures/castle.nbt".to_string(),
        };
        let record = component_to_record(&comp, None).expect("valid copy component should succeed");
        assert_eq!(record.content_type, "copy");
        assert_eq!(record.content, "structures/castle.nbt");
    }

    #[test]
    fn component_to_record_invalid_json_returns_err_with_context() {
        let comp = InvalidJsonComponent {
            loc: test_rl("test", "invalid_json"),
        };
        let err = component_to_record(&comp, None).expect_err("invalid JSON component must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("test:invalid_json"),
            "must include location: {msg}"
        );
        assert!(
            msg.contains("test_invalid_json"),
            "must include kind: {msg}"
        );
        assert!(msg.contains("test_field"), "must include field: {msg}");
    }

    #[test]
    fn component_to_record_invalid_copy_returns_err_with_context() {
        let comp = InvalidCopyComponent {
            loc: test_rl("test", "invalid_copy"),
        };
        let err = component_to_record(&comp, None).expect_err("invalid copy component must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("test:invalid_copy"),
            "must include location: {msg}"
        );
        assert!(
            msg.contains("test_invalid_copy"),
            "must include kind: {msg}"
        );
        assert!(
            !msg.contains("should_not_be_accepted"),
            "source path must not be accepted when validation fails: {msg}"
        );
    }

    #[test]
    fn component_to_record_serialization_failure_never_becomes_null() {
        struct FailingSerializationComponent {
            loc: crate::resource_location::ResourceLocation,
        }
        impl super::DatapackComponent for FailingSerializationComponent {
            fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
                &self.loc
            }
            fn to_json(&self) -> serde_json::Value {
                serde_json::Value::Null
            }
            fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
                self.validate()?;
                Err(SandError::Serialization(
                    serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
                ))
            }
            fn component_dir(&self) -> &'static str {
                "test_ser_fail"
            }
        }

        let comp = FailingSerializationComponent {
            loc: test_rl("test", "ser_fail"),
        };
        let result = component_to_record(&comp, None);
        assert!(
            result.is_err(),
            "serialization failure must return Err, not Value::Null"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("test:ser_fail") || msg.contains("serialization"),
            "err: {msg}"
        );
    }

    // ── Version-aware gating tests (#147) ──────────────────────────────────────

    use super::ExportCtx;
    use sand_version::VersionCaps;

    /// A component that requires dialogs.
    struct DialogComponent {
        loc: crate::resource_location::ResourceLocation,
    }
    impl super::DatapackComponent for DialogComponent {
        fn resource_location(&self) -> &crate::resource_location::ResourceLocation {
            &self.loc
        }
        fn to_json(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        fn component_dir(&self) -> &'static str {
            "dialog"
        }
        fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
            &[sand_version::ComponentFeature::Dialogs]
        }
    }

    #[test]
    fn dialog_component_succeeds_when_dialogs_supported() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_ok"),
        };
        let caps = VersionCaps::all_enabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.21.6",
            is_fallback: false,
        };
        let record = component_to_record(&comp, Some(&ctx))
            .expect("dialog should succeed when dialogs feature is supported");
        assert_eq!(record.dir, "dialog");
    }

    #[test]
    fn dialog_component_fails_when_dialogs_not_supported() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_bad"),
        };
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.19.4",
            is_fallback: false,
        };
        let err = component_to_record(&comp, Some(&ctx))
            .expect_err("dialog should fail when dialogs feature is not supported");
        let msg = err.to_string();
        assert!(msg.contains("dialog"), "must include kind: {msg}");
        assert!(msg.contains("dialogs"), "must include feature name: {msg}");
        assert!(
            msg.contains("1.19.4"),
            "must include requested version: {msg}"
        );
    }

    #[test]
    fn version_gating_error_includes_fallback_note() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_fallback"),
        };
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        let err =
            component_to_record(&comp, Some(&ctx)).expect_err("should fail for fallback profile");
        let msg = err.to_string();
        assert!(msg.contains("fallback"), "must mention fallback: {msg}");
        assert!(
            msg.contains("999.0"),
            "must include requested version: {msg}"
        );
    }

    #[test]
    fn unprofiled_export_does_not_gate_components() {
        let comp = DialogComponent {
            loc: test_rl("test", "dialog_unprofiled"),
        };
        // No ctx → no version gating
        let record = component_to_record(&comp, None).expect("unprofiled export should not gate");
        assert_eq!(record.dir, "dialog");
    }

    #[test]
    fn resolve_export_caps_latest_enables_all_features() {
        let resolved = crate::version::resolve_export_caps("latest");
        assert!(!resolved.is_fallback, "latest should be a known profile");
        for feature in sand_version::ComponentFeature::ALL {
            assert!(
                resolved.caps.supports(*feature),
                "latest should support {:?}",
                feature
            );
        }
    }

    #[test]
    fn resolve_export_caps_unknown_version_disables_all() {
        let resolved = crate::version::resolve_export_caps("999.0");
        assert!(resolved.is_fallback, "unknown version should be fallback");
        for feature in sand_version::ComponentFeature::ALL {
            assert!(
                !resolved.caps.supports(*feature),
                "unknown version should not support {:?}",
                feature
            );
        }
    }

    #[test]
    fn resolve_export_caps_known_version_gates_correctly() {
        // 1.19.4 supports damage_types and trim_assets but not dialogs or jukebox_songs.
        let resolved = crate::version::resolve_export_caps("1.19.4");
        assert!(!resolved.is_fallback, "1.19.4 should be a known profile");
        assert!(
            resolved
                .caps
                .supports(sand_version::ComponentFeature::DamageTypes)
        );
        assert!(
            resolved
                .caps
                .supports(sand_version::ComponentFeature::TrimAssets)
        );
        assert!(
            !resolved
                .caps
                .supports(sand_version::ComponentFeature::Dialogs)
        );
        assert!(
            !resolved
                .caps
                .supports(sand_version::ComponentFeature::JukeboxSongs)
        );
    }
}
