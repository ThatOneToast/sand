use sand_macros::component;

#[component]
fn greet(name: &str) -> sand_core::McFunction {
    sand_core::McFunction::new(name.parse().unwrap())
}

fn main() {}
