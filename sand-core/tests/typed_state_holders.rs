//! Typed score-holder coverage for `Flag`, `Timer`, and `Cooldown` (#146).
//!
//! These exercise the real public API: every `try_*` method must validate its
//! holder through the canonical [`sand_commands::ScoreHolder`] before any
//! command text is produced, and must render byte-identically to its raw
//! compatibility counterpart for equivalent valid input.

use sand_commands::{ScoreHolder, Target};
use sand_core::execute_when::when;
use sand_core::state::{Cooldown, Flag, Ticks, Timer};

static CASTING: Flag = Flag::new("casting");
static BLINK: Timer = Timer::new("blink_t", Ticks::new(40));
static DASH: Cooldown = Cooldown::new("dash_cd", Ticks::new(60));

// ── Holder fixtures ──────────────────────────────────────────────────────────

/// A multi-target selector — legal for `set`-style mutation, illegal wherever
/// vanilla requires exactly one holder.
fn multi_target() -> ScoreHolder {
    ScoreHolder::entity(Target::all_players())
}

/// A selector statically narrowed to one entity.
fn single_entity() -> ScoreHolder {
    ScoreHolder::entity(Target::all_entities().limit(1).unwrap())
}

// ── Flag ─────────────────────────────────────────────────────────────────────

#[test]
fn flag_typed_player_selector_matches_compatibility_rendering() {
    let typed = CASTING.try_enable(ScoreHolder::self_()).unwrap();
    assert_eq!(typed, CASTING.enable("@s"));
    assert_eq!(typed, "scoreboard players set @s casting 1");
}

#[test]
fn flag_typed_literal_player_holder_renders_the_player_name() {
    let typed = CASTING.try_enable(ScoreHolder::player("Steve")).unwrap();
    assert_eq!(typed, CASTING.enable("Steve"));
    assert_eq!(typed, "scoreboard players set Steve casting 1");
}

#[test]
fn flag_typed_fake_player_holder_is_supported() {
    let typed = CASTING.try_disable(ScoreHolder::fake("#global")).unwrap();
    assert_eq!(typed, CASTING.disable("#global"));
    assert_eq!(typed, "scoreboard players set #global casting 0");
}

#[test]
fn flag_typed_single_entity_selector_is_supported_for_conditions() {
    let cond = CASTING.try_of(single_entity()).unwrap().is_true();
    assert_eq!(
        when(cond).then_one("say ok"),
        vec!["execute if score @e[limit=1] casting matches 1 run say ok"]
    );
}

#[test]
fn flag_mutations_accept_multi_target_holders() {
    // `scoreboard players set @a <obj> 1` is legal vanilla — mutation must not
    // impose single-holder cardinality.
    let typed = CASTING.try_enable(multi_target()).unwrap();
    assert_eq!(typed, CASTING.enable("@a"));
}

#[test]
fn flag_conditions_reject_wildcard_holders() {
    let err = CASTING.try_of(ScoreHolder::wildcard()).unwrap_err();
    assert!(err.to_string().contains("Flag::try_of"), "got: {err}");

    assert!(CASTING.try_when_true(ScoreHolder::wildcard()).is_err());
    assert!(CASTING.try_when_false(ScoreHolder::wildcard()).is_err());
    assert!(CASTING.try_unless_true(ScoreHolder::wildcard()).is_err());
}

#[test]
fn flag_conditions_reject_multi_target_selectors() {
    let err = CASTING.try_of(multi_target()).unwrap_err();
    assert!(err.to_string().contains("exactly one holder"), "got: {err}");
}

#[test]
fn flag_rejects_whitespace_and_control_character_holders() {
    assert!(CASTING.try_enable(ScoreHolder::fake("bad name")).is_err());
    assert!(
        CASTING
            .try_enable(ScoreHolder::fake("bad\u{7}name"))
            .is_err()
    );
    assert!(CASTING.try_enable(ScoreHolder::fake("")).is_err());
}

#[test]
fn flag_rejects_malformed_selector_holders() {
    assert!(
        CASTING
            .try_enable(ScoreHolder::entity(Target::named_player(
                "not a valid name"
            )))
            .is_err()
    );
}

