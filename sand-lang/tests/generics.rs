mod support;
use support::*;

#[test]
fn generic_functions_specialize_owned_values_and_recursive_calls() {
    compile(&analyze_one(
        r#"#![pack(namespace = "generic", minecraft = "26.2")]
struct Point { x: number, title: string }
fn identity<T>(value: T) -> T { return value; }
fn first<T>(values: Vec<T>, fallback: T) -> T { return values.get_or(0, fallback); }
fn recursive<T>(value: T, n: number) -> T { if n == 0 { return value; } return recursive(value, n - 1); }
fn empty<T>() -> Vec<T> { let result: Vec<T>; return result; }
fn run() {
    let number = identity(3);
    let text = identity("hi");
    let point = identity(Point { x: 3, title: text });
    let array = identity([1, 2]);
    let points: Vec<Point> = [point];
    let copied = recursive(points, 3);
    let defaulted = first([1, 2], 3);
    let strings: Vec<string> = empty();
    let explicit = empty::<number>();
    let nested = identity(identity(point));
}
"#,
    ));
}

#[test]
fn generic_calls_reject_conflicting_missing_and_invalid_type_arguments() {
    for source in [
        "let bad = same(1, true);",
        "let bad = make();",
        "let bad = make::<number, string>();",
        "let bad = make::<Missing>();",
        "let bad = same::<number>(true, false);",
        "let bad = random::<number>(0..2);",
        "plain::<number>();",
    ] {
        let analysis = analyze_one(&format!(
            r#"#![pack(namespace = "generic")]
fn same<T>(a: T, b: T) -> T {{ return a; }}
fn make<T>() -> T {{ let value: T; return value; }}
fn plain() {{ }}
fn run() {{ {source} }}"#
        ));
        assert!(analysis.has_errors(), "accepted {source}");
    }
}

#[test]
fn specialization_references_keep_one_authored_identity() {
    use sand_lang::{FileId, ide};
    let source = r#"#![pack(namespace = "generic")]
fn copy<T>(value: T) -> T { let saved = value; return saved; }
fn run() { let n = copy(3); let text = copy("x"); }
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    for (offset, new_name, count) in [
        (source.find("copy<T>").unwrap(), "identity", 3),
        (source.find("saved =").unwrap(), "result", 2),
        (source.find("value: T").unwrap(), "input", 2),
        (source.find("<T>").unwrap() + 1, "Element", 3),
    ] {
        let edits = ide::rename(&analysis, FileId(0), offset, new_name).unwrap();
        assert_eq!(edits.len(), count, "{new_name}: {edits:?}");
        let mut renamed = source.to_owned();
        for edit in edits.iter().rev() {
            renamed.replace_range(edit.span.start..edit.span.end, &edit.text);
        }
        compile(&analyze_one(&renamed));
    }
    let outline = ide::document_symbols(&analysis, FileId(0));
    assert_eq!(
        outline
            .iter()
            .filter(|symbol| symbol.name == "copy")
            .count(),
        1
    );
    let formatted = sand_lang::format::format(source).unwrap();
    assert!(formatted.contains("copy<T>(value: T)"), "{formatted}");
    assert_eq!(sand_lang::format::format(&formatted).unwrap(), formatted);
}

#[test]
fn generic_records_share_owned_layouts_defaults_and_nested_inference() {
    compile(&analyze_one(
        r#"#![pack(namespace = "generic", minecraft = "26.2")]
struct Entry<K, V> { key: K, value: V }
struct Node<T> { value: T, children: Vec<Node<T>> }
struct Bag<T> { entries: Vec<T>, name: string }
global state Data { entry: Entry<string, number> = Entry { key: "a", value: 2 }, roots: Vec<Node<number>> }
fn identity<T>(value: T) -> T { return value; }
fn key<K, V>(entry: Entry<K, V>) -> K { return entry.key; }
fn run() {
    let inferred = Entry { key: "title", value: [1, 2] };
    let explicit = Entry::<number, string> { key: 1, value: "one" };
    let copied = identity(inferred);
    let title = key(copied);
    let Entry { key: renamed, value: [first, second] } = copied;
    let mut bag: Bag<Entry<number, string>> = Bag { entries: [explicit], name: title };
    bag.entries[0].value += "!";
    Data.entry = Entry { key: renamed, value: first + second };
    Data.roots = [Node { value: 3, children: [] }];
    let nested: Node<string> = Node { value: "root", children: [Node { value: "leaf", children: [] }] };
}
"#,
    ));
}

