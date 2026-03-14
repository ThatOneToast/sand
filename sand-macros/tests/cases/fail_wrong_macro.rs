use sand_macros::function;

// A function that returns a wrong type should fail.
#[function]
fn my_func() {
    "not a vec"
}

fn main() {}
