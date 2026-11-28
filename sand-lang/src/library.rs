//! Reusable Sand modules embedded for project and engine integration.

/// A bundled module that can be vendored into an ordinary Sand package.
///
/// Libraries use normal Sand values, generics, modules and privacy. They do not
/// introduce compiler-only storage models or require Cargo in the consuming pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Library {
    /// Insertion-ordered, bounded maps and sets over owned typed values.
    Collections,
    /// Optional values and success or failure results.
    Outcomes,
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
            Self::Outcomes => "outcomes",
            Self::Spatial => "spatial",
            Self::Time => "time",
        }
    }

    /// Complete `.sand` source, suitable for an ordinary module file/provider.
    ///
    /// `Collections` exports `Entry<K, V>`, `Map<K, V>` and `Set<T>`. Values use
    /// canonical vectors, with their 1,024-element bound and owned copy behavior.
    /// `Outcomes` exports `Option<T>` and `Result<T, E>` as owned enums.
    /// `Spatial` exports `Position` and `Time` exports `Duration`.
    pub fn source(self) -> &'static str {
        match self {
            Self::Collections => include_str!("../stdlib/collections.sand"),
            Self::Outcomes => include_str!("../stdlib/outcomes.sand"),
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
            "outcomes" => Ok(Self::Outcomes),
            "spatial" => Ok(Self::Spatial),
            "time" => Ok(Self::Time),
            _ => Err(format!(
                "unknown Sand library `{name}`; available: collections, outcomes, spatial, time"
            )),
        }
    }
}
