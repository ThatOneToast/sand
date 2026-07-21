//! Multi-line grouping and datapack diagnostic extraction.
//!
//! Minecraft's log4j setup only stamps the `[HH:MM:SS] [Thread/LEVEL]:`
//! prefix on the *first* line of a log message; stack frames, parser
//! context, and carets that follow are emitted as bare, unprefixed lines.
//! [`Grouper`] uses that fact to fold a headline together with its
//! continuation lines without depending on any single Minecraft version's
//! exact wording.

use serde::Serialize;

use super::classify::Category;
use super::log_record::{LogRecord, Stream};
use super::phase::RunPhase;

/// A headline line plus any continuation lines folded into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedEvent {
    pub category: Category,
    pub stream: Stream,
    pub lines: Vec<LogRecord>,
}

impl GroupedEvent {
    pub fn headline(&self) -> &LogRecord {
        &self.lines[0]
    }
}

/// Categories whose headline is worth buffering briefly in case
/// continuation lines (stack frames, parser context) follow.
fn buffers_continuations(category: Category) -> bool {
    matches!(category, Category::DatapackError | Category::FatalError)
}

/// Folds consecutive unprefixed lines onto a preceding buffered headline.
///
/// Only [`Category::DatapackError`] and [`Category::FatalError`] headlines
/// buffer at all, so an idle server producing occasional unrelated warnings
/// is never delayed waiting for a continuation that isn't coming.
#[derive(Debug, Default)]
pub struct Grouper {
    pending: Option<GroupedEvent>,
}

impl Grouper {
    pub fn new() -> Self {
        Self { pending: None }
    }

    /// Feed one classified line in. Returns the events this line completed:
    /// empty if it was folded into a buffered headline as a continuation,
    /// one event in the common case, or two when this line both closes out
    /// a previously buffered group *and* is itself immediately emitted
    /// (non-buffering categories are never held back).
    pub fn feed(
        &mut self,
        stream: Stream,
        record: LogRecord,
        category: Category,
    ) -> Vec<GroupedEvent> {
        let mut out = Vec::new();

        let is_continuation = record.prefix.is_none() && self.pending.is_some();
        if is_continuation {
            self.pending.as_mut().unwrap().lines.push(record);
            return out;
        }

        if let Some(finished) = self.pending.take() {
            out.push(finished);
        }

        let event = GroupedEvent {
            category,
            stream,
            lines: vec![record],
        };
        if buffers_continuations(category) {
            self.pending = Some(event);
        } else {
            out.push(event);
        }
        out
    }

    /// Flush any pending buffered group (call on quiet-period timeout or
    /// stream EOF so a trailing diagnostic is never lost).
    pub fn flush(&mut self) -> Option<GroupedEvent> {
        self.pending.take()
    }
}

// ── Diagnostic extraction ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Warning,
    Error,
}

/// A stable classification of *what kind* of failure a [`Diagnostic`]
/// represents, independent of the free-form `reason` text. Kept
/// conservative: [`DiagnosticCode::Unclassified`] is the safe default for
/// any recognized-as-a-problem line that doesn't confidently match one of
/// the named categories, rather than guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    /// A generated `.mcfunction` command failed to parse, at load time.
    CommandParseError,
    /// A datapack JSON/component document (advancement, recipe, loot
    /// table, predicate, item modifier, dialog, worldgen, ...) failed to
    /// parse or validate.
    JsonComponentError,
    /// A referenced function, tag, predicate, recipe, loot table,
    /// advancement, dialog, or other registry entry does not exist.
    MissingReference,
    /// The datapack (or one of its packs) uses an incompatible/unsupported
    /// pack format, or duplicates another pack's resources.
    PackFormatIncompatible,
    /// A `/reload` (or the reload portion of startup) failed.
    ReloadFailure,
    /// The server process failed to start: wrong Java version, missing
    /// Java, port already bound, world already locked, or EULA not
    /// accepted.
    StartupFailure,
    /// The server process exited unexpectedly (crash, uncaught exception).
    ProcessExited,
    /// A command failed after a successful reload, issued directly at
    /// runtime (console/operator input) rather than during load-time
    /// function parsing.
    RuntimeCommandError,
    /// Recognized as an error/warning worth surfacing, but not confidently
    /// matched to any of the categories above.
    Unclassified,
}

