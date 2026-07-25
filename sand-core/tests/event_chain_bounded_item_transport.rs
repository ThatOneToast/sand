//! Export coverage for `EventParticipantPlan::inherit_item_within` — bounded,
//! per-subject item-snapshot transport across a `.within(...)` correlation
//! window (#272).
//!
//! Companion to `event_chain_participant_item_inheritance.rs` (the
//! same-cycle `inherit_item` case, #264) and `event_chain_within_export.rs`
//! (the bounded age-counter machinery this feature reuses unmodified).

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, SandEventParticipants,
    TickEventDispatch, TickWindow,
};
use sand_core::participant::role::ParticipantHand;
use sand_core::participant::{EventParticipantPlan, ItemParticipantRole};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn revoke_true() -> bool {
    true
}

macro_rules! submit_handler {
    ($event:ty, $path:literal, $body:literal) => {
        const _: () = {
            fn chain() -> Option<ChainEventDispatch> {
                match <$event as SandEvent>::dispatch().into() {
                    SandEventDispatch::Chain(chain) => Some(chain),
                    _ => None,
                }
            }
            fn tick() -> Option<TickEventDispatch> {
                match <$event as SandEvent>::dispatch().into() {
                    SandEventDispatch::Tick(tick) => Some(tick),
                    _ => None,
                }
            }
            fn type_id() -> TypeId {
                TypeId::of::<$event>()
            }
            fn type_name() -> &'static str {
                std::any::type_name::<$event>()
            }
            fn body() -> Vec<String> {
                vec![$body.to_string()]
            }
            fn setup() -> EventSetup {
                <$event as SandEvent>::setup()
            }
            fn participants() -> EventParticipantPlan {
                <$event as SandEvent>::participants()
            }
            sand_core::inventory::submit! {
                EventDescriptor {
                    path: $path,
                    id_override: None,
                    make: body,
                    dispatch: EventDispatch::Custom {
                        make_trigger: no_trigger,
                        make_condition: no_condition,
                        make_tick: tick,
                        make_chain: chain,
                        make_tracked: || None,
                        make_participants: participants,
                        revoke: revoke_true,
                        event_type_id: type_id,
                        event_type_name: type_name,
                        make_setup: setup,
                    },
                }
            }
        };
    };
}

// ── Fixture: a root that captures its own weapon, and two bounded children
//    (distinct windows) that inherit it across the window ──────────────────

struct Source;
impl SandEvent for Source {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p272_source matches 1.."))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_weapon()
    }
}
submit_handler!(Source, "on_source", "say source fired");

const WINDOW_TICKS: u32 = 40;

struct BoundedChild;
impl SandEvent for BoundedChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<Source>()
            .within::<Source>(TickWindow::new(WINDOW_TICKS).unwrap())
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_item_within::<Source>(
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
            TickWindow::new(WINDOW_TICKS).unwrap(),
        )
    }
}
submit_handler!(BoundedChild, "on_bounded_child", "say bounded_child fired");

const LONG_WINDOW_TICKS: u32 = 100;

struct LongBoundedChild;
impl SandEvent for LongBoundedChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<Source>()
            .within::<Source>(TickWindow::new(LONG_WINDOW_TICKS).unwrap())
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_item_within::<Source>(
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
            TickWindow::new(LONG_WINDOW_TICKS).unwrap(),
        )
    }
}
submit_handler!(
    LongBoundedChild,
    "on_long_bounded_child",
    "say long_bounded_child fired"
);

