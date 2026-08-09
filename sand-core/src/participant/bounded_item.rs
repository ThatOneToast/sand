//! Bounded, per-subject item-snapshot storage for `.within(...)` correlation
//! (#272).
//!
//! [`crate::item::ItemSnapshot`] is explicitly scoped to one synchronous
//! invocation (see its module doc's "Lifetime" section) — its storage is one
//! deterministic, non-per-player command-storage path, safe only because
//! Minecraft never interleaves two players' synchronous call trees within
//! one capture/consume cycle. That safety argument stops holding the moment
//! a value must survive *across* ticks: a bounded `.within(...)` child may
//! observe a source occurrence several ticks after it happened, by which
//! point other players (or the same player, again) may have run through the
//! source's own capture and overwritten that one global path. Persisting an
//! item snapshot across a tick boundary therefore needs genuine per-subject
//! storage, not [`crate::item::ItemSnapshot`]'s global-path convention.
//!
//! # Why not entity NBT
//!
//! The obvious per-subject backend is entity NBT — `data ... entity @s
//! <path>` is scoped to whichever entity the selector resolves to, which
//! would make cross-player leakage structurally impossible. An earlier
//! revision of this module did exactly that. **It does not work.** A live
//! Minecraft Java 26.2 RCON round-trip reproduces the following every time:
//!
//! ```text
//! > data modify entity @e[tag=nbttest5,limit=1] __sand_bounded_item.testkey set value 1b
//! Modified entity data of Armor Stand
//! > data get entity @e[tag=nbttest5,limit=1] __sand_bounded_item
//! Found no elements matching __sand_bounded_item
//! ```
//!
//! The write reports success and the key is silently absent immediately
//! afterward. This is not a settle-delay artifact, not specific to the
//! `__sand_` prefix, not specific to nesting (a bare `simplekey set value 5`
//! reproduces identically), not specific to one entity kind, and not an
//! RCON/tooling artifact — a vanilla-recognized field (`CustomName`,
//! `tag ... add`) on the *same* entity in the *same* session round-trips
//! correctly. Vanilla drops **any** custom top-level entity NBT key, on any
//! entity, generalizing the narrower findings already recorded in
//! `crate::systems::player_data` ("Arbitrary player NBT and inventory writes
//! are rejected") and `crate::item::location`.
//!
//! # The backend actually used
//!
//! Command storage (`data storage <id> <path>`), which *is* designed for
//! arbitrary structured data, keyed per subject by a **scoreboard-assigned
//! integer slot** substituted into the storage path by a **function macro**:
//!
//! - Every subject that ever sources a bounded occurrence is lazily assigned
//!   a unique integer from the [`SUBJECT_SLOT_OBJECTIVE`] objective (see
//!   `slot_alloc_body`). Nothing is allocated for players who never trigger
//!   a bounded source.
//! - That integer is stored into `<storage> args.subject` and the generated
//!   persist/load/expire functions are invoked with `function <f> with
//!   storage <storage> args`, so their `$`-prefixed macro lines resolve
//!   `p$(subject).<key>` to a genuine per-subject path such as `p7.<key>`.
//!
//! A **scoreboard integer**, specifically, is what makes this safe. Vanilla
//! offers no way to compute a dynamic NBT path from runtime data *except*
//! macro substitution, and macro substitution is textual — substituting a
//! player-controlled string (a name) or a structured value (`UUID` is an
//! int-array, which does not even render as a legal path segment) would be
//! both fragile and an injection hazard. An `int`-typed NBT value can only
//! ever render as digits and an optional minus sign, so the substituted path
//! is closed by construction.
//!
//! # Read side: staging into a static path
//!
//! Handler-visible accessors ([`BoundedItemSnapshot::item_path`],
//! [`BoundedItemSnapshot::is_present`], …) must be *static* paths — they are
//! ordinary Rust values a handler composes commands from, and no vanilla
//! mechanism resolves a runtime-computed path outside a macro line. So the
//! read side stages: a consuming child first calls the generated **load**
//! macro function, which copies `p$(subject).<key>` into the fixed scratch
//! path `cur.<key>`, and every accessor then reads that scratch path.
//!
//! This reuses [`crate::item::ItemSnapshot`]'s existing, already-documented
//! concurrency argument rather than inventing a new one: the scratch path is
//! global and non-per-player, but it is written and read entirely within one
//! subject's synchronous per-player call tree, which Minecraft never
//! interleaves. The durable, cross-tick copy — the part that genuinely must
//! survive other players running through the same code — is the per-subject
//! one.
//!
//! **The bounded snapshot is readable from a consuming child's handler body**
//! (the same place `.item()`/`.weapon()` participants are consumed). It is
//! not readable from that child's own `condition()`, which vanilla evaluates
//! before the child's dispatch function — and therefore before the load — is
//! entered.
//!
//! # Lifecycle
//!
//! - **Persist** (source occurrence): unconditional reset to explicit
//!   absence, then a presence-gated copy and presence-gated mark — the exact
//!   reset-then-conditionally-write shape
//!   [`ItemSnapshot::capture`](crate::item::ItemSnapshot::capture)
//!   documents. Repeated occurrences therefore replace atomically; a child
//!   can never observe a torn mix of an old and new item, and an occurrence
//!   with no item present clears presence rather than leaving stale data.
//! - **Expiry**: driven by the *existing* `.within(...)` age objective
//!   (`se_<source>_wa`), not a new mechanism.
//! - **Cleanup**: [`BoundedItemSnapshot::reset_commands`] for explicit paths.
//!
//! `persist_macro_body`/`load_macro_body`/`expire_macro_body` are generated
//! by the export pipeline (`sand-core/src/compiler/export/pipeline.rs`)
//! rather than by any one event's own
//! `EventParticipantPlan::build` — see that module for
//! exactly where in the generated per-player call tree each is spliced.
//!
//! # Live validation
//!
//! This backend is proven against a real Minecraft Java 26.2 server by
//! `scripts/mc_validation/run_bounded_item_proof.py`, which round-trips the
//! raw primitive (write, read back, isolation between two subject slots,
//! replacement without remnants, absent-source presence clearing, `/reload`
//! survival, per-subject expiry, clean shutdown) over RCON.

