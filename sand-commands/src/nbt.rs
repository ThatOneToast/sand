//! Typed NBT targets, paths, values, and standalone `data` command IR.
//!
//! This module is the canonical command-layer representation for all three
//! vanilla data locations: storage, entity, and block entity. Public builders
//! retain structure until [`DataCommand::try_render`] is called. The explicit
//! [`NbtPath::raw`] and [`NbtValue::raw`] constructors are opaque escape
//! hatches for modded or newly introduced syntax.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use crate::Build;
use crate::coord::BlockPos;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::{Selector, TargetArgument};

// ── Values ───────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::NbtValue",
    aliases = ["sand::cmd::NbtValue", "sand::command::NbtValue", "sand::data::SnbtValue", "sand::prelude::SnbtValue", "sand::prelude::cmd::NbtValue", "sand::state::SnbtValue"],
    module = "sand::data",
    summary = "A typed SNBT value used by `data modify` and `data merge`.",
    context = "A typed SNBT value used by `data modify` and `data merge`. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::NbtValue;",
    variants(Bool = "Selects the bool NBT or data-command operation.", Byte = "Selects the byte NBT or data-command operation.", Compound = "Selects the compound NBT or data-command operation.", Double = "Selects the double NBT or data-command operation.", Float = "Selects the float NBT or data-command operation.", Int = "Selects the int NBT or data-command operation.", List = "Selects the list NBT or data-command operation.", Long = "Selects the long NBT or data-command operation.", Raw = "Explicit opaque SNBT. Sand renders this unchanged and does not parse it.", Short = "Selects the short NBT or data-command operation.", String = "Selects the string NBT or data-command operation."),
    variant_fields(Bool = ["Selects the bool NBT or data-command operation."], Byte = ["Selects the byte NBT or data-command operation."], Compound = ["Selects the compound NBT or data-command operation."], Double = ["Selects the double NBT or data-command operation."], Float = ["Selects the float NBT or data-command operation."], Int = ["Selects the int NBT or data-command operation."], List = ["Selects the list NBT or data-command operation."], Long = ["Selects the long NBT or data-command operation."], Raw = ["Explicit opaque SNBT. Sand renders this unchanged and does not parse it."], Short = ["Selects the short NBT or data-command operation."], String = ["Selects the string NBT or data-command operation."]),
)]
/// A typed SNBT value used by `data modify` and `data merge`.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    #[doc = "Selects the bool NBT or data-command operation."]
    Bool(#[doc = "Selects the bool NBT or data-command operation."] bool),
    #[doc = "Selects the byte NBT or data-command operation."]
    Byte(#[doc = "Selects the byte NBT or data-command operation."] i8),
    #[doc = "Selects the short NBT or data-command operation."]
    Short(#[doc = "Selects the short NBT or data-command operation."] i16),
    #[doc = "Selects the int NBT or data-command operation."]
    Int(#[doc = "Selects the int NBT or data-command operation."] i32),
    #[doc = "Selects the long NBT or data-command operation."]
    Long(#[doc = "Selects the long NBT or data-command operation."] i64),
    #[doc = "Selects the float NBT or data-command operation."]
    Float(#[doc = "Selects the float NBT or data-command operation."] f32),
    #[doc = "Selects the double NBT or data-command operation."]
    Double(#[doc = "Selects the double NBT or data-command operation."] f64),
    #[doc = "Selects the string NBT or data-command operation."]
    String(#[doc = "Selects the string NBT or data-command operation."] String),
    #[doc = "Selects the list NBT or data-command operation."]
    List(#[doc = "Selects the list NBT or data-command operation."] Vec<NbtValue>),
    #[doc = "Selects the compound NBT or data-command operation."]
    Compound(#[doc = "Selects the compound NBT or data-command operation."] NbtCompound),
    /// Explicit opaque SNBT. Sand renders this unchanged and does not parse it.
    Raw(#[doc = "Explicit opaque SNBT. Sand renders this unchanged and does not parse it."] String),
}

impl NbtValue {
    /// Creates an SNBT list from typed NBT values.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtValue::list",
        aliases = ["sand::cmd::NbtValue::list", "sand::command::NbtValue::list", "sand::data::SnbtValue::list", "sand::prelude::SnbtValue::list", "sand::prelude::cmd::NbtValue::list", "sand::state::SnbtValue::list"],
        module = "sand::data",
        kind = "method",
        summary = "Creates an SNBT list from typed NBT values.",
        context = "Creates an SNBT list from typed NBT values. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(values = "`values` is used when creating an SNBT list from typed NBT values."),
        returns = "A `NbtValue` representing an SNBT list from typed NBT values.",
        example = "use sand::prelude::*;\n\nfn demonstrate(values: impl IntoIterator < Item = impl Into < sand::data::NbtValue > >)  {\n    let nbt_value = sand::data::NbtValue::list(values);\n}",
    )]
    pub fn list(values: impl IntoIterator<Item = impl Into<NbtValue>>) -> Self {
        Self::List(values.into_iter().map(Into::into).collect())
    }

    /// Wraps a typed SNBT compound as an NBT value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtValue::compound",
        aliases = ["sand::cmd::NbtValue::compound", "sand::command::NbtValue::compound", "sand::data::SnbtValue::compound", "sand::prelude::SnbtValue::compound", "sand::prelude::cmd::NbtValue::compound", "sand::state::SnbtValue::compound"],
        module = "sand::data",
        kind = "method",
        summary = "Wraps a typed SNBT compound as an NBT value.",
        context = "Wraps a typed SNBT compound as an NBT value. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to wrap a typed SNBT compound as an NBT value."),
        returns = "A `NbtValue` wrapping a typed SNBT compound as an NBT value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: sand::data::NbtCompound)  {\n    let nbt_value = sand::data::NbtValue::compound(value);\n}",
    )]
    pub fn compound(value: NbtCompound) -> Self {
        Self::Compound(value)
    }

    /// Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtValue::raw",
        aliases = ["sand::cmd::NbtValue::raw", "sand::command::NbtValue::raw", "sand::data::SnbtValue::raw", "sand::prelude::SnbtValue::raw", "sand::prelude::cmd::NbtValue::raw", "sand::state::SnbtValue::raw"],
        module = "sand::data",
        kind = "method",
        summary = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        context = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(snbt = "`snbt` is used to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility."),
        returns = "A `NbtValue` that provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        example = "use sand::prelude::*;\n\nfn demonstrate(snbt: impl Into < String >)  {\n    let nbt_value = sand::data::NbtValue::raw(snbt);\n}",
    )]
    pub fn raw(snbt: impl Into<String>) -> Self {
        Self::Raw(snbt.into())
    }
}

impl fmt::Display for NbtValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => f.write_str(if *value { "1b" } else { "0b" }),
            Self::Byte(value) => write!(f, "{value}b"),
            Self::Short(value) => write!(f, "{value}s"),
            Self::Int(value) => write!(f, "{value}"),
            Self::Long(value) => write!(f, "{value}L"),
            Self::Float(value) => write!(f, "{value}f"),
            Self::Double(value) => write!(f, "{value}d"),
            Self::String(value) => write!(
                f,
                "\"{}\"",
                value.replace('\\', "\\\\").replace('"', "\\\"")
            ),
            Self::List(values) => {
                let values = values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "[{values}]")
            }
            Self::Compound(value) => value.fmt(f),
            Self::Raw(value) => f.write_str(value),
        }
    }
}

macro_rules! nbt_from {
    ($type:ty, $variant:ident) => {
        impl From<$type> for NbtValue {
            fn from(value: $type) -> Self {
                Self::$variant(value)
            }
        }
    };
}

nbt_from!(bool, Bool);
nbt_from!(i8, Byte);
nbt_from!(i16, Short);
nbt_from!(i32, Int);
nbt_from!(i64, Long);
nbt_from!(f32, Float);
nbt_from!(f64, Double);
nbt_from!(String, String);

impl From<&str> for NbtValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl<T: Into<NbtValue>> From<Vec<T>> for NbtValue {
    fn from(value: Vec<T>) -> Self {
        Self::List(value.into_iter().map(Into::into).collect())
    }
}

impl From<NbtCompound> for NbtValue {
    fn from(value: NbtCompound) -> Self {
        Self::Compound(value)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::NbtCompound",
    aliases = ["sand::cmd::NbtCompound", "sand::command::NbtCompound", "sand::data::SnbtCompound", "sand::prelude::NbtCompound", "sand::prelude::SnbtCompound", "sand::prelude::cmd::NbtCompound", "sand::state::SnbtCompound"],
    module = "sand::data",
    summary = "A typed SNBT compound preserving declaration order.",
    context = "A typed SNBT compound preserving declaration order. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::NbtCompound;",
)]
/// A typed SNBT compound preserving declaration order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NbtCompound {
    entries: Vec<(String, NbtValue)>,
}

