//! Immutable event-time item snapshots (#229 Phase 7).
//!
//! See the module doc on [`super`] for the split between [`super::location`]
//! (addressing) and this module (captured data). This file also documents
//! the exact capture-ordering, storage, lifetime, and concurrency contract —
//! read it before calling [`ItemSnapshot::capture`].
//!
//! # Capture semantics
//!
//! Capture is *always* attempted (there is no Rust-side `Option` — presence
//! is a runtime fact, discoverable only by the generated commands this
//! module emits, never by the Rust process). [`ItemSnapshot::capture`]
//! returns a handle plus the exact commands that:
//!
//! 1. reset the destination to explicit absence (`present = 0b`, `item = {}`) —
//!    unconditionally, so a stale snapshot from a previous invocation can
//!    never leak through;
//! 2. copy the source item compound to the destination *only if* the source
//!    path resolves (`execute if data <target> <path> run data modify ...`),
//!    leaving the destination at explicit absence otherwise — this is what
//!    makes "no item present" a distinct, typed, absence state rather than
//!    an item stack that happens to look like `minecraft:air`;
//! 3. mark the destination present *only if* step 2 actually copied
//!    something (a second `execute if data <source>`-gated command, not a
//!    bare unconditional `set value 1b` — an empty source must never be
//!    reported present).
//!
//! Embed the returned commands as the **first** commands your `SandEvent`
//! runs for this invocation — before any handler logic, before any other
//! generated mutation of the source. Concretely:
//!
//! - **Tick-backed events**: prepend to [`crate::events::EventSetup::pre_observation`]
//!   (already the earliest hook in the tick-dispatch pipeline — see
//!   `sand-core/src/component.rs`'s documented `pre_observation → condition
//!   test → handler → post_observation` contract).
//! - **Advancement-backed events** (direct `#[event]` handler, not a Phase 6
//!   graph-parent bridge): emit as the first command(s) of your own handler
//!   body. The generated entry always runs `advancement revoke` before
//!   calling your handler body, but revoke never touches item NBT, so this
//!   ordering is safe.
//! - **Phase 6 advancement graph-parent bridges**: **not supported** in this
//!   phase. A bridge parent cannot own a direct handler or `EventSetup`
//!   (Phase 6's own constraint), so there is no seam today for a bridge
//!   parent to run a capture command before its dependents dispatch.
//!   Automatically propagating a bridge parent's triggering item into its
//!   dependents' context is exactly the kind of context-propagation work
//!   reserved for #230, not this foundational phase.
//!
//! # Reliability
//!
//! [`SnapshotReliability::Exact`] for tick-backed captures (Sand's own
//! generated code runs first in `pre_observation`, before anything else
//! Sand controls). [`SnapshotReliability::ExactPostTrigger`] for
//! advancement-backed captures: Sand's capture command is the first thing
//! *Sand* runs, but vanilla's own item-consuming logic for some criteria
//! (documented, version/criterion-dependent — `minecraft:consume_item` is
//! the clearest known case) may already have mutated the source *before*
//! the criterion fired at all, i.e. before Sand ever gains control. Do not
//! upgrade an advancement-backed capture to `Exact` without runtime
//! verification for that specific criterion.
//!
//! # Storage and concurrency
//!
//! Snapshot storage uses **one deterministic, non-per-player path per
//! [`SnapshotSchema`]** (derived from a caller-supplied event label via the
//! same FNV-1a keying scheme the event graph uses for detector identity).
//! This is deliberately *not* per-player-keyed, and that is safe — not
//! despite concurrency, but because of how Minecraft executes commands:
//! `execute as @a ... run function X` fully completes one player's entire
//! synchronous call tree (function X and everything it calls) before moving
//! to the next player in the `@a` iteration. Command execution within one
//! server tick is single-threaded and never interleaves two players'
//! synchronous call trees. So as long as a snapshot is captured and fully
//! consumed within one synchronous dispatch invocation — exactly the
//! lifetime this type promises — no other player's capture can ever land in
//! between. What this design does **not** provide: safety for a snapshot
//! read *outside* its capturing invocation's synchronous call tree (a later
//! tick, a different event, a queued/deferred read) — nothing prevents a
//! later invocation (same player or a different one) from overwriting the
//! same global path once the capturing invocation's synchronous tree has
//! returned. Never persist a snapshot path/value into your own storage
//! across ticks; capture fresh each time you need one. Also not provided:
//! safety for two capture sites sharing one [`SnapshotSchema`] (same event
//! label) within *one* synchronous call tree — e.g. a same-cycle chained
//! child re-capturing at its parent's schema before the parent's own
//! snapshot has been fully consumed. `capture()` has no way to detect this
//! (it always resets-then-writes); give a nested/child capture its own
//! distinct event label if a still-in-use parent snapshot must not be
//! clobbered.
//!
//! # Lifetime
//!
//! Valid for the generated dispatch invocation that captured it and its
//! synchronous descendant graph calls (direct handlers, and any same-cycle
//! chained children reached from within that same call tree), then
//! considered expired. Sand does not currently enforce this at the type
//! level (the `ItemSnapshot` Rust value has no runtime component to
//! restrict) — this is a documented contract, not a compiler-enforced one.
//! Does not survive `/reload` in any meaningful sense
//! (the storage path is simply absent/reset — `/reload` does not clear
//! existing command storage, but nothing capture-scoped is expected to
//! exist between invocations anyway). Does not survive player disconnect
//! (nothing player-keyed exists to survive). Callers may copy a snapshot's
//! `item` compound into their own longer-lived storage explicitly — Sand
//! does not prevent this — but that copy is then the caller's own state
//! with the caller's own lifetime guarantees, not part of this contract.

