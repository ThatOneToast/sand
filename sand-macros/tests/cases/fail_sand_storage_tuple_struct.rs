use sand_macros::SandStorage;

#[derive(SandStorage)]
#[sand(storage = "test:data", root = "root")]
pub struct BadSchema(i32, String);

fn main() {}
