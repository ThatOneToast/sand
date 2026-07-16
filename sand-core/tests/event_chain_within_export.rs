//! Export coverage for bounded cross-tick correlation (`.within(...)`, #240
//! Phase 5).

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch,
    TickEventDispatch, TickWindow,
};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct CurrentEvent;
struct PriorEvent;
struct ProviderOnlyPrior;
struct OtherCurrent;

impl SandEvent for CurrentEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p5_current matches 1.."))
    }
}
impl SandEvent for PriorEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p5_prior matches 1.."))
    }
}
impl SandEvent for ProviderOnlyPrior {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p5_provider matches 1.."))
    }
}
impl SandEvent for OtherCurrent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p5_other matches 1.."))
    }
}

struct ShortWindowChild;
struct LongWindowChild;
struct WithWhileChild;
struct AfterAnyWithinChild;
struct ProviderOnlyChild;

impl SandEvent for ShortWindowChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<CurrentEvent>()
            .within::<PriorEvent>(TickWindow::new(3).unwrap())
    }
}
impl SandEvent for LongWindowChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<OtherCurrent>()
            .within::<PriorEvent>(TickWindow::new(9).unwrap())
    }
}
impl SandEvent for WithWhileChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<CurrentEvent>()
            .within::<PriorEvent>(TickWindow::new(5).unwrap())
            .while_::<PlayerSneakEvent>()
    }
}
impl SandEvent for AfterAnyWithinChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(CurrentEvent, OtherCurrent)>()
            .within::<PriorEvent>(TickWindow::new(4).unwrap())
    }
}
impl SandEvent for ProviderOnlyChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<CurrentEvent>()
            .within::<ProviderOnlyPrior>(TickWindow::new(6).unwrap())
    }
}

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<TickEventDispatch> {
    None
}
fn no_chain() -> Option<ChainEventDispatch> {
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
            sand_core::inventory::submit! {
                EventDescriptor {
                    path: $path,
                    id_override: None,
                    make: body,
                    dispatch: EventDispatch::Custom {
                        make_trigger: no_trigger,
                        make_condition: no_condition,
                        make_tick: no_tick,
                        make_chain: chain,
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

submit_handler!(ShortWindowChild, "on_short_window", "say short");
submit_handler!(LongWindowChild, "on_long_window", "say long");
submit_handler!(WithWhileChild, "on_with_while", "say with_while");
submit_handler!(
    AfterAnyWithinChild,
    "on_after_any_within",
    "say after_any_within"
);
submit_handler!(ProviderOnlyChild, "on_provider_only", "say provider_only");

fn current_tick() -> Option<TickEventDispatch> {
    match CurrentEvent::dispatch().into() {
        SandEventDispatch::Tick(tick) => Some(tick),
        _ => None,
    }
}
fn current_type_id() -> TypeId {
    TypeId::of::<CurrentEvent>()
}
fn current_type_name() -> &'static str {
    std::any::type_name::<CurrentEvent>()
}
fn current_body() -> Vec<String> {
    vec!["say current".into()]
}
fn current_setup() -> EventSetup {
    CurrentEvent::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_current",
        id_override: None,
        make: current_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: current_tick,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: current_type_id,
            event_type_name: current_type_name,
            make_setup: current_setup,
        },
    }
}

fn other_tick() -> Option<TickEventDispatch> {
    match OtherCurrent::dispatch().into() {
        SandEventDispatch::Tick(tick) => Some(tick),
        _ => None,
    }
}
fn other_type_id() -> TypeId {
    TypeId::of::<OtherCurrent>()
}
fn other_type_name() -> &'static str {
    std::any::type_name::<OtherCurrent>()
}
fn other_body() -> Vec<String> {
    vec!["say other".into()]
}
fn other_setup() -> EventSetup {
    OtherCurrent::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_other",
        id_override: None,
        make: other_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: other_tick,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: other_type_id,
            event_type_name: other_type_name,
            make_setup: other_setup,
        },
    }
}