use sand_commands::DataTarget;

use crate::cmd::{function_with, macro_line, macro_var};
use crate::events::graph::tick_event_resource_key;
use crate::item::ItemSnapshot;
use crate::item::snapshot::SnapshotReliability;
use crate::participant::role::{ItemParticipantRole, ParticipantHand};
use crate::state::storage::{Nbt, NbtPath, NbtRef, UntypedNbt};

/// The reserved command storage every bounded item snapshot lives in.
///
/// Separate from `sand:__participants` (the transient, same-cycle snapshot
/// storage) precisely because the contents have different lifetimes: this
/// one is durable across ticks and across `/reload`, and is keyed per
/// subject.
pub const BOUNDED_ITEM_STORAGE: &str = "sand:__bounded_item";

/// The scoreboard objective holding each subject's integer slot.
///
/// Short enough to survive `ObjectiveName::logical` unhashed, so the
/// generated commands stay readable.
pub const SUBJECT_SLOT_OBJECTIVE: &str = "sand_subj";

/// The fake player holding the highest slot handed out so far.
const SLOT_COUNTER_HOLDER: &str = "#sand_subj_next";

/// The macro-variable name (and `args` sub-field) carrying the subject slot.
const SUBJECT_VAR: &str = "subject";

/// The `args` compound macro functions in this module are invoked with.
const ARGS_PATH: &str = "args";

/// Deterministic, collision-checked identity for one bounded item entry's
/// per-subject storage. `(source_event_label, role, hand)` determines the
/// key — three distinct declarations that all name the same
/// source/role/hand always resolve to the same storage, and any distinct
/// triple always resolves to a different one, via the same FNV-1a scheme
/// [`crate::item::snapshot::SnapshotSchema`] uses for its own keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedItemSchema {
    key: String,
}

impl BoundedItemSchema {
    /// `source_event_label` should be `std::any::type_name::<Source>()` —
    /// the same convention every other generated participant-transport key
    /// uses — so two distinct source event types can never collide even
    /// when they declare the identical role/hand.
    pub(crate) fn new(
        source_event_label: &str,
        role: ItemParticipantRole,
        hand: ParticipantHand,
    ) -> Self {
        Self {
            key: tick_event_resource_key(&format!(
                "{source_event_label}::bounded_item::{role:?}::{hand:?}"
            )),
        }
    }

