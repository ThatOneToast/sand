//! Where the cursor is, syntactically, for completion.
//!
//! Classification is lexical: comments and string contents are masked, then
//! enclosing delimiters and the current statement prefix are read from the
//! text before the cursor. It never needs a successful parse, so it stays
//! accurate while the user is in the middle of typing.

/// An enclosing block or list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Scope {
    /// File or `mod name { ... }` level.
    Items,
    /// A function body; `returns` when declared `-> T`.
    Function { returns: bool },
    /// A flow hook's inline body.
    Hook,
    /// `if`, `else`, or `while` block.
    Block,
    /// `for name in players()` / `entities(..)` body.
    Loop { entities: bool },
    /// `player state Name { ... }`.
    State,
    /// `enum Name { ... }`.
    Enum,
    /// `flow Name on State.field { ... }`, with the field text.
    Flow { field: String },
    /// A data declaration body: keyword and, for tags, the grouped type.
    Data {
        keyword: &'static str,
        tag_type: Option<&'static str>,
    },
    /// A nested `{ ... }` inside a data declaration (such as `key: {`).
    DataObject {
        keyword: &'static str,
        field: String,
    },
    /// `use a::{ ... }`.
    UseGroup,
    /// `fn name( ... )` parameter list.
    Params,
    /// A call's argument list, with the callee text.
    Call { callee: String },
    /// `[ ... ]` inside a data declaration, with the field.
    DataArray {
        keyword: &'static str,
        tag_type: Option<&'static str>,
        field: String,
    },
    /// Something else, such as an attribute argument list.
    Other,
}

/// What may be written at the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Position {
    /// Nothing can be suggested: a comment, a plain string, or a new name.
    Nothing,
    /// `#[` or `#![`: an attribute name for what follows.
    Attribute { inner: bool, target: AttrTarget },
    /// Inside `#[name(` ...: the attribute and an optional `key =`.
    AttributeArg { attr: String, key: Option<String> },
    /// `{` inside a text string.
    Interpolation,
    /// The start of an item; `after_pub` once `pub` is written.
    ItemStart { after_pub: bool },
    /// After `player` or `global`: only `state`.
    StateKeyword,
    /// After `flow Name`: only `on`.
    FlowOn,
    /// After `flow Name on`: a State.
    FlowField,
    /// A `use` path.
    UsePath,
    /// A score type: `number`, `float`, `bool`, or an enum.
    ScoreType,
    /// The type of a State field: a score type or `timer`.
    FieldType,
    /// The grouped type of a tag: `Item`, `Block`, or `Entity`.
    TagType,
    /// A constant expression: const initializers, defaults, discriminants.
    ConstValue,
    /// The start of a statement in a body.
    Statement(Body),
    /// A value in a body.
    Expression(Body),
    /// The start of a condition: values and predicates such as `has_state`.
    Condition(Body),
    /// After `for name`: only `in`.
    ForIn,
    /// After `for name in`: an iterator source.
    ForSource,
    /// After `}` and `else`: only `if` or a block.
    AfterElse,
    /// A field of a State body.
    StateField,
    /// The start of a flow body entry, with the flow's field.
    FlowEntry { field: String },
    /// A phase in a flow (after `->`, `enter`, `exit`, `tick`).
    FlowPhase { field: String },
    /// A function after `=>` in a flow hook.
    FlowFunction,
    /// A data declaration body or value: the text of the current entry and
    /// the fields already written.
    Data {
        keyword: &'static str,
        tag_type: Option<&'static str>,
        line: String,
        used: Vec<String>,
    },
}

/// The enclosing body of a statement or expression.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Body {
    /// The function declares a return value.
    pub returns: bool,
    /// Innermost executor loop: `Some(true)` for `entities`, `Some(false)` for `players`.
    pub executor_loop: Option<bool>,
    /// Inside a flow hook body.
    pub hook: bool,
}

/// What an attribute is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttrTarget {
    Function,
    ExternFunction,
    Field,
    Other,
}

