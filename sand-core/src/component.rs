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
            .try_content_for(ctx.map(|c| c.caps))
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

fn xp_score_commands() -> Vec<String> {
    vec![
        "execute as @a store result score @s __sand_xp_lvl run experience query @s levels"
            .to_string(),
        "execute as @a unless score @s __sand_xp_seen matches 1 \
         run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
            .to_string(),
        "scoreboard players set @a __sand_xp_seen 1".to_string(),
        "execute as @a run scoreboard players operation @s __sand_xp_delta = @s __sand_xp_lvl"
            .to_string(),
        "execute as @a run scoreboard players operation @s __sand_xp_delta -= @s __sand_xp_prev"
            .to_string(),
    ]
}

fn xp_advance_command() -> String {
    "execute as @a run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
        .to_string()
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
    // chain parents (a parent referenced only by a chain child still gets a
    // node — its detector/setup is generated even with no direct handler).
    // Nodes are grouped by event_type_id — an in-process TypeId used only to
    // group/dedupe descriptors belonging to the same concrete SandEvent type
    // during this export run (distinct generic monomorphizations such as
    // `ElevatorUsed<GoUp>` vs `ElevatorUsed<GoDown>` never merge). TypeId is
    // NOT a stable cross-build identifier, so it is never used to derive
    // generated resource paths — see `tick_event_resource_key` below.
    //
    // This phase supports at most one parent per event, so the graph is
    // always a forest (every node has zero or one incoming chain edge) —
    // cycles are rejected by a parent-pointer walk during discovery.
    if !tick_lifecycle_events.is_empty() || !chain_events.is_empty() {
        let mut resolved: BTreeMap<std::any::TypeId, crate::events::graph::EventNode> =
            BTreeMap::new();

        for (desc, type_id, type_name, tick, setup) in &tick_lifecycle_events {
            crate::events::graph::discover_node(
                *type_id,
                type_name,
                crate::events::NormalizedEventDispatch::Tick(tick.clone()),
                setup.clone(),
                desc.path,
                &mut resolved,
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
            )
            .map_err(|e| tick_event_export_error(e.0))?;
        }
        for node in resolved.values_mut() {
            node.handlers.sort_unstable();
        }

        let graph = crate::events::graph::EventGraph {
            nodes: resolved
                .into_values()
                .map(|n| (n.type_name.to_string(), n))
                .collect(),
        };

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

        // Some built-in SandEvent types (e.g. `PlayerSneakEvent`) still use
        // the legacy `TickCondition` constructor with a Sand-owned entity
        // predicate condition string. Their raw condition now lives inside
        // the node's own `when`/`unless` clauses (see the `CustomDispatchBackend::TickPoll`
        // arm above), so scan root nodes for it here and emit the internal
        // predicate JSON exactly as the pre-#240-follow-up `TickPoll`
        // aggregation did — only the generation site moved, not the output.
        let mut state_predicates: BTreeMap<&'static str, &'static str> = BTreeMap::new();
        for root in graph.roots() {
            let crate::events::graph::NodeOrigin::Root(tick) = &root.origin else {
                continue;
            };
            for cond in tick.when.iter().chain(tick.unless.iter()) {
                if let crate::condition::Condition::Raw(s) = cond
                    && let Some((path, flag)) = sand_player_state_predicate(s)
                {
                    state_predicates.insert(path, flag);
                }
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

        // Build every node's dispatch function up front (recursing from
        // roots down through children — safe without memoization since each
        // node has at most one parent, so this visits each node exactly
        // once), collecting the root -> dispatch_ref mapping used below.
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
                    &mut guarded_children,
                    &mut records,
                ))
            } else {
                None
            };
            root_dispatch_ref.insert(root.type_name.to_string(), dispatch_ref.unwrap_or_default());
            root_self_guard.insert(root.type_name.to_string(), self_guard);
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
                    tick_cmds.push(format!("scoreboard players set @a {guard} 0"));
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

            tick_cmds.extend(root.setup.post_observation.iter().cloned());

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
                tag_map
                    .entry("minecraft:tick".to_string())
                    .or_default()
                    .push(format!("{namespace}:{check_path}"));
            }
        }
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
    // registered while constructing dialog components are not missed.
    let _dialog_callback_lock = dialog_callback_export_lock();
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
    // command builders and generated framework commands share this final
    // boundary. Explicit `cmd::raw` lines bypass typed construction but still
    // receive conservative file-integrity/foundational checks here.
    let command_profile = sand_commands::CommandProfile::new(
        ctx.map_or(sand_version::LATEST_KNOWN, |ctx| ctx.requested_version),
        ctx.is_some_and(|ctx| ctx.is_fallback),
    );
    validate_function_records(&mut records, &command_profile)?;

    Ok(records)
}