use sand_commands::Execute;
use sand_commands::nbt::{DataModify, DataTarget, NbtValue};

use crate::condition::Condition;
use crate::events::graph::tick_event_resource_key;
use crate::item::location::{ItemLocation, ItemLocationError};
use crate::state::storage::NbtPath;

/// Deterministic, collision-checked identity for one snapshot's generated
/// storage. `storage` is the fully-qualified `namespace:path` command
/// storage id (the caller's own pack namespace — Sand does not invent one,
/// consistent with every other Sand-generated storage resource). `key` is
/// derived from a caller-supplied stable label (conventionally
/// `std::any::type_name::<YourSandEvent>()`) via the same FNV-1a scheme
/// [`crate::events::graph`] uses for detector resource keys, so two
/// snapshots for different event labels can never collide, and the same
/// label always produces the same path across repeated builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSchema {
    storage: String,
    key: String,
}

impl SnapshotSchema {
    /// `storage` must be a fully-qualified `namespace:path` command storage
    /// id owned by the caller's pack (Sand does not create or reserve a
    /// storage namespace on the caller's behalf). `event_label` should be a
    /// stable, unique string per capturing event — `std::any::type_name`
    /// of the owning `SandEvent` is the recommended choice, matching the
    /// convention used for every other generated-resource key in Sand.
    pub fn new(storage: impl Into<String>, event_label: &str) -> Self {
        Self {
            storage: storage.into(),
            key: tick_event_resource_key(event_label),
        }
    }

    /// The fully-qualified storage id this schema writes to.
    pub fn storage(&self) -> &str {
        &self.storage
    }

    fn base_path(&self) -> NbtPath {
        NbtPath::new(self.storage.clone(), format!("snap.{}", self.key))
    }
}

/// Explicit evidence strength for a captured item value. Never claim a
/// stronger level than the capture mechanism actually provides — see the
/// module doc for exactly which mechanisms produce which level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnapshotReliability {
    /// Copied from the authoritative event-time item source before any
    /// mutation Sand's own generated code could have caused, with no known
    /// vanilla-side mutation window before Sand gained control (tick-backed
    /// captures, run in `pre_observation` before anything else).
    Exact,
    /// Exact at the moment Sand's capture command ran, but vanilla may
    /// already have transformed the source before the triggering criterion
    /// fired at all — i.e. before Sand ever had a chance to run anything
    /// (advancement-backed captures; see the module doc's
    /// `minecraft:consume_item` example).
    ExactPostTrigger,
    /// Associated with this event through bounded correlation (e.g. a
    /// nearby/plausible source) rather than directly supplied by the
    /// triggering mechanism. Phase 7 does not produce this level — it is
    /// reserved for future correlated-context work (#230's phase 9); listed
    /// here so the reliability contract has room for it without a breaking
    /// enum change later.
    Correlated,
    /// No capture was possible or attempted for this location/context.
    Unavailable,
}

