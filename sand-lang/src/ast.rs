//! Span-carrying syntax tree for one Sand source file.
//!
//! The tree mirrors the surface syntax. Every node records the byte [`Span`]
//! it covers so diagnostics, hover, go-to-definition, and rename can map back
//! to source text. Malformed items and statements are kept as
//! [`ItemKind::Error`] and [`Stmt::Error`] so the rest of the file stays usable.

/// A half-open byte range within one source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl Span {
    /// Construct a span from byte offsets.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Whether `offset` lies within the span, inclusive of its end so a cursor
    /// placed directly after an identifier still refers to it.
    pub fn contains(&self, offset: usize) -> bool {
        self.start <= offset && offset <= self.end
    }

    /// The smallest span covering both spans.
    pub fn to(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

/// An identifier together with its source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    /// Identifier text.
    pub name: String,
    /// Location of the identifier.
    pub span: Span,
}

/// A `::`-separated path such as `crate::combat::Phase`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// Path segments, including leading `crate`, `super`, or `self`.
    pub segments: Vec<Ident>,
    /// Location of the whole path.
    pub span: Span,
}

impl Path {
    /// The final segment, which names the referenced item.
    pub fn last(&self) -> &Ident {
        self.segments.last().expect("paths are never empty")
    }

    /// Render the path back to source form.
    pub fn display(&self) -> String {
        self.segments
            .iter()
            .map(|segment| segment.name.as_str())
            .collect::<Vec<_>>()
            .join("::")
    }
}

/// A parsed source file.
#[derive(Debug, Clone, Default)]
pub struct SourceFile {
    /// `//!` documentation lines.
    pub docs: Vec<String>,
    /// `#![...]` attributes.
    pub inner_attrs: Vec<Attr>,
    /// Items in source order.
    pub items: Vec<Item>,
}

/// An `#[name(args)]` or `#![name(args)]` attribute.
#[derive(Debug, Clone)]
pub struct Attr {
    /// Attribute name path.
    pub path: Path,
    /// Arguments in source order.
    pub args: Vec<AttrArg>,
    /// Whole attribute, including `#[` and `]`.
    pub span: Span,
}

impl Attr {
    /// The attribute name, if it is a single identifier.
    pub fn name(&self) -> &str {
        &self.path.last().name
    }
}

/// One attribute argument, optionally `key = value`.
#[derive(Debug, Clone)]
pub struct AttrArg {
    /// Key for `key = value` arguments.
    pub key: Option<Ident>,
    /// Argument value.
    pub value: AttrValue,
    /// Location of the whole argument.
    pub span: Span,
}

/// An attribute argument value.
#[derive(Debug, Clone)]
pub enum AttrValue {
    /// A string literal.
    Str(String, Span),
    /// A constant range such as `0..=100`.
    Range(Range),
    /// A constant expression.
    Expr(Expr),
}

/// A top-level or module-level declaration.
#[derive(Debug, Clone)]
pub struct Item {
    /// `///` documentation lines.
    pub docs: Vec<String>,
    /// Outer attributes.
    pub attrs: Vec<Attr>,
    /// Whether the item is `pub`.
    pub is_pub: bool,
    /// Declaration body.
    pub kind: ItemKind,
    /// Location of the whole item, including docs and attributes.
    pub span: Span,
}

impl Item {
    /// The declared name, when the item declares one.
    pub fn name(&self) -> Option<&Ident> {
        match &self.kind {
            ItemKind::Mod(item) => Some(&item.name),
            ItemKind::Const(item) => Some(&item.name),
            ItemKind::Enum(item) => Some(&item.name),
            ItemKind::State(item) => Some(&item.name),
            ItemKind::Flow(item) => Some(&item.name),
            ItemKind::Fn(item) => Some(&item.name),
            ItemKind::ExternFn(item) => Some(&item.name),
            ItemKind::Data(item) => Some(&item.name),
            ItemKind::Event(item) => Some(&item.name),
            ItemKind::Use(_) | ItemKind::Error(_) => None,
        }
    }

    /// Find an outer attribute by name.
    pub fn attr(&self, name: &str) -> Option<&Attr> {
        self.attrs.iter().find(|attr| attr.name() == name)
    }
}

