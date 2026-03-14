use sand_core::DatapackComponent;
use sand_macros::component;

#[component]
pub fn my_advancement() -> sand_core::Advancement {
    use sand_core::{Advancement, AdvancementTrigger, Criterion};
    Advancement::new("test:my_adv".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

fn main() {
    let adv = my_advancement();
    assert_eq!(adv.resource_location().to_string(), "test:my_adv");
}
