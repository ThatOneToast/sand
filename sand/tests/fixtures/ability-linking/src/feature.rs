use crate::schema::Reservoir;
use sand::prelude::*;
use sand::systems::ability::{Ability, ResourceMaintenance, ResourcePool};

pub fn ability(player: EntityContext<PlayerKind>) -> Ability {
    let state = Reservoir::on(player);
    Ability::new(
        ResourcePool::new(state.value, 32),
        state.cooldown,
        8,
        Ticks::new(16),
        player
            .inventory()
            .mainhand()
            .matches(vanilla::Item::Stick)
            .unwrap(),
    )
    .unwrap()
}

pub fn maintenance() -> ResourceMaintenance {
    ResourceMaintenance::new(
        ResourceLocation::local("recharge").unwrap(),
        Target::players().each(|player| {
            let state = Reservoir::on(*player);
            ResourcePool::new(state.value, 32)
                .regenerate(state.timer, 2, Ticks::new(10))
                .unwrap()
        }),
    )
}