#[test]
fn flag_toggle_and_init_have_typed_parity() {
    assert_eq!(
        CASTING.try_toggle(ScoreHolder::self_()).unwrap(),
        CASTING.toggle("@s")
    );
    assert_eq!(
        CASTING.try_init_false(ScoreHolder::self_()).unwrap(),
        CASTING.init_false("@s")
    );
    assert_eq!(
        CASTING.try_init_true(ScoreHolder::self_()).unwrap(),
        CASTING.init_true("@s")
    );
    assert_eq!(
        CASTING.try_set(ScoreHolder::self_(), true).unwrap(),
        CASTING.set("@s", true)
    );
    assert_eq!(
        CASTING.try_clear(ScoreHolder::self_()).unwrap(),
        CASTING.clear("@s")
    );
}

#[test]
fn flag_toggle_and_init_require_a_single_holder() {
    // `toggle`, `init_false`, and `init_true` all lower to
    // `execute if/unless score <holder> …`, which vanilla requires be exactly
    // one holder — unlike the plain `set`-style mutations above.
    for holder in [ScoreHolder::wildcard(), multi_target()] {
        assert!(
            CASTING.try_toggle(holder.clone()).is_err(),
            "toggle builds a score condition and must reject `{holder}`"
        );
        assert!(
            CASTING.try_init_false(holder.clone()).is_err(),
            "init_false builds a score condition and must reject `{holder}`"
        );
        assert!(
            CASTING.try_init_true(holder).is_err(),
            "init_true builds a score condition and must reject it"
        );
    }
}

// ── Timer ────────────────────────────────────────────────────────────────────

#[test]
fn timer_typed_mutations_match_compatibility_rendering() {
    assert_eq!(
        BLINK.try_start(ScoreHolder::self_()).unwrap(),
        BLINK.start("@s")
    );
    assert_eq!(
        BLINK.try_tick(ScoreHolder::self_()).unwrap(),
        BLINK.tick("@s")
    );
    assert_eq!(
        BLINK.try_reset(ScoreHolder::self_()).unwrap(),
        BLINK.reset("@s")
    );
}

#[test]
fn timer_typed_holders_cover_player_fake_and_single_entity() {
    assert_eq!(
        BLINK.try_start(ScoreHolder::player("Steve")).unwrap(),
        BLINK.start("Steve")
    );
    assert_eq!(
        BLINK.try_start(ScoreHolder::fake("#ticker")).unwrap(),
        BLINK.start("#ticker")
    );
    assert!(BLINK.try_expired(single_entity()).is_ok());
}

#[test]
fn timer_accepts_raw_single_target_after_score_holder_conversion() {
    let holder = ScoreHolder::from(Target::raw_single("@e[tag=clock,limit=1]"));
    assert_eq!(
        BLINK.try_tick(holder).unwrap(),
        BLINK.tick("@e[tag=clock,limit=1]")
    );
}

#[test]
fn timer_conditions_generate_the_expected_commands() {
    let expired = BLINK.try_expired(ScoreHolder::self_()).unwrap();
    assert_eq!(
        when(expired).then_one("say ok"),
        vec!["execute if score @s blink_t matches 0 run say ok"]
    );
    let active = BLINK.try_active(ScoreHolder::self_()).unwrap();
    assert_eq!(
        when(active).then_one("say ok"),
        vec!["execute if score @s blink_t matches 1.. run say ok"]
    );
}

#[test]
fn timer_conditions_and_guards_require_a_single_holder() {
    assert!(BLINK.try_expired(ScoreHolder::wildcard()).is_err());
    assert!(BLINK.try_active(ScoreHolder::wildcard()).is_err());
    assert!(BLINK.try_guard_active(ScoreHolder::wildcard()).is_err());
    assert!(BLINK.try_expired(multi_target()).is_err());
    assert!(BLINK.try_active(multi_target()).is_err());
    assert!(BLINK.try_guard_active(multi_target()).is_err());
}

#[test]
fn timer_rejects_whitespace_and_control_character_holders() {
    assert!(BLINK.try_start(ScoreHolder::fake("bad name")).is_err());
    assert!(BLINK.try_start(ScoreHolder::fake("bad\u{7}name")).is_err());
}