    /// The generated resource key for this triple.
    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    /// The durable per-subject root, usable **only inside a macro line** —
    /// `p$(subject).<key>` is not a resolvable path until Minecraft
    /// substitutes it.
    fn subject_base(&self) -> NbtRef<UntypedNbt> {
        let subject = macro_var(SUBJECT_VAR);
        Nbt::storage(BOUNDED_ITEM_STORAGE).path(NbtPath::raw(format!("p{subject}.{}", self.key)))
    }

    /// The static scratch root a consuming child's handler reads, populated
    /// by the generated load function. See the [module doc](self).
    fn staged_base(&self) -> NbtRef<UntypedNbt> {
        Nbt::storage(BOUNDED_ITEM_STORAGE).path(NbtPath::raw(format!("cur.{}", self.key)))
    }
}

/// A read-only handle to a bounded, per-subject item snapshot — the
/// item-side counterpart of [`ItemSnapshot`] for values that must survive
/// across a `.within(...)` correlation window rather than just one
/// synchronous invocation. See the [module doc](self) for the full storage
/// and safety contract, including why these accessors name a staged path
/// rather than the durable per-subject one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedItemSnapshot {
    schema: BoundedItemSchema,
    source_kind: &'static str,
    reliability: SnapshotReliability,
}

impl BoundedItemSnapshot {
    /// Reconstruct the handle a bounded declaration resolves to, given the
    /// same `(source_event_label, role, hand)` the persist function was
    /// generated with. `pub(crate)` — reached by
    /// [`crate::participant::EventParticipantPlan::resolve_bounded_item`],
    /// never constructed directly by handler code.
    pub(crate) fn reconstruct(
        source_event_label: &str,
        role: ItemParticipantRole,
        hand: ParticipantHand,
        source_kind: &'static str,
        reliability: SnapshotReliability,
    ) -> Self {
        Self {
            schema: BoundedItemSchema::new(source_event_label, role, hand),
            source_kind,
            reliability,
        }
    }

    /// The reliability this snapshot's *item data* carries, preserved
    /// unchanged from the source capture — persisting a copy across a
    /// bounded window changes nothing about how trustworthy the item data
    /// itself is, only how long the copy remains readable (see
    /// [`crate::participant::ParticipantLifetime::BoundedWindow`] for the
    /// lifetime half of that distinction).
    pub fn reliability(&self) -> SnapshotReliability {
        self.reliability
    }

    /// The [`crate::item::ItemLocation::kind`] the *source* snapshot was
    /// originally captured from.
    pub fn source_kind(&self) -> &'static str {
        self.source_kind
    }

    /// `if data storage <storage> cur.<key>{present:1b}` — true when the
    /// source occurrence that most recently wrote this subject's storage
    /// actually had an item present at capture time. Absence here is never
    /// conflated with "the window has not been refreshed yet": the persist
    /// function always writes a definite present/absent value on every
    /// source occurrence.
    pub fn is_present(&self) -> crate::condition::Condition {
        let base = self.schema.staged_base();
        crate::condition::Condition::nbt_exists(
            base.location().clone(),
            NbtPath::raw(format!("{}{{present:1b}}", base.path_value().as_str())),
        )
    }

    /// The negation of [`Self::is_present`].
    pub fn is_absent(&self) -> crate::condition::Condition {
        crate::condition::Condition::negate(self.is_present())
    }

    /// The typed NBT path to the persisted item compound.
    pub fn item_path(&self) -> NbtRef<UntypedNbt> {
        self.schema.staged_base().field("item")
    }

    /// The typed NBT path to the persisted item's `id` field.
    pub fn id_path(&self) -> NbtRef<UntypedNbt> {
        self.item_path().field("id")
    }

    /// The typed NBT path to the persisted item's `count` field.
    pub fn count_path(&self) -> NbtRef<UntypedNbt> {
        self.item_path().field("count")
    }

    /// The typed NBT path to the persisted item's version-appropriate
    /// component/tag data.
    pub fn components_path(&self) -> NbtRef<UntypedNbt> {
        self.item_path().field("components")
    }

    /// Commands that unconditionally reset this bounded snapshot's *staged*
    /// storage back to explicit absence, for explicit/manual reset paths
    /// (e.g. a handler wanting to invalidate what it just read rather than
    /// acting on it twice). Identical in shape to
    /// [`ItemSnapshot::cleanup_commands`] — cheap, deterministic, and
    /// idempotent to call unconditionally.
    ///
    /// Note this clears what the *current* call tree can see, not the
    /// durable per-subject copy; clearing the durable copy requires the
    /// subject's slot and therefore a macro call, which the export pipeline
    /// generates for window expiry (see `expire_macro_body`).
    pub fn reset_commands(&self) -> Vec<String> {
        reset_to_absence(&self.schema.staged_base())
    }
}

