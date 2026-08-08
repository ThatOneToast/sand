use sand::api;

#[api(
    summary = "Stores a predicate identifier.",
    context = "The wrapper would expose an unnamed public field.",
    minecraft = "The value identifies predicate JSON.",
    use_when = ["Never in this unsupported form"],
    avoid_when = ["A named field can explain its meaning"],
    example = "PredicateId(1)",
)]
pub struct PredicateId(pub u32);

fn main() {}