/// A compact, actionable summary of a recognized datapack failure, built
/// from a [`GroupedEvent`]. Every field is `Option` (except `severity`,
/// `phase`, `code`, and `reason`) because we only ever report what
/// Minecraft's own output supports — we never invent a path, position, or
/// hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub phase: RunPhase,
    pub severity: Severity,
    pub code: DiagnosticCode,
    /// `namespace:path` resource identifier, when one appears in the log.
    pub resource: Option<String>,
    /// Datapack subsystem, e.g. `"function"`, `"recipe"`, `"loot_table"`.
    pub subsystem: Option<String>,
    /// Best-effort generated file path under `dist/<namespace>/data/...`.
    pub file: Option<String>,
    /// Line/command position, when Minecraft supplies one, e.g. `"command 4"`.
    pub position: Option<String>,
    /// The rejected command / JSON fragment / parser context, when present.
    pub context: Option<String>,
    /// Minecraft's own explanation, extracted from the headline.
    pub reason: String,
    /// An actionable next step, only populated when one can be inferred safely.
    pub hint: Option<String>,
    /// The original raw lines that made up this diagnostic, for verbose mode.
    pub raw_lines: Vec<String>,
}

impl Diagnostic {
    /// A stable-ish key used to deduplicate repeated copies of what is
    /// semantically the same root failure (e.g. the same rejected command
    /// logged once per tick). Built only from fields that identify *what*
    /// failed, not incidental details like exact raw line contents.
    pub fn fingerprint(&self) -> String {
        format!(
            "{:?}|{:?}|{}|{}|{}",
            self.phase,
            self.code,
            self.resource.as_deref().unwrap_or(""),
            self.position.as_deref().unwrap_or(""),
            normalize_for_fingerprint(&self.reason),
        )
    }
}

