mod support;
use support::*;

#[test]
fn vectors_pack_nested_values_and_share_state_calls_and_defaults() {
    compile(&analyze_one(
        r#"#![pack(namespace = "lists", minecraft = "26.2")]
struct Node { value: number, title: string, children: Vec<Node> }
global state World { nodes: Vec<Node> = [Node { value: 3, title: "root", children: [] }], empty: Vec<string> }
fn copy(values: Vec<Node>) -> Vec<Node> { return values; }
fn run() {
    let nodes = copy(World.nodes);
    let first = nodes[0];
    let title = first.title;
    let nested: Vec<Vec<number>> = [[1, 2], [3]];
    let value = nested[0][1];
    let missing = nodes[20];
    let defaults: Vec<bool>;
    if defaults == [] { World.empty = [title, "ok"]; }
}
"#,
    ));
}

#[test]
fn stored_elements_support_indexed_assignments_and_borrowed_methods() {
    compile(&analyze_one(
        r#"#![pack(namespace = "lists", minecraft = "26.2")]
struct Point { x: number, name: string, grid: [[number; 2]; 2] }
impl Point { fn move_by(&mut self, dx: number) -> number { self.x += dx; self.name += "!"; return self.x; } }
global state World { points: Vec<Point>, rows: [Vec<Point>; 2] }
fn run() {
    World.points = [Point { x: 1, name: "p", grid: [[1, 2], [3, 4]] }];
    let i = 0;
    World.points[i].x += 5;
    let changed = World.points[i].move_by(3);
    World.points[i].grid[i][1] = 8;
    World.rows[i] = World.points;
    World.rows[i][i].grid[1][i] += 9;
}
"#,
    ));
}

#[test]
fn generic_type_delimiters_format_without_changing_comparisons() {
    let source =
        "fn f(n:number){let values:Vec<Vec<number>>=[[n]];if n>=0{let other:Vec<number> = [n];}}";
    let formatted = sand_lang::format::format(source).unwrap();
    assert!(
        formatted.contains("Vec<Vec<number>> = [[n]];"),
        "{formatted}"
    );
    assert!(formatted.contains("if n >= 0"), "{formatted}");
    assert_eq!(sand_lang::format::format(&formatted).unwrap(), formatted);
}

#[test]
fn vectors_reject_invalid_types_and_scalar_operators() {
    for code in [
        "global state S { a: Vec<number, string> }",
        "global state S { a: Vec<Missing> }",
        "fn f() { let a: Vec<number> = [true]; }",
        "fn f() { let a: Vec<number>; if a in .. { } }",
        "fn f() { let a: string = \"a\"; if a in .. { } }",
        "fn f() { let mut a: Vec<number>; a += []; }",
        "fn f() { let a: Vec<number>; let b = a as number; }",
        "fn f() { let a: Vec<number>; a.push(1); }",
        "fn f() { let mut a: Vec<number>; let b = a.clear(); }",
        "fn f() { let mut a: Vec<number>; a.push(0..2); }",
        "fn f() { let a: Vec<number>; let b = a[0.5]; }",
        "fn f() { let a: Vec<number>; let b = a.slice(false, 2); }",
        "fn f() { let a: Vec<number>; let b: Vec<string> = a.slice(0, 2); }",
        "fn f() { let mut a: Vec<number>; for x in a.borrow_slice(0, 2) {} }",
        "fn f() { let a: Vec<number>; let b = a.borrow_slice(0, 2); }",
        "player state S { values: Vec<number> } impl S { fn bad(&self) { self.values.push(1); } }",
    ] {
        assert!(
            analyze_one(&format!("#![pack(namespace = \"lists\")] {code}")).has_errors(),
            "accepted {code}"
        );
    }
}

#[test]
fn immutable_local_vector_ranges_are_iterable_even_across_waits() {
    compile(&analyze_one(
        r#"
#![pack(namespace = "lists", minecraft = "26.2")]
global state Report { total: number }
fn run(values: Vec<number>, start: number, end: number) {
    for value in values.borrow_slice(start, end) {
        wait(1, server = true);
        Report.total += value;
    }
}
"#,
    ));
}

#[test]
fn native_vector_fixture_compiles() {
    compile(&analyze_one(include_str!("fixtures/sequences.sand")));
}

#[test]
fn expanded_defaults_stop_before_exponential_allocation() {
    let analysis = analyze_one(
        r#"#![pack(namespace = "limits")]
global state Data { nested: Vec<Vec<number>> = [[0; 1024]; 1024] }
"#,
    );
    assert!(
        errors(&analysis)
            .iter()
            .any(|message| message.contains("expanded State default"))
    );
    let piece = "x".repeat(2048);
    let analysis = analyze_one(&format!(
        r#"#![pack(namespace = "limits")]
global state Data {{ values: [string; 1024] = ["{piece}"; 1024] }}
"#
    ));
    assert!(
        errors(&analysis)
            .iter()
            .any(|message| message.contains("expanded State default"))
    );
}