/// The kinds of item Sand understands.
#[derive(Debug, Clone)]
pub enum ItemKind {
    /// `mod name;` or `mod name { ... }`.
    Mod(ModItem),
    /// `use path;`.
    Use(UseItem),
    /// `const NAME: i32 = expr;`.
    Const(ConstItem),
    /// `enum Name { A, B = 2 }`.
    Enum(EnumItem),
    /// `player state Name { ... }` or `global state Name { ... }`.
    State(StateItem),
    /// `flow Name on State.field { ... }`.
    Flow(FlowItem),
    /// `fn name() { ... }`.
    Fn(FnItem),
    /// `extern "ns:id" fn name();`.
    ExternFn(ExternFnItem),
    /// `item`, `recipe`, a tag, boss bar, or loot table declaration.
    Data(DataItem),
    /// `event Name(params);`.
    Event(EventItem),
    /// Unparseable source, with the strict parser's explanation.
    Error(SyntaxError),
}

/// A syntax error recovered from by skipping source.
#[derive(Debug, Clone)]
pub struct SyntaxError {
    /// Human-readable expectation.
    pub message: String,
    /// The precise failure location.
    pub span: Span,
}

/// A module declaration.
#[derive(Debug, Clone)]
pub struct ModItem {
    /// Module name.
    pub name: Ident,
    /// Inline items for `mod name { ... }`; `None` for a file module.
    pub inline: Option<Vec<Item>>,
}

/// A `use` declaration.
#[derive(Debug, Clone)]
pub struct UseItem {
    /// The import tree.
    pub tree: UseTree,
}

/// One branch of a `use` tree.
#[derive(Debug, Clone)]
pub struct UseTree {
    /// Path prefix.
    pub path: Path,
    /// Nested `{a, b}` group following the prefix.
    pub group: Option<Vec<UseTree>>,
    /// `as name` alias.
    pub alias: Option<Ident>,
}

/// A named compile-time integer constant.
#[derive(Debug, Clone)]
pub struct ConstItem {
    /// Constant name.
    pub name: Ident,
    /// Declared type.
    pub ty: Path,
    /// Value expression.
    pub value: Expr,
}

/// An enumeration whose variants are integer phases.
#[derive(Debug, Clone)]
pub struct EnumItem {
    /// Enum name.
    pub name: Ident,
    /// Variants in declaration order.
    pub variants: Vec<Variant>,
}

/// One enum variant.
#[derive(Debug, Clone)]
pub struct Variant {
    /// Documentation lines.
    pub docs: Vec<String>,
    /// Variant name.
    pub name: Ident,
    /// Explicit discriminant.
    pub value: Option<Expr>,
}

/// Who owns a State's values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// One value per player.
    Player,
    /// One shared value for the server.
    Global,
    /// One value per entity that has it attached.
    Entity,
}

/// A persistent State declaration.
#[derive(Debug, Clone)]
pub struct StateItem {
    /// Ownership scope.
    pub scope: Scope,
    /// Location of the scope keyword.
    pub scope_span: Span,
    /// State name.
    pub name: Ident,
    /// Fields in declaration order.
    pub fields: Vec<Field>,
}

/// One State field.
#[derive(Debug, Clone)]
pub struct Field {
    /// Documentation lines.
    pub docs: Vec<String>,
    /// Field attributes such as `#[clamp(0..=100)]`.
    pub attrs: Vec<Attr>,
    /// Field name.
    pub name: Ident,
    /// Field type: `i32` or an enum.
    pub ty: Path,
    /// Default value.
    pub default: Option<Expr>,
    /// Location of the whole field.
    pub span: Span,
}

/// A state machine over one integer or enum State field.
#[derive(Debug, Clone)]
pub struct FlowItem {
    /// Flow name.
    pub name: Ident,
    /// The driven field.
    pub on: Place,
    /// Transitions in declaration order.
    pub transitions: Vec<Transition>,
    /// Hooks in declaration order.
    pub hooks: Vec<Hook>,
    /// Recovered malformed entries.
    pub errors: Vec<SyntaxError>,
}

