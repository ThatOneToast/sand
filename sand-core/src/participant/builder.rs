//! [`ParticipantBuilder`] — the public, typed API for declaring an event's
//! [`EventParticipantPlan`] (#273).
//!
//! # What a participant is
//!
//! A *participant* is not an ordinary runtime Rust value. It is either a
//! command-time reference to an entity Sand can still address when a
//! handler's generated commands run (an [`EntityParticipant`](crate::participant::EntityParticipant)), or an
//! immutable Sand-owned snapshot of an item copied out of the game state at
//! capture time (an [`ItemSnapshot`](crate::item::ItemSnapshot)). Both are
//! reconstructed from a plan's declarations, not fetched from any live
//! process running Minecraft — Sand is a compiler, and a participant handle
//! is a promise about what the *generated* commands will be able to
//! address, not a live query result.
//!
//! # Direct observation
//!
//! An event can capture a participant itself, from its own dispatch context:
//!
//! ```
//! use sand_core::participant::{EntityParticipantRole, ItemParticipantRole, ParticipantBuilder, ParticipantHand};
//!
//! fn participants() -> sand_core::participant::EventParticipantPlan {
//!     ParticipantBuilder::new()
//!         .observe_entity(EntityParticipantRole::Attacker)
//!         .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
//!         .build()
//! }
//! ```
//!
//! [`observe_entity`](ParticipantBuilder::observe_entity) generates a
//! correlated observation (`execute on attacker`, #230) at setup time and a
//! matching cleanup command after the event's synchronous descendants have
//! run — see [`EventParticipantPlan`]'s module doc for the exact ordering
//! contract. [`observe_item`](ParticipantBuilder::observe_item) captures an
//! exact snapshot of whatever is in the named hand; it has no separate
//! cleanup step (the storage is reset unconditionally on every capture, so a
//! stale value can never leak into a later invocation).
//!
//! Only [`EntityParticipantRole::Attacker`] and
//! [`EntityParticipantRole::Killer`] have a direct-observation backend
//! today — see `docs/testing/participant-role-evidence.md`'s support matrix
//! for the full picture of what is implemented versus merely modeled.
//!
//! # Inheritance
//!
//! A same-cycle child event can borrow a role its ancestor already captured,
//! instead of capturing it independently:
//!
//! ```
//! use sand_core::events::SandEvent;
//! use sand_core::participant::{EntityParticipantRole, ItemParticipantRole, ParticipantBuilder, ParticipantHand};
//!
//! struct ParentEvent;
//! impl SandEvent for ParentEvent {
//!     fn dispatch() -> impl Into<sand_core::events::SandEventDispatch> {
//!         sand_core::events::SandEventDispatch::tick().as_players()
//!     }
//!     fn participants() -> sand_core::participant::EventParticipantPlan {
//!         ParticipantBuilder::new()
//!             .observe_entity(EntityParticipantRole::Attacker)
//!             .build()
//!     }
//! }
//!
//! fn child_participants() -> sand_core::participant::EventParticipantPlan {
//!     ParticipantBuilder::new()
//!         .inherit_entity::<ParentEvent>(EntityParticipantRole::Attacker)
//!         .build()
//! }
//! ```
//!
//! `Source` must be a genuine same-cycle graph ancestor reachable through an
//! unbroken chain of single-parent, unbounded `.after(...)`/`chain::<...>()`
//! edges, and `Source` must declare the role via *direct* observation, not
//! itself via inheritance — transitive inheritance is not supported.
//! Neither condition can be checked from this builder alone (it has no view
//! of the event graph); both are enforced by `sand build`'s export-time
//! validation (`sand-core/src/compiler/export/participant_transport.rs`),
//! which fails with an actionable diagnostic naming the exact edge that
//! broke the chain rather than silently generating a dangling reference.
//! Because the child only ever runs inside the source's own synchronous
//! descendant call tree, the borrowed reference is valid for the child's
//! entire execution — inheritance contributes zero extra setup/cleanup
//! commands, since the source fully owns both.
//!
//! # Advancement parents
//!
//! An [`AdvancementEvent`](crate::event::AdvancementEvent) may be used as the
//! same-cycle parent of a [`SandEvent`](crate::events::SandEvent) — the
//! primary interoperability path between the two event families (#269),
//! and the number-one supported shape for combining them. `Event<T>`
//! remains the handler context for the `AdvancementEvent` side; the bare
//! `SandEvent` marker gets the identical accessor sugar via
//! [`crate::events::SandEventParticipants`], both reachable from
//! `use sand::prelude::*;` alone — no separate trait import needed:
//!
//! ```rust,ignore
//! use sand::prelude::*;
//!
//! pub struct SpecialKillEvent;
//!
//! impl SandEvent for SpecialKillEvent {
//!     fn dispatch() -> impl Into<SandEventDispatch> {
//!         SandEventDispatch::chain::<PlayerKillEvent>()
//!     }
//!
//!     fn participants() -> EventParticipantPlan {
//!         ParticipantBuilder::new()
//!             // PlayerKillEvent's own plan directly captures the killer
//!             // (a correlated observation — `@s` is the *victim* in this
//!             // advancement, so the killer is only reachable this way,
//!             // never an exact hand snapshot); inheriting it means
//!             // SpecialKillEvent resolves to the exact same binding.
//!             .inherit_entity::<PlayerKillEvent>(EntityParticipantRole::Killer)
//!             // PlayerKillEvent's plan does not declare a weapon (there is
//!             // nothing sensible to inherit for one), so SpecialKillEvent
//!             // observes its own — its own mainhand snapshot, composed
//!             // alongside the inherited entity in one plan.
//!             .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
//!             .build()
//!     }
//! }
//!
//! #[event]
//! fn direct_kill(event: Event<PlayerKillEvent>) {
//!     let killer = event.killer();
//! }
//!
//! #[event]
//! fn special_kill(event: SpecialKillEvent) {
//!     let killer = event.killer();
//!     let weapon = event.weapon();
//! }
//! ```
//!
//! See [`crate::event::AdvancementEvent::participants`] for how an
//! advancement-backed parent's own plan is applied around its synthesized
//! bridge entry (#269), before any dependent `SandEvent` runs, and
//! `examples/participant_audit`'s `SpecialKillEvent` for the complete,
//! exact-output-tested version of this example
//! (`sand-core/tests/event_chain_advancement_bridge_nested_siblings.rs`
//! additionally covers sibling and nested descendants of a bridge parent).
//!
//! A bridge parent may not currently also have a direct `#[event]` handler
//! of its own (#240 Phase 6) — split into two types (one for the direct
//! handler, one chained via `after`/`chain` for the composition) if both
//! are genuinely needed.
//!
//! # Unsupported graph shapes
//!
//! - Inheriting through `after_any`/`after_all` fan-in is rejected — which
//!   parent actually supplied the occurrence is not determinable from
//!   generated commands today (#271).
//! - Entity participants cannot be inherited across a `.within(...)` bounded
//!   window — a same-cycle borrowed reference cannot outlive the tick its
//!   source's temporary tag exists in.
//! - Item snapshots likewise cannot be automatically transported across
//!   `.within(...)` yet (#272) — copy a snapshot into your own storage by
//!   hand with [`ItemSnapshot::copy_to`](crate::item::snapshot::ItemSnapshot::copy_to)
//!   if you need that today.
//! - A tracked-transition [`SandEvent`](crate::events::SandEvent) cannot be a same-cycle chain/compose
//!   parent (it can still own and apply its own direct participant plan,
//!   #270) — see [`crate::events::graph`]'s `discover()` diagnostic.
//!
//! # Failure behavior
//!
//! Accessors built on a resolved plan (`event.killer()`, `event.weapon()`,
//! …) never return `Result`/`Option` — they return the typed participant
//! directly, for both handler forms. Statically-known mistakes are not
//! currently caught as `rustc` diagnostics at the call site: doing so
//! automatically for an arbitrary user-defined `participants()` function
//! would require the macro layer to parse the body of an ordinary Rust
//! function, which Sand deliberately does not do (see the crate-level
//! design notes) — only `sand build`'s mandatory graph/participant
//! validation is authoritative. A missing/unavailable participant access
//! fails export (before any datapack output is written — no partial output
//! is ever produced) with a diagnostic naming the event, handler, accessor,
//! and required role, e.g. (this is the exact rendered text, not a
//! paraphrase — see `sand-core/src/participant/diagnostic.rs::MissingParticipantPanic::render`
//! and `sand-core/tests/missing_participant_diagnostic.rs`):
//!
//! ```text
//! error[SAND-EVENT-PARTICIPANT]: unavailable event participant
//!
//! Event: vanilla_plus::SomeSandEvent
//! Handler: invalid_sand
//! Accessor: killer
//! Required role: EntityParticipantRole::Killer
//!
//! This event does not observe or inherit the requested participant (this role does not apply to this event).
//!
//! Declare it with ParticipantBuilder, for example:
//!
//!     ParticipantBuilder::new()
//!         .observe_entity(EntityParticipantRole::Killer)
//!         .build()
//!
//! or, if a same-cycle ancestor event already captures it:
//!
//!     ParticipantBuilder::new()
//!         .inherit_entity::<ParentEvent>(EntityParticipantRole::Killer)
//!         .build()
//! ```
//!
//! Internally, the infallible accessors panic with a structured
//! `MissingParticipantPanic` payload (crate-private; not a plain string)
//! when a role is unavailable; the export pipeline's handler-invocation
//! boundary catches it (via `catch_unwind`, with a scoped panic hook that
//! suppresses the default panic printer for exactly this payload type) and
//! converts it into the diagnostic above — a raw, unhandled Rust
//! panic/backtrace never reaches a `sand build` user.