/// The two commands that unconditionally reset `base`'s storage to explicit
/// absence — shared by the persist body's leading reset step, the load
/// body's leading reset step, [`expire_macro_body`], and
/// [`BoundedItemSnapshot::reset_commands`], so none of them can drift out of
/// sync with each other.
fn reset_to_absence(base: &NbtRef<UntypedNbt>) -> Vec<String> {
    vec![
        base.field("present").set_value(false).to_string(),
        base.field("item").set_raw("{}").to_string(),
    ]
}

/// Emit `copy_command` and the matching presence mark, both gated on
/// `present_guard`.
///
/// `copy_command` is passed in rather than built here so the persist path can
/// route through [`ItemSnapshot::copy_to_nbt`] (#267's typed copy API, which
/// #272 is specified in terms of) while the load path — which copies storage
/// to storage and has no [`ItemSnapshot`] in hand — uses the same underlying
/// [`NbtRef`] primitive directly. Both still share this one guard value, so
/// the copy and the mark can never drift apart — the same invariant
/// [`ItemSnapshot::capture`](crate::item::ItemSnapshot::capture) maintains
/// via its own single `presence_execute`.
fn gated_copy(
    present_guard: &crate::condition::Condition,
    copy_command: String,
    dest: &NbtRef<UntypedNbt>,
) -> Vec<String> {
    let mut commands = Vec::new();
    commands.extend(present_guard.execute_commands(false, &copy_command));
    commands.extend(
        present_guard.execute_commands(false, &dest.field("present").set_value(true).to_string()),
    );
    commands
}

/// Prefix only those commands that actually interpolate a macro variable.
///
/// Minecraft rejects a `$`-prefixed line containing no `$(...)` placeholder
/// outright — `Can't parse function line N` at *pack load time*, which takes
/// the whole function down, not just that line. Mixing plain and macro lines
/// in one macro-invoked function is fine, so the rule is simply: mark a line
/// iff it interpolates. (Found by live fire: an earlier revision marked every
/// line in the load body, two of which reset the static scratch path and so
/// interpolate nothing.)
fn as_macro_lines(commands: Vec<String>) -> Vec<String> {
    commands
        .into_iter()
        .map(|command| {
            if command.contains("$(") {
                macro_line(command)
            } else {
                command
            }
        })
        .collect()
}

/// The body of the generated **persist** macro function: copy `source`'s
/// currently-captured item into this subject's durable bounded storage.
///
/// Every line is a macro line (`$`-prefixed) because the destination path
/// contains `$(subject)`. Call it via [`call_macro`] after
/// [`bind_subject_commands`] has populated `args.subject`.
///
/// `pub(crate)` — called only from the export pipeline, which splices the
/// *call* into the source event's own generated dispatch function
/// immediately after its occurrence mark (guaranteed to still be the
/// source's own synchronous per-player call tree, which is what makes
/// reading its transient global [`ItemSnapshot`] capture safe — see this
/// module's doc).
pub(crate) fn persist_macro_body(schema: &BoundedItemSchema, source: &ItemSnapshot) -> Vec<String> {
    let dest = schema.subject_base();
    let mut commands = reset_to_absence(&dest);
    commands.extend(gated_copy(
        &source.is_present(),
        source.copy_to_nbt(&dest.field("item")),
        &dest,
    ));
    as_macro_lines(commands)
}

/// The body of the generated **load** macro function: stage this subject's
/// durable bounded storage into the static scratch path the read-side
/// accessors name.
///
/// Resets the scratch path first, so a subject with nothing persisted (or an
/// expired window) observes explicit absence rather than whatever the
/// previous consumer in this tick left behind.
pub(crate) fn load_macro_body(schema: &BoundedItemSchema) -> Vec<String> {
    let src = schema.subject_base();
    let dest = schema.staged_base();
    let present_guard = crate::condition::Condition::nbt_exists(
        src.location().clone(),
        NbtPath::raw(format!("{}{{present:1b}}", src.path_value().as_str())),
    );
    let mut commands = reset_to_absence(&dest);
    commands.extend(gated_copy(
        &present_guard,
        dest.field("item").copy_from(&src.field("item")).to_string(),
        &dest,
    ));
    as_macro_lines(commands)
}

