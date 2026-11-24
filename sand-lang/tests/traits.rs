mod support;
use support::*;

#[test]
fn bounded_calls_reuse_concrete_borrowed_methods() {
    compile(&analyze_one(
        r#"
#![pack(namespace = "contracts", minecraft = "26.2")]
trait Described { fn describe(&self) -> string; }
trait Named { fn name(&self) -> string; }
trait Update { fn replace(&mut self, value: string) -> Self; }
struct Label { value: string }
impl Named for Label { fn name(&self) -> string { return self.value; } }
impl Described for Label { fn describe(&self) -> string { return "label:" + self.value; } }
impl Update for Label {
    fn replace(&mut self, value: string) -> Self {
        let old = self;
        self.value = value;
        return old;
    }
}
struct Bag<T> { values: Vec<T> }
impl<T> Described for Bag<T> {
    fn describe(&self) -> string { return self.values.len().to_string(); }
}
fn text<T: Described>(value: T) -> string { return value.describe(); }
fn both<T: Described + Named>(value: T) -> string { return text(value) + value.name(); }
fn run() {
    let mut value = Label { value: "ok" };
    let prior = value.replace("new");
    let label = both(prior);
    let description = text(Bag { values: [1, 2] });
}
"#,
    ));
}

#[test]
fn traits_reject_incomplete_and_incompatible_implementations() {
    for (body, message) in [
        ("impl Named for Label {}", "missing trait method"),
        (
            "impl Named for Label { fn name(&mut self) -> string { return self.value; } }",
            "receiver must match",
        ),
        (
            "impl Named for Label { fn name(&self) -> number { return 0; } }",
            "incompatible trait signature",
        ),
        (
            "impl Named for Label { fn name(&self, n: number) -> string { return self.value; } }",
            "incompatible trait signature",
        ),
        (
            "impl Named for Label { fn other(&self) -> string { return self.value; } }",
            "method is not in this trait",
        ),
        ("fn run() { let v = echo(3); }", "does not implement trait"),
        (
            "fn invalid<T: Label>(value: T) -> T { return value; }",
            "expected a trait",
        ),
    ] {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"contracts\")] struct Label {{ value: string }} trait Named {{ fn name(&self) -> string; }} fn echo<T: Named>(value: T) -> T {{ return value; }} {body}"
        ));
        let diagnostics = errors(&analysis);
        assert!(
            diagnostics.iter().any(|error| error.contains(message)),
            "{body}: {diagnostics:?}"
        );
    }
}

#[test]
fn native_trait_fixture_compiles() {
    let source = include_str!("fixtures/traits.sand");
    compile(&analyze_one(source));
    let formatted = sand_lang::format::format(source).unwrap();
    assert_eq!(sand_lang::format::format(&formatted).unwrap(), formatted);
    compile(&analyze_one(&formatted));
}
