use sand::api;

#[api(
    summary = "Stores a predicate name.",
    context = "The static would expose global predicate state.",
    minecraft = "Does not directly produce Minecraft output.",
    use_when = ["Never"],
    avoid_when = ["Always"],
    example = "PREDICATE",
)]
pub static PREDICATE: &str = "demo:test";

fn main() {}