impl NbtCompound {
    /// Creates an empty typed SNBT compound.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtCompound::new",
        aliases = ["sand::cmd::NbtCompound::new", "sand::command::NbtCompound::new", "sand::data::SnbtCompound::new", "sand::prelude::NbtCompound::new", "sand::prelude::SnbtCompound::new", "sand::prelude::cmd::NbtCompound::new", "sand::state::SnbtCompound::new"],
        module = "sand::data",
        kind = "method",
        summary = "Creates an empty typed SNBT compound.",
        context = "Creates an empty typed SNBT compound. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "A `NbtCompound` representing an empty typed SNBT compound.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let nbt_compound = sand::data::NbtCompound::new();\n}",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces a named value in this SNBT compound builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtCompound::field",
        aliases = ["sand::cmd::NbtCompound::field", "sand::command::NbtCompound::field", "sand::data::SnbtCompound::field", "sand::prelude::NbtCompound::field", "sand::prelude::SnbtCompound::field", "sand::prelude::cmd::NbtCompound::field", "sand::state::SnbtCompound::field"],
        module = "sand::data",
        kind = "method",
        summary = "Adds or replaces a named value in this SNBT compound builder.",
        context = "Adds or replaces a named value in this SNBT compound builder. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to add or replaces a named value in this SNBT compound builder.", value = "`value` provides the value being applied or compared used to add or replaces a named value in this SNBT compound builder."),
        returns = "The `NbtCompound` value with the documented change applied to add or replaces a named value in this SNBT compound builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_compound_value: sand::data::NbtCompound, key: impl Into < String >, value: impl Into < sand::data::NbtValue >)  {\n    let updated_nbt_compound = nbt_compound_value.field(key, value);\n}",
    )]
    pub fn field(mut self, key: impl Into<String>, value: impl Into<NbtValue>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    /// Builds the typed Minecraft data modification for insert.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtCompound::insert",
        aliases = ["sand::cmd::NbtCompound::insert", "sand::command::NbtCompound::insert", "sand::data::SnbtCompound::insert", "sand::prelude::NbtCompound::insert", "sand::prelude::SnbtCompound::insert", "sand::prelude::cmd::NbtCompound::insert", "sand::state::SnbtCompound::insert"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for insert.",
        context = "Builds the typed Minecraft data modification for insert. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to build the typed Minecraft data modification for insert.", value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for insert."),
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_compound_value: &mut sand::data::NbtCompound, key: impl Into < String >, value: impl Into < sand::data::NbtValue >)  {\n    let insert = nbt_compound_value.insert(key, value);\n}",
    )]
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<NbtValue>) {
        self.entries.push((key.into(), value.into()));
    }

    /// Reports whether this SNBT compound contains no fields.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtCompound::is_empty",
        aliases = ["sand::cmd::NbtCompound::is_empty", "sand::command::NbtCompound::is_empty", "sand::data::SnbtCompound::is_empty", "sand::prelude::NbtCompound::is_empty", "sand::prelude::SnbtCompound::is_empty", "sand::prelude::cmd::NbtCompound::is_empty", "sand::state::SnbtCompound::is_empty"],
        module = "sand::data",
        kind = "method",
        summary = "Reports whether this SNBT compound contains no fields.",
        context = "Reports whether this SNBT compound contains no fields. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "`true` when the documented condition holds to report whether this SNBT compound contains no fields; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_compound_value: &sand::data::NbtCompound)  {\n    let is_is_empty = nbt_compound_value.is_empty();\n}",
    )]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for NbtCompound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self
            .entries
            .iter()
            .map(|(key, value)| format!("{}:{value}", render_compound_key(key)))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{{{entries}}}")
    }
}

fn render_compound_key(key: &str) -> String {
    if !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'))
    {
        key.to_owned()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

// ── Paths and targets ────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::NbtPath",
    aliases = ["sand::cmd::NbtPath", "sand::command::NbtPath", "sand::prelude::NbtPath", "sand::prelude::cmd::NbtPath", "sand::state::NbtPath"],
    module = "sand::data",
    summary = "A standalone NBT path, independent of the location it is later attached to.",
    context = "A standalone NBT path, independent of the location it is later attached to. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::NbtPath;",
)]
/// A standalone NBT path, independent of the location it is later attached to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtPath {
    value: String,
    raw: bool,
}

impl NbtPath {
    /// Construct a structurally checked path.
    ///
    /// Validation is performed at the fallible command-render/export boundary
    /// so ordinary command-producing call sites remain ergonomic.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::new",
        aliases = ["sand::cmd::NbtPath::new", "sand::command::NbtPath::new", "sand::prelude::NbtPath::new", "sand::prelude::cmd::NbtPath::new", "sand::state::NbtPath::new"],
        module = "sand::data",
        kind = "method",
        summary = "Construct a structurally checked path. Validation is performed at the fallible command-render/export boundary so ordinary command-producing call sites remain ergonomic.",
        context = "Construct a structurally checked path. Validation is performed at the fallible command-render/export boundary so ordinary command-producing call sites remain ergonomic. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Validation is performed at the fallible command-render/export boundary so ordinary command-producing call sites remain ergonomic.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to construct a structurally checked path. Validation is performed at the fallible command-render/export boundary so ordinary command-producing call sites remain ergonomic."),
        returns = "A `NbtPath` representing a structurally checked path. Validation is performed at the fallible command-render/export boundary so ordinary command-producing call sites remain ergonomic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: impl Into < String >)  {\n    let nbt_path = sand::data::NbtPath::new(path);\n}",
    )]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            value: path.into(),
            raw: false,
        }
    }

    /// Explicit opaque path escape hatch. The path renders unchanged.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::raw",
        aliases = ["sand::cmd::NbtPath::raw", "sand::command::NbtPath::raw", "sand::prelude::NbtPath::raw", "sand::prelude::cmd::NbtPath::raw", "sand::state::NbtPath::raw"],
        module = "sand::data",
        kind = "method",
        summary = "Explicit opaque path escape hatch. The path renders unchanged.",
        context = "Explicit opaque path escape hatch. The path renders unchanged. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to use explicit opaque path escape hatch. The path renders unchanged."),
        returns = "A `NbtPath` configured for explicit opaque path escape hatch. The path renders unchanged.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: impl Into < String >)  {\n    let nbt_path = sand::data::NbtPath::raw(path);\n}",
    )]
    pub fn raw(path: impl Into<String>) -> Self {
        Self {
            value: path.into(),
            raw: true,
        }
    }

    /// Compatibility spelling for a standalone typed path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::root",
        aliases = ["sand::cmd::NbtPath::root", "sand::command::NbtPath::root", "sand::prelude::NbtPath::root", "sand::prelude::cmd::NbtPath::root", "sand::state::NbtPath::root"],
        module = "sand::data",
        kind = "method",
        summary = "Compatibility spelling for a standalone typed path.",
        context = "Compatibility spelling for a standalone typed path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to use compatibility spelling for a standalone typed path."),
        returns = "A `NbtPath` configured for compatibility spelling for a standalone typed path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: impl Into < String >)  {\n    let nbt_path = sand::data::NbtPath::root(path);\n}",
    )]
    pub fn root(path: impl Into<String>) -> Self {
        Self::new(path)
    }

    /// Borrows the rendered NBT path text without allocating.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::as_str",
        aliases = ["sand::cmd::NbtPath::as_str", "sand::command::NbtPath::as_str", "sand::prelude::NbtPath::as_str", "sand::prelude::cmd::NbtPath::as_str", "sand::state::NbtPath::as_str"],
        module = "sand::data",
        kind = "method",
        summary = "Borrows the rendered NBT path text without allocating.",
        context = "Borrows the rendered NBT path text without allocating. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to borrow the rendered NBT path text without allocating.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_path_value: &sand::data::NbtPath)  {\n    let as_str = nbt_path_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::is_raw",
        aliases = ["sand::cmd::NbtPath::is_raw", "sand::command::NbtPath::is_raw", "sand::prelude::NbtPath::is_raw", "sand::prelude::cmd::NbtPath::is_raw", "sand::state::NbtPath::is_raw"],
        module = "sand::data",
        kind = "method",
        summary = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        context = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "`true` when the documented condition holds to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_path_value: &sand::data::NbtPath)  {\n    let is_is_raw = nbt_path_value.is_raw();\n}",
    )]
    pub fn is_raw(&self) -> bool {
        self.raw
    }

    /// Extends this typed NBT reference with the supplied field selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::field",
        aliases = ["sand::cmd::NbtPath::field", "sand::command::NbtPath::field", "sand::prelude::NbtPath::field", "sand::prelude::cmd::NbtPath::field", "sand::state::NbtPath::field"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied field selector.",
        context = "Extends this typed NBT reference with the supplied field selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to extend this typed NBT reference with the supplied field selector."),
        returns = "The `NbtPath` value with the documented change applied to extend this typed NBT reference with the supplied field selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_path_value: &sand::data::NbtPath, key: impl AsRef < str >)  {\n    let updated_nbt_path = nbt_path_value.field(key);\n}",
    )]
    pub fn field(&self, key: impl AsRef<str>) -> Self {
        let key = key.as_ref();
        let value = if self.value.is_empty() {
            render_path_key(key)
        } else {
            format!("{}.{}", self.value, render_path_key(key))
        };
        Self {
            value,
            raw: self.raw,
        }
    }

    /// Extends this typed NBT reference with the supplied key selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::key",
        aliases = ["sand::cmd::NbtPath::key", "sand::command::NbtPath::key", "sand::prelude::NbtPath::key", "sand::prelude::cmd::NbtPath::key", "sand::state::NbtPath::key"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied key selector.",
        context = "Extends this typed NBT reference with the supplied key selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to extend this typed NBT reference with the supplied key selector."),
        returns = "The `NbtPath` value with the documented change applied to extend this typed NBT reference with the supplied key selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_path_value: &sand::data::NbtPath, key: impl AsRef < str >)  {\n    let updated_nbt_path = nbt_path_value.key(key);\n}",
    )]
    pub fn key(&self, key: impl AsRef<str>) -> Self {
        self.field(key)
    }

    /// Extends this typed NBT reference with the supplied index selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtPath::index",
        aliases = ["sand::cmd::NbtPath::index", "sand::command::NbtPath::index", "sand::prelude::NbtPath::index", "sand::prelude::cmd::NbtPath::index", "sand::state::NbtPath::index"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied index selector.",
        context = "Extends this typed NBT reference with the supplied index selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(index = "`index` is used to extend this typed NBT reference with the supplied index selector."),
        returns = "The `NbtPath` value with the documented change applied to extend this typed NBT reference with the supplied index selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_path_value: &sand::data::NbtPath, index: i32)  {\n    let updated_nbt_path = nbt_path_value.index(index);\n}",
    )]
    pub fn index(&self, index: i32) -> Self {
        Self {
            value: format!("{}[{index}]", self.value),
            raw: self.raw,
        }
    }

    fn validate(&self) -> CommandResult<()> {
        if self.raw {
            return Ok(());
        }
        validate_nbt_path(&self.value)
    }
}

