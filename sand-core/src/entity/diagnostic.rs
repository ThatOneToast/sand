//! Structured diagnostics produced before entity runtime resources are written.

use thiserror::Error;

/// An entity schema, derivation, lifecycle, or property compilation error.
///
/// Variants carry stable codes and the most specific archetype, field,
/// property, derivation, or generated resource available.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum EntityDiagnostic {
    /// The entity kind does not implement a required capability.
    #[error("[SAND-ENTITY-CAPABILITY] `{archetype}` cannot apply `{property}` to `{entity_kind}`")]
    UnsupportedCapability {
        /// Archetype resource identifier.
        archetype: String,
        /// Entity-kind label.
        entity_kind: String,
        /// Unsupported property.
        property: String,
    },
    /// The target profile cannot lower a property.
    #[error(
        "[SAND-ENTITY-PROFILE] `{archetype}` property `{property}` is unsupported by `{profile}`: {reason}"
    )]
    UnsupportedProfile {
        /// Archetype resource identifier.
        archetype: String,
        /// Property or backend.
        property: String,
        /// Requested Minecraft profile.
        profile: String,
        /// Missing capability.
        reason: String,
    },
    /// Direct player entity-NBT mutation is unsafe.
    #[error(
        "[SAND-ENTITY-PLAYER-NBT] `{archetype}` property `{property}` would directly mutate player entity NBT"
    )]
    UnsafePlayerMutation {
        /// Archetype resource identifier.
        archetype: String,
        /// Unsafe property.
        property: String,
    },
    /// A schema field, generated objective, or encoding is duplicated.
    #[error("[SAND-ENTITY-STATE-DUPLICATE] schema `{schema}` field `{field}` conflicts: {detail}")]
    DuplicateStateField {
        /// Schema identifier.
        schema: String,
        /// Field or encoding.
        field: String,
        /// Conflict details.
        detail: String,
    },
    /// A typed enum encoding is invalid.
    #[error("[SAND-ENTITY-ENUM] schema `{schema}` field `{field}` has invalid encoding: {detail}")]
    InvalidEnumEncoding {
        /// Schema identifier.
        schema: String,
        /// Enum field.
        field: String,
        /// Duplicate or invalid encoding details.
        detail: String,
    },
    /// A curve contains NaN or infinity.
    #[error(
        "[SAND-ENTITY-CURVE-NON-FINITE] `{archetype}` derivation `{derivation}` contains `{value}`"
    )]
    NonFiniteCurve {
        /// Archetype resource identifier.
        archetype: String,
        /// Derivation identifier.
        derivation: String,
        /// Rejected value.
        value: String,
    },
    /// A numeric or selector range is empty or inverted.
    #[error("[SAND-ENTITY-RANGE] schema `{schema}` field `{field}` has invalid range `{range}`")]
    InvalidRange {
        /// Schema identifier.
        schema: String,
        /// State field.
        field: String,
        /// Rendered invalid range.
        range: String,
    },
    /// Fixed-point conversion or arithmetic exceeded its representation.
    #[error("[SAND-ENTITY-FIXED-OVERFLOW] `{archetype}` derivation `{derivation}`: {detail}")]
    FixedPointOverflow {
        /// Archetype resource identifier.
        archetype: String,
        /// Derivation identifier.
        derivation: String,
        /// Overflow details.
        detail: String,
    },
    /// The dependency graph contains a cycle.
    #[error("[SAND-ENTITY-DERIVATION-CYCLE] `{archetype}` cycle: {cycle}")]
    DerivationCycle {
        /// Archetype resource identifier.
        archetype: String,
        /// Ordered cycle path.
        cycle: String,
    },
    /// Multiple declarations own one native property.
    #[error(
        "[SAND-ENTITY-OWNERSHIP] `{archetype}` property `{property}` is owned by both `{first}` and `{second}`"
    )]
    ConflictingOwnership {
        /// Archetype resource identifier.
        archetype: String,
        /// Native property key.
        property: String,
        /// First declaration.
        first: String,
        /// Conflicting declaration.
        second: String,
    },
    /// Ordered migrations do not reach the current version.
    #[error("[SAND-ENTITY-MIGRATION-GAP] `{archetype}` has no migration path from {from} to {to}")]
    MissingMigrationPath {
        /// Archetype resource identifier.
        archetype: String,
        /// Old version.
        from: u32,
        /// Current version.
        to: u32,
    },
    /// An adoption query lacks a safe entity-type/locality constraint.
    #[error("[SAND-ENTITY-ADOPTION-UNBOUNDED] `{archetype}` adoption is unconstrained: {detail}")]
    UnconstrainedAdoption {
        /// Archetype resource identifier.
        archetype: String,
        /// Missing constraint.
        detail: String,
    },
    /// A periodic refresh interval is zero.
    #[error(
        "[SAND-ENTITY-REFRESH-INTERVAL] `{archetype}` property `{property}` has a zero-tick interval"
    )]
    InvalidRefreshInterval {
        /// Archetype resource identifier.
        archetype: String,
        /// Property or observation.
        property: String,
    },
    /// A health resize policy lacks required state.
    #[error("[SAND-ENTITY-HEALTH-RESIZE] `{archetype}` policy `{policy}` is invalid: {detail}")]
    InvalidHealthResizePolicy {
        /// Archetype resource identifier.
        archetype: String,
        /// Policy name.
        policy: String,
        /// Missing or incompatible input.
        detail: String,
    },
    /// A generated property requires unsupported function macros.
    #[error(
        "[SAND-ENTITY-MACRO-LOWERING] `{archetype}` resource `{resource}` requires function macros in `{profile}`"
    )]
    UnsupportedFunctionMacro {
        /// Archetype resource identifier.
        archetype: String,
        /// Generated helper resource.
        resource: String,
        /// Requested profile.
        profile: String,
    },
    /// An execution-scoped `@s` context was treated as durable identity.
    #[error(
        "[SAND-ENTITY-PERSISTENT-REFERENCE] `{context}` is execution-scoped to `@s` and cannot be stored"
    )]
    PersistentReferenceMisuse {
        /// Context or relationship that was misused.
        context: String,
    },
    /// Two definitions generated the same resource.
    #[error("[SAND-ENTITY-RESOURCE-COLLISION] `{resource}` is claimed by `{first}` and `{second}`")]
    ResourceCollision {
        /// Generated function/objective/tag/storage resource.
        resource: String,
        /// First owner.
        first: String,
        /// Conflicting owner.
        second: String,
    },
    /// An explicit raw extension failed validation.
    #[error("[SAND-ENTITY-RAW] `{archetype}` extension `{extension}` is invalid: {detail}")]
    InvalidRawExtension {
        /// Archetype resource identifier.
        archetype: String,
        /// Raw extension name.
        extension: String,
        /// Validation details.
        detail: String,
    },
}