use crate::participant::plan::{DuplicateParticipantRole, EventParticipantPlan};
use crate::participant::role::{EntityParticipantRole, ItemParticipantRole, ParticipantHand};

/// Builds an immutable [`EventParticipantPlan`] from ordinary typed method
/// calls — the normal-Rust replacement for attribute-macro-parameter-style
/// participant declarations. See the [module doc](self) for the full model.
#[derive(Debug, Clone, Default)]
pub struct ParticipantBuilder {
    plan: EventParticipantPlan,
}

impl ParticipantBuilder {
    /// Start building an empty plan.
    pub fn new() -> Self {
        Self {
            plan: EventParticipantPlan::new(),
        }
    }

    /// Declare a direct correlated observation of `role` on this event's own
    /// dispatch.
    ///
    /// Captures at setup time and cleans up after every synchronous
    /// descendant has run (see the [module doc](self)'s lifetime note).
    /// Duplicate role declarations within one builder are rejected — see
    /// [`Self::build`].
    ///
    /// Only [`EntityParticipantRole::Attacker`]/[`EntityParticipantRole::Killer`]
    /// have a direct-observation backend today.
    ///
    /// # Panics
    ///
    /// Panics immediately if `role` has no direct-observation backend — this
    /// is a fully local, statically-knowable mistake (it does not depend on
    /// the event graph), unlike the graph-dependent validation `sand build`
    /// performs.
    ///
    /// ```
    /// use sand_core::participant::{EntityParticipantRole, ParticipantBuilder};
    /// let plan = ParticipantBuilder::new()
    ///     .observe_entity(EntityParticipantRole::Attacker)
    ///     .build();
    /// assert!(!plan.is_empty());
    /// ```
    pub fn observe_entity(mut self, role: EntityParticipantRole) -> Self {
        if !matches!(
            role,
            EntityParticipantRole::Attacker | EntityParticipantRole::Killer
        ) {
            panic!(
                "ParticipantBuilder::observe_entity({role:?}) has no direct-observation backend today \
                 — only EntityParticipantRole::Attacker/Killer support direct capture (see \
                 docs/testing/participant-role-evidence.md's support matrix). Inherit this role from \
                 an ancestor with `.inherit_entity::<Source>(...)` instead, if one directly captures it."
            );
        }
        self.plan = self.plan.observe_correlated_attacker_as(role);
        self
    }

