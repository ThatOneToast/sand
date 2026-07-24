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
use std::sync::{Mutex, OnceLock};

use crate::Build;
use crate::coord::BlockPos;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;

// ── Values ───────────────────────────────────────────────────────────────────

/// A typed SNBT value used by `data modify` and `data merge`.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    Bool(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<NbtValue>),
    Compound(NbtCompound),
    /// Explicit opaque SNBT. Sand renders this unchanged and does not parse it.
    Raw(String),
}

impl NbtValue {
    pub fn list(values: impl IntoIterator<Item = impl Into<NbtValue>>) -> Self {
        Self::List(values.into_iter().map(Into::into).collect())
    }

    pub fn compound(value: NbtCompound) -> Self {
        Self::Compound(value)
    }

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

/// A typed SNBT compound preserving declaration order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NbtCompound {
    entries: Vec<(String, NbtValue)>,
}

impl NbtCompound {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<NbtValue>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<NbtValue>) {
        self.entries.push((key.into(), value.into()));
    }

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
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            value: path.into(),
            raw: false,
        }
    }

    /// Explicit opaque path escape hatch. The path renders unchanged.
    pub fn raw(path: impl Into<String>) -> Self {
        Self {
            value: path.into(),
            raw: true,
        }
    }

    /// Compatibility spelling for a standalone typed path.
    pub fn root(path: impl Into<String>) -> Self {
        Self::new(path)
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn is_raw(&self) -> bool {
        self.raw
    }

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

    pub fn key(&self, key: impl AsRef<str>) -> Self {
        self.field(key)
    }

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

/// Canonical typed location for vanilla `data` commands.
#[derive(Debug, Clone)]
pub enum DataTarget {
    Entity(Selector),
    Block(BlockPos),
    Storage(String),
}

impl PartialEq for DataTarget {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for DataTarget {}

impl DataTarget {
    pub fn entity(selector: Selector) -> Self {
        Self::Entity(selector)
    }

    pub fn block(position: BlockPos) -> Self {
        Self::Block(position)
    }

    pub fn storage(id: impl Into<String>) -> Self {
        Self::Storage(id.into())
    }

    pub fn path(&self, path: impl Into<NbtPath>) -> NbtRef {
        NbtRef::new(self.clone(), path.into())
    }

    pub fn typed_path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(self.clone(), path.into())
    }

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

/// Factory for discoverable typed NBT target construction.
pub struct Nbt;

impl Nbt {
    pub fn entity(selector: Selector) -> NbtTarget {
        NbtTarget::new(DataTarget::entity(selector))
    }

    pub fn block(position: BlockPos) -> NbtTarget {
        NbtTarget::new(DataTarget::block(position))
    }

    pub fn storage(id: impl Into<String>) -> NbtTarget {
        NbtTarget::new(DataTarget::storage(id))
    }
}

/// An NBT location before a path is selected.
#[derive(Debug, Clone)]
pub struct NbtTarget {
    location: DataTarget,
}

impl NbtTarget {
    pub fn new(location: DataTarget) -> Self {
        Self { location }
    }

    pub fn location(&self) -> &DataTarget {
        &self.location
    }

    pub fn path(&self, path: impl Into<NbtPath>) -> NbtRef {
        NbtRef::new(self.location.clone(), path.into())
    }

    pub fn typed_path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(self.location.clone(), path.into())
    }

    pub fn merge(&self, value: NbtCompound) -> DataCommand {
        self.location.merge(value)
    }
}

/// Marker for an NBT reference without a declared schema value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UntypedNbt;

/// A typed target-plus-path reference shared by storage, entity, block, schema,
/// and inventory APIs.
#[derive(Debug, Clone)]
pub struct NbtRef<T = UntypedNbt> {
    location: DataTarget,
    path: NbtPath,
    marker: PhantomData<fn() -> T>,
}

impl<T> NbtRef<T> {
    pub fn new(location: DataTarget, path: NbtPath) -> Self {
        Self {
            location,
            path,
            marker: PhantomData,
        }
    }

    pub fn location(&self) -> &DataTarget {
        &self.location
    }

    pub fn path_value(&self) -> &NbtPath {
        &self.path
    }

    /// The path text, without its location.
    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }

    /// Compatibility accessor for storage-backed references.
    ///
    /// New generic code should inspect [`location`](Self::location) because
    /// entity and block references intentionally have no storage ID.
    pub fn storage(&self) -> &str {
        match &self.location {
            DataTarget::Storage(id) => id,
            _ => "",
        }
    }

    pub fn field(&self, key: impl AsRef<str>) -> NbtRef<T> {
        NbtRef::new(self.location.clone(), self.path.field(key))
    }

    pub fn typed_field<U>(&self, key: impl AsRef<str>) -> NbtRef<U> {
        NbtRef::new(self.location.clone(), self.path.field(key))
    }

    pub fn key(&self, key: impl AsRef<str>) -> NbtRef<T> {
        self.field(key)
    }

    pub fn index(&self, index: i32) -> NbtRef<T> {
        NbtRef::new(self.location.clone(), self.path.index(index))
    }

    pub fn get(&self) -> DataCommand {
        DataCommand::Get {
            source: self.untyped(),
            scale: None,
        }
    }

    pub fn get_scaled(&self, scale: f64) -> DataCommand {
        DataCommand::Get {
            source: self.untyped(),
            scale: Some(scale),
        }
    }

    pub fn set(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(DataModifyOperation::Set, DataSource::Value(value.into()))
    }

    pub fn set_value(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.set(value)
    }

    pub fn set_int(&self, value: i32) -> DataCommand {
        self.set(value)
    }

    pub fn set_bool(&self, value: bool) -> DataCommand {
        self.set(value)
    }

    pub fn set_string(&self, value: impl Into<String>) -> DataCommand {
        self.set(NbtValue::String(value.into()))
    }

    pub fn set_raw(&self, value: impl Into<String>) -> DataCommand {
        self.set(NbtValue::raw(value))
    }

    pub fn copy_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(DataModifyOperation::Set, DataSource::From(source.untyped()))
    }

    pub fn set_string_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Set,
            DataSource::String(source.untyped()),
        )
    }

    pub fn append(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(DataModifyOperation::Append, DataSource::Value(value.into()))
    }

    pub fn append_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Append,
            DataSource::From(source.untyped()),
        )
    }

    pub fn prepend(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(
            DataModifyOperation::Prepend,
            DataSource::Value(value.into()),
        )
    }

    pub fn prepend_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Prepend,
            DataSource::From(source.untyped()),
        )
    }

    pub fn insert(&self, index: i32, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(
            DataModifyOperation::Insert(index),
            DataSource::Value(value.into()),
        )
    }

    pub fn insert_from<U>(&self, index: i32, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Insert(index),
            DataSource::From(source.untyped()),
        )
    }

    pub fn merge(&self, value: impl Into<NbtValue>) -> DataCommand {
        self.modify(DataModifyOperation::Merge, DataSource::Value(value.into()))
    }

    pub fn merge_from<U>(&self, source: &NbtRef<U>) -> DataCommand {
        self.modify(
            DataModifyOperation::Merge,
            DataSource::From(source.untyped()),
        )
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataModifyOperation {
    Set,
    Append,
    Prepend,
    Insert(i32),
    Merge,
}

#[derive(Debug, Clone)]
pub enum DataSource {
    Value(NbtValue),
    From(NbtRef),
    String(NbtRef),
}

#[derive(Debug, Clone)]
pub enum DataCommand {
    Get {
        source: NbtRef,
        scale: Option<f64>,
    },
    Remove {
        target: NbtRef,
    },
    Modify {
        target: NbtRef,
        operation: DataModifyOperation,
        source: DataSource,
    },
    Merge {
        target: DataTarget,
        value: NbtCompound,
    },
}

impl DataCommand {
    pub fn try_render(&self, profile: &CommandProfile) -> CommandResult<String> {
        self.validate(profile)?;
        let rendered = self.render_unchecked(profile);
        register_line(&rendered, self.clone());
        Ok(rendered)
    }

    /// Compatibility convenience for assertions on rendered command text.
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
    reference.path.validate()
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

impl PartialEq<str> for DataCommand {
    fn eq(&self, other: &str) -> bool {
        self.to_string() == other
    }
}

impl PartialEq<&str> for DataCommand {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

fn data_error(field: &'static str, message: impl Into<String>) -> CommandError {
    CommandError::new("DataCommand", field, message).with_code("SAND-DATA-TARGET")
}

// Retain validation metadata after compatibility rendering to String.
fn registered_lines() -> &'static Mutex<BTreeMap<String, DataCommand>> {
    static LINES: OnceLock<Mutex<BTreeMap<String, DataCommand>>> = OnceLock::new();
    LINES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_line(line: &str, command: DataCommand) {
    registered_lines()
        .lock()
        .expect("data command registry poisoned")
        .insert(line.to_owned(), command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    let command = registered_lines()
        .lock()
        .expect("data command registry poisoned")
        .get(line)
        .cloned();
    if let Some(command) = command {
        command.try_render(profile)?;
    }
    Ok(())
}

// ── Compatibility DataModify builder ────────────────────────────────────────

/// Compatibility adapter over [`NbtRef`]. New code should start from [`Nbt`].
#[derive(Debug, Clone)]
pub struct DataModify {
    reference: NbtRef,
}

impl DataModify {
    pub fn new(target: DataTarget, path: impl Into<NbtPath>) -> Self {
        Self {
            reference: NbtRef::new(target, path.into()),
        }
    }

    pub fn set(self, value: impl Into<NbtValue>) -> String {
        self.reference.set(value).to_string()
    }

    pub fn set_from(self, source: DataTarget, source_path: impl Into<NbtPath>) -> String {
        self.reference
            .copy_from(&NbtRef::<UntypedNbt>::new(source, source_path.into()))
            .to_string()
    }

    pub fn append(self, value: impl Into<NbtValue>) -> String {
        self.reference.append(value).to_string()
    }

    pub fn append_from(self, source: DataTarget, source_path: impl Into<NbtPath>) -> String {
        self.reference
            .append_from(&NbtRef::<UntypedNbt>::new(source, source_path.into()))
            .to_string()
    }

    pub fn prepend(self, value: impl Into<NbtValue>) -> String {
        self.reference.prepend(value).to_string()
    }

    pub fn insert(self, index: i32, value: impl Into<NbtValue>) -> String {
        self.reference.insert(index, value).to_string()
    }

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
    }

    #[test]
    fn compatibility_builder_keeps_output() {
        let command = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase").set(2_i32);
        assert_eq!(command, "data modify entity @s Custom.Phase set value 2");
    }
}