/// A validated, actionable diagnostic for snapshot capture. Always names
/// the requested location and the specific unsupported behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// The source [`ItemLocation`] could not be resolved.
    Location(ItemLocationError),
    /// The requested capture context is invalid for this location — e.g.
    /// a non-player-scoped location used where the caller asserted a
    /// player-only context, without an explicit selector.
    IncompatibleContext {
        location_kind: &'static str,
        reason: &'static str,
    },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Location(err) => write!(f, "{err}"),
            Self::IncompatibleContext {
                location_kind,
                reason,
            } => write!(
                f,
                "cannot capture item snapshot from `{location_kind}`: {reason}"
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl From<ItemLocationError> for SnapshotError {
    fn from(err: ItemLocationError) -> Self {
        Self::Location(err)
    }
}

/// An immutable handle to event-time item data captured into
/// Sand-generated storage. See the module doc for the full capture
/// ordering, reliability, absence, lifetime, and concurrency contract — a
/// value of this type is a *reference* to generated storage plus a
/// reliability/source label, never a live Rust-side copy of the item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSnapshot {
    schema: SnapshotSchema,
    source_kind: &'static str,
    reliability: SnapshotReliability,
}

/// Explicit typed absence classification, returned by nothing at
/// export/Rust time (presence is a runtime fact — see the module doc) but
/// named here as the vocabulary [`ItemSnapshot::is_present`]/
/// [`ItemSnapshot::is_absent`] answer at datapack runtime: a location
/// either had an item ([`SnapshotAbsence::Present`]) or genuinely had none
/// ([`SnapshotAbsence::Empty`]) — absence is never encoded as an item ID
/// such as `minecraft:air`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnapshotAbsence {
    Present,
    Empty,
}

impl ItemSnapshot {
    /// Resolve `location` and build the commands that capture it into
    /// `schema`'s storage, tagged with `reliability` (see the module doc —
    /// callers choose `Exact` for a tick-backed `pre_observation` capture,
    /// `ExactPostTrigger` for an advancement-backed capture; `Correlated`/
    /// `Unavailable` are not produced by this constructor, only by future
    /// #230 work).
    ///
    /// Returns the snapshot handle and the exact commands to run, in order,
    /// as the very first commands of the capturing invocation. See the
    /// module doc for exactly where to embed them for each dispatch kind.
    pub fn capture(
        location: &ItemLocation,
        schema: SnapshotSchema,
        reliability: SnapshotReliability,
    ) -> Result<(Self, Vec<String>), SnapshotError> {
        let (source_target, source_path) = location.nbt_source()?;
        let base = schema.base_path();
        let present_path = base.field("present");
        let item_path = base.field("item");
        let dest_storage = DataTarget::storage(schema.storage.clone());

        let mut commands = Vec::with_capacity(4);
        // 1. Unconditional reset to explicit absence — a stale snapshot
        //    from an earlier invocation must never leak through.
        commands.push(present_path.set_value(false));
        commands.push(
            DataModify::new(dest_storage.clone(), item_path.as_str().to_string())
                .set(NbtValue::raw("{}")),
        );
        // 2. Presence-gated exact copy — only runs (and only overwrites the
        //    reset-to-empty compound) if the source actually resolves.
        let copy_guard = presence_execute(&source_target, &source_path);
        commands.push(
            copy_guard.clone().run(
                DataModify::new(dest_storage.clone(), item_path.as_str().to_string())
                    .set_from(source_target, source_path),
            ),
        );
        // 3. Presence-gated mark — never a bare unconditional `set value
        //    1b`; an empty source must never be reported present.
        commands.push(copy_guard.run(present_path.set_value(true)));

        Ok((
            Self {
                schema,
                source_kind: location.kind(),
                reliability,
            },
            commands,
        ))
    }

    /// Reconstruct the handle for a snapshot captured elsewhere with the
    /// same `schema`, without generating (or re-generating) any capture
    /// commands.
    ///
    /// `pub(crate)` — reached by
    /// [`crate::participant::EventParticipantPlan::resolve_item`] so a
    /// handler body can address a plan-declared item snapshot without
    /// re-deriving its storage/key by hand.
    pub(crate) fn reconstruct(
        schema: SnapshotSchema,
        source_kind: &'static str,
        reliability: SnapshotReliability,
    ) -> Self {
        Self {
            schema,
            source_kind,
            reliability,
        }
    }

    /// The reliability level this snapshot was captured with.
    pub fn reliability(&self) -> SnapshotReliability {
        self.reliability
    }