    /// Declare a direct held-item snapshot of `role`, captured from `hand`.
    ///
    /// Always an exact snapshot — addressing a hand slot on `@s` is a
    /// directly queryable NBT path, never a correlated guess. Has no
    /// separate cleanup step (see the [module doc](self)).
    ///
    /// ```
    /// use sand_core::participant::{ItemParticipantRole, ParticipantBuilder, ParticipantHand};
    /// let plan = ParticipantBuilder::new()
    ///     .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
    ///     .build();
    /// assert!(!plan.is_empty());
    /// ```
    pub fn observe_item(mut self, role: ItemParticipantRole, hand: ParticipantHand) -> Self {
        self.plan = self.plan.observe_held_item(role, hand);
        self
    }

    /// Declare that this event borrows `role` from `Source`'s own same-cycle
    /// direct capture, instead of capturing it independently.
    ///
    /// `Source` must be a real same-cycle graph ancestor of this event,
    /// reached through an unbroken chain of single-parent, unbounded
    /// `.after(...)`/`chain::<...>()` edges, and must itself directly
    /// capture `role` (not itself inherit it — transitive inheritance is not
    /// supported). Neither condition is checked here; both are validated by
    /// `sand build` against the fully resolved event graph, with an
    /// actionable diagnostic on failure. `Source` may be a
    /// [`SandEvent`](crate::events::SandEvent) or (through the same-cycle
    /// advancement bridge, #269) an
    /// [`AdvancementEvent`](crate::event::AdvancementEvent) that also
    /// implements `SandEvent`.
    ///
    /// Contributes zero setup/cleanup commands — `Source` fully owns both.
    ///
    /// ```
    /// use sand_core::events::SandEvent;
    /// use sand_core::participant::{EntityParticipantRole, ParticipantBuilder};
    ///
    /// struct Parent;
    /// impl SandEvent for Parent {
    ///     fn dispatch() -> impl Into<sand_core::events::SandEventDispatch> {
    ///         sand_core::events::SandEventDispatch::tick().as_players()
    ///     }
    /// }
    ///
    /// let plan = ParticipantBuilder::new()
    ///     .inherit_entity::<Parent>(EntityParticipantRole::Attacker)
    ///     .build();
    /// assert!(!plan.is_empty());
    /// ```
    pub fn inherit_entity<Source: crate::events::SandEvent + 'static>(
        mut self,
        role: EntityParticipantRole,
    ) -> Self {
        self.plan = self.plan.inherit_entity::<Source>(role);
        self
    }

    /// The item-snapshot counterpart to [`Self::inherit_entity`]. `hand`
    /// must match the hand `Source`'s own declaration captured from — this
    /// builder does not look `Source`'s declaration up for you.
    pub fn inherit_item<Source: crate::events::SandEvent + 'static>(
        mut self,
        role: ItemParticipantRole,
        hand: ParticipantHand,
    ) -> Self {
        self.plan = self.plan.inherit_item::<Source>(role, hand);
        self
    }

    /// Finish building, producing the immutable, compiler-facing
    /// [`EventParticipantPlan`].
    ///
    /// # Panics
    ///
    /// Panics with an actionable message if the same entity or item role was
    /// declared more than once — a fully local, immediately-knowable
    /// invariant (it depends only on this builder's own declarations, never
    /// on the event graph), so it is checked here rather than deferred to
    /// `sand build`.
    pub fn build(self) -> EventParticipantPlan {
        match self.plan.validate() {
            Ok(()) => {}
            Err(DuplicateParticipantRole::Entity(role)) => panic!(
                "ParticipantBuilder declared entity participant role {role:?} more than once — \
                 a role has one observation or none, never two competing ones."
            ),
            Err(DuplicateParticipantRole::Item(role)) => panic!(
                "ParticipantBuilder declared item participant role {role:?} more than once — \
                 a role has one observation or none, never two competing ones."
            ),
        }
        self.plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_builder_produces_empty_plan() {
        let plan = ParticipantBuilder::new().build();
        assert!(plan.is_empty());
    }

    #[test]
    fn chained_declarations_are_not_empty() {
        let plan = ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Attacker)
            .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .build();
        assert!(!plan.is_empty());
    }

    #[test]
    #[should_panic(expected = "more than once")]
    fn duplicate_role_panics_on_build() {
        ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Attacker)
            .observe_entity(EntityParticipantRole::Attacker)
            .build();
    }

    #[test]
    #[should_panic(expected = "no direct-observation backend")]
    fn unsupported_direct_role_panics_immediately() {
        ParticipantBuilder::new().observe_entity(EntityParticipantRole::Victim);
    }
}
