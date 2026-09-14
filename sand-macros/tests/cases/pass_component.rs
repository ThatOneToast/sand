use sand_core::DatapackComponent;
use sand_core::{DatapackRegistration, IntoDatapack};
use sand_macros::datapack_component;

#[datapack_component]
pub fn my_advancement() -> sand_core::Advancement {
    use sand_core::{Advancement, AdvancementTrigger, Criterion};
    Advancement::new("test:my_adv".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

struct FutureFeature;

impl IntoDatapack for FutureFeature {
    fn into_datapack(self) -> DatapackRegistration {
        DatapackRegistration::new().component(sand_core::McFunction::new(
            "test:future_feature".parse().unwrap(),
        ))
    }
}

#[datapack_component]
fn future_feature() -> FutureFeature {
    FutureFeature
}

fn main() {
    let adv = my_advancement();
    assert_eq!(adv.resource_location().to_string(), "test:my_adv");
}