fn key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("withinpack").expect("export succeeds");
    serde_json::from_str(&json).expect("valid export JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

#[test]
fn bounded_parent_gets_one_shared_age_objective() {
    let records = records();
    let prior_key = key(std::any::type_name::<PriorEvent>());
    let objective = format!("scoreboard objectives add se_{prior_key}_wa dummy");
    assert_eq!(
        records
            .iter()
            .filter_map(|record| record["content"].as_str())
            .flat_map(str::lines)
            .filter(|line| *line == objective)
            .count(),
        1,
        "one shared age objective for PriorEvent even though two children (with two different windows) reference it"
    );
}

#[test]
fn provider_only_bounded_parent_is_subscribed() {
    let records = records();
    let provider_key = key(std::any::type_name::<ProviderOnlyPrior>());
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["dir"] == "function"
                    && record["path"] == format!("__sand_event_check/{provider_key}")
            })
            .count(),
        1,
        "ProviderOnlyPrior has no direct #[event] handler but must still get a detector because a `.within` reads its occurrence"
    );
    let objective = format!("scoreboard objectives add se_{provider_key}_wa dummy");
    assert!(
        records
            .iter()
            .filter_map(|record| record["content"].as_str())
            .flat_map(str::lines)
            .any(|line| line == objective)
    );
}

#[test]
fn age_counter_maintenance_runs_after_root_checks_and_before_staged_evaluation() {
    let records = records();
    let cycle = function(&records, "__sand_event_cycle");
    let prior_key = key(std::any::type_name::<PriorEvent>());
    let short_key = key(std::any::type_name::<ShortWindowChild>());

    let refresh_line = format!(
        "execute as @a if score @s se_{prior_key}_o matches 1 run scoreboard players set @s se_{prior_key}_wa 0"
    );
    let increment_line = format!(
        "execute as @a unless score @s se_{prior_key}_o matches 1 run scoreboard players add @s se_{prior_key}_wa 1"
    );
    assert!(cycle.contains(&refresh_line), "{cycle}");
    assert!(cycle.contains(&increment_line), "{cycle}");

    let root_check_position = cycle
        .find("function withinpack:__sand_event_check/")
        .expect("at least one root detector call");
    let refresh_position = cycle.find(&refresh_line).unwrap();
    let eval_position = cycle
        .find(&format!("__sand_event_multi_eval/{short_key}"))
        .expect("staged evaluation for the bounded child");

    assert!(
        root_check_position < refresh_position,
        "age refresh must run after root detectors have committed this tick's occurrence marks"
    );
    assert!(
        refresh_position < eval_position,
        "staged children must read the age counter only after it has been updated this tick"
    );
}

#[test]
fn resolved_condition_uses_window_minus_one_as_the_inclusive_upper_bound() {
    let records = records();
    let prior_key = key(std::any::type_name::<PriorEvent>());
    let short_key = key(std::any::type_name::<ShortWindowChild>());
    let long_key = key(std::any::type_name::<LongWindowChild>());

    let short_eval = function(&records, &format!("__sand_event_multi_eval/{short_key}"));
    assert!(
        short_eval.contains(&format!("if score @s se_{prior_key}_wa matches ..2")),
        "window 3 -> age <= 2: {short_eval}"
    );

    let long_eval = function(&records, &format!("__sand_event_multi_eval/{long_key}"));
    assert!(
        long_eval.contains(&format!("if score @s se_{prior_key}_wa matches ..8")),
        "window 9 -> age <= 8: {long_eval}"
    );
}

#[test]
fn within_composes_with_while_and_after_any() {
    let records = records();
    let prior_key = key(std::any::type_name::<PriorEvent>());
    let with_while_key = key(std::any::type_name::<WithWhileChild>());
    let any_key = key(std::any::type_name::<AfterAnyWithinChild>());

    let with_while_eval = function(
        &records,
        &format!("__sand_event_multi_eval/{with_while_key}"),
    );
    assert!(with_while_eval.contains(&format!("if score @s se_{prior_key}_wa matches ..4")));
    assert!(with_while_eval.contains("predicate withinpack:__sand/player_sneaking"));

    let any_gate = function(&records, &format!("__sand_event_multi_gate/{any_key}"));
    assert!(any_gate.contains(&format!("__sand_event_multi_eval/{any_key}")));
    let any_eval = function(&records, &format!("__sand_event_multi_eval/{any_key}"));
    assert!(any_eval.contains(&format!("if score @s se_{prior_key}_wa matches ..3")));
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("withinpack").unwrap();
    let second = sand_core::try_export_components_json("withinpack").unwrap();
    assert_eq!(first, second);
}
