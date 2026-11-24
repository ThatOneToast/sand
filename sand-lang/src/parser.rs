//! Pest-backed parser producing the span-carrying [`ast`](crate::ast).
//!
//! The grammar recovers from malformed items and statements. Each recovered
//! chunk is re-parsed with a strict entry rule purely to report the precise
//! position and expected tokens, so authors see the same message the strict
//! parser would give while the rest of the file still produces a usable tree.

use crate::ast::*;
use pest::Parser as _;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use std::sync::OnceLock;

#[derive(pest_derive::Parser)]
#[grammar = "sand.pest"]
struct SandParser;

type P<'i> = Pair<'i, Rule>;

/// Parse a complete source file.
///
/// Parsing never fails as a whole: malformed regions are returned as error
/// nodes, and every syntax problem is listed in the returned errors.
pub fn parse(source: &str) -> (SourceFile, Vec<SyntaxError>) {
    pest::set_error_detail(true);
    let mut errors = Vec::new();
    let file = match SandParser::parse(Rule::file, source) {
        Ok(mut pairs) => {
            let file = pairs.next().expect("file rule");
            let mut pending = vec![(file.clone(), false)];
            while let Some((pair, nested)) = pending.pop() {
                let in_closure = nested || pair.as_rule() == Rule::closure_expr;
                if in_closure && pair.as_rule() == Rule::bad_stmt {
                    Builder {
                        errors: &mut errors,
                    }
                    .bad_stmt(pair);
                    continue;
                }
                if pair.as_rule() == Rule::string_inner {
                    if let Err((offset, message)) = unescape_checked(pair.as_str()) {
                        let start = pair.as_span().start() + offset;
                        errors.push(SyntaxError {
                            message,
                            span: Span::new(start, start + 1),
                        });
                    }
                } else {
                    pending.extend(pair.into_inner().map(|pair| (pair, in_closure)));
                }
            }
            let mut builder = Builder {
                errors: &mut errors,
            };
            builder.file(file)
        }
        // The file rule only fails when recovery cannot make progress, for
        // example inside an unterminated block comment.
        Err(error) => {
            errors.push(convert_error(&error, source, 0));
            SourceFile::default()
        }
    };
    (file, errors)
}

/// Delimiters belonging to type applications, for whitespace-only formatting.
pub(crate) fn type_delimiters(source: &str) -> std::collections::BTreeSet<usize> {
    let mut result = std::collections::BTreeSet::new();
    let Ok(pairs) = SandParser::parse(Rule::file, source) else {
        return result;
    };
    let mut pending: Vec<_> = pairs.collect();
    while let Some(pair) = pending.pop() {
        if matches!(
            pair.as_rule(),
            Rule::generic_params | Rule::fn_generic_params | Rule::type_arguments
        ) {
            let location = pair.as_span();
            result.insert(location.start());
            result.insert(location.end() - 1);
        }
        if pair.as_rule() == Rule::applied_ty {
            let location = pair.as_span();
            let path = pair.clone().into_inner().next().expect("applied type path");
            if let Some(open) = source[path.as_span().end()..location.end()].find('<') {
                result.insert(path.as_span().end() + open);
            }
            if let Some(close) = pair.as_str().rfind('>') {
                result.insert(location.start() + close);
            }
        }
        pending.extend(pair.into_inner());
    }
    result
}

struct Builder<'e> {
    errors: &'e mut Vec<SyntaxError>,
}

fn span(pair: &P<'_>) -> Span {
    let s = pair.as_span();
    Span::new(s.start(), s.end())
}

fn is_keyword(rule: Rule) -> bool {
    matches!(
        rule,
        Rule::kw_fn
            | Rule::kw_struct
            | Rule::kw_impl
            | Rule::kw_trait
            | Rule::kw_move
            | Rule::kw_mod
            | Rule::kw_use
            | Rule::kw_const
            | Rule::kw_enum
            | Rule::kw_state
            | Rule::kw_flow
            | Rule::kw_event
            | Rule::kw_extern
            | Rule::kw_if
            | Rule::kw_else
            | Rule::kw_for
            | Rule::kw_break
            | Rule::kw_emit
            | Rule::kw_loop
            | Rule::kw_in
            | Rule::kw_as
            | Rule::kw_on
            | Rule::kw_every
            | Rule::kw_priority
            | Rule::kw_let
            | Rule::kw_mut
            | Rule::kw_return
            | Rule::kw_continue
            | Rule::kw_match
            | Rule::kw_while
    )
}

/// Significant children: keyword tokens carry no information beyond the rule.
fn children<'i>(pair: P<'i>) -> impl Iterator<Item = P<'i>> {
    pair.into_inner().filter(|p| !is_keyword(p.as_rule()))
}

fn fn_signature<'i>(mut parts: impl Iterator<Item = P<'i>>) -> TraitMethod {
    let name = ident(parts.next().expect("function name"));
    let mut params = Vec::new();
    let mut ret = None;
    let mut receiver = None;
    for part in parts {
        match part.as_rule() {
            Rule::fn_params => {
                for entry in part.into_inner() {
                    if entry.as_rule() == Rule::receiver {
                        let text = entry.as_str();
                        receiver = Some(Receiver {
                            mutable: text.contains("mut"),
                            borrowed: text.starts_with('&'),
                            span: span(&entry),
                        });
                    } else {
                        params.extend(entry.into_inner().map(parameter));
                    }
                }
            }
            Rule::ret_ty => ret = part.into_inner().next().map(value_type),
            _ => unreachable!("unexpected signature syntax"),
        }
    }
    TraitMethod {
        name,
        receiver,
        params,
        ret,
    }
}

fn parameter(pair: P<'_>) -> Param {
    let s = span(&pair);
    let mut pieces = pair.into_inner();
    let first = pieces.next().expect("param name");
    let mutable = first.as_rule() == Rule::mut_kw;
    let name = ident(if mutable {
        pieces.next().expect("param name")
    } else {
        first
    });
    let ty = value_type(pieces.next().expect("param type"));
    Param {
        mutable,
        name,
        ty,
        span: s,
    }
}

fn ident(pair: P<'_>) -> Ident {
    Ident {
        name: pair.as_str().to_string(),
        span: span(&pair),
    }
}

fn doc_text(pair: &P<'_>, prefix: &str) -> String {
    let text = pair.as_str().strip_prefix(prefix).unwrap_or(pair.as_str());
    text.strip_prefix(' ')
        .unwrap_or(text)
        .trim_end()
        .to_string()
}

pub(crate) fn unescape(raw: &str) -> String {
    unescape_checked(raw).unwrap_or_else(|_| raw.to_owned())
}