    /// The [`ItemLocation::kind`] this snapshot was captured from.
    pub fn source_kind(&self) -> &'static str {
        self.source_kind
    }

    /// The fully-qualified storage id backing this snapshot.
    pub fn storage(&self) -> &str {
        self.schema.storage()
    }

    /// `if data storage <s> <path>{present:1b}` — true when the captured
    /// location actually had an item at capture time.
    pub fn is_present(&self) -> Condition {
        let base = self.schema.base_path();
        Condition::StorageExists {
            location: base.storage().to_string(),
            path: format!("{}{{present:1b}}", base.as_str()),
        }
    }

    /// The negation of [`ItemSnapshot::is_present`] — true when the
    /// captured location had no item ([`SnapshotAbsence::Empty`]).
    pub fn is_absent(&self) -> Condition {
        Condition::negate(self.is_present())
    }

    /// The typed NBT path to the captured item compound (`id`, `count`,
    /// `components`/legacy tag data, depending on the target profile's
    /// item-component encoding — this snapshot layer copies the item
    /// compound verbatim and does not itself reinterpret component shape;
    /// combine with `sand_components::item::ItemMatcher` / typed
    /// component accessors to interpret it, exactly as you would any other
    /// captured item compound).
    pub fn item_path(&self) -> NbtPath {
        self.schema.base_path().field("item")
    }

    /// The typed NBT path to the captured item's `id` field.
    pub fn id_path(&self) -> NbtPath {
        self.item_path().field("id")
    }

    /// The typed NBT path to the captured item's `count` field.
    pub fn count_path(&self) -> NbtPath {
        self.item_path().field("count")
    }

    /// The typed NBT path to the captured item's version-appropriate
    /// component/tag data (`components` on component-era targets). Callers
    /// needing version-aware interpretation should route through the
    /// existing `ItemMatcher`/`VersionCaps`-aware machinery rather than
    /// reading this path as version-independent.
    pub fn components_path(&self) -> NbtPath {
        self.item_path().field("components")
    }

    /// Commands that reset this snapshot's storage back to explicit
    /// absence. Run these once the capturing invocation's synchronous call
    /// tree is done consuming the snapshot, to avoid retaining item data
    /// longer than the documented lifetime (not required for correctness —
    /// the next capture always resets first — but keeps storage from
    /// holding onto a stale item compound between invocations, and is
    /// cheap, deterministic, and idempotent to call unconditionally).
    pub fn cleanup_commands(&self) -> Vec<String> {
        let base = self.schema.base_path();
        vec![
            base.field("present").set_value(false),
            DataModify::new(
                DataTarget::storage(self.schema.storage.clone()),
                base.field("item").as_str().to_string(),
            )
            .set(NbtValue::raw("{}")),
        ]
    }
}

/// Build the `execute if data <target> <path>` guard shared by the
/// presence-gated copy and mark commands, so both read the exact same
/// condition (never two independently-typed-out fragments that could
/// silently drift apart).
fn presence_execute(target: &DataTarget, path: &str) -> Execute {
    match target {
        DataTarget::Entity(selector) => {
            Execute::new().if_data_entity(selector.clone(), path.to_string())
        }
        DataTarget::Block(pos) => Execute::new().if_data_block(pos.clone(), path.to_string()),
        DataTarget::Storage(id) => Execute::new().if_data_storage(id.clone(), path.to_string()),
    }
}

// ── Integration seam for #230 ───────────────────────────────────────────────

/// The role an item plays in a future participant-rich event context
/// (#230). Phase 7 defines this purely as a stable label to pair with an
/// [`ItemSnapshot`] via [`EventItem`] — it does not implement any
/// role-specific observation/correlation backend; that is #230's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemRole {
    /// The item directly used to trigger the event (e.g. a consumed item,
    /// a placed block's item, a used tool).
    UsedItem,
    /// The item wielded as a weapon in a combat event.
    Weapon,
    /// The tool used for a block-interaction event.
    Tool,
    /// A projectile's own item form, where representable.
    ProjectileItem,
    /// The ammunition item consumed to fire a projectile.
    Ammunition,
    /// An item that was dropped as part of the event.
    DroppedItem,
    /// An item equipped as part of the event.
    EquippedItem,
}