/// `From -> To if cond priority n;`.
#[derive(Debug, Clone)]
pub struct Transition {
    /// Source phase.
    pub from: Expr,
    /// Destination phase.
    pub to: Expr,
    /// Optional guard.
    pub guard: Option<Cond>,
    /// Optional priority.
    pub priority: Option<Expr>,
    /// Location of the whole transition.
    pub span: Span,
}

/// The lifecycle point a hook runs at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPhase {
    /// After entering the phase.
    Enter,
    /// Before leaving the phase.
    Exit,
    /// Each cycle while in the phase.
    Tick,
}

/// `enter Phase => function;` or `tick Phase every 2 { ... }`.
#[derive(Debug, Clone)]
pub struct Hook {
    /// Lifecycle point.
    pub phase: HookPhase,
    /// Phase that triggers the hook.
    pub state: Expr,
    /// Tick cadence.
    pub every: Option<Expr>,
    /// Named function or inline body.
    pub target: HookTarget,
    /// Location of the whole hook.
    pub span: Span,
}

/// What a hook runs.
#[derive(Debug, Clone)]
pub enum HookTarget {
    /// A named function.
    Function(Path),
    /// An inline body compiled into a generated function.
    Block(Block),
}

/// `event Name(params);`: a custom event.
#[derive(Debug, Clone)]
pub struct EventItem {
    /// Event name.
    pub name: Ident,
    /// Payload passed to every handler.
    pub params: Vec<Param>,
}

/// The kind of a data resource declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataKind {
    /// `item Name { ... }`: a custom item.
    Item,
    /// `recipe Name { ... }`.
    Recipe,
    /// `tag Name: Item { values: [...] }`.
    Tag,
    /// `bossbar Name { title: "..." }`.
    BossBar,
    /// `loot Name { items: [...] }`.
    Loot,
}

impl DataKind {
    /// Source keyword.
    pub fn keyword(self) -> &'static str {
        match self {
            DataKind::Item => "item",
            DataKind::Recipe => "recipe",
            DataKind::Tag => "tag",
            DataKind::BossBar => "bossbar",
            DataKind::Loot => "loot",
        }
    }
}

/// A data resource declaration such as `item RubySword { ... }`.
#[derive(Debug, Clone)]
pub struct DataItem {
    /// Resource kind.
    pub kind: DataKind,
    /// Declared name.
    pub name: Ident,
    /// The grouped type of a `tag`, such as `Item` in `tag Ores: Item`.
    pub tag_type: Option<Ident>,
    /// Fields in source order.
    pub fields: Vec<DataField>,
    /// The `{ ... }` body.
    pub body: Span,
}

/// One `key: value` field.
#[derive(Debug, Clone)]
pub struct DataField {
    /// Field name (an identifier or a quoted string).
    pub key: Ident,
    /// Field value.
    pub value: DataValue,
    /// Whole field.
    pub span: Span,
}

/// A field value.
#[derive(Debug, Clone)]
pub enum DataValue {
    /// A single expression: a string, number, path, or typed value.
    Expr(Expr),
    /// `[a, b, ...]`.
    Array(Vec<DataValue>, Span),
    /// `{ key: value, ... }`.
    Object(Vec<DataField>, Span),
}

impl DataValue {
    /// Location of the value.
    pub fn span(&self) -> Span {
        match self {
            DataValue::Expr(expr) => expr.span(),
            DataValue::Array(_, span) | DataValue::Object(_, span) => *span,
        }
    }
}

/// A function parameter: `name: Type`.
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name.
    pub name: Ident,
    /// Parameter type.
    pub ty: Path,
    /// Whole parameter.
    pub span: Span,
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct FnItem {
    /// Function name.
    pub name: Ident,
    /// Parameters in order.
    pub params: Vec<Param>,
    /// Declared return type.
    pub ret: Option<Path>,
    /// Body.
    pub body: Block,
}

/// An external function implemented outside this pack.
#[derive(Debug, Clone)]
pub struct ExternFnItem {
    /// Minecraft function identity.
    pub id: String,
    /// Location of the identity string.
    pub id_span: Span,
    /// Local name.
    pub name: Ident,
}