fn unescape_checked(raw: &str) -> Result<String, (usize, String)> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.char_indices();
    while let Some((offset, character)) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        let escaped = match chars.next().map(|(_, character)| character) {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('0') => '\0',
            Some('b') => '\u{0008}',
            Some('f') => '\u{000c}',
            Some(other @ ('"' | '\\' | '{' | '}')) => other,
            Some('u') => {
                if chars.next().map(|(_, character)| character) != Some('{') {
                    return Err((offset, "Unicode escapes use `\\u{hex}`".into()));
                }
                let mut hex = String::new();
                let mut closed = false;
                for (_, character) in chars.by_ref() {
                    if character == '}' {
                        closed = true;
                        break;
                    }
                    if !character.is_ascii_hexdigit() || hex.len() == 6 {
                        return Err((
                            offset,
                            "Unicode escapes require one to six hexadecimal digits".into(),
                        ));
                    }
                    hex.push(character);
                }
                if !closed {
                    return Err((offset, "unclosed Unicode escape".into()));
                }
                u32::from_str_radix(&hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or_else(|| {
                        (
                            offset,
                            "Unicode escape must name a valid Unicode scalar value".into(),
                        )
                    })?
            }
            Some(other) => return Err((offset, format!("unknown string escape `\\{other}`"))),
            None => return Err((offset, "incomplete string escape".into())),
        };
        out.push(escaped);
    }
    Ok(out)
}

fn string(pair: P<'_>) -> String {
    let inner = pair.into_inner().next().map(|p| p.as_str()).unwrap_or("");
    unescape(inner)
}

fn pratt() -> &'static PrattParser<Rule> {
    static PRATT: OnceLock<PrattParser<Rule>> = OnceLock::new();
    PRATT.get_or_init(|| {
        PrattParser::new()
            .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
            .op(Op::infix(Rule::mul, Assoc::Left)
                | Op::infix(Rule::div, Assoc::Left)
                | Op::infix(Rule::rem, Assoc::Left))
            // As in Rust, `-x as T` casts the negated value.
            .op(Op::postfix(Rule::cast))
            .op(Op::prefix(Rule::neg))
            .op(Op::postfix(Rule::method_postfix)
                | Op::postfix(Rule::field_postfix)
                | Op::postfix(Rule::index_postfix))
    })
}

