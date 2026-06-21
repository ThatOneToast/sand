//! Typed NBT storage variables backed by `data storage` commands.

use std::marker::PhantomData;

use crate::condition::Condition;

// ── NbtPath ───────────────────────────────────────────────────────────────────

/// A typed NBT path string for navigating storage/entity/block NBT.
///
/// # Example
/// ```rust,ignore
/// use sand_core::state::NbtPath;
///
/// let p = NbtPath::new("sand:data", "player.mana");
/// assert_eq!(p.as_str(), "player.mana");
/// assert_eq!(p.storage(), "sand:data");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtPath {
    storage: String,
    path: String,
}

impl NbtPath {
    /// Create a path for a storage location.
    pub fn new(storage: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            storage: storage.into(),
            path: path.into(),
        }
    }

    /// The storage namespace (e.g. `"sand:data"`).
    pub fn storage(&self) -> &str {
        &self.storage
    }

    /// The path within the storage (e.g. `"player.mana"`).
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Append a sub-key, returning a new path.
    pub fn key(&self, key: impl Into<String>) -> Self {
        Self {
            storage: self.storage.clone(),
            path: format!("{}.{}", self.path, key.into()),
        }
    }

    /// Index into an array, returning a new path.
    pub fn index(&self, i: usize) -> Self {
        Self {
            storage: self.storage.clone(),
            path: format!("{}[{i}]", self.path),
        }
    }

    /// `data get storage <storage> <path>` — read the NBT value.
    pub fn get(&self) -> String {
        format!("data get storage {} {}", self.storage, self.path)
    }

    /// `data remove storage <storage> <path>` — remove the NBT tag.
    pub fn remove(&self) -> String {
        format!("data remove storage {} {}", self.storage, self.path)
    }

    /// Build a `Condition` that checks `if data storage <storage> <path>`.
    pub fn exists(&self) -> Condition {
        Condition::StorageExists {
            location: self.storage.clone(),
            path: self.path.clone(),
        }
    }
}

// ── StorageVar ────────────────────────────────────────────────────────────────

/// A typed NBT storage variable.
///
/// Declare once as a `static` and use throughout your datapack. The type
/// parameter `T` is purely documentary — NBT does not carry Rust types at
/// runtime. Use `set_int`, `set_float`, `set_string`, etc. to pick the
/// correct SNBT literal.
///
/// # Example
/// ```rust,ignore
/// use sand_core::state::StorageVar;
///
/// static MANA: StorageVar<i32> = StorageVar::new("sand:data", "player.mana");
/// static NAME: StorageVar<String> = StorageVar::new("sand:data", "player.name");
///
/// fn load() -> Vec<String> {
///     vec![
///         MANA.set_int(100),
///         NAME.set_string("Steve"),
///     ]
/// }
/// ```
pub struct StorageVar<T = serde_json::Value> {
    storage: &'static str,
    path: &'static str,
    _marker: PhantomData<T>,
}