/// The body of the generated **expire** macro function: clear this subject's
/// durable bounded storage on window expiry.
pub(crate) fn expire_macro_body(schema: &BoundedItemSchema) -> Vec<String> {
    as_macro_lines(reset_to_absence(&schema.subject_base()))
}

/// The body of the generated slot-allocator function.
///
/// Invoked as `@s` = the subject being allocated, and only when that subject
/// has no slot yet (see [`bind_subject_commands`]), so the counter advances
/// exactly once per subject. Slots are never reused; the counter is an `i32`
/// like every other Sand score, which at one slot per distinct player is not
/// a practical exhaustion concern.
pub(crate) fn slot_alloc_body() -> Vec<String> {
    vec![
        format!("scoreboard players add {SLOT_COUNTER_HOLDER} {SUBJECT_SLOT_OBJECTIVE} 1"),
        format!(
            "scoreboard players operation @s {SUBJECT_SLOT_OBJECTIVE} = \
             {SLOT_COUNTER_HOLDER} {SUBJECT_SLOT_OBJECTIVE}"
        ),
    ]
}

/// `scoreboard objectives add <slot objective> dummy` — emitted once into
/// setup whenever any bounded item transport exists.
pub(crate) fn slot_objective_definition() -> String {
    format!("scoreboard objectives add {SUBJECT_SLOT_OBJECTIVE} dummy")
}

/// Commands that make `@s`'s slot available to a following [`call_macro`]:
/// allocate one if this subject has never had one, then store it into
/// `args.subject`.
///
/// `alloc_function` is the fully-qualified generated allocator, e.g.
/// `my_pack:__sand_bounded_item_slot_alloc`.
pub(crate) fn bind_subject_commands(alloc_function: &str) -> Vec<String> {
    vec![
        format!(
            "execute unless score @s {SUBJECT_SLOT_OBJECTIVE} matches {}.. run function \
             {alloc_function}",
            i32::MIN
        ),
        store_subject_command(),
    ]
}

/// `execute store result ... run scoreboard players get @s <slot objective>`.
///
/// Only correct where `@s` is already known to have a slot — an unset score
/// makes `scoreboard players get` fail, which leaves `args.subject`
/// *unchanged* rather than clearing it. See [`bind_subject_for_read`] for the
/// variant that is safe without that guarantee.
pub(crate) fn store_subject_command() -> String {
    format!(
        "execute store result storage {BOUNDED_ITEM_STORAGE} {ARGS_PATH}.{SUBJECT_VAR} int 1 run \
         scoreboard players get @s {SUBJECT_SLOT_OBJECTIVE}"
    )
}

/// Slot value meaning "this subject has never sourced a bounded occurrence".
///
/// [`slot_alloc_body`] increments a counter that starts unset (`0`) *before*
/// copying it, so the first slot handed out is `1` and `0` is never a real
/// subject. Reading `p0.<key>` therefore always finds nothing, which is
/// exactly the desired outcome for an unslotted subject.
const NO_SUBJECT_SENTINEL: i32 = 0;

/// Bind `args.subject` for a **read** (load) call, where `@s` may never have
/// sourced a bounded occurrence and so may have no slot at all.
///
/// Resets to [`NO_SUBJECT_SENTINEL`] first, then stores the real slot only if
/// one exists. Without the reset, a failed `scoreboard players get` would
/// leave whatever the previous caller wrote in `args.subject` — meaning an
/// unslotted player would load *another player's* snapshot. This is the one
/// place cross-subject leakage could realistically be reintroduced, so the
/// clear is unconditional and comes first.
pub(crate) fn bind_subject_for_read() -> Vec<String> {
    vec![
        format!(
            "data modify storage {BOUNDED_ITEM_STORAGE} {ARGS_PATH}.{SUBJECT_VAR} set value \
             {NO_SUBJECT_SENTINEL}"
        ),
        format!(
            "execute {} store result storage {BOUNDED_ITEM_STORAGE} {ARGS_PATH}.{SUBJECT_VAR} int \
             1 run scoreboard players get @s {SUBJECT_SLOT_OBJECTIVE}",
            has_slot_guard()
        ),
    ]
}

