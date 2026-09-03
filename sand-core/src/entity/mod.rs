//! Canonical targets, execution-scoped entity contexts, and typed vanilla
//! relationship traversal.
//!
//! This module is the foundation issue #227 adds ahead of #228 (entity
//! operations/blueprints/state), #229 (item model), and #230 (event
//! participant contexts), all of which build on the types here.
//!
//! # Quick start
//! ```
//! use sand_commands::Target;
//! use sand_core::entity::TargetExecution;
//!
//! let cmds = Target::entities()
//!     .entity_type("minecraft:zombie")
//!     .without_tag("friendly")
//!     .within_blocks(15.0)
//!     .nearest()
//!     .each(|entity| vec![entity.add_tag("observed")]);
//!
//! assert!(cmds[0].starts_with("execute as @e["));
//! ```
//!
//! # Concepts
//!
//! - [`sand_commands::Target`] — the one cardinality-aware entity/player
//!   selection model. [`TargetExecution`] adds state filtering and iteration.
//! - [`EntityContext`] — the execution-scoped "current entity" (`@s`) handle
//!   passed into a query's `.each(...)` closure. It is **not** a persistent
//!   entity reference; see its docs for what that means.
//! - [`relation`] — typed traversal of vanilla's `execute on <relation>`
//!   relationships (owner, leasher, target, vehicle, controller, attacker,
//!   origin, passengers), version-gated against a [`crate::version::VersionProfile`].
//! - [`EntityScope`] — preserves a working reference to a specific entity
//!   across relationship traversal, which reassigns `@s`.

pub mod archetype;
pub mod context;
pub mod curve;
pub mod diagnostic;
pub mod kind;
pub mod property;
pub mod query;
pub mod relation;
pub mod state;

pub use archetype::{
    Adoption, AdoptionSource, DerivedScoreEncoding, EntityAction, EntityArchetype,
    EntityDerivation, EntityTransition, EntityTransitionField, Migration, ReconcilePolicy,
    SpecialEntityPolicy, ThresholdDirection,
};
pub use context::{EntityContext, EntityScope, PlayerContext, ScopedEntityRef};
pub use curve::{
    CurveEvaluationError, CurveInputs, DEFAULT_FIXED_POINT_SCALE, FixedPoint, FixedValue,
    OverflowPolicy, RoundingPolicy, StatCurve,
};
pub use diagnostic::EntityDiagnostic;
pub use kind::{
    AnyEntity, EntityKind, KnownEntityKind, LivingEntityKind, MarkerKind, MutableLivingEntityKind,
    PlayerKind, SafeEntityDataWriteKind, ZombieKind,
};
pub use property::{
    AttributeBinding, AttributeModifierBinding, CurrentHealthSync, EffectBinding, EntityEventId,
    EntityNbtBinding, EntityNbtProperty, EntityNbtType, EntityNbtValue, EntityTag, EntityTeam,
    EntityText, EntityTextSegment, EquipmentBinding, HealthBinding, HealthResizePolicy,
    NameBinding, NumericPropertySource, OwnershipPolicy, PropertyNameError, RawEntityProperty,
    RawEntityStateField, RawPropertyAccess, RawStateBackend, RefreshPolicy, TagBinding,
    TeamBinding,
};
pub use query::{StateQueryOperations, TargetExecution};
pub use relation::{Relation, RelationQuery};
pub use state::{
    Data, EntityCooldown, EntityCooldownAccessor, EntityEnum, EntityEnumAccessor, EntityEnumValue,
    EntityFlag, EntityFlagAccessor, EntityScore, EntityScoreAccessor, EntityState,
    EntityStateField, EntityTimer, EntityTimerAccessor, EnumEncoding, FixedScore,
    FixedScoreAccessor, FixedScoreValue, GlobalStateBundleOperations, KeyedData, Score,
    StateComposition, StateFieldDescriptor, StateFieldKind, StatePredicate, StateSchema,
};