impl EntityDiagnostic {
    /// Stable machine-readable diagnostic code.
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::code` for the canonical contract."]
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedCapability { .. } => "SAND-ENTITY-CAPABILITY",
            Self::UnsupportedProfile { .. } => "SAND-ENTITY-PROFILE",
            Self::UnsafePlayerMutation { .. } => "SAND-ENTITY-PLAYER-NBT",
            Self::DuplicateStateField { .. } => "SAND-ENTITY-STATE-DUPLICATE",
            Self::InvalidEnumEncoding { .. } => "SAND-ENTITY-ENUM",
            Self::NonFiniteCurve { .. } => "SAND-ENTITY-CURVE-NON-FINITE",
            Self::InvalidRange { .. } => "SAND-ENTITY-RANGE",
            Self::FixedPointOverflow { .. } => "SAND-ENTITY-FIXED-OVERFLOW",
            Self::DerivationCycle { .. } => "SAND-ENTITY-DERIVATION-CYCLE",
            Self::ConflictingOwnership { .. } => "SAND-ENTITY-OWNERSHIP",
            Self::MissingMigrationPath { .. } => "SAND-ENTITY-MIGRATION-GAP",
            Self::UnconstrainedAdoption { .. } => "SAND-ENTITY-ADOPTION-UNBOUNDED",
            Self::InvalidRefreshInterval { .. } => "SAND-ENTITY-REFRESH-INTERVAL",
            Self::InvalidHealthResizePolicy { .. } => "SAND-ENTITY-HEALTH-RESIZE",
            Self::UnsupportedFunctionMacro { .. } => "SAND-ENTITY-MACRO-LOWERING",
            Self::PersistentReferenceMisuse { .. } => "SAND-ENTITY-PERSISTENT-REFERENCE",
            Self::ResourceCollision { .. } => "SAND-ENTITY-RESOURCE-COLLISION",
            Self::InvalidRawExtension { .. } => "SAND-ENTITY-RAW",
        }
    }
}