impl Builder<'_> {
    fn file(&mut self, pair: P<'_>) -> SourceFile {
        let mut file = SourceFile::default();
        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::inner_doc => file.docs.push(doc_text(&child, "//!")),
                Rule::inner_attr => {
                    let s = span(&child);
                    let body = child.into_inner().next().expect("attr body");
                    file.inner_attrs.push(self.attr_body(body, s));
                }
                Rule::item => file.items.push(self.item(child)),
                Rule::bad_item | Rule::bad_item_inline => file.items.push(self.bad_item(child)),
                _ => {}
            }
        }
        file
    }

    fn bad_item(&mut self, pair: P<'_>) -> Item {
        let s = span(&pair);
        let error =
            strict_error(Rule::item_only, pair.as_str(), s.start).unwrap_or_else(|| SyntaxError {
                message: "expected an item".into(),
                span: s,
            });
        self.errors.push(error.clone());
        Item {
            docs: Vec::new(),
            attrs: Vec::new(),
            is_pub: false,
            kind: ItemKind::Error(error),
            span: s,
        }
    }

    fn bad_stmt(&mut self, pair: P<'_>) -> SyntaxError {
        let s = span(&pair);
        // The builtin-call spelling of `emit`, from before it was a statement.
        if let Some(rest) = pair.as_str().trim_start().strip_prefix("emit")
            && rest.trim_start().starts_with('(')
        {
            let inner = rest.trim_start()[1..]
                .trim_end()
                .trim_end_matches(';')
                .trim_end();
            let inner = inner.strip_suffix(')').unwrap_or(inner);
            let (event, payload) = inner.split_once(',').unwrap_or((inner, ""));
            let error = SyntaxError {
                message: format!("write `emit {}({});`", event.trim(), payload.trim()),
                span: s,
            };
            self.errors.push(error.clone());
            return error;
        }
        let error =
            strict_error(Rule::stmt_only, pair.as_str(), s.start).unwrap_or_else(|| SyntaxError {
                message: "expected a statement".into(),
                span: s,
            });
        self.errors.push(error.clone());
        error
    }

    fn attr_body(&mut self, body: P<'_>, s: Span) -> Attr {
        let mut inner = body.into_inner();
        let path = path(inner.next().expect("attr path"));
        let mut args = Vec::new();
        if let Some(list) = inner.next() {
            for arg in list.into_inner() {
                let arg_span = span(&arg);
                let mut parts = arg.into_inner();
                let first = parts.next().expect("attr arg");
                let (key, value) = if first.as_rule() == Rule::ident {
                    (Some(ident(first)), parts.next().expect("attr value"))
                } else {
                    (None, first)
                };
                let value = value.into_inner().next().expect("attr value kind");
                let value = match value.as_rule() {
                    Rule::string => {
                        let s = span(&value);
                        AttrValue::Str(string(value), s)
                    }
                    Rule::range => AttrValue::Range(self.range(value)),
                    _ => AttrValue::Expr(self.expr(value)),
                };
                args.push(AttrArg {
                    key,
                    value,
                    span: arg_span,
                });
            }
        }
        Attr {
            path,
            args,
            span: s,
        }
    }

    fn item(&mut self, pair: P<'_>) -> Item {
        let s = span(&pair);
        let mut docs = Vec::new();
        let mut attrs = Vec::new();
        let mut is_pub = false;
        let mut kind = None;
        for child in pair.into_inner() {
            match child.as_rule() {
                Rule::outer_doc => docs.push(doc_text(&child, "///")),
                Rule::attr => {
                    let a = span(&child);
                    let body = child.into_inner().next().expect("attr body");
                    attrs.push(self.attr_body(body, a));
                }
                Rule::vis => is_pub = true,
                _ => kind = Some(self.item_kind(child)),
            }
        }
        Item {
            docs,
            attrs,
            is_pub,
            kind: kind.expect("item kind"),
            span: s,
        }
    }

    fn item_kind(&mut self, pair: P<'_>) -> ItemKind {
        let rule = pair.as_rule();
        let mut inner = children(pair);
        match rule {
            Rule::mod_item => {
                let name = ident(inner.next().expect("mod name"));
                let inline = inner.next().map(|body| {
                    body.into_inner()
                        .map(|child| match child.as_rule() {
                            Rule::item => self.item(child),
                            _ => self.bad_item(child),
                        })
                        .collect()
                });
                ItemKind::Mod(ModItem { name, inline })
            }
            Rule::use_item => ItemKind::Use(UseItem {
                tree: use_tree(inner.next().expect("use tree")),
            }),
            Rule::const_item => {
                let name = ident(inner.next().expect("const name"));
                let ty = value_type(inner.next().expect("const type"));
                let value = self.expr(inner.next().expect("const value"));
                ItemKind::Const(ConstItem { name, ty, value })
            }
            Rule::enum_item => {
                let name = ident(inner.next().expect("enum name"));
                let variants = inner
                    .map(|variant| {
                        let mut docs = Vec::new();
                        let mut name = None;
                        let mut value = None;
                        for part in variant.into_inner() {
                            match part.as_rule() {
                                Rule::outer_doc => docs.push(doc_text(&part, "///")),
                                Rule::ident => name = Some(ident(part)),
                                _ => value = Some(self.expr(part)),
                            }
                        }
                        Variant {
                            docs,
                            name: name.expect("variant name"),
                            value,
                        }
                    })
                    .collect();
                ItemKind::Enum(EnumItem { name, variants })
            }
            Rule::state_item => {
                let scope_pair = inner.next().expect("scope");
                let scope_span = span(&scope_pair);
                let scope = match scope_pair.as_str() {
                    "player" => Scope::Player,
                    "entity" => Scope::Entity,
                    _ => Scope::Global,
                };
                let name = ident(inner.next().expect("state name"));
                let fields = inner.map(|field| self.field(field)).collect();
                ItemKind::State(StateItem {
                    scope,
                    scope_span,
                    name,
                    fields,
                })
            }
            Rule::flow_item => {
                let name = ident(inner.next().expect("flow name"));
                let on = place(inner.next().expect("flow field"));
                let mut flow = FlowItem {
                    name,
                    on,
                    transitions: Vec::new(),
                    hooks: Vec::new(),
                    errors: Vec::new(),
                };
                for entry in inner {
                    match entry.as_rule() {
                        Rule::transition => flow.transitions.push(self.transition(entry)),
                        Rule::hook => flow.hooks.push(self.hook(entry)),
                        _ => {
                            let error = self.bad_stmt(entry);
                            flow.errors.push(error);
                        }
                    }
                }
                ItemKind::Flow(flow)
            }
            Rule::struct_item => {
                let name = ident(inner.next().expect("struct name"));
                let mut generics = Vec::new();
                let mut fields = Vec::new();
                for part in inner {
                    if part.as_rule() == Rule::generic_params {
                        generics = part.into_inner().map(ident).collect();
                    } else {
                        fields.push(self.field(part));
                    }
                }
                ItemKind::Struct(StructItem {
                    generics,
                    name,
                    fields,
                })
            }
            Rule::trait_item => {
                let name = ident(inner.next().expect("trait name"));
                ItemKind::Trait(TraitItem {
                    name,
                    methods: inner.map(|method| fn_signature(children(method))).collect(),
                })
            }
            Rule::impl_item => {
                let mut generics = Vec::new();
                let mut target = None;
                let mut contract = None;
                let mut methods = Vec::new();
                for part in inner {
                    match part.as_rule() {
                        Rule::generic_params => generics = part.into_inner().map(ident).collect(),
                        Rule::impl_contract => contract = children(part).next().map(path),
                        Rule::path => target = Some(path(part)),
                        Rule::type_arguments => {
                            let target = target.as_mut().expect("impl target");
                            target.applications.push(PathApplication {
                                segment: target.segments.len() - 1,
                                arguments: part.into_inner().map(value_type).collect(),
                            });
                        }
                        _ => methods.push(self.item(part)),
                    }
                }
                ItemKind::Impl(ImplItem {
                    generics,
                    contract,
                    target: target.expect("impl target"),
                    methods,
                })
            }
            Rule::data_item => {
                let kind = match inner.next().expect("data kind").as_str() {
                    "item" => DataKind::Item,
                    "recipe" => DataKind::Recipe,
                    "bossbar" => DataKind::BossBar,
                    "loot" => DataKind::Loot,
                    "damage_type" => DataKind::DamageType,
                    "predicate" => DataKind::Predicate,
                    "advancement" => DataKind::Advancement,
                    "dialog" => DataKind::Dialog,
                    "dimension" => DataKind::Dimension,
                    "biome" => DataKind::Biome,
                    "structure_template" => DataKind::StructureTemplate,
                    "inventory_transfer" => DataKind::InventoryTransfer,
                    "hud" => DataKind::Hud,
                    "menu" => DataKind::Menu,
                    "projectile" => DataKind::Projectile,
                    "progression" => DataKind::Progression,
                    "session" => DataKind::Session,
                    "ability" => DataKind::Ability,
                    _ => DataKind::Tag,
                };
                let name = ident(inner.next().expect("data name"));
                let mut next = inner.next().expect("data body");
                let mut tag_type = None;
                if next.as_rule() == Rule::ident {
                    tag_type = Some(ident(next));
                    next = inner.next().expect("data body");
                }
                let object = next;
                let body = span(&object);
                let fields = self.data_fields(object);
                ItemKind::Data(DataItem {
                    kind,
                    name,
                    tag_type,
                    fields,
                    body,
                })
            }
            Rule::event_item => {
                let name = ident(inner.next().expect("event name"));
                let mut params = Vec::new();
                for part in inner {
                    if part.as_rule() == Rule::params {
                        params = part.into_inner().map(parameter).collect();
                    }
                }
                ItemKind::Event(EventItem { name, params })
            }
            Rule::extern_fn => {
                let id_pair = inner.next().expect("extern id");
                let id_span = span(&id_pair);
                let id = string(id_pair);
                let name = ident(inner.next().expect("extern name"));
                ItemKind::ExternFn(ExternFnItem { id, id_span, name })
            }
            Rule::fn_item => {
                let mut generics = Vec::new();
                let mut bounds = Vec::new();
                let mut body = None;
                let signature = fn_signature(inner.filter_map(|part| match part.as_rule() {
                    Rule::fn_generic_params => {
                        for parameter in part.into_inner() {
                            let mut entries = parameter.into_inner();
                            let name = ident(entries.next().expect("type parameter"));
                            bounds.extend(entries.map(|bound| (name.clone(), path(bound))));
                            generics.push(name);
                        }
                        None
                    }
                    Rule::block => {
                        body = Some(self.block(part));
                        None
                    }
                    _ => Some(part),
                }));
                ItemKind::Fn(FnItem {
                    generics,
                    bounds,
                    name: signature.name,
                    receiver: signature.receiver,
                    params: signature.params,
                    ret: signature.ret,
                    body: body.expect("fn body"),
                })
            }
            other => unreachable!("unexpected item rule {other:?}"),
        }
    }

    fn field(&mut self, pair: P<'_>) -> Field {
        let s = span(&pair);
        let mut docs = Vec::new();
        let mut is_pub = false;
        let mut attrs = Vec::new();
        let mut name = None;
        let mut ty = None;
        let mut default = None;
        for part in pair.into_inner() {
            match part.as_rule() {
                Rule::vis => is_pub = true,
                Rule::outer_doc => docs.push(doc_text(&part, "///")),
                Rule::attr => {
                    let a = span(&part);
                    let body = part.into_inner().next().expect("attr body");
                    attrs.push(self.attr_body(body, a));
                }
                Rule::ident => name = Some(ident(part)),
                Rule::ty => ty = Some(value_type(part)),
                _ => default = Some(self.expr(part)),
            }
        }
        Field {
            is_pub,
            docs,
            attrs,
            name: name.expect("field name"),
            ty: ty.expect("field type"),
            default,
            span: s,
        }
    }

    fn transition(&mut self, pair: P<'_>) -> Transition {
        let s = span(&pair);
        let mut exprs = Vec::new();
        let mut guard = None;
        let mut priority = None;
        let mut saw_priority = false;
        for part in pair.into_inner() {
            match part.as_rule() {
                Rule::kw_priority => saw_priority = true,
                Rule::cond => guard = Some(self.cond(part)),
                Rule::expr if saw_priority => priority = Some(self.expr(part)),
                Rule::expr => exprs.push(self.expr(part)),
                _ => {}
            }
        }
        let mut exprs = exprs.into_iter();
        Transition {
            from: exprs.next().expect("transition source"),
            to: exprs.next().expect("transition destination"),
            guard,
            priority,
            span: s,
        }
    }

    fn hook(&mut self, pair: P<'_>) -> Hook {
        let s = span(&pair);
        let mut phase = HookPhase::Tick;
        let mut state = None;
        let mut every = None;
        let mut target = None;
        for part in children(pair) {
            match part.as_rule() {
                Rule::hook_phase => {
                    phase = match part.as_str() {
                        "enter" => HookPhase::Enter,
                        "exit" => HookPhase::Exit,
                        _ => HookPhase::Tick,
                    }
                }
                Rule::expr if state.is_none() => state = Some(self.expr(part)),
                Rule::expr => every = Some(self.expr(part)),
                Rule::path => target = Some(HookTarget::Function(path(part))),
                Rule::block => target = Some(HookTarget::Block(self.block(part))),
                _ => {}
            }
        }
        Hook {
            phase,
            state: state.expect("hook state"),
            every,
            target: target.expect("hook target"),
            span: s,
        }
    }

    fn block(&mut self, pair: P<'_>) -> Block {
        let s = span(&pair);
        let stmts = pair
            .into_inner()
            .map(|child| match child.as_rule() {
                Rule::stmt => self.stmt(child),
                _ => Stmt::Error(self.bad_stmt(child)),
            })
            .collect();
        Block { stmts, span: s }
    }

    fn stmt(&mut self, pair: P<'_>) -> Stmt {
        let inner = pair
            .into_inner()
            .find(|p| p.as_rule() != Rule::outer_doc)
            .expect("statement body");
        let s = span(&inner);
        match inner.as_rule() {
            Rule::match_stmt => {
                let mut parts = children(inner);
                let value = expr(parts.next().expect("matched value"));
                let arms = parts
                    .map(|arm| {
                        let arm_span = span(&arm);
                        let mut parts = arm.into_inner().peekable();
                        let pattern = match_pattern(parts.next().expect("pattern"));
                        let guard = parts
                            .next_if(|part| part.as_rule() == Rule::match_guard)
                            .map(|guard| {
                                self.cond(children(guard).next().expect("guard condition"))
                            });
                        let body = self.block(parts.next().expect("arm body"));
                        MatchArm {
                            pattern,
                            guard,
                            body,
                            span: arm_span,
                        }
                    })
                    .collect();
                Stmt::Match {
                    value,
                    arms,
                    span: s,
                }
            }
            Rule::if_stmt => Stmt::If(Box::new(self.if_stmt(inner))),
            Rule::for_stmt => {
                let mut parts = children(inner);
                let binding = ident(parts.next().expect("binding"));
                let source = parts.next().expect("iterator");
                if source.as_rule() == Rule::count_range {
                    let mut range_parts = source.into_inner();
                    let range = range(range_parts.next().expect("range"));
                    let mut reverse = false;
                    let mut step = None;
                    for part in range_parts {
                        match part.as_rule() {
                            Rule::reverse_range => reverse = true,
                            Rule::range_step => {
                                step = Some(expr(part.into_inner().next().expect("step")))
                            }
                            _ => unreachable!(),
                        }
                    }
                    let body = self.block(parts.next().expect("for body"));
                    return Stmt::ForRange {
                        binding,
                        range,
                        reverse,
                        step,
                        body,
                        span: s,
                    };
                }
                let value = expr(source);
                let body = self.block(parts.next().expect("for body"));
                match value {
                    Expr::Call { path, args, .. }
                        if path.segments.len() == 1
                            && matches!(
                                path.last().name.as_str(),
                                "players" | "entities" | "resolve_entity" | "resolve_player"
                            ) =>
                    {
                        Stmt::For {
                            binding,
                            source: path,
                            args,
                            body,
                            span: s,
                        }
                    }
                    value => Stmt::ForValues {
                        binding,
                        value,
                        body,
                        span: s,
                    },
                }
            }
            Rule::macro_stmt => {
                let mut parts = inner.into_inner();
                let name = ident(parts.next().expect("macro name"));
                let args = parts.next().map(args).unwrap_or_default();
                Stmt::Macro {
                    name,
                    args,
                    span: s,
                }
            }
            Rule::destructure_stmt => {
                let mut parts = children(inner);
                let pattern = value_pattern(parts.next().expect("pattern"));
                let next = parts.next().expect("type or value");
                let (ty, value) = if next.as_rule() == Rule::ty {
                    (
                        Some(value_type(next)),
                        self.value(parts.next().expect("initial value")),
                    )
                } else {
                    (None, self.value(next))
                };
                Stmt::Destructure {
                    pattern,
                    ty,
                    value,
                    span: s,
                }
            }
            Rule::let_stmt => {
                let mut mutable = false;
                let mut name = None;
                let mut ty = None;
                let mut value = None;
                for part in children(inner) {
                    match part.as_rule() {
                        Rule::mut_kw => mutable = true,
                        Rule::ident => name = Some(ident(part)),
                        Rule::ty => ty = Some(value_type(part)),
                        _ => value = Some(self.value(part)),
                    }
                }
                Stmt::Let {
                    mutable,
                    name: name.expect("let name"),
                    ty,
                    value,
                    span: s,
                }
            }
            Rule::return_stmt => Stmt::Return {
                value: children(inner).next().map(|e| self.value(e)),
                span: s,
            },
            Rule::continue_stmt => Stmt::Continue(s),
            Rule::break_stmt => Stmt::Break(s),
            Rule::emit_stmt => {
                let mut parts = children(inner);
                let event = path(parts.next().expect("emitted event"));
                let args = parts.next().map(args).unwrap_or_default();
                Stmt::Emit {
                    event,
                    args,
                    span: s,
                }
            }
            Rule::loop_stmt => {
                let mut parts = children(inner);
                let body = self.block(parts.next().expect("loop body"));
                Stmt::Loop { body, span: s }
            }
            Rule::while_stmt => {
                let mut parts = children(inner);
                let cond = self.cond(parts.next().expect("while condition"));
                let body = self.block(parts.next().expect("while body"));
                Stmt::While {
                    cond,
                    body,
                    span: s,
                }
            }
            Rule::assign_stmt => {
                let mut parts = inner.into_inner();
                let first = parts.next().expect("assign target");
                let target = match expr(first) {
                    Expr::Place(place) => AssignTarget::Place(place),
                    Expr::Path(path) if path.segments.len() == 1 => {
                        AssignTarget::Local(path.segments.into_iter().next().expect("local name"))
                    }
                    value => AssignTarget::Value(Box::new(value)),
                };
                let op = match parts.next().expect("assign op").as_str() {
                    "+=" => AssignOp::Add,
                    "-=" => AssignOp::Sub,
                    "*=" => AssignOp::Mul,
                    "/=" => AssignOp::Div,
                    "%=" => AssignOp::Rem,
                    _ => AssignOp::Set,
                };
                let value = self.value(parts.next().expect("assigned value"));
                Stmt::Assign {
                    target,
                    op,
                    value,
                    span: s,
                }
            }
            Rule::call_stmt => match expr(inner.into_inner().next().expect("call expression")) {
                Expr::Call { path, args, .. } => Stmt::Call {
                    path,
                    args,
                    span: s,
                },
                value => Stmt::Expr(value, s),
            },
            other => unreachable!("unexpected statement rule {other:?}"),
        }
    }

    fn if_stmt(&mut self, pair: P<'_>) -> IfStmt {
        let s = span(&pair);
        let mut parts = children(pair);
        let cond = self.cond(parts.next().expect("if condition"));
        let then = self.block(parts.next().expect("if body"));
        let otherwise = parts.next().map(|next| match next.as_rule() {
            Rule::if_stmt => Else::If(Box::new(self.if_stmt(next))),
            _ => Else::Block(self.block(next)),
        });
        IfStmt {
            cond,
            then,
            otherwise,
            span: s,
        }
    }

    fn cond(&mut self, pair: P<'_>) -> Cond {
        match pair.as_rule() {
            Rule::cond => {
                let mut parts = pair.into_inner();
                let mut acc = self.cond(parts.next().expect("cond operand"));
                for next in parts {
                    acc = Cond::Or(Box::new(acc), Box::new(self.cond(next)));
                }
                acc
            }
            Rule::cond_and => {
                let mut parts = pair.into_inner();
                let mut acc = self.cond(parts.next().expect("cond operand"));
                for next in parts {
                    acc = Cond::And(Box::new(acc), Box::new(self.cond(next)));
                }
                acc
            }
            Rule::cond_unary => {
                let mut nots = Vec::new();
                let mut atom = None;
                for part in pair.into_inner() {
                    if part.as_rule() == Rule::not_op {
                        nots.push(span(&part));
                    } else {
                        atom = Some(self.cond(part));
                    }
                }
                let mut cond = atom.expect("condition atom");
                for not in nots.into_iter().rev() {
                    let s = not.to(cond.span());
                    cond = Cond::Not(Box::new(cond), s);
                }
                cond
            }
            Rule::cond_expr => {
                Cond::Value(expr(pair.into_inner().next().expect("condition value")))
            }
            Rule::cond_call => {
                let s = span(&pair);
                let mut parts = pair.into_inner();
                let path = path(parts.next().expect("predicate"));
                let args = parts.next().map(args).unwrap_or_default();
                Cond::Call {
                    path,
                    args,
                    span: s,
                }
            }
            Rule::comparison => {
                let s = span(&pair);
                let mut parts = pair.into_inner();
                let lhs = self.expr(parts.next().expect("lhs"));
                let op = match parts.next().expect("cmp op").as_str() {
                    "==" => CmpOp::Eq,
                    "!=" => CmpOp::Ne,
                    "<=" => CmpOp::Le,
                    ">=" => CmpOp::Ge,
                    "<" => CmpOp::Lt,
                    _ => CmpOp::Gt,
                };
                let rhs = self.expr(parts.next().expect("rhs"));
                Cond::Compare {
                    lhs,
                    op,
                    rhs,
                    span: s,
                }
            }
            Rule::in_range => {
                let s = span(&pair);
                let mut parts = children(pair);
                let value = self.expr(parts.next().expect("tested value"));
                let range = self.range(parts.next().expect("range"));
                Cond::InRange {
                    value,
                    range,
                    span: s,
                }
            }
            other => unreachable!("unexpected condition rule {other:?}"),
        }
    }

    /// An expression, or a condition used as a boolean value.
    fn value(&mut self, pair: P<'_>) -> Expr {
        if pair.as_rule() == Rule::value_expr {
            let s = span(&pair);
            let cond = pair.into_inner().next().expect("condition value");
            Expr::Cond(Box::new(self.cond(cond)), s)
        } else {
            self.expr(pair)
        }
    }

    fn data_fields(&mut self, object: P<'_>) -> Vec<DataField> {
        object
            .into_inner()
            .map(|field| {
                let s = span(&field);
                let mut parts = field.into_inner();
                let key_pair = parts.next().expect("field key");
                let key = match key_pair.as_rule() {
                    Rule::ident => ident(key_pair),
                    _ => Ident {
                        span: span(&key_pair),
                        name: string(key_pair),
                    },
                };
                let value = self.data_value(parts.next().expect("field value"));
                DataField {
                    key,
                    value,
                    span: s,
                }
            })
            .collect()
    }

    fn data_value(&mut self, pair: P<'_>) -> DataValue {
        let inner = pair.into_inner().next().expect("data value kind");
        let s = span(&inner);
        match inner.as_rule() {
            Rule::data_array => {
                DataValue::Array(inner.into_inner().map(|v| self.data_value(v)).collect(), s)
            }
            Rule::data_object => DataValue::Object(self.data_fields(inner), s),
            _ => DataValue::Expr(self.expr(inner)),
        }
    }

    fn range(&mut self, pair: P<'_>) -> Range {
        range(pair)
    }

    fn expr(&mut self, pair: P<'_>) -> Expr {
        expr(pair)
    }
}