/// Lowercase and collapse whitespace so trivially-different renderings of
/// the same underlying message (e.g. differing internal spacing) still
/// dedupe together.
fn normalize_for_fingerprint(text: &str) -> String {
    text.to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build a [`Diagnostic`] from a grouped [`DatapackError`]/[`FatalError`]
/// event. Returns `None` for other categories. `phase` is the [`RunPhase`]
/// active when this group's headline was observed.
///
/// [`DatapackError`]: Category::DatapackError
/// [`FatalError`]: Category::FatalError
pub fn build_diagnostic(group: &GroupedEvent, phase: RunPhase) -> Option<Diagnostic> {
    if !matches!(
        group.category,
        Category::DatapackError | Category::FatalError
    ) {
        return None;
    }

    let headline = group.headline();
    let text = headline.message.as_str();

    let severity = match headline.prefix.as_ref().map(|p| p.level.as_str()) {
        Some("ERROR") | Some("FATAL") => Severity::Error,
        _ => Severity::Warning,
    };

    let resource = extract_resource_location(text).or_else(|| {
        group
            .lines
            .iter()
            .skip(1)
            .find_map(|l| extract_resource_location(&l.message))
    });

    let subsystem = detect_subsystem(text);

    let file = match (subsystem, &resource) {
        (Some("function"), Some(res)) => resource_to_function_path(res),
        _ => None,
    };

    let position = extract_position(text).or_else(|| {
        group
            .lines
            .iter()
            .skip(1)
            .find_map(|l| extract_position(&l.message))
    });

    let context = extract_context_line(&group.lines);
    let reason = extract_reason(text);
    let hint = build_hint(subsystem, text);
    let code = detect_code(group.category, phase, subsystem, text);

    Some(Diagnostic {
        phase,
        severity,
        code,
        resource,
        subsystem: subsystem.map(str::to_string),
        file,
        position,
        context,
        reason,
        hint,
        raw_lines: group.lines.iter().map(|l| l.raw.clone()).collect(),
    })
}

const MISSING_REFERENCE_MARKERS: &[&str] = &[
    "unknown function",
    "unknown recipe",
    "unknown advancement",
    "unknown loot table",
    "unknown predicate",
    "unknown tag",
    "unable to resolve",
    "no such function",
    "no function tag",
];

const PACK_FORMAT_MARKERS: &[&str] = &[
    "was designed for a newer version",
    "was designed for an older version",
    "requires format",
    "incompatible pack",
    "duplicate data pack",
];

const RELOAD_FAILURE_MARKERS: &[&str] = &["failed to reload", "reload failed"];

const STARTUP_FAILURE_MARKERS: &[&str] = &[
    "failed to bind to port",
    "address already in use",
    "unsupported class file version",
    "unsupportedclassversionerror",
    "requires using java",
    "has been compiled by a more recent version of the java runtime",
    "the world is currently being played",
    "perhaps a server is already running",
    "you need to agree to the eula",
];

/// Classify *what kind* of failure this diagnostic represents. Order
/// matters: more specific markers are checked before falling back to the
/// generic subsystem-based command/JSON split, and `phase` disambiguates a
/// command failure at runtime from the same wording during load-time
/// function parsing.
fn detect_code(
    category: Category,
    phase: RunPhase,
    subsystem: Option<&str>,
    text: &str,
) -> DiagnosticCode {
    let lower = text.to_ascii_lowercase();

    if contains_any(&lower, STARTUP_FAILURE_MARKERS) {
        return DiagnosticCode::StartupFailure;
    }
    if contains_any(&lower, MISSING_REFERENCE_MARKERS) {
        return DiagnosticCode::MissingReference;
    }
    if contains_any(&lower, PACK_FORMAT_MARKERS) {
        return DiagnosticCode::PackFormatIncompatible;
    }
    if contains_any(&lower, RELOAD_FAILURE_MARKERS) {
        return DiagnosticCode::ReloadFailure;
    }

    if category == Category::FatalError {
        return DiagnosticCode::ProcessExited;
    }

    match subsystem {
        Some("function") => {
            if lower.contains("couldn't parse command") || lower.contains("invalid function") {
                if phase == RunPhase::Runtime {
                    DiagnosticCode::RuntimeCommandError
                } else {
                    DiagnosticCode::CommandParseError
                }
            } else {
                DiagnosticCode::CommandParseError
            }
        }
        Some(_) => DiagnosticCode::JsonComponentError,
        None => {
            if lower.contains("couldn't parse command") {
                if phase == RunPhase::Runtime {
                    DiagnosticCode::RuntimeCommandError
                } else {
                    DiagnosticCode::CommandParseError
                }
            } else {
                DiagnosticCode::Unclassified
            }
        }
    }
}

fn contains_any(haystack_lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack_lower.contains(n))
}

/// Scan for a `namespace:path` resource identifier using vanilla's allowed
/// character set (lowercase ascii, digits, `_ - . /`), skipping anything
/// that looks like a URL.
fn extract_resource_location(text: &str) -> Option<String> {
    for word in text
        .split(|c: char| c.is_whitespace() || matches!(c, '\'' | '"' | '(' | ')' | ',' | '[' | ']'))
    {
        let word = word.trim_matches(|c: char| matches!(c, '.' | ':' | ';'));
        if word.contains("://") {
            continue;
        }
        if let Some((ns, path)) = word.split_once(':')
            && is_valid_namespace(ns)
            && is_valid_path(path)
        {
            return Some(word.to_string());
        }
    }
    None
}

fn is_valid_namespace(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'))
}

fn is_valid_path(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.' | '/')
        })
}

fn resource_to_function_path(resource: &str) -> Option<String> {
    let (ns, path) = resource.split_once(':')?;
    if ns.is_empty() || path.is_empty() {
        return None;
    }
    Some(format!("dist/{ns}/data/{ns}/function/{path}.mcfunction"))
}

