use sand::api;

#[api(
    summary = "Selects a predicate state.",
    context = "States distinguish configured and pending predicates.",
    minecraft = "Only ready predicates are emitted.",
    use_when = ["Inspecting predicate readiness"],
    avoid_when = ["Tracking runtime state"],
    example = "Status::Ready",
    variants(Ready = "Marks a configured predicate.")
)]
pub enum Status { Ready, Pending }

fn main() {}