fn record_value(pair: P<'_>) -> Expr {
    let value = pair.into_inner().next().expect("value");
    if value.as_rule() == Rule::cond {
        let location = span(&value);
        Expr::Cond(
            Box::new(
                Builder {
                    errors: &mut Vec::new(),
                }
                .cond(value),
            ),
            location,
        )
    } else {
        expr(value)
    }
}

fn match_pattern(pair: P<'_>) -> MatchPattern {
    let mut parts = pair.into_inner();
    let first = parts.next().expect("match pattern");
    match first.as_rule() {
        Rule::match_wildcard => MatchPattern::Wildcard,
        Rule::match_binding => MatchPattern::Binding(value_pattern(
            children(first).next().expect("binding pattern"),
        )),
        _ => MatchPattern::Constants(std::iter::once(first).chain(parts).map(expr).collect()),
    }
}

fn value_pattern(pair: P<'_>) -> Pattern {
    let location = span(&pair);
    match pair.as_rule() {
        Rule::value_pattern => value_pattern(pair.into_inner().next().expect("pattern")),
        Rule::ignore_pattern => Pattern::Ignore(location),
        Rule::array_pattern => {
            Pattern::Array(pair.into_inner().map(value_pattern).collect(), location)
        }
        Rule::binding_pattern => {
            let mut parts = pair.into_inner().peekable();
            let mutable = parts.peek().is_some_and(|p| p.as_rule() == Rule::mut_kw);
            if mutable {
                parts.next();
            }
            Pattern::Bind {
                name: ident(parts.next().expect("binding name")),
                mutable,
            }
        }
        Rule::record_pattern => {
            let mut parts = pair.into_inner();
            let path = path(parts.next().expect("record type"));
            let mut fields = Vec::new();
            let mut rest = false;
            for part in parts {
                if part.as_rule() == Rule::pattern_rest {
                    rest = true;
                    continue;
                }
                let mut parts = part.into_inner();
                let name = ident(parts.next().expect("field name"));
                let pattern = parts
                    .next()
                    .map(value_pattern)
                    .unwrap_or_else(|| Pattern::Bind {
                        name: name.clone(),
                        mutable: false,
                    });
                fields.push((name, pattern));
            }
            Pattern::Record {
                path,
                fields,
                rest,
                span: location,
            }
        }
        other => unreachable!("unexpected pattern {other:?}"),
    }
}

