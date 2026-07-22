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
//! Participant accessors (`event.attacker()`, `event.killer()`, …) are
//! infallible (#273): a role an event's plan does not declare would be a
//! build-time authoring bug (the plan is a static property of the event
//! type, not a per-tick runtime fact), caught by `sand build`'s mandatory
//! graph validation, not something ordinary handler code branches on.
//!
//! Whether an *item* snapshot actually captured something is a genuine
//! per-occurrence Minecraft-level fact (a player can swing with an empty
//! hand), so weapon snapshots are branched on with the typed
//! [`sand::execute_when::if_`] builder over [`ItemSnapshot::is_present`] —
//! ordinary Minecraft `execute if/unless`, not Rust control flow.

use sand::events::{
    EntityDamagePlayerEvent, EntityKillEvent, PlayerDamageEntityEvent, PlayerKillEvent, SandEvent,
    SandEventDispatch, SandEventParticipants,
};
use sand::participant::{EntityParticipantRole, ItemParticipantRole, ParticipantBuilder, ParticipantHand};
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
    /// `ComposedAttackerParent`'s own direct capture (#264 same-cycle
    /// composition scenario, see below).
    compose_parent_uuid: String,
    /// `ComposedAttackerChild`'s view of the *same* occurrence's attacker,
    /// via `inherit_entity::<ComposedAttackerParent>` — proves the child
    /// resolves to the same binding, not a fresh/empty one of its own.
    compose_child_uuid: String,
    /// `ComposedAttackerSibling`'s view of the same occurrence — a second,
    /// independent same-cycle child inheriting from the same parent, proves
    /// more than one dependent can observe the borrowed binding.
    compose_sibling_uuid: String,
    /// `SpecialKillEvent`'s view (#269) of the killer it inherits from its
    /// advancement-bridge parent `PlayerKillEvent`.
    bridge_killer_uuid: String,
    bridge_weapon_present: bool,
    bridge_weapon_item: String,
}

// ── Scoreboards ─────────────────────────────────────────────────────────────

static SEQ: ScoreVar<i32> = ScoreVar::new("paudit_seq");
static ATT1: ScoreVar<i32> = ScoreVar::new("paudit_att1");
static ATT2: ScoreVar<i32> = ScoreVar::new("paudit_att2");
static KILL: ScoreVar<i32> = ScoreVar::new("paudit_kill");
static WPN: ScoreVar<i32> = ScoreVar::new("paudit_wpn");
static KWPN: ScoreVar<i32> = ScoreVar::new("paudit_kwpn");
/// Manually-set trigger for the #264 same-cycle composition scenario below
/// (`scoreboard players set @s paudit_cmp_trg 1` over RCON/in-game).
static COMPOSE_TRIGGER: ScoreVar<i32> = ScoreVar::new("paudit_cmp_trg");

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
    COMPOSE_TRIGGER.define();
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
    let attacker = event.attacker();
    ParticipantAudit::attacker_present().set(true);
    ParticipantAudit::subject_uuid().copy_from_entity(Selector::self_(), "UUID");
    attacker.execute_at(
        ParticipantAudit::attacker_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

#[event]
pub fn audit_on_hurt_by_entity_b(event: Event<EntityDamagePlayerEvent>) {
    ATT2.add(Selector::self_(), 1);
    let attacker = event.attacker();
    attacker.execute_at(
        ParticipantAudit::attacker_b_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

/// `PlayerKillEvent` — correlated killer.
#[event]
pub fn audit_on_killed(event: Event<PlayerKillEvent>) {
    KILL.add(Selector::self_(), 1);
    bump_sequence();
    let killer = event.killer();
    ParticipantAudit::killer_present().set(true);
    killer.execute_at(ParticipantAudit::killer_uuid().copy_from_entity(Selector::self_(), "UUID"));
}

/// `PlayerDamageEntityEvent` — weapon (mainhand) snapshot.
#[event]
pub fn audit_on_hurt_entity(event: Event<PlayerDamageEntityEvent>) {
    WPN.add(Selector::self_(), 1);
    bump_sequence();
    let weapon = event.weapon();
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
    let weapon = event.weapon();
    if_(weapon.is_present())
        .then_all(mcfunction![
            ParticipantAudit::kill_weapon_present().set(true);
            weapon.copy_to(ParticipantAudit::kill_weapon_item());
        ])
        .else_all(mcfunction![ParticipantAudit::kill_weapon_present().set(false);]);
}

// ── Same-cycle composition scenario (#264) ─────────────────────────────────
//
// `ComposedAttackerParent` captures the attacker directly (a plain custom
// `SandEvent`, manually triggered via `paudit_cmp_trg` for a controlled,
// repeatable scenario). `ComposedAttackerChild` and `ComposedAttackerSibling`
// are both same-cycle chain children (`SandEventDispatch::chain::<...>()`)
// that borrow the parent's binding via `inherit_entity` instead of
// capturing their own — zero extra setup/cleanup commands are generated for
// either child (see `EventParticipantPlan::inherit_entity`'s doc). Custom
// `SandEvent` handlers get the identical `event.attacker()`-shaped accessor
// `AdvancementEvent` handlers use, via the blanket `SandEventParticipants`
// impl (#273) — no more manually-typed `EventParticipantPlan::resolve` calls
// naming the type and role by hand.

struct ComposedAttackerParent;
impl SandEvent for ComposedAttackerParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(COMPOSE_TRIGGER.of("@s").eq(1))
    }
    fn participants() -> sand::participant::EventParticipantPlan {
        ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Attacker)
            .build()
    }
}