/// Replace comment and string contents with spaces, keeping quotes and
/// byte offsets, so delimiters inside them are ignored.
fn mask(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    out[i] = b' ';
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                while i < bytes.len() && !(bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/')) {
                    if bytes[i] != b'\n' {
                        out[i] = b' ';
                    }
                    i += 1;
                }
                for _ in 0..2 {
                    if i < bytes.len() {
                        out[i] = b' ';
                        i += 1;
                    }
                }
            }
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' && bytes[i] != b'\n' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        out[i] = b' ';
                        i += 1;
                    }
                    // Keep interpolation braces so `{` inside text is visible.
                    if !matches!(bytes[i], b'{' | b'}') {
                        out[i] = b' ';
                    }
                    i += 1;
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Whether the cursor is inside a comment or string, and the string text.
fn literal_at(text: &str) -> Literal {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut state = Literal::Code;
    while i < bytes.len() {
        state = match state {
            Literal::Code => match bytes[i] {
                b'/' if bytes.get(i + 1) == Some(&b'/') => Literal::LineComment,
                b'/' if bytes.get(i + 1) == Some(&b'*') => {
                    i += 1;
                    Literal::BlockComment
                }
                b'"' => Literal::String(i + 1),
                _ => Literal::Code,
            },
            Literal::LineComment if bytes[i] == b'\n' => Literal::Code,
            Literal::BlockComment if bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/') => {
                i += 1;
                Literal::Code
            }
            Literal::String(_) if bytes[i] == b'\\' => {
                i += 1;
                state
            }
            Literal::String(_) if bytes[i] == b'"' || bytes[i] == b'\n' => Literal::Code,
            other => other,
        };
        i += 1;
    }
    state
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Literal {
    Code,
    LineComment,
    BlockComment,
    /// A string whose contents start at this byte.
    String(usize),
}

/// The keyword starting an item header, skipping `pub`.
fn header_keyword(header: &str) -> (&str, &str) {
    let header = header.trim_start();
    let header = header
        .strip_prefix("pub ")
        .map(str::trim_start)
        .unwrap_or(header);
    let word_end = header
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(header.len());
    (&header[..word_end], header[word_end..].trim_start())
}

/// Remove attributes and their arguments from a statement prefix.
fn strip_attributes(prefix: &str) -> &str {
    let mut rest = prefix.trim_start();
    while rest.starts_with('#') {
        let Some(close) = rest.find(']') else {
            return rest;
        };
        rest = rest[close + 1..].trim_start();
    }
    rest
}

/// The last statement boundary at depth zero within `text`, scanning back.
fn statement_start(masked: &str, from: usize, list: bool) -> usize {
    let bytes = masked.as_bytes();
    let mut depth = 0i32;
    let mut i = bytes.len();
    while i > from {
        i -= 1;
        match bytes[i] {
            b')' | b']' | b'}' if depth >= 0 => {
                if bytes[i] == b'}' && depth == 0 {
                    return i + 1;
                }
                depth += 1;
            }
            b'(' | b'[' | b'{' => {
                if depth == 0 {
                    return i + 1;
                }
                depth -= 1;
            }
            b';' if depth == 0 => return i + 1,
            b',' if depth == 0 && list => return i + 1,
            _ => {}
        }
    }
    from
}

fn tag_type(header: &str) -> Option<&'static str> {
    let ty = header.split_once(':')?.1.trim();
    let ty = ty.split(|c: char| !c.is_ascii_alphanumeric()).next()?;
    match ty {
        "Item" => Some("Item"),
        "Block" => Some("Block"),
        "Entity" => Some("Entity"),
        _ => None,
    }
}

fn data_keyword(word: &str) -> Option<&'static str> {
    ["item", "recipe", "tag", "bossbar", "loot"]
        .into_iter()
        .find(|k| *k == word)
}

