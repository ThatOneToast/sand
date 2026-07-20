//! Pipeline driver: collection → aggregation → validation → assembly.
//!
//! [`try_export_components_impl`] walks every link-time inventory registry
//! (functions, components, events, schedules, temp scoreboards, function
//! tags), lowers each dispatch strategy through the phase modules in this
//! directory, resolves the `__sand_local` sentinel, and validates every
//! collected function before any record is accepted. Output ordering is
//! deterministic and must not change.
#![allow(clippy::result_large_err)]

use crate::events::graph::tick_event_resource_key;

use super::ExportCtx;
use super::armor::{
    ArmorWatchEntry, ArmorWatchKey, allocate_armor_tag_keys, armor_watch_key, build_item_cond,
};
use super::diagnostics::validate_function_records;
use super::dialogs::{
    DialogCallbackExportReset, dialog_callback_export_lock, drain_dialog_callbacks_into,
};
use super::events::{
    ChildPostObservation, CustomDispatchBackend, build_child_edge, build_dispatch_function,
    build_staged_occurrence_lines, build_staged_post_observation_line, check_event_trigger,
    resolve_custom_dispatch_backend, setup_objective_owner, tick_event_export_error,
    xp_advance_command, xp_score_commands,
};
use super::functions::{drain_dynamic_functions_into, resolve_local_refs};
use super::lifecycle::{
    ensure_private_lifecycle_path_available, ensure_private_transition_path_available,
    lifecycle_export_error, transition_export_error,
};
use super::predicates::{
    collect_sand_player_state_predicates, player_state_predicate_json, sand_player_state_predicate,
};
use super::records::{ComponentRecord, ExportResult, component_to_record};
use super::schedules::{schedule_key, schedule_tick_commands};
use super::tags::{dedupe_preserve_order, sort_function_tag_entries};

