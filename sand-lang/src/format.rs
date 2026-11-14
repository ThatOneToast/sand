//! Deterministic source formatting shared by the CLI and language server.
//!
//! Formatting changes whitespace only. Strings and comments are kept verbatim,
//! and malformed files are rejected so recovery nodes never lose source text.

use crate::ast::SyntaxError;

/// Format one complete source file with four-space indentation and LF endings.
///
/// This accepts modules as well as package roots. It needs no successful name
/// resolution, disk access, or Minecraft installation. Syntax errors leave the
/// caller's original buffer untouched.
pub fn format(source: &str) -> Result<String, Vec<SyntaxError>> {
    let (_, errors) = crate::parser::parse(source);
    if !errors.is_empty() {
        return Err(errors);
    }
    let tokens = tokens(source);
    let mut out = String::new();
    let mut stack: Vec<&str> = Vec::new();
    let mut indent = 0usize;
    let mut attribute = false;
    let mut unary = false;
    for (i, token) in tokens.iter().copied().enumerate() {
        let prev = i.checked_sub(1).map(|i| tokens[i]).unwrap_or("");
        let next = tokens.get(i + 1).copied().unwrap_or("");
        if token.starts_with("//") || token.starts_with("/*") {
            if !out.ends_with('\n') && !out.is_empty() {
                space(&mut out);
            }
            write(&mut out, token, indent);
            newline(&mut out);
            continue;
        }
        match token {
            "#" => {
                newline(&mut out);
                write(&mut out, token, indent);
                attribute = true;
            }
            "{" => {
                space(&mut out);
                write(&mut out, token, indent);
                stack.push(token);
                indent += 1;
                if next != "}" {
                    newline(&mut out);
                }
            }
            "}" => {
                indent = indent.saturating_sub(1);
                if prev != "{" {
                    newline(&mut out);
                }
                write(&mut out, token, indent);
                stack.pop();
                if next == "else" {
                    space(&mut out);
                } else if !matches!(next, "," | ";" | ")" | "]" | "}") {
                    newline(&mut out);
                }
            }
            "(" => {
                if matches!(prev, "if" | "while" | "match" | "return" | "in") {
                    space(&mut out);
                }
                write(&mut out, token, indent);
                stack.push(token);
            }
            "[" => {
                write(&mut out, token, indent);
                stack.push(if attribute { "#[" } else { "[" });
                attribute = false;
            }
            ")" | "]" => {
                trim_spaces(&mut out);
                write(&mut out, token, indent);
                if stack.pop() == Some("#[") {
                    newline(&mut out);
                }
            }
            ";" => {
                trim_spaces(&mut out);
                write(&mut out, token, indent);
                newline(&mut out);
            }
            "," => {
                trim_spaces(&mut out);
                write(&mut out, token, indent);
                if stack.last() == Some(&"{") {
                    newline(&mut out);
                } else {
                    space(&mut out);
                }
            }
            ":" => {
                trim_spaces(&mut out);
                write(&mut out, ":", indent);
                space(&mut out);
            }
            "." | "::" | ".." | "..=" => {
                trim_spaces(&mut out);
                write(&mut out, token, indent);
            }
            "!" => {
                if matches!(prev, "if" | "while" | "return") {
                    space(&mut out);
                }
                write(&mut out, token, indent);
            }
            "-" if is_prefix(prev) => {
                if word(prev) {
                    space(&mut out);
                }
                write(&mut out, token, indent);
                unary = true;
            }
            "+" | "-" | "*" | "/" | "%" | "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "==" | "!="
            | "<" | ">" | "<=" | ">=" | "&&" | "||" | "|" | "->" | "=>" => {
                space(&mut out);
                write(&mut out, token, indent);
                space(&mut out);
            }
            _ => {
                if !unary && (word(prev) || matches!(prev, ")" | "]" | "}")) {
                    space(&mut out);
                }
                write(&mut out, token, indent);
                unary = false;
            }
        }
    }
    newline(&mut out);
    Ok(out)
}