impl From<&str> for NbtPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NbtPath {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for NbtPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

fn render_path_key(key: &str) -> String {
    if !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        key.to_owned()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn validate_nbt_path(path: &str) -> CommandResult<()> {
    if path.trim().is_empty() {
        return Err(data_error(
            "path",
            "NBT paths cannot be empty; use a location-level operation for root data",
        ));
    }
    if path != path.trim() || path.chars().any(char::is_control) {
        return Err(data_error(
            "path",
            format!(
                "malformed NBT path `{path}`: leading/trailing whitespace and controls are invalid"
            ),
        ));
    }
    let mut quote = false;
    let mut escaped = false;
    let mut brackets = 0_i32;
    let mut braces = 0_i32;
    for character in path.chars() {
        if quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quote = false;
            }
            continue;
        }
        match character {
            '"' => quote = true,
            '[' => brackets += 1,
            ']' => {
                brackets -= 1;
                if brackets < 0 {
                    return Err(data_error("path", format!("unbalanced `]` in `{path}`")));
                }
            }
            '{' => braces += 1,
            '}' => {
                braces -= 1;
                if braces < 0 {
                    return Err(data_error("path", format!("unbalanced `}}` in `{path}`")));
                }
            }
            _ => {}
        }
    }
    if quote || brackets != 0 || braces != 0 {
        return Err(data_error(
            "path",
            format!("unbalanced quoted key, list index, or match compound in `{path}`"),
        ));
    }
    if path.contains("..") || path.starts_with('.') || path.ends_with('.') {
        return Err(data_error(
            "path",
            format!("malformed empty path segment in `{path}`"),
        ));
    }
    Ok(())
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::DataTarget",
    aliases = ["sand::cmd::DataTarget", "sand::command::DataTarget", "sand::prelude::cmd::DataTarget"],
    module = "sand::data",
    summary = "Canonical typed location for vanilla `data` commands.",
    context = "Canonical typed location for vanilla `data` commands. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::DataTarget;",
    variants(Block = "Selects the block NBT or data-command operation.", Entity = "Selects the entity NBT or data-command operation.", Storage = "Selects the storage NBT or data-command operation."),
    variant_fields(Block = ["Selects the block NBT or data-command operation."], Entity = ["Selects the entity NBT or data-command operation."], Storage = ["Selects the storage NBT or data-command operation."]),
    member_aliases(
        Block = ["sand::data::NbtLocation::Block", "sand::prelude::NbtLocation::Block", "sand::state::NbtLocation::Block"],
        Entity = ["sand::data::NbtLocation::Entity", "sand::prelude::NbtLocation::Entity", "sand::state::NbtLocation::Entity"],
        Storage = ["sand::data::NbtLocation::Storage", "sand::prelude::NbtLocation::Storage", "sand::state::NbtLocation::Storage"],
    ),
)]
/// Canonical typed location for vanilla `data` commands.
#[derive(Debug, Clone)]
pub enum DataTarget {
    #[doc = "Selects the entity NBT or data-command operation."]
    Entity(#[doc = "Selects the entity NBT or data-command operation."] Selector),
    #[doc = "Selects the block NBT or data-command operation."]
    Block(#[doc = "Selects the block NBT or data-command operation."] BlockPos),
    #[doc = "Selects the storage NBT or data-command operation."]
    Storage(#[doc = "Selects the storage NBT or data-command operation."] String),
}

impl PartialEq for DataTarget {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for DataTarget {}

impl DataTarget {
    /// Creates an entity data-command target from a typed selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataTarget::entity",
        aliases = ["sand::cmd::DataTarget::entity", "sand::command::DataTarget::entity", "sand::data::NbtLocation::entity", "sand::prelude::NbtLocation::entity", "sand::prelude::cmd::DataTarget::entity", "sand::state::NbtLocation::entity"],
        module = "sand::data",
        kind = "method",
        summary = "Creates an entity data-command target from a typed selector.",
        context = "Creates an entity data-command target from a typed selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(selector = "`selector` provides the Minecraft target selection used to create an entity data-command target from a typed selector."),
        returns = "A `DataTarget` representing an entity data-command target from a typed selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target)  {\n    let data_target = sand::data::DataTarget::entity(selector);\n}",
    )]
    pub fn entity(selector: impl TargetArgument) -> Self {
        Self::Entity(selector.into_target_selector())
    }

    /// Creates a block data-command target from typed coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataTarget::block",
        aliases = ["sand::cmd::DataTarget::block", "sand::command::DataTarget::block", "sand::data::NbtLocation::block", "sand::prelude::NbtLocation::block", "sand::prelude::cmd::DataTarget::block", "sand::state::NbtLocation::block"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a block data-command target from typed coordinates.",
        context = "Creates a block data-command target from typed coordinates. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(position = "`position` is used when creating a block data-command target from typed coordinates."),
        returns = "A `DataTarget` representing a block data-command target from typed coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(position: sand::command::BlockPos)  {\n    let data_target = sand::data::DataTarget::block(position);\n}",
    )]
    pub fn block(position: BlockPos) -> Self {
        Self::Block(position)
    }

    /// Creates a command-storage data target from a namespaced identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataTarget::storage",
        aliases = ["sand::cmd::DataTarget::storage", "sand::command::DataTarget::storage", "sand::data::NbtLocation::storage", "sand::prelude::NbtLocation::storage", "sand::prelude::cmd::DataTarget::storage", "sand::state::NbtLocation::storage"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a command-storage data target from a namespaced identifier.",
        context = "Creates a command-storage data target from a namespaced identifier. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(id = "`id` provides the typed resource identifier or location used to create a command-storage data target from a namespaced identifier."),
        returns = "A `DataTarget` representing a command-storage data target from a namespaced identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl Into < String >)  {\n    let data_target = sand::data::DataTarget::storage(id);\n}",
    )]
    pub fn storage(id: impl Into<String>) -> Self {
        Self::Storage(id.into())
    }

    /// Creates an untyped NBT reference at the supplied path under this target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataTarget::path",
        aliases = ["sand::cmd::DataTarget::path", "sand::command::DataTarget::path", "sand::data::NbtLocation::path", "sand::prelude::NbtLocation::path", "sand::prelude::cmd::DataTarget::path", "sand::state::NbtLocation::path"],
        module = "sand::data",
        kind = "method",
        summary = "Creates an untyped NBT reference at the supplied path under this target.",
        context = "Creates an untyped NBT reference at the supplied path under this target. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to create an untyped NBT reference at the supplied path under this target."),
        returns = "The `NbtRef` value produced to create an untyped NBT reference at the supplied path under this target.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_target_value: &sand::data::DataTarget, path: impl Into < sand::data::NbtPath >)  {\n    let path = data_target_value.path(path);\n}",
    )]
    pub fn path(&self, path: impl Into<NbtPath>) -> NbtRef {
        NbtRef::new(self.clone(), path.into())
    }

    /// Creates a typed NBT reference at the supplied path under this target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataTarget::typed_path",
        aliases = ["sand::cmd::DataTarget::typed_path", "sand::command::DataTarget::typed_path", "sand::data::NbtLocation::typed_path", "sand::prelude::NbtLocation::typed_path", "sand::prelude::cmd::DataTarget::typed_path", "sand::state::NbtLocation::typed_path"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a typed NBT reference at the supplied path under this target.",
        context = "Creates a typed NBT reference at the supplied path under this target. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to create a typed NBT reference at the supplied path under this target."),
        returns = "The `NbtRef < T >` value produced to create a typed NBT reference at the supplied path under this target.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(data_target_value: &sand::data::DataTarget, path: impl Into < sand::data::NbtPath >)  {\n    let typed_path = data_target_value.typed_path::<T>(path);\n}",
    )]
    pub fn typed_path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(self.clone(), path.into())
    }

    /// Builds the typed Minecraft data modification for merge.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataTarget::merge",
        aliases = ["sand::cmd::DataTarget::merge", "sand::command::DataTarget::merge", "sand::data::NbtLocation::merge", "sand::prelude::NbtLocation::merge", "sand::prelude::cmd::DataTarget::merge", "sand::state::NbtLocation::merge"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for merge.",
        context = "Builds the typed Minecraft data modification for merge. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for merge."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for merge.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_target_value: &sand::data::DataTarget, value: sand::data::NbtCompound)  {\n    let merge = data_target_value.merge(value);\n}",
    )]
    pub fn merge(&self, value: NbtCompound) -> DataCommand {
        DataCommand::Merge {
            target: self.clone(),
            value,
        }
    }

    fn validate(&self, write: bool) -> CommandResult<()> {
        match self {
            Self::Storage(id) => validate_resource_location(id),
            Self::Block(_) => Ok(()),
            Self::Entity(selector) => {
                let rendered = selector.to_string();
                if write && selector_may_be_many(&rendered) {
                    return Err(data_error(
                        "target",
                        format!(
                            "entity data modification requires a single writable entity target; `{rendered}` may select multiple entities. Execute as each subject and use `@s`, or use a typed inventory/item operation"
                        ),
                    ));
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for DataTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entity(selector) => write!(f, "entity {selector}"),
            Self::Block(position) => write!(f, "block {position}"),
            Self::Storage(id) => write!(f, "storage {id}"),
        }
    }
}

fn validate_resource_location(id: &str) -> CommandResult<()> {
    let Some((namespace, path)) = id.split_once(':') else {
        return Err(data_error(
            "target",
            format!("storage resource location `{id}` must use `namespace:path`"),
        ));
    };
    let namespace_ok = !namespace.is_empty()
        && namespace
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-'));
    let path_ok = !path.is_empty()
        && path.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-' | '/')
        });
    if !namespace_ok || !path_ok {
        return Err(data_error(
            "target",
            format!("invalid storage resource location `{id}`"),
        ));
    }
    Ok(())
}

fn selector_may_be_many(selector: &str) -> bool {
    matches!(selector, "@a" | "@e")
        || ((selector.starts_with("@a[") || selector.starts_with("@e["))
            && !selector.contains("limit=1"))
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::Nbt",
    aliases = ["sand::cmd::Nbt", "sand::command::Nbt", "sand::prelude::Nbt", "sand::prelude::cmd::Nbt", "sand::state::Nbt"],
    module = "sand::data",
    summary = "Factory for discoverable typed NBT target construction.",
    context = "Factory for discoverable typed NBT target construction. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::Nbt;",
)]
/// Factory for discoverable typed NBT target construction.
pub struct Nbt;

