//! NBT value types and the `data modify` command builder.
//!
//! # Types
//!
//! - [`NbtValue`] — a typed NBT value (bool, byte, int, string, raw SNBT, etc.)
//! - [`DataTarget`] — where data lives (entity, block, or storage namespace)
//! - [`DataModify`] — builder for `data modify <target> <path> <operation>`
//!
//! The [`Storage`] HashMap abstraction lives in `sand-core` and is not part of
//! this crate. Use [`DataTarget::storage`] directly for storage-namespace targets.

use std::fmt;

use crate::Build;
use crate::coord::BlockPos;
use crate::selector::Selector;

// ── NbtValue ──────────────────────────────────────────────────────────────────

/// A typed NBT value for use with [`DataModify`].
///
/// Implements `From` for common Rust primitives:
///
/// ```
/// use sand_commands::nbt::NbtValue;
///
/// assert_eq!(NbtValue::from(2_i32).to_string(), "2");
/// assert_eq!(NbtValue::from(true).to_string(), "1b");
/// assert_eq!(NbtValue::from("hi").to_string(), "\"hi\"");
/// ```
#[derive(Debug, Clone)]
pub enum NbtValue {
    /// Boolean flag. `true` → `1b`, `false` → `0b`.
    Bool(bool),
    /// 8-bit signed integer. Serializes as `<n>b`.
    Byte(i8),
    /// 16-bit signed integer. Serializes as `<n>s`.
    Short(i16),
    /// 32-bit signed integer. Serializes as `<n>`.
    Int(i32),
    /// 64-bit signed integer. Serializes as `<n>L`.
    Long(i64),
    /// 32-bit float. Serializes as `<n>f`.
    Float(f32),
    /// 64-bit float. Serializes as `<n>d`.
    Double(f64),
    /// UTF-8 string. Serializes as `"<escaped>"`.
    String(String),
    /// Raw SNBT — use for compounds, lists, or pre-formatted NBT.
    Raw(String),
}

impl NbtValue {
    /// Wrap a pre-formatted SNBT string.
    pub fn raw(snbt: impl Into<String>) -> Self {
        NbtValue::Raw(snbt.into())
    }
}

impl fmt::Display for NbtValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtValue::Bool(b) => write!(f, "{}", if *b { "1b" } else { "0b" }),
            NbtValue::Byte(n) => write!(f, "{n}b"),
            NbtValue::Short(n) => write!(f, "{n}s"),
            NbtValue::Int(n) => write!(f, "{n}"),
            NbtValue::Long(n) => write!(f, "{n}L"),
            NbtValue::Float(n) => write!(f, "{n}f"),
            NbtValue::Double(n) => write!(f, "{n}d"),
            NbtValue::String(s) => {
                write!(f, "\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            }
            NbtValue::Raw(s) => write!(f, "{s}"),
        }
    }
}

impl From<bool> for NbtValue {
    fn from(v: bool) -> Self {
        NbtValue::Bool(v)
    }
}
impl From<i8> for NbtValue {
    fn from(v: i8) -> Self {
        NbtValue::Byte(v)
    }
}
impl From<i16> for NbtValue {
    fn from(v: i16) -> Self {
        NbtValue::Short(v)
    }
}
impl From<i32> for NbtValue {
    fn from(v: i32) -> Self {
        NbtValue::Int(v)
    }
}
impl From<i64> for NbtValue {
    fn from(v: i64) -> Self {
        NbtValue::Long(v)
    }
}
impl From<f32> for NbtValue {
    fn from(v: f32) -> Self {
        NbtValue::Float(v)
    }
}
impl From<f64> for NbtValue {
    fn from(v: f64) -> Self {
        NbtValue::Double(v)
    }
}
impl From<&str> for NbtValue {
    fn from(v: &str) -> Self {
        NbtValue::String(v.to_owned())
    }
}
impl From<String> for NbtValue {
    fn from(v: String) -> Self {
        NbtValue::String(v)
    }
}

// ── DataTarget ────────────────────────────────────────────────────────────────

/// Where data should be read from or written to.
#[derive(Debug, Clone)]
pub enum DataTarget {
    /// Entity NBT data (target must be a selector matching the entity).
    Entity(Selector),
    /// Block entity NBT data at a specific position.
    Block(BlockPos),
    /// A named NBT storage namespace (e.g. `"my_pack:global"`).
    Storage(String),
}

impl DataTarget {
    /// Create a data target for a specific entity.
    pub fn entity(selector: Selector) -> Self {
        DataTarget::Entity(selector)
    }
    /// Create a data target for a block entity.
    pub fn block(pos: BlockPos) -> Self {
        DataTarget::Block(pos)
    }
    /// Create a data target for a named storage namespace.
    pub fn storage(id: impl Into<String>) -> Self {
        DataTarget::Storage(id.into())
    }
}

impl fmt::Display for DataTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataTarget::Entity(sel) => write!(f, "entity {sel}"),
            DataTarget::Block(pos) => write!(f, "block {pos}"),
            DataTarget::Storage(id) => write!(f, "storage {id}"),
        }
    }
}

// ── DataModify ────────────────────────────────────────────────────────────────

/// Builder for `data modify <target> <path> <operation> <source>`.
///
/// # Example
/// ```
/// use sand_commands::nbt::{DataModify, DataTarget, NbtValue};
/// use sand_commands::selector::Selector;
///
/// let cmd = DataModify::new(DataTarget::entity(Selector::self_()), "Custom.ready")
///     .set(true);
/// assert_eq!(cmd, "data modify entity @s Custom.ready set value 1b");
/// ```
#[derive(Debug, Clone)]
pub struct DataModify {
    target: DataTarget,
    path: String,
}

