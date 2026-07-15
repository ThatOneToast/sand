//! Cardinality-aware entity queries, execution-scoped entity contexts, and
//! typed vanilla relationship traversal.
//!
//! This module is the foundation issue #227 adds ahead of #228 (entity
//! operations/blueprints/state), #229 (item model), and #230 (event
//! participant contexts), all of which build on the types here.
//!
//! # Quick start
//! ```
//! use sand_core::entity::EntityQuery;
//! use sand_commands::selector::SortOrder;
//!
//! let cmds = EntityQuery::entities()
//!     .entity_type("minecraft:zombie")
//!     .without_tag("friendly")
//!     .within_blocks(15.0)
//!     .sort(SortOrder::Nearest)
//!     .limit(1)
//!     .expect("a positive limit is valid")
//!     .each(|entity| vec![entity.add_tag("observed")]);
//!
//! assert!(cmds[0].starts_with("execute as @e["));
//! ```
//!
//! # Concepts
//!
//! - [`EntityQuery`] / [`PlayerQuery`] — cardinality-aware query builders on
//!   top of [`sand_commands::selector`]'s arity-typed selectors. Filtering
//!   methods are available while cardinality is [`sand_commands::selector::Many`];
//!   [`EntityQuery::limit`]/[`EntityQuery::nearest`] narrow to
//!   [`sand_commands::selector::One`].
//! - [`EntityContext`] — the execution-scoped "current entity" (`@s`) handle
//!   passed into a query's `.each(...)` closure. It is **not** a persistent
//!   entity reference; see its docs for what that means.
//! - [`relation`] — typed traversal of vanilla's `execute on <relation>`
//!   relationships (owner, leasher, target, vehicle, controller, attacker,
//!   origin, passengers), version-gated against a [`crate::version::VersionProfile`].
//! - [`EntityScope`] — preserves a working reference to a specific entity
//!   across relationship traversal, which reassigns `@s`.

pub mod context;
pub mod kind;
pub mod query;
pub mod relation;

pub use context::{EntityContext, EntityScope, PlayerContext, ScopedEntityRef};
pub use kind::{AnyEntity, EntityKind, PlayerKind};
pub use query::{
    EntityQueries, EntityQuery, PlayerQueries, PlayerQuery, SingleEntityQuery, SinglePlayerQuery,
};
pub use relation::{Relation, RelationQuery};
