use sand::api;

#[api(
    path = "sand::predicate::create",
    summary = "Creates a reusable predicate.",
    context = "Predicates share vanilla conditions.",
    minecraft = "Creates predicate JSON.",
    use_when = ["Sharing conditions"],
    avoid_when = ["Mutable state"],
    params(id = "The predicate identifier.", condition = "A parameter that does not exist."),
    returns = "A predicate value.",
    example = "create(id)",
)]
pub fn create(id: &str) -> &str { id }

fn main() {}
