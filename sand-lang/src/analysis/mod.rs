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
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

// Minecraft uses floor division, including when the divisor is negative.
// Rust's truncating and Euclidean operations each differ for some signs.
fn floor_div(a: i64, b: i64) -> Option<i64> {
    let quotient = a.checked_div(b)?;
    if a.checked_rem(b)? != 0 && (a < 0) != (b < 0) {
        quotient.checked_sub(1)
    } else {
        Some(quotient)
    }
}

fn floor_rem(a: i64, b: i64) -> Option<i64> {
    let remainder = a.checked_rem(b)?;
    if remainder != 0 && (remainder < 0) != (b < 0) {
        remainder.checked_add(b)
    } else {
        Some(remainder)
    }
}

mod callables;
mod collections;
mod continuations;
mod generics;
mod resources;
mod sequence_methods;
mod sequences;
mod source_map;
mod statements;
mod testing;
mod text;
mod traits;
mod values;

pub use testing::TestCase;

/// Index of a definition within an [`Analysis`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(pub usize);

/// The value type of a constant, field, or expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ty {
    /// A whole number, stored directly in a score.
    Number,
    /// A decimal stored as fixed point: the score holds `value * 1000`.
    Float,
    /// `true` or `false`, stored as one or zero.
    Bool,
    /// Owned Unicode text stored as a sequence of scalar characters.
    String,
    /// A growable owned homogeneous sequence.
    List(DefId),
    /// An enum whose variants are integer phases.
    Enum(DefId),
    /// A named, copyable record layout.
    Struct(DefId),
    /// A retained, pack-owned entity identity; zero is empty.
    EntityRef,
    /// An opaque reference to a checked function with a concrete signature.
    Callable(DefId),
    /// A type that failed to resolve; suppresses follow-up errors.
    Unknown,
}

/// Where a function runs: on the server, or as a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    /// A named record value type.
    Struct,
    /// A generic declaration's type parameter.
    TypeParameter,
    /// A named method contract.
    Trait,
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
    /// `event Name(...)`.
    Event,
    /// `bossbar Name { ... }`.
    BossBar,
    /// `loot Name { ... }`.
    Loot,
    /// `damage_type Name { ... }`.
    DamageType,
    /// `predicate Name { ... }`.
    Predicate,
    /// `advancement Name { ... }`.
    Advancement,
    /// `dialog Name { ... }`.
    Dialog,
    /// `dimension Name { ... }`.
    Dimension,
    /// `biome Name { ... }`.
    Biome,
    /// `structure_template Name { ... }`.
    StructureTemplate,
    /// `inventory_transfer Name { ... }`.
    InventoryTransfer,
    /// `hud Name { ... }`.
    Hud,
    /// `menu Name { ... }`.
    Menu,
    /// `projectile Name { ... }`.
    Projectile,
    /// A named player quest.
    Progression,
    /// A loaded-tick controller over a quest.
    Scene,
    Job,
    Session,
    /// A checked player activation.
    Ability,
    /// An authoritative custom health policy.
    Combat,
}

impl DefKind {
    /// Source-facing description.
    pub fn describe(self) -> &'static str {
        match self {
            DefKind::Module => "module",
            DefKind::Const => "constant",
            DefKind::Enum => "enum",
            DefKind::Struct => "struct",
            DefKind::TypeParameter => "type parameter",
            DefKind::Trait => "trait",
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
            DefKind::Event => "event",
            DefKind::BossBar => "boss bar",
            DefKind::Loot => "loot table",
            DefKind::DamageType => "damage type",
            DefKind::Predicate => "predicate",
            DefKind::Advancement => "advancement",
            DefKind::Dialog => "dialog",
            DefKind::Dimension => "dimension",
            DefKind::Biome => "biome",
            DefKind::StructureTemplate => "structure template",
            DefKind::InventoryTransfer => "inventory transfer",
            DefKind::Hud => "HUD",
            DefKind::Menu => "menu",
            DefKind::Projectile => "projectile",
            DefKind::Progression => "progression",
            DefKind::Scene => "scene",
            DefKind::Job => "job",
            DefKind::Session => "session",
            DefKind::Ability => "ability",
            DefKind::Combat => "combat",
        }
    }
}

/// Kind-specific facts about a definition.
#[derive(Debug, Clone)]
pub enum DefInfo {
    /// An interned callable signature.
    Callable {
        /// Minimum executor required at invocation.
        context: Context,
        /// Argument types in call order.
        parameters: Vec<Ty>,
        /// Return type when the callable produces a value.
        returns: Option<Ty>,
    },
    /// A generic type parameter, resolved by each concrete specialization.
    TypeParameter,
    /// An authored contract with required method signatures.
    Trait,
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
    /// A named record layout, shared by locals, arguments, returns, and State.
    Struct {
        /// Field definitions in declaration order.
        fields: Vec<DefId>,
    },
    /// An interned fixed-size array using the same leaf layout as records.
    Array {
        /// Element type.
        element: Ty,
        /// Compiler-owned element field definitions in index order.
        fields: Vec<DefId>,
    },
    /// An interned growable collection; its element may contain nested values.
    List { element: Ty },
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
        /// Whether the field counts down by itself.
        timer: bool,
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
        /// Parameters and return type as written, such as `(cost: number) -> number`.
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

/// A concrete function's value signature, projected from the shared call slots.
#[derive(Debug, Clone)]
pub struct FunctionType {
    /// Parameter definitions in call order, including a struct receiver.
    pub parameters: Vec<DefId>,
    /// Returned value type, absent for a statement-only function.
    pub returns: Option<Ty>,
}

/// The complete result of analysing a package.
#[derive(Debug, Clone)]
pub struct Analysis {
    /// The analysed package.
    pub package: Package,
    /// Every definition.
    pub defs: Vec<Def>,
    /// Concrete specialization definitions mapped to their authored origin.
    pub origins: BTreeMap<DefId, DefId>,
    /// Concrete signatures shared by compiler and editor value inference.
    pub function_types: BTreeMap<DefId, FunctionType>,
    /// Sand-authored tests, in discovery order.
    pub tests: Vec<TestCase>,
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
    /// Entity properties such as `Player.health`, for hover.
    pub property_uses: Vec<PropertyUse>,
    /// Loop bodies where a binding is an entity handle: (file, body, name,
    /// whether it is a player), for completion after `zombie.`.
    pub handle_scopes: Vec<(FileId, Span, String, bool)>,
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
            // Shorthand patterns name both a field and a binding. The local
            // is the symbol the author declares or reads at that location.
            .min_by_key(|(span, def)| {
                (
                    span.end - span.start,
                    self.defs[def.0].kind != DefKind::Local,
                )
            })
            .map(|(_, def)| def)
    }