fn expr(pair: P<'_>) -> Expr {
    pratt()
        .map_primary(|primary| match primary.as_rule() {
            Rule::closure_expr => {
                let location = span(&primary);
                let mut parameters = Vec::new();
                let mut returns = None;
                let mut body = None;
                let mut errors = Vec::new();
                for part in children(primary) {
                    match part.as_rule() {
                        Rule::params => parameters = part.into_inner().map(parameter).collect(),
                        Rule::ret_ty => returns = part.into_inner().next().map(value_type),
                        Rule::block => {
                            body = Some(
                                Builder {
                                    errors: &mut errors,
                                }
                                .block(part),
                            )
                        }
                        _ => unreachable!("closure syntax"),
                    }
                }
                Expr::Closure(Box::new(ClosureExpr {
                    parameters,
                    returns,
                    body: body.expect("closure body"),
                    span: location,
                }))
            }
            Rule::block_expr => {
                let location = span(&primary);
                let mut parts = primary.into_inner().peekable();
                let mut builder = Builder {
                    errors: &mut Vec::new(),
                };
                let mut stmts = Vec::new();
                while let Some(stmt) = parts.next_if(|part| part.as_rule() == Rule::stmt) {
                    stmts.push(builder.stmt(stmt));
                }
                let tail = parts.next().expect("value block tail");
                let value = Box::new(record_value(tail));
                Expr::Block {
                    body: Block {
                        stmts,
                        span: location,
                    },
                    value,
                    span: location,
                }
            }
            Rule::match_expr => {
                let location = span(&primary);
                let mut parts = children(primary);
                let value = Box::new(expr(parts.next().expect("matched value")));
                let arms = parts
                    .map(|arm| {
                        let location = span(&arm);
                        let mut parts = arm.into_inner().peekable();
                        let pattern = match_pattern(parts.next().expect("pattern"));
                        let guard = parts
                            .next_if(|part| part.as_rule() == Rule::match_guard)
                            .map(|guard| {
                                Builder {
                                    errors: &mut Vec::new(),
                                }
                                .cond(children(guard).next().expect("guard condition"))
                            });
                        let value = record_value(parts.next().expect("arm value"));
                        MatchValueArm {
                            pattern,
                            guard,
                            value,
                            span: location,
                        }
                    })
                    .collect();
                Expr::Match {
                    value,
                    arms,
                    span: location,
                }
            }
            Rule::array_expr => {
                let location = span(&primary);
                let mut parts = primary.into_inner().peekable();
                if parts
                    .peek()
                    .is_some_and(|p| p.as_rule() == Rule::array_repeat)
                {
                    let mut repeat = parts.next().expect("array repeat").into_inner();
                    let value = expr(repeat.next().expect("repeated value"));
                    let length = array_length(repeat.next().expect("repeat length"));
                    return Expr::ArrayRepeat {
                        value: Box::new(value),
                        length: Box::new(length),
                        span: location,
                    };
                }
                let values = parts
                    .map(|value| {
                        let inner = value.into_inner().next().expect("array value");
                        if inner.as_rule() == Rule::cond {
                            let location = span(&inner);
                            Expr::Cond(
                                Box::new(
                                    Builder {
                                        errors: &mut Vec::new(),
                                    }
                                    .cond(inner),
                                ),
                                location,
                            )
                        } else {
                            expr(inner)
                        }
                    })
                    .collect();
                Expr::Array(values, location)
            }
            Rule::record_literal => {
                let s = span(&primary);
                let mut parts = primary.into_inner();
                let path = path(parts.next().expect("record type"));
                let fields = parts
                    .map(|field| {
                        let mut parts = field.into_inner();
                        let name = ident(parts.next().expect("field name"));
                        let value = parts
                            .next()
                            .map(|value| {
                                let inner = value.into_inner().next().expect("field value");
                                if inner.as_rule() == Rule::cond {
                                    let s = span(&inner);
                                    Expr::Cond(
                                        Box::new(
                                            Builder {
                                                errors: &mut Vec::new(),
                                            }
                                            .cond(inner),
                                        ),
                                        s,
                                    )
                                } else {
                                    expr(inner)
                                }
                            })
                            .unwrap_or_else(|| {
                                Expr::Path(Path {
                                    applications: Vec::new(),
                                    segments: vec![name.clone()],
                                    span: name.span,
                                })
                            });
                        (name, value)
                    })
                    .collect();
                Expr::Struct {
                    path,
                    fields,
                    span: s,
                }
            }
            Rule::group => {
                let s = span(&primary);
                let inner = primary.into_inner().next().expect("group value");
                if inner.as_rule() == Rule::expr {
                    expr(inner)
                } else {
                    let cond = Builder {
                        errors: &mut Vec::new(),
                    }
                    .cond(inner);
                    Expr::Cond(Box::new(cond), s)
                }
            }
            Rule::int => {
                let s = span(&primary);
                let digits: String = primary.as_str().chars().filter(|c| *c != '_').collect();
                // Overflowing literals saturate; type checking rejects them.
                Expr::Int(digits.parse().unwrap_or(i64::MAX), s)
            }
            Rule::float => {
                let s = span(&primary);
                Expr::Float(primary.as_str().parse().unwrap_or(0.0), s)
            }
            Rule::bool_lit => {
                let s = span(&primary);
                Expr::Bool(primary.as_str() == "true", s)
            }
            Rule::string => {
                let s = span(&primary);
                let raw = primary
                    .clone()
                    .into_inner()
                    .next()
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_default();
                Expr::Str {
                    value: string(primary),
                    raw,
                    span: s,
                }
            }
            Rule::call_expr => {
                let s = span(&primary);
                let mut parts = primary.into_inner();
                let path = path(parts.next().expect("callee"));
                let args = parts.next().map(args).unwrap_or_default();
                Expr::Call {
                    path,
                    args,
                    span: s,
                }
            }
            Rule::place => Expr::Place(place(primary)),
            Rule::path => Expr::Path(path(primary)),
            Rule::expr => expr(primary),
            other => unreachable!("unexpected primary {other:?}"),
        })
        .map_prefix(|op, rhs| {
            let s = span(&op).to(rhs.span());
            Expr::Neg(Box::new(rhs), s)
        })
        .map_postfix(|lhs, op| {
            let s = lhs.span().to(span(&op));
            if op.as_rule() == Rule::method_postfix {
                let mut parts = op.into_inner();
                let name = ident(parts.next().expect("method name"));
                let mut type_arguments = Vec::new();
                let mut arguments = Vec::new();
                for part in parts {
                    if part.as_rule() == Rule::turbofish {
                        type_arguments = part
                            .into_inner()
                            .next()
                            .expect("type arguments")
                            .into_inner()
                            .map(value_type)
                            .collect();
                    } else {
                        arguments = args(part);
                    }
                }
                return Expr::Method {
                    receiver: Box::new(lhs),
                    name,
                    type_arguments,
                    args: arguments,
                    span: s,
                };
            }
            if op.as_rule() == Rule::index_postfix {
                let index = expr(op.into_inner().next().expect("array index"));
                return Expr::Index(Box::new(lhs), Box::new(index), s);
            }
            if op.as_rule() == Rule::field_postfix {
                let field = ident(op.into_inner().next().expect("field name"));
                return Expr::Project(Box::new(lhs), field, s);
            }
            let ty = op
                .into_inner()
                .find(|p| p.as_rule() == Rule::ty)
                .map(value_type)
                .expect("cast type");
            Expr::Cast(Box::new(lhs), ty, s)
        })
        .map_infix(|lhs, op, rhs| {
            let s = lhs.span().to(rhs.span());
            let op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mul => BinOp::Mul,
                Rule::div => BinOp::Div,
                _ => BinOp::Rem,
            };
            Expr::Binary(Box::new(lhs), op, Box::new(rhs), s)
        })
        .parse(pair.into_inner())
}