const SUBSYSTEM_MARKERS: &[(&str, &str)] = &[
    ("loot_table", "loot_table"),
    ("loot table", "loot_table"),
    ("function", "function"),
    ("recipe", "recipe"),
    ("advancement", "advancement"),
    ("predicate", "predicate"),
    ("item_modifier", "item_modifier"),
    ("item modifier", "item_modifier"),
    ("tag", "tag"),
];

fn detect_subsystem(text: &str) -> Option<&'static str> {
    let lower = text.to_ascii_lowercase();
    SUBSYSTEM_MARKERS
        .iter()
        .find(|(needle, _)| lower.contains(needle))
        .map(|(_, v)| *v)
}

const POSITION_MARKERS: &[&str] = &["position ", "command ", "line "];

fn extract_position(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for marker in POSITION_MARKERS {
        if let Some(idx) = lower.find(marker) {
            let rest = &text[idx + marker.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                let label = marker.trim();
                return Some(format!("{label} {digits}"));
            }
        }
    }
    None
}

const REASON_MARKERS: &[&str] = &[" due to: ", "Command error: "];

fn extract_reason(text: &str) -> String {
    for marker in REASON_MARKERS {
        if let Some(idx) = text.find(marker) {
            return text[idx + marker.len()..].trim().to_string();
        }
    }
    text.trim().to_string()
}

/// Pick the first continuation line that looks like echoed command/JSON
/// context rather than stack-frame noise.
fn extract_context_line(lines: &[LogRecord]) -> Option<String> {
    lines.iter().skip(1).find_map(|l| {
        let t = l.message.trim();
        if t.is_empty() {
            return None;
        }
        if t.starts_with("at ")
            || t.starts_with("Caused by:")
            || t.starts_with('^')
            || t.starts_with("...")
        {
            return None;
        }
        Some(t.to_string())
    })
}