    /// Authored identity of a concrete specialization definition.
    pub fn origin(&self, def: DefId) -> DefId {
        self.origins.get(&def).copied().unwrap_or(def)
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

/// Shared spelling for naming diagnostics and editor rename fixes.
pub(crate) fn name_suggestion(def: &Def) -> Option<(&'static str, String)> {
    let name = def.name.trim_start_matches('_');
    if name.is_empty() || def.name_span == Span::default() {
        return None;
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
        _ => ("PascalCase", pascal_case(name)),
    };
    if fixed == name || fixed.is_empty() {
        return None;
    }
    let prefix = &def.name[..def.name.len() - name.len()];
    Some((style, format!("{prefix}{fixed}")))
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
    analyze_mode(package, false)
}

/// Lower a package with its discovered test entry points enabled.
pub fn analyze_tests(package: Package) -> Analysis {
    analyze_mode(package, true)
}

fn analyze_mode(package: Package, test_mode: bool) -> Analysis {
    let mut cx = Cx {
        diagnostics: package.diagnostics.clone(),
        package,
        defs: Vec::new(),
        references: Vec::new(),
        scopes: Vec::new(),
        members: BTreeMap::new(),
        methods: BTreeMap::new(),
        record_defaults: BTreeMap::new(),
        array_types: BTreeMap::new(),
        list_types: BTreeMap::new(),
        const_src: BTreeMap::new(),
        variant_src: BTreeMap::new(),
        memo: BTreeMap::new(),
        pack: None,
        pointers: Vec::new(),
        builtin_uses: Vec::new(),
        local_scopes: Vec::new(),
        pattern_bindings: BTreeMap::new(),
        inferred_block_locals: BTreeMap::new(),
        vanilla_uses: Vec::new(),
        local_fields: Vec::new(),
        local_data_fields: BTreeMap::new(),
        text_constants: BTreeMap::new(),
        fn_slots: BTreeMap::new(),
        generics: generics::Generics::default(),
        traits: traits::Traits::default(),
        callables: callables::Callables::default(),
        test_mode,
        tests: BTreeMap::new(),
        test_functions: Default::default(),
        event_handlers: BTreeMap::new(),
        handler_attrs: HashSet::new(),
        call_frames: Vec::new(),
        property_uses: Vec::new(),
        handle_scopes: Vec::new(),
    };
    cx.collect();
    cx.resolve_imports();
    cx.pack_info();
    cx.declare_constant_types();
    cx.evaluate_structs();
    cx.evaluate_declarations();
    let program = cx.lower();
    cx.references
        .retain(|reference| reference.span.start < reference.span.end);
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
        origins: cx.generics.origins,
        tests: cx.tests.into_values().collect(),
        function_types: cx
            .fn_slots
            .into_iter()
            .map(|(def, slots)| {
                (
                    def,
                    FunctionType {
                        parameters: slots.params.into_iter().map(|(_, def, _)| def).collect(),
                        returns: slots.ret.map(|(_, ty)| ty),
                    },
                )
            })
            .collect(),
        references: cx.references,
        diagnostics: cx.diagnostics,
        pack: cx.pack,
        program: if has_errors { None } else { program },
        pointers: cx.pointers,
        builtin_uses: cx.builtin_uses,
        local_scopes: cx.local_scopes,
        vanilla_uses: cx.vanilla_uses,
        property_uses: cx.property_uses,
        handle_scopes: cx.handle_scopes,
    }
}

#[derive(Debug, Clone, Copy)]
struct Binding {
    def: DefId,
    is_pub: bool,
}

#[derive(Clone)]
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
    methods: BTreeMap<DefId, values::Method>,
    record_defaults: BTreeMap<DefId, Vec<text::LeafDefault>>,
    array_types: BTreeMap<(Ty, usize), DefId>,
    list_types: BTreeMap<Ty, DefId>,
    const_src: BTreeMap<DefId, ConstSrc>,
    variant_src: BTreeMap<DefId, VariantSrc>,
    memo: BTreeMap<DefId, Memo>,
    pack: Option<PackInfo>,
    pointers: Vec<(String, String, FileId, Span)>,
    builtin_uses: Vec<(FileId, Span, &'static str, crate::builtins::BuiltinKind)>,
    local_scopes: Vec<(FileId, Span, DefId)>,
    pattern_bindings: BTreeMap<(FileId, Span, String), Vec<statements::PatternLocal>>,
    inferred_block_locals: BTreeMap<(FileId, Span, String), DefId>,
    vanilla_uses: Vec<VanillaUse>,
    /// Fields of the compiler-owned locals State, in allocation order.
    local_fields: Vec<String>,
    local_data_fields: BTreeMap<String, sand::data::NbtValue>,
    text_constants: BTreeMap<DefId, String>,
    /// Parameter and return slots per function.
    fn_slots: BTreeMap<DefId, FnSlots>,
    generics: generics::Generics,
    traits: traits::Traits,
    callables: callables::Callables,
    test_mode: bool,
    tests: BTreeMap<DefId, TestCase>,
    test_functions: std::collections::BTreeSet<DefId>,
    /// Handlers of each custom event, in declaration order.
    event_handlers: BTreeMap<DefId, Vec<(DefId, Span)>>,
    /// `#[on(Event)]` attributes that name a custom event.
    handler_attrs: HashSet<(FileId, Span)>,
    /// Caller and callee of each frame marker, indexed by marker number.
    call_frames: Vec<(Option<DefId>, DefId)>,
    property_uses: Vec<PropertyUse>,
    handle_scopes: Vec<(FileId, Span, String, bool)>,
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

/// What `value as T` does to the stored score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CastStep {
    /// The score is already right.
    Same,
    /// Multiply by the fixed-point scale.
    ToFloat,
    /// Divide by the fixed-point scale, rounding down.
    ToNumber,
    /// One when nonzero, zero otherwise.
    ToBool,
}

/// One entity property written in source, such as `Player.health`.
#[derive(Debug, Clone)]
pub struct PropertyUse {
    /// File containing the use.
    pub file: FileId,
    /// The property name's span.
    pub span: Span,
    /// The handle as written: `Player` or a loop binding.
    pub handle: String,
    /// The property.
    pub property: crate::properties::Property,
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
    receiver_out: Option<(String, Ty)>,
}

type Resolved = Result<(DefId, Vec<Reference>), Box<Diagnostic>>;

impl Cx {
    fn error(&mut self, code: &str, file: FileId, span: Span, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(code, file, span, message));
    }