impl Nbt {
    /// Starts an entity-backed NBT target from a typed selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::Nbt::entity",
        aliases = ["sand::cmd::Nbt::entity", "sand::command::Nbt::entity", "sand::prelude::Nbt::entity", "sand::prelude::cmd::Nbt::entity", "sand::state::Nbt::entity"],
        module = "sand::data",
        kind = "method",
        summary = "Starts an entity-backed NBT target from a typed selector.",
        context = "Starts an entity-backed NBT target from a typed selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(selector = "`selector` provides the Minecraft target selection used to start an entity-backed NBT target from a typed selector."),
        returns = "The `NbtTarget` value produced to start an entity-backed NBT target from a typed selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target)  {\n    let entity = sand::data::Nbt::entity(selector);\n}",
    )]
    pub fn entity(selector: impl TargetArgument) -> NbtTarget {
        NbtTarget::new(DataTarget::entity(selector))
    }

    /// Starts a block-backed NBT target from typed coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::Nbt::block",
        aliases = ["sand::cmd::Nbt::block", "sand::command::Nbt::block", "sand::prelude::Nbt::block", "sand::prelude::cmd::Nbt::block", "sand::state::Nbt::block"],
        module = "sand::data",
        kind = "method",
        summary = "Starts a block-backed NBT target from typed coordinates.",
        context = "Starts a block-backed NBT target from typed coordinates. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(position = "`position` is used to start a block-backed NBT target from typed coordinates."),
        returns = "The `NbtTarget` value produced to start a block-backed NBT target from typed coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(position: sand::command::BlockPos)  {\n    let block = sand::data::Nbt::block(position);\n}",
    )]
    pub fn block(position: BlockPos) -> NbtTarget {
        NbtTarget::new(DataTarget::block(position))
    }

    /// Starts a command-storage-backed NBT target from a namespaced identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::Nbt::storage",
        aliases = ["sand::cmd::Nbt::storage", "sand::command::Nbt::storage", "sand::prelude::Nbt::storage", "sand::prelude::cmd::Nbt::storage", "sand::state::Nbt::storage"],
        module = "sand::data",
        kind = "method",
        summary = "Starts a command-storage-backed NBT target from a namespaced identifier.",
        context = "Starts a command-storage-backed NBT target from a namespaced identifier. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(id = "`id` provides the typed resource identifier or location used to start a command-storage-backed NBT target from a namespaced identifier."),
        returns = "The `NbtTarget` value produced to start a command-storage-backed NBT target from a namespaced identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl Into < String >)  {\n    let storage = sand::data::Nbt::storage(id);\n}",
    )]
    pub fn storage(id: impl Into<String>) -> NbtTarget {
        NbtTarget::new(DataTarget::storage(id))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::NbtTarget",
    aliases = ["sand::cmd::NbtTarget", "sand::command::NbtTarget", "sand::prelude::NbtTarget", "sand::prelude::cmd::NbtTarget", "sand::state::NbtTarget"],
    module = "sand::data",
    summary = "An NBT location before a path is selected.",
    context = "An NBT location before a path is selected. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::NbtTarget;",
)]
/// An NBT location before a path is selected.
#[derive(Debug, Clone)]
pub struct NbtTarget {
    location: DataTarget,
}

impl NbtTarget {
    /// Wraps a concrete data-command location as an NBT target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtTarget::new",
        aliases = ["sand::cmd::NbtTarget::new", "sand::command::NbtTarget::new", "sand::prelude::NbtTarget::new", "sand::prelude::cmd::NbtTarget::new", "sand::state::NbtTarget::new"],
        module = "sand::data",
        kind = "method",
        summary = "Wraps a concrete data-command location as an NBT target.",
        context = "Wraps a concrete data-command location as an NBT target. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(location = "`location` provides the typed resource identifier or location used to wrap a concrete data-command location as an NBT target."),
        returns = "A `NbtTarget` wrapping a concrete data-command location as an NBT target.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::data::DataTarget)  {\n    let nbt_target = sand::data::NbtTarget::new(location);\n}",
    )]
    pub fn new(location: DataTarget) -> Self {
        Self { location }
    }

    /// Returns the typed NBT location targeted by this reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtTarget::location",
        aliases = ["sand::cmd::NbtTarget::location", "sand::command::NbtTarget::location", "sand::prelude::NbtTarget::location", "sand::prelude::cmd::NbtTarget::location", "sand::state::NbtTarget::location"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the typed NBT location targeted by this reference.",
        context = "Returns the typed NBT location targeted by this reference. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the typed NBT location targeted by this reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_target_value: &sand::data::NbtTarget)  {\n    let location = nbt_target_value.location();\n}",
    )]
    pub fn location(&self) -> &DataTarget {
        &self.location
    }

    /// Extends this typed NBT reference with the supplied path selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtTarget::path",
        aliases = ["sand::cmd::NbtTarget::path", "sand::command::NbtTarget::path", "sand::prelude::NbtTarget::path", "sand::prelude::cmd::NbtTarget::path", "sand::state::NbtTarget::path"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied path selector.",
        context = "Extends this typed NBT reference with the supplied path selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to extend this typed NBT reference with the supplied path selector."),
        returns = "The `NbtRef` value produced to extend this typed NBT reference with the supplied path selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_target_value: &sand::data::NbtTarget, path: impl Into < sand::data::NbtPath >)  {\n    let path = nbt_target_value.path(path);\n}",
    )]
    pub fn path(&self, path: impl Into<NbtPath>) -> NbtRef {
        NbtRef::new(self.location.clone(), path.into())
    }

    /// Extends this typed NBT reference with the supplied typed path selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtTarget::typed_path",
        aliases = ["sand::cmd::NbtTarget::typed_path", "sand::command::NbtTarget::typed_path", "sand::prelude::NbtTarget::typed_path", "sand::prelude::cmd::NbtTarget::typed_path", "sand::state::NbtTarget::typed_path"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied typed path selector.",
        context = "Extends this typed NBT reference with the supplied typed path selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to extend this typed NBT reference with the supplied typed path selector."),
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied typed path selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_target_value: &sand::data::NbtTarget, path: impl Into < sand::data::NbtPath >)  {\n    let typed_path = nbt_target_value.typed_path::<T>(path);\n}",
    )]
    pub fn typed_path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(self.location.clone(), path.into())
    }

    /// Builds the typed Minecraft data modification for merge.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtTarget::merge",
        aliases = ["sand::cmd::NbtTarget::merge", "sand::command::NbtTarget::merge", "sand::prelude::NbtTarget::merge", "sand::prelude::cmd::NbtTarget::merge", "sand::state::NbtTarget::merge"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for merge.",
        context = "Builds the typed Minecraft data modification for merge. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for merge."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for merge.",
        example = "use sand::prelude::*;\n\nfn demonstrate(nbt_target_value: &sand::data::NbtTarget, value: sand::data::NbtCompound)  {\n    let merge = nbt_target_value.merge(value);\n}",
    )]
    pub fn merge(&self, value: NbtCompound) -> DataCommand {
        self.location.merge(value)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::UntypedNbt",
    aliases = ["sand::cmd::UntypedNbt", "sand::command::UntypedNbt", "sand::prelude::UntypedNbt", "sand::prelude::cmd::UntypedNbt", "sand::state::UntypedNbt"],
    module = "sand::data",
    summary = "Marker for an NBT reference without a declared schema value type.",
    context = "Marker for an NBT reference without a declared schema value type. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::UntypedNbt;",
)]
/// Marker for an NBT reference without a declared schema value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UntypedNbt;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::NbtRef",
    aliases = ["sand::cmd::NbtRef", "sand::command::NbtRef", "sand::prelude::NbtRef", "sand::prelude::cmd::NbtRef", "sand::state::NbtRef"],
    module = "sand::data",
    summary = "A typed target-plus-path reference shared by storage, entity, block, schema, and inventory APIs.",
    context = "A typed target-plus-path reference shared by storage, entity, block, schema, and inventory APIs. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::NbtRef;",
)]
/// A typed target-plus-path reference shared by storage, entity, block, schema,
/// and inventory APIs.
#[derive(Debug, Clone)]
pub struct NbtRef<T = UntypedNbt> {
    location: DataTarget,
    path: NbtPath,
    marker: PhantomData<fn() -> T>,
}

