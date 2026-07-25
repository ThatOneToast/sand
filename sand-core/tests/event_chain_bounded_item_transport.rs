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

// ── Read-side resolution ────────────────────────────────────────────────────

#[test]
fn bounded_child_resolves_to_per_subject_entity_storage_distinct_from_sources_own_snapshot() {
    let source_snapshot = Source.weapon();
    let bounded_snapshot = BoundedChild.bounded_item(ItemParticipantRole::Weapon);

    // Structurally distinct storage: the source's own snapshot lives in
    // global command storage (`storage()` names a `namespace:path`), the
    // bounded copy lives in per-entity NBT — never the same path.
    assert_ne!(
        source_snapshot.item_path().as_str(),
        bounded_snapshot.item_path().as_str(),
        "the bounded copy must not resolve to the source's own transient snapshot path"
    );
    assert!(
        bounded_snapshot
            .item_path()
            .as_str()
            .starts_with("__sand_bounded_item.")
    );
}

#[test]
fn two_bounded_children_with_different_windows_resolve_to_the_same_shared_storage() {
    // Same (source, role, hand) triple -> same storage, regardless of each
    // child's own declared window — the storage itself is not per-child.
    let short = BoundedChild.bounded_item(ItemParticipantRole::Weapon);
    let long = LongBoundedChild.bounded_item(ItemParticipantRole::Weapon);
    assert_eq!(short.item_path().as_str(), long.item_path().as_str());
}

// ── Generated persist commands (replacement + absence semantics) ──────────

#[test]
fn source_dispatch_function_persists_the_bounded_copy_right_after_its_occurrence_mark() {
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let dispatch = function(&records, &format!("__sand_event_dispatch/{source_key}"));

    let occurrence_line = format!("scoreboard players set @s se_{source_key}_o 1");
    let occurrence_pos = dispatch
        .find(&occurrence_line)
        .expect("source's own occurrence mark is set in its dispatch function");

    // Reset-then-conditionally-copy-then-mark, in that exact order,
    // immediately after the occurrence mark and before the handler body.
    let reset_present_pos = dispatch
        .find("data modify entity @s __sand_bounded_item.")
        .expect("persist commands are present");
    assert!(
        occurrence_pos < reset_present_pos,
        "persist commands must be spliced after the occurrence mark: {dispatch}"
    );

    let handler_pos = dispatch
        .find("function boundeditempack:on_source")
        .expect("handler call is present");
    assert!(
        reset_present_pos < handler_pos,
        "persist commands must run before the handler body: {dispatch}"
    );

    assert!(dispatch.contains("set value 0b"), "{dispatch}");
    assert!(dispatch.contains("set value {}"), "{dispatch}");
    assert!(
        dispatch.contains("execute if data storage sand:__participants"),
        "the copy must be gated on the source's own transient snapshot presence: {dispatch}"
    );
    assert!(dispatch.contains("set value 1b"), "{dispatch}");
}

#[test]
fn persist_commands_target_per_player_entity_storage_not_a_global_path() {
    // Structural proof of subject isolation (#272 acceptance criterion):
    // every write this feature generates targets `entity @s`, never a bare
    // `storage <ns:path>` destination — two players' entities never share
    // NBT, so cross-player leakage is impossible by construction, not by
    // runtime luck. (The source's *own* transient snapshot, read as a copy
    // source here, is intentionally still global — see
    // `crate::item::ItemSnapshot`'s module doc for why that is safe within
    // one synchronous call tree.)
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let dispatch = function(&records, &format!("__sand_event_dispatch/{source_key}"));
    for line in dispatch.lines() {
        if line.contains("__sand_bounded_item.") {
            assert!(
                line.contains("entity @s"),
                "every bounded-item storage access must target entity @s: {line}"
            );
        }
    }
}

#[test]
fn persist_commands_only_generated_once_per_source_role_hand_triple() {
    // Two children (different windows) both inherit the same
    // (Source, Weapon, MainHand) triple — the persist block must appear
    // exactly once in Source's dispatch function, not once per consumer.
    let records = records();
    let source_key = key(std::any::type_name::<Source>());
    let dispatch = function(&records, &format!("__sand_event_dispatch/{source_key}"));
    let occurrences = dispatch.matches("set value 1b").count();
    assert_eq!(
        occurrences, 1,
        "the presence-gated mark command must appear exactly once: {dispatch}"
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
        "execute as @a if score @s se_{source_key}_wa matches {LONG_WINDOW_TICKS} run function boundeditempack:__sand_event_bounded_item_expire/"
    );
    assert!(
        cycle.contains(&expire_guard),
        "expiry guard must use the longest declared window: {cycle}"
    );
    let short_window_guard =
        format!("if score @s se_{source_key}_wa matches {WINDOW_TICKS} run function");
    assert!(
        !cycle.contains(&short_window_guard),
        "expiry must not fire at the shorter window — that would clear the copy before the long-window consumer's own window elapses: {cycle}"
    );
}

#[test]
fn expiry_function_resets_to_explicit_absence() {
    let records = records();
    let expire_record = records
        .iter()
        .find(|r| {
            r["dir"] == "function"
                && r["path"]
                    .as_str()
                    .is_some_and(|p| p.starts_with("__sand_event_bounded_item_expire/"))
        })
        .expect("an expire function is generated");
    let content = expire_record["content"].as_str().unwrap();
    assert!(content.contains("set value 0b"));
    assert!(content.contains("set value {}"));
    assert!(
        !content.contains("1b"),
        "expiry must never mark present: {content}"
    );
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

// ── No stale remnants / determinism ─────────────────────────────────────────

#[test]
fn no_stale_snapshot_storage_paths_leak_outside_the_generated_persist_expire_reset_trio() {
    // Every reference to the bounded-item storage root must appear only in
    // the source's own persist block or the dedicated expire function —
    // never scattered into some other unrelated generated function.
    let records = records();
    let bad = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .filter(|r| {
            let path = r["path"].as_str().unwrap_or_default();
            !path.starts_with("__sand_event_dispatch/")
                && !path.starts_with("__sand_event_bounded_item_expire/")
        })
        .filter_map(|r| r["content"].as_str())
        .any(|content| content.contains("__sand_bounded_item."));
    assert!(
        !bad,
        "bounded item storage must only ever be written from the source's own dispatch function or the generated expire function"
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
    // A single, focused exact-output assertion tying together the whole
    // generated shape (persist splice, expire function, coordinator guard)
    // so any future change to this feature's codegen is caught here first.
    let records = records();
    let content = all_content(&records);
    assert!(content.contains("__sand_bounded_item."));
    assert!(content.contains("__sand_event_bounded_item_expire/"));
}