pub(crate) fn try_export_components_impl(
    namespace: &str,
    ctx: Option<&ExportCtx>,
) -> ExportResult<Vec<ComponentRecord>> {
    use crate::function::{
        ArmorEventDescriptor, ArmorEventKind, ComponentFactory, EventDescriptor,
        FunctionDescriptor, FunctionTagDescriptor,
    };
    use crate::inventory;
    use std::collections::BTreeMap;

    // Dialog callback IDs and registrations are process-global. Hold this for
    // the complete factory/export lifecycle so repeated or concurrent exports
    // cannot inherit callback state from one another. Callback registration
    // happens while component JSON is serialized below (`component_to_record`
    // → `Dialog::to_json` → `DialogAction::to_json`), so resetting here is
    // safe even when a factory returns a cached or otherwise prebuilt dialog
    // — it just re-registers on this export's serialization pass.
    let _dialog_callback_lock = dialog_callback_export_lock();
    sand_components::dialog::reset_dialog_callbacks_for_export();
    let _dialog_callback_reset = DialogCallbackExportReset;

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
    use crate::events::TickExecutionPlans;
    use crate::function::EventDispatch;

    // Categorise events by dispatch type so we can batch-generate aggregators.
    let mut join_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut death_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut respawn_tick_events: Vec<&EventDescriptor> = Vec::new();
    let mut xp_level_up_events: Vec<&EventDescriptor> = Vec::new();
    let mut transition_handlers: Vec<crate::transition::TransitionHandler> = Vec::new();
    let mut transition_private_metadata: BTreeMap<String, (String, String)> = BTreeMap::new();
    // (descriptor, condition_string)
    let mut tick_poll_events: Vec<(&EventDescriptor, String)> = Vec::new();
    // Structured tick-dispatch SandEvents with owned lifecycle (setup/sync).
    // Grouped by event_type_id below so multiple handlers of the same event
    // share one detector/synchronization function — see SandEvent::setup().
    // `event_type_id` is an in-process TypeId used only for grouping; the
    // deterministic generated resource key is derived from `event_type_name`
    // (the canonical `std::any::type_name::<T>()`), not from TypeId or from
    // the set of subscribed handler paths.
    let mut tick_lifecycle_events: Vec<(
        &EventDescriptor,
        std::any::TypeId,
        &'static str,
        crate::events::TickEventDispatch,
        crate::events::EventSetup,
    )> = Vec::new();
    // Same-cycle chained SandEvents (#240) — a child event whose dispatch()
    // is `SandEventDispatch::chain::<Parent>()`. Merged into the same event
    // graph as `tick_lifecycle_events` below so a parent referenced only by
    // chain children still gets its detector/setup generated.
    let mut chain_events: Vec<(
        &EventDescriptor,
        std::any::TypeId,
        &'static str,
        crate::events::ChainEventDispatch,
        crate::events::EventSetup,
    )> = Vec::new();
    // Canonical SandEvent type ids with at least one direct #[event] handler
    // resolving to advancement-backed dispatch — used to reject combining a
    // direct handler with graph composition on the same advancement-backed
    // event (#240 Phase 6; see the advancement-bridge collision check below).
    let mut advancement_handler_type_ids: std::collections::BTreeSet<std::any::TypeId> =
        std::collections::BTreeSet::new();
    // Shared armor watch map — populated by both EventDescriptor ArmorEquip/
    // ArmorUnequip dispatch and the legacy ArmorEventDescriptor entries.
    // Keyed by the exact (slot, item_id, custom_data_snbt) tuple — see
    // `armor::ArmorWatchKey` — so distinct filters can never merge under a
    // lossy sanitized-string key.
    let mut armor_watch_map: BTreeMap<ArmorWatchKey, ArmorWatchEntry> = BTreeMap::new();

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

                records.push(component_to_record(&advancement, ctx)?);
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

            // ── Reusable tracked transition ──────────────────────────────────
            EventDispatch::Tracked(transition) => {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: desc.path.to_string(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: commands.join("\n"),
                });
                transition_handlers.push(crate::transition::TransitionHandler {
                    path: desc.path.to_string(),
                    transition: *transition,
                });
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
                make_tick,
                make_chain,
                revoke,
                event_type_id,
                event_type_name,
                make_setup,
            } => {
                // Evaluate all factories once so we can distinguish "none
                // returned Some" from "more than one returned Some" — see
                // #121. One dispatch strategy silently winning over another
                // would export a working-looking datapack that doesn't match
                // what the `SandEvent` impl actually declared.
                match resolve_custom_dispatch_backend(
                    make_trigger(),
                    make_condition(),
                    make_tick(),
                    make_chain(),
                    desc.path,
                ) {
                    CustomDispatchBackend::Advancement(trigger) => {
                        // Advancement-backed custom (SandEvent) event.
                        // Same entry/body split as the typed Advancement arm.
                        advancement_handler_type_ids.insert(event_type_id());
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

                        records.push(component_to_record(&advancement, ctx)?);
                    }
                    CustomDispatchBackend::TickPoll(condition) => {
                        // Legacy single-fragment `SandEventDispatch::TickCondition`
                        // custom event. Normalized into the same structured
                        // `TickEventDispatch` shape as `SandEventDispatch::tick()`
                        // (matching `SandEventDispatch::normalize()` exactly) and
                        // fed into the same event graph discovery as structured
                        // tick events — not the unrelated legacy `tick_poll_events`
                        // aggregation (bare `EventDispatch::TickPoll`, e.g.
                        // `HoldingItemEvent`/`CurrentlyWearingEvent`, which have no
                        // `SandEvent`/chain-parent concept). A concrete SandEvent
                        // type must resolve to exactly one graph node — and
                        // therefore one generated detector — regardless of
                        // whether its dispatch() used the structured builder or
                        // this compatibility constructor, so a legacy parent
                        // referenced by a chain child never gets a second,
                        // independent detector (#240 follow-up).
                        records.push(ComponentRecord {
                            namespace: namespace.to_string(),
                            dir: "function".to_string(),
                            path: desc.path.to_string(),
                            ext: "mcfunction".to_string(),
                            content_type: "text".to_string(),
                            content: commands.join("\n"),
                        });
                        tick_lifecycle_events.push((
                            desc,
                            event_type_id(),
                            event_type_name(),
                            crate::events::TickEventDispatch::default()
                                .when(crate::condition::Condition::raw(condition)),
                            make_setup(),
                        ));
                    }
                    CustomDispatchBackend::TickLifecycle(tick) => {
                        // Structured tick dispatch — handler body emitted now;
                        // the shared detector/setup is aggregated below, grouped
                        // by event_type_id so multiple handlers of the same
                        // event share one detector and one copy of setup().
                        records.push(ComponentRecord {
                            namespace: namespace.to_string(),
                            dir: "function".to_string(),
                            path: desc.path.to_string(),
                            ext: "mcfunction".to_string(),
                            content_type: "text".to_string(),
                            content: commands.join("\n"),
                        });
                        tick_lifecycle_events.push((
                            desc,
                            event_type_id(),
                            event_type_name(),
                            tick,
                            make_setup(),
                        ));
                    }
                    CustomDispatchBackend::Chain(chain) => {
                        // Same-cycle chained dispatch (#240) — handler body
                        // emitted now; the child node (and its parent chain,
                        // discovered recursively) is resolved into the event
                        // graph below.
                        records.push(ComponentRecord {
                            namespace: namespace.to_string(),
                            dir: "function".to_string(),
                            path: desc.path.to_string(),
                            ext: "mcfunction".to_string(),
                            content_type: "text".to_string(),
                            content: commands.join("\n"),
                        });
                        chain_events.push((
                            desc,
                            event_type_id(),
                            event_type_name(),
                            chain,
                            make_setup(),
                        ));
                    }
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
    death_tick_events.sort_by_key(|desc| desc.path);
    respawn_tick_events.sort_by_key(|desc| desc.path);
    let needs_death_check = !death_tick_events.is_empty() || !respawn_tick_events.is_empty();
    if needs_death_check {
        let init_path = "__sand_death_init";
        let mut init_cmds = vec!["scoreboard objectives add __sand_dc deathCount".to_string()];
        if !respawn_tick_events.is_empty() {
            init_cmds.extend([
                "scoreboard objectives add __sand_tsd minecraft.custom:minecraft.time_since_death"
                    .to_string(),
                "scoreboard objectives add __sand_rp dummy".to_string(),
            ]);
        }
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

        // A single coordinator owns both halves of the lifecycle. The respawn
        // check runs before this tick's new death observation, so even when
        // immediate respawn makes `time_since_death` positive quickly, the
        // death and respawn handlers cannot dispatch from the same observation
        // cycle. This explicit function call is the ordering mechanism; the
        // relative order of minecraft:tick tag entries is irrelevant.
        if !respawn_tick_events.is_empty() {
            let respawn_dispatch_path = "__sand_respawn_dispatch";
            let mut dispatch_cmds: Vec<String> = respawn_tick_events
                .iter()
                .map(|desc| format!("function {namespace}:{}", desc.path))
                .collect();
            dispatch_cmds.push("scoreboard players set @s __sand_rp 0".to_string());
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: respawn_dispatch_path.to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: dispatch_cmds.join("\n"),
            });

            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: "__sand_respawn_check".to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: format!(
                    "execute as @a[scores={{__sand_rp=1,__sand_tsd=1..}}] \
                     run function {namespace}:{respawn_dispatch_path}"
                ),
            });
        }

        let check_path = "__sand_death_check";
        let mut check_cmds: Vec<String> = Vec::new();
        if !respawn_tick_events.is_empty() {
            check_cmds.push(format!("function {namespace}:__sand_respawn_check"));
        }
        check_cmds.push(
            "execute as @a[scores={__sand_dc=1..}] run tag @s add __sand_just_died".to_string(),
        );
        check_cmds.push("scoreboard players set @a __sand_dc 0".to_string());
        // Enter the waiting phase only after the respawn check above. The
        // custom statistic is reset to zero by vanilla on death and increments
        // only while the player is alive, so phase=1 + time_since_death=1..
        // is the first observable post-death active-player state.
        if !respawn_tick_events.is_empty() {
            check_cmds.push(
                "execute as @a[tag=__sand_just_died] run scoreboard players set @s __sand_rp 1"
                    .to_string(),
            );
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
        let mut xp_cmds = xp_score_commands();
        // Step 4 — fire handlers
        xp_cmds.extend(handler_cmds);
        // Step 5 — advance prev
        xp_cmds.push(xp_advance_command());

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

    // ── Structured tick-lifecycle + same-cycle chained SandEvent graph ────────
    //
    // Builds the event dependency graph (#240): direct `#[event]` handlers on
    // tick-lifecycle or chain-backed SandEvents, plus recursively-discovered
    // occurrence parents (a parent referenced only by a child still gets a
    // node — its detector/setup is generated even with no direct handler).
    // Nodes are grouped by event_type_id — an in-process TypeId used only to
    // group/dedupe descriptors belonging to the same concrete SandEvent type
    // during this export run (distinct generic monomorphizations such as
    // `ElevatorUsed<GoUp>` vs `ElevatorUsed<GoDown>` never merge). TypeId is
    // NOT a stable cross-build identifier, so it is never used to derive
    // generated resource paths — see `tick_event_resource_key` below.
    //
    // Single-parent `after` edges retain their established immediate fan-out
    // path. Multi-parent compositions use per-subject occurrence marks and a
    // deterministic staged coordinator; all dependency forms participate in
    // readable cycle validation.
    if !tick_lifecycle_events.is_empty() || !chain_events.is_empty() {
        let mut resolved: BTreeMap<std::any::TypeId, crate::events::graph::EventNode> =
            BTreeMap::new();
        // Advancement-backed graph parents (#240 Phase 6) — never inserted
        // into `resolved`/`nodes`; see `EventGraph::advancement_bridges`.
        let mut advancement_bridges: BTreeMap<String, crate::events::graph::AdvancementBridge> =
            BTreeMap::new();

        for (desc, type_id, type_name, tick, setup) in &tick_lifecycle_events {
            crate::events::graph::discover_node(
                *type_id,
                type_name,
                crate::events::NormalizedEventDispatch::Tick(tick.clone()),
                setup.clone(),
                desc.path,
                &mut resolved,
                &mut advancement_bridges,
            )
            .map_err(|e| tick_event_export_error(e.0))?;
        }
        for (desc, type_id, type_name, chain, setup) in chain_events {
            crate::events::graph::discover_node(
                type_id,
                type_name,
                crate::events::NormalizedEventDispatch::Chain(chain),
                setup,
                desc.path,
                &mut resolved,
                &mut advancement_bridges,
            )
            .map_err(|e| tick_event_export_error(e.0))?;
        }
        for node in resolved.values_mut() {
            node.handlers.sort_unstable();
        }

        let mut nodes = BTreeMap::new();
        for node in resolved.into_values() {
            if let Some(existing) = nodes.insert(node.type_name.to_string(), node) {
                return Err(tick_event_export_error(format!(
                    "canonical event identity collision: distinct Rust event types resolve to `{}`; rename one type so generated event resources cannot overwrite each other",
                    existing.type_name
                )));
            }
        }
        // An advancement-backed graph parent may not also have a direct
        // `#[event]` handler in this phase — see `component.rs`'s
        // advancement-lowering loop above, which generates that handler's
        // own advancement/entry/body independently of this graph and would
        // otherwise silently create two live advancement grants for one
        // criterion (this bridge's synthesized entry, plus the handler's
        // own). Detected by cross-referencing type ids collected while that
        // loop ran (`advancement_handler_type_ids`, populated above).
        for bridge in advancement_bridges.values() {
            if advancement_handler_type_ids.contains(&bridge.type_id) {
                return Err(tick_event_export_error(format!(
                    "advancement-backed graph parent `{}` also has a direct #[event] handler: #240 Phase 6 does not yet support combining a direct handler with graph composition on the same advancement-backed event — split into two SandEvent types (one for the handler, one chained via `after` for the composition), or remove the direct handler",
                    bridge.type_name
                )));
            }
        }
        let graph = crate::events::graph::EventGraph {
            nodes,
            advancement_bridges,
        };
        graph
            .validate_dependencies()
            .map_err(|e| tick_event_export_error(e.0))?;
        let staged_events = graph
            .staged_events()
            .map_err(|e| tick_event_export_error(e.0))?;
        let has_staged_composition = !staged_events.is_empty();
        let occurrence_marked = graph.occurrence_marked_nodes();
        let bounded_parents = graph.bounded_parents();

        // Deterministic resource key derived from the canonical concrete
        // event type name, not from TypeId (not stable across builds) and
        // not from the subscriber list (would rename on add/remove/reorder).
        // Retain a key -> type_name map so an (extremely unlikely) hash
        // collision between two distinct event types is caught rather than
        // silently merging their detectors.
        let mut key_registry: BTreeMap<String, &'static str> = BTreeMap::new();
        for node in graph.nodes.values() {
            let key = tick_event_resource_key(node.type_name);
            if let Some(existing) = key_registry.insert(key.clone(), node.type_name)
                && existing != node.type_name
            {
                return Err(tick_event_export_error(format!(
                    "generated resource key collision: event types `{existing}` and `{}` both \
                     hash to key `{key}` — rename one of the event types to avoid colliding \
                     generated detector/setup paths",
                    node.type_name
                )));
            }
        }
        // Advancement-backed bridge parents (#240 Phase 6) share the same
        // `tick_event_resource_key` keyspace (`__sand_event_advancement_bridge/{key}`)
        // even though they are never graph nodes — extend the same collision
        // guard to them so a 32-bit hash collision between two distinct
        // advancement-backed parent type names is caught rather than
        // silently merging their generated advancement/entry resources.
        for bridge in graph.advancement_bridges.values() {
            let key = tick_event_resource_key(bridge.type_name);
            if let Some(existing) = key_registry.insert(key.clone(), bridge.type_name)
                && existing != bridge.type_name
            {
                return Err(tick_event_export_error(format!(
                    "generated resource key collision: event types `{existing}` and `{}` both \
                     hash to key `{key}` — rename one of the event types to avoid colliding \
                     generated detector/setup paths",
                    bridge.type_name
                )));
            }
        }

        // Some built-in SandEvent types (e.g. `PlayerSneakEvent`) still use
        // the legacy `TickCondition` constructor with a Sand-owned entity
        // predicate condition string. Their raw condition now lives inside
        // the node's own `when`/`unless` clauses (see the `CustomDispatchBackend::TickPoll`
        // arm above), so scan root nodes for it here and emit the internal
        // predicate JSON exactly as the pre-#240-follow-up `TickPoll`
        // aggregation did — only the generation site moved, not the output.
        let mut state_predicates: BTreeMap<&'static str, &'static str> = BTreeMap::new();
        let mut root_checks = Vec::new();
        let mut deferred_root_post_observation = Vec::new();
        for root in graph.roots() {
            let crate::events::graph::NodeOrigin::Root(tick) = &root.origin else {
                continue;
            };
            for cond in tick.when.iter().chain(tick.unless.iter()) {
                collect_sand_player_state_predicates(cond, &mut state_predicates);
            }
        }
        for edge in graph
            .nodes
            .keys()
            .flat_map(|parent| graph.children_of(parent))
        {
            for dependency in edge.persistent {
                collect_sand_player_state_predicates(&dependency.condition, &mut state_predicates);
            }
        }
        for staged in &staged_events {
            for dependency in &staged.persistent {
                collect_sand_player_state_predicates(&dependency.condition, &mut state_predicates);
            }
        }
        for (path, flag) in state_predicates {
            if !records
                .iter()
                .any(|r| r.dir == "predicate" && r.path == path)
            {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "predicate".to_string(),
                    path: path.to_string(),
                    ext: "json".to_string(),
                    content_type: "text".to_string(),
                    content: serde_json::to_string_pretty(&player_state_predicate_json(flag))
                        .unwrap(),
                });
            }
        }

        // Nodes whose key needs a `se_{key}_g` per-player coalescing guard
        // objective, discovered while building dispatch functions below
        // (populated before setup is emitted, since setup is emitted after).
        let mut guarded_children: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();

        // Build root and immediate single-parent dispatch functions up front.
        // Multi-parent nodes are emitted once in deterministic topological
        // order below, so shared parents never duplicate their detectors.
        let mut root_dispatch_ref: BTreeMap<String, String> = BTreeMap::new();
        let mut root_self_guard: BTreeMap<String, Option<String>> = BTreeMap::new();
        for root in graph.roots() {
            let crate::events::graph::NodeOrigin::Root(tick) = &root.origin else {
                unreachable!("roots() only yields Root-origin nodes");
            };
            let plans = tick.execution_plans();
            let needs_guard = matches!(&plans, TickExecutionPlans::Plans(p) if p.len() > 1);
            let key = tick_event_resource_key(root.type_name);
            let self_guard = needs_guard.then(|| format!("se_{key}_f"));

            let dispatch_reachable = match &plans {
                TickExecutionPlans::Unconditional => true,
                TickExecutionPlans::Plans(p) => !p.is_empty(),
            };
            let dispatch_ref = if dispatch_reachable {
                Some(build_dispatch_function(
                    root.type_name,
                    &graph,
                    namespace,
                    self_guard.as_deref(),
                    &occurrence_marked,
                    &mut guarded_children,
                    &mut records,
                ))
            } else {
                None
            };
            root_dispatch_ref.insert(root.type_name.to_string(), dispatch_ref.unwrap_or_default());
            root_self_guard.insert(root.type_name.to_string(), self_guard);
        }

        let mut staged_evaluations = Vec::new();
        for staged in &staged_events {
            let edge = staged.condition_edge();
            let commands = build_child_edge(
                &edge,
                &graph,
                namespace,
                &occurrence_marked,
                ChildPostObservation::DeferredByOccurrence,
                &mut guarded_children,
                &mut records,
            );
            let key = tick_event_resource_key(&staged.child);
            let path = format!("__sand_event_multi_eval/{key}");
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: path.clone(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: commands,
            });
            staged_evaluations.push((staged.clone(), format!("{namespace}:{path}")));
        }

        let staged_by_child: BTreeMap<String, crate::events::graph::StagedEvent> = staged_events
            .iter()
            .map(|staged| (staged.child.clone(), staged.clone()))
            .collect();
        let deferred_attempt_marked: std::collections::BTreeSet<String> = graph
            .nodes
            .values()
            .filter_map(|node| match &node.origin {
                crate::events::graph::NodeOrigin::Chained { occurrence, .. }
                    if matches!(
                        occurrence.as_slice(),
                        [crate::events::graph::OccurrenceDependency::After(_)]
                    ) && occurrence_marked.contains(node.type_name)
                        && !node.setup.post_observation.is_empty() =>
                {
                    Some(node.type_name.to_string())
                }
                _ => None,
            })
            .collect();
        let mut deferred_post_refs = BTreeMap::new();
        for name in graph
            .occurrence_topological_nodes()
            .map_err(|e| tick_event_export_error(e.0))?
        {
            let node = &graph.nodes[&name];
            if node.setup.post_observation.is_empty()
                || (!staged_by_child.contains_key(&name)
                    && !deferred_attempt_marked.contains(&name))
            {
                continue;
            }
            let key = tick_event_resource_key(&name);
            let path = format!("__sand_event_multi_post/{key}");
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: path.clone(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: node.setup.post_observation.join("\n"),
            });
            deferred_post_refs.insert(name, format!("{namespace}:{path}"));
        }

        // Emit setup (objectives, + a `_g` guard objective for any node that
        // is chained-to via more than one OR-alternative edge condition) for
        // every node exactly once.
        for node in graph.nodes.values() {
            let key = tick_event_resource_key(node.type_name);
            let mut setup_cmds: Vec<String> = Vec::new();
            for cmd in &node.setup.objectives {
                if !setup_cmds.contains(cmd) {
                    setup_cmds.push(cmd.clone());
                }
            }
            if let Some(Some(guard)) = root_self_guard.get(node.type_name) {
                setup_cmds.push(format!("scoreboard objectives add {guard} dummy"));
            }
            if guarded_children.contains(node.type_name) {
                setup_cmds.push(format!("scoreboard objectives add se_{key}_g dummy"));
            }
            if occurrence_marked.contains(node.type_name) {
                let objective = format!("se_{key}_o");
                if let Some(owner) = setup_objective_owner(&graph, &objective) {
                    return Err(tick_event_export_error(format!(
                        "generated occurrence-state identity collision for event `{}`: setup for `{owner}` already declares reserved objective `{objective}`",
                        node.type_name,
                    )));
                }
                setup_cmds.push(format!("scoreboard objectives add {objective} dummy"));
            }
            if bounded_parents.contains(node.type_name) {
                // One shared per-subject age objective per bounded parent,
                // regardless of how many children or distinct `.within(...)`
                // windows read it — see `EventGraph::bounded_parents` and
                // `BoundedDependency::resolve`.
                let objective = format!("se_{key}_wa");
                if let Some(owner) = setup_objective_owner(&graph, &objective) {
                    return Err(tick_event_export_error(format!(
                        "generated occurrence-state identity collision for event `{}`: setup for `{owner}` already declares reserved objective `{objective}`",
                        node.type_name,
                    )));
                }
                setup_cmds.push(format!("scoreboard objectives add {objective} dummy"));
            }
            if deferred_attempt_marked.contains(node.type_name) {
                let objective = format!("se_{key}_c");
                if let Some(owner) = setup_objective_owner(&graph, &objective) {
                    return Err(tick_event_export_error(format!(
                        "generated occurrence-state identity collision for event `{}`: setup for `{owner}` already declares reserved objective `{objective}`",
                        node.type_name,
                    )));
                }
                setup_cmds.push(format!("scoreboard objectives add {objective} dummy"));
            }
            if staged_events.iter().any(|staged| {
                staged.child == node.type_name
                    && staged.occurrence.iter().any(|dependency| {
                        matches!(
                            dependency,
                            crate::events::graph::OccurrenceDependency::AfterAny(_)
                        )
                    })
            }) {
                let objective = format!("se_{key}_m");
                if let Some(owner) = setup_objective_owner(&graph, &objective) {
                    return Err(tick_event_export_error(format!(
                        "generated occurrence-state identity collision for event `{}`: setup for `{owner}` already declares reserved objective `{objective}`",
                        node.type_name,
                    )));
                }
                let guard = format!("scoreboard objectives add {objective} dummy");
                if !setup_cmds.contains(&guard) {
                    setup_cmds.push(guard);
                }
            }
            if !setup_cmds.is_empty() {
                let init_path = format!("__sand_event_setup/{key}");
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: init_path.clone(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: setup_cmds.join("\n"),
                });
                tag_map
                    .entry("minecraft:load".to_string())
                    .or_default()
                    .push(format!("{namespace}:{init_path}"));
            }
        }

        // Emit the tick-check wiring for every root: pre_observation, then
        // the detection execute line(s) calling the dispatch function built
        // above, then post_observation — unconditionally, regardless of
        // whether detection matched this tick, so tracked/synchronized state
        // always advances (required for delta-tracking events).
        for root in graph.roots() {
            let crate::events::graph::NodeOrigin::Root(tick) = &root.origin else {
                unreachable!("roots() only yields Root-origin nodes");
            };
            let plans = tick.execution_plans();
            let self_guard = root_self_guard.get(root.type_name).and_then(|g| g.clone());
            let key = tick_event_resource_key(root.type_name);

            let mut tick_cmds: Vec<String> = Vec::new();
            tick_cmds.extend(root.setup.pre_observation.iter().cloned());

            let dispatch_ref = root_dispatch_ref
                .get(root.type_name)
                .cloned()
                .unwrap_or_default();
            if !dispatch_ref.is_empty() {
                if let Some(guard) = &self_guard {
                    if has_staged_composition {
                        tick_cmds.push(format!(
                            "execute as @a run scoreboard players set @s {guard} 0"
                        ));
                    } else {
                        tick_cmds.push(format!("scoreboard players set @a {guard} 0"));
                    }
                }
                match &plans {
                    TickExecutionPlans::Unconditional => {
                        tick_cmds.push(format!("execute as @a at @s run function {dispatch_ref}"));
                    }
                    TickExecutionPlans::Plans(plans) => {
                        for plan in plans {
                            let mut clauses: Vec<String> = Vec::new();
                            if let Some(guard) = &self_guard {
                                clauses.push(format!("unless score @s {guard} matches 1"));
                            }
                            clauses.extend(plan.iter().cloned());
                            if clauses.is_empty() {
                                tick_cmds.push(format!(
                                    "execute as @a at @s run function {dispatch_ref}"
                                ));
                            } else {
                                tick_cmds.push(format!(
                                    "execute as @a at @s {} run function {dispatch_ref}",
                                    clauses.join(" ")
                                ));
                            }
                        }
                    }
                }
            }

            if has_staged_composition {
                deferred_root_post_observation.extend(root.setup.post_observation.iter().cloned());
            } else {
                tick_cmds.extend(root.setup.post_observation.iter().cloned());
            }

            if !tick_cmds.is_empty() {
                let check_path = format!("__sand_event_check/{key}");
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: check_path.clone(),
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: tick_cmds.join("\n"),
                });
                let check_ref = format!("{namespace}:{check_path}");
                if has_staged_composition {
                    root_checks.push(check_ref);
                } else {
                    tag_map
                        .entry("minecraft:tick".to_string())
                        .or_default()
                        .push(check_ref);
                }
            }
        }

        if has_staged_composition {
            let mut coordinator = Vec::new();
            for name in &occurrence_marked {
                let key = tick_event_resource_key(name);
                coordinator.push(format!(
                    "execute as @a run scoreboard players set @s se_{key}_o 0"
                ));
            }
            for (staged, _) in &staged_evaluations {
                if staged.occurrence.iter().any(|dependency| {
                    matches!(
                        dependency,
                        crate::events::graph::OccurrenceDependency::AfterAny(_)
                    )
                }) {
                    let key = tick_event_resource_key(&staged.child);
                    coordinator.push(format!(
                        "execute as @a run scoreboard players set @s se_{key}_m 0"
                    ));
                }
            }
            for name in &deferred_attempt_marked {
                let key = tick_event_resource_key(name);
                coordinator.push(format!(
                    "execute as @a run scoreboard players set @s se_{key}_c 0"
                ));
            }
            coordinator.extend(
                root_checks
                    .iter()
                    .map(|check_ref| format!("function {check_ref}")),
            );
            // Bounded (`.within`) age-counter maintenance: exactly one shared
            // per-subject age update per bounded parent, run once that
            // parent's own occurrence for this tick is fully committed and
            // before any staged evaluation reads it. Refresh always wins over
            // increment (mutually exclusive `if`/`unless` branches), so a
            // parent firing on the current tick unconditionally resets its
            // age to 0 regardless of the prior age — this is what makes a
            // window of 1 tick behave identically to `after::<E>()`, and
            // guarantees no staged child ever observes a stale age computed
            // before this tick's occurrence was known.
            //
            // A bounded parent that is itself a root (or reached only through
            // the immediate single-`after` fast path folded into a root's own
            // dispatch tree) has its `se_{key}_o` mark fully committed the
            // moment `root_checks` above finishes, so its age update belongs
            // here. A bounded parent that is itself staged (its own
            // `after_any`/`after_all`/`while_`/`within` clauses) only gets its
            // mark set when ITS OWN staged evaluation call below runs — for
            // those, the age update is emitted inline in the topological loop
            // immediately after that specific evaluation, never here.
            //
            // Minecraft scoreboard values are signed 32-bit; an unbounded
            // `add ... 1` on a permanently-idle parent would eventually
            // overflow and wrap negative, which would incorrectly re-satisfy
            // `age <= N - 1` for every window until the parent fires again.
            // The increment is therefore guarded to stop at
            // `TickWindow::MAX_TICKS` (the largest representable window) —
            // an age that has already reached the largest possible window
            // width is permanently "expired" for every valid `TickWindow`,
            // so clamping there rather than at `i32::MAX` is both safe and
            // ties the sentinel to the supported API range instead of an
            // arbitrary implementation constant.
            let staged_child_names: std::collections::BTreeSet<String> =
                staged_by_child.keys().cloned().collect();
            let age_sentinel = crate::events::TickWindow::MAX_TICKS;
            let bounded_age_update = |name: &str| -> [String; 2] {
                let key = tick_event_resource_key(name);
                [
                    format!(
                        "execute as @a if score @s se_{key}_o matches 1 run scoreboard players set @s se_{key}_wa 0"
                    ),
                    format!(
                        "execute as @a unless score @s se_{key}_o matches 1 unless score @s se_{key}_wa matches {age_sentinel}.. run scoreboard players add @s se_{key}_wa 1"
                    ),
                ]
            };
            for name in bounded_parents.difference(&staged_child_names) {
                coordinator.extend(bounded_age_update(name));
            }
            for (staged, evaluation_ref) in staged_evaluations {
                coordinator.extend(build_staged_occurrence_lines(
                    &staged,
                    &evaluation_ref,
                    namespace,
                    &mut records,
                ));
                if bounded_parents.contains(&staged.child) {
                    coordinator.extend(bounded_age_update(&staged.child));
                }
            }
            for name in graph
                .occurrence_topological_nodes()
                .map_err(|e| tick_event_export_error(e.0))?
                .into_iter()
                .rev()
            {
                let Some(post_ref) = deferred_post_refs.get(&name) else {
                    continue;
                };
                if let Some(staged) = staged_by_child.get(&name) {
                    coordinator.push(build_staged_post_observation_line(staged, post_ref));
                } else {
                    let key = tick_event_resource_key(&name);
                    coordinator.push(format!(
                        "execute as @a at @s if score @s se_{key}_c matches 1 run function {post_ref}"
                    ));
                }
            }
            coordinator.extend(deferred_root_post_observation);

            let coordinator_path = "__sand_event_cycle";
            ensure_private_lifecycle_path_available(&records, coordinator_path)?;
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: coordinator_path.to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: coordinator.join("\n"),
            });
            tag_map
                .entry("minecraft:tick".to_string())
                .or_default()
                .push(format!("{namespace}:{coordinator_path}"));
        }

        // ── Advancement-backed graph parent bridge (#240 Phase 6) ──────────
        //
        // An advancement-backed parent referenced by some child's sole
        // `after::<Parent>()` is never a graph node (see
        // `EventGraph::advancement_bridges`) — its detection stays owned by
        // a synthesized advancement + entry function, generated here rather
        // than through the ordinary per-handler advancement lowering above.
        // This phase requires zero direct `#[event]` handlers on the
        // bridged type (checked earlier via `advancement_handler_type_ids`),
        // so there is exactly one entry per bridged parent regardless of how
        // many children depend on it — multiple children append multiple
        // condition-gated dispatch lines to that same entry, all running
        // under the same `@s` the vanilla reward mechanism already binds to
        // the triggering player. `EventSetup` is intentionally not consulted
        // for the bridged parent itself, matching the pre-existing
        // advancement-lowering paths above (neither ever wires
        // pre_observation/post_observation/objectives for advancement
        // dispatch) — a dependent child's own `EventSetup` is unaffected and
        // still applied normally, since the child remains an ordinary graph
        // node.
        for (parent_name, bridge) in &graph.advancement_bridges {
            let children = graph.children_of(parent_name);
            let mut bridge_cmds = Vec::new();
            for edge in &children {
                bridge_cmds.push(build_child_edge(
                    edge,
                    &graph,
                    namespace,
                    &occurrence_marked,
                    ChildPostObservation::Inline,
                    &mut guarded_children,
                    &mut records,
                ));
            }

            let key = tick_event_resource_key(parent_name);
            let trigger = match (bridge.event_dispatch)().normalize() {
                crate::events::NormalizedEventDispatch::Advancement(trigger) => trigger,
                _ => unreachable!(
                    "AdvancementBridge is only constructed for advancement-backed dispatch"
                ),
            };
            let entry_path = format!("__sand_event_advancement_bridge/{key}");
            let entry_ref = format!("{namespace}:{entry_path}");
            let advancement_id = format!("{namespace}:{entry_path}");

            // Preserve the same revoke-before-effect ordering as the
            // existing per-handler advancement lowering (`component.rs`
            // above): the advancement always re-arms before any dependent
            // runs, so it can fire again on a later criterion match
            // regardless of what a dependent child's own condition does.
            let mut entry_cmds = Vec::new();
            if (bridge.event_revoke)() {
                entry_cmds.push(format!("advancement revoke @s only {advancement_id}"));
            }
            entry_cmds.extend(bridge_cmds);

            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: entry_path,
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: entry_cmds.join("\n"),
            });

            check_event_trigger(&trigger, &advancement_id, parent_name, ctx)?;
            let advancement = sand_components::Advancement::new(
                advancement_id
                    .parse()
                    .expect("generated advancement bridge id is a valid resource location"),
            )
            .criterion("event", sand_components::Criterion::new(trigger))
            .rewards(sand_components::AdvancementRewards::new().function(entry_ref));
            records.push(component_to_record(&advancement, ctx)?);
        }
    }

    // ── Armor check aggregation ───────────────────────────────────────────────
    if !armor_watch_map.is_empty() {
        let armor_path = "__sand_armor_check";
        let mut armor_cmds: Vec<String> = Vec::new();
        let armor_tag_keys = allocate_armor_tag_keys(&armor_watch_map);

        for (key, (slot, item_id, custom_data_snbt, handlers)) in &armor_watch_map {
            let tag_key = &armor_tag_keys[key];
            let tag_now = format!("__armor_{tag_key}_now");
            let tag_had = format!("__armor_{tag_key}_had");
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

    // ── Tracked transitions + typed-state lifecycle ───────────────────────────
    // Link-time declarations are rebuilt on every export. Manual registries are
    // still drained after all factories so existing registration paths remain
    // supported.
    {
        let mut transition_predicates = BTreeMap::new();
        for handler in &transition_handlers {
            if let crate::TrackedSource::BooleanCondition { condition, .. } =
                handler.transition.source
                && let Some((path, flag)) = sand_player_state_predicate(condition)
            {
                transition_predicates.insert(path, flag);
            }
        }
        for (path, flag) in transition_predicates {
            if !records
                .iter()
                .any(|record| record.dir == "predicate" && record.path == path)
            {
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "predicate".to_string(),
                    path: path.to_string(),
                    ext: "json".to_string(),
                    content_type: "text".to_string(),
                    content: serde_json::to_string_pretty(&player_state_predicate_json(flag))
                        .unwrap(),
                });
            }
        }

        let transition_plan =
            crate::transition::resolve_transition_plan(namespace, &transition_handlers)
                .map_err(transition_export_error)?;
        for generated in &transition_plan.functions {
            transition_private_metadata.insert(
                generated.path.clone(),
                (generated.tracker_id.clone(), generated.source.clone()),
            );
            ensure_private_transition_path_available(
                &records,
                &generated.path,
                &generated.tracker_id,
                &generated.source,
            )?;
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: generated.path.clone(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: generated.commands.join("\n"),
            });
        }

        let mut automatic =
            crate::state::registry::automatic_lifecycle().map_err(lifecycle_export_error)?;
        let transition_objectives = &transition_plan.private_objectives;
        for command in &automatic.load_commands {
            if let Some(objective) = command.split_whitespace().nth(3)
                && let Some((tracker_id, source)) = transition_objectives.get(objective)
            {
                return Err(transition_export_error(format!(
                    "tracker `{tracker_id}` source `{source}` generated private objective `{objective}`, which collides with an automatic state declaration"
                )));
            }
        }
        automatic
            .load_commands
            .extend(transition_plan.load_commands);
        automatic
            .tick_commands
            .extend(transition_plan.tick_commands);
        let manual_load_cmds = crate::state::drain_load_commands();
        for command in &manual_load_cmds {
            if let Some(objective) = command.split_whitespace().nth(3)
                && let Some((tracker_id, source)) = transition_objectives.get(objective)
            {
                return Err(transition_export_error(format!(
                    "tracker `{tracker_id}` source `{source}` generated private objective `{objective}`, which collides with a manual lifecycle registration"
                )));
            }
        }
        let mut load_definitions: BTreeMap<String, (String, String)> = BTreeMap::new();

        for command in automatic.load_commands.into_iter().chain(manual_load_cmds) {
            let mut parts = command.split_whitespace();
            let parsed = match (
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
            ) {
                (
                    Some("scoreboard"),
                    Some("objectives"),
                    Some("add"),
                    Some(objective),
                    Some(criterion),
                ) if parts.next().is_none() => Some((objective.to_string(), criterion.to_string())),
                _ => None,
            };

            if let Some((objective, criterion)) = parsed {
                match load_definitions.get(&objective) {
                    Some((existing, _)) if existing == &criterion => {}
                    Some((existing, _)) => {
                        return Err(lifecycle_export_error(format!(
                            "conflicting objective `{objective}`: criterion `{existing}` versus `{criterion}`"
                        )));
                    }
                    None => {
                        load_definitions.insert(objective, (criterion, command));
                    }
                }
            } else {
                return Err(lifecycle_export_error(format!(
                    "invalid registered load command `{command}`"
                )));
            }
        }
        let load_cmds: Vec<String> = load_definitions
            .into_values()
            .map(|(_, command)| command)
            .collect();
        if !load_cmds.is_empty() {
            let path = "__sand_lifecycle_load";
            ensure_private_lifecycle_path_available(&records, path)?;
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

        let init_path = "__sand_lifecycle_init";
        if !automatic.init_commands.is_empty() {
            ensure_private_lifecycle_path_available(&records, init_path)?;
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: init_path.to_string(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: automatic.init_commands.join("\n"),
            });
        }

        let mut tick_cmds = Vec::new();
        if !automatic.init_commands.is_empty() {
            tick_cmds.push(format!(
                "execute as @a run function {namespace}:{init_path}"
            ));
        }
        tick_cmds.extend(
            automatic
                .tick_commands
                .into_iter()
                .map(|command| format!("execute as @a run {command}")),
        );
        tick_cmds.extend(crate::state::drain_tick_commands());
        if !tick_cmds.is_empty() {
            let path = "__sand_lifecycle_tick";
            ensure_private_lifecycle_path_available(&records, path)?;
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
    // registered while constructing dialog components are not missed. The
    // export-scope lock/reset guards are acquired at the top of this
    // function — see `_dialog_callback_lock` / `_dialog_callback_reset`.
    drain_dialog_callbacks_into(&mut records, &mut tag_map, namespace);

    // ── FunctionTagDescriptors ────────────────────────────────────────────────
    // Append user-declared function tag entries after Sand-owned setup/dispatcher
    // entries so load/tick infrastructure exists before user functions run.
    let mut user_tag_entries: Vec<(String, String)> = inventory::iter::<FunctionTagDescriptor>()
        .map(|desc| {
            (
                desc.tag.to_string(),
                format!("{}:{}", namespace, desc.function_path),
            )
        })
        .collect();
    sort_function_tag_entries(&mut user_tag_entries);
    for (tag, function) in user_tag_entries {
        tag_map.entry(tag).or_default().push(function);
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

    for path in [
        "__sand_lifecycle_load",
        "__sand_lifecycle_init",
        "__sand_lifecycle_tick",
    ] {
        if records
            .iter()
            .filter(|record| record.dir == "function" && record.path == path)
            .count()
            > 1
        {
            return Err(lifecycle_export_error(format!(
                "generated private function `{path}` collides with a later generated function"
            )));
        }
    }
    let mut private_transition_paths = std::collections::BTreeSet::new();
    for record in records
        .iter()
        .filter(|record| record.dir == "function" && record.path.starts_with("__sand_transition/"))
    {
        if !private_transition_paths.insert(record.path.as_str()) {
            let (tracker_id, source) = transition_private_metadata
                .get(&record.path)
                .map(|(id, source)| (id.as_str(), source.as_str()))
                .unwrap_or(("unknown", "unknown"));
            return Err(transition_export_error(format!(
                "tracker `{tracker_id}` source `{source}` generated private function `{}`, which collides with a later generated function",
                record.path,
            )));
        }
    }

    // Validate every collected function before accepting any record. Typed
    // commands validate structurally before rendering; this final string
    // boundary always enforces line integrity and only inspects argument
    // positions in confidently recognized top-level command grammar. Explicit
    // `cmd::raw`, macro, unknown, and modded syntax otherwise remains verbatim.
    let command_profile = sand_commands::CommandProfile::new(
        ctx.map_or(sand_version::LATEST_KNOWN, |ctx| ctx.requested_version),
        ctx.is_some_and(|ctx| ctx.is_fallback),
    );
    validate_function_records(&mut records, &command_profile)?;

    Ok(records)
}