fn key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("boundeditempack").expect("export succeeds");
    serde_json::from_str(&json).expect("valid export JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

fn all_content(records: &[serde_json::Value]) -> String {
    records
        .iter()
        .filter(|r| r["dir"] == "function")
        .filter_map(|r| r["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn entry_key() -> String {
    key(&format!(
        "{}::bounded_item::Weapon::MainHand",
        std::any::type_name::<Source>()
    ))
}

// ── Read-side resolution ────────────────────────────────────────────────────

#[test]
fn bounded_child_resolves_to_staged_command_storage_distinct_from_sources_own_snapshot() {
    let source_snapshot = Source.weapon();
    let bounded_snapshot = BoundedChild.bounded_item(ItemParticipantRole::Weapon);

    assert_ne!(
        source_snapshot.item_path().as_str(),
        bounded_snapshot.item_path().as_str(),
        "the bounded copy must not resolve to the source's own transient snapshot path"
    );
    assert_eq!(
        bounded_snapshot.item_path().as_str(),
        format!("cur.{}.item", entry_key())
    );
}

#[test]
fn read_accessors_are_resolvable_outside_a_macro_line() {
    // The durable per-subject path contains `$(subject)` and is only legal
    // inside a macro line. Handler-composed commands are not macro lines, so
    // anything a handler can reach must be free of macro syntax — that is
    // the entire reason the load step stages into a static scratch path.
    let bounded = BoundedChild.bounded_item(ItemParticipantRole::Weapon);
    for path in [
        bounded.item_path(),
        bounded.id_path(),
        bounded.count_path(),
        bounded.components_path(),
    ] {
        assert!(!path.as_str().contains("$("), "{}", path.as_str());
    }
    assert!(!format!("{:?}", bounded.is_present()).contains("$("));
}

#[test]
fn two_bounded_children_with_different_windows_resolve_to_the_same_shared_storage() {
    // Same (source, role, hand) triple -> same storage, regardless of each
    // child's own declared window — the storage itself is not per-child.
    let short = BoundedChild.bounded_item(ItemParticipantRole::Weapon);
    let long = LongBoundedChild.bounded_item(ItemParticipantRole::Weapon);
    assert_eq!(short.item_path().as_str(), long.item_path().as_str());
}

// ── The regression this whole backend exists to prevent ────────────────────

#[test]
fn nothing_this_feature_generates_writes_custom_entity_nbt() {
    // A live Minecraft 26.2 RCON round-trip proved vanilla silently drops
    // writes to arbitrary custom top-level entity NBT keys, which is what an
    // earlier revision of this feature used. If this assertion ever fails
    // again, the feature is non-functional on a real server regardless of
    // what every other structural test says. See
    // `sand-core/src/participant/bounded_item.rs`'s module doc for the
    // transcript and `scripts/mc_validation/run_bounded_item_proof.py` for
    // the replacement backend's live proof.
    let records = records();
    for record in records.iter().filter(|r| r["dir"] == "function") {
        for line in record["content"].as_str().unwrap_or_default().lines() {
            // Only `data` commands can carry an NBT target; `function`/
            // `scoreboard` lines merely *name* the generated resources.
            if !line.contains("data modify") && !line.contains("data get") {
                continue;
            }
            if line.contains("bounded_item") {
                assert!(
                    !line.contains("entity @"),
                    "bounded item storage must never touch entity NBT: {line}"
                );
                assert!(
                    line.contains("storage sand:__bounded_item"),
                    "every bounded-item data access must target command storage: {line}"
                );
            }
        }
    }
}

// ── Persist (source side) ──────────────────────────────────────────────────

#[test]
fn source_dispatch_binds_the_subject_slot_then_persists_right_after_its_occurrence_mark() {
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let dispatch = function(&records, &format!("__sand_event_dispatch/{source_key}"));

    assert_eq!(
        dispatch,
        format!(
            "scoreboard players set @s se_{source_key}_o 1\n\
             execute unless score @s sand_subj matches -2147483648.. run function \
             boundeditempack:__sand_bounded_item_slot_alloc\n\
             execute store result storage sand:__bounded_item args.subject int 1 run scoreboard \
             players get @s sand_subj\n\
             function boundeditempack:__sand_event_bounded_item_persist/{} with storage \
             sand:__bounded_item args\n\
             function boundeditempack:on_source",
            entry_key()
        ),
        "persist must be bound and invoked after the occurrence mark and before the handler"
    );
}

#[test]
fn persist_body_replaces_atomically_and_gates_on_source_presence() {
    let records = records();
    let body = function(
        &records,
        &format!("__sand_event_bounded_item_persist/{}", entry_key()),
    );
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines.len(), 4, "{body}");
    // Every line is a macro line; the destination path is per-subject.
    for line in &lines {
        assert!(
            line.starts_with("$data modify") || line.starts_with("$execute"),
            "{line}"
        );
        assert!(line.contains("p$(subject)."), "{line}");
    }
    // Unconditional reset first => repeated occurrences replace atomically,
    // and an occurrence with no item leaves explicit absence, not stale data.
    assert!(lines[0].ends_with(".present set value 0b"));
    assert!(lines[1].ends_with(".item set value {}"));
    // Then a presence-gated copy and mark, both under the *same* guard.
    assert!(lines[2].starts_with("$execute if data storage sand:__participants"));
    assert!(lines[3].starts_with("$execute if data storage sand:__participants"));
    assert!(lines[3].ends_with(".present set value 1b"));
}

#[test]
fn persist_is_generated_once_per_source_role_hand_triple() {
    // Two children (different windows) both inherit the same
    // (Source, Weapon, MainHand) triple — one persist function, invoked once.
    let records = records();
    let persist_fns = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .filter(|r| {
            r["path"]
                .as_str()
                .is_some_and(|p| p.starts_with("__sand_event_bounded_item_persist/"))
        })
        .count();
    assert_eq!(persist_fns, 1, "one persist function per triple");

    let source_key = key(std::any::type_name::<Source>());
    let dispatch = function(&records, &format!("__sand_event_dispatch/{source_key}"));
    assert_eq!(
        dispatch
            .matches("__sand_event_bounded_item_persist/")
            .count(),
        1,
        "persist must be invoked once, not once per consumer: {dispatch}"
    );
}

// ── Load (consumer side) ───────────────────────────────────────────────────

#[test]
fn each_consumer_stages_its_own_subjects_snapshot_before_its_handler() {
    let records = records();
    for (child, handler) in [
        (std::any::type_name::<BoundedChild>(), "on_bounded_child"),
        (
            std::any::type_name::<LongBoundedChild>(),
            "on_long_bounded_child",
        ),
    ] {
        let dispatch = function(&records, &format!("__sand_event_dispatch/{}", key(child)));
        assert_eq!(
            dispatch,
            format!(
                "data modify storage sand:__bounded_item args.subject set value 0\n\
                 execute if score @s sand_subj matches -2147483648.. store result storage \
                 sand:__bounded_item args.subject int 1 run scoreboard players get @s sand_subj\n\
                 function boundeditempack:__sand_event_bounded_item_load/{} with storage \
                 sand:__bounded_item args\n\
                 function boundeditempack:{handler}",
                entry_key()
            ),
            "consumer {child} must stage its own subject's snapshot before its handler"
        );
    }
}

#[test]
fn an_unslotted_subject_cannot_read_another_subjects_snapshot() {
    // This is the one place cross-subject leakage could be reintroduced: a
    // player who has never sourced a bounded occurrence has no slot, so
    // `scoreboard players get` fails and `execute store result` leaves
    // `args.subject` *unchanged* — i.e. holding whichever subject ran last.
    // The unconditional reset to the never-allocated sentinel slot `0` must
    // therefore come first, and must not itself be conditional.
    let records = records();
    let dispatch = function(
        &records,
        &format!(
            "__sand_event_dispatch/{}",
            key(std::any::type_name::<BoundedChild>())
        ),
    );
    let first = dispatch.lines().next().unwrap();
    assert_eq!(
        first, "data modify storage sand:__bounded_item args.subject set value 0",
        "the subject slot must be unconditionally cleared before any conditional store"
    );
    assert!(
        !first.starts_with("execute"),
        "the clear must be unconditional: {first}"
    );
}

#[test]
fn load_body_resets_the_staged_path_then_copies_from_the_durable_per_subject_one() {
    let records = records();
    let key = entry_key();
    let body = function(&records, &format!("__sand_event_bounded_item_load/{key}"));
    assert_eq!(
        body,
        format!(
            "data modify storage sand:__bounded_item cur.{key}.present set value 0b\n\
             data modify storage sand:__bounded_item cur.{key}.item set value {{}}\n\
             $execute if data storage sand:__bounded_item p$(subject).{key}{{present:1b}} run data \
             modify storage sand:__bounded_item cur.{key}.item set from storage \
             sand:__bounded_item p$(subject).{key}.item\n\
             $execute if data storage sand:__bounded_item p$(subject).{key}{{present:1b}} run data \
             modify storage sand:__bounded_item cur.{key}.present set value 1b"
        )
    );
}

#[test]
fn staging_resets_first_so_one_consumer_never_observes_anothers_staged_value() {
    // Multiple consumers run in one cycle against one shared scratch path.
    // The leading unconditional reset is what keeps that safe: a consumer
    // whose own subject has nothing persisted observes explicit absence
    // rather than the previous consumer's staged item.
    let records = records();
    let body = function(
        &records,
        &format!("__sand_event_bounded_item_load/{}", entry_key()),
    );
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines[0].contains("cur.") && lines[0].ends_with("set value 0b"));
    assert!(lines[1].contains("cur.") && lines[1].ends_with("set value {}"));
    for line in &lines[..2] {
        assert!(
            !line.contains("execute if"),
            "resets must be unconditional: {line}"
        );
    }
}

#[test]
fn no_consumer_eagerly_clears_the_durable_copy_after_dispatch() {
    // Deliberate design decision: only expiry and replacement ever clear the
    // durable copy. An eager per-consumer clear would non-deterministically
    // starve whichever sibling consumer's staged evaluation happens to run
    // later in the same tick.
    let records = records();
    for child in [
        std::any::type_name::<BoundedChild>(),
        std::any::type_name::<LongBoundedChild>(),
    ] {
        let dispatch = function(&records, &format!("__sand_event_dispatch/{}", key(child)));
        assert!(
            !dispatch.contains("p$(subject)"),
            "a consumer must never write the durable per-subject copy: {dispatch}"
        );
        assert!(
            !dispatch.contains("_expire"),
            "a consumer must never invoke expiry: {dispatch}"
        );
    }
}

// ── Subject-slot allocation ────────────────────────────────────────────────

#[test]
fn subject_slots_are_allocated_lazily_from_a_shared_counter() {
    let records = records();
    assert_eq!(
        function(&records, "__sand_bounded_item_slot_alloc"),
        "scoreboard players add #sand_subj_next sand_subj 1\n\
         scoreboard players operation @s sand_subj = #sand_subj_next sand_subj"
    );
    // The counter is incremented before it is copied, so the first slot
    // handed out is 1 and slot 0 is never a real subject — which is what
    // makes 0 usable as the "no subject" sentinel on the read path.
    assert_eq!(
        function(&records, "__sand_bounded_item_setup"),
        "scoreboard objectives add sand_subj dummy"
    );
}

#[test]
fn the_slot_objective_is_registered_on_load() {
    let records = records();
    let load_tag = records
        .iter()
        .find(|r| r["dir"] == "tags/function" && r["path"] == "load")
        .or_else(|| {
            records.iter().find(|r| {
                r["content"]
                    .as_str()
                    .is_some_and(|c| c.contains("__sand_bounded_item_setup"))
            })
        })
        .expect("the slot objective setup is wired into a load hook");
    assert!(
        load_tag["content"]
            .as_str()
            .unwrap_or_default()
            .contains("__sand_bounded_item_setup")
    );
}

// ── Expiry ───────────────────────────────────────────────────────────────

#[test]
fn expiry_is_keyed_to_the_longest_declared_window_across_all_consumers() {
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let cycle = function(&records, "__sand_event_cycle");

    // Expiry must run at the *longer* of the two declared windows
    // (LongBoundedChild's 100, not BoundedChild's 40) — the storage stays
    // valid for as long as the longest-lived consumer needs it.
    let expire_guard = format!(
        "execute as @a if score @s sand_subj matches -2147483648.. if score @s se_{source_key}_wa matches {LONG_WINDOW_TICKS} run function boundeditempack:__sand_event_bounded_item_expire/"
    );
    assert!(
        cycle.contains(&expire_guard),
        "expiry guard must use the longest declared window and skip unslotted subjects: {cycle}"
    );
    let short_window_guard =
        format!("if score @s se_{source_key}_wa matches {WINDOW_TICKS} run function");
    assert!(
        !cycle.contains(&short_window_guard),
        "expiry must not fire at the shorter window — that would clear the copy before the long-window consumer's own window elapses: {cycle}"
    );
}

#[test]
fn expiry_skips_subjects_that_never_sourced_an_occurrence() {
    // Without the slot guard, `execute store result ... run scoreboard
    // players get @s sand_subj` on an unset score would leave a stale
    // subject bound and expire *someone else's* storage.
    let records = records();
    let cycle = function(&records, "__sand_event_cycle");
    let expire_line = cycle
        .lines()
        .find(|l| l.contains("__sand_event_bounded_item_expire/"))
        .expect("expiry guard is present");
    assert!(
        expire_line.contains("if score @s sand_subj matches -2147483648.."),
        "{expire_line}"
    );
}

#[test]
fn expiry_clears_the_durable_copy_and_never_marks_present() {
    let records = records();
    let expire_record = records
        .iter()
        .find(|r| {
            r["dir"] == "function"
                && r["path"]
                    .as_str()
                    .is_some_and(|p| p.starts_with("__sand_event_bounded_item_expire_m/"))
        })
        .expect("an expire macro function is generated");
    let content = expire_record["content"].as_str().unwrap();
    let key = entry_key();
    assert_eq!(
        content,
        format!(
            "$data modify storage sand:__bounded_item p$(subject).{key}.present set value 0b\n\
             $data modify storage sand:__bounded_item p$(subject).{key}.item set value {{}}"
        )
    );
    assert!(
        !content.contains("1b"),
        "expiry must never mark present: {content}"
    );
    assert!(
        !content.contains("cur."),
        "expiry clears the durable copy, not one call tree's staged view: {content}"
    );
}

#[test]
fn expiry_binds_the_subject_before_invoking_its_macro_body() {
    let records = records();
    let caller = records
        .iter()
        .find(|r| {
            r["dir"] == "function"
                && r["path"]
                    .as_str()
                    .is_some_and(|p| p.starts_with("__sand_event_bounded_item_expire/"))
        })
        .expect("an expire caller is generated");
    let content = caller["content"].as_str().unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "{content}");
    assert_eq!(
        lines[0],
        "execute store result storage sand:__bounded_item args.subject int 1 run scoreboard \
         players get @s sand_subj"
    );
    assert!(lines[1].starts_with("function boundeditempack:__sand_event_bounded_item_expire_m/"));
    assert!(lines[1].ends_with("with storage sand:__bounded_item args"));
}