    fn push_def(&mut self, def: Def) -> DefId {
        let id = DefId(self.defs.len());
        if let Some(template) = self.generics.active {
            let key = (template, def.file, def.name_span);
            if let Some(origin) = self.generics.source_defs.get(&key) {
                self.generics.origins.insert(id, *origin);
            } else {
                self.generics.source_defs.insert(key, id);
            }
        }
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
            if self.generics.origins.contains_key(&DefId(index)) {
                continue;
            }
            let def = &self.defs[index];
            let Some((style, fixed)) = name_suggestion(def) else {
                continue;
            };
            let what = def.kind.describe();
            let (file, span) = (def.file, def.name_span);
            self.diagnostics.push(
                Diagnostic::warning(
                    "naming",
                    file,
                    span,
                    format!("{what} `{}` should have a {style} name", def.name),
                )
                .with_help(format!("rename it to `{fixed}`")),
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
            ItemKind::Struct(record) => self.collect_struct(module, file, item, record),
            ItemKind::Trait(declaration) => {
                self.reject_attrs(file, item);
                let def = self.push_def(base(DefKind::Trait, DefInfo::Trait));
                self.collect_trait(def, declaration);
                def
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
                            timer: false,
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
            ItemKind::Event(_) => {
                // An event compiles to a dispatcher function, so it takes part
                // in context inference and recursion checks like one.
                self.push_def(base(
                    DefKind::Event,
                    DefInfo::Function {
                        id: event_id(&self.item_id(module, &name.name)),
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
                    crate::ast::DataKind::BossBar => DefKind::BossBar,
                    crate::ast::DataKind::Loot => DefKind::Loot,
                    crate::ast::DataKind::DamageType => DefKind::DamageType,
                    crate::ast::DataKind::Predicate => DefKind::Predicate,
                    crate::ast::DataKind::Advancement => DefKind::Advancement,
                    crate::ast::DataKind::Dialog => DefKind::Dialog,
                    crate::ast::DataKind::Dimension => DefKind::Dimension,
                    crate::ast::DataKind::Biome => DefKind::Biome,
                    crate::ast::DataKind::StructureTemplate => DefKind::StructureTemplate,
                    crate::ast::DataKind::InventoryTransfer => DefKind::InventoryTransfer,
                    crate::ast::DataKind::Hud => DefKind::Hud,
                    crate::ast::DataKind::Menu => DefKind::Menu,
                    crate::ast::DataKind::Projectile => DefKind::Projectile,
                    crate::ast::DataKind::Progression => DefKind::Progression,
                    crate::ast::DataKind::Scene => DefKind::Scene,
                    crate::ast::DataKind::Job => DefKind::Job,
                    crate::ast::DataKind::Session => DefKind::Session,
                    crate::ast::DataKind::Ability => DefKind::Ability,
                    crate::ast::DataKind::Combat => DefKind::Combat,
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
            ItemKind::Use(_) | ItemKind::Impl(_) | ItemKind::Error(_) => return,
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
        // Type aliases resolve first; methods then join the same import fixpoint.
        let mut implementations = false;
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
                if !implementations {
                    self.collect_impls();
                    implementations = true;
                    if !pending.is_empty() {
                        continue;
                    }
                }
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

    fn resolve_path(&mut self, from: ModuleId, path: &Path) -> Resolved {
        let file = self.package.module(from).file;
        let fail = |span: Span, message: String| {
            Err(Box::new(Diagnostic::error("resolve", file, span, message)))
        };
        let mut refs = Vec::new();
        let mut cur = Cur::Module(from);
        let mut segments = path.segments.iter().peekable();
        let first = segments.peek().expect("non-empty path");
        let method_target = self
            .generics
            .bindings
            .get(&first.name)
            .and_then(|(ty, parameter)| match ty {
                Ty::Struct(def) | Ty::Enum(def) => Some((*def, *parameter)),
                _ => None,
            })
            .or_else(|| {
                self.methods.iter().find_map(|(def, method)| {
                    let owner = &self.defs[def.0];
                    (owner.module == from
                        && owner.span.contains(path.span.start)
                        && (first.name == "Self"
                            || (first.name == "self"
                                && path.segments.len() == 1
                                && method.state_receiver)))
                        .then_some((method.target, method.target))
                })
            });
        if let Some((target, reference)) = method_target {
            let segment = segments.next().expect("Self");
            refs.push(Reference {
                file,
                span: segment.span,
                def: reference,
            });
            cur = Cur::Def(target);
        } else {
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
        }
        while let Some(segment) = segments.next() {
            let index = path.segments.len() - segments.len() - 1;
            if let Some(application) = path
                .applications
                .iter()
                .find(|application| application.segment + 1 == index)
            {
                let Cur::Def(def) = cur else {
                    return fail(path.span, "type arguments require a generic struct".into());
                };
                let types = application
                    .arguments
                    .iter()
                    .map(|ty| self.resolve_type(from, ty))
                    .collect();
                let Some(Ty::Struct(instance)) =
                    self.instantiate_record(def, types, file, path.span)
                else {
                    return fail(path.span, "cannot instantiate this generic type".into());
                };
                cur = Cur::Def(instance);
            }
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
                        (DefKind::State | DefKind::Struct, Some(method))
                            if self.defs[method.0].kind == DefKind::Function =>
                        {
                            if !self.method_visible(from, method) {
                                return fail(
                                    segment.span,
                                    format!("method `{}` is private", segment.name),
                                );
                            }
                            refs.push(Reference {
                                file,
                                span: segment.span,
                                def: method,
                            });
                            cur = Cur::Def(method);
                        }
                        (DefKind::State | DefKind::Struct, _) => {
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
                if !path.arguments().is_empty()
                    && !matches!(found.kind, DefKind::Function | DefKind::Struct)
                {
                    self.error(
                        "generic",
                        self.package.module(module).file,
                        path.span,
                        "type arguments require a generic function or struct",
                    );
                    return None;
                }
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
        let minecraft = if minecraft == "latest" {
            sand_version::LATEST_KNOWN.to_string()
        } else {
            minecraft
        };
        if sand_version::registry::variants(&minecraft, "minecraft:item").is_none() {
            self.error(
                "target",
                root,
                pack_span.unwrap_or(ns_span),
                format!("no verified vanilla catalog for Minecraft {minecraft}"),
            );
            return;
        }

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
            let mut fresh = self.item_id(def.module, &def.name);
            if let Some(method) = self.methods.get(&DefId(index)) {
                let target = &self.defs[method.target.0];
                fresh = format!(
                    "{}.method/{}",
                    self.item_id(target.module, &target.name),
                    def.name
                );
            } else if def.kind == DefKind::Event {
                fresh = event_id(&fresh);
            }
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

    fn resolve_type(&mut self, module: ModuleId, annotation: &Type) -> Ty {
        match &annotation.kind {
            TypeKind::Function {
                executor,
                parameters,
                returns,
            } => {
                let parameters = parameters
                    .iter()
                    .map(|ty| self.resolve_type(module, ty))
                    .collect();
                let returns = returns.as_ref().map(|ty| self.resolve_type(module, ty));
                self.callable_type(
                    module,
                    callables::Signature {
                        context: match executor {
                            Some(CallableExecutor::Player) => Context::Player,
                            Some(CallableExecutor::Entity) => Context::Entity,
                            None => Context::Server,
                        },
                        parameters,
                        returns,
                    },
                )
            }
            TypeKind::Named(path) => self.resolve_named_type(module, path),
            TypeKind::Applied { path, arguments } => {
                let file = self.package.module(module).file;
                if !path.arguments().is_empty() {
                    self.error(
                        "generic",
                        file,
                        annotation.span,
                        "duplicate type application",
                    );
                    return Ty::Unknown;
                }
                if path.segments.len() == 1 && path.last().name == "Vec" {
                    if arguments.len() != 1 {
                        self.error(
                            "type",
                            file,
                            annotation.span,
                            "expected `Vec<T>` with one element type",
                        );
                        return Ty::Unknown;
                    }
                    let element = self.resolve_type(module, &arguments[0]);
                    self.list_type(module, element)
                } else {
                    let Some(def) =
                        self.expect_path(module, path, &[DefKind::Struct], "a generic struct")
                    else {
                        return Ty::Unknown;
                    };
                    let types = arguments
                        .iter()
                        .map(|ty| self.resolve_type(module, ty))
                        .collect();
                    self.instantiate_record(def, types, file, annotation.span)
                        .unwrap_or(Ty::Unknown)
                }
            }
            TypeKind::Array { element, length } => {
                let element = self.resolve_type(module, element);
                let file = self.package.module(module).file;
                let Some(length) = self.eval_i32(module, file, length, Some(Ty::Number), None)
                else {
                    return Ty::Unknown;
                };
                if !(0..=1024).contains(&length) {
                    self.error(
                        "type",
                        file,
                        annotation.span,
                        "array length must be between 0 and 1024",
                    );
                    return Ty::Unknown;
                }
                self.array_type(module, element, length as usize)
            }
        }
    }

    fn resolve_named_type(&mut self, module: ModuleId, ty: &Path) -> Ty {
        if let [segment] = ty.segments.as_slice() {
            if !ty.arguments().is_empty()
                && (self.generics.bindings.contains_key(&segment.name)
                    || matches!(
                        segment.name.as_str(),
                        "number" | "float" | "bool" | "string" | "EntityRef"
                    ))
            {
                self.error(
                    "generic",
                    self.package.module(module).file,
                    ty.span,
                    "this type does not take type arguments",
                );
                return Ty::Unknown;
            }
            if let Some((resolved, parameter)) = self.generics.bindings.get(&segment.name).copied()
            {
                self.references.push(Reference {
                    file: self.package.module(module).file,
                    span: segment.span,
                    def: parameter,
                });
                return resolved;
            }
            match segment.name.as_str() {
                "number" => return Ty::Number,
                "float" => return Ty::Float,
                "bool" => return Ty::Bool,
                "EntityRef" => return Ty::EntityRef,
                "string" => return Ty::String,
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
                "timer" => {
                    let file = self.package.module(module).file;
                    self.diagnostics.push(
                        Diagnostic::error(
                            "type",
                            file,
                            segment.span,
                            "`timer` is only for `state` fields",
                        )
                        .with_help("use `number` to hold a duration here"),
                    );
                    return Ty::Unknown;
                }
                _ => {}
            }
        }
        match self.expect_path(
            module,
            ty,
            &[DefKind::Enum, DefKind::Struct],
            "a type (`number`, `float`, `bool`, `string`, `EntityRef`, enum, or struct)",
        ) {
            Some(def) if self.defs[def.0].kind == DefKind::Struct => {
                if self.generics.records.contains_key(&def) || !ty.arguments().is_empty() {
                    let types = ty
                        .arguments()
                        .iter()
                        .map(|ty| self.resolve_type(module, ty))
                        .collect();
                    self.instantiate_record(def, types, self.package.module(module).file, ty.span)
                        .unwrap_or(Ty::Unknown)
                } else {
                    Ty::Struct(def)
                }
            }
            Some(def) => Ty::Enum(def),
            None => Ty::Unknown,
        }
    }

    fn ty_name(&self, ty: Ty) -> String {
        match ty {
            Ty::Number => "number".into(),
            Ty::Float => "float".into(),
            Ty::Bool => "bool".into(),
            Ty::Enum(def) | Ty::Struct(def) | Ty::Callable(def) => self.defs[def.0].name.clone(),
            Ty::EntityRef => "EntityRef".into(),
            Ty::String => "string".into(),
            Ty::List(def) => self.defs[def.0].name.clone(),
            Ty::Unknown => "{unknown}".into(),
        }
    }

    fn declare_constant_types(&mut self) {
        // Scalar annotations precede composite lengths, including forward
        // references, so a bool constant cannot silently become an array size.
        for named in [true, false] {
            for index in 0..self.package.modules.len() {
                let module = ModuleId(index);
                for item in self.package.module(module).items.clone() {
                    let ItemKind::Const(c) = item.kind else {
                        continue;
                    };
                    if matches!(c.ty.kind, TypeKind::Named(_)) != named {
                        continue;
                    }
                    let Some(def) = self.def_by_span(module, c.name.span) else {
                        continue;
                    };
                    let ty = self.resolve_type(module, &c.ty);
                    if let Some(source) = self.const_src.get_mut(&def) {
                        source.ty = ty;
                    }
                }
            }
        }
    }

    fn evaluate_declarations(&mut self) {
        let consts: Vec<DefId> = self.const_src.keys().copied().collect();
        for def in consts {
            let ty = self.const_src[&def].ty;
            if ty == Ty::String {
                let source = self.const_src[&def].clone();
                if let Some(value) =
                    self.constant_text(source.module, source.file, &source.expr, &mut vec![def])
                {
                    self.text_constants.insert(def, value);
                }
                self.defs[def.0].info = DefInfo::Const { ty, value: None };
                continue;
            }
            let value = self.eval_def(def);
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
        let timer = matches!(&field.ty.kind, TypeKind::Named(path) if matches!(path.segments.as_slice(), [s] if s.name == "timer"));
        let ty = if timer {
            Ty::Number
        } else {
            self.resolve_type(module, &field.ty)
        };
        if field.name.name.starts_with("__sand_record_")
            || field.name.name.starts_with("__sand_data_")
        {
            self.error(
                "field",
                file,
                field.name.span,
                "`__sand_record_` is reserved for struct storage",
            );
        }
        let default = if matches!(
            ty,
            Ty::Struct(_) | Ty::String | Ty::List(_) | Ty::Callable(_)
        ) {
            if let Some(defaults) = self.record_constant(module, file, field.default.as_ref(), ty) {
                self.record_defaults.insert(def, defaults);
            }
            None
        } else {
            match (&field.default, ty) {
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
            }
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
            if !matches!(ty, Ty::Number | Ty::Float) || timer {
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
            timer,
        };
    }

    fn def_by_span(&self, module: ModuleId, span: Span) -> Option<DefId> {
        let file = self.package.module(module).file;
        self.defs
            .iter()
            .enumerate()
            .find(|(index, d)| {
                d.file == file
                    && d.name_span == span
                    && !self.generics.origins.contains_key(&DefId(*index))
            })
            .map(|(index, _)| DefId(index))
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
                    BinOp::Mul if float => a.checked_mul(b).and_then(|v| floor_div(v, FLOAT_SCALE)),
                    BinOp::Mul => a.checked_mul(b),
                    BinOp::Div | BinOp::Rem if b == 0 => {
                        self.error("const", file, rhs.span(), "division by zero");
                        return None;
                    }
                    BinOp::Div if float => a.checked_mul(FLOAT_SCALE).and_then(|v| floor_div(v, b)),
                    BinOp::Div => floor_div(a, b),
                    BinOp::Rem => floor_rem(a, b),
                };
                match result {
                    Some(value) => Some((value, if float { Ty::Float } else { Ty::Number })),
                    None => {
                        self.error("range", file, *span, "constant arithmetic overflows");
                        None
                    }
                }
            }
            Expr::Closure(closure) => {
                self.error("const", file, closure.span, "a closure is a runtime value");
                None
            }
            Expr::Struct { span, .. }
            | Expr::Method { span, .. }
            | Expr::Project(_, _, span)
            | Expr::Array(_, span)
            | Expr::Match { span, .. }
            | Expr::Block { span, .. }
            | Expr::ArrayRepeat { span, .. }
            | Expr::Index(_, _, span) => {
                self.error(
                    "const",
                    file,
                    *span,
                    "expected a scalar constant expression",
                );
                None
            }
            Expr::Cast(inner, ty, span) => {
                let (value, from) = self.eval(module, file, inner, bare)?;
                let to = self.cast_target(module, file, ty)?;
                let step = match self.cast_step(from, to) {
                    Ok(step) => step,
                    Err((message, help)) => {
                        self.diagnostics
                            .push(Diagnostic::error("type", file, *span, message).with_help(help));
                        return None;
                    }
                };
                let value = match step {
                    CastStep::Same => value,
                    CastStep::ToFloat => value * FLOAT_SCALE,
                    CastStep::ToNumber => value.div_euclid(FLOAT_SCALE),
                    CastStep::ToBool => i64::from(value != 0),
                };
                if let Ty::Enum(def) = to
                    && from != to
                {
                    let variants = match &self.defs[def.0].info {
                        DefInfo::Enum { variants } => variants.clone(),
                        _ => Vec::new(),
                    };
                    if !variants.iter().any(|v| self.eval_def(*v) == Some(value)) {
                        let name = self.defs[def.0].name.clone();
                        self.diagnostics.push(
                            Diagnostic::error(
                                "range",
                                file,
                                *span,
                                format!("no `{name}` variant has the value {value}"),
                            )
                            .with_help(format!(
                                "`{name}` variants are numbered from 0 in declaration order unless given a value"
                            )),
                        );
                        return None;
                    }
                }
                Some((value, to))
            }
        }
    }

    /// The type named after `as`, reporting unknown names.
    pub(super) fn cast_target(&mut self, module: ModuleId, file: FileId, ty: &Type) -> Option<Ty> {
        let _ = file;
        match self.resolve_type(module, ty) {
            Ty::Unknown => None,
            ty => Some(ty),
        }
    }

    /// The type named after `as`, resolved without diagnostics.
    pub(super) fn cast_target_quiet(&mut self, module: ModuleId, annotation: &Type) -> Ty {
        let TypeKind::Named(ty) = &annotation.kind else {
            return Ty::Unknown;
        };
        if let [segment] = ty.segments.as_slice() {
            if let Some((ty, _)) = self.generics.bindings.get(&segment.name) {
                return *ty;
            }
            match segment.name.as_str() {
                "number" => return Ty::Number,
                "float" => return Ty::Float,
                "bool" => return Ty::Bool,
                "EntityRef" => return Ty::EntityRef,
                "string" => return Ty::String,
                _ => {}
            }
        }
        match self.resolve_path(module, ty) {
            Ok((def, _)) if self.defs[def.0].kind == DefKind::Enum => Ty::Enum(def),
            _ => Ty::Unknown,
        }
    }

    /// How `value as T` converts a `from` value into `to`, or why it cannot.
    ///
    /// | from \\ to | number | float | bool | enum |
    /// |---|---|---|---|---|
    /// | number | same | ×1000 | `!= 0` | same value |
    /// | float | rounds down | same | `!= 0` | — |
    /// | bool | 0 or 1 | 0.0 or 1.0 | same | — |
    /// | enum | its value | its value | — | same enum only |
    pub(super) fn cast_step(&self, from: Ty, to: Ty) -> Result<CastStep, (String, &'static str)> {
        if matches!(
            from,
            Ty::Struct(_) | Ty::EntityRef | Ty::String | Ty::List(_) | Ty::Callable(_)
        ) || matches!(
            to,
            Ty::Struct(_) | Ty::EntityRef | Ty::String | Ty::List(_) | Ty::Callable(_)
        ) {
            return Err((
                "struct, string and entity-reference values cannot be cast".into(),
                "format scalars with `.to_string()`, use scalar fields, or resolve the entity reference",
            ));
        }
        Ok(match (from, to) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => CastStep::Same,
            (a, b) if a == b => CastStep::Same,
            (Ty::Number | Ty::Bool | Ty::Enum(_), Ty::Number) => CastStep::Same,
            (Ty::Float, Ty::Number) => CastStep::ToNumber,
            (Ty::Number | Ty::Bool | Ty::Enum(_), Ty::Float) => CastStep::ToFloat,
            (Ty::Number | Ty::Float, Ty::Bool) => CastStep::ToBool,
            (Ty::Number, Ty::Enum(_)) => CastStep::Same,
            (Ty::Enum(_), Ty::Bool) => {
                return Err((
                    format!("cannot cast `{}` to `bool`", self.ty_name(from)),
                    "compare the variant instead, such as `phase == Phase::Busy`",
                ));
            }
            (Ty::Bool, Ty::Enum(_)) => {
                return Err((
                    format!("cannot cast `bool` to `{}`", self.ty_name(to)),
                    "branch on the flag instead: `if flag { ... }`",
                ));
            }
            (Ty::Float, Ty::Enum(_)) => {
                return Err((
                    format!("cannot cast `float` to `{}`", self.ty_name(to)),
                    "convert to a whole number first: `(value as number) as Phase`",
                ));
            }
            (Ty::Enum(_), Ty::Enum(_)) => {
                return Err((
                    format!(
                        "cannot cast `{}` to `{}`",
                        self.ty_name(from),
                        self.ty_name(to)
                    ),
                    "go through its value: `value as number as Other`",
                ));
            }
            _ => CastStep::Same,
        })
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
        self.lower_generic_instances(&mut modules, &mut lowerer);
        self.lower_callable_dispatchers(&mut lowerer);
        self.infer_contexts(&mut lowerer);
        self.validate_callable_contexts();
        self.discover_tests(&lowerer);
        self.resolve_frames(&mut lowerer);
        if (!self.local_fields.is_empty() || !self.local_data_fields.is_empty())
            && let Some(root) = modules.first_mut()
            && let Ok(id) = self.locals_state_id().parse()
        {
            // One compiler-owned global State holds every local, parameter,
            // return value, and temporary slot.
            root.states.push(pc::State {
                data_fields: self
                    .local_data_fields
                    .iter()
                    .map(|(name, default)| pc::DataField {
                        name: name.clone(),
                        default: default.clone(),
                    })
                    .collect(),
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
                        timer: false,
                    })
                    .collect(),
            });
        }
        // Contexts are known; finalise function declarations.
        for (index, module) in modules.iter_mut().enumerate() {
            for pending in lowerer.functions.iter().filter(|f| f.module.0 == index) {
                if !self.test_mode
                    && pending
                        .def
                        .is_some_and(|def| self.test_functions.contains(&def))
                {
                    continue;
                }
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
                let mut function = pc::Function {
                    id,
                    context: match context {
                        Context::Server => pc::ExecutionContext::Server,
                        Context::Player => pc::ExecutionContext::Player,
                        Context::Entity => pc::ExecutionContext::Entity,
                    },
                    body: pending.body.clone(),
                };
                let joins = continuations::join_wait_paths(&mut function);
                module.functions.push(function);
                for join in joins {
                    let ptr = format!("/functions/{}", module.functions.len());
                    self.pointers.push((
                        module.id.to_string(),
                        ptr,
                        pending.file,
                        pending.name_span,
                    ));
                    module.functions.push(join);
                }
            }
        }
        if self.test_mode {
            for (index, (def, test)) in self.tests.iter().enumerate() {
                let module = self.defs[def.0].module.0;
                if let Some(function) = self.test_wrapper(*def, test, index) {
                    let ptr = format!("/functions/{}", modules[module].functions.len());
                    self.pointers
                        .push((modules[module].id.to_string(), ptr, test.file, test.span));
                    modules[module].functions.push(function);
                    lowerer.raw = true;
                }
            }
        }
        let mut requires = Vec::new();
        if lowerer.raw {
            requires.push("raw_commands".to_string());
        }
        let structure_assets = modules
            .iter()
            .flat_map(|module| &module.structure_templates)
            .map(|template| template.source_path())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|path| {
                let bytes = self.package.structure_assets.get(path)?.as_ref().ok()?;
                Some((path.to_owned(), bytes.clone()))
            })
            .collect();
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
            structure_assets,
        })
    }

    fn locals_state_id(&self) -> String {
        format!("{}:sand.locals", self.namespace())
    }

    /// Resolve every function's parameters and return type before bodies are
    /// lowered, so calls can target functions declared later.
    fn declare_signatures(&mut self) {
        self.declare_trait_defaults();
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            for item in self.package.module(module).items.clone() {
                match &item.kind {
                    ItemKind::Fn(function) if !function.generics.is_empty() => {
                        if let Some(def) = self.def_by_span(module, function.name.span) {
                            self.register_generic(def, &item, function);
                        }
                    }
                    ItemKind::Fn(function) => {
                        if let Some(receiver) = &function.receiver {
                            self.error(
                                "receiver",
                                file,
                                receiver.span,
                                "`self` parameters belong inside an impl block",
                            );
                        }
                        if let Some(def) = self.def_by_span(module, function.name.span) {
                            self.declare_slots(
                                module,
                                file,
                                def,
                                &function.params,
                                function.ret.as_ref(),
                            );
                            self.declare_test(def, &item, function);
                        }
                    }
                    ItemKind::Event(event) => {
                        if let Some(def) = self.def_by_span(module, event.name.span) {
                            self.declare_slots(module, file, def, &event.params, None);
                        }
                    }
                    _ => {}
                }
            }
        }
        self.collect_event_handlers();
    }

    /// Parameters and return slot of one declared function or event.
    fn declare_slots(
        &mut self,
        module: ModuleId,
        file: FileId,
        owner: DefId,
        declared: &[Param],
        ret: Option<&Type>,
    ) {
        let mut params = Vec::new();
        let mut seen = HashSet::new();
        let mut rendered = Vec::new();
        for param in declared {
            if !seen.insert(param.name.name.clone()) {
                self.error(
                    "duplicate",
                    file,
                    param.name.span,
                    format!("parameter `{}` is declared more than once", param.name.name),
                );
            }
            let ty = self.resolve_type(module, &param.ty);
            if param.name.name == "self"
                && let Some(receiver) = self
                    .methods
                    .get(&owner)
                    .and_then(|method| method.receiver.as_ref())
                && receiver.borrowed
            {
                rendered.push(format!(
                    "&{}self",
                    if receiver.mutable { "mut " } else { "" }
                ));
            } else {
                rendered.push(format!(
                    "{}{}: {}",
                    if param.mutable { "mut " } else { "" },
                    param.name.name,
                    self.ty_name(ty)
                ));
            }
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
                    mutable: param.mutable,
                    param: true,
                    owner: Some(owner),
                },
            });
            if matches!(ty, Ty::Struct(_)) {
                let base = self.local_ref(local);
                self.value_slots(&base, ty);
            }
            params.push((format!("v{}_{}", local.0, param.name.name), local, ty));
        }
        let ret = ret.map(|ret| {
            let ty = self.resolve_type(module, ret);
            let name = format!("f{}_ret", owner.0);
            if matches!(ty, Ty::Struct(_)) {
                let base = self.slot(name.clone());
                self.value_slots(&base, ty);
            }
            (name, ty)
        });
        let mut signature = format!("({})", rendered.join(", "));
        if let Some((_, ty)) = &ret {
            signature.push_str(&format!(" -> {}", self.ty_name(*ty)));
        }
        if let DefInfo::Function { signature: s, .. } = &mut self.defs[owner.0].info {
            *s = signature;
        }
        let receiver_out = self
            .methods
            .get(&owner)
            .filter(|method| {
                !method.state_receiver
                    && method
                        .receiver
                        .as_ref()
                        .is_some_and(|r| r.borrowed && r.mutable)
            })
            .and_then(|_| params.first())
            .map(|(_, _, ty)| (format!("f{}_receiver_out", owner.0), *ty));
        if let Some((field, ty)) = &receiver_out {
            let base = self.slot(field.clone());
            self.value_slots(&base, *ty);
        }
        self.fn_slots.insert(
            owner,
            FnSlots {
                params,
                ret,
                receiver_out,
            },
        );
    }

    /// Bind `#[on(Event)]` functions to their custom events.
    fn collect_event_handlers(&mut self) {
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            for item in self.package.module(module).items.clone() {
                let ItemKind::Fn(function) = &item.kind else {
                    continue;
                };
                if !function.generics.is_empty() {
                    continue;
                }
                let Some(def) = self.def_by_span(module, function.name.span) else {
                    continue;
                };
                for attr in item.attrs.iter().filter(|a| a.name() == "on") {
                    let [
                        AttrArg {
                            key: None,
                            value: AttrValue::Expr(Expr::Path(path)),
                            ..
                        },
                    ] = attr.args.as_slice()
                    else {
                        continue;
                    };
                    if path.segments.len() == 1
                        && crate::builtins::EVENTS
                            .iter()
                            .any(|(name, _, _)| *name == path.segments[0].name)
                    {
                        continue;
                    }
                    let Ok((event, refs)) = self.resolve_path(module, path) else {
                        continue;
                    };
                    if self.defs[event.0].kind != DefKind::Event {
                        continue;
                    }
                    self.references.extend(refs);
                    self.handler_attrs.insert((file, attr.span));
                    let expected: Vec<Ty> =
                        self.fn_slots[&event].params.iter().map(|p| p.2).collect();
                    let found: Vec<Ty> = self.fn_slots[&def].params.iter().map(|p| p.2).collect();
                    // A handler takes the whole payload, or ignores it.
                    if expected != found && !found.is_empty() {
                        let name = self.defs[event.0].name.clone();
                        let signature = match &self.defs[event.0].info {
                            DefInfo::Function { signature, .. } => signature.clone(),
                            _ => String::new(),
                        };
                        let (event_file, event_span) =
                            (self.defs[event.0].file, self.defs[event.0].name_span);
                        self.diagnostics.push(
                            Diagnostic::error(
                                "params",
                                file,
                                function.name.span,
                                format!(
                                    "`{}` handles `{name}`, so it must take {signature} or no parameters",
                                    function.name.name
                                ),
                            )
                            .with_note(
                                event_file,
                                event_span,
                                "event declared here",
                            ),
                        );
                        continue;
                    }
                    self.event_handlers
                        .entry(event)
                        .or_default()
                        .push((def, function.name.span));
                }
            }
        }
    }

    /// The dispatcher of a custom event: copy the payload into each handler's
    /// parameters, then call it, in declaration order.
    fn lower_event(
        &mut self,
        module: ModuleId,
        file: FileId,
        event: &crate::ast::EventItem,
        def: DefId,
        lowerer: &mut Lowering,
    ) {
        let DefInfo::Function { id, .. } = self.defs[def.0].info.clone() else {
            return;
        };
        let slots = self.fn_slots[&def].clone();
        let mut body = Vec::new();
        let mut calls = Vec::new();
        for (handler, span) in self.event_handlers.get(&def).cloned().unwrap_or_default() {
            let handler_slots = self.fn_slots[&handler].clone();
            // A handler that emits this event again re-enters the dispatcher.
            let marker = self.frame_marker(Some(def), handler);
            body.push(op_at(
                pc::Action::PushFrame {
                    fields: vec![marker.clone()],
                },
                file,
                span,
            ));
            for ((from, _, ty), (to, _, _)) in slots.params.iter().zip(&handler_slots.params) {
                let from = self.slot(from.clone());
                let to = self.slot(to.clone());
                self.copy_value(&to, &from, *ty, file, span, &mut body);
            }
            if let Some(function) = self.function_reference(handler) {
                body.push(op_at(pc::Action::Call { function }, file, span));
            }
            body.push(op_at(
                pc::Action::PopFrame {
                    fields: vec![marker],
                },
                file,
                span,
            ));
            calls.push((handler, span));
        }
        lowerer.facts.insert(
            def,
            Facts {
                entity_direct: None,
                entity_loop_calls: Vec::new(),
                all_calls: calls.clone(),
                schedules: Vec::new(),
                forced: None,
                direct: None,
                calls,
                entry: Vec::new(),
                file,
            },
        );
        lowerer.functions.push(PendingFn {
            module,
            file,
            def: Some(def),
            fixed: None,
            id,
            name_span: event.name.span,
            body,
        });
    }

    /// Resolve every call's frame placeholder now that the call graph is
    /// known.
    ///
    /// Locals live in shared slots, so a call that can lead back into its
    /// caller (directly or through other functions) saves the caller's
    /// locals, parameters, and temporaries on a storage stack and restores
    /// them afterwards. Return slots are left out: they carry the callee's
    /// result out across the restore. Calls that cannot re-enter their caller
    /// drop the placeholders and cost nothing.
    fn resolve_frames(&mut self, lowerer: &mut Lowering) {
        let graph: BTreeMap<DefId, Vec<DefId>> = lowerer
            .facts
            .iter()
            .map(|(def, facts)| (*def, facts.all_calls.iter().map(|(d, _)| *d).collect()))
            .collect();
        let reaches = |from: DefId, to: DefId| {
            let mut stack = vec![from];
            let mut seen = HashSet::new();
            while let Some(next) = stack.pop() {
                if next == to {
                    return true;
                }
                if seen.insert(next)
                    && let Some(calls) = graph.get(&next)
                {
                    stack.extend(calls.iter().copied());
                }
            }
            false
        };
        let mut frames: BTreeMap<DefId, Vec<String>> = BTreeMap::new();
        let resolved: Vec<Option<Vec<pc::FieldReference>>> = self
            .call_frames
            .clone()
            .into_iter()
            .map(|(caller, callee)| {
                let caller = caller?;
                if !reaches(callee, caller) {
                    return None;
                }
                let fields = frames
                    .entry(caller)
                    .or_insert_with(|| self.frame_fields(caller))
                    .clone();
                (!fields.is_empty()).then(|| {
                    fields
                        .into_iter()
                        .map(|field| pc::FieldReference {
                            state: self.locals_state_id().parse().expect("namespace validated"),
                            field,
                        })
                        .collect()
                })
            })
            .collect();
        for function in &mut lowerer.functions {
            resolve_frame_markers(&mut function.body, &resolved);
        }
    }

    /// The slots a function's own frame holds: its locals, parameters, and
    /// temporaries, in allocation order.
    fn frame_fields(&self, function: DefId) -> Vec<String> {
        let temps = format!("f{}_t", function.0);
        self.local_fields
            .iter()
            .chain(self.local_data_fields.keys())
            .filter(|name| {
                let name = name.strip_prefix("__sand_data_").unwrap_or(name);
                if name.starts_with(&temps) {
                    return true;
                }
                let Some(local) = name
                    .strip_prefix('v')
                    .and_then(|rest| rest.split_once('_'))
                    .and_then(|(index, _)| index.parse::<usize>().ok())
                else {
                    return false;
                };
                matches!(
                    self.defs.get(local).map(|d| &d.info),
                    Some(DefInfo::Local { owner: Some(owner), .. }) if *owner == function
                )
            })
            .cloned()
            .collect()
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
            loot_tables: Vec::new(),
            boss_bars: Vec::new(),
            damage_types: Vec::new(),
            predicates: Vec::new(),
            advancements: Vec::new(),
            dialogs: Vec::new(),
            dimensions: Vec::new(),
            biomes: Vec::new(),
            structure_templates: Vec::new(),
            inventory_transfers: Vec::new(),
            huds: Vec::new(),
            menus: Vec::new(),
            projectiles: Vec::new(),
            progressions: Vec::new(),
            scenes: Vec::new(),
            jobs: Vec::new(),
            sessions: Vec::new(),
            abilities: Vec::new(),
            combats: Vec::new(),
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
                    let mut data_fields = Vec::new();
                    for (index, field) in fields.iter().enumerate() {
                        let d = self.defs[field.0].clone();
                        self.pointers.push((
                            id_text.clone(),
                            format!("{ptr}/fields/{index}"),
                            file,
                            d.name_span,
                        ));
                        let DefInfo::Field {
                            default,
                            bounds,
                            timer,
                            ..
                        } = d.info
                        else {
                            continue;
                        };
                        if matches!(
                            self.field_ty(*field),
                            Ty::Struct(_) | Ty::String | Ty::List(_) | Ty::Callable(_)
                        ) {
                            let value_ty = self.field_ty(*field);
                            let base = self.field_reference(def, *field).expect("State field");
                            let defaults =
                                self.record_defaults.get(field).cloned().unwrap_or_default();
                            for (index, (leaf, _)) in
                                self.value_slots(&base, value_ty).into_iter().enumerate()
                            {
                                self.pointers.push((
                                    id_text.clone(),
                                    format!("{ptr}/fields/{}", score_fields.len()),
                                    file,
                                    d.name_span,
                                ));
                                match defaults
                                    .get(index)
                                    .cloned()
                                    .unwrap_or(text::LeafDefault::Score(0))
                                {
                                    text::LeafDefault::Score(default) => {
                                        score_fields.push(pc::ScoreField {
                                            name: leaf.field,
                                            default,
                                            bounds: None,
                                            timer: false,
                                        })
                                    }
                                    text::LeafDefault::Data(default) => {
                                        data_fields.push(pc::DataField {
                                            name: Self::data_field(&leaf).field,
                                            default,
                                        })
                                    }
                                }
                            }
                            continue;
                        }
                        score_fields.push(pc::ScoreField {
                            name: d.name.clone(),
                            default: default.unwrap_or(0) as i32,
                            bounds,
                            timer,
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
                        data_fields,
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
                ItemKind::Fn(function) if function.generics.is_empty() => {
                    self.lower_fn(module, file, &item, function, def, &mut out, lowerer);
                }
                ItemKind::Fn(_) => {}
                ItemKind::Event(event) => {
                    self.reject_attrs(file, &item);
                    self.lower_event(module, file, event, def, lowerer);
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
                ItemKind::Use(_)
                | ItemKind::Struct(_)
                | ItemKind::Trait(_)
                | ItemKind::Impl(_)
                | ItemKind::Error(_) => {}
            }
        }
        out
    }

    /// Parse a bare player event, a vanilla filter, or an authored advancement.
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
                    "advancement" => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "attribute",
                                file,
                                arg.span,
                                "expected an authored advancement",
                            )
                            .with_help("write `#[on(advancement = Milestone)]`"),
                        );
                        return None;
                    }
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
            (Some(key), value) if key.name == "advancement" => {
                let AttrValue::Expr(expr @ Expr::Path(path)) = value else {
                    self.error(
                        "attribute",
                        file,
                        arg.span,
                        "expected an authored advancement",
                    );
                    return None;
                };
                let advancement = self.native_data_id(module, file, expr, DefKind::Advancement)?;
                let label = format!("advancement = {}", path.display());
                Some((pc::PlayerEvent::AdvancementCompleted { advancement }, label))
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
            if name.starts_with('_') || span == Span::default() {
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
        let target = self
            .pack
            .as_ref()
            .map(|pack| pack.minecraft.clone())
            .unwrap_or_else(|| sand_version::LATEST_KNOWN.into());
        let table = crate::vanilla::for_target(&target, registry);
        let noun = table.map(|t| t.noun).unwrap_or("value");
        let declaration = match registry {
            "Item" => Some(DefKind::CustomItem),
            "DamageType" => Some(DefKind::DamageType),
            "Dimension" => Some(DefKind::Dimension),
            "Biome" => Some(DefKind::Biome),
            _ => None,
        };
        if let Some(declaration) = declaration
            && let Some((_, id)) = self.declared(module, file, expr, declaration)
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
                        if crate::vanilla::registry(registry)
                            .is_some_and(|table| table.id(&variant.name).is_some())
                        {
                            format!(
                                "`{registry}::{}` is not available in Minecraft {target}",
                                variant.name
                            )
                        } else {
                            format!("no {noun} named `{}` in `{registry}`", variant.name)
                        },
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
                                if crate::vanilla::registry(registry)
                                    .is_some_and(|table| table.by_id.contains_key(value))
                                {
                                    format!("Minecraft {target} has no {noun} `{value}`")
                                } else {
                                    format!("Minecraft has no {noun} `{value}`")
                                },
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
        if self.generics.origins.contains_key(&def) && item.attr("test").is_some() {
            self.error(
                "test",
                file,
                function.name.span,
                "a test cannot be a specialized method",
            );
        }
        let id_text = out.id.to_string();
        let reference = || pc::FunctionReference::Internal {
            id: id.parse().expect("validated below"),
        };
        let valid_id = id.parse::<sand::ResourceLocation>().is_ok();
        let mut forced = None;
        let mut entry = Vec::new();
        for attr in &item.attrs {
            match attr.name() {
                "test" => {}
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
                "on" if self.handler_attrs.contains(&(file, attr.span)) => {}
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
                        "unknown function attribute `{other}`; expected `tick`, `load`, `tag`, `on`, `player`, or `test`"
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
        if let Some(method) = self.methods.get(&def)
            && let Some(receiver) = &method.receiver
            && method.state_receiver
        {
            let reason = format!(
                "uses a receiver for State `{}`",
                self.defs[method.target.0].name
            );
            match self.defs[method.target.0].info {
                DefInfo::State {
                    scope: Scope::Player,
                    ..
                } => body.note_player(receiver.span, reason),
                DefInfo::State {
                    scope: Scope::Entity,
                    ..
                } => body.note_executor(receiver.span, reason),
                _ => {}
            }
        }
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
        if let Some(slots) = &slots
            && let Some((output, ty)) = &slots.receiver_out
            && let Some((_, receiver, _)) = slots.params.first()
        {
            body.receiver_out = Some((self.local_ref(*receiver), self.slot(output.clone()), *ty));
        }
        let mut ops = self.lower_block(&mut body, &function.body);
        self.finish_receiver(&body, function.body.span, &mut ops);
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
                        "`{}` must return a `{ty}` on every path",
                        function.name.name
                    ),
                )
                .with_help("end the function with `return <value>;`"),
            );
        }
        self.local_lints(&body);
        lowerer.raw |= body.raw && (self.test_mode || !self.test_functions.contains(&def));
        lowerer.facts.insert(
            def,
            Facts {
                entity_direct: body.entity_use.clone(),
                entity_loop_calls: body.entity_loop_calls.clone(),
                all_calls: body.all_calls.clone(),
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
                        if self.test_functions.contains(&target) {
                            self.error(
                                "test",
                                file,
                                path.span,
                                "a test function cannot be a flow hook",
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
                    for (target, span) in body.all_calls.iter().chain(&body.schedules) {
                        if self.test_functions.contains(target) {
                            self.error("test", file, *span, "a flow cannot call a test function");
                        }
                    }
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
            field: if matches!(
                self.defs[field.0].info,
                DefInfo::Field {
                    ty: Ty::Struct(_),
                    ..
                }
            ) {
                format!(
                    "__sand_record_{}_{}",
                    self.defs[field.0].name.len(),
                    self.defs[field.0].name
                )
            } else {
                self.defs[field.0].name.clone()
            },
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
        self.lower_block_with_tail(body, block, None)
    }

    fn lower_block_with_tail(
        &mut self,
        body: &mut BodyCx,
        block: &Block,
        tail: Option<(&pc::FieldReference, Ty, &Expr)>,
    ) -> Vec<pc::Operation> {
        enum Segment {
            Actions(Vec<pc::Operation>, Option<pc::FieldReference>),
            Wait(continuations::WaitSite),
        }
        let mut segments = Vec::new();
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
            if let Stmt::Call { path, args, span } = stmt
                && self.is_wait(body, path)
            {
                if let Some(wait) = self.lower_wait(body, path, args, *span) {
                    segments.push(Segment::Wait(wait));
                }
                continue;
            }
            let mut lowered = Vec::new();
            self.lower_stmt(body, stmt, &mut lowered);
            exited = stmt_exits(stmt);
            // After a nested `break` or `continue`, the rest of this pass
            // runs only while the jump flag is clear.
            let flag = body
                .jumps
                .last()
                .filter(|jump| jump.executor_loops == body.loops.len())
                .and_then(|jump| jump.flag.clone());
            let guard = (!exited
                && (statements::jumps_here(stmt, true) || statements::jumps_here(stmt, false)))
            .then_some(flag)
            .flatten();
            segments.push(Segment::Actions(lowered, guard));
        }
        let mut ops = Vec::new();
        if let Some((dest, ty, tail)) = tail {
            self.emit(body, dest, ty, tail, &mut ops);
        }
        for segment in segments.into_iter().rev() {
            let (lowered, guard) = match segment {
                Segment::Actions(lowered, guard) => (lowered, guard),
                Segment::Wait(wait) => {
                    ops = wait.suspend(body.file, ops);
                    continue;
                }
            };
            if let Some(flag) = guard
                && !ops.is_empty()
            {
                let origin = ops.first().and_then(|op: &pc::Operation| op.origin.clone());
                let rest = std::mem::take(&mut ops);
                ops.push(pc::Operation {
                    action: pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score: flag,
                            min: Some(0),
                            max: Some(0),
                        }),
                        then: rest,
                        otherwise: Vec::new(),
                    },
                    origin,
                });
            }
            let mut segment = lowered;
            segment.append(&mut ops);
            ops = segment;
        }
        body.block_ends.pop();
        body.scopes.pop();
        self.diagnostics.append(&mut body.pending);
        ops
    }

    fn lower_stmt(&mut self, body: &mut BodyCx, stmt: &Stmt, ops: &mut Vec<pc::Operation>) {
        let file = body.file;
        match stmt {
            Stmt::Expr(value, span) => self.lower_value_statement(body, value, *span, ops),
            Stmt::Assign {
                target,
                op,
                value,
                span,
            } => match target {
                AssignTarget::Place(place) if self.is_handle(body, place) => {
                    self.lower_property_assign(body, place, *op, value, *span, ops)
                }
                _ => self.lower_assign(body, target, *op, value, *span, ops),
            },
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
            Stmt::Destructure {
                pattern,
                ty,
                value,
                span,
            } => self.lower_destructure(body, pattern, ty.as_ref(), value, *span, ops),
            Stmt::Return { value, span } => self.lower_return(body, value.as_ref(), *span, ops),
            Stmt::Match { value, arms, span } => self.lower_match(body, value, arms, *span, ops),
            Stmt::Continue(span) => self.lower_jump(body, false, *span, ops),
            Stmt::Break(span) => self.lower_jump(body, true, *span, ops),
            Stmt::Emit { event, args, span } => self.lower_emit(body, event, args, *span, ops),
            Stmt::While {
                cond,
                body: block,
                span,
            } => self.lower_repeat(body, statements::Repeat::While(cond), block, *span, ops),
            Stmt::Loop { body: block, span } => {
                self.lower_repeat(body, statements::Repeat::Forever, block, *span, ops)
            }
            Stmt::ForValues {
                binding,
                value,
                body: block,
                span,
            } => {
                let ty = self.type_of(body, value);
                let length = if let Some((_, length)) = self.array_shape(ty) {
                    length
                } else if matches!(ty, Ty::List(_) | Ty::String) {
                    0
                } else {
                    self.error(
                        "for",
                        file,
                        value.span(),
                        "iterate an array, vector, string, player query, or entity query",
                    );
                    return;
                };
                let range = Range {
                    start: Some(Expr::Int(0, value.span())),
                    end: Some(Expr::Int(length as i64, value.span())),
                    inclusive: false,
                    span: value.span(),
                };
                self.lower_repeat(
                    body,
                    statements::Repeat::Count {
                        binding,
                        range: &range,
                        reverse: false,
                        step: None,
                        collection: Some(value),
                    },
                    block,
                    *span,
                    ops,
                );
            }
            Stmt::ForRange {
                binding,
                range,
                reverse,
                step,
                body: block,
                span,
            } => self.lower_repeat(
                body,
                statements::Repeat::Count {
                    binding,
                    range,
                    reverse: *reverse,
                    step: step.as_ref(),
                    collection: None,
                },
                block,
                *span,
                ops,
            ),
            Stmt::Call { path, args, span } => self.lower_call(body, path, args, *span, ops),
            Stmt::Macro { name, args, span } => self.lower_macro(body, name, args, *span, ops),
            Stmt::If(if_stmt) => {
                let lowered = self.lower_if(body, if_stmt);
                ops.extend(lowered);
            }
            Stmt::For {
                binding,
                source,
                args,
                body: block,
                span,
            } => {
                let name = (source.segments.len() == 1).then(|| source.segments[0].name.as_str());
                if matches!(name, Some("resolve_entity" | "resolve_player")) {
                    self.lower_reference_loop(body, stmt, ops);
                    return;
                }
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
                body.loop_bindings.push(binding.name.clone());
                self.handle_scopes.push((
                    file,
                    block.span,
                    binding.name.clone(),
                    kind == LoopKind::Players,
                ));
                if kind == LoopKind::Players {
                    body.player_loops += 1;
                }
                let inner = self.lower_block(body, block);
                if kind == LoopKind::Players {
                    body.player_loops -= 1;
                }
                body.loops.pop();
                body.loop_bindings.pop();
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
        let then = self.lower_waitable_block(body, &stmt.then);
        let otherwise = match &stmt.otherwise {
            None => Vec::new(),
            Some(Else::Block(block)) => self.lower_waitable_block(body, block),
            Some(Else::If(nested)) => self.lower_if(body, nested),
        };
        self.lower_branch_condition(body, &stmt.cond, then, otherwise, stmt.span)
            .unwrap_or_default()
    }

    fn lower_branch_condition(
        &mut self,
        body: &mut BodyCx,
        condition: &Cond,
        then: Vec<pc::Operation>,
        otherwise: Vec<pc::Operation>,
        span: Span,
    ) -> Option<Vec<pc::Operation>> {
        if !continuations::may_suspend(&then) && !continuations::may_suspend(&otherwise) {
            return self.lower_cond(body, condition, then, otherwise);
        }
        // Evaluate short-circuit conditions once and put each suspending arm
        // in a single branch, keeping the number of wait sites linear.
        let depth = body.temp_depth;
        let selected = self.temp(body);
        let test = |value| {
            vec![op_at(
                pc::Action::ScoreSet {
                    score: selected.clone(),
                    value,
                },
                body.file,
                span,
            )]
        };
        let yes = test(1);
        let no = test(0);
        let evaluated = self.lower_cond(body, condition, yes, no);
        body.temp_depth = depth;
        let mut operations = evaluated?;
        operations.push(op_at(
            pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: selected,
                    min: Some(1),
                    max: Some(1),
                }),
                then,
                otherwise,
            },
            body.file,
            span,
        ));
        Some(operations)
    }

    fn lower_waitable_block(&mut self, body: &mut BodyCx, block: &Block) -> Vec<pc::Operation> {
        let saved = body.wait_block_depth;
        let depth = body.block_ends.len();
        if saved == depth {
            body.wait_block_depth = depth + 1;
        }
        let operations = self.lower_block(body, block);
        body.wait_block_depth = saved;
        operations
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
                let name = &path.last().name;
                let builtin = path.segments.len() == 1
                    && !self.scopes[body.module.0].contains_key(name)
                    && Self::lookup_local(body, name).is_none()
                    && crate::builtins::find(name, crate::builtins::BuiltinKind::Condition)
                        .is_some();
                if !builtin {
                    // A function returning `bool`, tested like a variable.
                    let call = Expr::Call {
                        path: path.clone(),
                        args: args.clone(),
                        span: *span,
                    };
                    return self.atom_inner(body, &Cond::Value(call), pre);
                }
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
                if !matches!(ty, Ty::Number | Ty::Float | Ty::Unknown) {
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
                let left_value_ty = self.type_of(body, lhs);
                let right_value_ty = self.type_of(body, rhs);
                if matches!(left_value_ty, Ty::Callable(_))
                    || matches!(right_value_ty, Ty::Callable(_))
                {
                    self.error(
                        "callable",
                        file,
                        *span,
                        "function values cannot be compared directly",
                    );
                    return None;
                }
                if matches!(left_value_ty, Ty::String | Ty::List(_) | Ty::Struct(_))
                    || matches!(right_value_ty, Ty::String | Ty::List(_) | Ty::Struct(_))
                {
                    let value_ty = if matches!(left_value_ty, Ty::List(_)) {
                        left_value_ty
                    } else if matches!(right_value_ty, Ty::List(_)) {
                        right_value_ty
                    } else if matches!(left_value_ty, Ty::Struct(_)) {
                        left_value_ty
                    } else if matches!(right_value_ty, Ty::Struct(_)) {
                        right_value_ty
                    } else {
                        Ty::String
                    };
                    if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
                        self.error(
                            "type",
                            file,
                            *span,
                            "stored values support equality comparisons with `==` or `!=`",
                        );
                        return None;
                    }
                    let left = self.temp(body);
                    self.emit(body, &left, value_ty, lhs, pre)?;
                    let right = self.temp(body);
                    self.emit(body, &right, value_ty, rhs, pre)?;
                    let (left, right) = if matches!(value_ty, Ty::Struct(_)) {
                        let packed_left = self.temp(body);
                        let packed_right = self.temp(body);
                        self.register_data_slot(
                            &packed_left,
                            sand::data::NbtValue::List(Vec::new()),
                        );
                        self.register_data_slot(
                            &packed_right,
                            sand::data::NbtValue::List(Vec::new()),
                        );
                        self.pack_value(
                            body,
                            Self::data_reference(&packed_left),
                            &left,
                            value_ty,
                            *span,
                            pre,
                        );
                        self.pack_value(
                            body,
                            Self::data_reference(&packed_right),
                            &right,
                            value_ty,
                            *span,
                            pre,
                        );
                        (packed_left, packed_right)
                    } else {
                        (left, right)
                    };
                    let score = self.temp(body);
                    pre.push(op_at(
                        pc::Action::DataEqual {
                            score: score.clone(),
                            left: Self::data_reference(&left),
                            right: Self::data_reference(&right),
                        },
                        file,
                        *span,
                    ));
                    return Some((
                        pc::Condition::Range(pc::ScoreComparison {
                            score,
                            min: Some(1),
                            max: Some(1),
                        }),
                        *op == CmpOp::Ne,
                    ));
                }
                let left_ty = self.type_of(body, lhs);
                let ty = if Self::is_const_in(body, lhs) && !matches!(left_ty, Ty::Enum(_)) {
                    self.type_of(body, rhs)
                } else {
                    left_ty
                };
                if matches!(ty, Ty::Enum(_) | Ty::Bool | Ty::EntityRef | Ty::Callable(_))
                    && !matches!(op, CmpOp::Eq | CmpOp::Ne)
                {
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

/// The dispatcher function of an event, given the event's item identity.
fn event_id(item: &str) -> String {
    item.replacen(':', ":sand_event/", 1)
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
        Stmt::Destructure { span, .. }
        | Stmt::Let { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Continue(span)
        | Stmt::Break(span)
        | Stmt::Emit { span, .. }
        | Stmt::Match { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Loop { span, .. }
        | Stmt::ForRange { span, .. }
        | Stmt::ForValues { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Macro { span, .. }
        | Stmt::For { span, .. } => *span,
        Stmt::If(if_stmt) => if_stmt.span,
        Stmt::Expr(_, span) => *span,
        Stmt::Error(error) => error.span,
    }
}

/// Whether control never continues past this statement.
fn stmt_exits(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return { .. } | Stmt::Continue(_) | Stmt::Break(_) => true,
        Stmt::If(if_stmt) => if_exits(if_stmt),
        Stmt::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| arm.body.stmts.iter().any(stmt_exits))
        }
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
        Stmt::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| block_returns(&arm.body))
        }
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
            out.push((
                Path {
                    segments,
                    applications: Vec::new(),
                    span,
                },
                alias,
            ));
        }
    }
}

