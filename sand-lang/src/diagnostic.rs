//! Located diagnostics and their terminal rendering.

use crate::Package;
use crate::ast::Span;
use crate::source::FileId;
use std::fmt::Write as _;

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Compilation cannot produce a pack.
    Error,
    /// Suspicious but compilable source.
    Warning,
}

/// A secondary location that explains a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// File of the related location.
    pub file: FileId,
    /// Related location.
    pub span: Span,
    /// Explanation of why the location matters.
    pub message: String,
}

/// A problem located in Sand source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable machine-readable category, such as `syntax` or `context`.
    pub code: String,
    /// Severity.
    pub severity: Severity,
    /// Primary file.
    pub file: FileId,
    /// Primary location.
    pub span: Span,
    /// Human-readable explanation.
    pub message: String,
    /// Related locations.
    pub notes: Vec<Note>,
    /// Suggested fix.
    pub help: Option<String>,
}

impl Diagnostic {
    /// An error at `span` in `file`.
    pub fn error(code: &str, file: FileId, span: Span, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Error,
            file,
            span,
            message: message.into(),
            notes: Vec::new(),
            help: None,
        }
    }

    /// A warning at `span` in `file`.
    pub fn warning(code: &str, file: FileId, span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, file, span, message)
        }
    }

    /// Attach a related location.
    pub fn with_note(mut self, file: FileId, span: Span, message: impl Into<String>) -> Self {
        self.notes.push(Note {
            file,
            span,
            message: message.into(),
        });
        self
    }

    /// Attach a suggested fix.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render the diagnostic rustc-style with a source excerpt.
    pub fn render(&self, package: &Package) -> String {
        let mut out = String::new();
        let label = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let _ = writeln!(out, "{label}[{}]: {}", self.code, self.message);
        render_location(&mut out, package, self.file, self.span, None);
        for note in &self.notes {
            render_location(&mut out, package, note.file, note.span, Some(&note.message));
        }
        if let Some(help) = &self.help {
            let _ = writeln!(out, "  = help: {help}");
        }
        out
    }
}

fn render_location(
    out: &mut String,
    package: &Package,
    file: FileId,
    span: Span,
    note: Option<&str>,
) {
    let Some(source) = package.files.get(file.0) else {
        return;
    };
    let index = &source.lines;
    let span = index.clamp_span(span);
    let start = index.chars(span.start);
    let end = index.chars(span.end);
    let _ = writeln!(
        out,
        "  --> {}:{}:{}",
        source.path.display(),
        start.line + 1,
        start.col + 1
    );
    let line_text = index.line_text(start.line as usize);
    let gutter = (start.line + 1).to_string();
    let pad = " ".repeat(gutter.len());
    let width = if end.line == start.line {
        (end.col.saturating_sub(start.col)).max(1)
    } else {
        (line_text.chars().count() as u32)
            .saturating_sub(start.col)
            .max(1)
    };
    let _ = writeln!(out, "{pad} |");
    let _ = writeln!(out, "{gutter} | {line_text}");
    let _ = writeln!(
        out,
        "{pad} | {}{} {}",
        " ".repeat(start.col as usize),
        "^".repeat(width as usize),
        note.unwrap_or("")
    );
}