#[test]
fn generic_records_reject_invalid_layouts_types_and_missing_arguments() {
    for source in [
        "struct Pair<T> { value: T } global state S { pair: Pair }",
        "struct Pair<T> { value: T } fn run() { let bad: Pair<number> = Pair { value: true }; }",
        "struct Pair<T> { value: T } fn run() { let bad = Pair::<number, string> { value: 2 }; }",
        "struct Pair<T> { value: T } fn run() { let bad: Pair<number, string>; }",
        "struct Pair<T> { value: T } fn run() { let bad: Pair<Missing>; }",
        "struct Node<T> { value: T, next: Node<T> } fn run() { let bad: Node<number>; }",
    ] {
        let analysis = analyze_one(&format!("#![pack(namespace = \"generic\")] {source}"));
        assert!(analysis.has_errors(), "accepted {source}");
    }
}

#[test]
fn generic_impls_support_borrowed_receivers_self_calls_and_method_parameters() {
    compile(&analyze_one(
        r#"#![pack(namespace = "generic", minecraft = "26.2")]
struct Bag<T> { values: Vec<T> }
impl<T> Bag<T> {
    fn len(&self) -> number { return self.values.len(); }
    fn push(&mut self, value: T) -> bool { return self.values.push(value); }
    fn first(&self, fallback: T) -> T { return self.values.get_or(0, fallback); }
    fn repeated(value: T, n: number) -> Self { let mut result: Self = Self { values: [] }; for i in 0..n { result.push(value); } return result; }
    fn copy(&self) -> Self { return self; }
    fn choose<U>(&self, a: U, b: U) -> U { return match self.len() { 0 => a, _ => b }; }
}
fn numbers(n: number) -> Bag<number> { return Bag { values: [n] }; }
fn run() {
    let mut numbers = numbers(2);
    numbers.push(3);
    let number = numbers.first(0);
    let mut text: Bag<string> = Bag { values: [] };
    text.push("x");
    let chosen = text.choose("empty", "full");
    let flag = numbers.choose(false, true);
    let copied = text.copy();
}
"#,
    ));
}

#[test]
fn qualified_generic_constructors_and_explicit_method_types_compose() {
    compile(&analyze_one(
        r#"#![pack(namespace = "generic", minecraft = "26.2")]
struct Bag<T> { values: Vec<T> }
impl<T> Bag<T> {
    fn new() -> Self { return Self { values: [] }; }
    fn push(&mut self, value: T) { self.values.push(value); }
    fn get(&self, fallback: T) -> T { return self.values.get_or(0, fallback); }
    fn empty<U>(&self) -> Vec<U> { let result: Vec<U>; return result; }
    fn again() -> Self { return Self::new(); }
}
fn run() {
    let mut values = Bag::<number>::new();
    values.push(4);
    let copied = Bag::<string>::again();
    let nested = Bag::<Vec<number>>::new();
    let empty = values.empty::<string>();
    let same: Vec<number> = values.empty();
    let first = values.get(0);
}
"#,
    ));
}

#[test]
fn native_generic_fixture_compiles_and_formats_without_changes_in_behavior() {
    let source = include_str!("fixtures/generics.sand");
    compile(&analyze_one(source));
    let formatted = sand_lang::format::format(source).unwrap();
    assert_eq!(sand_lang::format::format(&formatted).unwrap(), formatted);
    compile(&analyze_one(&formatted));
}

#[test]
fn generic_impls_and_inference_work_across_modules_with_source_navigation() {
    use sand_lang::{FileId, ide};
    let root = r#"#![pack(namespace = "generic")]
mod types; mod operations;
use types::Bag;
fn run() { let mut values = Bag::<number>::new(); values.push(3); let n = operations::count(values); }
"#;
    let types = "pub struct Bag<T> { pub values: Vec<T> }";
    let operations = r#"use crate::types::Bag;
impl<T> Bag<T> {
    pub fn new() -> Self { return Self { values: [] }; }
    pub fn push(&mut self, value: T) { self.values.push(value); }
}
pub fn count<T>(value: Bag<T>) -> number { return value.values.len(); }
"#;
    let analysis = analyze(&[
        ("pack.sand", root),
        ("types.sand", types),
        ("operations.sand", operations),
    ]);
    compile(&analysis);
    let edits = ide::rename(
        &analysis,
        FileId(2),
        operations.find("impl<T>").unwrap() + 5,
        "Element",
    )
    .unwrap();
    assert_eq!(edits.len(), 3, "{edits:?}");
    assert!(edits.iter().all(|edit| edit.file == FileId(2)), "{edits:?}");
    let (file, span) =
        ide::definition(&analysis, FileId(0), root.find("push(3)").unwrap()).unwrap();
    assert_eq!(file, FileId(2));
    assert_eq!(&operations[span.start..span.end], "push");
    let pack = compile(&analysis);
    assert_eq!(
        pack.resources,
        compile(&analyze(&[
            ("pack.sand", root),
            ("types.sand", types),
            ("operations.sand", operations)
        ]))
        .resources
    );
}