/// A `{ ... }` statement block.
#[derive(Debug, Clone)]
pub struct Block {
    /// Statements in order.
    pub stmts: Vec<Stmt>,
    /// Location including braces.
    pub span: Span,
}

/// A statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let mut name: Type = value;`.
    Let {
        /// Whether the binding is `mut`.
        mutable: bool,
        /// Bound name.
        name: Ident,
        /// Declared type.
        ty: Option<Path>,
        /// Initial value.
        value: Option<Expr>,
        /// Whole statement.
        span: Span,
    },
    /// `return;` or `return value;`.
    Return {
        /// Returned value.
        value: Option<Expr>,
        /// Whole statement.
        span: Span,
    },
    /// `continue;` inside `for player in players()`.
    Continue(Span),
    /// `while condition { ... }`.
    While {
        /// Checked before every pass.
        cond: Cond,
        /// Repeated body.
        body: Block,
        /// Whole statement.
        span: Span,
    },
    /// `target = expr;`, `target += expr;`, and the other compound forms.
    Assign {
        /// Assigned field or local.
        target: AssignTarget,
        /// Operator.
        op: AssignOp,
        /// Constant value.
        value: Expr,
        /// Whole statement.
        span: Span,
    },
    /// `name(args);`.
    Call {
        /// Called function or builtin.
        path: Path,
        /// Arguments in source order.
        args: Vec<Arg>,
        /// Whole statement.
        span: Span,
    },
    /// `name!(args);`, such as `chat!("Hi {player}")` or `raw!("say hi")`.
    Macro {
        /// Macro name.
        name: Ident,
        /// Arguments in source order.
        args: Vec<Arg>,
        /// Whole statement.
        span: Span,
    },
    /// `if cond { } else { }`.
    If(Box<IfStmt>),
    /// `for player in players() { }`.
    For {
        /// Loop binding.
        binding: Ident,
        /// Iterator source.
        source: Path,
        /// Iterator arguments, such as `entities(type = Entity::Zombie)`.
        args: Vec<Arg>,
        /// Loop body.
        body: Block,
        /// Whole statement.
        span: Span,
    },
    /// A recovered malformed statement.
    Error(SyntaxError),
}

/// An `if` statement.
#[derive(Debug, Clone)]
pub struct IfStmt {
    /// Condition.
    pub cond: Cond,
    /// Taken branch.
    pub then: Block,
    /// `else` branch.
    pub otherwise: Option<Else>,
    /// Whole statement.
    pub span: Span,
}

/// The `else` part of an `if`.
#[derive(Debug, Clone)]
pub enum Else {
    /// `else if ...`.
    If(Box<IfStmt>),
    /// `else { ... }`.
    Block(Block),
}

/// The left-hand side of an assignment.
#[derive(Debug, Clone)]
pub enum AssignTarget {
    /// A State field.
    Place(Place),
    /// A local variable or parameter.
    Local(Ident),
}

impl AssignTarget {
    /// Location of the target.
    pub fn span(&self) -> Span {
        match self {
            AssignTarget::Place(place) => place.span,
            AssignTarget::Local(ident) => ident.span,
        }
    }
}

/// Assignment operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    /// `=`.
    Set,
    /// `+=`.
    Add,
    /// `-=`.
    Sub,
    /// `*=`.
    Mul,
    /// `/=`.
    Div,
    /// `%=`.
    Rem,
}

/// `State.field`.
#[derive(Debug, Clone)]
pub struct Place {
    /// State path.
    pub state: Path,
    /// Field name.
    pub field: Ident,
    /// Whole place.
    pub span: Span,
}

/// A boolean condition over State fields.
#[derive(Debug, Clone)]
pub enum Cond {
    /// `a || b`.
    Or(Box<Cond>, Box<Cond>),
    /// `a && b`.
    And(Box<Cond>, Box<Cond>),
    /// `!a`.
    Not(Box<Cond>, Span),
    /// `lhs op rhs`.
    Compare {
        /// Left operand.
        lhs: Expr,
        /// Operator.
        op: CmpOp,
        /// Right operand.
        rhs: Expr,
        /// Whole comparison.
        span: Span,
    },
    /// A boolean value tested directly: `if ready { ... }`.
    Value(Expr),
    /// A predicate call such as `has_state(Mana)`.
    Call {
        /// Called predicate.
        path: Path,
        /// Arguments.
        args: Vec<Arg>,
        /// Whole call.
        span: Span,
    },
    /// `value in range`.
    InRange {
        /// Tested value.
        value: Expr,
        /// Range.
        range: Range,
        /// Whole test.
        span: Span,
    },
}

