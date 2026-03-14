use sand_macros::function;

#[function]
fn greet(name: &str) {
    mcfunction! {
        "say hello";
    }
}

fn main() {}
