//! Stateless, single-line classification of Minecraft server log messages.
//!
//! [`classify`] looks only at one [`LogRecord`] at a time and never mutates
//! state; multi-line grouping (stack traces, parser context) lives in
//! [`crate::console::diagnostic`]. Classification is intentionally
//! conservative: if a line isn't confidently recognized as noise, it's
//! shown.

use super::log_record::{LogRecord, Stream};

/// The category a single log line falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Routine noise; hidden by default.
    Suppress,
    /// Server has finished starting up.
    Ready,
    /// Server is shutting down / has shut down.
    Shutdown,
    /// Player join, leave, or disconnect.
    PlayerEvent,
    /// Feedback broadcast from an operator-issued command.
    CommandResponse,
    /// A `WARN`-level line that isn't part of a recognized datapack failure.
    Warning,
    /// A recognized datapack loading/parsing/validation failure.
    DatapackError,
    /// A crash, uncaught exception, or other fatal condition.
    FatalError,
    /// Not specially recognized, but shown by default (conservative default).
    Visible,
}

impl Category {
    /// Whether lines of this category are shown by default (non-verbose mode).
    pub fn visible_by_default(self) -> bool {
        !matches!(self, Category::Suppress)
    }
}

/// Substrings of routine, low-value INFO lines suppressed by default.
/// Matched against the message with the `[HH:MM:SS] [Thread/LEVEL]:` prefix
/// already stripped.
const SUPPRESS_SUBSTRINGS: &[&str] = &[
    "Starting minecraft server version",
    "Starting Minecraft server on",
    "Loading properties",
    "Default game type",
    "Generating keypair",
    "Preparing level \"",
    "Preparing start region",
    "Preparing spawn area:",
    "Time elapsed:",
    "Reloading ResourceManager",
    "Server permissions file",
    "Using epoll channel type",
    "Environment: authHost",
    "Environment: sessionHost",
    "Environment: servicesHost",
    "Environment: name=",
    "Saving chunks for level",
    "Saving players",
    "ThreadedAnvilChunkStorage",
    "This server is running",
    "Flushing Chunk IO",
];

const READY_SUBSTRINGS: &[&str] = &["For help, type \"help\""];

const SHUTDOWN_SUBSTRINGS: &[&str] = &[
    "Stopping the server",
    "Stopping server",
    "Closing Server",
    "Saving worlds",
];

const DATAPACK_ERROR_KEYWORDS: &[&str] = &[
    "couldn't parse command",
    "couldn't load data pack",
    "error loading function",
    "error parsing function",
    "failed to load datapack",
    "failed to load recipes",
    "failed to load loot table",
    "failed to load advancement",
    "couldn't parse loot table",
    "couldn't parse recipe",
    "couldn't parse advancement",
    "couldn't parse tag",
    "error loading recipe",
    "skipping loading recipe",
    "skipping loading advancement",
    "failed execution",
    "invalid function",
    "unbalanced curly brackets",
    // Missing references / unknown registry IDs.
    "unknown function",
    "unknown recipe",
    "unknown advancement",
    "unknown loot table",
    "unknown predicate",
    "unknown tag",
    "unable to resolve",
    "no such function",
    "no function tag",
    // Reload / discovery / pack-format failures.
    "failed to reload",
    "reload failed",
    "failed to load data pack",
    "was designed for a newer version",
    "was designed for an older version",
    "requires format",
    "incompatible pack",
    "duplicate data pack",
];

