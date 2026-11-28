//! Shared helpers: analyse in-memory packages without touching disk.
#![allow(dead_code)]

use sand_lang::{Analysis, OverlayFiles, Severity};
use std::path::PathBuf;

pub const ROOT: &str = "/virtual/pack.sand";

/// Analyse a package from `(relative path, text)` pairs; the first is the root.
pub fn analyze(files: &[(&str, &str)]) -> Analysis {
    let provider = OverlayFiles::in_memory(
        files
            .iter()
            .map(|(path, text)| (PathBuf::from("/virtual").join(path), text.to_string())),
    );
    sand_lang::analyze_path(&PathBuf::from("/virtual").join(files[0].0), &provider)
}

pub fn analyze_one(text: &str) -> Analysis {
    analyze(&[("pack.sand", text)])
}

/// Error messages, rendered without paths, for compact assertions.
pub fn errors(analysis: &Analysis) -> Vec<String> {
    analysis
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.message.clone())
        .collect()
}

pub fn warnings(analysis: &Analysis) -> Vec<String> {
    analysis
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .map(|d| d.message.clone())
        .collect()
}

/// The text a diagnostic's primary span covers.
pub fn spanned<'a>(analysis: &'a Analysis, message_part: &str) -> &'a str {
    let d = analysis
        .diagnostics
        .iter()
        .find(|d| d.message.contains(message_part))
        .unwrap_or_else(|| {
            panic!(
                "no diagnostic containing {message_part:?}: {:#?}",
                errors(analysis)
            )
        });
    &analysis.package.files[d.file.0].text[d.span.start..d.span.end]
}

/// Assert analysis is clean and compile through the portable backend.
pub fn compile(analysis: &Analysis) -> sand::advanced::compiler::CompiledPack {
    assert!(
        !analysis.has_errors(),
        "unexpected errors: {:#?}",
        errors(analysis)
    );
    match analysis.compile() {
        Ok(pack) => pack,
        Err(errors) => panic!(
            "backend rejected the program: {:#?}",
            errors.iter().map(|e| &e.message).collect::<Vec<_>>()
        ),
    }
}

pub fn function<'a>(pack: &'a sand::advanced::compiler::CompiledPack, path: &str) -> &'a str {
    let key = format!("data/{path}.mcfunction");
    std::str::from_utf8(pack.resources.get(&key).unwrap_or_else(|| {
        panic!(
            "missing {key}; have {:#?}",
            pack.resources.keys().collect::<Vec<_>>()
        )
    }))
    .unwrap()
}
