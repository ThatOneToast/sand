// Canonical typed SandEvent: structured tick dispatch via SandEventDispatch::tick(),
// built from Sand's typed Condition/ScoreVar IR (not a hand-formatted string), plus
// owned lifecycle setup (objectives + post-observation sync).
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;

static JUMPS: ScoreVar<i32> = ScoreVar::new("mtst_jumps");
static SYNC_JUMPS: ScoreVar<i32> = ScoreVar::new("mtst_sync_jumps");

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
                "scoreboard objectives add mtst_jumps minecraft.custom:minecraft.jump"
                    .to_string(),
                "scoreboard objectives add mtst_sync_jumps dummy".to_string(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "scoreboard players operation @a mtst_sync_jumps = @a mtst_jumps".to_string(),
            ],
        }
    }
}

#[event]
pub fn on_player_jump(event: PlayerJumpEvent) {
    cmd::say("jumped!");
}

fn main() {
    let commands = on_player_jump();
    assert!(commands.iter().any(|cmd| cmd.contains("say jumped!")));

    let mut found = false;
    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if descriptor.path == "on_player_jump" {
            if let sand_core::EventDispatch::Custom {
                make_tick,
                make_setup,
                event_type_id,
                ..
            } = descriptor.dispatch
            {
                let tick = make_tick().expect("typed tick dispatch should be registered");
                let plans = tick.execution_plans();
                let sand_core::events::TickExecutionPlans::Plans(plans) = plans else {
                    panic!("expected Plans, got Unconditional");
                };
                assert_eq!(plans.len(), 1);
                assert_eq!(plans[0], vec!["if score @s mtst_sync_jumps < @s mtst_jumps".to_string()]);

                let setup = make_setup();
                assert_eq!(setup.objectives.len(), 2);
                assert_eq!(setup.post_observation.len(), 1);

                assert_eq!(event_type_id(), std::any::TypeId::of::<PlayerJumpEvent>());
                found = true;
            } else {
                panic!("bare SandEvent handler must use Custom dispatch");
            }
        }
    }
    assert!(found);
}
