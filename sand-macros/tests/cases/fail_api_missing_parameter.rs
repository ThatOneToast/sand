use sand::api;

#[api(
    path = "sand::predicate::create",
    summary = "Creates a reusable predicate.",
    context = "Predicates share vanilla conditions.",
    minecraft = "Creates predicate JSON.",
    use_when = ["Sharing conditions"],
    avoid_when = ["Mutable state"],
    params(id = "The predicate identifier."),
    returns = "A predicate value.",
    example = "create(id, condition)",
)]
pub fn create(id: &str, condition: bool) -> bool { condition && !id.is_empty() }

fn main() {}
