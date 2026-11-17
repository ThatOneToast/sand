mod support;
use support::*;

#[test]
fn arrays_share_state_record_call_and_borrowed_receiver_layouts() {
    compile(&analyze_one(
        r#"#![pack(namespace = "arrays", minecraft = "26.2")]
const SIZE: number = 3;
struct Point { x: number }
impl Point { fn add(&mut self, dx: number) { self.x += dx; } }
global state World { values: [number; SIZE] = [1, 2, 3], points: [Point; 2] }
fn rotate(values: [number; 3]) -> [number; 3] { return [values[2], values[0], values[1]]; }
fn run() {
    let mut values = rotate(World.values);
    let i = 1;
    values[i] += 4;
    World.values = values;
    World.points[0].x = values[0];
    World.points[i].add(values[i]);
    let mut grid: [[float; 2]; 2] = [[1.0, 2.0], [3.0, 4.0]];
    grid[i][0] *= 1.5;
    let empty: [number; 0] = [];
}
"#,
    ));
}

#[test]
fn arrays_validate_size_types_indexes_and_mutability() {
    for code in [
        "fn f() { let a: [number; 2] = [1]; }",
        "fn f() { let a = [1, true]; }",
        "fn f() { let a = [1, 2]; let n = a[2]; }",
        "fn f() { let a = [1, 2]; a[0] = 4; }",
        "fn f() { let mut a = [1, 2]; a[0.5] = 4; }",
        "fn f() { let a: [number; 1025]; }",
        "const SIZE: bool = true; global state S { values: [number; SIZE] }",
        "struct Recursive { children: [Recursive; 2] }",
    ] {
        let source = format!("#![pack(namespace = \"arrays\")] {code}");
        assert!(analyze_one(&source).has_errors(), "accepted {code}");
    }
}

#[test]
fn native_collection_fixture_compiles() {
    compile(&analyze_one(include_str!("fixtures/collections.sand")));
}

#[test]
fn array_iteration_snapshots_values_and_keeps_loop_control() {
    compile(&analyze_one(
        r#"#![pack(namespace = "arrays", minecraft = "26.2")]
global state Result { total: number }
fn first() -> number { for value in [2, 3] { return value; } return 0; }
fn run() {
    let values = [1, 2, 3];
    for value in values { if value == 2 { continue; } Result.total += value; if value == 3 { break; } }
    for values in values { Result.total += values; }
    let matrix = [[1, 2], [3, 4]];
    for row in matrix { for value in row { Result.total += value; } }
    let repeated = [first(); 3];
    Result.total += repeated[1];
}
"#,
    ));
}