#[test]
fn timer_guard_active_has_typed_parity() {
    assert_eq!(
        BLINK.try_guard_active(ScoreHolder::self_()).unwrap(),
        BLINK.guard_active("@s")
    );
}

// ── Cooldown ─────────────────────────────────────────────────────────────────

#[test]
fn cooldown_typed_mutations_match_compatibility_rendering() {
    assert_eq!(
        DASH.try_start(ScoreHolder::self_()).unwrap(),
        DASH.start("@s")
    );
    assert_eq!(
        DASH.try_stop(ScoreHolder::self_()).unwrap(),
        DASH.stop("@s")
    );
    assert_eq!(
        DASH.try_tick(ScoreHolder::self_()).unwrap(),
        DASH.tick("@s")
    );
    assert_eq!(
        DASH.try_start_for(ScoreHolder::self_()).unwrap(),
        DASH.start_for("@s")
    );
    assert_eq!(
        DASH.try_reset_for(ScoreHolder::self_()).unwrap(),
        DASH.reset_for("@s")
    );
}

#[test]
fn cooldown_typed_holders_cover_player_fake_and_single_entity() {
    assert_eq!(
        DASH.try_start(ScoreHolder::player("Steve")).unwrap(),
        DASH.start("Steve")
    );
    assert_eq!(
        DASH.try_start(ScoreHolder::fake("#shared")).unwrap(),
        DASH.start("#shared")
    );
    assert!(DASH.try_ready(single_entity()).is_ok());
}

#[test]
fn cooldown_conditions_generate_the_expected_commands() {
    let ready = DASH.try_ready(ScoreHolder::self_()).unwrap();
    assert_eq!(
        when(ready.clone()).then_one("say ok"),
        vec!["execute if score @s dash_cd matches 0 run say ok"]
    );
    let active = DASH.try_active(ScoreHolder::self_()).unwrap();
    assert_eq!(
        when(active).then_one("say ok"),
        vec!["execute if score @s dash_cd matches 1.. run say ok"]
    );
    let expired = DASH.try_expired(ScoreHolder::self_()).unwrap();
    assert_eq!(expired, ready, "`expired` is an alias of `ready`");
}

#[test]
fn cooldown_conditions_and_guards_require_a_single_holder() {
    for holder in [ScoreHolder::wildcard(), multi_target()] {
        assert!(DASH.try_ready(holder.clone()).is_err());
        assert!(DASH.try_active(holder.clone()).is_err());
        assert!(DASH.try_expired(holder.clone()).is_err());
        assert!(DASH.try_guard(holder.clone()).is_err());
        assert!(DASH.try_guard_active(holder.clone()).is_err());
        assert!(DASH.try_guard_ready(holder).is_err());
    }
}

#[test]
fn cooldown_guards_have_typed_parity() {
    assert_eq!(
        DASH.try_guard(ScoreHolder::self_()).unwrap(),
        DASH.guard("@s")
    );
    assert_eq!(
        DASH.try_guard_active(ScoreHolder::self_()).unwrap(),
        DASH.guard_active("@s")
    );
    assert_eq!(
        DASH.try_guard_ready(ScoreHolder::self_()).unwrap(),
        DASH.guard_ready("@s")
    );
}

#[test]
fn cooldown_rejects_whitespace_and_control_character_holders() {
    assert!(DASH.try_start(ScoreHolder::fake("bad name")).is_err());
    assert!(DASH.try_start(ScoreHolder::fake("bad\u{7}name")).is_err());
}

// ── Diagnostics ──────────────────────────────────────────────────────────────

#[test]
fn diagnostics_identify_the_state_primitive_and_operation() {
    let cases: Vec<(&str, String)> = vec![
        (
            "Flag::try_of",
            CASTING
                .try_of(ScoreHolder::wildcard())
                .unwrap_err()
                .to_string(),
        ),
        (
            "Timer::try_expired",
            BLINK
                .try_expired(ScoreHolder::wildcard())
                .unwrap_err()
                .to_string(),
        ),
        (
            "Cooldown::try_guard",
            DASH.try_guard(ScoreHolder::wildcard())
                .unwrap_err()
                .to_string(),
        ),
    ];
    for (expected, rendered) in cases {
        assert!(
            rendered.contains(expected),
            "diagnostic should name `{expected}`, got: {rendered}"
        );
    }
}
