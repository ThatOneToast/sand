use serde::Serialize;

// ── Unified traits ────────────────────────────────────────────────────────────
// Re-export the canonical definitions from sand-components so the entire
// workspace shares ONE set of traits.  All builders in sand-components already
// implement these; McFunction (below) does too via crate::resource_location
// which now resolves to sand_components::ResourceLocation.

pub use sand_components::component::{ComponentContent, DatapackComponent, IntoDatapack};

// ── sand-core-specific types ──────────────────────────────────────────────────

/// A serializable record of a datapack component for output during the build
/// process.  Consumed by `sand-build` / the generated `sand_export` binary.
#[derive(Serialize)]
pub struct ComponentRecord {
    /// The namespace (e.g. `"my_pack"`).
    pub namespace: String,
    /// The component type directory (e.g. `"function"`, `"advancement"`).
    pub dir: String,
    /// The resource location path (e.g. `"my_tick"`, `"utils/helper"`).
    pub path: String,
    /// The file extension without the dot (e.g. `"mcfunction"`, `"json"`).
    pub ext: String,
    /// The serialized content of the component.
    pub content: String,
}

/// Collect all inventory-registered components and return them as a JSON string
/// for consumption by `sand build`.  Called by the generated `sand_export` binary.
///
/// Iterates through all registered:
/// - `FunctionDescriptor`s — `.mcfunction` files
/// - `ComponentFactory`s — component JSON files
/// - `FunctionTagDescriptor`s — function tag JSON files
/// - `ArmorEventDescriptor`s — armor event handlers
/// - `EventDescriptor`s — advancement-backed events
///
/// Returns a JSON string containing an array of [`ComponentRecord`] objects.
pub fn export_components_json(namespace: &str) -> String {
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
            content: commands.join("\n"),
        });
    }

    // ── Dynamic anonymous functions (run_fn! blocks) ──────────────────────────
    for (path, commands) in crate::drain_dyn_fns() {
        records.push(ComponentRecord {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path,
            ext: "mcfunction".to_string(),
            content: commands.join("\n"),
        });
    }

    // ── ComponentFactories ────────────────────────────────────────────────────
    for factory in inventory::iter::<ComponentFactory>() {
        let comp = (factory.make)();
        let rl = comp.resource_location();
        let content = match comp.content() {
            ComponentContent::Json(v) => serde_json::to_string_pretty(&v).unwrap(),
            ComponentContent::Text(t) => t,
        };
        records.push(ComponentRecord {
            namespace: rl.namespace().to_string(),
            dir: comp.component_dir().to_string(),
            path: rl.path().to_string(),
            ext: comp.file_extension().to_string(),
            content,
        });
    }

    // ── FunctionTagDescriptors ────────────────────────────────────────────────
    for desc in inventory::iter::<FunctionTagDescriptor>() {
        let fn_ref = format!("{}:{}", namespace, desc.function_path);
        tag_map
            .entry(desc.tag.to_string())
            .or_default()
            .push(fn_ref);
    }

    // ── EventDescriptors + ArmorEventDescriptors ─────────────────────────────
    use crate::function::EventDispatch;

    // Categorise events by dispatch type so we can batch-generate aggregators.
    let mut join_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut death_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut respawn_tick_events: Vec<&EventDescriptor> = Vec::new();
    // (descriptor, condition_string)
    let mut tick_poll_events: Vec<(&EventDescriptor, String)> = Vec::new();
    // Shared armor watch map — populated by both EventDescriptor ArmorEquip/
    // ArmorUnequip dispatch and the legacy ArmorEventDescriptor entries.
    // (slot, item_id, custom_data_snbt, Vec<(is_equip, path)>)
    let mut armor_watch_map: BTreeMap<
        String,
        (
            ArmorSlot,
            Option<&'static str>,
            Option<&'static str>,
            Vec<(bool, &'static str)>, // (is_equip, path)
        ),
    > = BTreeMap::new();

    for desc in inventory::iter::<EventDescriptor>() {
        // Always emit the handler function body first.
        let commands = (desc.make)();

        match &desc.dispatch {
            // ── Advancement-backed ────────────────────────────────────────────
            EventDispatch::Advancement {
                make_trigger,
                revoke,
            } => {
                let advancement_id = desc
                    .id_override
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}:{}", namespace, desc.path));

                let mut body = commands;
                if *revoke {
                    body.insert(0, format!("advancement revoke @s only {}", advancement_id));
                }
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content: body.join("\n"),
                });

                let trigger = make_trigger();
                let fn_ref = format!("{}:{}", namespace, desc.path);
                let advancement = sand_components::Advancement::new(
                    advancement_id
                        .parse()
                        .expect("invalid advancement ID in #[event]"),
                )
                .criterion("event", sand_components::Criterion::new(trigger))
                .rewards(sand_components::AdvancementRewards::new().function(fn_ref));

                let content = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
                let rl = advancement.resource_location();
                records.push(ComponentRecord {
                    namespace: rl.namespace().to_string(),
                    dir: advancement.component_dir().to_string(),
                    path: rl.path().to_string(),
                    ext: advancement.file_extension().to_string(),
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
                    content: commands.join("\n"),
                });
                tick_poll_events.push((desc, make_condition()));
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
                    // Advancement-backed custom event.
                    let advancement_id = desc
                        .id_override
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("{}:{}", namespace, desc.path));

                    let mut body = commands;
                    if (revoke)() {
                        body.insert(0, format!("advancement revoke @s only {}", advancement_id));
                    }
                    records.push(ComponentRecord {
                        namespace: namespace.to_string(),
                        dir: "function".to_string(),
                        path: desc.path.to_string(),
                        ext: "mcfunction".to_string(),
                        content: body.join("\n"),
                    });

                    let fn_ref = format!("{}:{}", namespace, desc.path);
                    let advancement = sand_components::Advancement::new(
                        advancement_id
                            .parse()
                            .expect("invalid advancement ID in custom #[event]"),
                    )
                    .criterion("event", sand_components::Criterion::new(trigger))
                    .rewards(sand_components::AdvancementRewards::new().function(fn_ref));

                    let content = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
                    let rl = advancement.resource_location();
                    records.push(ComponentRecord {
                        namespace: rl.namespace().to_string(),
                        dir: advancement.component_dir().to_string(),
                        path: rl.path().to_string(),
                        ext: advancement.file_extension().to_string(),
                        content,
                    });
                } else if let Some(condition) = make_condition() {
                    // Tick-poll custom event — same as TickPoll.
                    records.push(ComponentRecord {
                        namespace: namespace.to_string(),
                        dir: "function".to_string(),
                        path: desc.path.to_string(),
                        ext: "mcfunction".to_string(),
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
            content: respawn_cmds.join("\n"),
        });
        tag_map
            .entry("minecraft:tick".to_string())
            .or_default()
            .push(format!("{namespace}:{respawn_path}"));
    }

    // ── TickPoll aggregation ──────────────────────────────────────────────────
    if !tick_poll_events.is_empty() {
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
                content: start_cmds.join("\n"),
            });
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: format!("{}_stop", desc.path),
                ext: "mcfunction".to_string(),
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
                content: ts_cmds.join("\n"),
            });
            tag_map
                .entry("minecraft:load".to_string())
                .or_default()
                .push(format!("{namespace}:{ts_path}"));
        }
    }

    // ── Finalize tag_map → records ────────────────────────────────────────────
    for (tag_rl, values) in tag_map {
        let (tag_ns, tag_path) = match tag_rl.split_once(':') {
            Some((ns, path)) => (ns.to_string(), path.to_string()),
            None => (namespace.to_string(), tag_rl.clone()),
        };
        let json = serde_json::json!({ "values": values });
        records.push(ComponentRecord {
            namespace: tag_ns,
            dir: "tags/function".to_string(),
            path: tag_path,
            ext: "json".to_string(),
            content: serde_json::to_string_pretty(&json).unwrap(),
        });
    }

    serde_json::to_string_pretty(&records).unwrap()
}

// ── Private helpers ───────────────────────────────────────────────────────────

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