#[test]
fn expiry_guard_runs_after_the_age_increment_line_for_the_same_source() {
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let cycle = function(&records, "__sand_event_cycle");
    let increment_pos = cycle
        .find(&format!(
            "unless score @s se_{source_key}_o matches 1 unless score @s se_{source_key}_wa matches"
        ))
        .expect("age increment line is present");
    let expire_pos = cycle
        .find(&format!(
            "if score @s se_{source_key}_wa matches {LONG_WINDOW_TICKS} run function"
        ))
        .expect("expiry guard is present");
    assert!(
        increment_pos < expire_pos,
        "expiry must observe the just-incremented age, not a stale pre-increment value: {cycle}"
    );
}

#[test]
fn expiry_reuses_the_existing_within_age_objective_unmodified() {
    // #272 must not introduce a second age mechanism — the `.within(...)`
    // counter from #240 Phase 5 is the only one.
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let content = all_content(&records);
    assert!(content.contains(&format!(
        "scoreboard objectives add se_{source_key}_wa dummy"
    )));
    for line in content.lines() {
        if line.contains("__sand_event_bounded_item_expire/") {
            assert!(
                line.contains(&format!("se_{source_key}_wa")),
                "expiry must be driven by the existing age objective: {line}"
            );
        }
    }
}

// ── No stale remnants / determinism ─────────────────────────────────────────

