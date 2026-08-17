use sand_core::events::{
    BoundedEventDependency, SameCycleEventDependency, SameCycleEventRequirement,
    SandEvent, SandEventDispatch,
};

struct Parent;

impl SandEvent for Parent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
    }
}

fn main() {
    let mut dispatch = SandEventDispatch::chain::<Parent>();
    dispatch.occurrence.clear();
    let _ = std::mem::size_of::<BoundedEventDependency>();
    let _ = std::mem::size_of::<SameCycleEventDependency>();
    let _ = std::mem::size_of::<SameCycleEventRequirement>();
}