fn validate_function_records(
    records: &mut [ComponentRecord],
    command_profile: &sand_commands::CommandProfile,
) -> ExportResult<()> {
    for record in records
        .iter_mut()
        .filter(|record| record.ext == "mcfunction")
    {
        let location =
            sand_components::ResourceLocation::new(record.namespace.clone(), record.path.clone())?;
        let mut validated = Vec::new();
        for (index, line) in record.content.lines().enumerate() {
            let line = sand_commands::render::validate_collected_line(line, command_profile)
                .map_err(|error| ComponentExportError::ComponentValidation {
                    location: location.clone(),
                    kind: "function".to_string(),
                    field: format!("commands[{index}].{}", error.field),
                    message: format!(
                        "{} (Minecraft profile {})",
                        error,
                        command_profile.requested_version()
                    ),
                })?;
            validated.push(line);
        }
        record.content = validated.join("\n");
    }
    Ok(())
}

fn ensure_private_lifecycle_path_available(
    records: &[ComponentRecord],
    path: &str,
) -> ExportResult<()> {
    if records
        .iter()
        .any(|record| record.dir == "function" && record.path == path)
    {
        return Err(lifecycle_export_error(format!(
            "generated private function `{path}` collides with a user or component function"
        )));
    }
    Ok(())
}

fn lifecycle_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "lifecycle")
            .expect("fixed lifecycle resource location is valid"),
        kind: "state_lifecycle".to_string(),
        field: "declarations".to_string(),
        message: message.into(),
    }
}

fn transition_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "transitions")
            .expect("fixed transition resource location is valid"),
        kind: "tracked_transition".to_string(),
        field: "trackers".to_string(),
        message: message.into(),
    }
}

