use sand::api;

#[api(
    path = "sand::predicate::make_predicate",
    aliases = ["sand::prelude::make_predicate"],
    summary = "Creates a reusable Minecraft predicate resource.",
    context = "Predicates name conditions that multiple generated resources can share.",
    minecraft = "Creates predicate JSON evaluated whenever Minecraft references it.",
    use_when = ["Sharing a condition between resources"],
    avoid_when = ["Representing mutable runtime state"],
    params(id = "The namespaced predicate identifier."),
    returns = "The canonical identifier for the generated predicate.",
    example = r#"make_predicate("demo:wearing_boots")"#,
    availability = ["all features"],
)]
pub fn make_predicate(id: &str) -> &str {
    id
}

pub struct Predicate;

impl Predicate {
    #[api(
        path = "sand::predicate::Predicate::test",
        summary = "Tests a predicate against the current command source.",
        context = "Predicate tests let command plans reuse a named vanilla condition.",
        minecraft = "Emits an execute-if-predicate condition.",
        use_when = ["Branching on a reusable predicate"],
        avoid_when = ["The condition is only used once"],
        params(id = "The namespaced predicate identifier."),
        returns = "Whether the generated command condition succeeds.",
        example = r#"Predicate::test("demo:wearing_boots")"#,
    )]
    pub fn test(&self, id: &str) -> bool {
        !id.is_empty()
    }
}

pub trait PredicateExt {
    #[api(
        path = "sand::predicate::PredicateExt::matches",
        summary = "Converts a predicate into a reusable match condition.",
        context = "Extensions keep author code expressed in typed predicate terms.",
        minecraft = "Produces a vanilla predicate condition.",
        use_when = ["Composing typed execute conditions"],
        avoid_when = ["Mutating runtime state"],
        params(id = "The namespaced predicate identifier."),
        returns = "Whether this value matches the predicate contract.",
        example = r#"value.matches("demo:wearing_boots")"#,
    )]
    fn matches(&self, id: &str) -> bool;
}

fn main() {
    assert_eq!(make_predicate("demo:test"), "demo:test");
}
