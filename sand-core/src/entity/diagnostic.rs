//! Structured diagnostics produced before entity runtime resources are written.

use thiserror::Error;

#[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic` for the canonical contract."]
/// An entity schema, derivation, lifecycle, or property compilation error.
///
/// Variants carry stable codes and the most specific archetype, field,
/// property, derivation, or generated resource available.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum EntityDiagnostic {
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedCapability` for the canonical contract."]
    /// The entity kind does not implement a required capability.
    #[error("[SAND-ENTITY-CAPABILITY] `{archetype}` cannot apply `{property}` to `{entity_kind}`")]
    UnsupportedCapability {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedCapability::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedCapability::entity_kind` for the canonical contract."]
        /// Entity-kind label.
        entity_kind: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedCapability::property` for the canonical contract."]
        /// Unsupported property.
        property: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedProfile` for the canonical contract."]
    /// The target profile cannot lower a property.
    #[error(
        "[SAND-ENTITY-PROFILE] `{archetype}` property `{property}` is unsupported by `{profile}`: {reason}"
    )]
    UnsupportedProfile {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedProfile::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedProfile::property` for the canonical contract."]
        /// Property or backend.
        property: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedProfile::profile` for the canonical contract."]
        /// Requested Minecraft profile.
        profile: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedProfile::reason` for the canonical contract."]
        /// Missing capability.
        reason: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsafePlayerMutation` for the canonical contract."]
    /// Direct player entity-NBT mutation is unsafe.
    #[error(
        "[SAND-ENTITY-PLAYER-NBT] `{archetype}` property `{property}` would directly mutate player entity NBT"
    )]
    UnsafePlayerMutation {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsafePlayerMutation::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsafePlayerMutation::property` for the canonical contract."]
        /// Unsafe property.
        property: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DuplicateStateField` for the canonical contract."]
    /// A schema field, generated objective, or encoding is duplicated.
    #[error("[SAND-ENTITY-STATE-DUPLICATE] schema `{schema}` field `{field}` conflicts: {detail}")]
    DuplicateStateField {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DuplicateStateField::schema` for the canonical contract."]
        /// Schema identifier.
        schema: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DuplicateStateField::field` for the canonical contract."]
        /// Field or encoding.
        field: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DuplicateStateField::detail` for the canonical contract."]
        /// Conflict details.
        detail: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidEnumEncoding` for the canonical contract."]
    /// A typed enum encoding is invalid.
    #[error("[SAND-ENTITY-ENUM] schema `{schema}` field `{field}` has invalid encoding: {detail}")]
    InvalidEnumEncoding {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidEnumEncoding::schema` for the canonical contract."]
        /// Schema identifier.
        schema: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidEnumEncoding::field` for the canonical contract."]
        /// Enum field.
        field: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidEnumEncoding::detail` for the canonical contract."]
        /// Duplicate or invalid encoding details.
        detail: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::NonFiniteCurve` for the canonical contract."]
    /// A curve contains NaN or infinity.
    #[error(
        "[SAND-ENTITY-CURVE-NON-FINITE] `{archetype}` derivation `{derivation}` contains `{value}`"
    )]
    NonFiniteCurve {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::NonFiniteCurve::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::NonFiniteCurve::derivation` for the canonical contract."]
        /// Derivation identifier.
        derivation: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::NonFiniteCurve::value` for the canonical contract."]
        /// Rejected value.
        value: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRange` for the canonical contract."]
    /// A numeric or selector range is empty or inverted.
    #[error("[SAND-ENTITY-RANGE] schema `{schema}` field `{field}` has invalid range `{range}`")]
    InvalidRange {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRange::schema` for the canonical contract."]
        /// Schema identifier.
        schema: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRange::field` for the canonical contract."]
        /// State field.
        field: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRange::range` for the canonical contract."]
        /// Rendered invalid range.
        range: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::FixedPointOverflow` for the canonical contract."]
    /// Fixed-point conversion or arithmetic exceeded its representation.
    #[error("[SAND-ENTITY-FIXED-OVERFLOW] `{archetype}` derivation `{derivation}`: {detail}")]
    FixedPointOverflow {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::FixedPointOverflow::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::FixedPointOverflow::derivation` for the canonical contract."]
        /// Derivation identifier.
        derivation: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::FixedPointOverflow::detail` for the canonical contract."]
        /// Overflow details.
        detail: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DerivationCycle` for the canonical contract."]
    /// The dependency graph contains a cycle.
    #[error("[SAND-ENTITY-DERIVATION-CYCLE] `{archetype}` cycle: {cycle}")]
    DerivationCycle {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DerivationCycle::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::DerivationCycle::cycle` for the canonical contract."]
        /// Ordered cycle path.
        cycle: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ConflictingOwnership` for the canonical contract."]
    /// Multiple declarations own one native property.
    #[error(
        "[SAND-ENTITY-OWNERSHIP] `{archetype}` property `{property}` is owned by both `{first}` and `{second}`"
    )]
    ConflictingOwnership {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ConflictingOwnership::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ConflictingOwnership::property` for the canonical contract."]
        /// Native property key.
        property: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ConflictingOwnership::first` for the canonical contract."]
        /// First declaration.
        first: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ConflictingOwnership::second` for the canonical contract."]
        /// Conflicting declaration.
        second: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::MissingMigrationPath` for the canonical contract."]
    /// Ordered migrations do not reach the current version.
    #[error("[SAND-ENTITY-MIGRATION-GAP] `{archetype}` has no migration path from {from} to {to}")]
    MissingMigrationPath {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::MissingMigrationPath::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::MissingMigrationPath::from` for the canonical contract."]
        /// Old version.
        from: u32,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::MissingMigrationPath::to` for the canonical contract."]
        /// Current version.
        to: u32,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnconstrainedAdoption` for the canonical contract."]
    /// An adoption query lacks a safe entity-type/locality constraint.
    #[error("[SAND-ENTITY-ADOPTION-UNBOUNDED] `{archetype}` adoption is unconstrained: {detail}")]
    UnconstrainedAdoption {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnconstrainedAdoption::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnconstrainedAdoption::detail` for the canonical contract."]
        /// Missing constraint.
        detail: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRefreshInterval` for the canonical contract."]
    /// A periodic refresh interval is zero.
    #[error(
        "[SAND-ENTITY-REFRESH-INTERVAL] `{archetype}` property `{property}` has a zero-tick interval"
    )]
    InvalidRefreshInterval {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRefreshInterval::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRefreshInterval::property` for the canonical contract."]
        /// Property or observation.
        property: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidHealthResizePolicy` for the canonical contract."]
    /// A health resize policy lacks required state.
    #[error("[SAND-ENTITY-HEALTH-RESIZE] `{archetype}` policy `{policy}` is invalid: {detail}")]
    InvalidHealthResizePolicy {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidHealthResizePolicy::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidHealthResizePolicy::policy` for the canonical contract."]
        /// Policy name.
        policy: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidHealthResizePolicy::detail` for the canonical contract."]
        /// Missing or incompatible input.
        detail: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedFunctionMacro` for the canonical contract."]
    /// A generated property requires unsupported function macros.
    #[error(
        "[SAND-ENTITY-MACRO-LOWERING] `{archetype}` resource `{resource}` requires function macros in `{profile}`"
    )]
    UnsupportedFunctionMacro {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedFunctionMacro::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedFunctionMacro::resource` for the canonical contract."]
        /// Generated helper resource.
        resource: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::UnsupportedFunctionMacro::profile` for the canonical contract."]
        /// Requested profile.
        profile: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::PersistentReferenceMisuse` for the canonical contract."]
    /// An execution-scoped `@s` context was treated as durable identity.
    #[error(
        "[SAND-ENTITY-PERSISTENT-REFERENCE] `{context}` is execution-scoped to `@s` and cannot be stored"
    )]
    PersistentReferenceMisuse {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::PersistentReferenceMisuse::context` for the canonical contract."]
        /// Context or relationship that was misused.
        context: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ResourceCollision` for the canonical contract."]
    /// Two definitions generated the same resource.
    #[error("[SAND-ENTITY-RESOURCE-COLLISION] `{resource}` is claimed by `{first}` and `{second}`")]
    ResourceCollision {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ResourceCollision::resource` for the canonical contract."]
        /// Generated function/objective/tag/storage resource.
        resource: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ResourceCollision::first` for the canonical contract."]
        /// First owner.
        first: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::ResourceCollision::second` for the canonical contract."]
        /// Conflicting owner.
        second: String,
    },
    #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRawExtension` for the canonical contract."]
    /// An explicit raw extension failed validation.
    #[error("[SAND-ENTITY-RAW] `{archetype}` extension `{extension}` is invalid: {detail}")]
    InvalidRawExtension {
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRawExtension::archetype` for the canonical contract."]
        /// Archetype resource identifier.
        archetype: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRawExtension::extension` for the canonical contract."]
        /// Raw extension name.
        extension: String,
        #[doc = "**API Contract:** Run `sand api show sand::entity::EntityDiagnostic::InvalidRawExtension::detail` for the canonical contract."]
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
