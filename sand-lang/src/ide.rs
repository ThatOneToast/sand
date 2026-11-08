//! Editor queries over an [`Analysis`]: hover, navigation, rename,
//! completion, outline, and inlay hints.
//!
//! These functions are protocol-agnostic. Positions are byte offsets into a
//! file's text; `sand-lsp` converts them to and from LSP positions.

use crate::analysis::{Analysis, Context, DefId, DefInfo, DefKind, Ty, scope_names};
use crate::ast::{Scope, Span};
use crate::package::ModuleId;
use crate::source::FileId;
use std::collections::HashSet;

/// Markdown hover content for a source range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    /// Markdown body.
    pub markdown: String,
    /// The identifier the hover describes.
    pub span: Span,
}

fn ty_name(analysis: &Analysis, ty: Ty) -> String {
    match ty {
        Ty::Number => "number".into(),
        Ty::Float => "float".into(),
        Ty::Bool => "bool".into(),
        Ty::Enum(def) => analysis.def(def).name.clone(),
        Ty::Unknown => "{unknown}".into(),
    }
}

/// Render the signature line and facts for a definition.
pub fn describe(analysis: &Analysis, id: DefId) -> String {
    let def = analysis.def(id);
    let vis = if def.is_pub { "pub " } else { "" };
    let (signature, facts): (String, Vec<String>) = match &def.info {
        DefInfo::Module { target } => {
            let path = target
                .map(|t| analysis.package.module(t).display_path())
                .unwrap_or_else(|| def.name.clone());
            (format!("{vis}mod {}", def.name), vec![format!("`{path}`")])
        }
        DefInfo::Const { ty, value } => (
            format!(
                "{vis}const {}: {} = {}",
                def.name,
                ty_name(analysis, *ty),
                value.map(|v| v.to_string()).unwrap_or_else(|| "?".into())
            ),
            Vec::new(),
        ),
        DefInfo::Enum { variants } => {
            let body = variants
                .iter()
                .map(|v| {
                    let variant = analysis.def(*v);
                    match variant.info {
                        DefInfo::Variant { value: Some(n) } => {
                            format!("    {} = {n},", variant.name)
                        }
                        _ => format!("    {},", variant.name),
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            (
                format!("{vis}enum {} {{\n{body}\n}}", def.name),
                vec!["Stored in State as its integer discriminant.".into()],
            )
        }
        DefInfo::Variant { value } => {
            let owner = def.parent.map(|p| analysis.def(p).name.clone());
            (
                format!(
                    "{}::{}{}",
                    owner.unwrap_or_default(),
                    def.name,
                    value.map(|v| format!(" = {v}")).unwrap_or_default()
                ),
                Vec::new(),
            )
        }
        DefInfo::State { scope, id, fields } => {
            let body = fields
                .iter()
                .map(|f| field_line(analysis, *f))
                .collect::<Vec<_>>()
                .join("\n");
            let scope_text = match scope {
                Scope::Player => "player",
                Scope::Global => "global",
                Scope::Entity => "entity",
            };
            let ownership = match scope {
                Scope::Player => "One value per player; reading or writing needs a player.",
                Scope::Global => "One shared value for the whole server.",
                Scope::Entity => "One value per entity that has it attached with `attach_state`.",
            };
            (
                format!("{vis}{scope_text} state {} {{\n{body}\n}}", def.name),
                vec![format!("State `{id}`"), ownership.into()],
            )
        }
        DefInfo::Field { bounds, .. } => {
            let owner = def.parent.map(|p| analysis.def(p));
            let mut facts = Vec::new();
            if let Some(owner) = owner
                && let DefInfo::State { scope, .. } = owner.info
            {
                facts.push(format!(
                    "Field of {} state `{}`",
                    match scope {
                        Scope::Player => "player",
                        Scope::Global => "global",
                        Scope::Entity => "entity",
                    },
                    owner.name
                ));
            }
            if let Some((min, max)) = bounds {
                facts.push(format!("Writes are clamped to `{min}..={max}`."));
            }
            (field_line(analysis, id).trim().to_string(), facts)
        }
        DefInfo::Function {
            id,
            context,
            player_reason,
            entry,
            signature,
        } => {
            let mut facts = vec![format!("Function `{id}`")];
            facts.push(match (context, player_reason) {
                (Context::Player, Some(reason)) => {
                    format!("Runs as a **player** because it {reason}.")
                }
                (Context::Player, None) => "Runs as a **player**.".into(),
                (Context::Entity, Some(reason)) => format!(
                    "Runs as an **entity** (any entity, including players) because it {reason}."
                ),
                (Context::Entity, None) => "Runs as an **entity**.".into(),
                (Context::Server, _) => "Runs on the **server**; callable from anywhere.".into(),
            });
            if !entry.is_empty() {
                facts.push(format!("Entry point: {}", entry.join(", ")));
            }
            (format!("{vis}fn {}{signature}", def.name), facts)
        }
        DefInfo::ExternFunction { id, context } => (
            format!("{vis}extern \"{id}\" fn {}()", def.name),
            vec![format!(
                "Implemented outside this pack; runs as {}.",
                match context {
                    Context::Player => "a **player**",
                    Context::Entity => "an **entity**",
                    Context::Server => "the **server**",
                }
            )],
        ),
        DefInfo::Flow { id } => (
            format!("{vis}flow {}", def.name),
            vec![format!("Flow `{id}`")],
        ),
        DefInfo::Data { kind, id, tag_type } => {
            let (signature, fact) = match (kind, tag_type) {
                (crate::ast::DataKind::Item, _) => (
                    format!("{vis}item {}", def.name),
                    format!(
                        "Custom item `{id}`. Use it with `give_item`, `clear_item`, or as a recipe `result`."
                    ),
                ),
                (crate::ast::DataKind::Recipe, _) => (
                    format!("{vis}recipe {}", def.name),
                    format!("Recipe `{id}`."),
                ),
                (crate::ast::DataKind::Tag, ty) => {
                    let ty = ty.unwrap_or("?");
                    let usage = if ty == "Item" {
                        " Use it as a recipe ingredient or in other `Item` tags."
                    } else {
                        ""
                    };
                    (
                        format!("{vis}tag {}: {ty}", def.name),
                        format!("`{ty}` tag `#{id}`.{usage}"),
                    )
                }
            };
            (signature, vec![fact])
        }
        DefInfo::Local {
            ty,
            mutable,
            param,
            owner,
        } => {
            let owner = owner
                .map(|o| format!(" of `fn {}`", analysis.def(o).name))
                .unwrap_or_default();
            let kind = if *param {
                "Parameter"
            } else {
                "Local variable"
            };
            let signature = if *param {
                format!("{}: {}", def.name, ty_name(analysis, *ty))
            } else {
                format!(
                    "let {}{}: {}",
                    if *mutable { "mut " } else { "" },
                    def.name,
                    ty_name(analysis, *ty)
                )
            };
            (
                signature,
                vec![format!(
                    "{kind}{owner}. Stored in a compiler-owned scoreboard slot for this function."
                )],
            )
        }
    };
    let mut out = format!("```sand\n{signature}\n```");
    if !facts.is_empty() {
        out.push_str("\n\n");
        out.push_str(&facts.join("  \n"));
    }
    if !def.docs.is_empty() {
        out.push_str("\n\n---\n\n");
        out.push_str(&def.docs.join("\n"));
    }
    out
}

fn field_line(analysis: &Analysis, id: DefId) -> String {
    let def = analysis.def(id);
    let DefInfo::Field {
        ty,
        default,
        bounds,
    } = &def.info
    else {
        return String::new();
    };
    let default_text = match (ty, default) {
        (Ty::Enum(e), Some(v)) => variant_name(analysis, *e, *v)
            .map(|name| format!("{}::{name}", analysis.def(*e).name))
            .unwrap_or_else(|| v.to_string()),
        (Ty::Float, Some(v)) => format!("{}", *v as f64 / crate::analysis::FLOAT_SCALE as f64),
        (Ty::Bool, Some(v)) => (*v != 0).to_string(),
        (_, Some(v)) => v.to_string(),
        (_, None) => "?".into(),
    };
    let clamp = bounds
        .map(|(min, max)| format!("    #[clamp({min}..={max})]\n"))
        .unwrap_or_default();
    format!(
        "{clamp}    {}: {} = {default_text},",
        def.name,
        ty_name(analysis, *ty)
    )
}

fn variant_name(analysis: &Analysis, enum_def: DefId, value: i64) -> Option<String> {
    let DefInfo::Enum { variants } = &analysis.def(enum_def).info else {
        return None;
    };
    variants.iter().find_map(|v| match analysis.def(*v).info {
        DefInfo::Variant { value: Some(n) } if n == value => Some(analysis.def(*v).name.clone()),
        _ => None,
    })
}

/// Hover information at `offset`.
pub fn hover(analysis: &Analysis, file: FileId, offset: usize) -> Option<Hover> {
    let Some(def) = analysis.def_at(file, offset) else {
        return vocabulary_hover(analysis, file, offset);
    };
    let span = analysis
        .references
        .iter()
        .filter(|r| r.file == file && r.span.contains(offset) && r.def == def)
        .map(|r| r.span)
        .chain(
            Some(analysis.def(def))
                .filter(|d| d.file == file && d.name_span.contains(offset))
                .map(|d| d.name_span),
        )
        .next()?;
    Some(Hover {
        markdown: describe(analysis, def),
        span,
    })
}

/// The identifier-like word around `offset`.
fn word_at(text: &str, offset: usize) -> Option<(usize, usize)> {
    let offset = offset.min(text.len());
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let start = text[..offset]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(offset);
    let end = offset
        + text[offset..]
            .char_indices()
            .find(|(_, c)| !is_word(*c))
            .map(|(i, _)| i)
            .unwrap_or(text.len() - offset);
    (start < end).then_some((start, end))
}

/// Explain builtins, attributes, events, text styles, and keywords.
fn vocabulary_hover(analysis: &Analysis, file: FileId, offset: usize) -> Option<Hover> {
    use crate::builtins;
    if let Some(used) = analysis
        .vanilla_uses
        .iter()
        .find(|u| u.file == file && u.span.contains(offset))
    {
        return Some(Hover {
            markdown: vanilla_doc(used.registry, &used.variant, &used.id),
            span: used.span,
        });
    }
    if let Some((_, span, name, kind)) = analysis
        .builtin_uses
        .iter()
        .find(|(f, span, _, _)| *f == file && span.contains(offset))
    {
        return Some(Hover {
            markdown: builtins::describe(builtins::find(name, *kind)?),
            span: *span,
        });
    }
    let text = &analysis.package.files[file.0].text;
    let (start, end) = word_at(text, offset)?;
    let word = &text[start..end];
    let span = Span::new(start, end);
    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let before = &text[line_start..start];
    let markdown = |body: String| {
        Some(Hover {
            markdown: body,
            span,
        })
    };

    // Inside `#[...]` or `#![...]`.
    if let Some(open) = before.rfind("#[").or_else(|| before.rfind("#!["))
        && !before[open..].contains(']')
    {
        if before[open..].contains("on(")
            && let Some((name, _, doc)) = builtins::EVENTS.iter().find(|(name, _, _)| *name == word)
        {
            return markdown(format!("```sand\n#[on({name})]\n```\n\n{doc}"));
        }
        if let Some((_, signature, doc)) = builtins::ATTRIBUTES
            .iter()
            .find(|(name, _, _)| *name == word)
        {
            return markdown(format!("```sand\n{signature}\n```\n\n{doc}"));
        }
    }
    // Interpolation inside text: `{gold}`, `{player}`.
    if before.trim_end().ends_with('{') && before.matches('"').count() % 2 == 1 {
        if word == "player" {
            return markdown("`{player}` shows the executing player's name. Using it makes the function need a player.".into());
        }
        if builtins::STYLES.contains(&word) {
            let doc = match word {
                "bold" | "italic" | "underlined" => format!("Makes the following text {word}."),
                "reset" => "Clears colour and formatting for the following text.".into(),
                color => format!("Colours the following text {}.", color.replace('_', " ")),
            };
            return markdown(format!("`{{{word}}}`: {doc}"));
        }
    }
    // A builtin written without a recorded use (for example with errors).
    let after = &text[end..];
    let kind = if after.starts_with('!') {
        builtins::BuiltinKind::Macro
    } else if word == "random" {
        builtins::BuiltinKind::Expression
    } else {
        builtins::BuiltinKind::Statement
    };
    if after.trim_start().starts_with(['(', '!'])
        && let Some(builtin) = builtins::find(word, kind)
    {
        return markdown(builtins::describe(builtin));
    }
    // Data declarations: the keyword and field names.
    let file_ast = &analysis.package.files[file.0].ast;
    let data_keyword = data_item_at(&file_ast.items, offset)
        .map(|data| data.kind.keyword())
        .or_else(|| match crate::position::classify(text, offset) {
            crate::position::Position::Data { keyword, .. } => Some(keyword),
            _ => None,
        })
        .or_else(|| {
            // On the declaration keyword itself.
            ["item", "recipe", "tag"]
                .into_iter()
                .find(|k| *k == word && before.trim().is_empty())
        });
    if let Some(keyword) = data_keyword {
        if let Some(field) = builtins::data_fields(keyword).find(|f| f.name == word)
            && after.trim_start().starts_with(':')
        {
            return markdown(format!(
                "```sand\n{}: {}\n```\n\n{}",
                field.name,
                snippet_text(field.snippet),
                field.doc
            ));
        }
        if word == keyword
            && let Some((_, doc)) = builtins::DATA_KINDS.iter().find(|(k, _)| *k == word)
        {
            return markdown(format!("**`{word}`**\n\n{doc}"));
        }
    }
    builtins::KEYWORDS
        .iter()
        .find(|(name, _)| *name == word)
        .and_then(|(name, doc)| markdown(format!("**`{name}`**\n\n{doc}")))
}

fn vanilla_doc(registry: &str, variant: &str, id: &str) -> String {
    let table = crate::vanilla::registry(registry);
    let noun = table.map(|t| t.noun).unwrap_or("value");
    let rust = table.map(|t| t.rust_path).unwrap_or(registry);
    format!(
        "```sand\n{registry}::{variant}\n```\n\nThe vanilla {noun} `{id}`.\n\nRust: `{rust}::{variant}`"
    )
}

fn registry_completions(
    registry: &'static crate::vanilla::Registry,
    qualified: bool,
) -> Vec<Completion> {
    registry
        .order
        .iter()
        .map(|variant| (variant, &registry.variants[variant]))
        .map(|(variant, id)| {
            let label = if qualified {
                format!("{}::{variant}", registry.name)
            } else {
                variant.clone()
            };
            Completion {
                detail: Some(id.clone()),
                documentation: Some(vanilla_doc(registry.name, variant, id)),
                ..Completion::plain(&label, CompletionKind::Variant)
            }
        })
        .collect()
}

/// A completion snippet as plain text: `Item::${1:Diamond}` -> `Item::Diamond`.
fn snippet_text(snippet: &str) -> String {
    let mut out = String::new();
    let mut rest = snippet;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let inner = &rest[start + 2..];
        let end = inner.find('}').unwrap_or(inner.len());
        let placeholder = &inner[..end];
        out.push_str(
            placeholder
                .split_once(':')
                .map(|(_, text)| text)
                .unwrap_or(""),
        );
        rest = inner.get(end + 1..).unwrap_or("");
    }
    out.push_str(rest);
    out.replace("$0", "")
}

fn data_item_at(items: &[crate::ast::Item], offset: usize) -> Option<&crate::ast::DataItem> {
    items.iter().find_map(|item| match &item.kind {
        crate::ast::ItemKind::Data(data)
            if item.span.start <= offset && offset <= item.span.end =>
        {
            Some(data)
        }
        crate::ast::ItemKind::Mod(module) => module
            .inline
            .as_ref()
            .and_then(|inner| data_item_at(inner, offset)),
        _ => None,
    })
}

/// Where the definition named at `offset` is declared.
pub fn definition(analysis: &Analysis, file: FileId, offset: usize) -> Option<(FileId, Span)> {
    let def = analysis.def_at(file, offset)?;
    let d = analysis.def(def);
    // A file module's definition is the module file itself.
    if let DefInfo::Module {
        target: Some(target),
    } = d.info
    {
        let module = analysis.package.module(target);
        if module.file != d.file {
            return Some((module.file, Span::new(0, 0)));
        }
    }
    Some((d.file, d.name_span))
}

/// Every location naming the definition at `offset`.
pub fn references(
    analysis: &Analysis,
    file: FileId,
    offset: usize,
    include_declaration: bool,
) -> Vec<(FileId, Span)> {
    let Some(def) = analysis.def_at(file, offset) else {
        return Vec::new();
    };
    let mut out: Vec<(FileId, Span)> = analysis
        .references
        .iter()
        .filter(|r| r.def == def)
        .map(|r| (r.file, r.span))
        .collect();
    if include_declaration {
        let d = analysis.def(def);
        out.insert(0, (d.file, d.name_span));
    }
    out.sort();
    out.dedup();
    out
}

/// A text replacement produced by rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// File to edit.
    pub file: FileId,
    /// Replaced range.
    pub span: Span,
    /// Replacement text.
    pub text: String,
}

/// The range and current name that would be renamed at `offset`.
pub fn prepare_rename(analysis: &Analysis, file: FileId, offset: usize) -> Result<Span, String> {
    let def = analysis
        .def_at(file, offset)
        .ok_or_else(|| "no renameable symbol here".to_string())?;
    if analysis.def(def).kind == DefKind::Module {
        return Err("renaming modules would rename files; rename the file instead".into());
    }
    hover(analysis, file, offset)
        .map(|h| h.span)
        .ok_or_else(|| "no renameable symbol here".into())
}

/// Rename the definition at `offset` and every use of it.
///
/// Uses through a differently-named `use ... as alias` keep their alias.
pub fn rename(
    analysis: &Analysis,
    file: FileId,
    offset: usize,
    new_name: &str,
) -> Result<Vec<Edit>, String> {
    prepare_rename(analysis, file, offset)?;
    let valid = new_name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && new_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !KEYWORDS.contains(&new_name);
    if !valid {
        return Err(format!("`{new_name}` is not a valid identifier"));
    }
    let def = analysis.def_at(file, offset).expect("checked by prepare");
    let old = analysis.def(def).name.clone();
    Ok(references(analysis, file, offset, true)
        .into_iter()
        .filter(|(f, span)| {
            analysis.package.files[f.0]
                .text
                .get(span.start..span.end)
                .is_some_and(|text| text == old)
        })
        .map(|(file, span)| Edit {
            file,
            span,
            text: new_name.to_string(),
        })
        .collect())
}

/// Reserved words that cannot name items.
pub const KEYWORDS: &[&str] = &[
    "fn", "pub", "mod", "use", "const", "enum", "state", "flow", "if", "else", "for", "in",
    "extern", "as", "let", "return", "while", "loop", "match", "struct", "true", "false", "crate",
    "self", "super",
];

/// What kind of thing a completion inserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// A language keyword.
    Keyword,
    /// A snippet template.
    Snippet,
    /// A module.
    Module,
    /// A function.
    Function,
    /// A constant.
    Constant,
    /// An enum.
    Enum,
    /// An enum variant.
    Variant,
    /// A State.
    State,
    /// A State field.
    Field,
    /// A flow.
    Flow,
    /// An attribute name.
    Attribute,
    /// A local variable or parameter.
    Variable,
}

