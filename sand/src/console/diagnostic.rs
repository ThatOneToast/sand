//! Multi-line grouping and datapack diagnostic extraction.
//!
//! Minecraft's log4j setup only stamps the `[HH:MM:SS] [Thread/LEVEL]:`
//! prefix on the *first* line of a log message; stack frames, parser
//! context, and carets that follow are emitted as bare, unprefixed lines.
//! [`Grouper`] uses that fact to fold a headline together with its
//! continuation lines without depending on any single Minecraft version's
//! exact wording.

use super::classify::Category;
use super::log_record::{LogRecord, Stream};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// A compact, actionable summary of a recognized datapack failure, built
/// from a [`GroupedEvent`]. Every field is `Option` (except `severity` and
/// `reason`) because we only ever report what Minecraft's own output
/// supports — we never invent a path, position, or hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
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

/// Build a [`Diagnostic`] from a grouped [`DatapackError`]/[`FatalError`]
/// event. Returns `None` for other categories.
///
/// [`DatapackError`]: Category::DatapackError
/// [`FatalError`]: Category::FatalError
pub fn build_diagnostic(group: &GroupedEvent) -> Option<Diagnostic> {
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

    Some(Diagnostic {
        severity,
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

        let diag = build_diagnostic(&events[0]).unwrap();
        assert_eq!(diag.resource.as_deref(), Some("arcane:combat/dash"));
        assert_eq!(diag.subsystem.as_deref(), Some("function"));
        assert_eq!(
            diag.file.as_deref(),
            Some("dist/arcane/data/arcane/function/combat/dash.mcfunction")
        );
    }

    #[test]
    fn extracts_position_and_context_from_multiline_group() {
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Function arcane:combat/dash failed execution due to: command 4 invalid",
            "execute as @a run particle minecraft:cloud ~ ~ ~ 0 0 0 1 5",
        ]);
        let diag = build_diagnostic(&events[0]).unwrap();
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
        assert!(build_diagnostic(&events[0]).is_none());
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
        let diag = build_diagnostic(&events[0]).unwrap();
        assert!(diag.position.is_none());
        assert!(diag.context.is_none());
    }
}
