//! Architecture guard for #273/#280 item 3: bare `SandEvent`-backed
//! `#[event]` handlers get the same infallible `.entity`/`.attacker`/
//! `.killer`/`.victim`/`.interacted_entity`/`.weapon` accessor sugar
//! `Event<E: AdvancementEvent>` handlers have, reachable from
//! `use sand::prelude::*;` alone — no separate
//! `use sand::events::SandEventParticipants;` required (only the concrete
//! built-in vanilla event marker type, `PlayerKillEvent`, needs its own
//! explicit import, same as any other built-in event type). This is the
//! exact canonical example from the #273/#269 design:
//! a bare `SandEvent` (`SpecialKillEvent`) chained after an `AdvancementEvent`
//! (`PlayerKillEvent`) through the same-cycle advancement bridge (#269),
//! inheriting a killer entity and directly observing its own weapon
//! snapshot, with both handler forms present in one file.

use sand::events::PlayerKillEvent;
use sand::prelude::*;

pub struct SpecialKillEvent;

impl SandEvent for SpecialKillEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<PlayerKillEvent>()
    }

    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<PlayerKillEvent>(EntityParticipantRole::Killer)
            .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .build()
    }
}

#[event]
fn direct_kill(event: Event<PlayerKillEvent>) {
    let killer = event.killer();
    let _ = killer.selector();
}

#[event]
fn special_kill(event: SpecialKillEvent) {
    // `.killer()`/`.weapon()` resolve here purely through `SandEventParticipants`,
    // brought into scope by the glob prelude import above — no explicit
    // `use sand::events::SandEventParticipants;` anywhere in this file.
    let killer = event.killer();
    let weapon = event.weapon();
    let _ = (killer.selector(), weapon.storage());
}

#[test]
fn prelude_alone_resolves_bare_sand_event_participant_accessors() {
    let json = sand::advanced::try_export_components_json("preludespecial")
        .expect("export must succeed through the facade");
    assert!(json.contains("direct_kill"));
    assert!(json.contains("special_kill"));
}