/// One completion candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// Label shown and filtered on.
    pub label: String,
    /// Candidate kind.
    pub kind: CompletionKind,
    /// Short type or signature detail.
    pub detail: Option<String>,
    /// Markdown documentation.
    pub documentation: Option<String>,
    /// Text to insert, when it differs from the label.
    pub insert: Option<String>,
    /// Whether `insert` uses LSP snippet syntax.
    pub snippet: bool,
}

impl Completion {
    fn plain(label: &str, kind: CompletionKind) -> Self {
        Self {
            label: label.into(),
            kind,
            detail: None,
            documentation: None,
            insert: None,
            snippet: false,
        }
    }

    fn snippet(label: &str, detail: &str, body: &str) -> Self {
        Self {
            label: label.into(),
            kind: CompletionKind::Snippet,
            detail: Some(detail.into()),
            documentation: None,
            insert: Some(body.into()),
            snippet: true,
        }
    }
}

enum Target {
    Module(ModuleId),
    Def(DefId),
}

fn kind_of(kind: DefKind) -> CompletionKind {
    match kind {
        DefKind::Module => CompletionKind::Module,
        DefKind::Const => CompletionKind::Constant,
        DefKind::Enum => CompletionKind::Enum,
        DefKind::Variant => CompletionKind::Variant,
        DefKind::State => CompletionKind::State,
        DefKind::Field => CompletionKind::Field,
        DefKind::Function | DefKind::ExternFunction => CompletionKind::Function,
        DefKind::Flow => CompletionKind::Flow,
        DefKind::Local => CompletionKind::Variable,
        DefKind::CustomItem | DefKind::Recipe | DefKind::Tag => CompletionKind::Constant,
    }
}

