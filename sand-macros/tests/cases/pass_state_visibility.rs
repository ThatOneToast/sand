#![deny(private_interfaces)]

use sand::prelude::*;

mod private_state {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, EntityStateEnum)]
    enum PrivatePhase {
        Ready,
    }

    #[derive(State)]
    #[state(namespace = "visibility", scope = player)]
    struct PrivateState {
        phase: EntityEnum<PrivatePhase>,
    }

    pub fn commands() -> Vec<String> {
        let bound: PrivateStateBound = PrivateState::on(PlayerContext::default());
        bound.phase.set(PrivatePhase::Ready)
    }
}

mod crate_state {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, EntityStateEnum)]
    pub(crate) enum CratePhase {
        Ready,
    }

    #[derive(State)]
    #[state(namespace = "visibility", scope = player)]
    pub(crate) struct CrateState {
        phase: EntityEnum<CratePhase>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EntityStateEnum)]
pub enum PublicPhase {
    Ready,
}

#[derive(State)]
#[state(namespace = "visibility", scope = player)]
pub struct PublicState {
    phase: EntityEnum<PublicPhase>,
}

fn main() {
    let _ = private_state::commands();

    let crate_bound: crate_state::CrateStateBound =
        crate_state::CrateState::on(PlayerContext::default());
    let _ = crate_bound.phase.set(crate_state::CratePhase::Ready);

    let public_bound: PublicStateBound = PublicState::on(PlayerContext::default());
    let _ = public_bound.phase.set(PublicPhase::Ready);
}