fn build_hint(subsystem: Option<&str>, text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("unknown or incomplete command") || lower.contains("couldn't parse command") {
        return Some("Check the command syntax at the reported position.".to_string());
    }
    if lower.contains("couldn't load data pack") {
        return Some(
            "Run `sand build` and check pack_format compatibility in pack.mcmeta.".to_string(),
        );
    }
    match subsystem {
        Some("function") => Some("Check the .mcfunction source for this function.".to_string()),
        Some(other) => Some(format!(
            "Check the generated {other} JSON for this resource."
        )),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::classify::classify;
    use crate::console::log_record::parse_line;

    fn group_lines(lines: &[&str]) -> Vec<GroupedEvent> {
        let mut grouper = Grouper::new();
        let mut out = Vec::new();
        for line in lines {
            let record = parse_line(line);
            let category = classify(Stream::Stdout, &record);
            out.extend(grouper.feed(Stream::Stdout, record, category));
        }
        if let Some(finished) = grouper.flush() {
            out.push(finished);
        }
        out
    }

    #[test]
    fn groups_multiline_datapack_error_without_swallowing_next_message() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Couldn't parse command: execute as @a run dash",
            "     run dash",
            "     ^--[HERE]",
            "[12:00:01] [Server thread/INFO]: Toast joined the game",
        ]);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].category, Category::DatapackError);
        assert_eq!(events[0].lines.len(), 3);
        assert_eq!(events[1].category, Category::PlayerEvent);
    }

    #[test]
    fn single_line_datapack_error_flushes_on_stream_end() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Error loading function arcane:combat/dash",
        ]);
        assert_eq!(events.len(), 1);

        let diag = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert_eq!(diag.resource.as_deref(), Some("arcane:combat/dash"));
        assert_eq!(diag.subsystem.as_deref(), Some("function"));
        assert_eq!(
            diag.file.as_deref(),
            Some("dist/arcane/data/arcane/function/combat/dash.mcfunction")
        );
        assert_eq!(diag.code, DiagnosticCode::CommandParseError);
    }

    #[test]
    fn extracts_position_and_context_from_multiline_group() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Function arcane:combat/dash failed execution due to: command 4 invalid",
            "execute as @a run particle minecraft:cloud ~ ~ ~ 0 0 0 1 5",
        ]);
        let diag = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert_eq!(diag.position.as_deref(), Some("command 4"));
        assert_eq!(
            diag.context.as_deref(),
            Some("execute as @a run particle minecraft:cloud ~ ~ ~ 0 0 0 1 5")
        );
        assert_eq!(diag.reason, "command 4 invalid");
    }

    #[test]
    fn does_not_classify_solely_on_namespace_presence() {
        // A plain INFO line mentioning a namespaced id is not, by itself, an error.
        let events =
            group_lines(&["[12:00:00] [Server thread/INFO]: Loaded recipe arcane:combat/dash"]);
        assert_eq!(events[0].category, Category::Visible);
        assert!(build_diagnostic(&events[0], RunPhase::Runtime).is_none());
    }

    #[test]
    fn preserves_which_stream_a_group_came_from() {
        let mut grouper = Grouper::new();
        let record = parse_line("[12:00:00] [Server thread/ERROR]: Exception initializing server");
        let category = classify(Stream::Stderr, &record);
        // FatalError buffers, so it won't be emitted until flush.
        assert!(grouper.feed(Stream::Stderr, record, category).is_empty());
        let group = grouper.flush().expect("fatal error should flush");
        assert_eq!(group.stream, Stream::Stderr);
        assert_eq!(group.category, Category::FatalError);
    }

    #[test]
    fn does_not_invent_missing_fields() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Skipping loading recipe minecraft:foo",
        ]);
        let diag = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert!(diag.position.is_none());
        assert!(diag.context.is_none());
    }

    #[test]
    fn classifies_missing_reference_distinctly_from_parse_error() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Unknown function tag arcane:combat/dash",
        ]);
        let diag = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert_eq!(diag.code, DiagnosticCode::MissingReference);
    }

    #[test]
    fn does_not_infer_reload_failure_merely_from_phase() {
        // A JSON/component error observed while the phase happens to be
        // `Reload` (e.g. it fires during a /reload's discovery window) must
        // not be relabeled ReloadFailure just because of the phase — only
        // wording that actually indicates the reload mechanism itself
        // failed should produce that code.
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Error loading recipe arcane:combat/dodge",
        ]);
        let diag = build_diagnostic(&events[0], RunPhase::Reload).unwrap();
        assert_ne!(diag.code, DiagnosticCode::ReloadFailure);
    }

    #[test]
    fn classifies_startup_failure_regardless_of_subsystem() {
        let events =
            group_lines(&["[12:00:00] [Server thread/ERROR]: **** FAILED TO BIND TO PORT!"]);
        let diag = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert_eq!(diag.code, DiagnosticCode::StartupFailure);
    }

    #[test]
    fn same_command_failure_is_runtime_error_after_ready_but_parse_error_before() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Couldn't parse command: execute as @a run dash",
        ]);
        let startup = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert_eq!(startup.code, DiagnosticCode::CommandParseError);

        let runtime = build_diagnostic(&events[0], RunPhase::Runtime).unwrap();
        assert_eq!(runtime.code, DiagnosticCode::RuntimeCommandError);
    }

    #[test]
    fn fingerprint_ignores_incidental_whitespace_but_distinguishes_resource() {
        let a = build_diagnostic(
            &group_lines(&[
                "[12:00:00] [Server thread/WARN]: Error loading function arcane:combat/dash",
            ])[0],
            RunPhase::ServerStartup,
        )
        .unwrap();
        let b = build_diagnostic(
            &group_lines(&[
                "[12:00:05] [Server thread/WARN]: Error loading function arcane:combat/dash",
            ])[0],
            RunPhase::ServerStartup,
        )
        .unwrap();
        let c = build_diagnostic(
            &group_lines(&[
                "[12:00:05] [Server thread/WARN]: Error loading function arcane:combat/dodge",
            ])[0],
            RunPhase::ServerStartup,
        )
        .unwrap();

        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), c.fingerprint());
    }
}