fn args(pair: P<'_>) -> Vec<Arg> {
    pair.into_inner()
        .map(|arg| {
            let s = span(&arg);
            let mut parts = arg.into_inner();
            let first = parts.next().expect("argument");
            let (name, value) = if first.as_rule() == Rule::ident {
                (Some(ident(first)), parts.next().expect("argument value"))
            } else {
                (None, first)
            };
            let value = value.into_inner().next().expect("argument value kind");
            let value = match value.as_rule() {
                Rule::range => ArgValue::Range(range(value)),
                Rule::arg_condition => {
                    let s = span(&value);
                    let cond = Builder {
                        errors: &mut Vec::new(),
                    }
                    .cond(value.into_inner().next().expect("condition argument"));
                    ArgValue::Expr(Expr::Cond(Box::new(cond), s))
                }
                _ => ArgValue::Expr(expr(value)),
            };
            Arg {
                name,
                value,
                span: s,
            }
        })
        .collect()
}

fn range(pair: P<'_>) -> Range {
    let s = span(&pair);
    let mut start = None;
    let mut end = None;
    let mut inclusive = false;
    let mut seen_op = false;
    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::range_op => {
                seen_op = true;
                inclusive = part.as_str() == "..=";
            }
            _ if seen_op => end = Some(expr(part)),
            _ => start = Some(expr(part)),
        }
    }
    Range {
        start,
        end,
        inclusive,
        span: s,
    }
}

