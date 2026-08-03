use sand::api;

#[api(
    summary = "Builds a predicate.",
    context = "The function creates one reusable predicate.",
    minecraft = "It emits predicate JSON.",
    use_when = ["Sharing a condition"],
    avoid_when = ["Tracking mutable state"],
    example = "build()",
    fields(value = "Functions do not have fields.")
)]
pub fn build() {}

fn main() {}