impl<T> NbtRef<T> {
    /// Creates a typed NBT reference from a target and structured path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::new",
        aliases = ["sand::cmd::NbtRef::new", "sand::command::NbtRef::new", "sand::prelude::NbtRef::new", "sand::prelude::cmd::NbtRef::new", "sand::state::NbtRef::new"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a typed NBT reference from a target and structured path.",
        context = "Creates a typed NBT reference from a target and structured path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(location = "`location` provides the typed resource identifier or location used to create a typed NBT reference from a target and structured path.", path = "`path` provides the typed resource identifier or location used to create a typed NBT reference from a target and structured path."),
        returns = "A `NbtRef` representing a typed NBT reference from a target and structured path.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(location: sand::data::DataTarget, path: sand::data::NbtPath)  {\n    let nbt_ref = sand::data::NbtRef ::< T >::new(location, path);\n}",
    )]
    pub fn new(location: DataTarget, path: NbtPath) -> Self {
        Self {
            location,
            path,
            marker: PhantomData,
        }
    }

    /// Returns the typed NBT location targeted by this reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::location",
        aliases = ["sand::cmd::NbtRef::location", "sand::command::NbtRef::location", "sand::prelude::NbtRef::location", "sand::prelude::cmd::NbtRef::location", "sand::state::NbtRef::location"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the typed NBT location targeted by this reference.",
        context = "Returns the typed NBT location targeted by this reference. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the typed NBT location targeted by this reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >)  {\n    let location = nbt_ref_value.location();\n}",
    )]
    pub fn location(&self) -> &DataTarget {
        &self.location
    }

    /// Returns the typed NBT path carried by this reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::path_value",
        aliases = ["sand::cmd::NbtRef::path_value", "sand::command::NbtRef::path_value", "sand::prelude::NbtRef::path_value", "sand::prelude::cmd::NbtRef::path_value", "sand::state::NbtRef::path_value"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the typed NBT path carried by this reference.",
        context = "Returns the typed NBT path carried by this reference. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the typed NBT path carried by this reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >)  {\n    let path_value = nbt_ref_value.path_value();\n}",
    )]
    pub fn path_value(&self) -> &NbtPath {
        &self.path
    }

    /// The path text, without its location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::as_str",
        aliases = ["sand::cmd::NbtRef::as_str", "sand::command::NbtRef::as_str", "sand::prelude::NbtRef::as_str", "sand::prelude::cmd::NbtRef::as_str", "sand::state::NbtRef::as_str"],
        module = "sand::data",
        kind = "method",
        summary = "The path text, without its location.",
        context = "The path text, without its location. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to use the path text, without its location.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >)  {\n    let as_str = nbt_ref_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }

    /// Compatibility accessor for storage-backed references.
    ///
    /// New generic code should inspect [`location`](Self::location) because
    /// entity and block references intentionally have no storage ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::storage",
        aliases = ["sand::cmd::NbtRef::storage", "sand::command::NbtRef::storage", "sand::prelude::NbtRef::storage", "sand::prelude::cmd::NbtRef::storage", "sand::state::NbtRef::storage"],
        module = "sand::data",
        kind = "method",
        summary = "Compatibility accessor for storage-backed references.",
        context = "Compatibility accessor for storage-backed references. New generic code should inspect [`location`](Self::location) because entity and block references intentionally have no storage ID.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to use compatibility accessor for storage-backed references.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >)  {\n    let storage = nbt_ref_value.storage();\n}",
    )]
    pub fn storage(&self) -> &str {
        match &self.location {
            DataTarget::Storage(id) => id,
            _ => "",
        }
    }

    /// Extends this typed NBT reference with the supplied field selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::field",
        aliases = ["sand::cmd::NbtRef::field", "sand::command::NbtRef::field", "sand::prelude::NbtRef::field", "sand::prelude::cmd::NbtRef::field", "sand::state::NbtRef::field"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied field selector.",
        context = "Extends this typed NBT reference with the supplied field selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to extend this typed NBT reference with the supplied field selector."),
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied field selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, key: impl AsRef < str >)  {\n    let field = nbt_ref_value.field(key);\n}",
    )]
    pub fn field(&self, key: impl AsRef<str>) -> NbtRef<T> {
        NbtRef::new(self.location.clone(), self.path.field(key))
    }

    /// Extends this typed NBT reference with the supplied typed field selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::typed_field",
        aliases = ["sand::cmd::NbtRef::typed_field", "sand::command::NbtRef::typed_field", "sand::prelude::NbtRef::typed_field", "sand::prelude::cmd::NbtRef::typed_field", "sand::state::NbtRef::typed_field"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied typed field selector.",
        context = "Extends this typed NBT reference with the supplied typed field selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to extend this typed NBT reference with the supplied typed field selector."),
        returns = "The `NbtRef < U >` value produced to extend this typed NBT reference with the supplied typed field selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, key: impl AsRef < str >)  {\n    let typed_field = nbt_ref_value.typed_field::<U>(key);\n}",
    )]
    pub fn typed_field<U>(&self, key: impl AsRef<str>) -> NbtRef<U> {
        NbtRef::new(self.location.clone(), self.path.field(key))
    }

    /// Extends this typed NBT reference with the supplied key selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::key",
        aliases = ["sand::cmd::NbtRef::key", "sand::command::NbtRef::key", "sand::prelude::NbtRef::key", "sand::prelude::cmd::NbtRef::key", "sand::state::NbtRef::key"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied key selector.",
        context = "Extends this typed NBT reference with the supplied key selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(key = "`key` provides the key that identifies the setting or entry used to extend this typed NBT reference with the supplied key selector."),
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied key selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, key: impl AsRef < str >)  {\n    let key = nbt_ref_value.key(key);\n}",
    )]
    pub fn key(&self, key: impl AsRef<str>) -> NbtRef<T> {
        self.field(key)
    }

    /// Extends this typed NBT reference with the supplied index selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::index",
        aliases = ["sand::cmd::NbtRef::index", "sand::command::NbtRef::index", "sand::prelude::NbtRef::index", "sand::prelude::cmd::NbtRef::index", "sand::state::NbtRef::index"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied index selector.",
        context = "Extends this typed NBT reference with the supplied index selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(index = "`index` is used to extend this typed NBT reference with the supplied index selector."),
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied index selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, index: i32)  {\n    let index = nbt_ref_value.index(index);\n}",
    )]
    pub fn index(&self, index: i32) -> NbtRef<T> {
        NbtRef::new(self.location.clone(), self.path.index(index))
    }

    /// Builds the typed Minecraft data query for get.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::get",
        aliases = ["sand::cmd::NbtRef::get", "sand::command::NbtRef::get", "sand::prelude::NbtRef::get", "sand::prelude::cmd::NbtRef::get", "sand::state::NbtRef::get"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for get.",
        context = "Builds the typed Minecraft data query for get. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `DataCommand` value produced to build the typed Minecraft data query for get.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >)  {\n    let get = nbt_ref_value.get();\n}",
    )]
    pub fn get(&self) -> DataCommand {
        DataCommand::Get {
            source: self.untyped(),
            scale: None,
        }
    }

    /// Builds the typed Minecraft data query for get scaled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::get_scaled",
        aliases = ["sand::cmd::NbtRef::get_scaled", "sand::command::NbtRef::get_scaled", "sand::prelude::NbtRef::get_scaled", "sand::prelude::cmd::NbtRef::get_scaled", "sand::state::NbtRef::get_scaled"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for get scaled.",
        context = "Builds the typed Minecraft data query for get scaled. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(scale = "`scale` provides the scale used to build the typed Minecraft data query for get scaled."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data query for get scaled.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, scale: f64)  {\n    let get_scaled = nbt_ref_value.get_scaled(scale);\n}",
    )]
    pub fn get_scaled(&self, scale: f64) -> DataCommand {
        DataCommand::Get {
            source: self.untyped(),
            scale: Some(scale),
        }
    }

    /// Builds the typed Minecraft data modification for set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set",
        aliases = ["sand::cmd::NbtRef::set", "sand::command::NbtRef::set", "sand::prelude::NbtRef::set", "sand::prelude::cmd::NbtRef::set", "sand::state::NbtRef::set"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set.",
        context = "Builds the typed Minecraft data modification for set. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for set.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < sand::data::NbtValue >)  {\n    let set = nbt_ref_value.set(value);\n}",
    )]
    pub fn set(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(DataModifyOperation::Set, DataSource::Value(value.into()))
    }

    /// Builds the typed Minecraft data modification for set value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set_value",
        aliases = ["sand::cmd::NbtRef::set_value", "sand::command::NbtRef::set_value", "sand::prelude::NbtRef::set_value", "sand::prelude::cmd::NbtRef::set_value", "sand::state::NbtRef::set_value"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set value.",
        context = "Builds the typed Minecraft data modification for set value. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set value."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for set value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < sand::data::NbtValue >)  {\n    let set_value = nbt_ref_value.set_value(value);\n}",
    )]
    pub fn set_value(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.set(value)
    }

    /// Builds the typed Minecraft data modification for set int.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set_int",
        aliases = ["sand::cmd::NbtRef::set_int", "sand::command::NbtRef::set_int", "sand::prelude::NbtRef::set_int", "sand::prelude::cmd::NbtRef::set_int", "sand::state::NbtRef::set_int"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set int.",
        context = "Builds the typed Minecraft data modification for set int. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set int."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for set int.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: i32)  {\n    let set_int = nbt_ref_value.set_int(value);\n}",
    )]
    pub fn set_int(&self, value: i32) -> DataCommand {
        self.set(value)
    }

    /// Builds the typed Minecraft data modification for set bool.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set_bool",
        aliases = ["sand::cmd::NbtRef::set_bool", "sand::command::NbtRef::set_bool", "sand::prelude::NbtRef::set_bool", "sand::prelude::cmd::NbtRef::set_bool", "sand::state::NbtRef::set_bool"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set bool.",
        context = "Builds the typed Minecraft data modification for set bool. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set bool."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for set bool.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: bool)  {\n    let set_bool = nbt_ref_value.set_bool(value);\n}",
    )]
    pub fn set_bool(&self, value: bool) -> DataCommand {
        self.set(value)
    }

    /// Builds the typed Minecraft data modification for set string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set_string",
        aliases = ["sand::cmd::NbtRef::set_string", "sand::command::NbtRef::set_string", "sand::prelude::NbtRef::set_string", "sand::prelude::cmd::NbtRef::set_string", "sand::state::NbtRef::set_string"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set string.",
        context = "Builds the typed Minecraft data modification for set string. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set string."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for set string.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < String >)  {\n    let set_string = nbt_ref_value.set_string(value);\n}",
    )]
    pub fn set_string(&self, value: impl Into<String>) -> DataCommand {
        self.set(NbtValue::String(value.into()))
    }

    /// Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set_raw",
        aliases = ["sand::cmd::NbtRef::set_raw", "sand::command::NbtRef::set_raw", "sand::prelude::NbtRef::set_raw", "sand::prelude::cmd::NbtRef::set_raw", "sand::state::NbtRef::set_raw"],
        module = "sand::data",
        kind = "method",
        summary = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        context = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility."),
        returns = "The `DataCommand` value produced to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < String >)  {\n    let set_raw = nbt_ref_value.set_raw(value);\n}",
    )]
    pub fn set_raw(&self, value: impl Into<String>) -> DataCommand {
        self.set(NbtValue::raw(value))
    }

    /// Builds the typed Minecraft data modification for copy from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::copy_from",
        aliases = ["sand::cmd::NbtRef::copy_from", "sand::command::NbtRef::copy_from", "sand::prelude::NbtRef::copy_from", "sand::prelude::cmd::NbtRef::copy_from", "sand::state::NbtRef::copy_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for copy from.",
        context = "Builds the typed Minecraft data modification for copy from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source = "`source` provides the source used to build the typed Minecraft data modification for copy from."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for copy from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, source: & sand::data::NbtRef < U >)  {\n    let copy_from = nbt_ref_value.copy_from::<U>(source);\n}",
    )]
    pub fn copy_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(DataModifyOperation::Set, DataSource::From(source.untyped()))
    }

    /// Builds the typed Minecraft data modification for set string from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::set_string_from",
        aliases = ["sand::cmd::NbtRef::set_string_from", "sand::command::NbtRef::set_string_from", "sand::prelude::NbtRef::set_string_from", "sand::prelude::cmd::NbtRef::set_string_from", "sand::state::NbtRef::set_string_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set string from.",
        context = "Builds the typed Minecraft data modification for set string from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source = "`source` provides the source used to build the typed Minecraft data modification for set string from."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for set string from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, source: & sand::data::NbtRef < U >)  {\n    let set_string_from = nbt_ref_value.set_string_from::<U>(source);\n}",
    )]
    pub fn set_string_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Set,
            DataSource::String(source.untyped()),
        )
    }

    /// Builds the typed Minecraft data modification for append.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::append",
        aliases = ["sand::cmd::NbtRef::append", "sand::command::NbtRef::append", "sand::prelude::NbtRef::append", "sand::prelude::cmd::NbtRef::append", "sand::state::NbtRef::append"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for append.",
        context = "Builds the typed Minecraft data modification for append. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for append."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for append.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < sand::data::NbtValue >)  {\n    let append = nbt_ref_value.append(value);\n}",
    )]
    pub fn append(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(DataModifyOperation::Append, DataSource::Value(value.into()))
    }

    /// Builds the typed Minecraft data modification for append from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::append_from",
        aliases = ["sand::cmd::NbtRef::append_from", "sand::command::NbtRef::append_from", "sand::prelude::NbtRef::append_from", "sand::prelude::cmd::NbtRef::append_from", "sand::state::NbtRef::append_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for append from.",
        context = "Builds the typed Minecraft data modification for append from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source = "`source` provides the source used to build the typed Minecraft data modification for append from."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for append from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, source: & sand::data::NbtRef < U >)  {\n    let append_from = nbt_ref_value.append_from::<U>(source);\n}",
    )]
    pub fn append_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Append,
            DataSource::From(source.untyped()),
        )
    }

    /// Builds the typed Minecraft data modification for prepend.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::prepend",
        aliases = ["sand::cmd::NbtRef::prepend", "sand::command::NbtRef::prepend", "sand::prelude::NbtRef::prepend", "sand::prelude::cmd::NbtRef::prepend", "sand::state::NbtRef::prepend"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for prepend.",
        context = "Builds the typed Minecraft data modification for prepend. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for prepend."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for prepend.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < sand::data::NbtValue >)  {\n    let prepend = nbt_ref_value.prepend(value);\n}",
    )]
    pub fn prepend(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(
            DataModifyOperation::Prepend,
            DataSource::Value(value.into()),
        )
    }

    /// Builds the typed Minecraft data modification for prepend from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::prepend_from",
        aliases = ["sand::cmd::NbtRef::prepend_from", "sand::command::NbtRef::prepend_from", "sand::prelude::NbtRef::prepend_from", "sand::prelude::cmd::NbtRef::prepend_from", "sand::state::NbtRef::prepend_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for prepend from.",
        context = "Builds the typed Minecraft data modification for prepend from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source = "`source` provides the source used to build the typed Minecraft data modification for prepend from."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for prepend from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, source: & sand::data::NbtRef < U >)  {\n    let prepend_from = nbt_ref_value.prepend_from::<U>(source);\n}",
    )]
    pub fn prepend_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Prepend,
            DataSource::From(source.untyped()),
        )
    }

    /// Builds the typed Minecraft data modification for insert.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::insert",
        aliases = ["sand::cmd::NbtRef::insert", "sand::command::NbtRef::insert", "sand::prelude::NbtRef::insert", "sand::prelude::cmd::NbtRef::insert", "sand::state::NbtRef::insert"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for insert.",
        context = "Builds the typed Minecraft data modification for insert. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(index = "`index` provides the index used to build the typed Minecraft data modification for insert.", value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for insert."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for insert.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, index: i32, value: impl Into < sand::data::NbtValue >)  {\n    let insert = nbt_ref_value.insert(index, value);\n}",
    )]
    pub fn insert(&self, index: i32, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(
            DataModifyOperation::Insert(index),
            DataSource::Value(value.into()),
        )
    }

    /// Builds the typed Minecraft data modification for insert from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::insert_from",
        aliases = ["sand::cmd::NbtRef::insert_from", "sand::command::NbtRef::insert_from", "sand::prelude::NbtRef::insert_from", "sand::prelude::cmd::NbtRef::insert_from", "sand::state::NbtRef::insert_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for insert from.",
        context = "Builds the typed Minecraft data modification for insert from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(index = "`index` provides the index used to build the typed Minecraft data modification for insert from.", source = "`source` provides the source used to build the typed Minecraft data modification for insert from."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for insert from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, index: i32, source: & sand::data::NbtRef < U >)  {\n    let insert_from = nbt_ref_value.insert_from::<U>(index, source);\n}",
    )]
    pub fn insert_from<U>(&self, index: i32, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Insert(index),
            DataSource::From(source.untyped()),
        )
    }

    /// Builds the typed Minecraft data modification for merge.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::merge",
        aliases = ["sand::cmd::NbtRef::merge", "sand::command::NbtRef::merge", "sand::prelude::NbtRef::merge", "sand::prelude::cmd::NbtRef::merge", "sand::state::NbtRef::merge"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for merge.",
        context = "Builds the typed Minecraft data modification for merge. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for merge."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for merge.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, value: impl Into < sand::data::NbtValue >)  {\n    let merge = nbt_ref_value.merge(value);\n}",
    )]
    pub fn merge(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(DataModifyOperation::Merge, DataSource::Value(value.into()))
    }

    /// Builds the typed Minecraft data modification for merge from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::merge_from",
        aliases = ["sand::cmd::NbtRef::merge_from", "sand::command::NbtRef::merge_from", "sand::prelude::NbtRef::merge_from", "sand::prelude::cmd::NbtRef::merge_from", "sand::state::NbtRef::merge_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for merge from.",
        context = "Builds the typed Minecraft data modification for merge from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source = "`source` provides the source used to build the typed Minecraft data modification for merge from."),
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for merge from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(nbt_ref_value: &sand::data::NbtRef < T >, source: & sand::data::NbtRef < U >)  {\n    let merge_from = nbt_ref_value.merge_from::<U>(source);\n}",
    )]
    pub fn merge_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Merge,
            DataSource::From(source.untyped()),
        )
    }

    /// Builds the typed Minecraft data modification for remove.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::NbtRef::remove",
        aliases = ["sand::cmd::NbtRef::remove", "sand::command::NbtRef::remove", "sand::prelude::NbtRef::remove", "sand::prelude::cmd::NbtRef::remove", "sand::state::NbtRef::remove"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for remove.",
        context = "Builds the typed Minecraft data modification for remove. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `DataCommand` value produced to build the typed Minecraft data modification for remove.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(nbt_ref_value: &sand::data::NbtRef < T >)  {\n    let remove = nbt_ref_value.remove();\n}",
    )]
    pub fn remove(&self) -> DataCommand {
        DataCommand::Remove {
            target: self.untyped(),
        }
    }

    fn modify(&self, operation: DataModifyOperation, source: DataSource) -> DataCommand {
        DataCommand::Modify {
            target: self.untyped(),
            operation,
            source,
        }
    }

    fn untyped(&self) -> NbtRef {
        NbtRef::new(self.location.clone(), self.path.clone())
    }
}

