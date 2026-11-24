//! Reusable Sand modules embedded for project and engine integration.

/// A bundled module that can be vendored into an ordinary Sand package.
///
/// Libraries use normal Sand values, generics, modules and privacy. They do not
/// introduce compiler-only storage models or require Cargo in the consuming pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Library {
    /// Insertion-ordered, bounded maps and sets over owned typed values.
    Collections,
}

impl Library {
    /// The module name and source-file stem used by the CLI installer.
    pub fn name(self) -> &'static str {
        match self {
            Self::Collections => "collections",
        }
    }

    /// Complete `.sand` source, suitable for an ordinary module file/provider.
    ///
    /// `Collections` exports `Entry<K, V>`, `Map<K, V>` and `Set<T>`. Values use
    /// canonical vectors, with their 1,024-element bound and owned copy behavior.
    pub fn source(self) -> &'static str {
        match self {
            Self::Collections => include_str!("../stdlib/collections.sand"),
        }
    }
}

impl std::str::FromStr for Library {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "collections" => Ok(Self::Collections),
            _ => Err(format!(
                "unknown Sand library `{name}`; available: collections"
            )),
        }
    }
}