fn ensure_private_transition_path_available(
    records: &[ComponentRecord],
    path: &str,
    tracker_id: &str,
    source: &str,
) -> ExportResult<()> {
    if records
        .iter()
        .any(|record| record.dir == "function" && record.path == path)
    {
        return Err(transition_export_error(format!(
            "tracker `{tracker_id}` source `{source}` generated private function `{path}`, which collides with a user or component function"
        )));
    }
    Ok(())
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

fn sort_function_tag_entries(entries: &mut [(String, String)]) {
    entries.sort_by(|(left_tag, left_function), (right_tag, right_function)| {
        left_tag
            .cmp(right_tag)
            .then_with(|| left_function.cmp(right_function))
    });
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

/// FNV-1a hash of a string, rendered as lowercase hex — used to derive stable,
/// deterministic generated function paths.
fn fnv1a_hex(input: impl AsRef<str>) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in input.as_ref().bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

/// Deterministic generated resource key for a tick-lifecycle `SandEvent`
/// group, derived from the canonical concrete event type name
/// (`std::any::type_name::<T>()`).
///
/// This is intentionally **not** derived from `TypeId` (not stable across
/// compiler versions/builds) and **not** from the subscribed handler path
/// list (would rename the generated detector/setup whenever a handler is
/// added, removed, or re-registered in a different order). The same concrete
/// event type always produces the same key, and distinct generic
/// monomorphizations (different canonical type names) always produce
/// different keys — see the caller for the collision guard against 32-bit
/// hash collisions between two distinct type names.
fn tick_event_resource_key(canonical_type_name: &str) -> String {
    fnv1a_hex(canonical_type_name)
}

/// Human-readable description of which part of two `SandEvent` definitions
/// for the same event type differ, for the conflicting-descriptor export
/// error.
fn tick_event_export_error(message: impl Into<String>) -> ComponentExportError {
    ComponentExportError::ComponentValidation {
        location: sand_components::ResourceLocation::new("sand", "events")
            .expect("fixed events resource location is valid"),
        kind: "sand_event".to_string(),
        field: "dispatch".to_string(),
        message: message.into(),
    }
}

/// Build (or reuse the sole handler as) `name`'s dispatch function: direct
/// `#[event]` handler calls (sorted), then child edges (sorted by canonical
/// child name, recursing into each child's own dispatch function first so
/// nested references always resolve), inheriting the current `@s`/position —
/// never re-issuing `execute as @a`.
///
/// This function never contains lifecycle (`pre_observation`/
/// `post_observation`) commands, for either roots or chained nodes. Root
/// lifecycle stays in the tick-check wrapper (run unconditionally every tick
/// regardless of detection, established by the caller). A chained node's own
/// lifecycle is wrapped around *its* condition test by its parent — see
/// [`build_child_edge`] — so it always observes before testing and always
/// advances after testing, whether or not its condition matched.
///
/// Safe to call at most once per node without memoization: this phase
/// supports only one parent per event, so the graph is a forest and every
/// node is reached from exactly one call site (its unique parent, or a
/// top-level call for roots).
fn build_dispatch_function(
    name: &str,
    graph: &crate::events::graph::EventGraph,
    namespace: &str,
    self_guard: Option<&str>,
    guarded_children: &mut std::collections::BTreeSet<String>,
    records: &mut Vec<ComponentRecord>,
) -> String {
    let node = &graph.nodes[name];
    let key = tick_event_resource_key(node.type_name);
    let children = graph.children_of(name);

    let needs_wrapper = node.handlers.len() != 1 || !children.is_empty() || self_guard.is_some();

    if !needs_wrapper {
        return format!("{namespace}:{}", node.handlers[0]);
    }

    let mut cmds: Vec<String> = Vec::new();
    if let Some(guard) = self_guard {
        cmds.push(format!("scoreboard players set @s {guard} 1"));
    }
    for handler in &node.handlers {
        cmds.push(format!("function {namespace}:{handler}"));
    }
    for edge in &children {
        cmds.push(build_child_edge(
            edge,
            graph,
            namespace,
            guarded_children,
            records,
        ));
    }

    let dispatch_path = format!("__sand_event_dispatch/{key}");
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: dispatch_path.clone(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: cmds.join("\n"),
    });
    format!("{namespace}:{dispatch_path}")
}

/// Build the call line a parent uses to reach one child edge, and — when the
/// child owns lifecycle commands — the dedicated `__sand_event_observe/<child>`
/// function that wraps its condition test between `pre_observation` and
/// `post_observation`.
///
/// The required per-invocation order for a chained child is always:
/// `pre_observation` → condition test → handler/descendant dispatch if
/// matched → `post_observation`, with `post_observation` reached whether or
/// not the condition matched (mirroring the tick-lifecycle contract for
/// roots). A child with no lifecycle commands has no such ordering concern,
/// so it keeps the direct (no-wrapper) call shape.
fn build_child_edge(
    edge: &crate::events::graph::EventEdge,
    graph: &crate::events::graph::EventGraph,
    namespace: &str,
    guarded_children: &mut std::collections::BTreeSet<String>,
    records: &mut Vec<ComponentRecord>,
) -> String {
    let child_node = &graph.nodes[&edge.child];
    let child_ref = build_dispatch_function(
        &edge.child,
        graph,
        namespace,
        None,
        guarded_children,
        records,
    );
    let has_lifecycle = !child_node.setup.pre_observation.is_empty()
        || !child_node.setup.post_observation.is_empty();

    // Build the condition-gated dispatch line(s) that reach `dispatch_ref`,
    // shared by both the no-lifecycle (emitted directly into the caller's
    // command list) and lifecycle (emitted into the observe function body)
    // shapes.
    let conditional_dispatch_lines = |dispatch_ref: &str,
                                      guarded_children: &mut std::collections::BTreeSet<String>,
                                      records: &mut Vec<ComponentRecord>|
     -> Vec<String> {
        match edge.execution_plans() {
            crate::events::TickExecutionPlans::Unconditional => {
                vec![format!("function {dispatch_ref}")]
            }
            crate::events::TickExecutionPlans::Plans(plans) if plans.len() <= 1 => plans
                .into_iter()
                .next()
                .map(|plan| {
                    if plan.is_empty() {
                        format!("function {dispatch_ref}")
                    } else {
                        format!("execute {} run function {dispatch_ref}", plan.join(" "))
                    }
                })
                .into_iter()
                .collect(),
            // else: an explicit `Any([])`-shaped edge condition can never
            // hold — no dead wiring emitted for an unreachable edge.
            crate::events::TickExecutionPlans::Plans(plans) => {
                // More than one OR-alternative plan means this child could be
                // reached more than once from the same parent invocation.
                // Coalesce via a per-child, per-player guard reset right
                // before evaluating the plans, mirroring the root
                // self-detection guard.
                guarded_children.insert(edge.child.clone());
                let child_key = tick_event_resource_key(&edge.child);
                let guard = format!("se_{child_key}_g");

                let edge_path = format!("__sand_event_edge/{child_key}");
                let edge_ref = format!("{namespace}:{edge_path}");
                records.push(ComponentRecord {
                    namespace: namespace.to_string(),
                    dir: "function".to_string(),
                    path: edge_path,
                    ext: "mcfunction".to_string(),
                    content_type: "text".to_string(),
                    content: [
                        format!("scoreboard players set @s {guard} 1"),
                        format!("function {dispatch_ref}"),
                    ]
                    .join("\n"),
                });

                let mut lines = vec![format!("scoreboard players set @s {guard} 0")];
                for plan in &plans {
                    let mut clauses = vec![format!("unless score @s {guard} matches 1")];
                    clauses.extend(plan.iter().cloned());
                    lines.push(format!(
                        "execute {} run function {edge_ref}",
                        clauses.join(" ")
                    ));
                }
                lines
            }
        }
    };

    if !has_lifecycle {
        // No ordering concern — the condition test (and any multi-plan
        // guard) can be emitted directly into the parent's own command list,
        // exactly as before.
        return conditional_dispatch_lines(&child_ref, guarded_children, records).join("\n");
    }

    // The child owns pre_observation/post_observation: wrap the condition
    // test in a dedicated observe function so post_observation is always
    // structurally reached after a condition attempt, matched or not — never
    // only inside a function reached solely on success.
    let child_key = tick_event_resource_key(&edge.child);
    let mut observe_cmds: Vec<String> = Vec::new();
    observe_cmds.extend(child_node.setup.pre_observation.iter().cloned());
    observe_cmds.extend(conditional_dispatch_lines(
        &child_ref,
        guarded_children,
        records,
    ));
    observe_cmds.extend(child_node.setup.post_observation.iter().cloned());

    let observe_path = format!("__sand_event_observe/{child_key}");
    records.push(ComponentRecord {
        namespace: namespace.to_string(),
        dir: "function".to_string(),
        path: observe_path.clone(),
        ext: "mcfunction".to_string(),
        content_type: "text".to_string(),
        content: observe_cmds.join("\n"),
    });
    format!("function {namespace}:{observe_path}")
}

/// The resolved dispatch backend for a custom [`crate::events::SandEvent`],
/// after enforcing that exactly one of `make_trigger()` / `make_condition()` /
/// `make_tick()` returned `Some` (see #121).
#[allow(clippy::large_enum_variant)]
enum CustomDispatchBackend {
    Advancement(crate::AdvancementTrigger),
    /// Legacy single-fragment `SandEventDispatch::TickCondition` string.
    ///
    /// Normalized into the same structured `TickEventDispatch` shape as
    /// [`TickLifecycle`](Self::TickLifecycle) (matching
    /// `SandEventDispatch::normalize()`) and fed into the same event graph
    /// discovery, so a legacy parent shares exactly one generated detector
    /// with any structured-`tick()` sibling handlers and same-cycle chain
    /// children — never a second, independent detector.
    TickPoll(String),
    /// Structured, typed tick dispatch with lifecycle/setup support.
    TickLifecycle(crate::events::TickEventDispatch),
    /// Structured, same-cycle chained dispatch (#240).
    Chain(crate::events::ChainEventDispatch),
}

/// Resolve which dispatch backend a custom `SandEvent` uses, enforcing the
/// documented `EventDispatch::Custom` contract: exactly one of `make_trigger()`
/// / `make_condition()` / `make_tick()` / `make_chain()` must return `Some`.
///
/// All four factories are evaluated by the caller *before* this function
/// runs, so this is a pure decision function — panicking here (rather than
/// returning a `Result`) matches the existing "both `None`" precedent: this is
/// a Rust-level authoring bug in the `SandEvent` impl, detected at
/// export/codegen time, not a runtime datapack-validity issue.
fn resolve_custom_dispatch_backend(
    trigger: Option<crate::AdvancementTrigger>,
    condition: Option<String>,
    tick: Option<crate::events::TickEventDispatch>,
    chain: Option<crate::events::ChainEventDispatch>,
    handler_path: &str,
) -> CustomDispatchBackend {
    let some_count = [
        trigger.is_some(),
        condition.is_some(),
        tick.is_some(),
        chain.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count();
    match (trigger, condition, tick, chain, some_count) {
        (Some(trigger), None, None, None, 1) => CustomDispatchBackend::Advancement(trigger),
        (None, Some(condition), None, None, 1) => CustomDispatchBackend::TickPoll(condition),
        (None, None, Some(tick), None, 1) => CustomDispatchBackend::TickLifecycle(tick),
        (None, None, None, Some(chain), 1) => CustomDispatchBackend::Chain(chain),
        (_, _, _, _, 0) => {
            panic!(
                "Custom SandEvent for handler `{handler_path}` returned None from \
                 make_trigger(), make_condition(), make_tick(), and make_chain() — implement \
                 exactly one dispatch strategy from SandEvent::dispatch()"
            );
        }
        _ => {
            panic!(
                "Custom SandEvent for handler `{handler_path}` returned more than one dispatch \
                 strategy (make_trigger/make_condition/make_tick/make_chain) — implement exactly \
                 one"
            );
        }
    }
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
            // A fallback profile is deliberately not an exact compatibility
            // claim. Known vanilla triggers therefore require an exact profile
            // even when their historical range appears to include the requested
            // (unknown/future) version. Explicit custom/modded triggers remain
            // the raw escape hatch below.
            if ctx.is_fallback {
                return Err(sand_components::error::SandError::VersionGating {
                    location: advancement_id.to_string(),
                    kind: format!("trigger `{trigger_id}`"),
                    requested_version: ctx.requested_version.to_string(),
                    is_fallback: true,
                    feature_name: "known trigger coverage (exact profile required)".to_string(),
                    fallback_note: " (fallback profile: select an exact known version or `mc_version = \"latest\"` to export known vanilla triggers)".to_string(),
                });
            }
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
        } else if !matches!(trigger, crate::AdvancementTrigger::Custom { .. }) {
            // Typed triggers are Sand-owned and must have coverage metadata;
            // accepting one without it would silently claim compatibility.
            return Err(sand_components::error::SandError::VersionGating {
                location: advancement_id.to_string(),
                kind: format!("trigger `{trigger_id}`"),
                requested_version: ctx.requested_version.to_string(),
                is_fallback: ctx.is_fallback,
                feature_name: "trigger coverage metadata".to_string(),
                fallback_note: " (missing metadata is rejected conservatively; use AdvancementTrigger::Custom for intentional raw/modded triggers)".to_string(),
            });
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
    use crate::events::{PlayerSwimmingEvent, SandEvent};
    use crate::{
        Advancement, AdvancementRewards, AdvancementTrigger, Criterion, DatapackComponent,
        ResourceLocation,
    };

    inventory::submit! {
        crate::function::FunctionTagDescriptor {
            tag: "minecraft:load",
            function_path: "__test_user_load_after_setup",
        }
    }

    #[test]
    fn function_validation_fails_before_records_are_accepted_with_owner_context() {
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "invalid_selector".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: "say valid\nkill @e[limit=-1]".to_string(),
        }];
        let profile = sand_commands::CommandProfile::new("1.21.11", false);
        let error = super::validate_function_records(&mut records, &profile)
            .expect_err("malformed typed output must fail before export")
            .to_string();
        assert!(error.contains("audit:invalid_selector"), "{error}");
        assert!(error.contains("commands[1].limit"), "{error}");
        assert!(error.contains("1.21.11"), "{error}");
        assert_eq!(records[0].content, "say valid\nkill @e[limit=-1]");
    }

    #[test]
    fn explicit_raw_function_line_preserves_unmodelled_syntax() {
        let mut records = vec![super::ComponentRecord {
            namespace: "audit".to_string(),
            dir: "function".to_string(),
            path: "raw".to_string(),
            ext: "mcfunction".to_string(),
            content_type: "text".to_string(),
            content: "modded command syntax".to_string(),
        }];
        super::validate_function_records(
            &mut records,
            &sand_commands::CommandProfile::unprofiled(),
        )
        .unwrap();
        assert_eq!(records[0].content, "modded command syntax");
    }

    #[test]
    fn xp_score_operations_are_lowered_per_player() {
        let commands = super::xp_score_commands();
        assert!(
            commands
                .iter()
                .all(|command| !command.contains("operation @a"))
        );
        assert!(commands.contains(&"execute as @a run scoreboard players operation @s __sand_xp_delta = @s __sand_xp_lvl".to_string()));
        assert!(commands.contains(&"execute as @a run scoreboard players operation @s __sand_xp_delta -= @s __sand_xp_prev".to_string()));
        assert_eq!(
            super::xp_advance_command(),
            "execute as @a run scoreboard players operation @s __sand_xp_prev = @s __sand_xp_lvl"
        );
    }

    #[test]
    fn player_state_events_use_predicate_flags() {
        let dispatch: crate::events::SandEventDispatch = PlayerSwimmingEvent::dispatch();
        let condition = match dispatch {
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

    // ── Custom SandEvent dispatch backend validation (#121) ────────────────────

    #[test]
    fn custom_dispatch_backend_accepts_trigger_only() {
        let backend = super::resolve_custom_dispatch_backend(
            Some(AdvancementTrigger::Tick),
            None,
            None,
            None,
            "my_pack:on_thing",
        );
        assert!(matches!(
            backend,
            super::CustomDispatchBackend::Advancement(_)
        ));
    }

    #[test]
    fn custom_dispatch_backend_accepts_condition_only() {
        let backend = super::resolve_custom_dispatch_backend(
            None,
            Some("score @s foo matches 1..".to_string()),
            None,
            None,
            "my_pack:on_thing",
        );
        assert!(matches!(backend, super::CustomDispatchBackend::TickPoll(_)));
    }

    #[test]
    fn custom_dispatch_backend_accepts_tick_only() {
        let backend = super::resolve_custom_dispatch_backend(
            None,
            None,
            Some(crate::events::TickEventDispatch::default()),
            None,
            "my_pack:on_thing",
        );
        assert!(matches!(
            backend,
            super::CustomDispatchBackend::TickLifecycle(_)
        ));
    }

    #[test]
    #[should_panic(
        expected = "returned None from make_trigger(), make_condition(), make_tick(), and make_chain()"
    )]
    fn custom_dispatch_backend_rejects_neither_backend() {
        super::resolve_custom_dispatch_backend(None, None, None, None, "my_pack:on_thing");
    }

    #[test]
    #[should_panic(expected = "returned more than one dispatch strategy")]
    fn custom_dispatch_backend_rejects_both_backends() {
        super::resolve_custom_dispatch_backend(
            Some(AdvancementTrigger::Tick),
            Some("score @s foo matches 1..".to_string()),
            None,
            None,
            "my_pack:on_thing",
        );
    }

    #[test]
    fn custom_dispatch_backend_both_backends_panic_names_handler_path() {
        let result = std::panic::catch_unwind(|| {
            super::resolve_custom_dispatch_backend(
                Some(AdvancementTrigger::Tick),
                Some("score @s foo matches 1..".to_string()),
                None,
                None,
                "my_pack:on_elevator_placed",
            )
        });
        let err = match result {
            Ok(_) => panic!("expected panic, got Ok"),
            Err(err) => err,
        };
        let message = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(message.contains("my_pack:on_elevator_placed"), "{message}");
        assert!(
            message.contains("more than one dispatch strategy"),
            "{message}"
        );
        assert!(message.contains("exactly one"), "{message}");
    }

    #[test]
    fn invalid_advancement_fails_at_component_record_boundary_with_owner_context() {
        let advancement = Advancement::new(ResourceLocation::new("test", "invalid").unwrap());
        let error = super::component_to_record(&advancement, None).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("test:invalid"), "{message}");
        assert!(message.contains("(advancement)"), "{message}");
        assert!(message.contains("field: criteria"), "{message}");
    }

    #[test]
    fn generated_event_advancement_json_remains_unchanged_through_fallible_export() {
        let advancement = Advancement::new(ResourceLocation::new("test", "event").unwrap())
            .criterion("event", Criterion::new(AdvancementTrigger::Tick))
            .rewards(AdvancementRewards::new().function("test:event"));

        let legacy = serde_json::to_string_pretty(&advancement.to_json()).unwrap();
        let record = super::component_to_record(&advancement, None).unwrap();
        assert_eq!(record.content, legacy);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&record.content).unwrap(),
            json!({
                "criteria": {"event": {"trigger": "minecraft:tick"}},
                "requirements": [["event"]],
                "rewards": {"function": "test:event"}
            })
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
    fn event_trigger_gating_rejects_unsupported_and_fallback_profiles() {
        let caps = VersionCaps::all_enabled();
        let old_ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.18.2",
            is_fallback: false,
        };
        let unsupported = super::check_event_trigger(
            &AdvancementTrigger::AllayDropItemOnBlock {
                item: None,
                location: None,
            },
            "test:allay_event",
            "allay_event",
            Some(&old_ctx),
        )
        .expect_err("allay trigger was introduced after 1.18.2");
        assert!(
            unsupported
                .to_string()
                .contains("minecraft:allay_drop_item_on_block")
        );
        assert!(unsupported.to_string().contains("1.18.2"));

        let fallback_ctx = ExportCtx {
            caps: &caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        let fallback = super::check_event_trigger(
            &AdvancementTrigger::Tick,
            "test:fallback_event",
            "fallback_event",
            Some(&fallback_ctx),
        )
        .expect_err("fallback profiles must not claim exact trigger support");
        assert!(fallback.to_string().contains("fallback"));
        assert!(fallback.to_string().contains("minecraft:tick"));
    }

    #[test]
    fn event_trigger_gating_accepts_supported_and_custom_triggers() {
        let caps = VersionCaps::all_enabled();
        let exact_ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.19",
            is_fallback: false,
        };
        super::check_event_trigger(
            &AdvancementTrigger::AllayDropItemOnBlock {
                item: None,
                location: None,
            },
            "test:allay_event",
            "allay_event",
            Some(&exact_ctx),
        )
        .expect("allay trigger should be valid in 1.19");

        let fallback_ctx = ExportCtx {
            caps: &caps,
            requested_version: "999.0",
            is_fallback: true,
        };
        super::check_event_trigger(
            &AdvancementTrigger::Custom {
                trigger: "examplemod:custom_trigger".to_string(),
                conditions: None,
            },
            "test:custom_event",
            "custom_event",
            Some(&fallback_ctx),
        )
        .expect("explicit custom triggers remain a raw/modded escape hatch");
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
    fn user_function_tag_entries_sort_deterministically() {
        let mut entries = vec![
            ("minecraft:tick".to_string(), "pack:z".to_string()),
            ("minecraft:load".to_string(), "pack:m".to_string()),
            ("minecraft:load".to_string(), "pack:a".to_string()),
        ];
        super::sort_function_tag_entries(&mut entries);
        assert_eq!(
            entries,
            vec![
                ("minecraft:load".to_string(), "pack:a".to_string()),
                ("minecraft:load".to_string(), "pack:m".to_string()),
                ("minecraft:tick".to_string(), "pack:z".to_string()),
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
        crate::state::register_load_objective("tag_order_life", "dummy");

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

    #[test]
    fn component_to_record_rejects_empty_item_modifier_with_owner_context() {
        let modifier = sand_components::ItemModifier::new(test_rl("test", "empty_modifier"));
        let error = component_to_record(&modifier, None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("test:empty_modifier"));
        assert!(error.contains("item_modifier"));
        assert!(error.contains("functions"));
    }

    #[test]
    fn component_to_record_retains_nested_item_modifier_function_path() {
        let modifier = sand_components::ItemModifier::new(test_rl("test", "bad_modifier"))
            .function(sand_components::LootFunction::SetDamage {
                damage: sand_components::NumberProvider::Uniform {
                    min: 0.0,
                    max: f64::INFINITY,
                },
                add: false,
            });
        let error = component_to_record(&modifier, None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("test:bad_modifier"));
        assert!(error.contains("functions[0].damage.max"));
        assert!(error.contains("finite"));
    }

    #[test]
    fn component_to_record_preserves_item_modifier_root_shapes() {
        let single = sand_components::ItemModifier::new(test_rl("test", "single_modifier"))
            .function(sand_components::LootFunction::ExplosionDecay);
        let single_record = component_to_record(&single, None).unwrap();
        assert_eq!(single_record.namespace, "test");
        assert_eq!(single_record.dir, "item_modifier");
        assert_eq!(single_record.path, "single_modifier");
        assert_eq!(single_record.ext, "json");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&single_record.content).unwrap(),
            serde_json::json!({"function": "minecraft:explosion_decay"})
        );

        let multiple = sand_components::ItemModifier::new(test_rl("test", "multi_modifier"))
            .function(sand_components::LootFunction::ExplosionDecay)
            .function(sand_components::LootFunction::FurnaceSmelt);
        let multiple_record = component_to_record(&multiple, None).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&multiple_record.content).unwrap(),
            serde_json::json!([
                {"function": "minecraft:explosion_decay"},
                {"function": "minecraft:furnace_smelt"}
            ])
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
        let resolved = crate::version::resolve_export_caps("latest").unwrap();
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
        let resolved = crate::version::resolve_export_caps("999.0").unwrap();
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
        let resolved = crate::version::resolve_export_caps("1.19.4").unwrap();
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

    #[test]
    fn resolve_export_caps_rejects_malformed_version() {
        let err = crate::version::resolve_export_caps("not-a-version")
            .expect_err("malformed export version must not silently use a fallback");
        assert!(err.to_string().contains("not-a-version"));
    }

    // ── Component-bearing recipe result version gating (#226) ─────────────────

    fn elevator_recipe(
        loc: crate::resource_location::ResourceLocation,
    ) -> sand_components::recipe::ShapedRecipe {
        let elevator = sand_components::CustomItem::new("minecraft:white_wool")
            .custom_data("elevator_block_item")
            .component(sand_components::ItemComponent::EnchantmentGlintOverride(
                true,
            ));
        let result = sand_components::recipe::RecipeResult::custom_item(&elevator)
            .expect("component-bearing custom item should convert to a recipe result");
        sand_components::recipe::ShapedRecipe::new(loc)
            .pattern(["X"])
            .key(
                'X',
                sand_components::recipe::Ingredient::item("minecraft:white_wool"),
            )
            .result(result)
    }

    #[test]
    fn component_bearing_recipe_result_rejected_when_item_components_unsupported() {
        let recipe = elevator_recipe(test_rl("test", "elevator_gated"));
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.19.4",
            is_fallback: false,
        };
        let err = component_to_record(&recipe, Some(&ctx))
            .expect_err("component-bearing recipe result must be gated on item_components");
        let msg = err.to_string();
        assert!(msg.contains("item_components"), "err: {msg}");
        assert!(msg.contains("1.19.4"), "err: {msg}");
    }

    #[test]
    fn component_bearing_recipe_result_accepted_when_item_components_supported() {
        let recipe = elevator_recipe(test_rl("test", "elevator_ok"));
        let caps = VersionCaps::all_enabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.21.4",
            is_fallback: false,
        };
        let record = component_to_record(&recipe, Some(&ctx))
            .expect("component-bearing recipe result should succeed when supported");
        assert_eq!(record.dir, "recipe");
        assert!(record.content.contains("elevator_block_item"));
    }

    #[test]
    fn component_free_recipe_result_never_gated() {
        let recipe = sand_components::recipe::ShapedRecipe::new(test_rl("test", "plain_recipe"))
            .pattern(["X"])
            .key(
                'X',
                sand_components::recipe::Ingredient::item("minecraft:stick"),
            )
            .result(sand_components::recipe::RecipeResult::raw(
                "minecraft:diamond",
                1,
            ));
        let caps = VersionCaps::all_disabled();
        let ctx = ExportCtx {
            caps: &caps,
            requested_version: "1.18.1",
            is_fallback: false,
        };
        component_to_record(&recipe, Some(&ctx))
            .expect("component-free recipe results must never be version-gated");
    }
}
