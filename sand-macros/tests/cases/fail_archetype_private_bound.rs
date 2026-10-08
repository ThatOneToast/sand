use sand::prelude::*;
mod hidden {
    use super::*;
    #[derive(Archetype)]
    #[archetype(id = "test:private", entity = Marker)]
    struct Private;
}
fn main() { let _: hidden::PrivateBound; }