/// Classify one enclosing delimiter from the text before it.
fn scope_of(masked: &str, open: usize, kind: u8, parents: &[Scope]) -> Scope {
    let header_start = statement_start(&masked[..open], 0, kind == b'(' || kind == b'[');
    let header = strip_attributes(&masked[header_start..open]).trim();
    let parent = parents.last();
    match kind {
        b'(' => {
            let (word, rest) = header_keyword(header);
            if word == "fn" && !rest.contains('(') {
                return Scope::Params;
            }
            if header.starts_with('#') {
                return Scope::Other;
            }
            let callee = header
                .rsplit(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '!'))
                .next()
                .unwrap_or("")
                .trim_end_matches('!')
                .to_string();
            Scope::Call { callee }
        }
        b'[' => match parent {
            Some(Scope::Data { keyword, tag_type })
            | Some(Scope::DataArray {
                keyword, tag_type, ..
            }) => {
                let field = header.trim_end_matches(':').trim().to_string();
                let field = field
                    .rsplit(':')
                    .next()
                    .unwrap_or(&field)
                    .trim()
                    .to_string();
                Scope::DataArray {
                    keyword,
                    tag_type: *tag_type,
                    field: if field.is_empty() {
                        match parent {
                            Some(Scope::DataArray { field, .. }) => field.clone(),
                            _ => String::new(),
                        }
                    } else {
                        field
                    },
                }
            }
            _ => Scope::Other,
        },
        _ => {
            if let Some(Scope::Data { keyword, .. } | Scope::DataObject { keyword, .. }) = parent {
                let field = header.trim_end_matches(':').trim().to_string();
                return Scope::DataObject { keyword, field };
            }
            let (word, rest) = header_keyword(header);
            match word {
                "fn" => Scope::Function {
                    returns: rest.contains("->"),
                },
                "player" | "global" | "entity" => Scope::State,
                "enum" => Scope::Enum,
                "flow" => Scope::Flow {
                    field: rest
                        .split_once(" on ")
                        .map(|(_, field)| field.trim().to_string())
                        .unwrap_or_default(),
                },
                "mod" => Scope::Items,
                "use" => Scope::UseGroup,
                "for" => Scope::Loop {
                    entities: rest.contains("entities"),
                },
                "if" | "else" | "while" => Scope::Block,
                "enter" | "exit" | "tick" if matches!(parent, Some(Scope::Flow { .. })) => {
                    Scope::Hook
                }
                word => match data_keyword(word) {
                    Some(keyword) => Scope::Data {
                        keyword,
                        tag_type: tag_type(rest),
                    },
                    None if header.starts_with('}') || header.is_empty() => Scope::Block,
                    None => Scope::Other,
                },
            }
        }
    }
}