fn word(token: &str) -> bool {
    token
        .chars()
        .last()
        .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '"')
}

fn is_prefix(token: &str) -> bool {
    token.is_empty()
        || matches!(
            token,
            "(" | "["
                | "{"
                | ","
                | ":"
                | "="
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "+"
                | "-"
                | "*"
                | "/"
                | "%"
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "&&"
                | "||"
                | "|"
                | "=>"
                | ".."
                | "..="
                | "return"
                | "in"
        )
}

fn trim_spaces(out: &mut String) {
    while out.ends_with(' ') {
        out.pop();
    }
}

fn space(out: &mut String) {
    if !out.is_empty() && !out.ends_with([' ', '\n']) {
        out.push(' ');
    }
}

fn newline(out: &mut String) {
    trim_spaces(out);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

fn write(out: &mut String, text: &str, indent: usize) {
    if out.is_empty() || out.ends_with('\n') {
        out.push_str(&"    ".repeat(indent));
    }
    out.push_str(text);
}

// Lexemes are slices of the original file, so even escaped strings, Unicode,
// nested block comments, and raw commands remain byte-identical.
fn tokens(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if source[i..].starts_with("//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if source[i..].starts_with("/*") {
            i += 2;
            let mut depth = 1;
            while i < bytes.len() && depth > 0 {
                if source[i..].starts_with("/*") {
                    depth += 1;
                    i += 2;
                } else if source[i..].starts_with("*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += source[i..].chars().next().unwrap().len_utf8();
                }
            }
        } else if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                match bytes[i] {
                    b'\\' => i += 2,
                    b'"' => {
                        i += 1;
                        break;
                    }
                    _ => i += 1,
                }
            }
        } else if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            if bytes[start].is_ascii_digit()
                && bytes.get(i) == Some(&b'.')
                && bytes.get(i + 1).is_some_and(u8::is_ascii_digit)
            {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
        } else if let Some(op) = [
            "..=", "::", "..", "->", "=>", "+=", "-=", "*=", "/=", "%=", "==", "!=", "<=", ">=",
            "&&", "||",
        ]
        .iter()
        .find(|op| source[i..].starts_with(**op))
        {
            i += op.len();
        } else {
            i += source[i..].chars().next().unwrap().len_utf8();
        }
        out.push(&source[start..i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_blocks_and_preserves_literals_comments_and_attributes() {
        let source = "//! Pack\n#![pack(namespace=\"demo\")]\n// café 🦀\nfn sum(mut x:number)->number{ /* outer /* nested */ kept */ x+=-2;return x;}";
        let result = format(source).unwrap();
        assert!(
            result.contains("fn sum(mut x: number) -> number {\n"),
            "{result}"
        );
        assert!(
            result.contains("    x += -2;\n    return x;\n}"),
            "{result}"
        );
        assert!(result.contains("/* outer /* nested */ kept */"));
        assert!(result.contains("// café 🦀"));
        assert_eq!(format(&result).unwrap(), result);
    }

    #[test]
    fn formats_real_projects_without_changing_tokens_or_emitted_packs() {
        for source in [
            include_str!("../tests/fixtures/control_flow.sand"),
            include_str!("../../examples/lang_encounters/pack.sand"),
        ] {
            let result = format(source).unwrap();
            assert_eq!(tokens(source), tokens(&result));
            assert_eq!(format(&result).unwrap(), result);
            let provider = |text: String| {
                crate::OverlayFiles::in_memory([(std::path::PathBuf::from("/pack.sand"), text)])
            };
            let before =
                crate::analyze_path(std::path::Path::new("/pack.sand"), &provider(source.into()));
            let after = crate::analyze_path(std::path::Path::new("/pack.sand"), &provider(result));
            assert_eq!(before.has_errors(), after.has_errors());
            if !before.has_errors() {
                assert_eq!(
                    before.compile().unwrap().resources,
                    after.compile().unwrap().resources
                );
            }
        }
    }

    #[test]
    fn malformed_files_are_never_rewritten() {
        assert!(format("fn broken( {").is_err());
    }
}
