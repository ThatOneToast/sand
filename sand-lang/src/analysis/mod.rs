//! Name resolution, type checking, execution-context inference, and lowering.
//!
//! Analysis turns a loaded [`Package`] into Sand's portable
//! [`Program`](sand::advanced::compiler::Program). The portable compiler then
//! validates and emits the datapack, so the language shares one backend with
//! Rust-authored packs. Analysis also records every definition and resolved
//! reference, which powers hover, go-to-definition, references, and rename.

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::package::{ModuleId, Package};
use crate::source::FileId;
use sand::advanced::compiler as pc;
use std::collections::{BTreeMap, HashMap, HashSet};

mod resources;
mod statements;

/// Index of a definition within an [`Analysis`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(pub usize);

/// The value type of a constant, field, or expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    /// A whole number, stored directly in a score.
    Number,
    /// A decimal stored as fixed point: the score holds `value * 1000`.
    Float,
    /// `true` or `false`, stored as one or zero.
    Bool,
    /// An enum whose variants are integer phases.
    Enum(DefId),
    /// A type that failed to resolve; suppresses follow-up errors.
    Unknown,
}

/// Where a function runs: on the server, or as a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// No player executor; callable from anywhere.
    Server,
    /// Requires a player executor.
    Player,
    /// Requires an entity executor (any entity, including players).
    Entity,
}

impl Context {
    /// Source-facing name.
    pub fn name(self) -> &'static str {
        match self {
            Context::Server => "server",
            Context::Player => "player",
            Context::Entity => "entity",
        }
    }
}

/// What a definition is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    /// `mod name`.
    Module,
    /// `const NAME`.
    Const,
    /// `enum Name`.
    Enum,
    /// An enum variant.
    Variant,
    /// `player state` or `global state`.
    State,
    /// A State field.
    Field,
    /// `fn name`.
    Function,
    /// `extern "id" fn name`.
    ExternFunction,
    /// `flow Name`.
    Flow,
    /// A `let` binding or function parameter.
    Local,
    /// `item Name { ... }`.
    CustomItem,
    /// `recipe name { ... }`.
    Recipe,
    /// `tag Name: Type { ... }`.
    Tag,
}

impl DefKind {
    /// Source-facing description.
    pub fn describe(self) -> &'static str {
        match self {
            DefKind::Module => "module",
            DefKind::Const => "constant",
            DefKind::Enum => "enum",
            DefKind::Variant => "variant",
            DefKind::State => "state",
            DefKind::Field => "field",
            DefKind::Function => "function",
            DefKind::ExternFunction => "external function",
            DefKind::Flow => "flow",
            DefKind::Local => "local variable",
            DefKind::CustomItem => "custom item",
            DefKind::Recipe => "recipe",
            DefKind::Tag => "tag",
        }
    }
}

/// Kind-specific facts about a definition.
#[derive(Debug, Clone)]
pub enum DefInfo {
    /// A module; `None` when its file could not be loaded.
    Module {
        /// Loaded module.
        target: Option<ModuleId>,
    },
    /// A constant.
    Const {
        /// Declared type.
        ty: Ty,
        /// Evaluated value.
        value: Option<i64>,
    },
    /// An enum.
    Enum {
        /// Variants in order.
        variants: Vec<DefId>,
    },
    /// An enum variant.
    Variant {
        /// Evaluated discriminant.
        value: Option<i64>,
    },
    /// A State.
    State {
        /// Ownership scope.
        scope: Scope,
        /// Minecraft State identity.
        id: String,
        /// Fields in order.
        fields: Vec<DefId>,
    },
    /// A State field.
    Field {
        /// Value type.
        ty: Ty,
        /// Initial value.
        default: Option<i64>,
        /// Inclusive clamp bounds.
        bounds: Option<(i32, i32)>,
    },
    /// A function.
    Function {
        /// Minecraft function identity.
        id: String,
        /// Inferred or declared execution context.
        context: Context,
        /// Why the function needs a player, when it does.
        player_reason: Option<String>,
        /// Function tags the function joins, including `#[tick]`/`#[load]`.
        entry: Vec<String>,
        /// Parameters and return type as written, such as `(cost: i32) -> i32`.
        signature: String,
    },
    /// An external function.
    ExternFunction {
        /// Minecraft function identity.
        id: String,
        /// Declared context.
        context: Context,
    },
    /// A flow.
    Flow {
        /// Minecraft flow identity.
        id: String,
    },
    /// A data resource declaration.
    Data {
        /// Declaration kind.
        kind: crate::ast::DataKind,
        /// Minecraft resource identity.
        id: String,
        /// The grouped type of a `tag`: `Item`, `Block`, or `Entity`.
        tag_type: Option<&'static str>,
    },
    /// A local variable or parameter.
    Local {
        /// Value type.
        ty: Ty,
        /// Whether it may be reassigned.
        mutable: bool,
        /// Whether it is a function parameter.
        param: bool,
        /// The function that owns it.
        owner: Option<DefId>,
    },
}

/// A named declaration.
#[derive(Debug, Clone)]
pub struct Def {
    /// Kind.
    pub kind: DefKind,
    /// Declared name.
    pub name: String,
    /// Module the definition belongs to.
    pub module: ModuleId,
    /// File holding the declaration.
    pub file: FileId,
    /// Location of the declared name.
    pub name_span: Span,
    /// Location of the whole declaration.
    pub span: Span,
    /// Documentation lines.
    pub docs: Vec<String>,
    /// Whether the definition is visible outside its module.
    pub is_pub: bool,
    /// Enclosing enum or State for variants and fields.
    pub parent: Option<DefId>,
    /// Kind-specific facts.
    pub info: DefInfo,
}

/// A resolved use of a definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference {
    /// File of the use.
    pub file: FileId,
    /// Location of the referring identifier.
    pub span: Span,
    /// Referenced definition.
    pub def: DefId,
}

/// Pack metadata from `#![pack(...)]`.
#[derive(Debug, Clone)]
pub struct PackInfo {
    /// Namespace for every generated identity.
    pub namespace: String,
    /// Exact Minecraft target.
    pub minecraft: String,
    /// `pack.mcmeta` description.
    pub description: String,
}

/// The complete result of analysing a package.
#[derive(Debug, Clone)]
pub struct Analysis {
    /// The analysed package.
    pub package: Package,
    /// Every definition.
    pub defs: Vec<Def>,
    /// Every resolved reference.
    pub references: Vec<Reference>,
    /// Syntax, resolution, and type diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Pack metadata, when declared.
    pub pack: Option<PackInfo>,
    /// The lowered portable program; present only without errors.
    pub program: Option<pc::Program>,
    /// Uses of builtin statements and macros, for hover and navigation.
    pub builtin_uses: Vec<(FileId, Span, &'static str, crate::builtins::BuiltinKind)>,
    /// Where each local variable is visible, for scope-aware completion.
    pub local_scopes: Vec<(FileId, Span, DefId)>,
    /// Typed vanilla values such as `Item::Diamond`, for hover.
    pub vanilla_uses: Vec<VanillaUse>,
    /// Program pointers mapped back to source, for backend diagnostics.
    pointers: Vec<(String, String, FileId, Span)>,
}

impl Analysis {
    /// Whether any diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == crate::diagnostic::Severity::Error)
    }

    /// The definition a source location names, if any.
    pub fn def_at(&self, file: FileId, offset: usize) -> Option<DefId> {
        self.references
            .iter()
            .filter(|r| r.file == file && r.span.contains(offset))
            .map(|r| (r.span, r.def))
            .chain(
                self.defs
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| d.file == file && d.name_span.contains(offset))
                    .map(|(i, d)| (d.name_span, DefId(i))),
            )
            .min_by_key(|(span, _)| span.end - span.start)
            .map(|(_, def)| def)
    }

    /// Definition by id.
    pub fn def(&self, id: DefId) -> &Def {
        &self.defs[id.0]
    }

    /// Compile the lowered program through Sand's portable compiler.
    ///
    /// Backend diagnostics are mapped back to the Sand source that produced
    /// the offending operation or declaration.
    pub fn compile(&self) -> Result<pc::CompiledPack, Vec<Diagnostic>> {
        let Some(program) = &self.program else {
            return Err(self.diagnostics.clone());
        };
        pc::Compiler::compile(program)
            .map_err(|errors| errors.iter().map(|e| self.map_backend(e)).collect())
    }

    fn map_backend(&self, error: &pc::Diagnostic) -> Diagnostic {
        let located = error
            .origin
            .as_ref()
            .and_then(|origin| {
                let file = origin.get("file")?.as_u64()? as usize;
                let start = origin.get("start")?.as_u64()? as usize;
                let end = origin.get("end")?.as_u64()? as usize;
                (file < self.package.files.len()).then_some((FileId(file), Span::new(start, end)))
            })
            .or_else(|| {
                let mut pointer = error.pointer.as_str();
                loop {
                    if let Some((_, _, file, span)) = self
                        .pointers
                        .iter()
                        .find(|(module, p, _, _)| *module == error.module && p == pointer)
                    {
                        return Some((*file, *span));
                    }
                    let (head, _) = pointer.rsplit_once('/')?;
                    pointer = head;
                }
            })
            .unwrap_or((FileId(0), Span::default()));
        Diagnostic::error(
            &format!("backend/{}", error.code),
            located.0,
            located.1,
            error.message.clone(),
        )
    }
}

/// Convert an identifier to the snake_case used in Minecraft identities.
/// `ruby_sword` or `RUBY_SWORD` -> `RubySword`.
pub fn pascal_case(name: &str) -> String {
    snake_case(name)
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// Lowercase words joined by underscores: `heartPrice` -> `heart_price`.
pub fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev = i.checked_sub(1).map(|p| chars[p]);
            let next = chars.get(i + 1);
            let boundary = prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit())
                || (prev.is_some_and(char::is_uppercase) && next.is_some_and(|n| n.is_lowercase()));
            if boundary && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Analyse a loaded package.
pub fn analyze(package: Package) -> Analysis {
    let mut cx = Cx {
        diagnostics: package.diagnostics.clone(),
        package,
        defs: Vec::new(),
        references: Vec::new(),
        scopes: Vec::new(),
        members: BTreeMap::new(),
        const_src: BTreeMap::new(),
        variant_src: BTreeMap::new(),
        memo: BTreeMap::new(),
        pack: None,
        pointers: Vec::new(),
        builtin_uses: Vec::new(),
        local_scopes: Vec::new(),
        vanilla_uses: Vec::new(),
        local_fields: Vec::new(),
        fn_slots: BTreeMap::new(),
    };
    cx.collect();
    cx.resolve_imports();
    cx.pack_info();
    cx.evaluate_declarations();
    let program = cx.lower();
    cx.naming_lints();
    cx.diagnostics
        .sort_by_key(|d| (d.file, d.span.start, d.span.end));
    let has_errors = cx
        .diagnostics
        .iter()
        .any(|d| d.severity == crate::diagnostic::Severity::Error);
    Analysis {
        package: cx.package,
        defs: cx.defs,
        references: cx.references,
        diagnostics: cx.diagnostics,
        pack: cx.pack,
        program: if has_errors { None } else { program },
        pointers: cx.pointers,
        builtin_uses: cx.builtin_uses,
        local_scopes: cx.local_scopes,
        vanilla_uses: cx.vanilla_uses,
    }
}

#[derive(Debug, Clone, Copy)]
struct Binding {
    def: DefId,
    is_pub: bool,
}

struct ConstSrc {
    module: ModuleId,
    file: FileId,
    expr: Expr,
    ty: Ty,
}

struct VariantSrc {
    module: ModuleId,
    file: FileId,
    expr: Option<Expr>,
    prev: Option<DefId>,
}

#[derive(Debug, Clone, Copy)]
enum Memo {
    Evaluating,
    Done(Option<i64>),
}

enum Cur {
    Module(ModuleId),
    Def(DefId),
}

struct Cx {
    package: Package,
    defs: Vec<Def>,
    references: Vec<Reference>,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<BTreeMap<String, Binding>>,
    members: BTreeMap<DefId, BTreeMap<String, DefId>>,
    const_src: BTreeMap<DefId, ConstSrc>,
    variant_src: BTreeMap<DefId, VariantSrc>,
    memo: BTreeMap<DefId, Memo>,
    pack: Option<PackInfo>,
    pointers: Vec<(String, String, FileId, Span)>,
    builtin_uses: Vec<(FileId, Span, &'static str, crate::builtins::BuiltinKind)>,
    local_scopes: Vec<(FileId, Span, DefId)>,
    vanilla_uses: Vec<VanillaUse>,
    /// Fields of the compiler-owned locals State, in allocation order.
    local_fields: Vec<String>,
    /// Parameter and return slots per function.
    fn_slots: BTreeMap<DefId, FnSlots>,
}

/// How many score units represent `1.0` in a `float`, matching Rust's
/// default fixed-point scale.
pub(crate) const FLOAT_SCALE: i64 = 1_000;

/// Whether an expression is a whole-number literal, optionally negated.
pub(crate) fn is_int_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Int(..) => true,
        Expr::Neg(inner, _) => is_int_literal(inner),
        _ => false,
    }
}

/// One typed vanilla value written in source, such as `Item::Diamond`.
#[derive(Debug, Clone)]
pub struct VanillaUse {
    /// File containing the value.
    pub file: FileId,
    /// The whole `Registry::Variant` path.
    pub span: Span,
    /// Registry name, such as `Item`.
    pub registry: &'static str,
    /// Variant name, such as `Diamond`.
    pub variant: String,
    /// Namespaced identity, such as `minecraft:diamond`.
    pub id: String,
}

/// Where a function's parameters and return value live.
#[derive(Debug, Clone)]
pub(crate) struct FnSlots {
    params: Vec<(String, DefId, Ty)>,
    ret: Option<(String, Ty)>,
}

type Resolved = Result<(DefId, Vec<Reference>), Box<Diagnostic>>;

