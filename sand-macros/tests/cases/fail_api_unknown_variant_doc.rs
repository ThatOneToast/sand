use sand::api;

#[api(
    summary = "Selects a predicate state.",
    context = "States distinguish configured predicates.",
    minecraft = "Ready predicates are emitted.",
    use_when = ["Inspecting predicate readiness"],
    avoid_when = ["Tracking runtime state"],
    example = "Status::Ready",
    variants(Ready = "Marks a configured predicate.", Missing = "Does not exist.")
)]
pub enum Status { Ready }

fn main() {}