/// `execute as @a` guard fragment restricting a per-player line to subjects
/// that actually have a slot — i.e. that have sourced at least one bounded
/// occurrence. Without it, `execute store result ... run scoreboard players
/// get @s <obj>` on an unset score would store `0` and address slot `p0`,
/// which belongs to nobody.
pub(crate) fn has_slot_guard() -> String {
    format!(
        "if score @s {SUBJECT_SLOT_OBJECTIVE} matches {}..",
        i32::MIN
    )
}

/// `function <name> with storage <storage> args` — invoke one of this
/// module's generated macro functions against the currently-bound subject.
pub(crate) fn call_macro(function: &str) -> String {
    function_with(
        function,
        DataTarget::Storage(BOUNDED_ITEM_STORAGE.to_string()),
        ARGS_PATH,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ItemLocation;
    use crate::item::snapshot::SnapshotSchema;

    fn source_snapshot() -> ItemSnapshot {
        let (snapshot, _) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            SnapshotSchema::new("test:participants", "test::SourceEvent"),
            SnapshotReliability::ExactPostTrigger,
        )
        .unwrap();
        snapshot
    }

    fn schema() -> BoundedItemSchema {
        BoundedItemSchema::new(
            "SourceEvent",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
        )
    }

    #[test]
    fn schema_key_is_deterministic_and_distinct_per_triple() {
        let a = BoundedItemSchema::new(
            "EventA",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
        );
        let b = BoundedItemSchema::new(
            "EventA",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
        );
        let c = BoundedItemSchema::new(
            "EventA",
            ItemParticipantRole::Weapon,
            ParticipantHand::OffHand,
        );
        let d = BoundedItemSchema::new(
            "EventB",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    /// The whole point of the redesign: nothing may write entity NBT, and
    /// every durable write must be a macro line into command storage.
    #[test]
    fn no_generated_body_touches_entity_nbt() {
        let schema = schema();
        let bodies = [
            persist_macro_body(&schema, &source_snapshot()),
            load_macro_body(&schema),
            expire_macro_body(&schema),
        ];
        for body in bodies.iter().flatten() {
            assert!(
                !body.contains("entity @s"),
                "bounded item storage must never use entity NBT: {body}"
            );
            // A `$`-prefixed line must interpolate something, and a line
            // that interpolates must be `$`-prefixed. Minecraft rejects the
            // former at pack load time.
            assert_eq!(
                body.starts_with('$'),
                body.contains("$("),
                "macro-line marking must match actual interpolation: {body}"
            );
            assert!(
                body.contains(BOUNDED_ITEM_STORAGE),
                "must target bounded storage: {body}"
            );
        }
    }

    #[test]
    fn persist_body_resets_before_conditionally_copying_and_marking() {
        let commands = persist_macro_body(&schema(), &source_snapshot());
        assert_eq!(commands.len(), 4, "{commands:#?}");
        assert!(commands[0].starts_with("$data modify storage sand:__bounded_item p$(subject)."));
        assert!(commands[0].ends_with(".present set value 0b"));
        assert!(commands[1].ends_with(".item set value {}"));
        assert!(commands[2].starts_with("$execute if data storage test:participants"));
        assert!(commands[2].contains("run data modify storage sand:__bounded_item p$(subject)."));
        assert!(commands[3].starts_with("$execute if data storage test:participants"));
        assert!(commands[3].ends_with(".present set value 1b"));
    }

    #[test]
    fn load_body_stages_subject_storage_into_the_static_scratch_path() {
        let schema = schema();
        let commands = load_macro_body(&schema);
        assert_eq!(commands.len(), 4, "{commands:#?}");
        let key = schema.key();
        // Resets target the scratch path, so a subject with nothing
        // persisted observes absence rather than the previous consumer's
        // staged value.
        assert!(commands[0].contains(&format!("cur.{key}.present")));
        assert!(commands[1].contains(&format!("cur.{key}.item")));
        // The gated copy reads the durable per-subject path.
        assert!(commands[2].contains(&format!("p$(subject).{key}{{present:1b}}")));
        assert!(commands[2].contains(&format!(
            "run data modify storage sand:__bounded_item cur.{key}.item"
        )));
        assert!(commands[2].contains(&format!(
            "from storage sand:__bounded_item p$(subject).{key}.item"
        )));
        assert!(commands[3].ends_with(&format!("cur.{key}.present set value 1b")));
    }

    #[test]
    fn expire_body_clears_only_the_durable_subject_copy() {
        let schema = schema();
        let commands = expire_macro_body(&schema);
        assert_eq!(commands.len(), 2);
        for command in &commands {
            assert!(command.contains("p$(subject)."));
            assert!(!command.contains("cur."));
        }
        assert!(commands[0].ends_with("set value 0b"));
        assert!(commands[1].ends_with("set value {}"));
    }

    #[test]
    fn read_accessors_name_the_static_staged_path_not_a_macro_path() {
        let snapshot = BoundedItemSnapshot::reconstruct(
            "SourceEvent",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
            "player_main_hand",
            SnapshotReliability::ExactPostTrigger,
        );
        for path in [
            snapshot.item_path(),
            snapshot.id_path(),
            snapshot.count_path(),
            snapshot.components_path(),
        ] {
            assert!(
                !path.as_str().contains("$("),
                "read accessors must be resolvable outside a macro line: {}",
                path.as_str()
            );
            assert!(path.as_str().starts_with("cur."));
        }
        assert!(!format!("{:?}", snapshot.is_present()).contains("$("));
    }

    #[test]
    fn reset_commands_clear_the_staged_path() {
        let snapshot = BoundedItemSnapshot::reconstruct(
            "SourceEvent",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
            "player_main_hand",
            SnapshotReliability::ExactPostTrigger,
        );
        let commands = snapshot.reset_commands();
        assert_eq!(commands.len(), 2);
        for command in &commands {
            assert!(command.contains("cur."));
            assert!(!command.starts_with('$'));
        }
    }

    #[test]
    fn is_present_and_is_absent_are_exact_negations() {
        let snapshot = BoundedItemSnapshot::reconstruct(
            "SourceEvent",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
            "player_main_hand",
            SnapshotReliability::ExactPostTrigger,
        );
        assert_eq!(
            snapshot.is_absent(),
            crate::condition::Condition::negate(snapshot.is_present())
        );
    }

    #[test]
    fn typed_field_paths_are_nested_under_the_item_compound() {
        let snapshot = BoundedItemSnapshot::reconstruct(
            "SourceEvent",
            ItemParticipantRole::Weapon,
            ParticipantHand::MainHand,
            "player_main_hand",
            SnapshotReliability::ExactPostTrigger,
        );
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
    fn subject_binding_allocates_lazily_then_stores_an_int() {
        let commands = bind_subject_commands("ns:alloc");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            "execute unless score @s sand_subj matches -2147483648.. run function ns:alloc"
        );
        // `int 1` matters: the substituted value must render as digits only,
        // which is what closes the macro-path injection surface.
        assert_eq!(
            commands[1],
            "execute store result storage sand:__bounded_item args.subject int 1 run scoreboard \
             players get @s sand_subj"
        );
    }

    #[test]
    fn slot_allocation_advances_a_shared_counter_exactly_once_per_subject() {
        assert_eq!(
            slot_alloc_body(),
            vec![
                "scoreboard players add #sand_subj_next sand_subj 1".to_string(),
                "scoreboard players operation @s sand_subj = #sand_subj_next sand_subj".to_string(),
            ]
        );
    }

    #[test]
    fn call_macro_invokes_with_the_bound_args_compound() {
        assert_eq!(
            call_macro("ns:persist"),
            "function ns:persist with storage sand:__bounded_item args"
        );
    }

    #[test]
    fn has_slot_guard_excludes_subjects_that_never_sourced_an_occurrence() {
        assert_eq!(
            has_slot_guard(),
            "if score @s sand_subj matches -2147483648.."
        );
    }

    #[test]
    fn repeated_body_generation_is_deterministic() {
        let schema = schema();
        assert_eq!(
            persist_macro_body(&schema, &source_snapshot()),
            persist_macro_body(&schema, &source_snapshot())
        );
        assert_eq!(load_macro_body(&schema), load_macro_body(&schema));
        assert_eq!(expire_macro_body(&schema), expire_macro_body(&schema));
    }
}
