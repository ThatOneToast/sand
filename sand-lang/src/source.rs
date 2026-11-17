//! Source files, file providers, and position conversion.

use crate::ast::Span;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Index of a file within one loaded [`Package`](crate::Package).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub usize);

/// Supplies source text for paths, so editors can overlay unsaved buffers.
pub trait FileProvider {
    /// Return the current text of `path`, or `None` if it does not exist.
    fn read(&self, path: &Path) -> Option<String>;
}

/// Reads files from disk.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiskFiles;

impl FileProvider for DiskFiles {
    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// Serves in-memory text first and falls back to disk.
///
/// Language servers use this to analyse unsaved editor buffers.
#[derive(Debug, Clone, Default)]
pub struct OverlayFiles {
    /// Open buffers keyed by absolute path.
    pub overlay: HashMap<PathBuf, String>,
    /// When false, paths absent from the overlay are treated as missing.
    pub disk_fallback: bool,
}

impl OverlayFiles {
    /// Create an overlay that falls back to disk.
    pub fn with_disk() -> Self {
        Self {
            overlay: HashMap::new(),
            disk_fallback: true,
        }
    }

    /// Create a purely in-memory provider; useful for tests and embedding.
    pub fn in_memory<I, P, S>(files: I) -> Self
    where
        I: IntoIterator<Item = (P, S)>,
        P: Into<PathBuf>,
        S: Into<String>,
    {
        Self {
            overlay: files
                .into_iter()
                .map(|(path, text)| (path.into(), text.into()))
                .collect(),
            disk_fallback: false,
        }
    }
}

impl FileProvider for OverlayFiles {
    fn read(&self, path: &Path) -> Option<String> {
        if let Some(text) = self.overlay.get(path) {
            return Some(text.clone());
        }
        if self.disk_fallback {
            std::fs::read_to_string(path).ok()
        } else {
            None
        }
    }
}

/// Converts byte offsets to line/column positions and back.
///
/// Columns are reported both as UTF-8 byte offsets (for terminal rendering)
/// and as UTF-16 code units (the Language Server Protocol default).
#[derive(Debug, Clone)]
pub struct LineIndex {
    starts: Vec<usize>,
    text: String,
}

/// A zero-based line and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    /// Zero-based line.
    pub line: u32,
    /// Zero-based column in the requested unit.
    pub col: u32,
}

impl LineIndex {
    /// Index `text`.
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Self {
            starts,
            text: text.to_string(),
        }
    }

    fn line_of(&self, offset: usize) -> usize {
        match self.starts.binary_search(&offset) {
            Ok(line) => line,
            Err(next) => next - 1,
        }
    }

    /// Line and UTF-16 column of a byte offset.
    pub fn utf16(&self, offset: usize) -> LineCol {
        let offset = self.clamp(offset);
        let line = self.line_of(offset);
        let col = self.text[self.starts[line]..offset].encode_utf16().count();
        LineCol {
            line: line as u32,
            col: col as u32,
        }
    }

    /// Line and character (Unicode scalar) column of a byte offset.
    pub fn chars(&self, offset: usize) -> LineCol {
        let offset = self.clamp(offset);
        let line = self.line_of(offset);
        let col = self.text[self.starts[line]..offset].chars().count();
        LineCol {
            line: line as u32,
            col: col as u32,
        }
    }

    /// Byte offset of a line and UTF-16 column, clamped to the line.
    pub fn offset_utf16(&self, pos: LineCol) -> usize {
        let Some(&start) = self.starts.get(pos.line as usize) else {
            return self.text.len();
        };
        let end = self
            .starts
            .get(pos.line as usize + 1)
            .map(|next| next - 1)
            .unwrap_or(self.text.len());
        let mut units = 0u32;
        for (i, c) in self.text[start..end].char_indices() {
            if units >= pos.col {
                return start + i;
            }
            units += c.len_utf16() as u32;
        }
        end
    }

    /// The text of a zero-based line without its terminator.
    pub fn line_text(&self, line: usize) -> &str {
        let start = self.starts.get(line).copied().unwrap_or(self.text.len());
        let end = self
            .starts
            .get(line + 1)
            .map(|next| next - 1)
            .unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches('\r')
    }

    fn clamp(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// Clamp a span to the indexed text.
    pub fn clamp_span(&self, span: Span) -> Span {
        Span::new(self.clamp(span.start), self.clamp(span.end.max(span.start)))
    }
}