fn def_completion(analysis: &Analysis, name: &str, def: DefId) -> Completion {
    let d = analysis.def(def);
    let detail = describe(analysis, def).lines().nth(1).map(str::to_string);
    let mut snippet = false;
    let insert = match &d.info {
        DefInfo::Function { signature, .. } => {
            let params: Vec<&str> = signature
                .strip_prefix('(')
                .and_then(|s| s.split_once(')'))
                .map(|(inner, _)| inner.split(", ").filter(|p| !p.is_empty()).collect())
                .unwrap_or_default();
            snippet = !params.is_empty();
            let args = params
                .iter()
                .enumerate()
                .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.split(':').next().unwrap_or(p)))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("{name}({args})"))
        }
        DefInfo::ExternFunction { .. } => Some(format!("{name}()")),
        _ => None,
    };
    Completion {
        label: name.into(),
        kind: kind_of(d.kind),
        detail,
        documentation: (!d.docs.is_empty()).then(|| d.docs.join("\n")),
        insert,
        snippet,
    }
}

fn resolve_names(analysis: &Analysis, from: ModuleId, names: &[&str]) -> Option<Target> {
    let mut cur = Target::Module(from);
    let enter = |def: DefId| match analysis.def(def).info {
        DefInfo::Module {
            target: Some(module),
        } => Target::Module(module),
        _ => Target::Def(def),
    };
    for (index, name) in names.iter().enumerate() {
        cur = match (*name, &cur) {
            ("crate", _) if index == 0 => Target::Module(ModuleId(0)),
            ("self", _) if index == 0 => Target::Module(from),
            ("super", Target::Module(m)) => Target::Module(analysis.package.module(*m).parent?),
            (name, Target::Module(m)) => {
                let def = scope_names(analysis, *m)
                    .into_iter()
                    .find(|(n, _)| n == name)?
                    .1;
                enter(def)
            }
            (name, Target::Def(def)) => {
                let DefInfo::Enum { variants } = &analysis.def(*def).info else {
                    return None;
                };
                Target::Def(*variants.iter().find(|v| analysis.def(**v).name == name)?)
            }
        };
    }
    Some(cur)
}

