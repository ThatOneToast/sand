//! Reusable Sand modules embedded for project and engine integration.

/// A bundled module that can be vendored into an ordinary Sand package.
///
/// Libraries use normal Sand values, generics, modules and privacy. They do not
/// introduce compiler-only storage models or require Cargo in the consuming pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Library {
    /// Insertion-ordered, bounded maps and sets over owned typed values.
    Collections,
    /// Persistent positions and coordinate arithmetic.
    Spatial,
    /// Typed, bounded loaded-tick durations.
    Time,
}

impl Library {
    /// The module name and source-file stem used by the CLI installer.
    pub fn name(self) -> &'static str {
        match self {
            Self::Collections => "collections",
            Self::Spatial => "spatial",
            Self::Time => "time",
        }
    }

    /// Complete `.sand` source, suitable for an ordinary module file/provider.
    ///
    /// `Collections` exports `Entry<K, V>`, `Map<K, V>` and `Set<T>`. Values use
    /// canonical vectors, with their 1,024-element bound and owned copy behavior.
    /// `Spatial` exports `Position` and `Time` exports `Duration`.
    pub fn source(self) -> &'static str {
        match self {
            Self::Collections => include_str!("../stdlib/collections.sand"),
            Self::Spatial => include_str!("../stdlib/spatial.sand"),
            Self::Time => include_str!("../stdlib/time.sand"),
        }
    }
}

impl std::str::FromStr for Library {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "collections" => Ok(Self::Collections),
            "spatial" => Ok(Self::Spatial),
            "time" => Ok(Self::Time),
            _ => Err(format!(
                "unknown Sand library `{name}`; available: collections, spatial, time"
            )),
        }
    }
}