fn array_length(pair: P<'_>) -> Expr {
    if pair.as_rule() == Rule::int {
        Expr::Int(
            pair.as_str().replace('_', "").parse().unwrap_or(i64::MAX),
            span(&pair),
        )
    } else {
        Expr::Path(path(pair))
    }
}

fn value_type(pair: P<'_>) -> Type {
    let location = span(&pair);
    let inner = pair.into_inner().next().expect("type form");
    if inner.as_rule() == Rule::function_ty {
        let mut parameters = Vec::new();
        let mut returns = None;
        for part in children(inner) {
            if part.as_rule() == Rule::ret_ty {
                returns = part.into_inner().next().map(value_type).map(Box::new);
            } else {
                parameters.push(value_type(part));
            }
        }
        Type {
            kind: TypeKind::Function {
                parameters,
                returns,
            },
            span: location,
        }
    } else if inner.as_rule() == Rule::array_ty {
        let mut parts = inner.into_inner();
        let element = value_type(parts.next().expect("array element type"));
        let length = parts.next().expect("array length");
        let length = array_length(length);
        Type {
            kind: TypeKind::Array {
                element: Box::new(element),
                length: Box::new(length),
            },
            span: location,
        }
    } else if inner.as_rule() == Rule::applied_ty {
        let mut parts = inner.into_inner();
        let path = path(parts.next().expect("applied type"));
        Type {
            kind: TypeKind::Applied {
                path,
                arguments: parts.map(value_type).collect(),
            },
            span: location,
        }
    } else {
        Type::named(path(inner))
    }
}