// ── Typed data command IR ────────────────────────────────────────────────────

#[doc = "Defines data modify operation for typed Minecraft NBT and data commands."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::DataModifyOperation",
    aliases = ["sand::cmd::DataModifyOperation", "sand::command::DataModifyOperation", "sand::prelude::cmd::DataModifyOperation"],
    module = "sand::data",
    summary = "Defines data modify operation for typed Minecraft NBT and data commands.",
    context = "Defines data modify operation for typed Minecraft NBT and data commands. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::DataModifyOperation;",
    variants(Append = "Selects the append NBT or data-command operation.", Insert = "Selects the insert NBT or data-command operation.", Merge = "Selects the merge NBT or data-command operation.", Prepend = "Selects the prepend NBT or data-command operation.", Set = "Selects the set NBT or data-command operation."),
    variant_fields(Insert = ["Selects the insert NBT or data-command operation."]),
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataModifyOperation {
    #[doc = "Selects the set NBT or data-command operation."]
    Set,
    #[doc = "Selects the append NBT or data-command operation."]
    Append,
    #[doc = "Selects the prepend NBT or data-command operation."]
    Prepend,
    #[doc = "Selects the insert NBT or data-command operation."]
    Insert(#[doc = "Selects the insert NBT or data-command operation."] i32),
    #[doc = "Selects the merge NBT or data-command operation."]
    Merge,
}

#[doc = "Defines data source for typed Minecraft NBT and data commands."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::DataSource",
    aliases = ["sand::cmd::DataSource", "sand::command::DataSource", "sand::prelude::cmd::DataSource"],
    module = "sand::data",
    summary = "Defines data source for typed Minecraft NBT and data commands.",
    context = "Defines data source for typed Minecraft NBT and data commands. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::DataSource;",
    variants(From = "Selects the from NBT or data-command operation.", String = "Selects the string NBT or data-command operation.", Value = "Selects the value NBT or data-command operation."),
    variant_fields(From = ["Selects the from NBT or data-command operation."], String = ["Selects the string NBT or data-command operation."], Value = ["Selects the value NBT or data-command operation."]),
)]
#[derive(Debug, Clone)]
pub enum DataSource {
    #[doc = "Selects the value NBT or data-command operation."]
    Value(#[doc = "Selects the value NBT or data-command operation."] NbtValue),
    #[doc = "Selects the from NBT or data-command operation."]
    From(#[doc = "Selects the from NBT or data-command operation."] NbtRef),
    #[doc = "Selects the string NBT or data-command operation."]
    String(#[doc = "Selects the string NBT or data-command operation."] NbtRef),
}

#[doc = "Defines data command for typed Minecraft NBT and data commands."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::DataCommand",
    aliases = ["sand::cmd::DataCommand", "sand::command::DataCommand", "sand::prelude::DataCommand", "sand::prelude::cmd::DataCommand", "sand::state::DataCommand"],
    module = "sand::data",
    summary = "Defines data command for typed Minecraft NBT and data commands.",
    context = "Defines data command for typed Minecraft NBT and data commands. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::DataCommand;",
    variants(Get = "Selects the get NBT or data-command operation.", Merge = "Selects the merge NBT or data-command operation.", Modify = "Selects the modify NBT or data-command operation.", Remove = "Selects the remove NBT or data-command operation."),
    variant_fields(Get(scale = "`scale` optionally supplies the particle scale for the data get operation.", source = "`source` supplies the source for the data get operation."), Merge(target = "`target` supplies the command target for the data merge operation.", value = "`value` supplies the value for the data merge operation."), Modify(operation = "`operation` supplies the operation for the data modify operation.", source = "`source` supplies the source for the data modify operation.", target = "`target` supplies the command target for the data modify operation."), Remove(target = "`target` supplies the command target for the data remove operation.")),
)]
#[derive(Debug, Clone)]
pub enum DataCommand {
    #[doc = "Selects the get NBT or data-command operation."]
    Get {
        /// `source` supplies the source for the data get operation.
        source: NbtRef,
        /// `scale` optionally supplies the particle scale for the data get operation.
        scale: Option<f64>,
    },
    #[doc = "Selects the remove NBT or data-command operation."]
    Remove {
        /// `target` supplies the command target for the data remove operation.
        target: NbtRef,
    },
    #[doc = "Selects the modify NBT or data-command operation."]
    Modify {
        /// `target` supplies the command target for the data modify operation.
        target: NbtRef,
        /// `operation` supplies the operation for the data modify operation.
        operation: DataModifyOperation,
        /// `source` supplies the source for the data modify operation.
        source: DataSource,
    },
    #[doc = "Selects the merge NBT or data-command operation."]
    Merge {
        /// `target` supplies the command target for the data merge operation.
        target: DataTarget,
        /// `value` supplies the value for the data merge operation.
        value: NbtCompound,
    },
}

impl DataCommand {
    /// Validates and renders this typed Minecraft data command for the selected command profile.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataCommand::try_render",
        aliases = ["sand::cmd::DataCommand::try_render", "sand::command::DataCommand::try_render", "sand::prelude::DataCommand::try_render", "sand::prelude::cmd::DataCommand::try_render", "sand::state::DataCommand::try_render"],
        module = "sand::data",
        kind = "method",
        summary = "Validates and renders this typed Minecraft data command for the selected command profile.",
        context = "Validates and renders this typed Minecraft data command for the selected command profile. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(profile = "`profile` is the profile checked when validating and renders this typed Minecraft data command for the selected command profile."),
        returns = "On success, the value produced to validate and renders this typed Minecraft data command for the selected command profile; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_command_value: &sand::data::DataCommand, profile: & sand::command::CommandProfile)  {\n    let try_render = data_command_value.try_render(profile);\n}",
    )]
    pub fn try_render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        let rendered = self.render_unchecked(profile);
        register_line(&rendered, self.clone());
        Ok(rendered)
    }

    /// Compatibility convenience for assertions on rendered command text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::DataCommand::contains",
        aliases = ["sand::cmd::DataCommand::contains", "sand::command::DataCommand::contains", "sand::prelude::DataCommand::contains", "sand::prelude::cmd::DataCommand::contains", "sand::state::DataCommand::contains"],
        module = "sand::data",
        kind = "method",
        summary = "Compatibility convenience for assertions on rendered command text.",
        context = "Compatibility convenience for assertions on rendered command text. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(pattern = "`pattern` sets the pattern for compatibility convenience for assertions on rendered command text."),
        returns = "`true` when the documented condition holds to use compatibility convenience for assertions on rendered command text; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_command_value: &sand::data::DataCommand, pattern: & str)  {\n    let is_contains = data_command_value.contains(pattern);\n}",
    )]
    pub fn contains(&self, pattern: &str) -> bool {
        self.to_string().contains(pattern)
    }
}