struct ComposedAttackerChild;
impl SandEvent for ComposedAttackerChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<ComposedAttackerParent>()
    }
    fn participants() -> sand::participant::EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<ComposedAttackerParent>(EntityParticipantRole::Attacker)
            .build()
    }
}

struct ComposedAttackerSibling;
impl SandEvent for ComposedAttackerSibling {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<ComposedAttackerParent>()
    }
    fn participants() -> sand::participant::EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<ComposedAttackerParent>(EntityParticipantRole::Attacker)
            .build()
    }
}

#[event]
pub fn audit_on_composed_parent(event: ComposedAttackerParent) {
    COMPOSE_TRIGGER.set(Selector::self_(), 0);
    let attacker = event.attacker();
    attacker.execute_at(
        ParticipantAudit::compose_parent_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

#[event]
pub fn audit_on_composed_child(event: ComposedAttackerChild) {
    let attacker = event.attacker();
    attacker.execute_at(
        ParticipantAudit::compose_child_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

#[event]
pub fn audit_on_composed_sibling(event: ComposedAttackerSibling) {
    let attacker = event.attacker();
    attacker.execute_at(
        ParticipantAudit::compose_sibling_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
}

// ── Advancement-bridge composition scenario (#269) ─────────────────────────
//
// `SpecialKillEvent` is a plain `SandEvent` chained after `PlayerKillEvent`
// — an `AdvancementEvent` — through the same-cycle advancement bridge
// (#240 Phase 6). It inherits the killer entity `PlayerKillEvent`'s own
// `AdvancementEvent::participants()` plan declares, proving a bridge
// parent's plan is now actually applied around its synthesized entry
// (previously it was silently never applied at all). It also directly
// observes its own weapon snapshot — `@s` is the victim in `PlayerKillEvent`
// (the killer is only reachable via correlated observation, never an exact
// hand snapshot), so the weapon here is *this event's own* mainhand
// capture, composed alongside the inherited entity in one plan. `PlayerKillEvent`
// itself has no direct `#[event]` handler here — #240 Phase 6 still
// requires a bridged advancement-backed parent's sole dependents to be
// same-cycle chain children, not a mix of a direct handler and graph
// composition.

struct SpecialKillEvent;
impl SandEvent for SpecialKillEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<PlayerKillEvent>()
    }
    fn participants() -> sand::participant::EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<PlayerKillEvent>(EntityParticipantRole::Killer)
            .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .build()
    }
}

#[event]
pub fn audit_on_special_kill(event: SpecialKillEvent) {
    let killer = event.killer();
    killer.execute_at(
        ParticipantAudit::bridge_killer_uuid().copy_from_entity(Selector::self_(), "UUID"),
    );
    let weapon = event.weapon();
    if_(weapon.is_present())
        .then_all(mcfunction![
            ParticipantAudit::bridge_weapon_present().set(true);
            weapon.copy_to(ParticipantAudit::bridge_weapon_item());
        ])
        .else_all(mcfunction![ParticipantAudit::bridge_weapon_present().set(false);]);
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
