use sand::api;

#[api(
    path = "sand::predicate::create",
    summary = "Creates a predicate from one condition.",
    context = "Predicates let generated resources share conditions.",
    minecraft = "Creates predicate JSON.",
    use_when = ["Sharing conditions"],
    avoid_when = ["Mutable state"],
    example = "create()",
)]
pub fn create() {}

#[api(
    path = "sand::predicate::create",
    summary = "Builds a predicate from one condition.",
    context = "Predicates let generated resources share conditions.",
    minecraft = "Creates predicate JSON.",
    use_when = ["Sharing conditions"],
    avoid_when = ["Mutable state"],
    example = "build()",
)]
pub fn build() {}

fn main() {}
