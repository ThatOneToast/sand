mod support;
use support::*;

const VALUES: &str = r#"#![pack(namespace = "records", minecraft = "26.2")]
struct Point { x: number, y: number }
struct Shape { center: Point, visible: bool }
global state World { shape: Shape = Shape { center: Point { x: 2, y: 3 }, visible: true } }
player state Wallet { coins: number }
impl Point {
    pub fn new(x: number, y: number) -> Self { return Self { x, y }; }
    pub fn sum(self) -> number { return self.x + self.y; }
    pub fn shifted(mut self, dx: number) -> Self { self.x += dx; return self; }
}
impl Wallet {
    pub fn deposit(&mut self, amount: number) { self.coins += amount; }
    pub fn balance(&self) -> number { return Self.coins; }
}
fn transform(mut point: Point) -> Point { point.y += 4; return point; }
fn run() {
    let original = Point::new(2, 3);
    let moved = original.shifted(4);
    let mut copied = transform(moved);
    copied.x += original.y;
    World.shape.center = copied;
    World.shape.visible = moved.sum() > 0;
    for player in players() { Wallet.deposit(copied.sum()); let balance = Wallet.balance(); chat!("{balance}"); }
}
"#;

#[test]
fn records_and_impl_methods_share_calls_and_state_storage() {
    let analysis = analyze_one(VALUES);
    let pack = compile(&analysis);
    assert!(
        pack.resources
            .keys()
            .any(|p| p.ends_with("point.method/new.mcfunction")),
        "{:?}",
        pack.resources.keys()
    );
    assert!(
        pack.resources
            .keys()
            .any(|p| p.ends_with("wallet.method/deposit.mcfunction"))
    );
}

#[test]
fn records_reject_invalid_layouts_and_mutation() {
    for body in [
        "struct R { x: number } fn f() { let r = R { x: 1 }; r.x = 2; }",
        "struct R { x: number } fn f() { let r = R { y: 1 }; }",
        "struct R { x: number } fn f() { let r = R { x: true }; }",
        "struct R { x: R }",
        "#[tick] struct R { x: number }",
        "player state S { t: timer } impl S { fn f(&self) { start_timer(Self.t, ticks = 5); } }",
        "player state S { t: timer } impl S { fn f(&self) { stop_timer(Self.t); } }",
        "entity state S { x: number } impl S { fn f(&self) { detach_state(Self); } }",
        "struct R { x: number } impl R { fn f(&mut self) {} }",
        "struct R { x: number } impl R { fn f(self) { self.x = 2; } }",
        "player state S { x: number } impl S { fn f(&self) { self.x = 2; } }",
        "player state S { x: number } impl S { fn f(&self) { Self.g(); } fn g(&mut self) { self.x = 2; } }",
        "player state S { x: number } impl S { fn f(&self) -> number { return 1; } } #[load] fn run() { S.f(); }",
    ] {
        let source = format!("#![pack(namespace = \"records\", minecraft = \"26.2\")]\n{body}");
        let analysis = analyze_one(&source);
        assert!(analysis.has_errors(), "accepted {body}");
    }
}

#[test]
fn record_values_cover_nested_storage_recursion_events_and_evaluation_order() {
    compile(&analyze_one(include_str!("fixtures/structs.sand")));
}

#[test]
fn record_storage_is_declared_even_when_values_or_functions_are_unused() {
    compile(&analyze_one(
        r#"#![pack(namespace = "records", minecraft = "26.2")]
struct Point { x: number, y: number }
fn unused(p: Point) -> number { return p.x; }
fn run() { let unused = Point { x: 1, y: 2 }; let mut zero: Point; zero.y = 3; }
"#,
    ));
}

#[test]
fn implementations_and_associated_imports_can_live_in_another_module() {
    let analysis = analyze(&[
        (
            "pack.sand",
            r#"#![pack(namespace = "records", minecraft = "26.2")]
pub struct Point { x: number }
mod methods;
use Point::new;
fn run() { let p = new(2); let n = p.value(); }
"#,
        ),
        (
            "methods.sand",
            r#"use crate::Point;
impl Point {
    pub fn new(x: number) -> Self { return Self { x }; }
    pub fn value(self) -> number { return self.x; }
}"#,
        ),
    ]);
    compile(&analysis);
}

#[test]
fn record_fields_and_methods_enforce_module_privacy() {
    for access in [
        "let p = model::Point { x: 1 };",
        "let p = model::new(); let n = p.x;",
        "let p = model::new(); p.hidden();",
    ] {
        let root = format!("#![pack(namespace = \"records\")]\nmod model; fn run() {{ {access} }}");
        let analysis = analyze(&[
            ("pack.sand", &root),
            (
                "model.sand",
                "pub struct Point { x: number } pub fn new() -> Point { return Point { x: 1 }; } impl Point { fn hidden(self) {} }",
            ),
        ]);
        assert!(analysis.has_errors(), "accepted {access}");
    }
}