/// Classify the cursor position.
pub(crate) fn classify(text: &str, offset: usize) -> Position {
    let before = &text[..offset.min(text.len())];
    match literal_at(before) {
        Literal::LineComment | Literal::BlockComment => return Position::Nothing,
        Literal::String(start) => {
            let content = &before[start..];
            let open = content.rfind('{');
            let close = content.rfind('}');
            let interpolating = open.is_some_and(|o| close.is_none_or(|c| c < o))
                && !content[open.unwrap_or(0)..].starts_with("{{");
            return if interpolating {
                Position::Interpolation
            } else {
                Position::Nothing
            };
        }
        Literal::Code => {}
    }
    let masked = mask(before);

    // Attributes are decided by the current line alone.
    let line_start = masked.rfind('\n').map(|n| n + 1).unwrap_or(0);
    let line = masked[line_start..].trim_start();
    if (line.starts_with("#[") || line.starts_with("#![")) && !line.contains(']') {
        let inner = line.starts_with("#![");
        let body = &line[if inner { 3 } else { 2 }..];
        if let Some(paren) = body.find('(') {
            let attr = body[..paren].trim().to_string();
            let args = &body[paren + 1..];
            let current = args.rsplit(',').next().unwrap_or("");
            let key = current
                .split_once('=')
                .map(|(key, _)| key.trim().to_string());
            return Position::AttributeArg { attr, key };
        }
        let target = if inner {
            AttrTarget::Other
        } else {
            attribute_target(&text[offset.min(text.len())..], &masked)
        };
        return Position::Attribute { inner, target };
    }

    // Enclosing delimiters, outermost first.
    let mut stack: Vec<(usize, u8)> = Vec::new();
    for (i, b) in masked.bytes().enumerate() {
        match b {
            b'{' | b'(' | b'[' => stack.push((i, b)),
            b'}' | b')' | b']' => {
                stack.pop();
            }
            _ => {}
        }
    }
    let mut scopes = vec![Scope::Items];
    for (open, kind) in &stack {
        let scope = scope_of(&masked, *open, *kind, &scopes);
        scopes.push(scope);
    }
    let innermost = scopes.last().cloned().unwrap_or(Scope::Items);
    let from = stack.last().map(|(open, _)| open + 1).unwrap_or(0);
    let list = matches!(
        innermost,
        Scope::State
            | Scope::Enum
            | Scope::Params
            | Scope::Call { .. }
            | Scope::UseGroup
            | Scope::Data { .. }
            | Scope::DataObject { .. }
            | Scope::DataArray { .. }
            | Scope::Other
    );
    let start = statement_start(&masked, from, list);
    let raw_prefix = &masked[start..];
    let prefix = strip_attributes(raw_prefix);
    // A word touching the cursor is still being typed and is not context;
    // a completed word followed by whitespace is.
    let typing = raw_prefix.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_');
    let prefix = squeeze(prefix);
    let context = if typing {
        prefix
            .trim_end_matches(|c: char| c.is_ascii_alphanumeric() || c == '_')
            .trim_end()
            .to_string()
    } else {
        prefix
    };

    // A fresh line after an unfinished statement (say, a missing `;`) starts
    // a new statement when the previous line cannot continue.
    let on_fresh_line = before
        .rsplit('\n')
        .next()
        .is_some_and(|line| line.trim().is_empty())
        && before.contains('\n');
    let context = if on_fresh_line
        && !context.is_empty()
        && !context.ends_with([
            '=', '+', '-', '*', '/', '%', '<', '>', '!', '&', '|', '(', ',', ':', '.',
        ])
        && !matches!(innermost, Scope::Call { .. } | Scope::Params | Scope::Other)
    {
        String::new()
    } else {
        context
    };

    let body = || {
        let mut body = Body::default();
        for scope in scopes.iter().rev() {
            match scope {
                Scope::Loop { entities } if body.executor_loop.is_none() => {
                    body.executor_loop = Some(*entities)
                }
                Scope::Function { returns } => {
                    body.returns = *returns;
                    break;
                }
                Scope::Hook => {
                    body.hook = true;
                    break;
                }
                _ => {}
            }
        }
        body
    };

    match &innermost {
        Scope::Items => items_position(&context),
        Scope::Function { .. } | Scope::Hook | Scope::Block | Scope::Loop { .. } => {
            statement_position(&context, body())
        }
        Scope::Call { .. } => Position::Expression(body()),
        Scope::Params => {
            if context.ends_with(':') {
                Position::ScoreType
            } else {
                Position::Nothing
            }
        }
        Scope::State => {
            if context.ends_with('=') {
                Position::ConstValue
            } else if context.ends_with(':') {
                Position::FieldType
            } else if context.is_empty() {
                Position::StateField
            } else {
                Position::Nothing
            }
        }
        Scope::Enum => {
            if context.ends_with('=') {
                Position::ConstValue
            } else {
                Position::Nothing
            }
        }
        Scope::Flow { field } => flow_position(&context, field),
        Scope::Data { keyword, tag_type } => Position::Data {
            keyword,
            tag_type: *tag_type,
            line: raw_prefix.trim_start().to_string(),
            used: used_fields(&masked[from..start]),
        },
        Scope::DataObject { keyword, field } => Position::Data {
            keyword,
            tag_type: None,
            // Values inside `key: { A: ... }` complete like the field itself.
            line: if context.ends_with(':') {
                format!("{field}: ")
            } else {
                String::new()
            },
            used: Vec::new(),
        },
        Scope::DataArray {
            keyword,
            tag_type,
            field,
        } => Position::Data {
            keyword,
            tag_type: *tag_type,
            line: format!("{field}: "),
            used: Vec::new(),
        },
        Scope::UseGroup => Position::UsePath,
        Scope::Other => Position::Nothing,
    }
}

/// Field names of the entries in a data body before the current one.
fn used_fields(body: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut depth = 0i32;
    let mut entry = String::new();
    for c in body.chars() {
        match c {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                if let Some((key, _)) = entry.split_once(':') {
                    fields.push(key.trim().trim_matches('"').to_string());
                }
                entry.clear();
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            entry.push(c);
        }
    }
    fields
}

