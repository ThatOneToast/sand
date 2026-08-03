use sand::api;

#[api(
    summary = "Creates a predicate.",
    context = "Predicates share conditions.",
    minecraft = "Creates predicate JSON.",
    use_when = ["Sharing conditions"],
    avoid_when = ["Mutable state"],
    example = "create()",
    stability = "forever",
)]
pub fn create() {}

fn main() {}