/// One role-tagged item snapshot. The smallest stable seam a future
/// participant context (#230) needs to hold typed, immutable, event-time
/// item data without inventing its own snapshot representation — Phase 7
/// stops here; #230 owns assembling `EventItem`s into a full participant
/// context, subscription-based observation backends, and
/// exact/correlated/approximate reliability for *non-item* participant
/// fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventItem {
    pub role: ItemRole,
    pub snapshot: ItemSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> SnapshotSchema {
        SnapshotSchema::new("my_pack:snapshots", "my_pack::MyEvent")
    }

    #[test]
    fn schema_key_is_deterministic_and_distinct_per_label() {
        let a = SnapshotSchema::new("my_pack:snapshots", "EventA");
        let b = SnapshotSchema::new("my_pack:snapshots", "EventA");
        let c = SnapshotSchema::new("my_pack:snapshots", "EventB");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn capture_resets_before_conditionally_copying_and_marking_present() {
        let (_, commands) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        assert_eq!(commands.len(), 4, "{commands:#?}");
        assert!(commands[0].starts_with("data modify storage my_pack:snapshots"));
        assert!(commands[0].contains("present"));
        assert!(commands[0].ends_with("set value 0b"));
        assert!(commands[1].contains("item"));
        assert!(commands[1].ends_with("set value {}"));
        assert!(
            commands[2]
                .starts_with("execute if data entity @s SelectedItem run data modify storage"),
            "{}",
            commands[2]
        );
        assert!(commands[2].contains("set from entity @s SelectedItem"));
        assert!(
            commands[3]
                .starts_with("execute if data entity @s SelectedItem run data modify storage"),
            "{}",
            commands[3]
        );
        assert!(commands[3].ends_with("set value 1b"));
    }

    #[test]
    fn presence_guard_is_identical_between_copy_and_mark_commands() {
        let (_, commands) = ItemSnapshot::capture(
            &ItemLocation::PlayerOffHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        let copy_guard = commands[2].split(" run ").next().unwrap();
        let mark_guard = commands[3].split(" run ").next().unwrap();
        assert_eq!(copy_guard, mark_guard);
    }

    #[test]
    fn block_container_capture_uses_block_data_guard() {
        let location = ItemLocation::BlockContainer {
            position: sand_commands::coord::BlockPos::absolute(1, 2, 3),
            slot: crate::item::ContainerIndex::new(4).unwrap(),
        };
        let (_, commands) =
            ItemSnapshot::capture(&location, schema(), SnapshotReliability::Exact).unwrap();
        assert!(commands[2].starts_with("execute if data block 1 2 3 Items[{Slot:4b}]"));
    }

    #[test]
    fn is_present_and_is_absent_are_exact_negations() {
        let (snapshot, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        let present = snapshot.is_present();
        let absent = snapshot.is_absent();
        assert_eq!(absent, Condition::negate(present));
    }

    #[test]
    fn typed_field_paths_are_nested_under_the_item_compound() {
        let (snapshot, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        assert!(
            snapshot
                .id_path()
                .as_str()
                .starts_with(snapshot.item_path().as_str())
        );
        assert!(snapshot.id_path().as_str().ends_with(".id"));
        assert!(snapshot.count_path().as_str().ends_with(".count"));
        assert!(snapshot.components_path().as_str().ends_with(".components"));
    }

    #[test]
    fn cleanup_commands_reset_to_explicit_absence() {
        let (snapshot, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        let cleanup = snapshot.cleanup_commands();
        assert_eq!(cleanup.len(), 2);
        assert!(cleanup[0].ends_with("set value 0b"));
        assert!(cleanup[1].ends_with("set value {}"));
    }

    #[test]
    fn reliability_and_source_kind_are_recorded_on_the_handle() {
        let (snapshot, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerOffHand,
            schema(),
            SnapshotReliability::ExactPostTrigger,
        )
        .unwrap();
        assert_eq!(
            snapshot.reliability(),
            SnapshotReliability::ExactPostTrigger
        );
        assert_eq!(snapshot.source_kind(), "player_off_hand");
    }

    #[test]
    fn distinct_event_labels_produce_distinct_storage_paths() {
        let (a, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            SnapshotSchema::new("my_pack:snapshots", "EventA"),
            SnapshotReliability::Exact,
        )
        .unwrap();
        let (b, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            SnapshotSchema::new("my_pack:snapshots", "EventB"),
            SnapshotReliability::Exact,
        )
        .unwrap();
        assert_ne!(a.item_path().as_str(), b.item_path().as_str());
    }

    #[test]
    fn repeated_capture_for_the_same_schema_is_deterministic() {
        let (_, first) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        let (_, second) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn event_item_pairs_a_role_with_a_snapshot() {
        let (snapshot, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            schema(),
            SnapshotReliability::Exact,
        )
        .unwrap();
        let event_item = EventItem {
            role: ItemRole::UsedItem,
            snapshot,
        };
        assert_eq!(event_item.role, ItemRole::UsedItem);
    }
}
