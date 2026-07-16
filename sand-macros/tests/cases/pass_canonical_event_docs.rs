#![allow(refining_impl_trait)]

// Compile contract for the canonical public event documentation (#116):
// AdvancementEvent + Event<T>, typed SandEvent dispatch/lifecycle, a generic
// SandEvent family subscribed through a unit adapter, and same-cycle chaining.
use sand_core::condition::Condition;
use sand_core::events::{
    EventSetup, OnJoinEvent, PlayerSneakEvent, PlayerStartSneakingEvent, SandEvent,
    SandEventDispatch,
};
use sand_core::prelude::*;
use sand_macros::event;
use std::marker::PhantomData;

pub struct AteGoldenApple;

impl AdvancementEvent for AteGoldenApple {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
    }
}

#[event]
pub fn on_ate(event: Event<AteGoldenApple>) {
    cmd::tellraw(event.player(), Text::new("Golden apple eaten").gold());
}

// Generated built-ins share Event<T> context without claiming that their
// marker types implement AdvancementEvent.
#[event]
pub fn on_join(event: Event<OnJoinEvent>) {
    cmd::tellraw(event.player(), Text::new("Welcome"));
}

#[event]
pub fn on_sneak_start(event: Event<PlayerStartSneakingEvent>) {
    cmd::tellraw(event.subject(), Text::new("Sneaking"));
}

static CURRENT: ScoreVar<i32> = ScoreVar::new("docs_current_jump");
static PREVIOUS: ScoreVar<i32> = ScoreVar::new("docs_previous_jump");

pub struct PlayerJumped;

impl SandEvent for PlayerJumped {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(PREVIOUS.of("@s").lt_score(CURRENT.of("@s")))
            .into()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add docs_current_jump minecraft.custom:minecraft.jump"
                    .into(),
                "scoreboard objectives add docs_previous_jump dummy".into(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "execute as @a run scoreboard players operation @s docs_previous_jump = @s docs_current_jump"
                    .into(),
            ],
        }
    }
}

#[event]
pub fn on_jump(_event: PlayerJumped) {
    cmd::say("Jumped");
}

pub trait Direction {
    const TAG: &'static str;
}

pub struct Up;
impl Direction for Up {
    const TAG: &'static str = "up";
}

pub struct ElevatorUsed<D: Direction>(PhantomData<D>);

impl<D: Direction> SandEvent for ElevatorUsed<D> {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw(format!(
                "entity @s[tag=elevator_{}]",
                D::TAG
            )))
            .into()
    }
}

pub struct ElevatorGoingUp;
impl SandEvent for ElevatorGoingUp {
    fn dispatch() -> SandEventDispatch {
        ElevatorUsed::<Up>::dispatch()
    }
}

#[event]
pub fn on_elevator_up(_event: ElevatorGoingUp) {
    cmd::say("Going up");
}

pub struct JumpedOnElevator;

impl SandEvent for JumpedOnElevator {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::chain::<PlayerJumped>()
            .while_::<PlayerSneakEvent>()
            .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
            .into()
    }
}

#[event]
pub fn on_elevator_jump(_event: JumpedOnElevator) {
    cmd::say("Elevator jump");
}

fn main() {
    assert!(!on_ate().is_empty());
    assert!(!on_join().is_empty());
    assert!(!on_sneak_start().is_empty());
    assert!(!on_jump().is_empty());
    assert!(!on_elevator_up().is_empty());
    assert!(!on_elevator_jump().is_empty());
}