/// Completion candidates at `offset`.
pub fn completions(analysis: &Analysis, file: FileId, offset: usize) -> Vec<Completion> {
    use crate::position::{self, Position};
    let text = &analysis.package.files[file.0].text;
    let offset = offset.min(text.len());
    let before = &text[..offset];
    let line = before.rsplit('\n').next().unwrap_or("");
    let module = analysis
        .package
        .module_at(file, offset)
        .unwrap_or(ModuleId(0));
    let mut position = position::classify(text, offset);
    // A path being typed (`Hero.en`, `Phase::F`) takes the position where
    // the path starts.
    let path_start = line
        .char_indices()
        .rev()
        .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_' || *c == ':' || *c == '.'))
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    if position == Position::Nothing && line[path_start..].contains(['.', ':']) {
        let start = offset - (line.len() - path_start);
        position = position::classify(text, start);
    }

    match &position {
        Position::Nothing => return Vec::new(),
        Position::Attribute { inner, target } => return attribute_completions(*inner, *target),
        Position::AttributeArg { attr, key } => {
            return attribute_arg_completions(attr, key.as_deref());
        }
        Position::Interpolation => {
            return interpolation_completions(analysis, file, offset, module);
        }
        _ => {}
    }

    // The path being typed, such as `crate::rewards::gr` or `Hero.en`.
    let token_start = line
        .char_indices()
        .rev()
        .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_' || *c == ':' || *c == '.'))
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    let token = &line[token_start..];
    let value_position = matches!(
        position,
        Position::Statement(_)
            | Position::Expression(_)
            | Position::Condition(_)
            | Position::ConstValue
            | Position::FlowField
            | Position::Data { .. }
    );
    if value_position && let Some(dot) = token.rfind('.') {
        let base: Vec<&str> = token[..dot].split("::").collect();
        let Some(Target::Def(def)) = resolve_names(analysis, module, &base) else {
            return Vec::new();
        };
        let DefInfo::State { fields, .. } = &analysis.def(def).info else {
            return Vec::new();
        };
        return fields
            .iter()
            .map(|f| def_completion(analysis, &analysis.def(*f).name, *f))
            .collect();
    }
    if let Some(sep) = token.rfind("::") {
        let base: Vec<&str> = token[..sep].split("::").collect();
        if let [name] = base.as_slice()
            && let Some(registry) = crate::vanilla::registry(name)
            && !scope_names(analysis, module).iter().any(|(n, _)| n == name)
        {
            return registry_completions(registry, false);
        }
        let types_only = matches!(position, Position::ScoreType);
        return match resolve_names(analysis, module, &base) {
            Some(Target::Module(m)) => scope_names(analysis, m)
                .into_iter()
                .filter(|(_, def)| analysis.def(*def).is_pub || is_ancestor(analysis, m, module))
                .filter(|(_, def)| {
                    !types_only
                        || matches!(analysis.def(*def).kind, DefKind::Enum | DefKind::Module)
                })
                .map(|(name, def)| def_completion(analysis, &name, def))
                .collect(),
            Some(Target::Def(def)) => match &analysis.def(def).info {
                DefInfo::Enum { variants } if !types_only => variants
                    .iter()
                    .map(|v| def_completion(analysis, &analysis.def(*v).name, *v))
                    .collect(),
                _ => Vec::new(),
            },
            None => Vec::new(),
        };
    }

    let in_scope = scope_names(analysis, module);
    let defs = |keep: &dyn Fn(&crate::analysis::Def) -> bool| scope_defs(analysis, &in_scope, keep);
    let paths = || {
        ["crate", "self", "super"]
            .into_iter()
            .map(|p| Completion::plain(p, CompletionKind::Keyword))
    };
    match position {
        Position::ItemStart { after_pub } => item_snippets(after_pub),
        Position::StateKeyword => vec![Completion::plain("state", CompletionKind::Keyword)],
        Position::FlowOn => vec![Completion::plain("on", CompletionKind::Keyword)],
        Position::FlowField => defs(&|d| d.kind == DefKind::State),
        Position::UsePath => {
            let mut out = defs(&|d| d.kind != DefKind::Local);
            out.extend(paths());
            out
        }
        Position::ScoreType => {
            let mut out = vec![Completion {
                documentation: Some("A whole number.".into()),
                ..Completion::plain("number", CompletionKind::Keyword)
            }];
            out.push(Completion {
                documentation: Some("A decimal with three places, such as `1.5`.".into()),
                ..Completion::plain("float", CompletionKind::Keyword)
            });
            out.push(Completion {
                documentation: Some("`true` or `false`.".into()),
                ..Completion::plain("bool", CompletionKind::Keyword)
            });
            out.extend(defs(&|d| matches!(d.kind, DefKind::Enum | DefKind::Module)));
            out
        }
        Position::TagType => ["Item", "Block", "Entity"]
            .into_iter()
            .map(|ty| Completion {
                detail: Some(format!("a tag of `{ty}` values")),
                ..Completion::plain(ty, CompletionKind::Enum)
            })
            .collect(),
        Position::ConstValue => {
            let mut out =
                defs(&|d| matches!(d.kind, DefKind::Const | DefKind::Enum | DefKind::Module));
            out.extend(paths());
            out
        }
        Position::Statement(body) => {
            statement_completions(analysis, file, offset, &body, &in_scope, paths())
        }
        Position::Expression(body) => {
            expression_completions(analysis, file, offset, &body, &in_scope, paths())
        }
        Position::Condition(body) => {
            let mut out = expression_completions(analysis, file, offset, &body, &in_scope, paths());
            out.extend(
                crate::builtins::BUILTINS
                    .iter()
                    .filter(|b| b.kind == crate::builtins::BuiltinKind::Condition)
                    .map(builtin_completion),
            );
            out
        }
        Position::ForIn => vec![Completion::plain("in", CompletionKind::Keyword)],
        Position::ForSource => vec![
            Completion {
                documentation: Some("Run the body as each online player.".into()),
                insert: Some("players()".into()),
                ..Completion::plain("players()", CompletionKind::Function)
            },
            crate::builtins::find("entities", crate::builtins::BuiltinKind::Expression)
                .map(|b| Completion {
                    label: "entities(..)".into(),
                    kind: CompletionKind::Function,
                    detail: Some(b.summary.into()),
                    documentation: Some(crate::builtins::describe(b)),
                    insert: Some("entities(type = Entity::${1:Zombie})".into()),
                    snippet: true,
                })
                .expect("entities builtin"),
        ],
        Position::AfterElse => vec![Completion::plain("if", CompletionKind::Keyword)],
        Position::StateField => vec![Completion::snippet(
            "#[clamp]",
            "clamp writes to a range",
            "#[clamp(${1:0}..=${2:100})]",
        )],
        Position::FlowEntry { field } => {
            let mut out = vec![
                Completion::snippet(
                    "enter",
                    "run when entering a phase",
                    "enter ${1:Phase} => ${2:function};",
                ),
                Completion::snippet(
                    "exit",
                    "run when leaving a phase",
                    "exit ${1:Phase} => ${2:function};",
                ),
                Completion::snippet(
                    "tick",
                    "run every tick in a phase",
                    "tick ${1:Phase} {\n    $0\n}",
                ),
            ];
            out.extend(flow_phases(analysis, module, &field));
            out
        }
        Position::FlowPhase { field } => flow_phases(analysis, module, &field),
        Position::FlowFunction => defs(&|d| {
            matches!(&d.info, DefInfo::Function { signature, .. } if signature == "()")
                || d.kind == DefKind::ExternFunction
        }),
        Position::Data {
            keyword,
            tag_type,
            line,
            used,
        } => data_completions(analysis, module, keyword, tag_type, &line, &used),
        Position::Nothing
        | Position::Attribute { .. }
        | Position::AttributeArg { .. }
        | Position::Interpolation => Vec::new(),
    }
}