impl Validate for DataCommand {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Get { source, scale } => {
                validate_ref(source, false)?;
                if scale.is_some_and(|value| !value.is_finite()) {
                    return Err(data_error("scale", "data get scale must be finite"));
                }
            }
            Self::Remove { target } => validate_ref(target, true)?,
            Self::Modify { target, source, .. } => {
                validate_ref(target, true)?;
                validate_source(source)?;
            }
            Self::Merge { target, value } => {
                target.validate(true)?;
                validate_compound(value)?;
                if value.is_empty() {
                    return Err(data_error(
                        "value",
                        "data merge requires a non-empty compound",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn validate_ref(reference: &NbtRef, write: bool) -> CommandResult<()> {
    reference.location.validate(write)?;
    reference.path.validate()?;
    if write
        && matches!(reference.location, DataTarget::Entity(_))
        && matches!(
            reference
                .path
                .as_str()
                .split(['.', '['])
                .next()
                .unwrap_or_default(),
            "Inventory" | "SelectedItem" | "EnderItems"
        )
    {
        return Err(data_error(
            "target",
            format!(
                "entity inventory path `{}` is not a safe `/data` write target; use a typed item location and `/item replace`",
                reference.path
            ),
        ));
    }
    Ok(())
}

fn validate_source(source: &DataSource) -> CommandResult<()> {
    match source {
        DataSource::Value(value) => validate_value(value),
        DataSource::From(reference) | DataSource::String(reference) => {
            validate_ref(reference, false)
        }
    }
}

fn validate_value(value: &NbtValue) -> CommandResult<()> {
    match value {
        NbtValue::Float(value) if !value.is_finite() => {
            Err(data_error("value", "SNBT floats must be finite"))
        }
        NbtValue::Double(value) if !value.is_finite() => {
            Err(data_error("value", "SNBT doubles must be finite"))
        }
        NbtValue::String(value) if value.chars().any(char::is_control) => Err(data_error(
            "value",
            "typed SNBT strings cannot contain control characters",
        )),
        NbtValue::List(values) => {
            for value in values {
                validate_value(value)?;
            }
            Ok(())
        }
        NbtValue::Compound(value) => validate_compound(value),
        _ => Ok(()),
    }
}

fn validate_compound(compound: &NbtCompound) -> CommandResult<()> {
    for (key, value) in &compound.entries {
        if key.is_empty() || key.chars().any(char::is_control) {
            return Err(data_error(
                "value",
                "typed SNBT compound keys must be non-empty and contain no control characters",
            ));
        }
        validate_value(value)?;
    }
    Ok(())
}

fn render_source(source: &DataSource) -> String {
    match source {
        DataSource::Value(value) => format!("value {value}"),
        DataSource::From(reference) => {
            format!("from {} {}", reference.location, reference.path)
        }
        DataSource::String(reference) => {
            format!("string {} {}", reference.location, reference.path)
        }
    }
}

impl RenderCommand for DataCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        match self {
            Self::Get { source, scale } => match scale {
                Some(scale) => format!("data get {} {} {scale}", source.location, source.path),
                None => format!("data get {} {}", source.location, source.path),
            },
            Self::Remove { target } => {
                format!("data remove {} {}", target.location, target.path)
            }
            Self::Modify {
                target,
                operation,
                source,
            } => {
                let operation = match operation {
                    DataModifyOperation::Set => "set".to_owned(),
                    DataModifyOperation::Append => "append".to_owned(),
                    DataModifyOperation::Prepend => "prepend".to_owned(),
                    DataModifyOperation::Insert(index) => format!("insert {index}"),
                    DataModifyOperation::Merge => "merge".to_owned(),
                };
                format!(
                    "data modify {} {} {operation} {}",
                    target.location,
                    target.path,
                    render_source(source)
                )
            }
            Self::Merge { target, value } => format!("data merge {target} {value}"),
        }
    }
}

impl fmt::Display for DataCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&rendered, self.clone());
        f.write_str(&rendered)
    }
}

impl Build for DataCommand {
    fn build(&self) -> String {
        self.to_string()
    }
}

impl From<DataCommand> for String {
    fn from(value: DataCommand) -> Self {
        value.to_string()
    }
}

// Compatibility comparisons intentionally render at the final command
// boundary; DataCommand's structural fields do not have a string view.
#[allow(clippy::cmp_owned)]
impl PartialEq<str> for DataCommand {
    fn eq(&self, other: &str) -> bool {
        self.to_string() == other
    }
}

#[allow(clippy::cmp_owned)]
impl PartialEq<&str> for DataCommand {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

fn data_error(field: &'static str, message: impl Into<String>) -> CommandError {
    CommandError::new("DataCommand", field, message).with_code("SAND-DATA-TARGET")
}

// Retain validation metadata after compatibility rendering to String.
/// Export-scoped registry family holding this module's rendered
/// `data` command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct DataLines;

impl crate::export_registry::RegistryFamily for DataLines {
    type State = BTreeMap<String, DataCommand>;
}

fn register_line(line: &str, command: DataCommand) {
    crate::export_registry::register_line::<DataLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<DataLines, _>(
        line,
        profile,
        |command, profile| command.try_render(profile).map(|_| ()),
    )
}

// ── Compatibility DataModify builder ────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::DataModify",
    aliases = ["sand::cmd::DataModify", "sand::prelude::cmd::DataModify"],
    module = "sand::command",
    summary = "Compatibility adapter over [`NbtRef`]. New code should start from [`Nbt`].",
    context = "Compatibility adapter over [`NbtRef`]. New code should start from [`Nbt`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::DataModify;",
)]
/// Compatibility adapter over [`NbtRef`]. New code should start from [`Nbt`].
#[derive(Debug, Clone)]
pub struct DataModify {
    reference: NbtRef,
}

