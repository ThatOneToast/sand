use sand_macros::function;

#[function]
fn bad() {
    if true {
        cmd::say("hello")
    } else {
        cmd::say("goodbye")
    };
}

fn main() {}