/// Replace frame placeholders with their resolved fields, or drop them.
fn resolve_frame_markers(
    ops: &mut Vec<pc::Operation>,
    resolved: &[Option<Vec<pc::FieldReference>>],
) {
    let index_of = |fields: &[pc::FieldReference]| {
        fields
            .first()
            .and_then(|field| field.field.strip_prefix(statements::FRAME_MARKER))
            .and_then(|index| index.parse::<usize>().ok())
    };
    ops.retain_mut(|op| match &mut op.action {
        pc::Action::PushFrame { fields } | pc::Action::PopFrame { fields } => {
            match index_of(fields).and_then(|index| resolved.get(index).cloned().flatten()) {
                Some(frame) => {
                    *fields = frame;
                    true
                }
                None => index_of(fields).is_none(),
            }
        }
        pc::Action::Branch {
            then, otherwise, ..
        } => {
            resolve_frame_markers(then, resolved);
            resolve_frame_markers(otherwise, resolved);
            true
        }
        pc::Action::Players { body }
        | pc::Action::Entities { body, .. }
        | pc::Action::WithEntity { body, .. }
        | pc::Action::While { body, .. }
        | pc::Action::InDimension { body, .. }
        | pc::Action::Suspend { body, .. } => {
            resolve_frame_markers(body, resolved);
            true
        }
        _ => true,
    });
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
    schedules: Vec<(DefId, Span)>,
    forced: Option<Span>,
    direct: Option<(Span, String)>,
    calls: Vec<(DefId, Span)>,
    entry: Vec<(String, Span)>,
    file: FileId,
}