impl DataModify {
    /// Creates a typed data modify command builder from the supplied command inputs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::new",
        aliases = ["sand::cmd::DataModify::new", "sand::prelude::cmd::DataModify::new"],
        module = "sand::command",
        kind = "method",
        summary = "Creates a typed data modify command builder from the supplied command inputs.",
        context = "Creates a typed data modify command builder from the supplied command inputs. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to create a typed data modify command builder from the supplied command inputs.", path = "`path` provides the typed resource identifier or location used to create a typed data modify command builder from the supplied command inputs."),
        returns = "A `DataModify` representing a typed data modify command builder from the supplied command inputs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::data::DataTarget, path: impl Into < sand::data::NbtPath >)  {\n    let data_modify = sand::command::DataModify::new(target, path);\n}",
    )]
    pub fn new(target: DataTarget, path: impl Into<NbtPath>) -> Self {
        Self {
            reference: NbtRef::new(target, path.into()),
        }
    }

    /// Renders the Minecraft set command for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::set",
        aliases = ["sand::cmd::DataModify::set", "sand::prelude::cmd::DataModify::set"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft set command for the selected data modify.",
        context = "Renders the Minecraft set command for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to render the Minecraft set command for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft set command for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, value: impl Into < sand::data::NbtValue >)  {\n    let command = data_modify_value.set(value);\n}",
    )]
    pub fn set(self, value: impl Into<NbtValue>) -> String {
        self.reference.set(value).to_string()
    }

    /// Renders the Minecraft command that sets from for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::set_from",
        aliases = ["sand::cmd::DataModify::set_from", "sand::prelude::cmd::DataModify::set_from"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets from for the selected data modify.",
        context = "Renders the Minecraft command that sets from for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(source = "`source` provides the source rendered when the Minecraft command that sets from for the selected data modify.", source_path = "`source_path` provides the source path rendered when the Minecraft command that sets from for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets from for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, source: sand::data::DataTarget, source_path: impl Into < sand::data::NbtPath >)  {\n    let command = data_modify_value.set_from(source, source_path);\n}",
    )]
    pub fn set_from(self, source: DataTarget, source_path: impl Into<NbtPath>) -> String {
        self.reference
            .copy_from(&NbtRef::<UntypedNbt>::new(source, source_path.into()))
            .to_string()
    }

    /// Renders the Minecraft append command for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::append",
        aliases = ["sand::cmd::DataModify::append", "sand::prelude::cmd::DataModify::append"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft append command for the selected data modify.",
        context = "Renders the Minecraft append command for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to render the Minecraft append command for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft append command for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, value: impl Into < sand::data::NbtValue >)  {\n    let command = data_modify_value.append(value);\n}",
    )]
    pub fn append(self, value: impl Into<NbtValue>) -> String {
        self.reference.append(value).to_string()
    }

    /// Renders the Minecraft append from command for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::append_from",
        aliases = ["sand::cmd::DataModify::append_from", "sand::prelude::cmd::DataModify::append_from"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft append from command for the selected data modify.",
        context = "Renders the Minecraft append from command for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(source = "`source` provides the source rendered when the Minecraft append from command for the selected data modify.", source_path = "`source_path` provides the source path rendered when the Minecraft append from command for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft append from command for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, source: sand::data::DataTarget, source_path: impl Into < sand::data::NbtPath >)  {\n    let command = data_modify_value.append_from(source, source_path);\n}",
    )]
    pub fn append_from(self, source: DataTarget, source_path: impl Into<NbtPath>) -> String {
        self.reference
            .append_from(&NbtRef::<UntypedNbt>::new(source, source_path.into()))
            .to_string()
    }

    /// Renders the Minecraft prepend command for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::prepend",
        aliases = ["sand::cmd::DataModify::prepend", "sand::prelude::cmd::DataModify::prepend"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft prepend command for the selected data modify.",
        context = "Renders the Minecraft prepend command for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to render the Minecraft prepend command for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft prepend command for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, value: impl Into < sand::data::NbtValue >)  {\n    let command = data_modify_value.prepend(value);\n}",
    )]
    pub fn prepend(self, value: impl Into<NbtValue>) -> String {
        self.reference.prepend(value).to_string()
    }

    /// Renders the Minecraft insert command for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::insert",
        aliases = ["sand::cmd::DataModify::insert", "sand::prelude::cmd::DataModify::insert"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft insert command for the selected data modify.",
        context = "Renders the Minecraft insert command for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(index = "`index` provides the index rendered when the Minecraft insert command for the selected data modify.", value = "`value` provides the value being applied or compared used to render the Minecraft insert command for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft insert command for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, index: i32, value: impl Into < sand::data::NbtValue >)  {\n    let command = data_modify_value.insert(index, value);\n}",
    )]
    pub fn insert(self, index: i32, value: impl Into<NbtValue>) -> String {
        self.reference.insert(index, value).to_string()
    }

    /// Renders the Minecraft merge command for the selected data modify.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::DataModify::merge",
        aliases = ["sand::cmd::DataModify::merge", "sand::prelude::cmd::DataModify::merge"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft merge command for the selected data modify.",
        context = "Renders the Minecraft merge command for the selected data modify. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to render the Minecraft merge command for the selected data modify."),
        returns = "The rendered Minecraft command text produced to render the Minecraft merge command for the selected data modify.",
        example = "use sand::prelude::*;\n\nfn demonstrate(data_modify_value: sand::command::DataModify, value: impl Into < sand::data::NbtValue >)  {\n    let command = data_modify_value.merge(value);\n}",
    )]
    pub fn merge(self, value: impl Into<NbtValue>) -> String {
        self.reference.merge(value).to_string()
    }
}

impl fmt::Display for DataModify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "data modify {} {}",
            self.reference.location, self.reference.path
        )
    }
}

impl Build for DataModify {
    fn build(&self) -> String {
        self.to_string()
    }
}

impl From<DataModify> for String {
    fn from(value: DataModify) -> Self {
        value.to_string()
    }
}

/// Starts a typed Minecraft data-modification builder for the target and NBT path.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::data_modify",
    aliases = ["sand::cmd::data_modify", "sand::prelude::cmd::data_modify"],
    module = "sand::command",
    summary = "Starts a typed Minecraft data-modification builder for the target and NBT path.",
    context = "Starts a typed Minecraft data-modification builder for the target and NBT path. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(target = "`target` provides the entity, block, or command target used to start a typed Minecraft data-modification builder for the target and NBT path.", path = "`path` provides the typed resource identifier or location used to start a typed Minecraft data-modification builder for the target and NBT path."),
    returns = "The `DataModify` value produced to start a typed Minecraft data-modification builder for the target and NBT path.",
    example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::data::DataTarget, path: impl Into < sand::data::NbtPath >)  {\n    let data_modify = sand::command::data_modify(target, path);\n}",
)]
pub fn data_modify(target: DataTarget, path: impl Into<NbtPath>) -> DataModify {
    DataModify::new(target, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_locations_and_operations_render() {
        let cache = Nbt::storage("my_pack:cache").path("items");
        let selected = Nbt::entity(Selector::self_()).path("SelectedItem");
        let block = Nbt::block(BlockPos::here()).path("Items[0]");

        assert_eq!(
            cache.get().to_string(),
            "data get storage my_pack:cache items"
        );
        assert_eq!(
            cache.get_scaled(10.0).to_string(),
            "data get storage my_pack:cache items 10"
        );
        assert_eq!(
            cache.copy_from(&selected).to_string(),
            "data modify storage my_pack:cache items set from entity @s SelectedItem"
        );
        assert_eq!(
            block.append_from(&cache).to_string(),
            "data modify block ~ ~ ~ Items[0] append from storage my_pack:cache items"
        );
        assert_eq!(
            cache.prepend(true).to_string(),
            "data modify storage my_pack:cache items prepend value 1b"
        );
        assert_eq!(
            cache.insert(-1, 7).to_string(),
            "data modify storage my_pack:cache items insert -1 value 7"
        );
        assert_eq!(
            cache
                .merge(NbtCompound::new().field("ready", true))
                .to_string(),
            "data modify storage my_pack:cache items merge value {ready:1b}"
        );
        assert_eq!(
            cache.remove().to_string(),
            "data remove storage my_pack:cache items"
        );
    }

    #[test]
    fn typed_and_raw_paths_have_distinct_validation() {
        let invalid = Nbt::storage("my_pack:data").path("bad..path");
        assert_eq!(
            invalid
                .get()
                .try_render(&CommandProfile::unprofiled())
                .unwrap_err()
                .code,
            "SAND-DATA-TARGET"
        );
        let raw =
            Nbt::storage("my_pack:data").path(NbtPath::raw("custom..modded[{anything:true}]"));
        assert!(raw.get().try_render(&CommandProfile::unprofiled()).is_ok());
    }

    #[test]
    fn rejects_invalid_storage_scale_and_many_entity_write() {
        let invalid_storage = Nbt::storage("Not Valid").path("x");
        assert!(
            invalid_storage
                .get()
                .try_render(&CommandProfile::unprofiled())
                .is_err()
        );
        let scale = Nbt::storage("pack:data").path("x");
        assert!(
            scale
                .get_scaled(f64::NAN)
                .try_render(&CommandProfile::unprofiled())
                .is_err()
        );
        let many = Nbt::entity(Selector::all_entities()).path("Health");
        assert!(
            many.set(1)
                .try_render(&CommandProfile::unprofiled())
                .is_err()
        );
        let player_inventory = Nbt::entity(Selector::self_()).path("Inventory[0]");
        let error = player_inventory
            .set(NbtCompound::new().field("id", "minecraft:stone"))
            .try_render(&CommandProfile::unprofiled())
            .unwrap_err();
        assert_eq!(error.code, "SAND-DATA-TARGET");
        assert!(error.message.contains("typed item location"));
    }

    #[test]
    fn compatibility_builder_keeps_output() {
        let command = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase").set(2_i32);
        assert_eq!(command, "data modify entity @s Custom.Phase set value 2");
    }
}