impl Cond {
    /// Location of the whole condition.
    pub fn span(&self) -> Span {
        match self {
            Cond::Or(a, b) | Cond::And(a, b) => a.span().to(b.span()),
            Cond::Not(_, span)
            | Cond::Compare { span, .. }
            | Cond::InRange { span, .. }
            | Cond::Call { span, .. } => *span,
            Cond::Value(expr) => expr.span(),
        }
    }
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    /// `==`.
    Eq,
    /// `!=`.
    Ne,
    /// `<`.
    Lt,
    /// `<=`.
    Le,
    /// `>`.
    Gt,
    /// `>=`.
    Ge,
}

impl CmpOp {
    /// The operator that holds when operands are swapped.
    pub fn flipped(self) -> CmpOp {
        match self {
            CmpOp::Lt => CmpOp::Gt,
            CmpOp::Le => CmpOp::Ge,
            CmpOp::Gt => CmpOp::Lt,
            CmpOp::Ge => CmpOp::Le,
            other => other,
        }
    }
}

/// A constant range `a..b` or `a..=b`; either end may be omitted.
#[derive(Debug, Clone)]
pub struct Range {
    /// Lower bound.
    pub start: Option<Expr>,
    /// Upper bound.
    pub end: Option<Expr>,
    /// Whether the upper bound is inclusive.
    pub inclusive: bool,
    /// Whole range.
    pub span: Span,
}

/// One call or macro argument, optionally named.
#[derive(Debug, Clone)]
pub struct Arg {
    /// Name for `name = value` arguments.
    pub name: Option<Ident>,
    /// Argument value.
    pub value: ArgValue,
    /// Whole argument.
    pub span: Span,
}

/// An argument value.
#[derive(Debug, Clone)]
pub enum ArgValue {
    /// An expression.
    Expr(Expr),
    /// A range such as `1..=6`.
    Range(Range),
}

impl ArgValue {
    /// Location of the value.
    pub fn span(&self) -> Span {
        match self {
            ArgValue::Expr(expr) => expr.span(),
            ArgValue::Range(range) => range.span,
        }
    }
}

/// An expression: a constant, a State field read, or a builtin call.
#[derive(Debug, Clone)]
pub enum Expr {
    /// An integer literal.
    Int(i64, Span),
    /// A decimal literal such as `0.5`.
    Float(f64, Span),
    /// `true` or `false`.
    Bool(bool, Span),
    /// A string literal; `raw` keeps the escaped source text for spans.
    Str {
        /// Unescaped value.
        value: String,
        /// Text between the quotes, exactly as written.
        raw: String,
        /// Location including quotes.
        span: Span,
    },
    /// A condition used as a boolean value: `let ok = a > b;`.
    Cond(Box<Cond>, Span),
    /// A call such as `random(1..=6)`.
    Call {
        /// Called builtin.
        path: Path,
        /// Arguments.
        args: Vec<Arg>,
        /// Whole call.
        span: Span,
    },
    /// A path to a constant, enum variant, or bare variant.
    Path(Path),
    /// A State field.
    Place(Place),
    /// Unary negation.
    Neg(Box<Expr>, Span),
    /// Binary arithmetic.
    Binary(Box<Expr>, BinOp, Box<Expr>, Span),
    /// `expr as Type`.
    Cast(Box<Expr>, Path, Span),
}

impl Expr {
    /// Location of the expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::Bool(_, span)
            | Expr::Str { span, .. }
            | Expr::Call { span, .. }
            | Expr::Cond(_, span)
            | Expr::Neg(_, span)
            | Expr::Binary(_, _, _, span)
            | Expr::Cast(_, _, span) => *span,
            Expr::Path(path) => path.span,
            Expr::Place(place) => place.span,
        }
    }
}

/// Binary arithmetic operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
    /// `%`.
    Rem,
}