/// Keywords that indicate the server process itself failed to come up, as
/// opposed to a datapack content problem. These always classify as
/// [`Category::FatalError`] regardless of log level, since a startup
/// failure usually has no `FATAL`-level line at all (Java prints its own
/// unprefixed diagnostics before the server's logger even attaches).
const STARTUP_FAILURE_KEYWORDS: &[&str] = &[
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

const FATAL_KEYWORDS: &[&str] = &[
    "this crash report has been saved to",
    "exception in server tick loop",
    "exception initializing server",
    "encountered an unexpected exception",
];

fn contains_any_ci(haystack: &str, needles: &[&str]) -> bool {
    let lower = haystack.to_ascii_lowercase();
    needles.iter().any(|n| lower.contains(n))
}

/// Classify a single log line. `stream` is provided for context (some
/// distinctions, like fatal crash traces, are more likely on stderr) but no
/// rule here relies on the datapack's own namespace — vanilla can report a
/// related failure on a neighboring line with no namespace at all.
pub fn classify(stream: Stream, record: &LogRecord) -> Category {
    let message = record.message.as_str();
    let level = record.prefix.as_ref().map(|p| p.level.as_str());

    if level == Some("FATAL")
        || contains_any_ci(message, FATAL_KEYWORDS)
        || contains_any_ci(message, STARTUP_FAILURE_KEYWORDS)
    {
        return Category::FatalError;
    }

    if contains_any_ci(message, DATAPACK_ERROR_KEYWORDS) {
        return Category::DatapackError;
    }

    if message.contains("Done (") && message.contains(")!") {
        return Category::Ready;
    }
    if READY_SUBSTRINGS.iter().any(|s| message.contains(s)) {
        return Category::Ready;
    }

    if SHUTDOWN_SUBSTRINGS.iter().any(|s| message.contains(s)) {
        return Category::Shutdown;
    }

    if is_player_event(message) {
        return Category::PlayerEvent;
    }

    if is_command_response(message) {
        return Category::CommandResponse;
    }

    if level == Some("WARN") {
        return Category::Warning;
    }

    if level == Some("INFO") && SUPPRESS_SUBSTRINGS.iter().any(|s| message.contains(s)) {
        return Category::Suppress;
    }

    // Server thread stderr with no recognized prefix at all and no content
    // is not classifiable here; treat conservatively as visible.
    let _ = stream;
    Category::Visible
}

fn is_player_event(message: &str) -> bool {
    message.contains(" joined the game")
        || message.contains(" left the game")
        || message.contains(" lost connection:")
        || message.contains("issued server command:")
}

fn is_command_response(message: &str) -> bool {
    // Broadcast feedback from an op-issued command looks like
    // `[Server: Set the time to 0]` once the log prefix is stripped.
    message.starts_with('[') && message.ends_with(']') && message.len() > 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::log_record::parse_line;

    fn classify_line(line: &str) -> Category {
        classify(Stream::Stdout, &parse_line(line))
    }

    #[test]
    fn suppresses_known_routine_lines() {
        assert_eq!(
            classify_line(
                "[12:00:00] [Server thread/INFO]: Starting minecraft server version 1.21.1"
            ),
            Category::Suppress
        );
        assert_eq!(
            classify_line("[12:00:01] [Server thread/INFO]: Preparing spawn area: 42%"),
            Category::Suppress
        );
    }

    #[test]
    fn unknown_info_lines_stay_visible() {
        assert_eq!(
            classify_line(
                "[12:00:02] [Server thread/INFO]: Some new message future Minecraft added"
            ),
            Category::Visible
        );
    }

    #[test]
    fn warnings_and_errors_are_not_suppressed() {
        assert_eq!(
            classify_line(
                "[12:00:03] [Server thread/WARN]: Can't keep up! Is the server overloaded?"
            ),
            Category::Warning
        );
    }

    #[test]
    fn detects_ready_line() {
        assert_eq!(
            classify_line(
                "[12:00:04] [Server thread/INFO]: Done (3.421s)! For help, type \"help\""
            ),
            Category::Ready
        );
    }

    #[test]
    fn detects_player_join_and_leave() {
        assert_eq!(
            classify_line("[12:00:05] [Server thread/INFO]: Toast joined the game"),
            Category::PlayerEvent
        );
        assert_eq!(
            classify_line("[12:00:06] [Server thread/INFO]: Toast left the game"),
            Category::PlayerEvent
        );
    }

    #[test]
    fn detects_command_response() {
        assert_eq!(
            classify_line("[12:00:07] [Server thread/INFO]: [Server: Set the time to 0]"),
            Category::CommandResponse
        );
    }

    #[test]
    fn detects_datapack_error_without_requiring_namespace() {
        assert_eq!(
            classify_line(
                "[12:00:08] [Server thread/WARN]: Couldn't parse command: execute as @a run foo"
            ),
            Category::DatapackError
        );
    }

    #[test]
    fn generic_errors_remain_visible_when_not_recognized_as_datapack_failures() {
        assert_eq!(
            classify_line(
                "[12:00:10] [Server thread/ERROR]: Unrecognized future feature this classifier doesn't know"
            ),
            Category::Visible
        );
    }

    #[test]
    fn classification_is_consistent_across_stdout_and_stderr() {
        let record =
            parse_line("[12:00:11] [Server thread/WARN]: Can't keep up! Is the server overloaded?");
        assert_eq!(classify(Stream::Stdout, &record), Category::Warning);
        assert_eq!(classify(Stream::Stderr, &record), Category::Warning);
    }

    #[test]
    fn detects_fatal_by_level() {
        assert_eq!(
            classify_line("[12:00:09] [Server thread/FATAL]: Unhandled exception"),
            Category::FatalError
        );
    }

    #[test]
    fn detects_port_in_use_as_fatal_even_without_fatal_level() {
        assert_eq!(
            classify_line("[12:00:12] [Server thread/ERROR]: **** FAILED TO BIND TO PORT!"),
            Category::FatalError
        );
    }

    #[test]
    fn detects_missing_function_reference_as_datapack_error() {
        assert_eq!(
            classify_line(
                "[12:00:13] [Server thread/WARN]: Unknown function tag arcane:combat/dash"
            ),
            Category::DatapackError
        );
    }

    #[test]
    fn detects_incompatible_pack_format_as_datapack_error() {
        assert_eq!(
            classify_line(
                "[12:00:14] [Server thread/WARN]: Data pack \"arcane\" was designed for a newer version of Minecraft"
            ),
            Category::DatapackError
        );
    }
}
