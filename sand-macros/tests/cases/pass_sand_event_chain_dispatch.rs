// Validation case for #240: same-cycle chained SandEvent dispatch via
// `SandEventDispatch::chain::<Parent>()`, including a generic child family
// sharing one parent (each concrete monomorphization keeps a distinct
// identity — same property #239 established for `tick()`, now proven for
// `chain()`).
use sand_core::condition::Condition;
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;
use std::marker::PhantomData;

static JUMPS: ScoreVar<i32> = ScoreVar::new("cdt_jumps");
static SYNC_JUMPS: ScoreVar<i32> = ScoreVar::new("cdt_sync_jumps");

pub struct PlayerJumpEvent;

impl SandEvent for PlayerJumpEvent {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(SYNC_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
            .into()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add cdt_jumps minecraft.custom:minecraft.jump".to_string(),
                "scoreboard objectives add cdt_sync_jumps dummy".to_string(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "scoreboard players operation @a cdt_sync_jumps = @a cdt_jumps".to_string(),
            ],
        }
    }
}

pub struct JumpedOnElevator;

impl SandEvent for JumpedOnElevator {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::chain::<PlayerJumpEvent>()
            .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
            .into()
    }
}

#[event]
pub fn on_player_jump(event: PlayerJumpEvent) {
    cmd::say("jumped!");
}

#[event]
pub fn on_jumped_on_elevator(event: JumpedOnElevator) {
    cmd::say("jumped on elevator!");
}

// ── Generic child family sharing one parent ─────────────────────────────────

pub trait ElevatorDirection {
    const NAME: &'static str;
}
pub struct GoUp;
pub struct GoDown;
impl ElevatorDirection for GoUp {
    const NAME: &'static str = "up";
}
impl ElevatorDirection for GoDown {
    const NAME: &'static str = "down";
}

pub struct ElevatorUsed<D: ElevatorDirection>(PhantomData<D>);

impl<D: ElevatorDirection> SandEvent for ElevatorUsed<D> {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::chain::<PlayerJumpEvent>()
            .when(Condition::raw(format!(
                "entity @s[tag=going_{}]",
                D::NAME
            )))
            .into()
    }
}

pub struct ElevatorGoingUp;
impl SandEvent for ElevatorGoingUp {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        ElevatorUsed::<GoUp>::dispatch()
    }
}
pub struct ElevatorGoingDown;
impl SandEvent for ElevatorGoingDown {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        ElevatorUsed::<GoDown>::dispatch()
    }
}

#[event]
pub fn on_elevator_up(event: ElevatorGoingUp) {
    cmd::say("going up");
}

#[event]
pub fn on_elevator_down(event: ElevatorGoingDown) {
    cmd::say("going down");
}

fn main() {
    // Distinct concrete monomorphizations of the generic child family keep
    // distinct identity, same property #239 established for tick().
    assert_ne!(
        std::any::TypeId::of::<ElevatorUsed<GoUp>>(),
        std::any::TypeId::of::<ElevatorUsed<GoDown>>()
    );

    let mut found_jumped_on_elevator = false;
    let mut up_id = None;
    let mut down_id = None;
    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if let sand_core::EventDispatch::Custom {
            make_chain,
            event_type_id,
            ..
        } = descriptor.dispatch
        {
            if descriptor.path == "on_jumped_on_elevator" {
                let chain = make_chain().expect("chain dispatch should be registered");
                assert_eq!(
                    (chain.parent_type_id)(),
                    std::any::TypeId::of::<PlayerJumpEvent>()
                );
                assert_eq!((chain.parent_type_name)(), std::any::type_name::<PlayerJumpEvent>());
                assert_eq!(chain.when.len(), 1);
                found_jumped_on_elevator = true;
            } else if descriptor.path == "on_elevator_up" {
                up_id = Some(event_type_id());
            } else if descriptor.path == "on_elevator_down" {
                down_id = Some(event_type_id());
            }
        }
    }
    assert!(found_jumped_on_elevator);
    let up_id = up_id.expect("on_elevator_up must register Custom dispatch");
    let down_id = down_id.expect("on_elevator_down must register Custom dispatch");
    assert_ne!(up_id, down_id);
}