#[test]
fn generic_expansion_limits_and_invalid_applications_are_diagnostics() {
    for source in [
        "struct Box<T> { value: T } impl<U> Box<number> { fn value(&self) -> number { return self.value; } }",
        "struct Node<T> { children: Vec<Node<Vec<T>>> } global state S { node: Node<number> }",
        "fn grow<T>(value: T) { grow([value].to_vec()); } fn run() { grow(1); }",
        "fn run() { let n = 1; let x: number::<number> = 2; }",
        "fn run() { let n = 1; let x = n.to_string::<string>(); }",
        "fn run() { let mut values: Vec<number>; values.push::<number>(2); }",
        "fn run() { if is_sneaking::<number>() { } }",
        "struct Point { x: number } fn run() { let value = Point::<number> { x: 2 }; }",
    ] {
        let analysis = analyze_one(&format!("#![pack(namespace = \"generic\")] {source}"));
        assert!(analysis.has_errors(), "accepted {source}");
    }
}

#[test]
fn generic_signature_help_and_call_result_completion_use_concrete_types() {
    use sand_lang::{FileId, ide};
    let source = r#"#![pack(namespace = "generic")]
struct Entry<K, V> { key: K, value: V }
struct Bag<T> { values: Vec<T> }
impl<T> Bag<T> { fn new(value: T) -> Self { return Self { values: [value] }; } fn first(&self, fallback: T) -> T { return self.values.get_or(0, fallback); } }
fn identity<T>(value: T) -> T { return value; }
fn consume(value: Entry<string, number>, count: number) { }
fn run() {
    let entry = Entry { key: "x", value: 3 };
    consume(entry, 1);
    let bag = Bag::<Entry<string, number>>::new(entry);
    let value = identity(entry).value;
    let key = bag.first(entry).key;
}
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let help = ide::signature_help(
        &analysis,
        FileId(0),
        source.find("consume(entry, 1").unwrap() + "consume(entry, ".len(),
    )
    .unwrap();
    assert_eq!(help.params.len(), 2, "{help:?}");
    assert_eq!(help.active, 1);
    assert!(help.label.contains("Entry<string, number>"), "{help:?}");
    let explicit = ide::signature_help(
        &analysis,
        FileId(0),
        source.find("::new(entry)").unwrap() + "::new(".len(),
    )
    .unwrap();
    assert_eq!(explicit.params.len(), 1, "{explicit:?}");
    for (call, field) in [
        ("identity(entry).value", "value"),
        ("bag.first(entry).key", "key"),
    ] {
        let end = source.find(call).unwrap() + call.rfind('.').unwrap() + 1;
        assert!(
            ide::completions(&analysis, FileId(0), end)
                .iter()
                .any(|completion| completion.label == field),
            "missing {field} after {call}"
        );
    }
}

#[test]
fn type_parameter_qualified_calls_keep_parameter_references_for_rename() {
    use sand_lang::{FileId, ide};
    let source = r#"#![pack(namespace = "generic")]
struct Point { x: number }
impl Point { fn new() -> Self { return Self { x: 3 }; } }
fn make<T>() -> T { return T::new(); }
fn run() { let point = make::<Point>(); }
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let edits = ide::rename(
        &analysis,
        FileId(0),
        source.find("make<T>").unwrap() + 5,
        "Value",
    )
    .unwrap();
    assert_eq!(edits.len(), 3, "{edits:?}");
    let mut renamed = source.to_owned();
    for edit in edits.iter().rev() {
        renamed.replace_range(edit.span.start..edit.span.end, &edit.text);
    }
    compile(&analyze_one(&renamed));
}