impl Cx {
    fn error(&mut self, code: &str, file: FileId, span: Span, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(code, file, span, message));
    }

    fn push_def(&mut self, def: Def) -> DefId {
        let id = DefId(self.defs.len());
        self.defs.push(def);
        id
    }

    fn namespace(&self) -> String {
        self.pack
            .as_ref()
            .map(|p| p.namespace.clone())
            .unwrap_or_else(|| "sand".into())
    }

    fn item_id(&self, module: ModuleId, name: &str) -> String {
        let module = self.package.module(module);
        let mut path: Vec<String> = module.path.clone();
        path.push(snake_case(name));
        format!("{}:{}", self.namespace(), path.join("/"))
    }

    /// Warn about names that break the language's naming convention:
    /// PascalCase for declared things and typed sets, snake_case for
    /// functions, fields, variables, and modules, SCREAMING_SNAKE_CASE for
    /// constants. Mirrors Rust's naming lints.
    fn naming_lints(&mut self) {
        for index in 0..self.defs.len() {
            let def = &self.defs[index];
            let name = def.name.trim_start_matches('_');
            if name.is_empty() || def.name_span == Span::default() {
                continue;
            }
            let (style, fixed) = match def.kind {
                DefKind::Function
                | DefKind::ExternFunction
                | DefKind::Field
                | DefKind::Local
                | DefKind::Module => ("snake_case", snake_case(name).to_ascii_lowercase()),
                DefKind::Const => (
                    "SCREAMING_SNAKE_CASE",
                    snake_case(name).to_ascii_uppercase(),
                ),
                DefKind::State
                | DefKind::Enum
                | DefKind::Variant
                | DefKind::Flow
                | DefKind::CustomItem
                | DefKind::Recipe
                | DefKind::Tag => ("PascalCase", pascal_case(name)),
            };
            if fixed == name || fixed.is_empty() {
                continue;
            }
            let what = def.kind.describe();
            let (file, span) = (def.file, def.name_span);
            let prefix = if def.name.starts_with('_') { "_" } else { "" };
            self.diagnostics.push(
                Diagnostic::warning(
                    "naming",
                    file,
                    span,
                    format!("{what} `{}` should have a {style} name", def.name),
                )
                .with_help(format!("rename it to `{prefix}{fixed}`")),
            );
        }
    }

    // ── Collection ──────────────────────────────────────────────────────────

    fn collect(&mut self) {
        self.scopes = vec![BTreeMap::new(); self.package.modules.len()];
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            let items = self.package.module(module).items.clone();
            for item in &items {
                self.collect_item(module, file, item);
            }
        }
    }

    fn bind(&mut self, module: ModuleId, name: &Ident, file: FileId, binding: Binding) {
        if let Some(existing) = self.scopes[module.0].get(&name.name).copied() {
            let prior = &self.defs[existing.def.0];
            let diagnostic = Diagnostic::error(
                "duplicate",
                file,
                name.span,
                format!("the name `{}` is defined multiple times", name.name),
            )
            .with_note(prior.file, prior.name_span, "previous definition here");
            self.diagnostics.push(diagnostic);
            return;
        }
        self.scopes[module.0].insert(name.name.clone(), binding);
    }

    fn collect_item(&mut self, module: ModuleId, file: FileId, item: &Item) {
        let Some(name) = item.name().cloned() else {
            return;
        };
        let base = |kind, info| Def {
            kind,
            name: name.name.clone(),
            module,
            file,
            name_span: name.span,
            span: item.span,
            docs: item.docs.clone(),
            is_pub: item.is_pub,
            parent: None,
            info,
        };
        let def = match &item.kind {
            ItemKind::Mod(_) => {
                let target = self
                    .package
                    .module(module)
                    .children
                    .get(&name.name)
                    .copied();
                let docs = target
                    .filter(|t| self.package.module(*t).file != file)
                    .map(|t| {
                        self.package.files[self.package.module(t).file.0]
                            .ast
                            .docs
                            .clone()
                    })
                    .unwrap_or_default();
                let mut def = base(DefKind::Module, DefInfo::Module { target });
                def.docs.extend(docs);
                self.push_def(def)
            }
            ItemKind::Const(c) => {
                let id = self.push_def(base(
                    DefKind::Const,
                    DefInfo::Const {
                        ty: Ty::Unknown,
                        value: None,
                    },
                ));
                self.const_src.insert(
                    id,
                    ConstSrc {
                        module,
                        file,
                        expr: c.value.clone(),
                        ty: Ty::Unknown,
                    },
                );
                id
            }
            ItemKind::Enum(e) => {
                let id = self.push_def(base(
                    DefKind::Enum,
                    DefInfo::Enum {
                        variants: Vec::new(),
                    },
                ));
                let mut variants = Vec::new();
                let mut members = BTreeMap::new();
                let mut prev = None;
                for variant in &e.variants {
                    let vid = self.push_def(Def {
                        kind: DefKind::Variant,
                        name: variant.name.name.clone(),
                        module,
                        file,
                        name_span: variant.name.span,
                        span: variant.name.span,
                        docs: variant.docs.clone(),
                        is_pub: item.is_pub,
                        parent: Some(id),
                        info: DefInfo::Variant { value: None },
                    });
                    if members.insert(variant.name.name.clone(), vid).is_some() {
                        self.error(
                            "duplicate",
                            file,
                            variant.name.span,
                            format!("variant `{}` is declared more than once", variant.name.name),
                        );
                    }
                    self.variant_src.insert(
                        vid,
                        VariantSrc {
                            module,
                            file,
                            expr: variant.value.clone(),
                            prev,
                        },
                    );
                    prev = Some(vid);
                    variants.push(vid);
                }
                if variants.is_empty() {
                    self.error(
                        "enum",
                        file,
                        name.span,
                        "an enum needs at least one variant to be stored in State",
                    );
                }
                self.defs[id.0].info = DefInfo::Enum { variants };
                self.members.insert(id, members);
                id
            }
            ItemKind::State(s) => {
                let state_id = self.item_id(module, &name.name);
                let id = self.push_def(base(
                    DefKind::State,
                    DefInfo::State {
                        scope: s.scope,
                        id: state_id,
                        fields: Vec::new(),
                    },
                ));
                let mut fields = Vec::new();
                let mut members = BTreeMap::new();
                for field in &s.fields {
                    let fid = self.push_def(Def {
                        kind: DefKind::Field,
                        name: field.name.name.clone(),
                        module,
                        file,
                        name_span: field.name.span,
                        span: field.span,
                        docs: field.docs.clone(),
                        is_pub: item.is_pub,
                        parent: Some(id),
                        info: DefInfo::Field {
                            ty: Ty::Unknown,
                            default: None,
                            bounds: None,
                        },
                    });
                    if members.insert(field.name.name.clone(), fid).is_some() {
                        self.error(
                            "duplicate",
                            file,
                            field.name.span,
                            format!("field `{}` is declared more than once", field.name.name),
                        );
                    }
                    fields.push(fid);
                }
                if let DefInfo::State { fields: f, .. } = &mut self.defs[id.0].info {
                    *f = fields;
                }
                self.members.insert(id, members);
                id
            }
            ItemKind::Flow(_) => {
                let flow_id = self.item_id(module, &name.name);
                self.push_def(base(DefKind::Flow, DefInfo::Flow { id: flow_id }))
            }
            ItemKind::Fn(_) => {
                let fn_id = self.item_id(module, &name.name);
                self.push_def(base(
                    DefKind::Function,
                    DefInfo::Function {
                        id: fn_id,
                        context: Context::Server,
                        player_reason: None,
                        entry: Vec::new(),
                        signature: "()".into(),
                    },
                ))
            }
            ItemKind::ExternFn(e) => {
                let context = if item.attr("player").is_some() {
                    Context::Player
                } else {
                    Context::Server
                };
                self.push_def(base(
                    DefKind::ExternFunction,
                    DefInfo::ExternFunction {
                        id: e.id.clone(),
                        context,
                    },
                ))
            }
            ItemKind::Data(data) => {
                let kind = match data.kind {
                    crate::ast::DataKind::Item => DefKind::CustomItem,
                    crate::ast::DataKind::Recipe => DefKind::Recipe,
                    crate::ast::DataKind::Tag => DefKind::Tag,
                };
                let tag_type = match (&data.kind, &data.tag_type) {
                    (crate::ast::DataKind::Tag, Some(ty)) => match ty.name.as_str() {
                        "Item" => Some("Item"),
                        "Block" => Some("Block"),
                        "Entity" => Some("Entity"),
                        other => {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "tag",
                                    file,
                                    ty.span,
                                    format!("cannot make a tag of `{other}`"),
                                )
                                .with_help("tags group `Item`, `Block`, or `Entity` values"),
                            );
                            None
                        }
                    },
                    (crate::ast::DataKind::Tag, None) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "tag",
                                file,
                                name.span,
                                "a tag needs the type it groups",
                            )
                            .with_help(format!(
                                "write `tag {}: Item {{ ... }}` (or `Block`, `Entity`)",
                                name.name
                            )),
                        );
                        None
                    }
                    (_, Some(ty)) => {
                        self.error(
                            "syntax",
                            file,
                            ty.span,
                            format!("only tags declare a type; remove `: {}`", ty.name),
                        );
                        None
                    }
                    (_, None) => None,
                };
                let id = self.item_id(module, &name.name);
                self.push_def(base(
                    kind,
                    DefInfo::Data {
                        kind: data.kind,
                        id,
                        tag_type,
                    },
                ))
            }
            ItemKind::Use(_) | ItemKind::Error(_) => return,
        };
        self.bind(
            module,
            &name,
            file,
            Binding {
                def,
                is_pub: item.is_pub,
            },
        );
    }

    // ── Imports and path resolution ─────────────────────────────────────────

    fn resolve_imports(&mut self) {
        struct Pending {
            module: ModuleId,
            file: FileId,
            path: Path,
            alias: Ident,
            is_pub: bool,
        }
        let mut pending = Vec::new();
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            for item in self.package.module(module).items.clone() {
                if let ItemKind::Use(u) = &item.kind {
                    let mut flat = Vec::new();
                    flatten_use(&u.tree, &[], &mut flat);
                    for (path, alias) in flat {
                        pending.push(Pending {
                            module,
                            file,
                            path,
                            alias,
                            is_pub: item.is_pub,
                        });
                    }
                }
            }
        }
        // Imports may resolve through other imports; iterate to a fixpoint.
        loop {
            let mut progressed = false;
            let mut remaining = Vec::new();
            for import in pending {
                match self.resolve_path(import.module, &import.path) {
                    Ok((def, refs)) => {
                        self.references.extend(refs);
                        self.bind(
                            import.module,
                            &import.alias,
                            import.file,
                            Binding {
                                def,
                                is_pub: import.is_pub,
                            },
                        );
                        progressed = true;
                    }
                    Err(_) => remaining.push(import),
                }
            }
            pending = remaining;
            if !progressed || pending.is_empty() {
                break;
            }
        }
        for import in pending {
            if let Err(diagnostic) = self.resolve_path(import.module, &import.path) {
                let mut diagnostic = *diagnostic;
                diagnostic.message = format!("unresolved import: {}", diagnostic.message);
                self.diagnostics.push(diagnostic);
            }
        }
    }

    fn visible(&self, from: ModuleId, owner: ModuleId, is_pub: bool) -> bool {
        if is_pub {
            return true;
        }
        let mut cursor = Some(from);
        while let Some(module) = cursor {
            if module == owner {
                return true;
            }
            cursor = self.package.module(module).parent;
        }
        false
    }

    fn resolve_path(&self, from: ModuleId, path: &Path) -> Resolved {
        let file = self.package.module(from).file;
        let fail = |span: Span, message: String| {
            Err(Box::new(Diagnostic::error("resolve", file, span, message)))
        };
        let mut refs = Vec::new();
        let mut cur = Cur::Module(from);
        let mut segments = path.segments.iter().peekable();
        let first = segments.peek().expect("non-empty path");
        match first.name.as_str() {
            "crate" => {
                cur = Cur::Module(ModuleId(0));
                segments.next();
            }
            "self" => {
                segments.next();
            }
            "super" => {
                while segments.peek().is_some_and(|s| s.name == "super") {
                    let segment = segments.next().expect("peeked");
                    let Cur::Module(module) = cur else {
                        unreachable!()
                    };
                    match self.package.module(module).parent {
                        Some(parent) => cur = Cur::Module(parent),
                        None => {
                            return fail(
                                segment.span,
                                "there are too many leading `super` keywords".into(),
                            );
                        }
                    }
                }
            }
            _ => {
                let segment = segments.next().expect("peeked");
                let Some(binding) = self.scopes[from.0].get(&segment.name) else {
                    let mut message = format!("cannot find `{}` in this scope", segment.name);
                    if self.package.module(from).parent.is_some()
                        && self.scopes[0].contains_key(&segment.name)
                    {
                        message.push_str(&format!(
                            "; it is defined at the crate root, so import it with `use crate::{};`",
                            segment.name
                        ));
                    }
                    return fail(segment.span, message);
                };
                refs.push(Reference {
                    file,
                    span: segment.span,
                    def: binding.def,
                });
                cur = self.enter(binding.def);
            }
        }
        for segment in segments {
            match cur {
                Cur::Module(module) => {
                    let Some(binding) = self.scopes[module.0].get(&segment.name) else {
                        return fail(
                            segment.span,
                            format!(
                                "cannot find `{}` in `{}`",
                                segment.name,
                                self.package.module(module).display_path()
                            ),
                        );
                    };
                    if !self.visible(from, module, binding.is_pub) {
                        return Err(Box::new(
                            Diagnostic::error(
                                "privacy",
                                file,
                                segment.span,
                                format!("`{}` is private", segment.name),
                            )
                            .with_note(
                                self.defs[binding.def.0].file,
                                self.defs[binding.def.0].name_span,
                                "declared here; mark it `pub` to use it from other modules",
                            ),
                        ));
                    }
                    refs.push(Reference {
                        file,
                        span: segment.span,
                        def: binding.def,
                    });
                    cur = self.enter(binding.def);
                }
                Cur::Def(def) => {
                    let parent = &self.defs[def.0];
                    let member = self
                        .members
                        .get(&def)
                        .and_then(|members| members.get(&segment.name))
                        .copied();
                    match (parent.kind, member) {
                        (DefKind::Enum, Some(variant)) => {
                            refs.push(Reference {
                                file,
                                span: segment.span,
                                def: variant,
                            });
                            cur = Cur::Def(variant);
                        }
                        (DefKind::Enum, None) => {
                            return fail(
                                segment.span,
                                format!(
                                    "no variant named `{}` in enum `{}`",
                                    segment.name, parent.name
                                ),
                            );
                        }
                        (DefKind::State, _) => {
                            return fail(
                                segment.span,
                                format!(
                                    "State fields are accessed with `.`: write `{}.{}`",
                                    parent.name, segment.name
                                ),
                            );
                        }
                        (kind, _) => {
                            return fail(
                                segment.span,
                                format!(
                                    "`{}` is a {} and has no `::` members",
                                    parent.name,
                                    kind.describe()
                                ),
                            );
                        }
                    }
                }
            }
        }
        match cur {
            Cur::Def(def) => Ok((def, refs)),
            Cur::Module(module) => {
                // Paths ending in a module refer to its declaring `mod` item.
                let def = self.defs.iter().position(
                    |d| matches!(d.info, DefInfo::Module { target: Some(t) } if t == module),
                );
                match def {
                    Some(def) => Ok((DefId(def), refs)),
                    None => fail(path.span, "expected an item, found the crate root".into()),
                }
            }
        }
    }

    fn enter(&self, def: DefId) -> Cur {
        match self.defs[def.0].info {
            DefInfo::Module {
                target: Some(module),
            } => Cur::Module(module),
            _ => Cur::Def(def),
        }
    }

    /// Resolve a path expecting particular definition kinds; records references.
    fn expect_path(
        &mut self,
        module: ModuleId,
        path: &Path,
        kinds: &[DefKind],
        what: &str,
    ) -> Option<DefId> {
        match self.resolve_path(module, path) {
            Ok((def, refs)) => {
                self.references.extend(refs);
                let found = &self.defs[def.0];
                if kinds.contains(&found.kind) {
                    Some(def)
                } else {
                    let message = format!(
                        "expected {what}, found {} `{}`",
                        found.kind.describe(),
                        found.name
                    );
                    let note = (found.file, found.name_span);
                    let file = self.package.module(module).file;
                    self.diagnostics.push(
                        Diagnostic::error("kind", file, path.span, message).with_note(
                            note.0,
                            note.1,
                            "defined here",
                        ),
                    );
                    None
                }
            }
            Err(diagnostic) => {
                self.diagnostics.push(*diagnostic);
                None
            }
        }
    }

    // ── Pack metadata ───────────────────────────────────────────────────────

    fn pack_info(&mut self) {
        let mut namespace = None;
        let mut minecraft = None;
        let mut description = None;
        let mut pack_span = None;
        for (index, source) in self.package.files.clone().iter().enumerate() {
            let file = FileId(index);
            for attr in &source.ast.inner_attrs {
                if attr.name() != "pack" {
                    self.error(
                        "attribute",
                        file,
                        attr.path.span,
                        format!(
                            "unknown inner attribute `{}`; only `#![pack(...)]` is supported",
                            attr.name()
                        ),
                    );
                    continue;
                }
                if index != 0 {
                    self.error(
                        "attribute",
                        file,
                        attr.span,
                        "`#![pack(...)]` belongs in the package root file",
                    );
                    continue;
                }
                pack_span = Some(attr.span);
                for arg in &attr.args {
                    let (Some(key), AttrValue::Str(value, _)) = (&arg.key, &arg.value) else {
                        self.error(
                            "attribute",
                            file,
                            arg.span,
                            "expected `key = \"value\"` in `#![pack(...)]`",
                        );
                        continue;
                    };
                    match key.name.as_str() {
                        "namespace" => namespace = Some((value.clone(), arg.span)),
                        "minecraft" => minecraft = Some(value.clone()),
                        "description" => description = Some(value.clone()),
                        other => self.error(
                            "attribute",
                            file,
                            key.span,
                            format!(
                                "unknown pack setting `{other}`; expected `namespace`, `minecraft`, or `description`"
                            ),
                        ),
                    }
                }
            }
        }
        let root = FileId(0);
        let Some((namespace, ns_span)) = namespace else {
            let span = pack_span.unwrap_or_default();
            self.diagnostics.push(
                Diagnostic::error(
                    "pack",
                    root,
                    span,
                    "the package root must declare a namespace",
                )
                .with_help("add `#![pack(namespace = \"my_pack\")]` at the top of the root file"),
            );
            return;
        };
        let valid = !namespace.is_empty()
            && namespace
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-.".contains(&b));
        if !valid {
            self.error(
                "pack",
                root,
                ns_span,
                "namespaces may only contain lowercase letters, digits, `_`, `-`, and `.`",
            );
            return;
        }
        let minecraft = minecraft.unwrap_or_else(latest_target);
        let description = description.unwrap_or_else(|| {
            let docs = self.package.files[0].ast.docs.join(" ");
            if docs.trim().is_empty() {
                format!("{namespace} datapack")
            } else {
                docs.trim().to_string()
            }
        });
        self.pack = Some(PackInfo {
            namespace,
            minecraft,
            description,
        });
        // Identities depend on the namespace; refresh them now that it is known.
        for index in 0..self.defs.len() {
            let def = &self.defs[index];
            let fresh = self.item_id(def.module, &def.name);
            match &mut self.defs[index].info {
                DefInfo::State { id, .. }
                | DefInfo::Flow { id }
                | DefInfo::Function { id, .. }
                | DefInfo::Data { id, .. } => *id = fresh,
                _ => {}
            }
        }
    }

    // ── Constants, enums, and State declarations ────────────────────────────

    fn resolve_type(&mut self, module: ModuleId, ty: &Path) -> Ty {
        if let [segment] = ty.segments.as_slice() {
            match segment.name.as_str() {
                "number" => return Ty::Number,
                "float" => return Ty::Float,
                "bool" => return Ty::Bool,
                "i32" => {
                    let file = self.package.module(module).file;
                    self.diagnostics.push(
                        Diagnostic::error(
                            "type",
                            file,
                            segment.span,
                            "`i32` is written `number` in Sand",
                        )
                        .with_help("use `number` for whole numbers or `float` for decimals"),
                    );
                    return Ty::Unknown;
                }
                _ => {}
            }
        }
        match self.expect_path(
            module,
            ty,
            &[DefKind::Enum],
            "a type (`number`, `float`, `bool`, or an enum)",
        ) {
            Some(def) => Ty::Enum(def),
            None => Ty::Unknown,
        }
    }

    fn ty_name(&self, ty: Ty) -> String {
        match ty {
            Ty::Number => "number".into(),
            Ty::Float => "float".into(),
            Ty::Bool => "bool".into(),
            Ty::Enum(def) => self.defs[def.0].name.clone(),
            Ty::Unknown => "{unknown}".into(),
        }
    }

    fn evaluate_declarations(&mut self) {
        // Constant types first, so evaluation can check against them.
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            for item in self.package.module(module).items.clone() {
                let Some(name) = item.name() else { continue };
                let Some(def) = self.def_by_span(module, name.span) else {
                    continue;
                };
                if let ItemKind::Const(c) = &item.kind {
                    let ty = self.resolve_type(module, &c.ty);
                    if let Some(src) = self.const_src.get_mut(&def) {
                        src.ty = ty;
                    }
                }
            }
        }
        let consts: Vec<DefId> = self.const_src.keys().copied().collect();
        for def in consts {
            let value = self.eval_def(def);
            let ty = self.const_src[&def].ty;
            self.defs[def.0].info = DefInfo::Const { ty, value };
        }
        let variants: Vec<DefId> = self.variant_src.keys().copied().collect();
        for def in variants {
            let value = self.eval_def(def);
            self.defs[def.0].info = DefInfo::Variant { value };
        }
        // Enums must use distinct discriminants to be told apart in State.
        let enums: Vec<DefId> = (0..self.defs.len())
            .map(DefId)
            .filter(|d| self.defs[d.0].kind == DefKind::Enum)
            .collect();
        for def in enums {
            let DefInfo::Enum { variants } = self.defs[def.0].info.clone() else {
                continue;
            };
            let mut seen: HashMap<i64, DefId> = HashMap::new();
            for variant in variants {
                let DefInfo::Variant { value: Some(v) } = self.defs[variant.0].info else {
                    continue;
                };
                if let Some(prior) = seen.insert(v, variant) {
                    let (pf, ps) = (self.defs[prior.0].file, self.defs[prior.0].name_span);
                    let d = &self.defs[variant.0];
                    self.diagnostics.push(
                        Diagnostic::error(
                            "enum",
                            d.file,
                            d.name_span,
                            format!("discriminant {v} is already used"),
                        )
                        .with_note(pf, ps, "first used here"),
                    );
                }
            }
        }
        // State fields: types, defaults, and bounds.
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            for item in self.package.module(module).items.clone() {
                let ItemKind::State(state) = &item.kind else {
                    continue;
                };
                for attr in &item.attrs {
                    self.error(
                        "attribute",
                        file,
                        attr.path.span,
                        format!("unknown attribute `{}` on a state", attr.name()),
                    );
                }
                let Some(state_def) = self.def_by_span(module, state.name.span) else {
                    continue;
                };
                let field_defs = match &self.defs[state_def.0].info {
                    DefInfo::State { fields, .. } => fields.clone(),
                    _ => continue,
                };
                for (field, def) in state.fields.iter().zip(field_defs) {
                    self.field(module, file, field, def);
                }
            }
        }
    }

    fn field(&mut self, module: ModuleId, file: FileId, field: &Field, def: DefId) {
        let ty = self.resolve_type(module, &field.ty);
        let default = match (&field.default, ty) {
            (Some(expr), _) => self.eval_i32(module, file, expr, Some(ty), None),
            (None, Ty::Enum(e)) => match &self.defs[e.0].info {
                DefInfo::Enum { variants } => {
                    variants.first().and_then(|v| match self.defs[v.0].info {
                        DefInfo::Variant { value } => value,
                        _ => None,
                    })
                }
                _ => None,
            },
            (None, _) => Some(0),
        };
        let mut bounds = None;
        for attr in &field.attrs {
            if attr.name() != "clamp" {
                self.error(
                    "attribute",
                    file,
                    attr.path.span,
                    format!(
                        "unknown field attribute `{}`; expected `#[clamp(min..=max)]`",
                        attr.name()
                    ),
                );
                continue;
            }
            if !matches!(ty, Ty::Number | Ty::Float) {
                self.error(
                    "attribute",
                    file,
                    attr.span,
                    "only `number` and `float` fields can be clamped",
                );
                continue;
            }
            let [
                AttrArg {
                    key: None,
                    value: AttrValue::Range(range),
                    ..
                },
            ] = attr.args.as_slice()
            else {
                self.error(
                    "attribute",
                    file,
                    attr.span,
                    "expected `#[clamp(min..=max)]`",
                );
                continue;
            };
            let (Some(start), Some(end)) = (&range.start, &range.end) else {
                self.error(
                    "attribute",
                    file,
                    range.span,
                    "clamp needs both bounds, like `0..=100`",
                );
                continue;
            };
            let min = self.eval_i32(module, file, start, Some(ty), None);
            let max = self.eval_i32(module, file, end, Some(ty), None);
            let max = max.map(|m| if range.inclusive { m } else { m - 1 });
            if let (Some(min), Some(max)) = (min, max) {
                if min > max {
                    self.error("range", file, range.span, "clamp range is empty");
                } else {
                    bounds = Some((min as i32, max as i32));
                    if let Some(d) = default
                        && (d < min || d > max)
                    {
                        self.diagnostics.push(Diagnostic::warning(
                            "clamp",
                            file,
                            field.default.as_ref().map(Expr::span).unwrap_or(field.name.span),
                            format!(
                                "default {d} is outside the clamp range {min}..={max}; Sand keeps the declared default until the first write"
                            ),
                        ));
                    }
                }
            }
        }
        self.defs[def.0].info = DefInfo::Field {
            ty,
            default,
            bounds,
        };
    }

    fn def_by_span(&self, module: ModuleId, span: Span) -> Option<DefId> {
        let file = self.package.module(module).file;
        self.defs
            .iter()
            .position(|d| d.file == file && d.name_span == span)
            .map(DefId)
    }

    fn eval_def(&mut self, def: DefId) -> Option<i64> {
        match self.memo.get(&def) {
            Some(Memo::Done(value)) => return *value,
            Some(Memo::Evaluating) => {
                let d = &self.defs[def.0];
                let (file, span, name) = (d.file, d.name_span, d.name.clone());
                self.error(
                    "cycle",
                    file,
                    span,
                    format!("cycle detected while evaluating `{name}`"),
                );
                self.memo.insert(def, Memo::Done(None));
                return None;
            }
            None => {}
        }
        self.memo.insert(def, Memo::Evaluating);
        let value = if let Some(src) = self.const_src.get(&def) {
            let (module, file, expr, ty) = (src.module, src.file, src.expr.clone(), src.ty);
            self.eval_i32(module, file, &expr, Some(ty), None)
        } else if let Some(src) = self.variant_src.get(&def) {
            let (module, file, expr, prev) = (src.module, src.file, src.expr.clone(), src.prev);
            match expr {
                Some(expr) => self.eval_i32(module, file, &expr, Some(Ty::Number), None),
                None => match prev {
                    None => Some(0),
                    Some(prev) => self.eval_def(prev).and_then(|v| {
                        let next = v + 1;
                        if next > i32::MAX as i64 {
                            let d = &self.defs[def.0];
                            let (file, span) = (d.file, d.name_span);
                            self.error("range", file, span, "discriminant overflows i32");
                            None
                        } else {
                            Some(next)
                        }
                    }),
                },
            }
        } else {
            None
        };
        self.memo.insert(def, Memo::Done(value));
        value
    }

    /// Evaluate a constant expression and require it to fit in an i32.
    fn eval_i32(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        expected: Option<Ty>,
        bare: Option<DefId>,
    ) -> Option<i64> {
        let (value, ty) = self.eval(module, file, expr, bare)?;
        if let Some(expected) = expected
            && expected != Ty::Unknown
            && ty != Ty::Unknown
            && expected != ty
        {
            // A whole-number literal is a valid decimal: `speed: float = 1`.
            if expected == Ty::Float && ty == Ty::Number && is_int_literal(expr) {
                return Some(value * FLOAT_SCALE);
            }
            let message = format!(
                "mismatched types: expected `{}`, found `{}`",
                self.ty_name(expected),
                self.ty_name(ty)
            );
            let mut diagnostic = Diagnostic::error("type", file, expr.span(), message);
            if let (Ty::Number, Ty::Enum(_)) = (expected, ty) {
                diagnostic = diagnostic.with_help("convert the variant with `as number`");
            }
            self.diagnostics.push(diagnostic);
            return None;
        }
        if value < i32::MIN as i64 || value > i32::MAX as i64 {
            self.error(
                "range",
                file,
                expr.span(),
                format!("{value} does not fit in an i32 score"),
            );
            return None;
        }
        Some(value)
    }

    fn eval(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        bare: Option<DefId>,
    ) -> Option<(i64, Ty)> {
        match expr {
            Expr::Int(value, _) => Some((*value, Ty::Number)),
            Expr::Path(path) => {
                // Inside a flow, bare variant names refer to the field's enum.
                if let (Some(enum_def), [segment]) = (bare, path.segments.as_slice())
                    && let Some(variant) = self
                        .members
                        .get(&enum_def)
                        .and_then(|m| m.get(&segment.name))
                        .copied()
                    && !self.scopes[module.0].contains_key(&segment.name)
                {
                    self.references.push(Reference {
                        file,
                        span: segment.span,
                        def: variant,
                    });
                    let value = self.eval_def(variant)?;
                    return Some((value, Ty::Enum(enum_def)));
                }
                let def = self.expect_path(
                    module,
                    path,
                    &[DefKind::Const, DefKind::Variant],
                    "a constant or enum variant",
                )?;
                let value = self.eval_def(def)?;
                let ty = match self.defs[def.0].kind {
                    DefKind::Variant => Ty::Enum(self.defs[def.0].parent.expect("variant owner")),
                    _ => self
                        .const_src
                        .get(&def)
                        .map(|s| s.ty)
                        .unwrap_or(Ty::Unknown),
                };
                Some((value, ty))
            }
            Expr::Float(value, _) => Some(((value * FLOAT_SCALE as f64).round() as i64, Ty::Float)),
            Expr::Bool(value, _) => Some((i64::from(*value), Ty::Bool)),
            Expr::Str { span, .. } => {
                self.error("type", file, *span, "expected a constant value");
                None
            }
            Expr::Cond(_, span) => {
                self.error("const", file, *span, "a condition is not a constant");
                None
            }
            Expr::Call { span, path, .. } => {
                self.error(
                    "const",
                    file,
                    *span,
                    format!("`{}(...)` is not a constant", path.display()),
                );
                None
            }
            Expr::Place(place) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        "const",
                        file,
                        place.span,
                        "expected a constant value, found a State field",
                    )
                    .with_help(
                        "Sand currently writes constants into State; copying one field into another is not supported yet",
                    ),
                );
                None
            }
            Expr::Neg(inner, span) => {
                let (value, ty) = self.eval(module, file, inner, bare)?;
                if let Ty::Enum(_) = ty {
                    self.error("type", file, *span, "cannot negate an enum variant");
                    return None;
                }
                Some((-value, ty))
            }
            Expr::Binary(lhs, op, rhs, span) => {
                let (mut a, ta) = self.eval(module, file, lhs, bare)?;
                let (mut b, tb) = self.eval(module, file, rhs, bare)?;
                let bad = |t: Ty| matches!(t, Ty::Enum(_) | Ty::Bool);
                if bad(ta) || bad(tb) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "type",
                            file,
                            *span,
                            "arithmetic needs `number` or `float` operands",
                        )
                        .with_help("convert variants with `as number`"),
                    );
                    return None;
                }
                // A whole-number literal joins a decimal calculation.
                let float = match (ta, tb) {
                    (Ty::Float, Ty::Number) if is_int_literal(rhs) => {
                        b *= FLOAT_SCALE;
                        true
                    }
                    (Ty::Number, Ty::Float) if is_int_literal(lhs) => {
                        a *= FLOAT_SCALE;
                        true
                    }
                    (Ty::Float, Ty::Float) => true,
                    (Ty::Number, Ty::Number) => false,
                    (Ty::Unknown, _) | (_, Ty::Unknown) => false,
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "type",
                                file,
                                *span,
                                "cannot mix `number` and `float` here",
                            )
                            .with_help("convert one side with `as float` or `as number`"),
                        );
                        return None;
                    }
                };
                let result = match op {
                    BinOp::Add => a.checked_add(b),
                    BinOp::Sub => a.checked_sub(b),
                    BinOp::Mul if float => a.checked_mul(b).map(|v| v / FLOAT_SCALE),
                    BinOp::Mul => a.checked_mul(b),
                    BinOp::Div | BinOp::Rem if b == 0 => {
                        self.error("const", file, rhs.span(), "division by zero");
                        return None;
                    }
                    BinOp::Div if float => {
                        a.checked_mul(FLOAT_SCALE).and_then(|v| v.checked_div(b))
                    }
                    BinOp::Div => a.checked_div(b),
                    BinOp::Rem => a.checked_rem(b),
                };
                match result {
                    Some(value) => Some((value, if float { Ty::Float } else { Ty::Number })),
                    None => {
                        self.error("range", file, *span, "constant arithmetic overflows");
                        None
                    }
                }
            }
            Expr::Cast(inner, ty, span) => {
                let (value, from) = self.eval(module, file, inner, bare)?;
                let target = ty.segments.first().map(|s| s.name.as_str());
                match (from, target) {
                    (Ty::Float, Some("number")) => Some((value / FLOAT_SCALE, Ty::Number)),
                    (Ty::Number, Some("float")) => Some((value * FLOAT_SCALE, Ty::Float)),
                    (Ty::Float, Some("float")) => Some((value, Ty::Float)),
                    (_, Some("number")) => Some((value, Ty::Number)),
                    _ => {
                        self.error(
                            "type",
                            file,
                            *span,
                            "only `as number` and `as float` casts are supported",
                        );
                        None
                    }
                }
            }
        }
    }

    // ── Lowering ────────────────────────────────────────────────────────────

    fn lower(&mut self) -> Option<pc::Program> {
        let pack = self.pack.clone()?;
        let mut lowerer = Lowering::default();
        self.declare_signatures();
        let mut modules = Vec::new();
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            modules.push(self.lower_module(module, &mut lowerer));
        }
        self.infer_contexts(&mut lowerer);
        self.check_recursion(&lowerer);
        if !self.local_fields.is_empty()
            && let Some(root) = modules.first_mut()
            && let Ok(id) = self.locals_state_id().parse()
        {
            // One compiler-owned global State holds every local, parameter,
            // return value, and temporary slot.
            root.states.push(pc::State {
                id,
                scope: "global".into(),
                revision: 1,
                fields: self
                    .local_fields
                    .iter()
                    .map(|name| pc::ScoreField {
                        name: name.clone(),
                        default: 0,
                        bounds: None,
                    })
                    .collect(),
            });
        }
        // Contexts are known; finalise function declarations.
        for (index, module) in modules.iter_mut().enumerate() {
            for pending in lowerer.functions.iter().filter(|f| f.module.0 == index) {
                let context = match pending.def {
                    Some(def) => match &self.defs[def.0].info {
                        DefInfo::Function { context, .. } => *context,
                        _ => Context::Server,
                    },
                    None => pending.fixed.unwrap_or(Context::Server),
                };
                let ptr = format!("/functions/{}", module.functions.len());
                self.pointers
                    .push((module.id.to_string(), ptr, pending.file, pending.name_span));
                let Ok(id) = pending.id.parse() else {
                    self.error(
                        "id",
                        pending.file,
                        pending.name_span,
                        format!("`{}` is not a valid function identity", pending.id),
                    );
                    continue;
                };
                module.functions.push(pc::Function {
                    id,
                    context: match context {
                        Context::Server => pc::ExecutionContext::Server,
                        Context::Player => pc::ExecutionContext::Player,
                        Context::Entity => pc::ExecutionContext::Entity,
                    },
                    body: pending.body.clone(),
                });
            }
        }
        let mut requires = Vec::new();
        if lowerer.raw {
            requires.push("raw_commands".to_string());
        }
        Some(pc::Program {
            format: "sand.program".into(),
            format_version: 1,
            target: pc::ProgramTarget {
                minecraft: pack.minecraft.clone(),
            },
            pack: pc::Pack {
                namespace: pack.namespace.clone(),
                description: pack.description.clone(),
            },
            requires,
            modules,
        })
    }

    fn locals_state_id(&self) -> String {
        format!("{}:sand.locals", self.namespace())
    }

    /// Resolve every function's parameters and return type before bodies are
    /// lowered, so calls can target functions declared later.
    fn declare_signatures(&mut self) {
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            for item in self.package.module(module).items.clone() {
                let ItemKind::Fn(function) = &item.kind else {
                    continue;
                };
                let Some(def) = self.def_by_span(module, function.name.span) else {
                    continue;
                };
                let mut params = Vec::new();
                let mut seen = HashSet::new();
                let mut rendered = Vec::new();
                for param in &function.params {
                    if !seen.insert(param.name.name.clone()) {
                        self.error(
                            "duplicate",
                            file,
                            param.name.span,
                            format!("parameter `{}` is declared more than once", param.name.name),
                        );
                    }
                    let ty = self.resolve_type(module, &param.ty);
                    rendered.push(format!("{}: {}", param.name.name, self.ty_name(ty)));
                    let local = self.push_def(Def {
                        kind: DefKind::Local,
                        name: param.name.name.clone(),
                        module,
                        file,
                        name_span: param.name.span,
                        span: param.span,
                        docs: Vec::new(),
                        is_pub: false,
                        parent: None,
                        info: DefInfo::Local {
                            ty,
                            mutable: false,
                            param: true,
                            owner: Some(def),
                        },
                    });
                    params.push((format!("v{}_{}", local.0, param.name.name), local, ty));
                }
                let ret = function.ret.as_ref().map(|ret| {
                    let ty = self.resolve_type(module, ret);
                    (format!("f{}_ret", def.0), ty)
                });
                let mut signature = format!("({})", rendered.join(", "));
                if let Some((_, ty)) = &ret {
                    signature.push_str(&format!(" -> {}", self.ty_name(*ty)));
                }
                if let DefInfo::Function { signature: s, .. } = &mut self.defs[def.0].info {
                    *s = signature;
                }
                self.fn_slots.insert(def, FnSlots { params, ret });
            }
        }
    }

    /// Locals live in shared slots, so a function must not re-enter itself.
    fn check_recursion(&mut self, lowerer: &Lowering) {
        let graph: BTreeMap<DefId, Vec<(DefId, Span)>> = lowerer
            .facts
            .iter()
            .map(|(def, facts)| (*def, facts.all_calls.clone()))
            .collect();
        for (start, facts) in &lowerer.facts {
            let uses_slots = facts.has_slots;
            if !uses_slots {
                continue;
            }
            // Depth-first search for a path back to `start`.
            let mut stack: Vec<DefId> = graph[start].iter().map(|(d, _)| *d).collect();
            let mut seen = HashSet::new();
            let mut cyclic = false;
            while let Some(next) = stack.pop() {
                if next == *start {
                    cyclic = true;
                    break;
                }
                if seen.insert(next)
                    && let Some(calls) = graph.get(&next)
                {
                    stack.extend(calls.iter().map(|(d, _)| *d));
                }
            }
            if cyclic {
                let d = &self.defs[start.0];
                let (file, span, name) = (d.file, d.name_span, d.name.clone());
                self.diagnostics.push(
                    Diagnostic::error(
                        "recursion",
                        file,
                        span,
                        format!("`{name}` calls itself (directly or indirectly), but its variables live in shared slots"),
                    )
                    .with_help("Minecraft functions have no call stack; use a `while` loop instead of recursion"),
                );
            }
        }
    }

    fn module_identity(&self, module: ModuleId) -> String {
        let data = self.package.module(module);
        let mut path = vec!["crate".to_string()];
        path.extend(data.path.iter().cloned());
        format!("{}:{}", self.namespace(), path.join("/"))
    }

    fn lower_module(&mut self, module: ModuleId, lowerer: &mut Lowering) -> pc::Module {
        let file = self.package.module(module).file;
        let id_text = self.module_identity(module);
        let mut out = pc::Module {
            id: id_text.parse().expect("module identities are valid"),
            flows: Vec::new(),
            states: Vec::new(),
            functions: Vec::new(),
            load: Vec::new(),
            tick: Vec::new(),
            tags: Vec::new(),
            events: Vec::new(),
            items: Vec::new(),
            recipes: Vec::new(),
            resource_tags: Vec::new(),
        };
        self.pointers
            .push((id_text.clone(), String::new(), file, Span::default()));
        for item in self.package.module(module).items.clone() {
            let Some(name) = item.name().cloned() else {
                continue;
            };
            let Some(def) = self.def_by_span(module, name.span) else {
                continue;
            };
            match &item.kind {
                ItemKind::Data(data) => self.lower_data(module, file, def, data, &item, &mut out),
                ItemKind::State(state) => {
                    let DefInfo::State { id, fields, .. } = self.defs[def.0].info.clone() else {
                        continue;
                    };
                    let ptr = format!("/states/{}", out.states.len());
                    self.pointers
                        .push((id_text.clone(), ptr.clone(), file, name.span));
                    let mut score_fields = Vec::new();
                    for (index, field) in fields.iter().enumerate() {
                        let d = &self.defs[field.0];
                        self.pointers.push((
                            id_text.clone(),
                            format!("{ptr}/fields/{index}"),
                            file,
                            d.name_span,
                        ));
                        let DefInfo::Field {
                            default, bounds, ..
                        } = d.info
                        else {
                            continue;
                        };
                        score_fields.push(pc::ScoreField {
                            name: d.name.clone(),
                            default: default.unwrap_or(0) as i32,
                            bounds,
                        });
                    }
                    let Ok(state_id) = id.parse() else {
                        self.error(
                            "id",
                            file,
                            name.span,
                            format!("`{id}` is not a valid State identity"),
                        );
                        continue;
                    };
                    out.states.push(pc::State {
                        id: state_id,
                        scope: match state.scope {
                            Scope::Player => "player".into(),
                            Scope::Global => "global".into(),
                            Scope::Entity => "entity".into(),
                        },
                        revision: 1,
                        fields: score_fields,
                    });
                }
                ItemKind::Fn(function) => {
                    self.lower_fn(module, file, &item, function, def, &mut out, lowerer);
                }
                ItemKind::ExternFn(_) => {
                    for attr in &item.attrs {
                        if attr.name() != "player" {
                            self.error(
                                "attribute",
                                file,
                                attr.path.span,
                                format!(
                                    "unknown attribute `{}` on an extern function; expected `#[player]`",
                                    attr.name()
                                ),
                            );
                        }
                    }
                }
                ItemKind::Flow(flow) => {
                    self.reject_attrs(file, &item);
                    if let Some(lowered) = self.lower_flow(module, file, flow, def, lowerer) {
                        let ptr = format!("/flows/{}", out.flows.len());
                        self.pointers.push((id_text.clone(), ptr, file, name.span));
                        out.flows.push(lowered);
                    }
                }
                ItemKind::Const(_) | ItemKind::Enum(_) | ItemKind::Mod(_) => {
                    self.reject_attrs(file, &item);
                }
                ItemKind::Use(_) | ItemKind::Error(_) => {}
            }
        }
        out
    }

    /// Parse `#[on(event)]` or `#[on(use_item = "minecraft:item")]`.
    fn event_attr(
        &mut self,
        module: ModuleId,
        file: FileId,
        attr: &Attr,
    ) -> Option<(pc::PlayerEvent, String)> {
        let usage = || {
            crate::builtins::EVENTS
                .iter()
                .map(|(name, _, _)| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let [arg] = attr.args.as_slice() else {
            self.diagnostics.push(
                Diagnostic::error(
                    "attribute",
                    file,
                    attr.span,
                    "expected one event: `#[on(death)]`",
                )
                .with_help(format!("events: {}", usage())),
            );
            return None;
        };
        // Events that filter on a typed value: (key, value set, build).
        type Filtered = (&'static str, &'static str, fn(String) -> pc::PlayerEvent);
        const FILTERED: &[Filtered] = &[
            ("use_item", "Item", |item| pc::PlayerEvent::UseItem { item }),
            ("craft_item", "Item", |item| pc::PlayerEvent::CraftItem {
                item,
            }),
            ("pick_up_item", "Item", |item| pc::PlayerEvent::PickUpItem {
                item,
            }),
            ("mine_block", "Block", |block| pc::PlayerEvent::MineBlock {
                block,
            }),
            ("kill", "Entity", |entity| pc::PlayerEvent::KillEntityType {
                entity,
            }),
            ("killed_by", "Entity", |entity| pc::PlayerEvent::KilledBy {
                entity,
            }),
        ];
        match (&arg.key, &arg.value) {
            (None, AttrValue::Expr(Expr::Path(path))) if path.segments.len() == 1 => {
                let name = path.segments[0].name.as_str();
                let event = match name {
                    "join" => pc::PlayerEvent::Join,
                    "first_join" => pc::PlayerEvent::FirstJoin,
                    "death" => pc::PlayerEvent::Death,
                    "kill" => pc::PlayerEvent::KillEntity,
                    "kill_player" => pc::PlayerEvent::KillPlayer,
                    "jump" => pc::PlayerEvent::Jump,
                    "drop_item" => pc::PlayerEvent::DropItem,
                    "sleep" => pc::PlayerEvent::Sleep,
                    "take_damage" => pc::PlayerEvent::TakeDamage,
                    "deal_damage" => pc::PlayerEvent::DealDamage,
                    filtered => {
                        let diagnostic = match FILTERED.iter().find(|(key, _, _)| *key == filtered)
                        {
                            Some((key, set, _)) => Diagnostic::error(
                                "attribute",
                                file,
                                arg.span,
                                format!("`{key}` needs {} `{set}`", article(set)),
                            )
                            .with_help(format!("write `#[on({key} = {set}::...)]`")),
                            None => Diagnostic::error(
                                "attribute",
                                file,
                                path.span,
                                format!("unknown event `{name}`"),
                            )
                            .with_help(format!("events: {}", usage())),
                        };
                        self.diagnostics.push(diagnostic);
                        return None;
                    }
                };
                Some((event, name.to_string()))
            }
            (Some(key), value) if FILTERED.iter().any(|(k, _, _)| *k == key.name) => {
                let (_, set, build) = FILTERED
                    .iter()
                    .find(|(k, _, _)| *k == key.name)
                    .expect("checked above");
                let expr = match value {
                    AttrValue::Expr(expr) => expr.clone(),
                    AttrValue::Str(text, span) => Expr::Str {
                        value: text.clone(),
                        raw: text.clone(),
                        span: *span,
                    },
                    _ => {
                        self.error(
                            "attribute",
                            file,
                            arg.span,
                            format!("expected {} `{set}` such as `{set}::...`", article(set)),
                        );
                        return None;
                    }
                };
                if self
                    .declared(module, file, &expr, DefKind::CustomItem)
                    .is_some()
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "attribute",
                            file,
                            expr.span(),
                            format!("`{}` observes vanilla items", key.name),
                        )
                        .with_help("Minecraft counts statistics by base item; use the custom item's base item here"),
                    );
                    return None;
                }
                let id = self.vanilla_value(module, file, &expr, set)?;
                let label = match &expr {
                    Expr::Path(path) => format!("{} = {}", key.name, path.display()),
                    _ => format!("{} = \"{id}\"", key.name),
                };
                Some((build(id), label))
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error("attribute", file, arg.span, "expected an event name")
                        .with_help(format!("events: {}", usage())),
                );
                None
            }
        }
    }

    /// Warn about unused variables and needless `mut`.
    fn local_lints(&mut self, body: &BodyCx) {
        for local in &body.locals {
            let d = &self.defs[local.0];
            let (file, span, name) = (d.file, d.name_span, d.name.clone());
            let (mutable, param) = match d.info {
                DefInfo::Local { mutable, param, .. } => (mutable, param),
                _ => continue,
            };
            if name.starts_with('_') {
                continue;
            }
            if !body.used.contains(local) {
                let what = if param { "parameter" } else { "variable" };
                self.diagnostics.push(
                    Diagnostic::warning("unused", file, span, format!("unused {what} `{name}`"))
                        .with_help(format!("remove it, or name it `_{name}` to silence this")),
                );
            } else if mutable && !body.assigned.contains(local) {
                self.diagnostics.push(
                    Diagnostic::warning(
                        "mutability",
                        file,
                        span,
                        format!("variable `{name}` does not need to be mutable"),
                    )
                    .with_help("remove `mut`"),
                );
            }
        }
    }

    /// Resolve a typed vanilla value (`Item::Diamond`) or an identity string
    /// to a namespaced identity for `registry`.
    pub(crate) fn vanilla_value(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        registry: &'static str,
    ) -> Option<String> {
        let table = crate::vanilla::registry(registry);
        let noun = table.map(|t| t.noun).unwrap_or("value");
        if registry == "Item"
            && let Some((_, id)) = self.declared(module, file, expr, DefKind::CustomItem)
        {
            return Some(id);
        }
        match expr {
            Expr::Path(path)
                if path.segments.len() == 2
                    && crate::vanilla::registry(&path.segments[0].name).is_some()
                    && !self.scopes[module.0].contains_key(&path.segments[0].name) =>
            {
                let (given, variant) = (&path.segments[0], &path.segments[1]);
                if given.name != registry {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "type",
                            file,
                            given.span,
                            format!(
                                "expected {} `{registry}`, found {} `{}`",
                                article(registry),
                                article(&given.name),
                                given.name
                            ),
                        )
                        .with_help(format!("write `{registry}::...`")),
                    );
                    return None;
                }
                let table = table?;
                let Some(id) = table.id(&variant.name) else {
                    let suggestions = table.suggest(&variant.name);
                    let mut diagnostic = Diagnostic::error(
                        "resolve",
                        file,
                        variant.span,
                        format!("no {noun} named `{}` in `{registry}`", variant.name),
                    );
                    if let Some(best) = suggestions.first() {
                        diagnostic =
                            diagnostic.with_help(format!("did you mean `{registry}::{best}`?"));
                    }
                    self.diagnostics.push(diagnostic);
                    return None;
                };
                let span = Span::new(given.span.start, variant.span.end);
                self.vanilla_uses.push(VanillaUse {
                    file,
                    span,
                    registry: table.name,
                    variant: variant.name.clone(),
                    id: id.to_string(),
                });
                Some(id.to_string())
            }
            Expr::Str { value, span, .. } => {
                let valid = value.split_once(':').is_some_and(|(ns, path)| {
                    !ns.is_empty()
                        && !path.is_empty()
                        && ns.bytes().all(|b| {
                            b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-.".contains(&b)
                        })
                        && path.bytes().all(|b| {
                            b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-./".contains(&b)
                        })
                });
                if !valid {
                    self.diagnostics.push(
                        Diagnostic::error("id", file, *span, format!("`{value}` is not a valid {noun} identity"))
                            .with_help(format!("use a typed value such as `{registry}::...`, or a namespaced id like `\"mymod:name\"`")),
                    );
                    return None;
                }
                if let Some(table) = table
                    && value.starts_with("minecraft:")
                {
                    match table.by_id.get(value.as_str()) {
                        Some(variant) => self.diagnostics.push(
                            Diagnostic::warning(
                                "typed",
                                file,
                                *span,
                                format!("use `{registry}::{variant}` instead of a string"),
                            )
                            .with_help(
                                "typed values are checked, completed, and explained by the editor",
                            ),
                        ),
                        None => {
                            let path = value.trim_start_matches("minecraft:");
                            let guess: String = path
                                .split(['_', '.', '/'])
                                .map(|w| {
                                    let mut c = w.chars();
                                    c.next()
                                        .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                                        .unwrap_or_default()
                                })
                                .collect();
                            let mut diagnostic = Diagnostic::error(
                                "resolve",
                                file,
                                *span,
                                format!("Minecraft has no {noun} `{value}`"),
                            );
                            if let Some(best) = table.suggest(&guess).first() {
                                diagnostic = diagnostic
                                    .with_help(format!("did you mean `{registry}::{best}`?"));
                            }
                            self.diagnostics.push(diagnostic);
                            return None;
                        }
                    }
                }
                Some(value.clone())
            }
            other => {
                self.diagnostics.push(
                    Diagnostic::error("type", file, other.span(), format!("expected {} `{registry}` value", article(registry)))
                        .with_help(format!("for example `{registry}::...`; the editor lists every {noun} after `{registry}::`")),
                );
                None
            }
        }
    }

    fn reject_attrs(&mut self, file: FileId, item: &Item) {
        for attr in &item.attrs {
            self.error(
                "attribute",
                file,
                attr.path.span,
                format!("unknown attribute `{}`", attr.name()),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_fn(
        &mut self,
        module: ModuleId,
        file: FileId,
        item: &Item,
        function: &FnItem,
        def: DefId,
        out: &mut pc::Module,
        lowerer: &mut Lowering,
    ) {
        let DefInfo::Function { id, .. } = self.defs[def.0].info.clone() else {
            return;
        };
        let id_text = out.id.to_string();
        let reference = || pc::FunctionReference::Internal {
            id: id.parse().expect("validated below"),
        };
        let valid_id = id.parse::<sand::ResourceLocation>().is_ok();
        let mut forced = None;
        let mut entry = Vec::new();
        for attr in &item.attrs {
            match attr.name() {
                "player" => forced = Some(attr.span),
                "tick" | "load" => {
                    if !attr.args.is_empty() {
                        self.error(
                            "attribute",
                            file,
                            attr.span,
                            format!("`#[{}]` takes no arguments", attr.name()),
                        );
                    }
                    if !valid_id {
                        continue;
                    }
                    let call = pc::Operation {
                        action: pc::Action::Call {
                            function: reference(),
                        },
                        origin: Some(origin(file, function.name.span)),
                    };
                    let (list, label) = if attr.name() == "tick" {
                        (&mut out.tick, "tick")
                    } else {
                        (&mut out.load, "load")
                    };
                    self.pointers.push((
                        id_text.clone(),
                        format!("/{label}/{}", list.len()),
                        file,
                        attr.span,
                    ));
                    list.push(call);
                    entry.push((format!("#[{label}]"), attr.span));
                }
                "tag" => {
                    let [AttrArg {
                        key: None,
                        value: AttrValue::Str(tag, tag_span),
                        ..
                    }] = attr.args.as_slice()
                    else {
                        self.error(
                            "attribute",
                            file,
                            attr.span,
                            "expected `#[tag(\"namespace:name\")]`",
                        );
                        continue;
                    };
                    let Ok(tag_id) = tag.parse() else {
                        self.error(
                            "attribute",
                            file,
                            *tag_span,
                            format!("`{tag}` is not a valid function tag identity"),
                        );
                        continue;
                    };
                    if !valid_id {
                        continue;
                    }
                    self.pointers.push((
                        id_text.clone(),
                        format!("/tags/{}", out.tags.len()),
                        file,
                        attr.span,
                    ));
                    out.tags.push(pc::TagMembership {
                        tag: tag_id,
                        function: reference(),
                    });
                    entry.push((format!("#[tag(\"{tag}\")]"), attr.span));
                }
                "on" => {
                    let Some((event, label)) = self.event_attr(module, file, attr) else {
                        continue;
                    };
                    if !valid_id {
                        continue;
                    }
                    self.pointers.push((
                        id_text.clone(),
                        format!("/events/{}", out.events.len()),
                        file,
                        attr.span,
                    ));
                    out.events.push(pc::EventHook {
                        event,
                        function: reference(),
                    });
                    entry.push((format!("#[on({label})]"), attr.span));
                }
                other => self.error(
                    "attribute",
                    file,
                    attr.path.span,
                    format!(
                        "unknown function attribute `{other}`; expected `tick`, `load`, `tag`, `on`, or `player`"
                    ),
                ),
            }
        }
        let slots = self.fn_slots.get(&def).cloned();
        if !function.params.is_empty()
            && let Some((label, span)) = entry.first()
        {
            self.diagnostics.push(
                Diagnostic::error(
                    "params",
                    file,
                    *span,
                    format!("`{}` takes parameters, so {label} cannot call it", function.name.name),
                )
                .with_help("entry points run without arguments; call a parameterised function from inside them"),
            );
        }
        let mut body = BodyCx::new(module, file, false);
        body.slot = format!("f{}", def.0);
        body.owner = Some(def);
        if let Some(slots) = &slots {
            if let Some((field, ty)) = &slots.ret {
                body.ret = Some((self.slot(field.clone()), *ty));
            }
            for (field, param, _) in &slots.params {
                let name = self.defs[param.0].name.clone();
                body.scopes[0].push((name, *param));
                body.locals.push(*param);
                self.local_scopes.push((file, function.body.span, *param));
                let _ = field;
            }
        }
        let ops = self.lower_block(&mut body, &function.body);
        if let Some((_, ty)) = slots.as_ref().and_then(|s| s.ret.as_ref())
            && !block_returns(&function.body)
        {
            let ty = self.ty_name(*ty);
            self.diagnostics.push(
                Diagnostic::error(
                    "return",
                    file,
                    function.name.span,
                    format!(
                        "`{}` must return an `{ty}` on every path",
                        function.name.name
                    ),
                )
                .with_help("end the function with `return <value>;`"),
            );
        }
        self.local_lints(&body);
        let has_slots = !body.locals.is_empty()
            || slots
                .as_ref()
                .is_some_and(|s| s.ret.is_some() || !s.params.is_empty())
            || self
                .local_fields
                .iter()
                .any(|f| f.starts_with(&format!("{}_t", body.slot)));
        lowerer.raw |= body.raw;
        lowerer.facts.insert(
            def,
            Facts {
                entity_direct: body.entity_use.clone(),
                entity_loop_calls: body.entity_loop_calls.clone(),
                all_calls: body.all_calls.clone(),
                has_slots,
                schedules: body.schedules.clone(),
                forced,
                direct: body.player_use,
                calls: body.calls,
                entry,
                file,
            },
        );
        lowerer.functions.push(PendingFn {
            module,
            file,
            def: Some(def),
            fixed: None,
            id,
            name_span: function.name.span,
            body: ops,
        });
    }

    fn lower_flow(
        &mut self,
        module: ModuleId,
        file: FileId,
        flow: &FlowItem,
        def: DefId,
        lowerer: &mut Lowering,
    ) -> Option<pc::Flow> {
        let DefInfo::Flow { id } = self.defs[def.0].info.clone() else {
            return None;
        };
        let (state_def, field_def, field_ty) = self.resolve_place(module, file, &flow.on)?;
        let scope = match self.defs[state_def.0].info {
            DefInfo::State { scope, .. } => scope,
            _ => return None,
        };
        let hook_context = match scope {
            Scope::Player => Context::Player,
            Scope::Global => Context::Server,
            Scope::Entity => {
                self.diagnostics.push(
                    Diagnostic::error(
                        "flow",
                        file,
                        flow.on.span,
                        "flows run over player or global State, not entity State",
                    )
                    .with_help(
                        "drive entity behaviour from a `#[tick]` loop over `entities(with = ...)`",
                    ),
                );
                return None;
            }
        };
        let bare = match field_ty {
            Ty::Enum(e) => Some(e),
            _ => None,
        };
        let field_ref = self.field_reference(state_def, field_def)?;
        let phase = |cx: &mut Cx, expr: &Expr| -> Option<i32> {
            cx.eval_i32(module, file, expr, Some(field_ty), bare)
                .map(|v| v as i32)
        };
        let mut transitions = Vec::new();
        for transition in &flow.transitions {
            let from = phase(self, &transition.from);
            let to = phase(self, &transition.to);
            let when = match &transition.guard {
                None => Some(None),
                Some(cond) => self.flow_guard(module, file, cond, scope).map(Some),
            };
            let priority = match &transition.priority {
                None => Some(0),
                Some(expr) => self
                    .eval_i32(module, file, expr, Some(Ty::Number), None)
                    .map(|v| v as i32),
            };
            if let (Some(from), Some(to), Some(when), Some(priority)) = (from, to, when, priority) {
                if from == to {
                    self.error(
                        "flow",
                        file,
                        transition.span,
                        "a transition must move to a different phase",
                    );
                }
                transitions.push(pc::FlowTransition {
                    from,
                    to,
                    when,
                    priority,
                });
            }
        }
        let mut hooks = Vec::new();
        let mut generated: HashMap<String, usize> = HashMap::new();
        for hook in &flow.hooks {
            let state = phase(self, &hook.state);
            let cadence = match &hook.every {
                None => Some(1),
                Some(expr) => {
                    if hook.phase != HookPhase::Tick {
                        self.error(
                            "flow",
                            file,
                            expr.span(),
                            "only `tick` hooks can run `every` n cycles",
                        );
                        None
                    } else {
                        match self.eval_i32(module, file, expr, Some(Ty::Number), None) {
                            Some(v) if v >= 1 => Some(v as u32),
                            Some(_) => {
                                self.error("flow", file, expr.span(), "cadence must be at least 1");
                                None
                            }
                            None => None,
                        }
                    }
                }
            };
            let phase_name = match hook.phase {
                HookPhase::Enter => "enter",
                HookPhase::Exit => "exit",
                HookPhase::Tick => "tick",
            };
            let function = match &hook.target {
                HookTarget::Function(path) => {
                    let target = self.expect_path(
                        module,
                        path,
                        &[DefKind::Function, DefKind::ExternFunction],
                        "a function",
                    );
                    target.and_then(|target| {
                        if self
                            .fn_slots
                            .get(&target)
                            .is_some_and(|s| !s.params.is_empty())
                        {
                            self.error(
                                "params",
                                file,
                                path.span,
                                format!(
                                    "`{}` takes parameters, so a flow hook cannot call it",
                                    path.display()
                                ),
                            );
                            return None;
                        }
                        if scope == Scope::Global {
                            lowerer.global_hooks.push((target, file, path.span));
                        }
                        self.function_reference(target)
                    })
                }
                HookTarget::Block(block) => {
                    let label = match &hook.state {
                        Expr::Path(path) => snake_case(&path.last().name),
                        _ => state
                            .map(|v| v.to_string().replace('-', "neg"))
                            .unwrap_or_default(),
                    };
                    let base = format!("{id}.flow/{phase_name}_{label}");
                    let count = generated.entry(base.clone()).or_default();
                    *count += 1;
                    let fn_id = if *count == 1 {
                        base
                    } else {
                        format!("{base}_{count}")
                    };
                    let mut body = BodyCx::new(module, file, scope == Scope::Player);
                    body.slot = format!("h{}_{}", def.0, hooks.len() + generated.len());
                    let ops = self.lower_block(&mut body, block);
                    self.local_lints(&body);
                    lowerer.raw |= body.raw;
                    if let Some((span, reason)) = &body.player_use {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "context",
                                file,
                                *span,
                                format!("this global flow hook has no player, but it {reason}"),
                            )
                            .with_help("wrap it in `for player in players() { ... }`"),
                        );
                    }
                    for (callee, span) in &body.calls {
                        lowerer
                            .global_hooks_calls
                            .push((*callee, file, *span, scope));
                    }
                    for (target, span) in &body.schedules {
                        lowerer.hook_schedules.push((*target, file, *span));
                    }
                    let parsed = fn_id.parse().ok();
                    lowerer.functions.push(PendingFn {
                        module,
                        file,
                        def: None,
                        fixed: Some(hook_context),
                        id: fn_id,
                        name_span: hook.span,
                        body: ops,
                    });
                    parsed.map(|id| pc::FunctionReference::Internal { id })
                }
            };
            if let (Some(state), Some(cadence), Some(function)) = (state, cadence, function) {
                hooks.push(pc::FlowHook {
                    phase: match hook.phase {
                        HookPhase::Enter => pc::FlowHookPhase::Enter,
                        HookPhase::Exit => pc::FlowHookPhase::Exit,
                        HookPhase::Tick => pc::FlowHookPhase::Tick,
                    },
                    state,
                    function,
                    cadence,
                });
            }
        }
        if transitions.is_empty() && hooks.is_empty() && flow.errors.is_empty() {
            self.error(
                "flow",
                file,
                flow.name.span,
                "a flow needs at least one transition or hook",
            );
        }
        Some(pc::Flow {
            id: id.parse().ok()?,
            state: field_ref,
            transitions,
            hooks,
        })
    }

    fn flow_guard(
        &mut self,
        module: ModuleId,
        file: FileId,
        cond: &Cond,
        scope: Scope,
    ) -> Option<pc::ScoreComparison> {
        let simple = matches!(cond, Cond::Compare { op, .. } if *op != CmpOp::Ne)
            || matches!(cond, Cond::InRange { .. });
        if !simple {
            self.diagnostics.push(
                Diagnostic::error(
                    "flow",
                    file,
                    cond.span(),
                    "a transition guard must be a single comparison or `in` range",
                )
                .with_help("split the transition, or use priorities to order alternatives"),
            );
            return None;
        }
        let mut body = BodyCx::new(module, file, scope == Scope::Player);
        body.slot = "guard".into();
        let mut pre = Vec::new();
        let (condition, negated) = self.atom(&mut body, cond, &mut pre)?;
        let (pc::Condition::Range(comparison), true) = (condition, pre.is_empty()) else {
            self.diagnostics.push(
                Diagnostic::error(
                    "flow",
                    file,
                    cond.span(),
                    "a transition guard compares a field with constants",
                )
                .with_help("compare fields inside a hook and write the phase instead"),
            );
            return None;
        };
        debug_assert!(!negated);
        if let Some((span, reason)) = body.player_use {
            self.error(
                "context",
                file,
                span,
                format!("this global flow guard has no player, but it {reason}"),
            );
            return None;
        }
        Some(comparison)
    }

    fn field_reference(&self, state: DefId, field: DefId) -> Option<pc::FieldReference> {
        let DefInfo::State { id, .. } = &self.defs[state.0].info else {
            return None;
        };
        Some(pc::FieldReference {
            state: id.parse().ok()?,
            field: self.defs[field.0].name.clone(),
        })
    }

    fn function_reference(&self, def: DefId) -> Option<pc::FunctionReference> {
        match &self.defs[def.0].info {
            DefInfo::Function { id, .. } => Some(pc::FunctionReference::Internal {
                id: id.parse().ok()?,
            }),
            DefInfo::ExternFunction { id, context } => Some(pc::FunctionReference::External {
                id: id.parse().ok()?,
                context: match context {
                    Context::Server => pc::ExecutionContext::Server,
                    Context::Player => pc::ExecutionContext::Player,
                    Context::Entity => pc::ExecutionContext::Entity,
                },
            }),
            _ => None,
        }
    }

    fn resolve_place(
        &mut self,
        module: ModuleId,
        file: FileId,
        place: &Place,
    ) -> Option<(DefId, DefId, Ty)> {
        let state = self.expect_path(module, &place.state, &[DefKind::State], "a state")?;
        let Some(field) = self
            .members
            .get(&state)
            .and_then(|m| m.get(&place.field.name))
            .copied()
        else {
            let d = &self.defs[state.0];
            let available = self.members[&state]
                .keys()
                .map(|k| format!("`{k}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let message = format!("state `{}` has no field `{}`", d.name, place.field.name);
            let (df, ds) = (d.file, d.name_span);
            self.diagnostics.push(
                Diagnostic::error("resolve", file, place.field.span, message)
                    .with_note(df, ds, "state declared here")
                    .with_help(format!("available fields: {available}")),
            );
            return None;
        };
        self.references.push(Reference {
            file,
            span: place.field.span,
            def: field,
        });
        let ty = match self.defs[field.0].info {
            DefInfo::Field { ty, .. } => ty,
            _ => Ty::Unknown,
        };
        Some((state, field, ty))
    }

    fn lower_block(&mut self, body: &mut BodyCx, block: &Block) -> Vec<pc::Operation> {
        let mut ops = Vec::new();
        body.scopes.push(Vec::new());
        body.block_ends.push(block.span.end);
        let mut exited = false;
        for stmt in &block.stmts {
            if exited && !matches!(stmt, Stmt::Error(_)) {
                self.diagnostics.push(
                    Diagnostic::warning(
                        "unreachable",
                        body.file,
                        stmt_span(stmt),
                        "unreachable statement",
                    )
                    .with_help("the previous statement always returns or continues"),
                );
                break;
            }
            self.lower_stmt(body, stmt, &mut ops);
            exited = stmt_exits(stmt);
        }
        body.block_ends.pop();
        body.scopes.pop();
        self.diagnostics.append(&mut body.pending);
        ops
    }

    fn lower_stmt(&mut self, body: &mut BodyCx, stmt: &Stmt, ops: &mut Vec<pc::Operation>) {
        let file = body.file;
        match stmt {
            Stmt::Assign {
                target,
                op,
                value,
                span,
            } => self.lower_assign(body, target, *op, value, *span, ops),
            Stmt::Let {
                mutable,
                name,
                ty,
                value,
                span,
            } => self.lower_let(
                body,
                *mutable,
                name,
                ty.as_ref(),
                value.as_ref(),
                *span,
                ops,
            ),
            Stmt::Return { value, span } => self.lower_return(body, value.as_ref(), *span, ops),
            Stmt::Continue(span) => {
                if body.loops.is_empty() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "control",
                            file,
                            *span,
                            "`continue` only works inside a `for` loop",
                        )
                        .with_help("it skips to the next player or entity"),
                    );
                } else {
                    ops.push(op_at(pc::Action::Return, file, *span));
                }
            }
            Stmt::While {
                cond,
                body: block,
                span,
            } => self.lower_while(body, cond, block, *span, ops),
            Stmt::Call { path, args, span } => self.lower_call(body, path, args, *span, ops),
            Stmt::Macro { name, args, span } => self.lower_macro(body, name, args, *span, ops),
            Stmt::If(if_stmt) => {
                let lowered = self.lower_if(body, if_stmt);
                ops.extend(lowered);
            }
            Stmt::For {
                source,
                args,
                body: block,
                span,
                ..
            } => {
                let name = (source.segments.len() == 1).then(|| source.segments[0].name.as_str());
                let kind = match name {
                    Some("players") => LoopKind::Players,
                    Some("entities") => LoopKind::Entities,
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "for",
                                file,
                                source.span,
                                format!("cannot iterate `{}()`", source.display()),
                            )
                            .with_help(
                                "iterate `players()` to run as each player, or `entities(..)` to run as each matching entity",
                            ),
                        );
                        return;
                    }
                };
                let query = match kind {
                    LoopKind::Players => {
                        if let Some(first) = args.first() {
                            self.error("for", file, first.span, "`players()` takes no arguments; use `entities(type = Entity::Player, ..)` to filter");
                            return;
                        }
                        None
                    }
                    LoopKind::Entities => {
                        let builtin = crate::builtins::find(
                            "entities",
                            crate::builtins::BuiltinKind::Expression,
                        )
                        .expect("entities builtin");
                        self.builtin_uses
                            .push((file, source.span, builtin.name, builtin.kind));
                        let Some(query) = self.entity_query(body, builtin, args, source.span)
                        else {
                            return;
                        };
                        Some(query)
                    }
                };
                let was = body.in_players;
                body.in_players = true;
                body.loops.push(kind);
                if kind == LoopKind::Players {
                    body.player_loops += 1;
                }
                let inner = self.lower_block(body, block);
                if kind == LoopKind::Players {
                    body.player_loops -= 1;
                }
                body.loops.pop();
                body.in_players = was;
                let action = match query {
                    None => pc::Action::Players { body: inner },
                    Some(query) => pc::Action::Entities { query, body: inner },
                };
                ops.push(op_at(action, file, *span));
            }
            Stmt::Error(_) => {}
        }
    }

    fn lower_if(&mut self, body: &mut BodyCx, stmt: &IfStmt) -> Vec<pc::Operation> {
        let then = self.lower_block(body, &stmt.then);
        let otherwise = match &stmt.otherwise {
            None => Vec::new(),
            Some(Else::Block(block)) => self.lower_block(body, block),
            Some(Else::If(nested)) => self.lower_if(body, nested),
        };
        self.lower_cond(body, &stmt.cond, then, otherwise)
            .unwrap_or_default()
    }

    /// Lower a boolean condition into nested single-comparison branches.
    ///
    /// `a && b` becomes `if a { if b {T} else {E} } else {E}` and `a || b`
    /// becomes `if a {T} else { if b {T} else {E} }`. Each comparison is
    /// evaluated at most once and exactly one body runs.
    fn lower_cond(
        &mut self,
        body: &mut BodyCx,
        cond: &Cond,
        then: Vec<pc::Operation>,
        otherwise: Vec<pc::Operation>,
    ) -> Option<Vec<pc::Operation>> {
        match cond {
            Cond::Not(inner, _) => self.lower_cond(body, inner, otherwise, then),
            Cond::And(a, b) => {
                let inner = self.lower_cond(body, b, then, otherwise.clone())?;
                self.lower_cond(body, a, inner, otherwise)
            }
            Cond::Or(a, b) => {
                let inner = self.lower_cond(body, b, then.clone(), otherwise)?;
                self.lower_cond(body, a, then, inner)
            }
            Cond::Value(expr) => {
                let span = expr.span();
                let mut ops = Vec::new();
                let (condition, _) = self.atom(body, cond, &mut ops)?;
                if then.is_empty() && otherwise.is_empty() {
                    return Some(ops);
                }
                ops.push(op_at(
                    pc::Action::Branch {
                        condition,
                        then,
                        otherwise,
                    },
                    body.file,
                    span,
                ));
                Some(ops)
            }
            Cond::Compare { span, .. } | Cond::InRange { span, .. } | Cond::Call { span, .. } => {
                let mut ops = Vec::new();
                let (condition, negated) = self.atom(body, cond, &mut ops)?;
                let (then, otherwise) = if negated {
                    (otherwise, then)
                } else {
                    (then, otherwise)
                };
                if then.is_empty() && otherwise.is_empty() {
                    return Some(ops);
                }
                ops.push(op_at(
                    pc::Action::Branch {
                        condition,
                        then,
                        otherwise,
                    },
                    body.file,
                    *span,
                ));
                Some(ops)
            }
        }
    }

    /// Lower one comparison to a score test, plus whether the branches must
    /// be swapped (for `!=`). Sides that need computation are evaluated into
    /// temporaries by operations appended to `pre`, which run before the test.
    fn atom(
        &mut self,
        body: &mut BodyCx,
        cond: &Cond,
        pre: &mut Vec<pc::Operation>,
    ) -> Option<(pc::Condition, bool)> {
        let file = body.file;
        let depth = body.temp_depth;
        let result = self.atom_inner(body, cond, pre);
        body.temp_depth = depth;
        let _ = file;
        result
    }

    fn cond_value(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        ty: Ty,
        pre: &mut Vec<pc::Operation>,
    ) -> Option<statements::Value> {
        match self.simple(body, expr, ty) {
            Ok(Some(value)) => Some(value),
            Ok(None) => {
                let temp = self.temp(body);
                self.emit(body, &temp, ty, expr, pre)?;
                Some(statements::Value::Read(temp))
            }
            Err(()) => None,
        }
    }

    fn atom_inner(
        &mut self,
        body: &mut BodyCx,
        cond: &Cond,
        pre: &mut Vec<pc::Operation>,
    ) -> Option<(pc::Condition, bool)> {
        use statements::Value;
        let (module, file) = (body.module, body.file);
        let out_of_range =
            |v: Option<i64>| v.is_some_and(|v| v < i32::MIN as i64 || v > i32::MAX as i64);
        let range_condition = |cx: &mut Cx,
                               score: pc::FieldReference,
                               min: Option<i64>,
                               max: Option<i64>,
                               span: Span|
         -> Option<pc::Condition> {
            if out_of_range(min) || out_of_range(max) || min.zip(max).is_some_and(|(a, b)| a > b) {
                cx.error("condition", file, span, "this condition can never be true");
                return None;
            }
            Some(pc::Condition::Range(pc::ScoreComparison {
                score,
                min: min.map(|v| v as i32),
                max: max.map(|v| v as i32),
            }))
        };
        match cond {
            Cond::Call { path, args, span } => {
                self.predicate(body, path, args, *span).map(|c| (c, false))
            }
            Cond::Value(expr) => {
                let ty = self.type_of(body, expr);
                if !matches!(ty, Ty::Bool | Ty::Unknown) {
                    let found = self.ty_name(ty);
                    self.diagnostics.push(
                        Diagnostic::error(
                            "type",
                            file,
                            expr.span(),
                            format!("expected a `bool` condition, found `{found}`"),
                        )
                        .with_help("compare it, like `value > 0`"),
                    );
                    return None;
                }
                let Value::Read(score) = self.cond_value(body, expr, Ty::Bool, pre)? else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "condition",
                            file,
                            expr.span(),
                            "this condition is always the same",
                        )
                        .with_help("test a `bool` field or variable"),
                    );
                    return None;
                };
                Some((
                    range_condition(self, score, Some(1), Some(1), expr.span())?,
                    false,
                ))
            }
            Cond::InRange { value, range, span } => {
                let ty = self.type_of(body, value);
                if matches!(ty, Ty::Enum(_) | Ty::Bool) {
                    self.error(
                        "type",
                        file,
                        range.span,
                        "only `number` and `float` values can be tested against ranges",
                    );
                    return None;
                }
                let Value::Read(score) = self.cond_value(body, value, ty, pre)? else {
                    self.error(
                        "condition",
                        file,
                        value.span(),
                        "`in` tests a field or variable, like `Hero.energy in 0..=10`",
                    );
                    return None;
                };
                let start = match &range.start {
                    Some(e) => Some(self.eval_i32(module, file, e, Some(ty), None)?),
                    None => None,
                };
                let end = match &range.end {
                    Some(e) => Some(
                        self.eval_i32(module, file, e, Some(ty), None)?
                            - if range.inclusive { 0 } else { 1 },
                    ),
                    None => None,
                };
                Some((range_condition(self, score, start, end, *span)?, false))
            }
            Cond::Compare { lhs, op, rhs, span } => {
                let left_ty = self.type_of(body, lhs);
                let ty = if Self::is_const_in(body, lhs) && !matches!(left_ty, Ty::Enum(_)) {
                    self.type_of(body, rhs)
                } else {
                    left_ty
                };
                if matches!(ty, Ty::Enum(_) | Ty::Bool) && !matches!(op, CmpOp::Eq | CmpOp::Ne) {
                    let what = if ty == Ty::Bool { "bool" } else { "enum" };
                    self.error(
                        "type",
                        file,
                        *span,
                        format!("{what} values can only be compared with `==` or `!=`"),
                    );
                    return None;
                }
                let left = self.cond_value(body, lhs, ty, pre)?;
                let right = self.cond_value(body, rhs, ty, pre)?;
                let (score, op, value) = match (left, right) {
                    (Value::Read(a), Value::Read(b)) => {
                        let (relation, negated) = match op {
                            CmpOp::Eq => (pc::Relation::Eq, false),
                            CmpOp::Ne => (pc::Relation::Eq, true),
                            CmpOp::Lt => (pc::Relation::Lt, false),
                            CmpOp::Le => (pc::Relation::Le, false),
                            CmpOp::Gt => (pc::Relation::Gt, false),
                            CmpOp::Ge => (pc::Relation::Ge, false),
                        };
                        return Some((
                            pc::Condition::Relation(pc::ScoreRelation {
                                left: a,
                                relation,
                                right: b,
                            }),
                            negated,
                        ));
                    }
                    (Value::Read(a), Value::Const(c)) => (a, *op, c),
                    (Value::Const(c), Value::Read(b)) => (b, op.flipped(), c),
                    (Value::Const(_), Value::Const(_)) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "condition",
                                file,
                                *span,
                                "this condition compares two constants",
                            )
                            .with_help("compare a field or variable, like `Hero.energy >= 30`"),
                        );
                        return None;
                    }
                };
                let (min, max, negated) = match op {
                    CmpOp::Eq => (Some(value), Some(value), false),
                    CmpOp::Ne => (Some(value), Some(value), true),
                    CmpOp::Lt => (None, Some(value - 1), false),
                    CmpOp::Le => (None, Some(value), false),
                    CmpOp::Gt => (Some(value + 1), None, false),
                    CmpOp::Ge => (Some(value), None, false),
                };
                Some((range_condition(self, score, min, max, *span)?, negated))
            }
            _ => unreachable!("atoms are comparisons"),
        }
    }

    // ── Execution context inference ─────────────────────────────────────────

    fn infer_contexts(&mut self, lowerer: &mut Lowering) {
        let mut player: HashMap<DefId, String> = HashMap::new();
        for (def, facts) in &lowerer.facts {
            if facts.forced.is_some() {
                player.insert(*def, "is marked `#[player]`".into());
            } else if let Some((_, reason)) = &facts.direct {
                player.insert(*def, reason.clone());
            }
        }
        loop {
            let mut changed = false;
            for (def, facts) in &lowerer.facts {
                if player.contains_key(def) {
                    continue;
                }
                for (callee, _) in &facts.calls {
                    let needs = match &self.defs[callee.0].info {
                        DefInfo::ExternFunction { context, .. } => *context == Context::Player,
                        _ => player.contains_key(callee),
                    };
                    if needs {
                        let name = &self.defs[callee.0].name;
                        player.insert(*def, format!("calls `{name}`, which needs a player"));
                        changed = true;
                        break;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        // Functions that need some entity as `@s`, but not a player.
        let mut entity: HashMap<DefId, String> = HashMap::new();
        for (def, facts) in &lowerer.facts {
            if !player.contains_key(def)
                && let Some((_, reason)) = &facts.entity_direct
            {
                entity.insert(*def, reason.clone());
            }
        }
        loop {
            let mut changed = false;
            for (def, facts) in &lowerer.facts {
                if player.contains_key(def) || entity.contains_key(def) {
                    continue;
                }
                if let Some((callee, _)) = facts.calls.iter().find(|(c, _)| entity.contains_key(c))
                {
                    let name = &self.defs[callee.0].name;
                    entity.insert(*def, format!("calls `{name}`, which runs as an entity"));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for (def, facts) in &lowerer.facts {
            let (ctx, reason) = match (player.get(def), entity.get(def)) {
                (Some(reason), _) => (Context::Player, Some(reason.clone())),
                (None, Some(reason)) => (Context::Entity, Some(reason.clone())),
                (None, None) => (Context::Server, None),
            };
            if let DefInfo::Function {
                context,
                player_reason,
                entry,
                ..
            } = &mut self.defs[def.0].info
            {
                *context = ctx;
                *player_reason = reason.clone();
                *entry = facts.entry.iter().map(|(label, _)| label.clone()).collect();
            }
            let Some(reason) = reason else { continue };
            if ctx == Context::Entity {
                for (label, attr_span) in &facts.entry {
                    if label.starts_with("#[on(") {
                        continue;
                    }
                    let d = &self.defs[def.0];
                    self.diagnostics.push(
                        Diagnostic::error(
                            "context",
                            d.file,
                            d.name_span,
                            format!(
                                "`{}` runs from {label} on the server, but it {reason}",
                                d.name
                            ),
                        )
                        .with_note(facts.file, *attr_span, "runs on the server here")
                        .with_help("loop over entities with `for entity in entities(..) { ... }`"),
                    );
                }
                continue;
            }
            // Entry points run without a player executor.
            for (label, attr_span) in &facts.entry {
                let server_only = label == "#[tick]"
                    || label == "#[load]"
                    || label.contains("\"minecraft:tick\"")
                    || label.contains("\"minecraft:load\"");
                if !server_only {
                    continue;
                }
                let d = &self.defs[def.0];
                let mut diagnostic = Diagnostic::error(
                    "context",
                    d.file,
                    d.name_span,
                    format!(
                        "`{}` runs from {label} without a player, but it {reason}",
                        d.name
                    ),
                )
                .with_note(facts.file, *attr_span, "runs on the server here")
                .with_help("move player work into `for player in players() { ... }`");
                if let Some((span, _)) = &facts.direct {
                    diagnostic = diagnostic.with_note(facts.file, *span, "player state used here");
                } else if let Some((_, span)) = facts
                    .calls
                    .iter()
                    .find(|(callee, _)| player.contains_key(callee))
                {
                    diagnostic =
                        diagnostic.with_note(facts.file, *span, "player function called here");
                }
                self.diagnostics.push(diagnostic);
            }
        }
        for facts in lowerer.facts.values() {
            for (callee, span) in &facts.entity_loop_calls {
                let needs_player = match &self.defs[callee.0].info {
                    DefInfo::Function { context, .. } | DefInfo::ExternFunction { context, .. } => {
                        *context == Context::Player
                    }
                    _ => false,
                };
                if needs_player {
                    let name = self.defs[callee.0].name.clone();
                    self.diagnostics.push(
                        Diagnostic::error(
                            "context",
                            facts.file,
                            *span,
                            format!("`{name}` needs a player, but `entities(..)` loops run as entities that may not be players"),
                        )
                        .with_help("call it from `for player in players()` instead"),
                    );
                }
            }
        }
        let schedules: Vec<(DefId, FileId, Span)> = lowerer
            .facts
            .values()
            .flat_map(|facts| facts.schedules.iter().map(|(d, s)| (*d, facts.file, *s)))
            .chain(std::mem::take(&mut lowerer.hook_schedules))
            .collect();
        for (target, file, span) in schedules {
            let reason = match &self.defs[target.0].info {
                DefInfo::Function {
                    context: Context::Player | Context::Entity,
                    player_reason,
                    ..
                } => player_reason
                    .clone()
                    .unwrap_or_else(|| "needs a player".into()),
                DefInfo::ExternFunction {
                    context: Context::Player,
                    ..
                } => "is declared `#[player]`".into(),
                _ => continue,
            };
            let name = self.defs[target.0].name.clone();
            self.diagnostics.push(
                Diagnostic::error(
                    "context",
                    file,
                    span,
                    format!("`{name}` runs later on the server without a player or entity, but it {reason}"),
                )
                .with_help(
                    "schedule a server function that loops `for player in players() { ... }`",
                ),
            );
        }
        for (target, file, span) in std::mem::take(&mut lowerer.global_hooks) {
            let needs = match &self.defs[target.0].info {
                DefInfo::Function { context, .. } | DefInfo::ExternFunction { context, .. } => {
                    *context != Context::Server
                }
                _ => false,
            };
            if needs {
                self.error(
                    "context",
                    file,
                    span,
                    format!(
                        "`{}` needs a player or entity, but hooks of a global flow run on the server",
                        self.defs[target.0].name
                    ),
                );
            }
        }
        for (callee, file, span, scope) in std::mem::take(&mut lowerer.global_hooks_calls) {
            let needs = match &self.defs[callee.0].info {
                DefInfo::Function { context, .. } | DefInfo::ExternFunction { context, .. } => {
                    *context != Context::Server
                }
                _ => false,
            };
            if needs && scope == Scope::Global {
                self.error(
                    "context",
                    file,
                    span,
                    format!(
                        "`{}` needs a player or entity, but this global flow hook runs on the server",
                        self.defs[callee.0].name
                    ),
                );
            }
        }
    }
}

/// "a" or "an" before a name.
pub(crate) fn article(name: &str) -> &'static str {
    if name.starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    }
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Let { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Continue(span)
        | Stmt::While { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Macro { span, .. }
        | Stmt::For { span, .. } => *span,
        Stmt::If(if_stmt) => if_stmt.span,
        Stmt::Error(error) => error.span,
    }
}

/// Whether control never continues past this statement.
fn stmt_exits(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return { .. } | Stmt::Continue(_) => true,
        Stmt::If(if_stmt) => if_exits(if_stmt),
        _ => false,
    }
}

fn if_exits(if_stmt: &IfStmt) -> bool {
    let then = if_stmt.then.stmts.iter().any(stmt_exits);
    let otherwise = match &if_stmt.otherwise {
        None => false,
        Some(Else::Block(block)) => block.stmts.iter().any(stmt_exits),
        Some(Else::If(nested)) => if_exits(nested),
    };
    then && otherwise
}

/// Whether every path through the block ends in `return`.
pub(crate) fn block_returns(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Return { .. } => true,
        Stmt::If(if_stmt) => if_returns(if_stmt),
        _ => false,
    })
}

fn if_returns(if_stmt: &IfStmt) -> bool {
    let otherwise = match &if_stmt.otherwise {
        None => false,
        Some(Else::Block(block)) => block_returns(block),
        Some(Else::If(nested)) => if_returns(nested),
    };
    block_returns(&if_stmt.then) && otherwise
}

fn latest_target() -> String {
    pc::Compiler::capabilities()["targets"]
        .as_array()
        .and_then(|targets| targets.last())
        .and_then(|target| target.as_str())
        .unwrap_or("26.2")
        .to_string()
}

fn origin(file: FileId, span: Span) -> serde_json::Value {
    serde_json::json!({ "file": file.0, "start": span.start, "end": span.end })
}

fn op_at(action: pc::Action, file: FileId, span: Span) -> pc::Operation {
    pc::Operation {
        action,
        origin: Some(origin(file, span)),
    }
}

fn flatten_use(tree: &UseTree, prefix: &[Ident], out: &mut Vec<(Path, Ident)>) {
    let mut segments: Vec<Ident> = prefix.to_vec();
    segments.extend(tree.path.segments.iter().cloned());
    match &tree.group {
        Some(group) => {
            for child in group {
                flatten_use(child, &segments, out);
            }
        }
        None => {
            // `use a::{self}` imports `a` itself.
            if segments.len() > 1 && segments.last().is_some_and(|s| s.name == "self") {
                segments.pop();
            }
            let alias = tree
                .alias
                .clone()
                .unwrap_or_else(|| segments.last().expect("non-empty").clone());
            let span = segments
                .first()
                .expect("non-empty")
                .span
                .to(segments.last().expect("non-empty").span);
            out.push((Path { segments, span }, alias));
        }
    }
}

#[derive(Default)]
struct Lowering {
    /// Schedules made from generated flow hook bodies.
    hook_schedules: Vec<(DefId, FileId, Span)>,
    functions: Vec<PendingFn>,
    facts: BTreeMap<DefId, Facts>,
    raw: bool,
    global_hooks: Vec<(DefId, FileId, Span)>,
    global_hooks_calls: Vec<(DefId, FileId, Span, Scope)>,
}

struct PendingFn {
    module: ModuleId,
    file: FileId,
    def: Option<DefId>,
    fixed: Option<Context>,
    id: String,
    name_span: Span,
    body: Vec<pc::Operation>,
}

struct Facts {
    entity_direct: Option<(Span, String)>,
    entity_loop_calls: Vec<(DefId, Span)>,
    all_calls: Vec<(DefId, Span)>,
    has_slots: bool,
    schedules: Vec<(DefId, Span)>,
    forced: Option<Span>,
    direct: Option<(Span, String)>,
    calls: Vec<(DefId, Span)>,
    entry: Vec<(String, Span)>,
    file: FileId,
}

/// The kind of an executor-changing loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopKind {
    Players,
    Entities,
}

struct BodyCx {
    module: ModuleId,
    file: FileId,
    /// Slot prefix of the function being lowered, such as `f12`.
    slot: String,
    /// The function that owns locals declared in this body.
    owner: Option<DefId>,
    /// Locals declared in this body, for lints.
    locals: Vec<DefId>,
    /// Visible locals, innermost scope last.
    scopes: Vec<Vec<(String, DefId)>>,
    /// Current temporary nesting depth.
    temp_depth: usize,
    /// The return slot, when the function returns a value.
    ret: Option<(pc::FieldReference, Ty)>,
    /// Nesting depth of `for player in players()` loops.
    player_loops: usize,
    /// Locals read and reassigned, for lints.
    used: HashSet<DefId>,
    assigned: HashSet<DefId>,
    /// Every call, including those inside player loops.
    all_calls: Vec<(DefId, Span)>,
    /// Enclosing executor loops, innermost last.
    loops: Vec<LoopKind>,
    /// Calls made as entities inside `entities(..)` loops.
    entity_loop_calls: Vec<(DefId, Span)>,
    /// First use that needs an entity executor outside any loop.
    entity_use: Option<(Span, String)>,
    /// Errors found while noting uses, emitted by the enclosing block.
    pending: Vec<Diagnostic>,
    /// End offsets of enclosing blocks, for local visibility ranges.
    block_ends: Vec<usize>,
    in_players: bool,
    player_use: Option<(Span, String)>,
    calls: Vec<(DefId, Span)>,
    schedules: Vec<(DefId, Span)>,
    raw: bool,
}

impl BodyCx {
    fn new(module: ModuleId, file: FileId, in_players: bool) -> Self {
        Self {
            module,
            file,
            in_players,
            slot: "body".into(),
            owner: None,
            locals: Vec::new(),
            scopes: vec![Vec::new()],
            temp_depth: 0,
            ret: None,
            player_loops: 0,
            used: HashSet::new(),
            assigned: HashSet::new(),
            all_calls: Vec::new(),
            loops: Vec::new(),
            entity_loop_calls: Vec::new(),
            entity_use: None,
            pending: Vec::new(),
            block_ends: Vec::new(),
            player_use: None,
            calls: Vec::new(),
            schedules: Vec::new(),
            raw: false,
        }
    }

    fn note_player(&mut self, span: Span, reason: String) {
        if self.loops.last() == Some(&LoopKind::Entities) {
            self.entity_violation(span, &reason);
            return;
        }
        if !self.in_players && self.player_use.is_none() {
            self.player_use = Some((span, reason));
        }
    }

    /// A use that needs `@s` to be some entity, such as `kill()`.
    fn note_executor(&mut self, span: Span, reason: String) {
        if self.loops.is_empty() && self.entity_use.is_none() {
            self.entity_use = Some((span, reason));
        }
    }

    fn entity_violation(&mut self, span: Span, reason: &str) {
        self.pending.push(
            Diagnostic::error(
                "context",
                self.file,
                span,
                format!("this {reason}, but `entities(..)` loops run as entities that may not be players"),
            )
            .with_help("do player work in `for player in players()` instead"),
        );
    }

    fn note_state(&mut self, cx: &Cx, state: DefId, place: &Place, verb: &str) {
        if let DefInfo::State {
            scope: Scope::Entity,
            ..
        } = cx.defs[state.0].info
        {
            self.note_executor(
                place.span,
                format!(
                    "{verb} entity state `{}.{}`",
                    place.state.display(),
                    place.field.name
                ),
            );
            return;
        }
        if self.loops.last() == Some(&LoopKind::Entities)
            && let DefInfo::State {
                scope: Scope::Player,
                ..
            } = cx.defs[state.0].info
        {
            let reason = format!(
                "{verb} player state `{}.{}`",
                place.state.display(),
                place.field.name
            );
            self.entity_violation(place.span, &reason);
            return;
        }
        if self.in_players || self.player_use.is_some() {
            return;
        }
        if let DefInfo::State {
            scope: Scope::Player,
            ..
        } = cx.defs[state.0].info
        {
            self.player_use = Some((
                place.span,
                format!(
                    "{verb} player state `{}.{}`",
                    place.state.display(),
                    place.field.name
                ),
            ));
        }
    }
}

/// Collect the names of every identifier a set of definitions shadows; used
/// by completion to avoid offering both an import and its target.
pub(crate) fn scope_names(analysis: &Analysis, module: ModuleId) -> Vec<(String, DefId)> {
    let mut names: Vec<(String, DefId)> = Vec::new();
    let mut seen = HashSet::new();
    // Items declared in the module.
    for (index, def) in analysis.defs.iter().enumerate() {
        if def.module == module
            && def.parent.is_none()
            && def.kind != DefKind::Local
            && seen.insert(def.name.clone())
        {
            names.push((def.name.clone(), DefId(index)));
        }
    }
    // Imports, recovered from references inside `use` items.
    let data = analysis.package.module(module);
    for item in &data.items {
        if let ItemKind::Use(u) = &item.kind {
            let mut flat = Vec::new();
            flatten_use(&u.tree, &[], &mut flat);
            for (path, alias) in flat {
                let target = analysis
                    .references
                    .iter()
                    .find(|r| r.file == data.file && r.span == path.last().span)
                    .map(|r| r.def);
                if let Some(def) = target
                    && seen.insert(alias.name.clone())
                {
                    names.push((alias.name, def));
                }
            }
        }
    }
    names
}
