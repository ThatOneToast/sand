use sand::api;

#[api(
    summary = "Stores predicate configuration.",
    context = "The field configures one predicate resource.",
    minecraft = "The value affects emitted predicate JSON.",
    use_when = ["Building a predicate"],
    avoid_when = ["Tracking runtime state"],
    example = "Config { id: 1 }",
    fields(id = "Identifies the resource.", missing = "Does not exist.")
)]
pub struct Config { pub id: u32 }

fn main() {}
