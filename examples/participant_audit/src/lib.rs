//! Dedicated runtime-validation datapack for #230's participant backends
//! (#265) — every scenario writes deterministic, machine-readable evidence
//! to a typed scoreboard/storage schema rather than relying on tellraw
//! text, and every operation goes through the public `sand` façade's typed
//! command builders. There are no handwritten Minecraft command strings in
//! this file.
//!
//! Evidence schema: [`ParticipantAudit`] (`#[derive(SandStorage)]`, storage
//! `paudit:audit`). Scoreboards (all `dummy`, ≤16 chars, declared as
//! [`ScoreVar`]s below): `paudit_seq` (global occurrence sequence number,
//! mirrored into [`ParticipantAudit::sequence`] on every handler run),
//! `paudit_att1`/`paudit_att2` (per-handler invocation counts for the
//! two-handlers-share-one-occurrence scenario), `paudit_kill`, `paudit_wpn`,
//! `paudit_kwpn`.
//!
//! # Availability vs. presence
//!
//! [`ParticipantAvailability`] answers a build-time question — did this
//! event type's declared participant plan include this role at all? For
//! every role this pack observes, the answer is always yes (the plan is a
//! static property of the event type, not a per-tick runtime fact), so each
//! handler unwraps it with [`ParticipantAvailability::available`] in a
//! plain `let` binding rather than branching on it — no bespoke
//! `Vec<String>`-returning helper function needed.
//!
//! Whether an *item* snapshot actually captured something is a genuine
//! per-occurrence Minecraft-level fact (a player can swing with an empty
//! hand), so weapon snapshots are branched on with the typed
//! [`sand::execute_when::if_`] builder over [`ItemSnapshot::is_present`] —
//! ordinary Minecraft `execute if/unless`, not Rust control flow.

use sand::events::{
    EntityDamagePlayerEvent, EntityKillEvent, PlayerDamageEntityEvent, PlayerKillEvent,
};
use sand::participant::ParticipantAvailability;
use sand::prelude::*;

/// Typed evidence schema for every scenario this pack validates. Field
/// paths are flat (`state.<field>`) — nested schema support does not exist
/// in [`sand::SandStorage`] yet; see the derive macro's own docs.
#[derive(SandStorage)]
#[sand(storage = "paudit:audit", root = "state")]
#[allow(dead_code)] // fields are never constructed — they exist only to be
// named by the derive macro's generated per-field `StorageField` accessors.
struct ParticipantAudit {
    /// Global occurrence counter, mirrored from `paudit_seq` on every run.
    sequence: i32,
    attacker_present: bool,
    /// The correlated attacker's UUID, captured via `EntityParticipant::execute_at`.
    attacker_uuid: String,
    /// The victim's own UUID — captured alongside `attacker_uuid` so a
    /// human/automated review can confirm the two never collide.
    subject_uuid: String,
    /// Second independent handler's view of the same occurrence's attacker
    /// (see `audit_on_hurt_by_entity_b`) — proves the binding is stable
    /// across more than one handler reading the same event instance.
    attacker_b_uuid: String,
    killer_present: bool,
    killer_uuid: String,
    weapon_present: bool,
    weapon_item: String,
    kill_weapon_present: bool,
    kill_weapon_item: String,
}

// ── Scoreboards ─────────────────────────────────────────────────────────────

static SEQ: ScoreVar<i32> = ScoreVar::new("paudit_seq");
static ATT1: ScoreVar<i32> = ScoreVar::new("paudit_att1");
static ATT2: ScoreVar<i32> = ScoreVar::new("paudit_att2");
static KILL: ScoreVar<i32> = ScoreVar::new("paudit_kill");
static WPN: ScoreVar<i32> = ScoreVar::new("paudit_wpn");
static KWPN: ScoreVar<i32> = ScoreVar::new("paudit_kwpn");

/// Fake-player scoreboard holder for the global sequence counter — it has
/// no per-entity meaning, only a global tally, so it is not tied to `@s`.
const SEQ_HOLDER: &str = "audit_seq_holder";

fn bump_sequence() -> Vec<String> {
    vec![
        SEQ.add(SEQ_HOLDER, 1),
        SEQ.of(SEQ_HOLDER).store_into(ParticipantAudit::sequence()),
    ]
}

// ── Load-adjacent placeholder ────────────────────────────────────────────────

