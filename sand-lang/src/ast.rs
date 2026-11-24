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

/// A source type annotation, shared by fields, parameters, returns and casts.
#[derive(Debug, Clone)]
pub struct Type {
    /// Type syntax.
    pub kind: TypeKind,
    /// Whole annotation.
    pub span: Span,
}

/// Named and composite source type forms.
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A named scalar, enum, record or imported type.
    Named(Path),
    /// A concrete callable signature.
    Function {
        /// Argument types in call order.
        parameters: Vec<Type>,
        /// Return type, or no value for a statement function.
        returns: Option<Box<Type>>,
    },
    /// A named type applied to other value types.
    Applied { path: Path, arguments: Vec<Type> },
    /// A fixed-length homogeneous array with a constant length.
    Array {
        element: Box<Type>,
        length: Box<Expr>,
    },
}

impl Type {
    /// Construct a named annotation without losing its source span.
    pub fn named(path: Path) -> Self {
        Self {
            span: path.span,
            kind: TypeKind::Named(path),
        }
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
#[derive(Debug, Clone)]
pub struct Path {
    /// Path segments, including leading `crate`, `super`, or `self`.
    pub segments: Vec<Ident>,
    /// Explicit applications on individual segments, written `::<T>`.
    pub applications: Vec<PathApplication>,
    /// Location of the whole path.
    pub span: Span,
}

/// A type application on one segment of a qualified path.
#[derive(Debug, Clone)]
pub struct PathApplication {
    /// Zero-based index into the path's segments.
    pub segment: usize,
    /// Concrete type syntax.
    pub arguments: Vec<Type>,
}

impl Path {
    /// Explicit arguments on the final item; preceding applications belong to types.
    pub fn arguments(&self) -> &[Type] {
        self.applications
            .iter()
            .find(|application| application.segment + 1 == self.segments.len())
            .map_or(&[], |application| application.arguments.as_slice())
    }
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
            ItemKind::Struct(item) => Some(&item.name),
            ItemKind::Trait(item) => Some(&item.name),
            ItemKind::Flow(item) => Some(&item.name),
            ItemKind::Fn(item) => Some(&item.name),
            ItemKind::ExternFn(item) => Some(&item.name),
            ItemKind::Data(item) => Some(&item.name),
            ItemKind::Event(item) => Some(&item.name),
            ItemKind::Use(_) | ItemKind::Impl(_) | ItemKind::Error(_) => None,
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
    /// A named, copyable record value.
    Struct(StructItem),
    /// A contract for a set of methods on a value type.
    Trait(TraitItem),
    /// Associated functions and methods for a State or struct.
    Impl(ImplItem),
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
    pub ty: Type,
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

/// A named record with typed fields.
#[derive(Debug, Clone)]
pub struct StructItem {
    /// Type parameters specialized in each concrete record layout.
    pub generics: Vec<Ident>,
    /// Type name.
    pub name: Ident,
    /// Fields in declaration order.
    pub fields: Vec<Field>,
}

/// A named method contract.
#[derive(Debug, Clone)]
pub struct TraitItem {
    /// Contract name.
    pub name: Ident,
    /// Methods required by this contract.
    pub methods: Vec<TraitMethod>,
}

/// The source signature of a required method.
#[derive(Debug, Clone)]
pub struct TraitMethod {
    /// Method name.
    pub name: Ident,
    /// Optional value or borrowed receiver.
    pub receiver: Option<Receiver>,
    /// Explicit parameters.
    pub params: Vec<Param>,
    /// Declared return type.
    pub ret: Option<Type>,
}

/// An inherent implementation for a State or struct.
#[derive(Debug, Clone)]
pub struct ImplItem {
    /// Contract implemented by this block, if any.
    pub contract: Option<Path>,
    /// Type parameters bound by the implemented type.
    pub generics: Vec<Ident>,
    /// Implemented type.
    pub target: Path,
    /// Functions declared inside the implementation.
    pub methods: Vec<Item>,
}

/// The explicit `self` parameter of a method.
#[derive(Debug, Clone)]
pub struct Receiver {
    /// Whether writes through this receiver are allowed.
    pub mutable: bool,
    /// Whether the receiver uses `&` syntax.
    pub borrowed: bool,
    /// Source location.
    pub span: Span,
}

/// One State field.
#[derive(Debug, Clone)]
pub struct Field {
    /// Whether this struct field is public outside its module.
    pub is_pub: bool,
    /// Documentation lines.
    pub docs: Vec<String>,
    /// Field attributes such as `#[clamp(0..=100)]`.
    pub attrs: Vec<Attr>,
    /// Field name.
    pub name: Ident,
    /// Field value type.
    pub ty: Type,
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
    /// `damage_type Name { message_id: "..." }`.
    DamageType,
    /// `predicate Name { ... }`.
    Predicate,
    /// `advancement Name { ... }`.
    Advancement,
    /// `dialog Name { ... }`.
    Dialog,
    Dimension,
    Biome,
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
            DataKind::DamageType => "damage_type",
            DataKind::Predicate => "predicate",
            DataKind::Advancement => "advancement",
            DataKind::Dialog => "dialog",
            DataKind::Dimension => "dimension",
            DataKind::Biome => "biome",
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
    /// Whether this parameter can be reassigned with `mut`.
    pub mutable: bool,
    /// Parameter name.
    pub name: Ident,
    /// Parameter type.
    pub ty: Type,
    /// Whole parameter.
    pub span: Span,
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct FnItem {
    /// Required traits, paired with their declared type parameters.
    pub bounds: Vec<(Ident, Path)>,
    /// Type parameters specialized for each concrete call.
    pub generics: Vec<Ident>,
    /// Optional method receiver; valid only inside an impl.
    pub receiver: Option<Receiver>,
    /// Function name.
    pub name: Ident,
    /// Parameters in order.
    pub params: Vec<Param>,
    /// Declared return type.
    pub ret: Option<Type>,
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
    /// A method call or evaluated value statement.
    Expr(Expr, Span),
    /// `let mut name: Type = value;`.
    Let {
        /// Whether the binding is `mut`.
        mutable: bool,
        /// Bound name.
        name: Ident,
        /// Declared type.
        ty: Option<Type>,
        /// Initial value.
        value: Option<Expr>,
        /// Whole statement.
        span: Span,
    },
    /// Irrefutable destructuring of an owned record or fixed array value.
    Destructure {
        pattern: Pattern,
        ty: Option<Type>,
        value: Expr,
        span: Span,
    },
    /// `return;` or `return value;`.
    Return {
        /// Returned value.
        value: Option<Expr>,
        /// Whole statement.
        span: Span,
    },
    /// `continue;`: skip to the next pass of the innermost loop.
    Continue(Span),
    /// `break;`: leave the innermost `while`, `loop`, or counting `for`.
    Break(Span),
    /// `emit Event(args);`: run every handler of a custom event.
    Emit {
        /// The emitted event.
        event: Path,
        /// Payload arguments, in order.
        args: Vec<Arg>,
        /// Whole statement.
        span: Span,
    },
    /// `loop { ... }`: repeat until `break` or `return`.
    Loop {
        /// Repeated body.
        body: Block,
        /// Whole statement.
        span: Span,
    },
    /// Iterate an independent snapshot of a fixed-size array.
    ForValues {
        /// Immutable element binding.
        binding: Ident,
        /// Array expression, evaluated once.
        value: Expr,
        /// Loop body.
        body: Block,
        /// Whole statement.
        span: Span,
    },
    /// `for i in start..end { ... }`: count through a range of numbers.
    ForRange {
        /// Loop variable.
        binding: Ident,
        /// Counted range; `..=` includes the end.
        range: Range,
        /// Iterate from the last included value down to the start.
        reverse: bool,
        /// Positive constant increment magnitude, defaulting to one.
        step: Option<Expr>,
        /// Loop body.
        body: Block,
        /// Whole statement.
        span: Span,
    },
    /// Select one exhaustive arm after evaluating the value once.
    Match {
        /// Matched value.
        value: Expr,
        /// Arms in source order.
        arms: Vec<MatchArm>,
        /// Whole statement.
        span: Span,
    },
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

/// An irrefutable value pattern. Bindings receive independent owned copies.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// A binding, with optional per-binding mutability.
    Bind { name: Ident, mutable: bool },
    /// Ignore this value.
    Ignore(Span),
    /// A record's named fields; `rest` permits omitted fields.
    Record {
        path: Path,
        fields: Vec<(Ident, Pattern)>,
        rest: bool,
        span: Span,
    },
    /// An exact-length fixed array pattern.
    Array(Vec<Pattern>, Span),
}

impl Pattern {
    /// Location of the whole pattern.
    pub fn span(&self) -> Span {
        match self {
            Self::Bind { name, .. } => name.span,
            Self::Ignore(span) | Self::Array(_, span) | Self::Record { span, .. } => *span,
        }
    }
}

/// A selector for one match arm.
#[derive(Debug, Clone)]
pub enum MatchPattern {
    /// Covers every remaining value without introducing bindings.
    Wildcard,
    /// Constant alternatives separated by `|`.
    Constants(Vec<Expr>),
    /// Copies some or all of the matched value into arm-local bindings.
    Binding(Pattern),
}

/// One arm of a `match` statement.
#[derive(Debug, Clone)]
pub struct MatchArm {
    /// Selector and any local bindings.
    pub pattern: MatchPattern,
    /// Optional condition evaluated after the pattern matches.
    pub guard: Option<Cond>,
    /// Body scoped to this arm.
    pub body: Block,
    /// Whole arm.
    pub span: Span,
}

/// One value-producing match arm. Only the selected expression is evaluated.
#[derive(Debug, Clone)]
pub struct MatchValueArm {
    /// Selector and any local bindings.
    pub pattern: MatchPattern,
    /// Optional condition evaluated after the pattern matches.
    pub guard: Option<Cond>,
    /// Value produced by this arm.
    pub value: Expr,
    /// Whole arm.
    pub span: Span,
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
    /// An indexed or projected writable value.
    Value(Box<Expr>),
}

impl AssignTarget {
    /// Location of the target.
    pub fn span(&self) -> Span {
        match self {
            AssignTarget::Place(place) => place.span,
            AssignTarget::Local(ident) => ident.span,
            AssignTarget::Value(value) => value.span(),
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
    /// Further record-field projections.
    pub projections: Vec<Ident>,
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

/// One authored move closure.
#[derive(Debug, Clone)]
pub struct ClosureExpr {
    /// Explicitly typed arguments.
    pub parameters: Vec<Param>,
    /// Return annotation, if the closure produces a value.
    pub returns: Option<Type>,
    /// Authored closure body.
    pub body: Block,
    /// Whole closure.
    pub span: Span,
}

/// An expression: a constant, a State field read, or a builtin call.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A callable with an owned snapshot of its captured locals.
    Closure(Box<ClosureExpr>),
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
    /// A homogeneous array literal, evaluated from left to right.
    Array(Vec<Expr>, Span),
    /// Repeat one evaluated value a constant number of times.
    ArrayRepeat {
        value: Box<Expr>,
        length: Box<Expr>,
        span: Span,
    },
    /// Scoped statements followed by a value expression.
    Block {
        /// Statements evaluated before the result.
        body: Block,
        /// The result expression, without a terminating semicolon.
        value: Box<Expr>,
        /// Whole expression.
        span: Span,
    },
    /// An exhaustive match that evaluates only the selected arm's value.
    Match {
        value: Box<Expr>,
        arms: Vec<MatchValueArm>,
        span: Span,
    },
    /// An indexed element of an array value.
    Index(Box<Expr>, Box<Expr>, Span),
    /// A named record literal, evaluated in authored field order.
    Struct {
        /// Record type.
        path: Path,
        /// Field initializers.
        fields: Vec<(Ident, Expr)>,
        /// Whole expression.
        span: Span,
    },
    /// A call using a value or State receiver.
    Method {
        /// Receiver expression.
        receiver: Box<Expr>,
        /// Method name.
        name: Ident,
        /// Explicit generic arguments, written `::<T>`.
        type_arguments: Vec<Type>,
        /// Explicit arguments.
        args: Vec<Arg>,
        /// Whole expression.
        span: Span,
    },
    /// A field selected from a computed record value.
    Project(Box<Expr>, Ident, Span),
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
    Cast(Box<Expr>, Type, Span),
}

impl Expr {
    /// Location of the expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::Closure(closure) => closure.span,
            Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::Bool(_, span)
            | Expr::Str { span, .. }
            | Expr::Call { span, .. }
            | Expr::Struct { span, .. }
            | Expr::Method { span, .. }
            | Expr::Project(_, _, span)
            | Expr::Array(_, span)
            | Expr::Match { span, .. }
            | Expr::Block { span, .. }
            | Expr::ArrayRepeat { span, .. }
            | Expr::Index(_, _, span)
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