/// A `while`, `loop`, or counting `for` that `break` and `continue` target.
#[derive(Clone)]
struct Jump {
    /// 0 during a pass, 1 after `continue`, 2 after `break`; present when
    /// the body jumps.
    flag: Option<pc::FieldReference>,
    /// Executor loops already open when this loop started; a jump inside a
    /// deeper `players()` or `entities(..)` loop targets that loop instead.
    executor_loops: usize,
}

/// The kind of an executor-changing loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopKind {
    Players,
    Entities,
}

#[derive(Clone)]
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
    /// Mutable borrowed receiver output, separate from recursive frame slots.
    receiver_out: Option<(pc::FieldReference, pc::FieldReference, Ty)>,
    /// Nesting depth of `for player in players()` loops.
    player_loops: usize,
    /// Locals read and reassigned, for lints.
    used: HashSet<DefId>,
    assigned: HashSet<DefId>,
    /// Every call, including those inside player loops.
    all_calls: Vec<(DefId, Span)>,
    /// Enclosing executor loops, innermost last.
    loops: Vec<LoopKind>,
    /// The binding of each executor loop, parallel to `loops`.
    loop_bindings: Vec<String>,
    /// Enclosing `while`, `loop`, and counting `for` loops, innermost last.
    jumps: Vec<Jump>,
    /// Where a block expression prevents jumps into surrounding loops.
    value_blocks: Vec<(usize, usize)>,
    /// Calls made as entities inside `entities(..)` loops.
    entity_loop_calls: Vec<(DefId, Span)>,
    /// First use that needs an entity executor outside any loop.
    entity_use: Option<(Span, String)>,
    /// Errors found while noting uses, emitted by the enclosing block.
    pending: Vec<Diagnostic>,
    /// End offsets of enclosing blocks, for local visibility ranges.
    block_ends: Vec<usize>,
    /// The innermost block where a wait may suspend this function frame.
    wait_block_depth: usize,
    /// Compiler State needed to finish dispatch after a resumed arm.
    wait_slots: Vec<pc::FieldReference>,
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
            receiver_out: None,
            player_loops: 0,
            used: HashSet::new(),
            assigned: HashSet::new(),
            all_calls: Vec::new(),
            loops: Vec::new(),
            loop_bindings: Vec::new(),
            jumps: Vec::new(),
            value_blocks: Vec::new(),
            entity_loop_calls: Vec::new(),
            entity_use: None,
            pending: Vec::new(),
            block_ends: Vec::new(),
            wait_block_depth: 1,
            wait_slots: Vec::new(),
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
            && !analysis.origins.contains_key(&DefId(index))
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