#[component]
fn root_advancement() -> Advancement {
    // Keeps `sand build`'s output non-empty even before any handler fires.
    Advancement::new("paudit:root".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

// ── Init ──────────────────────────────────────────────────────────────────────

/// Declares every objective and seeds storage to explicit absence. Run
/// manually over RCON (`function paudit:init`) as part of the validation
/// procedure — see `scripts/mc_validation/README.md`.
#[function]
pub fn init() {
    SEQ.define();
    ATT1.define();
    ATT2.define();
    KILL.define();
    WPN.define();
    KWPN.define();
    SEQ.set(SEQ_HOLDER, 0);
    ParticipantAudit::attacker_present().set(false);
    ParticipantAudit::killer_present().set(false);
    ParticipantAudit::weapon_present().set(false);
    ParticipantAudit::kill_weapon_present().set(false);
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `EntityDamagePlayerEvent` — correlated attacker. Two independent
/// handlers, both reading `.attacker()`, to validate that a same-occurrence
/// attacker binding is observable from more than one handler.
#[event]
pub fn audit_on_hurt_by_entity_a(event: Event<EntityDamagePlayerEvent>) {
    ATT1.add(Selector::self_(), 1);
    bump_sequence();
    let attacker = event
        .attacker()
        .available()
        .expect("EntityDamagePlayerEvent's participant plan always declares an attacker");
    ParticipantAudit::attacker_present().set(true);
    ParticipantAudit::subject_uuid().copy_from_entity(Selector::self_(), "UUID");
    attacker.execute_at(
        ParticipantAudit::attacker_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

#[event]
pub fn audit_on_hurt_by_entity_b(event: Event<EntityDamagePlayerEvent>) {
    ATT2.add(Selector::self_(), 1);
    let attacker = event
        .attacker()
        .available()
        .expect("EntityDamagePlayerEvent's participant plan always declares an attacker");
    attacker.execute_at(
        ParticipantAudit::attacker_b_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

/// `PlayerKillEvent` — correlated killer.
#[event]
pub fn audit_on_killed(event: Event<PlayerKillEvent>) {
    KILL.add(Selector::self_(), 1);
    bump_sequence();
    let killer = event
        .killer()
        .available()
        .expect("PlayerKillEvent's participant plan always declares a killer");
    ParticipantAudit::killer_present().set(true);
    killer.execute_at(ParticipantAudit::killer_uuid().copy_from_entity(Selector::self_(), "UUID"));
}

/// `PlayerDamageEntityEvent` — weapon (mainhand) snapshot.
#[event]
pub fn audit_on_hurt_entity(event: Event<PlayerDamageEntityEvent>) {
    WPN.add(Selector::self_(), 1);
    bump_sequence();
    let weapon = weapon_snapshot(event.weapon());
    if_(weapon.is_present())
        .then_all(mcfunction![
            ParticipantAudit::weapon_present().set(true);
            weapon.copy_to(ParticipantAudit::weapon_item());
        ])
        .else_all(mcfunction![ParticipantAudit::weapon_present().set(false);]);
}

/// `EntityKillEvent` — weapon (mainhand) snapshot on a killing blow.
#[event]
pub fn audit_on_killed_entity(event: Event<EntityKillEvent>) {
    KWPN.add(Selector::self_(), 1);
    bump_sequence();
    let weapon = weapon_snapshot(event.weapon());
    if_(weapon.is_present())
        .then_all(mcfunction![
            ParticipantAudit::kill_weapon_present().set(true);
            weapon.copy_to(ParticipantAudit::kill_weapon_item());
        ])
        .else_all(mcfunction![ParticipantAudit::kill_weapon_present().set(false);]);
}

/// Both weapon-snapshot events declare their weapon role unconditionally
/// (see the module doc's "availability vs. presence" note) — this just
/// names the shared unwrap so both handlers read identically.
fn weapon_snapshot(
    availability: ParticipantAvailability<sand::item::ItemSnapshot>,
) -> sand::item::ItemSnapshot {
    availability
        .available()
        .expect("this event's participant plan always declares a weapon snapshot")
}

// ── Export hook (required by `sand build`) ───────────────────────────────────

/// Invoked by the generated `sand_export` binary — mirrors
/// `examples/book_project`'s `__sand_export` exactly.
#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    let resolved = match sand::version::resolve_export_caps(mc_version) {
        Ok(resolved) => resolved,
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    };
    match sand::advanced::try_export_components_json_for_version(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    ) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    }
}
