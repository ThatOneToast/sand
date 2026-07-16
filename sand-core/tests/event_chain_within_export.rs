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

struct RootA;
struct RootB;

impl SandEvent for RootA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p5_root_a matches 1.."))
    }
}
impl SandEvent for RootB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p5_root_b matches 1.."))
    }
}

/// A bounded parent that is itself staged (composed via `after_any`), not a
/// root or immediate single-`after` fast-path node. Its own occurrence mark
/// is only set when ITS OWN staged evaluation runs, not merely once root
/// detectors have finished — the age-counter update for it must be emitted
/// after that specific evaluation, never in the flat root-adjacent block.
struct StagedParent;
impl SandEvent for StagedParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(RootA, RootB)>()
    }
}

struct BoundedOnStagedParent;
impl SandEvent for BoundedOnStagedParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<CurrentEvent>()
            .within::<StagedParent>(TickWindow::new(7).unwrap())
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
submit_handler!(
    BoundedOnStagedParent,
    "on_bounded_on_staged_parent",
    "say bounded_on_staged_parent"
);

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

macro_rules! submit_root {
    ($event:ty, $path:literal, $body:literal) => {
        const _: () = {
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
            sand_core::inventory::submit! {
                EventDescriptor {
                    path: $path,
                    id_override: None,
                    make: body,
                    dispatch: EventDispatch::Custom {
                        make_trigger: no_trigger,
                        make_condition: no_condition,
                        make_tick: tick,
                        make_chain: no_chain,
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

submit_root!(RootA, "on_root_a", "say root_a");
submit_root!(RootB, "on_root_b", "say root_b");

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
        "execute as @a unless score @s se_{prior_key}_o matches 1 unless score @s se_{prior_key}_wa matches 24000.. run scoreboard players add @s se_{prior_key}_wa 1"
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

/// Regression test: a bounded parent that is itself staged (composed via
/// `after_any`, not a root or immediate single-`after` fast-path node) must
/// have its age-counter update run *after* its own staged evaluation call —
/// never in the flat block placed right after `root_checks`, since that
/// parent's occurrence mark is not committed until its own evaluation runs.
#[test]
fn age_update_for_a_staged_bounded_parent_runs_after_its_own_evaluation_not_after_root_checks() {
    let records = records();
    let cycle = function(&records, "__sand_event_cycle");
    let staged_parent_key = key(std::any::type_name::<StagedParent>());
    let bounded_child_key = key(std::any::type_name::<BoundedOnStagedParent>());

    let refresh_line = format!(
        "execute as @a if score @s se_{staged_parent_key}_o matches 1 run scoreboard players set @s se_{staged_parent_key}_wa 0"
    );
    let increment_line = format!(
        "execute as @a unless score @s se_{staged_parent_key}_o matches 1 unless score @s se_{staged_parent_key}_wa matches 24000.. run scoreboard players add @s se_{staged_parent_key}_wa 1"
    );
    assert!(cycle.contains(&refresh_line), "{cycle}");
    assert!(cycle.contains(&increment_line), "{cycle}");

    // StagedParent's own evaluation (an after_any gate over RootA/RootB) must
    // run, and its result must be committed, before the age update reads it.
    let staged_parent_gate_position = cycle
        .find(&format!("__sand_event_multi_gate/{staged_parent_key}"))
        .or_else(|| cycle.find(&format!("__sand_event_multi_eval/{staged_parent_key}")))
        .expect("StagedParent's own staged evaluation is emitted");
    let refresh_position = cycle.find(&refresh_line).unwrap();
    assert!(
        staged_parent_gate_position < refresh_position,
        "age update for a staged bounded parent must run after that parent's own evaluation: {cycle}"
    );

    // The bounded child's own evaluation must run after the age update, so it
    // never reads a stale (pre-refresh) age for this tick.
    let child_eval_position = cycle
        .find(&format!("__sand_event_multi_eval/{bounded_child_key}"))
        .expect("bounded child evaluation is emitted");
    assert!(
        refresh_position < child_eval_position,
        "bounded child must read the age counter only after it has been updated this tick: {cycle}"
    );

    let child_eval = function(
        &records,
        &format!("__sand_event_multi_eval/{bounded_child_key}"),
    );
    assert!(
        child_eval.contains(&format!(
            "if score @s se_{staged_parent_key}_wa matches ..6"
        )),
        "window 7 -> age <= 6: {child_eval}"
    );
}

// ── Overflow/saturation coverage ────────────────────────────────────────────
//
// Minecraft scoreboard values are signed 32-bit. An unguarded `add ... 1` on
// a permanently-idle bounded parent would eventually overflow and wrap
// negative, which would incorrectly re-satisfy `age <= N - 1` for every
// window until the parent fires again. The generated age update is guarded
// to stop incrementing once it reaches `TickWindow::MAX_TICKS` (24000) — the
// largest representable window, and therefore already permanently "expired"
// for every valid window regardless of how much further real time passes.

#[test]
fn generated_increment_line_is_guarded_by_the_sentinel_and_cannot_reach_it_from_a_single_add() {
    let records = records();
    let prior_key = key(std::any::type_name::<PriorEvent>());
    let cycle = function(&records, "__sand_event_cycle");
    let guarded_increment = format!(
        "execute as @a unless score @s se_{prior_key}_o matches 1 unless score @s se_{prior_key}_wa matches {}.. run scoreboard players add @s se_{prior_key}_wa 1",
        TickWindow::MAX_TICKS
    );
    assert!(cycle.contains(&guarded_increment), "{cycle}");
    // Never emit a bare, unguarded add on the age objective anywhere in the
    // coordinator — every mutation of `_wa` must go through the sentinel
    // guard or the reset-to-0 refresh line.
    for line in cycle.lines() {
        if line.contains(&format!("se_{prior_key}_wa")) && line.contains("scoreboard players add") {
            assert!(
                line.contains(&format!("unless score @s se_{prior_key}_wa matches")),
                "every age-objective increment must be sentinel-guarded: {line}"
            );
        }
    }
}

/// Pure model of the two generated age-update commands (refresh-on-occurrence,
/// sentinel-guarded increment otherwise), independent of the export pipeline,
/// so the *algorithm* the generated commands implement can be exercised over
/// many simulated ticks without needing a live Minecraft server. The sentinel
/// and sequencing here are kept identical to the generated command shape
/// asserted above; a change to one without the other should be caught by
/// `generated_increment_line_is_guarded_by_the_sentinel_and_cannot_reach_it_from_a_single_add`.
struct AgeCounterModel {
    /// `None` mirrors an objective a player has no score on yet — Minecraft's
    /// `scoreboard players add` semantics implicitly initialize an absent
    /// score to 0 before adding, which this model reproduces explicitly.
    age: Option<i64>,
}

impl AgeCounterModel {
    fn new() -> Self {
        Self { age: None }
    }

    /// One simulated tick. `occurred` mirrors `se_{key}_o` being 1 this tick.
    fn tick(&mut self, occurred: bool, sentinel: i64) {
        if occurred {
            self.age = Some(0);
            return;
        }
        let current = self.age.unwrap_or(0);
        if current < sentinel {
            self.age = Some(current + 1);
        }
        // else: sentinel-guarded — no-op, exactly mirroring the generated
        // `unless score @s se_{key}_wa matches {sentinel}..` guard.
    }

    fn age(&self) -> i64 {
        self.age.unwrap_or(0)
    }
}

#[test]
fn absent_age_score_initializes_through_the_increment_path() {
    let mut model = AgeCounterModel::new();
    assert_eq!(model.age, None);
    model.tick(false, i64::from(TickWindow::MAX_TICKS));
    assert_eq!(
        model.age(),
        1,
        "an absent score initializes to 0 then increments to 1, matching vanilla `scoreboard players add` on an unset score"
    );
}

#[test]
fn age_increments_normally_below_the_sentinel() {
    let sentinel = i64::from(TickWindow::MAX_TICKS);
    let mut model = AgeCounterModel::new();
    for expected in 1..=100 {
        model.tick(false, sentinel);
        assert_eq!(model.age(), expected);
    }
}

#[test]
fn age_does_not_increment_at_or_above_the_sentinel_and_cannot_wrap_negative() {
    let sentinel = i64::from(TickWindow::MAX_TICKS);
    let mut model = AgeCounterModel::new();
    // Drive well past the sentinel — many more ticks than the sentinel value
    // itself, standing in for arbitrarily long world/pack uptime.
    for _ in 0..(sentinel * 3) {
        model.tick(false, sentinel);
        assert!(model.age() >= 0, "age must never go negative");
        assert!(
            model.age() <= sentinel,
            "age must never exceed the sentinel"
        );
    }
    assert_eq!(
        model.age(),
        sentinel,
        "age must saturate exactly at the sentinel, not drift past it"
    );
}

#[test]
fn an_occurrence_always_resets_a_saturated_age_to_zero() {
    let sentinel = i64::from(TickWindow::MAX_TICKS);
    let mut model = AgeCounterModel::new();
    for _ in 0..(sentinel * 2) {
        model.tick(false, sentinel);
    }
    assert_eq!(model.age(), sentinel, "precondition: age is saturated");
    model.tick(true, sentinel);
    assert_eq!(
        model.age(),
        0,
        "refresh takes precedence over the sentinel guard: an occurrence always resets age to 0, even from saturation"
    );
}

#[test]
fn the_sentinel_does_not_satisfy_the_largest_supported_window() {
    // Condition::Score { range: ScoreRange::Lte(N - 1) } for the largest
    // supported window N = TickWindow::MAX_TICKS: age <= MAX_TICKS - 1.
    let sentinel = i64::from(TickWindow::MAX_TICKS);
    let largest_window_upper_bound = i64::from(TickWindow::MAX_TICKS) - 1;
    assert!(
        sentinel > largest_window_upper_bound,
        "the sentinel must sit strictly above every valid window's inclusive upper bound, so a saturated age is expired for every valid window (age <= N - 1 is false for all supported N when age == sentinel)"
    );

    let mut model = AgeCounterModel::new();
    for _ in 0..(sentinel * 2) {
        model.tick(false, sentinel);
    }
    assert!(
        model.age() > largest_window_upper_bound,
        "saturated age does not satisfy even the largest supported window"
    );
}

#[test]
fn tick_window_max_ticks_is_the_sentinel_used_by_generated_commands() {
    // Ties this whole test module's sentinel constant to the same public API
    // value referenced in the generated command (asserted exactly in
    // `generated_increment_line_is_guarded_by_the_sentinel_and_cannot_reach_it_from_a_single_add`),
    // so a change to one is guaranteed to be caught by the other.
    assert_eq!(TickWindow::MAX_TICKS, 24_000);
}
