use sand_macros::datapack_component;

#[datapack_component]
fn greet(name: &str) -> sand_core::McFunction {
    sand_core::McFunction::new(name.parse().unwrap())
}

fn main() {}
