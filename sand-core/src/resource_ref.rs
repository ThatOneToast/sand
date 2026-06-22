//! Typed resource location references.
//!
//! Every datapack/resourcepack asset is addressed by a `namespace:path`
//! identifier. These typed wrappers validate the identifier at construction
//! time and make the asset kind explicit in the type system.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::resource_ref::{FunctionRef, PredicateRef};
//! use sand_core::condition::Condition;
//!
//! let can_cast = PredicateRef::new("my_pack:can_cast").unwrap();
//! let heal = FunctionRef::new("my_pack:heal").unwrap();
//!
//! let cond = Condition::predicate(&can_cast);
//! let cmd = format!("function {heal}");
//! ```

use std::fmt;

use sand_components::resource_location::ResourceLocation;

use crate::error::Result;

// ── Macro to reduce boilerplate ───────────────────────────────────────────────

macro_rules! resource_ref {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(ResourceLocation);

        impl $name {
            /// Construct a typed reference from a `"namespace:path"` string.
            ///
            /// Returns an error if the namespace or path contains invalid characters.
            pub fn new(location: impl AsRef<str>) -> Result<Self> {
                let loc = location.as_ref();
                let (ns, path) = loc.split_once(':').ok_or_else(|| {
                    crate::error::SandError::InvalidPath(format!(
                        "missing ':' in resource location '{loc}'"
                    ))
                })?;
                Ok(Self(ResourceLocation::new(ns, path)?))
            }

            /// Construct a typed reference to an external function (in another datapack).
            ///
            /// Alias for [`new`](Self::new) — the name makes the cross-pack intent explicit.
            pub fn external(location: impl AsRef<str>) -> Result<Self> {
                Self::new(location)
            }

            /// Return the underlying [`ResourceLocation`].
            pub fn location(&self) -> &ResourceLocation {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

resource_ref!(
    /// A typed reference to an `.mcfunction` file.
    FunctionRef
);

resource_ref!(
    /// A typed reference to a predicate JSON file.
    PredicateRef
);

resource_ref!(
    /// A typed reference to an advancement JSON file.
    AdvancementRef
);

resource_ref!(
    /// A typed reference to a loot table JSON file.
    LootTableRef
);

resource_ref!(
    /// A typed reference to a recipe JSON file.
    RecipeRef
);

resource_ref!(
    /// A typed reference to a dialog JSON file (requires Minecraft 1.21.5+ / 26.x).
    ///
    /// Use `VersionProfile::supports_dialogs()` before generating dialog references.
    DialogRef
);

impl DialogRef {
    /// Construct a local dialog reference whose namespace is resolved during export.
    ///
    /// Panics if `path` is not a valid resource path. Use [`try_local`](Self::try_local)
    /// when the path is user-provided.
    pub fn local(path: impl AsRef<str>) -> Self {
        Self::try_local(path).expect("invalid local dialog path")
    }

    /// Fallibly construct a local dialog reference whose namespace is resolved during export.
    pub fn try_local(path: impl AsRef<str>) -> Result<Self> {
        Ok(Self(ResourceLocation::new(
            crate::function::SAND_LOCAL_NS,
            path,
        )?))
    }
}

impl sand_components::dialog::IntoDialogRef for DialogRef {
    fn into_dialog_ref(self) -> String {
        self.to_string()
    }
}

impl sand_components::dialog::IntoDialogRef for &DialogRef {
    fn into_dialog_ref(self) -> String {
        self.to_string()
    }
}

// ── Condition::predicate integration ─────────────────────────────────────────

impl crate::condition::Condition {
    /// Build a predicate condition from a typed [`PredicateRef`].
    pub fn predicate_ref(r: &PredicateRef) -> Self {
        crate::condition::Condition::Predicate(r.to_string())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    #[test]
    fn function_ref_valid() {
        let r = FunctionRef::new("my_pack:heal").unwrap();
        assert_eq!(r.to_string(), "my_pack:heal");
    }

    #[test]
    fn predicate_ref_valid() {
        let r = PredicateRef::new("my_pack:can_cast").unwrap();
        assert_eq!(r.to_string(), "my_pack:can_cast");
    }

    #[test]
    fn advancement_ref_valid() {
        AdvancementRef::new("my_pack:first_join").unwrap();
    }

    #[test]
    fn loot_table_ref_valid() {
        LootTableRef::new("my_pack:mob/zombie").unwrap();
    }

    #[test]
    fn recipe_ref_valid() {
        RecipeRef::new("my_pack:iron_sword").unwrap();
    }

    #[test]
    fn invalid_no_colon() {
        assert!(FunctionRef::new("my_pack_heal").is_err());
    }

    #[test]
    fn invalid_namespace_chars() {
        assert!(FunctionRef::new("MY_PACK:heal").is_err());
    }

    #[test]
    fn predicate_ref_condition() {
        let r = PredicateRef::new("my_pack:can_cast").unwrap();
        let cond = Condition::predicate_ref(&r);
        match cond {
            Condition::Predicate(s) => assert_eq!(s, "my_pack:can_cast"),
            other => panic!("unexpected: {other:?}"),
        }
    }
}
