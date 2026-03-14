use sand_macros::function;

// A trailing expression that isn't IntoIterator<Item = String> should fail.
#[function]
fn wrong_return_type() {
    42i32
}

fn main() {}