/// Definitions in scope that satisfy `keep`.
fn scope_defs(
    analysis: &Analysis,
    in_scope: &[(String, DefId)],
    keep: &dyn Fn(&crate::analysis::Def) -> bool,
) -> Vec<Completion> {
    in_scope
        .iter()
        .filter(|(_, def)| keep(analysis.def(*def)))
        .map(|(name, def)| def_completion(analysis, name, *def))
        .collect()
}

fn statement_completions(
    analysis: &Analysis,
    file: FileId,
    offset: usize,
    body: &crate::position::Body,
    in_scope: &[(String, DefId)],
    paths: impl Iterator<Item = Completion>,
) -> Vec<Completion> {
    let mut out = local_completions(analysis, file, offset);
    // Statements call functions and assign fields.
    out.extend(scope_defs(analysis, in_scope, &|d| {
        matches!(
            d.kind,
            DefKind::Function | DefKind::ExternFunction | DefKind::State | DefKind::Module
        )
    }));
    out.extend([
        Completion::snippet("let", "declare a variable", "let ${1:name} = ${2:0};"),
        Completion::snippet(
            "let mut",
            "declare a mutable variable",
            "let mut ${1:name} = ${2:0};",
        ),
        Completion::snippet(
            "if",
            "branch on a condition",
            "if ${1:condition} {\n    $0\n}",
        ),
        Completion::snippet(
            "while",
            "repeat while a condition holds",
            "while ${1:condition} {\n    $0\n}",
        ),
        Completion::snippet(
            "for player in players()",
            "run as each online player",
            "for player in players() {\n    $0\n}",
        ),
        Completion::snippet(
            "for entity in entities(..)",
            "run as each matching entity",
            "for ${1:entity} in entities(type = Entity::${2:Zombie}) {\n    $0\n}",
        ),
        if body.returns {
            Completion::snippet("return", "return a value", "return ${1:value};")
        } else {
            Completion::snippet("return", "end the function early", "return;")
        },
    ]);
    if body.executor_loop.is_some() {
        out.push(Completion::snippet(
            "continue",
            "skip to the next one",
            "continue;",
        ));
    }
    let in_entities = body.executor_loop == Some(true);
    out.extend(
        crate::builtins::BUILTINS
            .iter()
            .filter(|b| {
                !matches!(
                    b.kind,
                    crate::builtins::BuiltinKind::Expression
                        | crate::builtins::BuiltinKind::Condition
                )
            })
            // An entity may not be a player.
            .filter(|b| !(in_entities && b.needs_player))
            .map(builtin_completion),
    );
    out.extend(paths);
    out
}

fn expression_completions(
    analysis: &Analysis,
    file: FileId,
    offset: usize,
    _body: &crate::position::Body,
    in_scope: &[(String, DefId)],
    paths: impl Iterator<Item = Completion>,
) -> Vec<Completion> {
    let mut out = Vec::new();
    // Inside a builtin call: its remaining named arguments, and typed values
    // for the argument being written.
    if let Some(help) = signature_help(analysis, file, offset) {
        if help
            .params
            .get(help.active)
            .is_some_and(|(start, end, _)| help.label[*start..*end].ends_with(": EntityState"))
        {
            return scope_defs(analysis, in_scope, &|d| {
                matches!(
                    d.info,
                    DefInfo::State {
                        scope: Scope::Entity,
                        ..
                    }
                )
            });
        }
        if let Some(registry) = help
            .params
            .get(help.active)
            .and_then(|(start, end, _)| help.label[*start..*end].split_once(": "))
            .and_then(|(_, ty)| crate::vanilla::registry(ty))
        {
            out.extend(registry_completions(registry, true));
        }
        if let Some((name, _)) = help.label.split_once('(')
            && let Some(builtin) = crate::builtins::BUILTINS
                .iter()
                .find(|b| b.name == name.trim_end_matches('!'))
        {
            let text = &analysis.package.files[file.0].text[..offset];
            let call = &text[text.rfind('(').map(|p| p + 1).unwrap_or(0)..];
            for param in builtin.params.iter().filter(|p| !p.required) {
                if !call.contains(&format!("{} =", param.name)) {
                    out.push(Completion {
                        detail: Some(param.doc.into()),
                        insert: Some(format!("{} = ", param.name)),
                        ..Completion::plain(&format!("{} =", param.name), CompletionKind::Field)
                    });
                }
            }
        }
    }
    out.extend(local_completions(analysis, file, offset));
    out.extend(scope_defs(analysis, in_scope, &|d| match &d.info {
        DefInfo::Function { signature, .. } => signature.contains("->"),
        DefInfo::Const { .. }
        | DefInfo::Enum { .. }
        | DefInfo::State { .. }
        | DefInfo::Module { .. } => true,
        DefInfo::Data { kind, .. } => *kind == crate::ast::DataKind::Item,
        _ => false,
    }));
    for registry in crate::vanilla::registries() {
        out.push(Completion {
            detail: Some(format!("{} values", registry.noun)),
            documentation: Some(format!(
                "Typed {} values, such as `{}::{}`. Rust: `{}`.",
                registry.noun,
                registry.name,
                registry.order.first().map(String::as_str).unwrap_or(""),
                registry.rust_path
            )),
            insert: Some(format!("{}::", registry.name)),
            ..Completion::plain(registry.name, CompletionKind::Enum)
        });
    }
    out.extend(
        crate::builtins::BUILTINS
            .iter()
            // `entities(..)` is only a `for` source.
            .filter(|b| b.kind == crate::builtins::BuiltinKind::Expression && b.name != "entities")
            .map(builtin_completion),
    );
    out.extend(paths);
    out
}