fn path(pair: P<'_>) -> Path {
    let location = span(&pair);
    let mut segments = Vec::new();
    let mut applications = Vec::new();
    for part in pair.into_inner() {
        if part.as_rule() == Rule::turbofish {
            applications.push(PathApplication {
                segment: segments.len() - 1,
                arguments: part
                    .into_inner()
                    .next()
                    .expect("type arguments")
                    .into_inner()
                    .map(value_type)
                    .collect(),
            });
        } else {
            segments.push(ident(part));
        }
    }
    Path {
        segments,
        applications,
        span: location,
    }
}

fn place(pair: P<'_>) -> Place {
    let s = span(&pair);
    let mut parts = pair.into_inner();
    let state = path(parts.next().expect("place state"));
    let field = ident(parts.next().expect("place field"));
    Place {
        state,
        field,
        projections: parts.map(ident).collect(),
        span: s,
    }
}

fn use_tree(pair: P<'_>) -> UseTree {
    let mut parts = children(pair);
    let prefix = path(parts.next().expect("use path"));
    let mut tree = UseTree {
        path: prefix,
        group: None,
        alias: None,
    };
    if let Some(next) = parts.next() {
        match next.as_rule() {
            Rule::use_group => tree.group = Some(next.into_inner().map(use_tree).collect()),
            _ => tree.alias = Some(ident(next)),
        }
    }
    tree
}

/// Re-parse a recovered chunk strictly to explain why it was rejected.
fn strict_error(rule: Rule, text: &str, offset: usize) -> Option<SyntaxError> {
    match SandParser::parse(rule, text) {
        Ok(_) => None,
        Err(error) => Some(convert_error(&error, text, offset)),
    }
}

fn friendly_rule(rule: &Rule) -> Option<&'static str> {
    Some(match rule {
        Rule::ident => "identifier",
        Rule::string => "string literal",
        Rule::expr | Rule::neg | Rule::int | Rule::float | Rule::primary => "expression",
        Rule::args | Rule::arg | Rule::arg_value => "argument",
        Rule::data_object | Rule::data_field | Rule::data_value | Rule::data_array => "field",
        Rule::path => "path",
        Rule::place => "`State.field`",
        Rule::cond | Rule::comparison | Rule::in_range => "condition",
        Rule::block => "`{`",
        Rule::item | Rule::item_kind => "item",
        Rule::stmt => "statement",
        Rule::ty => "type",
        Rule::vis => "`pub`",
        Rule::cmp_op => "comparison operator",
        Rule::assign_op => "`=`, `+=`, or `-=`",
        Rule::field => "field",
        Rule::variant => "variant",
        Rule::attr => "attribute",
        Rule::outer_doc => "doc comment",
        Rule::hook_phase => "`enter`, `exit`, or `tick`",
        Rule::scope => "`player` or `global`",
        Rule::EOI => "end of input",
        _ => return None,
    })
}

fn convert_error(error: &pest::error::Error<Rule>, text: &str, offset: usize) -> SyntaxError {
    use pest::error::{ErrorVariant, InputLocation};
    let pos = match error.location {
        InputLocation::Pos(pos) => pos,
        InputLocation::Span((start, _)) => start,
    };
    let mut expected: Vec<String> = Vec::new();
    let mut at = pos;
    if let Some(attempts) = error.parse_attempts() {
        at = attempts.max_position.max(pos);
        for stack in attempts.call_stacks() {
            let rule = stack.deepest.get_rule().copied().or(stack.parent);
            if let Some(name) = rule.and_then(|rule| friendly_rule(&rule)) {
                expected.push(name.to_string());
            }
        }
        for token in attempts.expected_tokens() {
            // Character ranges render as `a..z`; built-in rules carry no text.
            let token = token.to_string();
            let token = token.trim();
            let is_char_range =
                token.chars().count() == 4 && token.chars().skip(1).take(2).all(|c| c == '.');
            let noise = token.is_empty()
                || is_char_range
                || matches!(
                    token,
                    "BUILTIN_RULE" | "/*" | "//" | "///" | "//!" | "_" | "crate" | "self" | "super"
                );
            // Tokens that merely start an expression are summarised by it.
            let starts_expression = matches!(token, "(" | "-" | "\"" | "true" | "false")
                && expected.iter().any(|e| e == "expression");
            if !noise && !starts_expression {
                expected.push(format!("`{token}`"));
            }
        }
    } else if let ErrorVariant::ParsingError { positives, .. } = &error.variant {
        for rule in positives {
            if let Some(name) = friendly_rule(rule) {
                expected.push(name.to_string());
            }
        }
    }
    if expected.iter().any(|e| e == "expression") {
        expected.retain(|e| {
            !matches!(
                e.as_str(),
                "path" | "identifier" | "integer" | "`State.field`"
            )
        });
    }
    expected.sort();
    expected.dedup();
    expected.truncate(8);
    let at = at.min(text.len());
    let found = text[at..]
        .split(|c: char| c.is_whitespace())
        .find(|token| !token.is_empty())
        .map(|token| {
            let short: String = token.chars().take(16).collect();
            format!("found `{short}`")
        })
        .unwrap_or_else(|| "found end of input".to_string());
    let message = match expected.len() {
        0 => format!("unexpected syntax; {found}"),
        1 => format!("expected {}; {found}", expected[0]),
        _ => {
            let (last, rest) = expected.split_last().expect("non-empty");
            format!("expected {} or {last}; {found}", rest.join(", "))
        }
    };
    let token_len = text[at..]
        .chars()
        .take_while(|c| !c.is_whitespace())
        .map(char::len_utf8)
        .sum::<usize>()
        .max(1);
    let end = (at + token_len).min(text.len()).max(at);
    SyntaxError {
        message,
        span: Span::new(offset + at, offset + end),
    }
}
