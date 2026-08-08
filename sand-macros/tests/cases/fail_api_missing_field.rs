use sand::api;

#[api(
    summary = "Stores predicate configuration.",
    context = "The fields configure one predicate resource.",
    minecraft = "The values affect emitted predicate JSON.",
    use_when = ["Building a predicate"],
    avoid_when = ["Tracking runtime state"],
    example = "Config { id: 1, enabled: true }",
    fields(id = "Identifies the predicate resource.")
)]
pub struct Config { pub id: u32, pub enabled: bool }

fn main() {}