/// Locals visible at `offset`, inner declarations first. Falls back to a
/// lexical scan of the enclosing function when it does not parse yet.
fn local_completions(analysis: &Analysis, file: FileId, offset: usize) -> Vec<Completion> {
    let mut seen = HashSet::new();
    let mut locals: Vec<&(FileId, Span, DefId)> = analysis
        .local_scopes
        .iter()
        .filter(|(f, span, _)| *f == file && span.contains(offset))
        .collect();
    locals.sort_by_key(|(_, _, def)| std::cmp::Reverse(def.0));
    let mut out: Vec<Completion> = locals
        .into_iter()
        .filter_map(|(_, _, def)| {
            let name = analysis.def(*def).name.clone();
            seen.insert(name.clone())
                .then(|| def_completion(analysis, &name, *def))
        })
        .collect();
    if !out.is_empty() {
        return out;
    }
    let text = &analysis.package.files[file.0].text[..offset];
    let Some(start) = text.rfind("fn ") else {
        return out;
    };
    let function = &text[start..];
    let words: Vec<&str> = function
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
        .filter(|w| !w.is_empty())
        .collect();
    let mut names = Vec::new();
    // Parameters: `name:` inside the signature.
    if let Some(open) = function.find('(')
        && let Some(close) = function[open..].find(')')
    {
        for param in function[open + 1..open + close].split(',') {
            if let Some((name, _)) = param.split_once(':') {
                names.push(name.trim().to_string());
            }
        }
    }
    for (i, word) in words.iter().enumerate() {
        if *word == "let" {
            let next = if words.get(i + 1) == Some(&"mut") {
                words.get(i + 2)
            } else {
                words.get(i + 1)
            };
            if let Some(name) = next {
                names.push(name.trim_end_matches(':').to_string());
            }
        }
    }
    for name in names.into_iter().rev() {
        if !name.is_empty() && seen.insert(name.clone()) {
            out.push(Completion::plain(&name, CompletionKind::Variable));
        }
    }
    out
}

fn attribute_completions(inner: bool, target: crate::position::AttrTarget) -> Vec<Completion> {
    use crate::position::AttrTarget;
    let names: &[&str] = if inner {
        &["pack"]
    } else {
        match target {
            AttrTarget::Function => &["tick", "load", "on", "tag", "player"],
            AttrTarget::ExternFunction => &["player"],
            AttrTarget::Field => &["clamp"],
            AttrTarget::Other => &[],
        }
    };
    crate::builtins::ATTRIBUTES
        .iter()
        .filter(|(name, _, _)| names.contains(name))
        .map(|(label, signature, doc)| Completion {
            detail: Some((*signature).into()),
            documentation: Some((*doc).into()),
            ..Completion::plain(label, CompletionKind::Attribute)
        })
        .collect()
}

fn attribute_arg_completions(attr: &str, key: Option<&str>) -> Vec<Completion> {
    match (attr, key) {
        ("on", None) => crate::builtins::EVENTS
            .iter()
            .map(|(label, set, doc)| Completion {
                documentation: Some((*doc).into()),
                // `kill` works bare or filtered; the bare form is offered.
                insert: set
                    .filter(|_| *label != "kill")
                    .map(|set| format!("{label} = {set}::${{1}}")),
                snippet: set.is_some() && *label != "kill",
                ..Completion::plain(label, CompletionKind::Attribute)
            })
            .collect(),
        ("on", Some(key)) => crate::builtins::EVENTS
            .iter()
            .find(|(label, _, _)| *label == key)
            .and_then(|(_, set, _)| *set)
            .and_then(crate::vanilla::registry)
            .map(|r| registry_completions(r, true))
            .unwrap_or_default(),
        ("pack", None) => [
            (
                "namespace",
                "namespace = \"${1:my_pack}\"",
                "Namespace of every generated resource.",
            ),
            (
                "minecraft",
                "minecraft = \"${1:26.2}\"",
                "Exact Minecraft target version.",
            ),
            (
                "description",
                "description = \"${1}\"",
                "Pack description shown in the pack list.",
            ),
        ]
        .into_iter()
        .map(|(label, insert, doc)| Completion {
            documentation: Some(doc.into()),
            insert: Some(insert.into()),
            snippet: true,
            ..Completion::plain(label, CompletionKind::Field)
        })
        .collect(),
        ("clamp", None) => vec![Completion::snippet(
            "min..=max",
            "inclusive range",
            "${1:0}..=${2:100}",
        )],
        ("tag", None) => vec![Completion::snippet(
            "\"namespace:name\"",
            "function tag",
            "\"${1:minecraft}:${2:tick}\"",
        )],
        _ => Vec::new(),
    }
}

fn interpolation_completions(
    analysis: &Analysis,
    file: FileId,
    offset: usize,
    module: ModuleId,
) -> Vec<Completion> {
    let mut out: Vec<Completion> = std::iter::once(Completion {
        documentation: Some("The executing player's name.".into()),
        ..Completion::plain("player", CompletionKind::Keyword)
    })
    .chain(
        crate::builtins::STYLES
            .iter()
            .map(|style| Completion::plain(style, CompletionKind::Attribute)),
    )
    .collect();
    out.extend(local_completions(analysis, file, offset));
    for (name, def) in scope_names(analysis, module) {
        if let DefInfo::State { fields, .. } = &analysis.def(def).info {
            for field in fields {
                let field_name = &analysis.def(*field).name;
                out.push(Completion {
                    detail: Some("live score".into()),
                    ..Completion::plain(&format!("{name}.{field_name}"), CompletionKind::Field)
                });
            }
        }
    }
    out
}

/// The phases of a flow's enum field, such as `Idle` and `Fighting`.
fn flow_phases(analysis: &Analysis, module: ModuleId, field: &str) -> Vec<Completion> {
    let Some((state, field)) = field.rsplit_once('.') else {
        return Vec::new();
    };
    let names: Vec<&str> = state.split("::").collect();
    let Some(Target::Def(state)) = resolve_names(analysis, module, &names) else {
        return Vec::new();
    };
    let DefInfo::State { fields, .. } = &analysis.def(state).info else {
        return Vec::new();
    };
    let Some(field) = fields
        .iter()
        .find(|f| analysis.def(**f).name == field.trim())
    else {
        return Vec::new();
    };
    let DefInfo::Field {
        ty: crate::analysis::Ty::Enum(enum_def),
        ..
    } = analysis.def(*field).info
    else {
        return Vec::new();
    };
    let DefInfo::Enum { variants } = &analysis.def(enum_def).info else {
        return Vec::new();
    };
    variants
        .iter()
        .map(|v| def_completion(analysis, &analysis.def(*v).name, *v))
        .collect()
}

fn builtin_completion(b: &crate::builtins::Builtin) -> Completion {
    Completion {
        label: match b.kind {
            crate::builtins::BuiltinKind::Macro => format!("{}!", b.name),
            _ => b.name.to_string(),
        },
        kind: match b.kind {
            crate::builtins::BuiltinKind::Expression => CompletionKind::Function,
            _ => CompletionKind::Snippet,
        },
        detail: Some(b.summary.into()),
        documentation: Some(crate::builtins::describe(b)),
        insert: Some(b.snippet.into()),
        snippet: true,
    }
}