#[test]
fn bounded_storage_is_only_touched_by_the_generated_transport_functions() {
    let records = records();
    let bad: Vec<&str> = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .filter(|r| {
            let path = r["path"].as_str().unwrap_or_default();
            !path.starts_with("__sand_event_dispatch/")
                && !path.starts_with("__sand_event_bounded_item_")
                && !path.starts_with("__sand_bounded_item_")
                && path != "__sand_event_cycle"
        })
        .filter(|r| {
            r["content"]
                .as_str()
                .is_some_and(|c| c.contains("sand:__bounded_item"))
        })
        .filter_map(|r| r["path"].as_str())
        .collect();
    assert!(
        bad.is_empty(),
        "bounded item storage leaked into unrelated generated functions: {bad:?}"
    );
}

#[test]
fn repeated_export_is_byte_identical() {
    let first = sand_core::try_export_components_json("boundeditempack").unwrap();
    let second = sand_core::try_export_components_json("boundeditempack").unwrap();
    assert_eq!(first, second);
}

#[test]
fn full_generated_bounded_item_wiring_snapshot() {
    // One focused exact-output assertion tying the whole generated shape
    // together, so any future change to this feature's codegen is caught
    // here first.
    let records = records();
    let key = entry_key();
    let source_key = key_of::<Source>();
    let cycle = function(&records, "__sand_event_cycle");
    assert!(cycle.contains(&format!(
        "execute as @a if score @s sand_subj matches -2147483648.. if score @s se_{source_key}_wa \
         matches {LONG_WINDOW_TICKS} run function \
         boundeditempack:__sand_event_bounded_item_expire/"
    )));
    for path in [
        format!("__sand_event_bounded_item_persist/{key}"),
        format!("__sand_event_bounded_item_load/{key}"),
        "__sand_bounded_item_slot_alloc".to_string(),
        "__sand_bounded_item_setup".to_string(),
    ] {
        assert!(
            !function(&records, &path).is_empty(),
            "missing generated function {path}"
        );
    }
}

fn key_of<T: ?Sized + 'static>() -> String {
    key(std::any::type_name::<T>())
}

#[test]
#[ignore]
fn dump_export_to_file_for_external_hash_comparison() {
    // Not part of the normal suite. Serves two purposes, both of which need
    // the export to cross a process boundary:
    //
    //  1. Determinism: invoked twice as two independent OS processes to
    //     prove byte-identical output across genuinely separate export runs,
    //     not just repeated calls within one process's memory state (the
    //     convention established in `event_chain_participant_multiparent.rs`).
    //  2. Live validation: `scripts/mc_validation/run_bounded_item_event_proof.py`
    //     materializes these records into a real datapack and loads them on a
    //     real Minecraft 26.2 server, so the generated event path is exercised
    //     as-shipped rather than re-derived by the harness.
    let json =
        sand_core::try_export_components_json("boundeditempack").expect("export should succeed");
    std::fs::write(
        std::env::var("SAND_DETERMINISM_DUMP_PATH").expect("SAND_DETERMINISM_DUMP_PATH not set"),
        json,
    )
    .expect("write dump file");
}