impl<T> StorageVar<T> {
    /// Create a new `StorageVar` pointing at `<storage> <path>`.
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// The storage namespace string (e.g. `"sand:data"`).
    pub fn storage(&self) -> &'static str {
        self.storage
    }

    /// The path string (e.g. `"player.mana"`).
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Build an [`NbtPath`] for this variable.
    pub fn as_path(&self) -> NbtPath {
        NbtPath::new(self.storage, self.path)
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    /// `data get storage <storage> <path>` — read the value.
    pub fn get(&self) -> String {
        format!("data get storage {} {}", self.storage, self.path)
    }

    /// `data get storage <storage> <path> <scale>` — read a numeric value with scale.
    pub fn get_scaled(&self, scale: f64) -> String {
        format!("data get storage {} {} {scale}", self.storage, self.path)
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT.
    pub fn set_raw(&self, snbt: &str) -> String {
        format!(
            "data modify storage {} {} set value {}",
            self.storage, self.path, snbt
        )
    }

    /// Set an integer value.
    pub fn set_int(&self, v: i32) -> String {
        self.set_raw(&v.to_string())
    }

    /// Set a long value (`<v>L` SNBT).
    pub fn set_long(&self, v: i64) -> String {
        self.set_raw(&format!("{v}L"))
    }

    /// Set a float value (`<v>f` SNBT).
    pub fn set_float(&self, v: f32) -> String {
        self.set_raw(&format!("{v}f"))
    }

    /// Set a double value (`<v>d` SNBT).
    pub fn set_double(&self, v: f64) -> String {
        self.set_raw(&format!("{v}d"))
    }

    /// Set a string value (auto-quoted, backslash-escaping inner quotes).
    pub fn set_string(&self, v: &str) -> String {
        let escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        self.set_raw(&format!("\"{escaped}\""))
    }

    /// Set a boolean as a byte (0b or 1b SNBT).
    pub fn set_bool(&self, v: bool) -> String {
        self.set_raw(if v { "1b" } else { "0b" })
    }

    /// `data modify storage <storage> <path> set from storage <src> <src_path>` — copy.
    pub fn copy_from(&self, src_storage: &str, src_path: &str) -> String {
        format!(
            "data modify storage {} {} set from storage {} {}",
            self.storage, self.path, src_storage, src_path
        )
    }

    // ── Delete / exists ───────────────────────────────────────────────────────

    /// `data remove storage <storage> <path>` — remove the tag.
    pub fn remove(&self) -> String {
        format!("data remove storage {} {}", self.storage, self.path)
    }

    /// Build a `Condition` that checks `if data storage <storage> <path>`.
    pub fn exists(&self) -> Condition {
        Condition::StorageExists {
            location: self.storage.to_string(),
            path: self.path.to_string(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    static MANA: StorageVar<i32> = StorageVar::new("sand:data", "player.mana");
    static NAME: StorageVar<String> = StorageVar::new("sand:data", "player.name");

    #[test]
    fn get_command() {
        assert_eq!(MANA.get(), "data get storage sand:data player.mana");
    }

    #[test]
    fn get_scaled() {
        assert_eq!(
            MANA.get_scaled(1.0),
            "data get storage sand:data player.mana 1"
        );
    }

    #[test]
    fn set_int() {
        assert_eq!(
            MANA.set_int(100),
            "data modify storage sand:data player.mana set value 100"
        );
    }

    #[test]
    fn set_string_escaping() {
        assert_eq!(
            NAME.set_string("Steve"),
            r#"data modify storage sand:data player.name set value "Steve""#
        );
        assert_eq!(
            NAME.set_string(r#"say "hi""#),
            r#"data modify storage sand:data player.name set value "say \"hi\"""#
        );
    }

    #[test]
    fn set_bool() {
        assert_eq!(
            MANA.set_bool(true),
            "data modify storage sand:data player.mana set value 1b"
        );
        assert_eq!(
            MANA.set_bool(false),
            "data modify storage sand:data player.mana set value 0b"
        );
    }

    #[test]
    fn set_float() {
        assert_eq!(
            MANA.set_float(1.5),
            "data modify storage sand:data player.mana set value 1.5f"
        );
    }

    #[test]
    fn set_long() {
        assert_eq!(
            MANA.set_long(9999),
            "data modify storage sand:data player.mana set value 9999L"
        );
    }

    #[test]
    fn remove_command() {
        assert_eq!(MANA.remove(), "data remove storage sand:data player.mana");
    }

    #[test]
    fn copy_from() {
        assert_eq!(
            MANA.copy_from("other:ns", "foo.bar"),
            "data modify storage sand:data player.mana set from storage other:ns foo.bar"
        );
    }

    #[test]
    fn exists_condition() {
        let cond = MANA.exists();
        match &cond {
            Condition::StorageExists { location, path } => {
                assert_eq!(location, "sand:data");
                assert_eq!(path, "player.mana");
            }
            other => panic!("unexpected: {other:?}"),
        }
        let cmds = cond.execute_commands(false, "run say exists");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if data storage sand:data player.mana"));
    }

    #[test]
    fn nbt_path_navigate() {
        let base = NbtPath::new("sand:data", "player");
        let mana = base.key("mana");
        assert_eq!(mana.as_str(), "player.mana");
        assert_eq!(mana.storage(), "sand:data");

        let first = mana.index(0);
        assert_eq!(first.as_str(), "player.mana[0]");
    }

    #[test]
    fn nbt_path_get_remove() {
        let p = NbtPath::new("sand:data", "player.mana");
        assert_eq!(p.get(), "data get storage sand:data player.mana");
        assert_eq!(p.remove(), "data remove storage sand:data player.mana");
    }

    #[test]
    fn nbt_path_exists() {
        let p = NbtPath::new("sand:data", "player.mana");
        let cond = p.exists();
        assert!(matches!(cond, Condition::StorageExists { .. }));
    }

    #[test]
    fn golden_mana_system() {
        let init = MANA.set_int(100);
        let check = MANA.exists();
        let drain = MANA.set_int(95);
        let cmds = check.execute_commands(false, &format!("run {drain}"));
        assert_eq!(
            init,
            "data modify storage sand:data player.mana set value 100"
        );
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if data storage sand:data player.mana"));
        assert!(cmds[0].contains("run data modify storage sand:data player.mana set value 95"));
    }
}