/// Field names, or values for the field being written, inside a data body.
fn data_completions(
    analysis: &Analysis,
    module: ModuleId,
    keyword: &str,
    tag_type: Option<&'static str>,
    line: &str,
    used: &[String],
) -> Vec<Completion> {
    use crate::builtins::{self, FieldKind};
    // The field whose value is being typed: `name:` earlier on this line.
    let field = line
        .rfind(':')
        .filter(|colon| !line[colon + 1..].contains(':') || line[colon + 1..].contains("::"))
        .and_then(|colon| {
            let before = line[..colon].trim_end();
            let start = before
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map(|i| i + 1)
                .unwrap_or(0);
            builtins::data_fields(keyword).find(|f| f.name == &before[start..])
        });
    let declared = |kind: DefKind| -> Vec<Completion> {
        scope_names(analysis, module)
            .into_iter()
            .filter(|(_, def)| analysis.def(*def).kind == kind)
            .map(|(name, def)| def_completion(analysis, &name, def))
            .collect()
    };
    let registry = |name: &str| {
        crate::vanilla::registry(name)
            .map(|r| registry_completions(r, true))
            .unwrap_or_default()
    };
    let Some(field) = field else {
        let used = |name: &str| used.iter().any(|u| u == name);
        let recipe_type = builtins::RECIPE_TYPES.iter().find(|t| used(t)).copied();
        let cooking = matches!(
            recipe_type,
            Some("smelting" | "blasting" | "smoking" | "campfire")
        );
        return builtins::data_fields(keyword)
            .filter(|f| !used(f.name))
            .filter(|f| {
                // Only the fields the chosen recipe type accepts.
                match (keyword, f.name) {
                    ("recipe", name) if builtins::RECIPE_TYPES.contains(&name) => {
                        recipe_type.is_none()
                    }
                    ("recipe", "key") => recipe_type == Some("shaped"),
                    ("recipe", "experience" | "time") => cooking,
                    _ => true,
                }
            })
            .map(|f| Completion {
                detail: Some(f.doc.split(". ").next().unwrap_or(f.doc).to_string()),
                documentation: Some(f.doc.into()),
                insert: Some(format!("{}: {},", f.name, f.snippet)),
                snippet: true,
                ..Completion::plain(f.name, CompletionKind::Field)
            })
            .collect();
    };
    match field.value {
        FieldKind::Item => {
            let mut out = registry("Item");
            if field.name == "result" {
                out.extend(declared(DefKind::CustomItem));
            }
            out
        }
        FieldKind::Ingredient | FieldKind::Ingredients | FieldKind::Key => {
            let mut out = registry("Item");
            out.extend(
                scope_names(analysis, module)
                    .into_iter()
                    .filter(|(_, def)| {
                        matches!(
                            analysis.def(*def).info,
                            DefInfo::Data {
                                tag_type: Some("Item"),
                                ..
                            }
                        )
                    })
                    .map(|(name, def)| def_completion(analysis, &name, def)),
            );
            out
        }
        FieldKind::TagValues => {
            let ty = tag_type.unwrap_or("Item");
            let mut out = registry(ty);
            out.extend(
                scope_names(analysis, module)
                    .into_iter()
                    .filter(|(_, def)| {
                        matches!(&analysis.def(*def).info, DefInfo::Data { tag_type: Some(t), .. } if *t == ty)
                    })
                    .map(|(name, def)| def_completion(analysis, &name, def)),
            );
            out
        }
        FieldKind::Registry(name) => registry(name),
        FieldKind::Bool => vec![
            Completion::plain("true", CompletionKind::Keyword),
            Completion::plain("false", CompletionKind::Keyword),
        ],
        _ => Vec::new(),
    }
}

fn item_snippets(after_pub: bool) -> Vec<Completion> {
    let mut all = vec![
        Completion::snippet("fn", "function", "fn ${1:name}() {\n    $0\n}"),
        Completion::snippet(
            "fn ->",
            "function returning a value",
            "fn ${1:name}(${2:value}: number) -> number {\n    return ${3:value};\n}",
        ),
        Completion::snippet(
            "tick fn",
            "function run every tick",
            "#[tick]\nfn ${1:name}() {\n    $0\n}",
        ),
        Completion::snippet(
            "event fn",
            "function run on a player event",
            "#[on(${1:death})]\nfn ${2:name}() {\n    $0\n}",
        ),
        Completion::snippet(
            "player state",
            "per-player State",
            "player state ${1:Name} {\n    ${2:field}: number = ${3:0},\n}",
        ),
        Completion::snippet(
            "global state",
            "server-wide State",
            "global state ${1:Name} {\n    ${2:field}: number = ${3:0},\n}",
        ),
        Completion::snippet(
            "entity state",
            "State attached to individual entities",
            "entity state ${1:Name} {\n    ${2:field}: number = ${3:0},\n}",
        ),
        Completion::snippet(
            "enum",
            "phases stored in State",
            "enum ${1:Name} {\n    ${2:First},\n    ${3:Second},\n}",
        ),
        Completion::snippet(
            "flow",
            "state machine",
            "flow ${1:Name} on ${2:State}.${3:field} {\n    ${4:From} -> ${5:To};\n}",
        ),
        Completion::snippet("const", "constant", "const ${1:NAME}: number = ${2:0};"),
        Completion::snippet(
            "item",
            "custom item",
            "item ${1:Name} {\n    base: Item::${2:DiamondSword},\n    name: \"${3:Name}\",\n}",
        ),
        Completion::snippet(
            "recipe shaped",
            "crafting-table recipe",
            "recipe ${1:Name} {\n    shaped: [\"${2:AAA}\", \"${3: B }\", \"${4: B }\"],\n    key: { A: Item::${5:Diamond}, B: Item::${6:Stick} },\n    result: Item::${7:DiamondPickaxe},\n}",
        ),
        Completion::snippet(
            "recipe shapeless",
            "shapeless recipe",
            "recipe ${1:Name} {\n    shapeless: [Item::${2:Poppy}],\n    result: Item::${3:RedDye},\n}",
        ),
        Completion::snippet(
            "recipe smelting",
            "furnace recipe",
            "recipe ${1:Name} {\n    smelting: Item::${2:RawIron},\n    result: Item::${3:IronIngot},\n    experience: ${4:0.7},\n}",
        ),
        Completion::snippet(
            "tag",
            "group of items, blocks, or entities",
            "tag ${1:Name}: ${2:Item} { values: [${3}] }",
        ),
        Completion::snippet("mod", "module", "mod ${1:name};"),
        Completion::snippet("use", "import", "use ${1:crate}::${2:item};"),
        Completion::snippet(
            "extern fn",
            "function from another pack",
            "extern \"${1:namespace:path}\" fn ${2:name}();",
        ),
        Completion::plain("pub", CompletionKind::Keyword),
    ];
    if after_pub {
        // Attributes come before `pub`, and `pub` cannot repeat.
        all.retain(|c| !matches!(c.label.as_str(), "tick fn" | "event fn" | "pub"));
    }
    all
}

/// Parameter help for the call being typed at `offset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelp {
    /// Full signature label, such as `give_item(item: Item, count?: number)`.
    pub label: String,
    /// Byte ranges of each parameter within `label`.
    pub params: Vec<(usize, usize, String)>,
    /// Index of the parameter being typed.
    pub active: usize,
    /// Markdown documentation for the callee.
    pub documentation: Option<String>,
}

