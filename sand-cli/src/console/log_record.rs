//! Parsing of raw Minecraft server log lines into a structured [`LogRecord`].

/// Which pipe a line came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stream {
    Stdout,
    Stderr,
}

/// The `[HH:MM:SS] [Thread/LEVEL]:` prefix Minecraft's log4j setup writes on
/// the first line of every log message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefix {
    pub timestamp: String,
    pub thread: String,
    pub level: String,
}

/// A single line of output from the server process, with its prefix (if any)
/// separated from the message body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    /// The original, unmodified line (sans trailing newline). Only ever
    /// written to the terminal in `--verbose` mode, which is an explicit,
    /// opt-in trust of the raw server log.
    pub raw: String,
    /// The parsed `[HH:MM:SS] [Thread/LEVEL]:` prefix, when recognized.
    pub prefix: Option<Prefix>,
    /// `raw` with the prefix (if any) stripped and terminal control
    /// characters (e.g. a stray ESC starting an ANSI escape sequence)
    /// removed. This is what Sand's filtered console prints by default, so
    /// server output can't inject terminal control sequences.
    pub message: String,
}

/// Parse a raw line into a [`LogRecord`], stripping the standard Minecraft
/// log prefix when present. Lines that don't match the expected shape are
/// preserved verbatim as an unprefixed record — we never drop or mangle
/// content we don't recognize.
pub fn parse_line(raw: &str) -> LogRecord {
    match parse_prefix(raw) {
        Some((prefix, message)) => LogRecord {
            raw: raw.to_string(),
            prefix: Some(prefix),
            message: sanitize(message),
        },
        None => LogRecord {
            raw: raw.to_string(),
            prefix: None,
            message: sanitize(raw),
        },
    }
}

/// Strips ASCII control characters (including ESC, which starts ANSI escape
/// sequences) other than plain tab, so untrusted server output can never
/// smuggle terminal control sequences into Sand's filtered console.
fn sanitize(s: &str) -> String {
    if s.chars().all(|c| c == '\t' || !c.is_control()) {
        return s.to_string();
    }
    s.chars()
        .filter(|&c| c == '\t' || !c.is_control())
        .collect()
}

/// Attempt to parse a leading `[12:34:56] [Server thread/INFO]: ` prefix.
///
/// Returns the parsed prefix and the remainder of the line (the message)
/// with the prefix stripped, or `None` if the line doesn't start with a
/// recognizable prefix.
fn parse_prefix(line: &str) -> Option<(Prefix, &str)> {
    let rest = line.strip_prefix('[')?;
    let (timestamp, rest) = rest.split_once(']')?;
    let timestamp = timestamp.trim();
    if timestamp.len() != 8 || timestamp.as_bytes()[2] != b':' || timestamp.as_bytes()[5] != b':' {
        return None;
    }
    if !timestamp.bytes().enumerate().all(|(i, b)| match i {
        2 | 5 => b == b':',
        _ => b.is_ascii_digit(),
    }) {
        return None;
    }

    let rest = rest.trim_start();
    let rest = rest.strip_prefix('[')?;
    let (thread_level, rest) = rest.split_once(']')?;
    let (thread, level) = thread_level.rsplit_once('/')?;
    if thread.is_empty() || level.is_empty() {
        return None;
    }

    let rest = rest.strip_prefix(':')?;
    let message = rest.strip_prefix(' ').unwrap_or(rest);

    Some((
        Prefix {
            timestamp: timestamp.to_string(),
            thread: thread.to_string(),
            level: level.to_string(),
        },
        message,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_prefix() {
        let rec = parse_line("[12:34:56] [Server thread/INFO]: Done (1.234s)!");
        let prefix = rec.prefix.expect("prefix should parse");
        assert_eq!(prefix.timestamp, "12:34:56");
        assert_eq!(prefix.thread, "Server thread");
        assert_eq!(prefix.level, "INFO");
        assert_eq!(rec.message, "Done (1.234s)!");
    }

    #[test]
    fn leaves_unrecognized_lines_alone() {
        let rec =
            parse_line("\tat net.minecraft.server.MinecraftServer.run(MinecraftServer.java:1234)");
        assert!(rec.prefix.is_none());
        assert_eq!(
            rec.message,
            "\tat net.minecraft.server.MinecraftServer.run(MinecraftServer.java:1234)"
        );
    }

    #[test]
    fn handles_worker_thread_names_with_slash_free_content() {
        let rec = parse_line("[12:34:56] [Server-Worker-3/WARN]: Something happened");
        let prefix = rec.prefix.unwrap();
        assert_eq!(prefix.thread, "Server-Worker-3");
        assert_eq!(prefix.level, "WARN");
    }

    #[test]
    fn does_not_panic_on_malformed_input() {
        for line in [
            "",
            "[",
            "[]",
            "[12:34:56]",
            "[12:34:56] [",
            "not a log line at all",
            "[abc] [def/GHI]: msg",
        ] {
            let rec = parse_line(line);
            assert_eq!(rec.raw, line);
        }
    }

    #[test]
    fn strips_ansi_escape_sequences_from_message_but_keeps_raw_intact() {
        let malicious = "[12:34:56] [Server thread/INFO]: \u{1b}[31mfake danger\u{1b}[0m done";
        let rec = parse_line(malicious);
        assert!(!rec.message.contains('\u{1b}'));
        assert_eq!(rec.message, "[31mfake danger[0m done");
        // `raw` is preserved verbatim for verbose mode, which is an
        // explicit opt-in to trusting the server's raw log.
        assert_eq!(rec.raw, malicious);
    }

    #[test]
    fn non_utf8_safe_input_is_handled_upstream() {
        // parse_line only ever receives already-lossily-decoded &str, so it
        // just needs to not panic on arbitrary replacement-character input.
        let rec = parse_line("[12:34:56] [Server thread/INFO]: bad bytes \u{FFFD}\u{FFFD}");
        assert!(rec.message.contains('\u{FFFD}'));
    }
}