impl DataModify {
    /// Create a new `DataModify` builder for the given target and NBT path.
    pub fn new(target: DataTarget, path: impl Into<String>) -> Self {
        Self {
            target,
            path: path.into(),
        }
    }

    /// `set value <nbt>` — overwrite the path with a typed value.
    pub fn set(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} set value {}",
            self.target,
            self.path,
            value.into()
        )
    }

    /// `set from <source> <source_path>` — copy a value from another location.
    pub fn set_from(self, source: DataTarget, source_path: impl Into<String>) -> String {
        format!(
            "data modify {} {} set from {} {}",
            self.target,
            self.path,
            source,
            source_path.into()
        )
    }

    /// `append value <nbt>` — push to the end of a list.
    pub fn append(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} append value {}",
            self.target,
            self.path,
            value.into()
        )
    }

    /// `prepend value <nbt>` — push to the front of a list.
    pub fn prepend(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} prepend value {}",
            self.target,
            self.path,
            value.into()
        )
    }

    /// `append from <source> <source_path>` — copy-append from another location.
    pub fn append_from(self, source: DataTarget, source_path: impl Into<String>) -> String {
        format!(
            "data modify {} {} append from {} {}",
            self.target,
            self.path,
            source,
            source_path.into()
        )
    }

    /// `insert <index> value <nbt>` — insert into a list at the given index.
    pub fn insert(self, index: i32, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} insert {} value {}",
            self.target,
            self.path,
            index,
            value.into()
        )
    }

    /// `merge value <nbt>` — merge a compound value into the path.
    pub fn merge(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} merge value {}",
            self.target,
            self.path,
            value.into()
        )
    }
}

impl fmt::Display for DataModify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data modify {} {}", self.target, self.path)
    }
}

impl Build for DataModify {
    fn build(&self) -> String {
        self.to_string()
    }
}

impl From<DataModify> for String {
    fn from(v: DataModify) -> Self {
        v.build()
    }
}

/// Build a `data modify` command targeting `target` at `path`.
///
/// # Example
/// ```
/// use sand_commands::nbt::{data_modify, DataTarget};
/// use sand_commands::selector::Selector;
///
/// let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase")
///     .set(2_i32);
/// assert_eq!(cmd, "data modify entity @s Custom.Phase set value 2");
/// ```
pub fn data_modify(target: DataTarget, path: impl Into<String>) -> DataModify {
    DataModify::new(target, path)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbt_value_display() {
        assert_eq!(NbtValue::Bool(true).to_string(), "1b");
        assert_eq!(NbtValue::Bool(false).to_string(), "0b");
        assert_eq!(NbtValue::Byte(42_i8).to_string(), "42b");
        assert_eq!(NbtValue::Short(100_i16).to_string(), "100s");
        assert_eq!(NbtValue::Int(42_i32).to_string(), "42");
        assert_eq!(NbtValue::Long(99_i64).to_string(), "99L");
        assert_eq!(NbtValue::String("hi".into()).to_string(), "\"hi\"");
        assert_eq!(NbtValue::Raw("{x:0}".into()).to_string(), "{x:0}");
    }

    #[test]
    fn nbt_value_string_escaping() {
        let v = NbtValue::from("say \"hello\"");
        assert_eq!(v.to_string(), r#""say \"hello\"""#);
    }

    #[test]
    fn nbt_value_from_primitives() {
        assert_eq!(NbtValue::from(true).to_string(), "1b");
        assert_eq!(NbtValue::from(2_i32).to_string(), "2");
        assert_eq!(NbtValue::from("hi").to_string(), "\"hi\"");
        assert_eq!(NbtValue::from(1.5_f64).to_string(), "1.5d");
    }

    #[test]
    fn data_modify_set() {
        let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase").set(2_i32);
        assert_eq!(cmd, "data modify entity @s Custom.Phase set value 2");
    }

    #[test]
    fn data_modify_set_bool() {
        let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Active").set(true);
        assert_eq!(cmd, "data modify entity @s Custom.Active set value 1b");
    }

    #[test]
    fn data_modify_append() {
        let cmd = data_modify(DataTarget::storage("my_pack:log"), "kills")
            .append(NbtValue::raw(r#"{type:"zombie"}"#));
        assert_eq!(
            cmd,
            r#"data modify storage my_pack:log kills append value {type:"zombie"}"#
        );
    }

    #[test]
    fn data_modify_set_from_entity() {
        let cmd = data_modify(DataTarget::storage("my_pack:debug"), "health")
            .set_from(DataTarget::entity(Selector::self_()), "Health");
        assert_eq!(
            cmd,
            "data modify storage my_pack:debug health set from entity @s Health"
        );
    }

    #[test]
    fn data_target_display() {
        assert_eq!(
            DataTarget::entity(Selector::self_()).to_string(),
            "entity @s"
        );
        assert_eq!(DataTarget::storage("ns:key").to_string(), "storage ns:key");
    }

    #[test]
    fn data_modify_implements_build() {
        use crate::Build;
        let dm = DataModify::new(DataTarget::entity(Selector::self_()), "Custom.Phase");
        assert_eq!(dm.build(), "data modify entity @s Custom.Phase");
        let s: String = dm.into();
        assert_eq!(s, "data modify entity @s Custom.Phase");
    }
}