/// Signature help for a builtin, macro, or function call around `offset`.
pub fn signature_help(analysis: &Analysis, file: FileId, offset: usize) -> Option<SignatureHelp> {
    use crate::builtins::{self, BuiltinKind, ParamKind};
    let text = &analysis.package.files[file.0].text;
    let offset = offset.min(text.len());
    let bytes = text.as_bytes();
    // Walk back to the unmatched `(`, counting top-level commas.
    let mut depth = 0i32;
    let mut commas = Vec::new();
    let mut in_string = false;
    let mut open = None;
    let mut i = offset;
    while i > 0 {
        i -= 1;
        let c = bytes[i];
        if c == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match c {
            b')' => depth += 1,
            b'(' if depth == 0 => {
                open = Some(i);
                break;
            }
            b'(' => depth -= 1,
            b',' if depth == 0 => commas.push(i),
            b';' | b'{' | b'}' if depth == 0 => return None,
            _ => {}
        }
    }
    let open = open?;
    let mut name_end = open;
    let is_macro = name_end > 0 && bytes[name_end - 1] == b'!';
    if is_macro {
        name_end -= 1;
    }
    let name_start = text[..name_end]
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
        .last()
        .map(|(i, _)| i)?;
    let name = &text[name_start..name_end];
    let current_arg_start = commas.first().map(|c| c + 1).unwrap_or(open + 1);
    let current = text[current_arg_start..offset].trim_start();
    let named = current
        .split_once('=')
        .filter(|(key, rest)| {
            !rest.starts_with('=')
                && key
                    .trim()
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        .map(|(key, _)| key.trim().to_string());

    let mut label = String::new();
    let mut params = Vec::new();
    let documentation;
    let mut push = |label: &mut String, text: String, name: String| {
        if !params.is_empty() {
            label.push_str(", ");
        }
        let start = label.len();
        label.push_str(&text);
        params.push((start, label.len(), name));
    };
    let module = analysis
        .package
        .module_at(file, offset)
        .unwrap_or(ModuleId(0));
    let kinds = if is_macro {
        vec![BuiltinKind::Macro]
    } else {
        vec![
            BuiltinKind::Statement,
            BuiltinKind::Expression,
            BuiltinKind::Condition,
        ]
    };
    let builtin = (!scope_names(analysis, module).iter().any(|(n, _)| n == name))
        .then(|| kinds.iter().find_map(|kind| builtins::find(name, *kind)))
        .flatten();
    if let Some(builtin) = builtin {
        label.push_str(builtin.name);
        if is_macro {
            label.push('!');
        }
        label.push('(');
        for param in builtin.params {
            let kind = match param.kind {
                ParamKind::Registry(registry) => registry,
                ParamKind::Format => "text",
                ParamKind::Text => "string",
                ParamKind::Int => "number",
                ParamKind::Number => "number",
                ParamKind::Bool => "bool",
                ParamKind::Function => "fn",
                ParamKind::Range => "range",
                ParamKind::EntityState => "EntityState",
            };
            let rendered = if param.required {
                format!("{}: {kind}", param.name)
            } else {
                format!("{}?: {kind}", param.name)
            };
            push(&mut label, rendered, param.name.to_string());
        }
        label.push(')');
        documentation = Some(builtins::describe(builtin));
    } else {
        let segments: Vec<&str> = name.split("::").collect();
        let Target::Def(def) = resolve_names(analysis, module, &segments)? else {
            return None;
        };
        let d = analysis.def(def);
        let DefInfo::Function { signature, .. } = &d.info else {
            return None;
        };
        label.push_str(&d.name);
        label.push('(');
        let inner = signature
            .strip_prefix('(')
            .and_then(|s| s.split_once(')'))
            .map(|(inner, _)| inner)
            .unwrap_or("");
        for param in inner.split(", ").filter(|p| !p.is_empty()) {
            let param_name = param.split(':').next().unwrap_or(param).to_string();
            push(&mut label, param.to_string(), param_name);
        }
        label.push(')');
        if let Some((_, ret)) = signature.split_once(") ") {
            label.push(' ');
            label.push_str(ret);
        }
        documentation = Some(describe(analysis, def));
    }
    let active = named
        .and_then(|key| params.iter().position(|(_, _, n)| *n == key))
        .unwrap_or(commas.len());
    Some(SignatureHelp {
        label,
        params,
        active,
        documentation,
    })
}

fn is_ancestor(analysis: &Analysis, ancestor: ModuleId, module: ModuleId) -> bool {
    let mut cursor = Some(module);
    while let Some(m) = cursor {
        if m == ancestor {
            return true;
        }
        cursor = analysis.package.module(m).parent;
    }
    false
}

/// One outline entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// Name.
    pub name: String,
    /// Short detail, such as a signature.
    pub detail: Option<String>,
    /// Definition kind.
    pub kind: DefKind,
    /// Whole declaration.
    pub span: Span,
    /// Declared name.
    pub selection: Span,
    /// Nested entries: fields, variants, and inline module items.
    pub children: Vec<Symbol>,
}

/// Hierarchical outline for a file.
pub fn document_symbols(analysis: &Analysis, file: FileId) -> Vec<Symbol> {
    let Some(top) = analysis
        .package
        .modules
        .iter()
        .position(|m| m.file == file)
        .map(ModuleId)
    else {
        return Vec::new();
    };
    module_symbols(analysis, file, top)
}

fn module_symbols(analysis: &Analysis, file: FileId, module: ModuleId) -> Vec<Symbol> {
    analysis
        .defs
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            d.module == module && d.parent.is_none() && d.file == file && d.kind != DefKind::Local
        })
        .map(|(_, d)| {
            let children = match &d.info {
                DefInfo::State { fields: list, .. } | DefInfo::Enum { variants: list } => list
                    .iter()
                    .map(|child| {
                        let c = analysis.def(*child);
                        Symbol {
                            name: c.name.clone(),
                            detail: None,
                            kind: c.kind,
                            span: c.span,
                            selection: c.name_span,
                            children: Vec::new(),
                        }
                    })
                    .collect(),
                DefInfo::Module {
                    target: Some(target),
                } if analysis.package.module(*target).file == file => {
                    module_symbols(analysis, file, *target)
                }
                _ => Vec::new(),
            };
            let detail = match &d.info {
                DefInfo::Function { context, .. } => Some(context.name().to_string()),
                DefInfo::State { scope, .. } => Some(
                    match scope {
                        Scope::Player => "player",
                        Scope::Global => "global",
                        Scope::Entity => "entity",
                    }
                    .to_string(),
                ),
                DefInfo::Const { value: Some(v), .. } => Some(v.to_string()),
                _ => None,
            };
            Symbol {
                name: d.name.clone(),
                detail,
                kind: d.kind,
                span: d.span,
                selection: d.name_span,
                children,
            }
        })
        .collect()
}

/// An inline annotation rendered by the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHint {
    /// Byte offset the hint is attached after.
    pub offset: usize,
    /// Hint text.
    pub label: String,
    /// Markdown tooltip.
    pub tooltip: Option<String>,
}

/// Inferred execution context after each function signature.
pub fn inlay_hints(analysis: &Analysis, file: FileId) -> Vec<InlayHint> {
    let text = &analysis.package.files[file.0].text;
    analysis
        .defs
        .iter()
        .filter(|d| d.file == file)
        .filter_map(|d| {
            let DefInfo::Function {
                context,
                player_reason,
                ..
            } = &d.info
            else {
                return None;
            };
            let after = text.get(d.name_span.end..)?;
            let close = after.find(')')?;
            Some(InlayHint {
                offset: d.name_span.end + close + 1,
                label: format!(": {}", context.name()),
                tooltip: Some(match (context, player_reason) {
                    (Context::Player, Some(reason)) => {
                        format!("Runs as a player because it {reason}.")
                    }
                    (Context::Player, None) => "Runs as a player.".into(),
                    (Context::Entity, Some(reason)) => {
                        format!("Runs as an entity because it {reason}.")
                    }
                    (Context::Entity, None) => "Runs as an entity.".into(),
                    (Context::Server, _) => "Runs on the server; callable from anywhere.".into(),
                }),
            })
        })
        .collect()
}
