//! End-to-end pipeline tests using exact captured Minecraft 26.2 log
//! fixtures (`tests/fixtures/console/`). Each fixture is replayed through
//! the same stages `console::process::drive` uses — `parse_line` ->
//! `classify` -> `Grouper` -> `PhaseTracker` -> `build_diagnostic` ->
//! `Correlator` -> `Deduplicator` -> `HealthTracker` — without spawning a
//! real process, proving the classifier/grouping/correlation/health model
//! against real server output rather than only synthetic unit strings.

use sand_cli::console::classify::{Category, classify};
use sand_cli::console::correlate::Correlator;
use sand_cli::console::dedup::{Deduplicator, Observation};
use sand_cli::console::diagnostic::{Diagnostic, DiagnosticCode, Grouper, build_diagnostic};
use sand_cli::console::health::{HealthTracker, RunHealth};
use sand_cli::console::log_record::{Stream, parse_line};
use sand_cli::console::phase::{PhaseTracker, RunPhase};

const FUNCTION_PARSE_LOAD_FAILURE: &str =
    include_str!("fixtures/console/minecraft_26_2_function_parse_load_failure.log");
const CLEAN_STARTUP: &str = include_str!("fixtures/console/minecraft_26_2_clean_startup.log");

/// The diagnostics that would actually be printed, and the final
/// [`RunHealth`], from replaying a raw captured log block through the full
/// pipeline in the same order `process::drive` uses.
struct Replay {
    diagnostics: Vec<Diagnostic>,
    health: RunHealth,
}

fn replay(raw: &str) -> Replay {
    let mut grouper = Grouper::new();
    let mut phase = PhaseTracker::new();
    let mut correlator = Correlator::new();
    let mut dedup = Deduplicator::new();
    let mut health = HealthTracker::new();
    let mut diagnostics = Vec::new();

    let handle_group = |group: sand_cli::console::diagnostic::GroupedEvent,
                        phase: RunPhase,
                        correlator: &mut Correlator,
                        dedup: &mut Deduplicator,
                        health: &mut HealthTracker,
                        out: &mut Vec<Diagnostic>| {
        if !matches!(
            group.category,
            Category::DatapackError | Category::FatalError
        ) {
            return;
        }
        let Some(diag) = build_diagnostic(&group, phase) else {
            return;
        };
        health.observe(&diag);
        for correlated in correlator.observe(diag) {
            if let Observation::New { diagnostic, .. } = dedup.observe(correlated) {
                out.push(*diagnostic);
            }
        }
    };

    for line in raw.lines() {
        let record = parse_line(line);
        phase.observe_log(&record.message);
        let category = classify(Stream::Stdout, &record);
        for group in grouper.feed(Stream::Stdout, record, category) {
            handle_group(
                group,
                phase.current(),
                &mut correlator,
                &mut dedup,
                &mut health,
                &mut diagnostics,
            );
        }
    }
    if let Some(group) = grouper.flush() {
        handle_group(
            group,
            phase.current(),
            &mut correlator,
            &mut dedup,
            &mut health,
            &mut diagnostics,
        );
    }
    if let Some(root) = correlator.flush()
        && let Observation::New { diagnostic, .. } = dedup.observe(root)
    {
        diagnostics.push(*diagnostic);
    }

    Replay {
        diagnostics,
        health: health.current(),
    }
}

#[test]
fn function_parse_failure_produces_exactly_one_root_diagnostic() {
    let replay = replay(FUNCTION_PARSE_LOAD_FAILURE);
    assert_eq!(
        replay.diagnostics.len(),
        1,
        "the stack trace, Caused by, and missing-tag consequence must all fold into one \
         diagnostic, not stand alone: {:#?}",
        replay.diagnostics
    );
}

#[test]
fn root_diagnostic_has_the_correct_function_and_position() {
    let diag = &replay(FUNCTION_PARSE_LOAD_FAILURE).diagnostics[0];
    assert_eq!(diag.code, DiagnosticCode::CommandParseError);
    assert_eq!(diag.resource.as_deref(), Some("vanilla_plus:on_load"));
    assert_eq!(diag.line, Some(6));
    assert_eq!(diag.cursor, Some(0));
    assert_eq!(diag.reason, "Unknown or incomplete command");
    assert_eq!(diag.phase, RunPhase::DatapackDiscovery);
}

#[test]
fn root_diagnostic_carries_the_full_raw_stack_trace_for_verbose_mode() {
    let diag = &replay(FUNCTION_PARSE_LOAD_FAILURE).diagnostics[0];
    let joined = diag.raw_lines.join("\n");
    assert!(joined.contains("Failed to load function vanilla_plus:on_load"));
    assert!(joined.contains("CompletionException"));
    assert!(joined.contains("Caused by:"));
    assert!(joined.contains("... 8 more"));
    // The stack trace appears exactly once — not duplicated by grouping.
    assert_eq!(joined.matches("Caused by:").count(), 1);
}

#[test]
fn missing_tag_consequence_is_linked_not_a_second_top_level_failure() {
    let diag = &replay(FUNCTION_PARSE_LOAD_FAILURE).diagnostics[0];
    assert_eq!(diag.related.len(), 1);
    let related = &diag.related[0];
    assert_eq!(related.code, DiagnosticCode::MissingReference);
    assert_eq!(related.resource, "minecraft:load");
    assert_eq!(related.missing, "vanilla_plus:on_load");
    assert_eq!(related.source.as_deref(), Some("file/vanilla_plus"));
}

#[test]
fn ready_banner_does_not_restore_healthy_status() {
    let replay = replay(FUNCTION_PARSE_LOAD_FAILURE);
    // The fixture's log ends with `Done (...)! For help, type "help"` —
    // health must remain Degraded despite that ready banner appearing
    // after the failure.
    assert_eq!(replay.health, RunHealth::Degraded);
}

#[test]
fn clean_startup_fixture_reports_no_diagnostics_and_healthy_status() {
    let replay = replay(CLEAN_STARTUP);
    assert!(
        replay.diagnostics.is_empty(),
        "a clean startup log must not produce any diagnostics: {:#?}",
        replay.diagnostics
    );
    assert_eq!(replay.health, RunHealth::Healthy);
}

#[test]
fn multiple_distinct_function_failures_remain_distinct_diagnostics() {
    let combined = format!(
        "{}\n[10:16:00] [Server thread/ERROR]: Failed to load function vanilla_plus:on_tick\n\
         java.util.concurrent.CompletionException: java.lang.IllegalArgumentException: Whilst parsing command on line 2: Unknown or incomplete command. See below for error at position 3: <--[HERE]\n\
         \tat java.base/java.util.concurrent.CompletableFuture.wrapInCompletionException(CompletableFuture.java:323)\n",
        FUNCTION_PARSE_LOAD_FAILURE.trim_end()
    );
    let replay = replay(&combined);
    assert_eq!(replay.diagnostics.len(), 2);
    let resources: Vec<_> = replay
        .diagnostics
        .iter()
        .filter_map(|d| d.resource.as_deref())
        .collect();
    assert!(resources.contains(&"vanilla_plus:on_load"));
    assert!(resources.contains(&"vanilla_plus:on_tick"));
}