/// Collapse runs of whitespace (including newlines) to single spaces.
fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn items_position(context: &str) -> Position {
    let tokens: Vec<&str> = context.split(' ').filter(|t| !t.is_empty()).collect();
    match tokens.as_slice() {
        [] => Position::ItemStart { after_pub: false },
        ["pub"] => Position::ItemStart { after_pub: true },
        ["player" | "global" | "entity"] | ["pub", "player" | "global" | "entity"] => {
            Position::StateKeyword
        }
        ["use", ..] | ["pub", "use", ..] => Position::UsePath,
        ["flow", _] | ["pub", "flow", _] => Position::FlowOn,
        ["flow", _, "on"] | ["pub", "flow", _, "on"] => Position::FlowField,
        _ => {
            if context.ends_with("->") {
                return Position::ScoreType;
            }
            let (word, _) = header_keyword(context);
            match word {
                "const" if context.ends_with('=') => Position::ConstValue,
                "const" if context.ends_with(':') => Position::ScoreType,
                "tag" if context.ends_with(':') => Position::TagType,
                _ => Position::Nothing,
            }
        }
    }
}

fn statement_position(context: &str, body: Body) -> Position {
    let tokens: Vec<&str> = context.split(' ').filter(|t| !t.is_empty()).collect();
    match tokens.as_slice() {
        [] => Position::Statement(body),
        ["else"] => Position::AfterElse,
        ["for", _] => Position::ForIn,
        ["for", _, "in"] => Position::ForSource,
        ["let"] | ["let", "mut"] | ["let", _] | ["let", "mut", _] => Position::Nothing,
        ["return"] if !body.returns => Position::Nothing,
        _ if context.ends_with(':') && tokens.first() == Some(&"let") => Position::ScoreType,
        _ => {
            let last = context.chars().last().unwrap_or(' ');
            let conditional = matches!(tokens.first(), Some(&"if") | Some(&"while"))
                || matches!(tokens.first(), Some(&"else")) && tokens.get(1) == Some(&"if");
            let condition_start =
                matches!(tokens.last(), Some(&"if" | &"while" | &"&&" | &"||")) || last == '!';
            if conditional && condition_start {
                return Position::Condition(body);
            }
            let keyword_end = ["if", "while", "return", "in", "&&", "||"]
                .iter()
                .any(|k| tokens.last() == Some(k));
            if "=+-*/%<>!&|(,".contains(last) || keyword_end {
                Position::Expression(body)
            } else {
                Position::Nothing
            }
        }
    }
}

fn flow_position(context: &str, field: &str) -> Position {
    let tokens: Vec<&str> = context.split(' ').filter(|t| !t.is_empty()).collect();
    let field = field.to_string();
    match tokens.as_slice() {
        [] => Position::FlowEntry { field },
        ["enter" | "exit" | "tick"] => Position::FlowPhase { field },
        [.., "->"] => Position::FlowPhase { field },
        [.., "=>"] => Position::FlowFunction,
        [.., "if" | "&&" | "||" | "priority" | "every"] => Position::ConstValue,
        _ if context.ends_with(['=', '<', '>', '!']) => Position::ConstValue,
        _ => Position::Nothing,
    }
}

/// What the attribute being written will be attached to: the next
/// non-attribute, non-comment line after the cursor.
fn attribute_target(after: &str, masked_before: &str) -> AttrTarget {
    let in_state = {
        let open = masked_before.rfind('{');
        let close = masked_before.rfind('}');
        open.is_some_and(|o| close.is_none_or(|c| c < o))
            && masked_before[..open.unwrap_or(0)]
                .rsplit('\n')
                .next()
                .is_some_and(|line| {
                    let (word, _) = header_keyword(strip_attributes(line));
                    matches!(word, "player" | "global" | "entity")
                })
    };
    if in_state {
        return AttrTarget::Field;
    }
    let next = after
        .lines()
        .skip(1)
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("#[") && !line.starts_with("//"))
        .unwrap_or("");
    let (word, rest) = header_keyword(next);
    match word {
        "fn" => AttrTarget::Function,
        "extern" if rest.contains("fn") => AttrTarget::ExternFunction,
        "" => AttrTarget::Function,
        _ => AttrTarget::Other,
    }
}
