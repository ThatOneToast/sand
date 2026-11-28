mod support;
use sand_lang::library::Library;
use support::*;

fn with_library(source: &str) -> sand_lang::Analysis {
    analyze(&[
        ("pack.sand", source),
        ("collections.sand", Library::Collections.source()),
    ])
}

#[test]
fn generic_collections_are_normal_sand_modules_with_owned_values() {
    compile(&with_library(
        r#"#![pack(namespace = "maps", minecraft = "26.2")]
mod collections;
use collections::{Map, Set};
struct Point { x: number, label: string, values: [number; 2] }
global state Data { map: Map<string, Point>, set: Set<Point> }
fn run() {
    let mut numbers = Map::<string, number>::new();
    numbers.insert("key", 3);
    let value = numbers.get_or("key", 0);
    let entries = numbers.entries();
    let keys = numbers.keys();
    let values = numbers.values();
    let removed = numbers.remove_or("key", 0);
    let changed = numbers.remove("missing");
    let empty = numbers.is_empty();
    numbers.clear();
    let point = Point { x: 1, label: "p", values: [2, 3] };
    Data.map.insert("point", point);
    Data.set.insert(point);
    let contains = Data.set.contains(point);
    let result = Data.set.remove(point);
    let mut arrays = Set::<[number; 2]>::new();
    arrays.insert([1, 2]); arrays.remove([1, 2]);
    let mut records = Map::<Point, Vec<string>>::new();
    records.insert(point, ["a", "b"]);
    let copy = records;
    records.remove(point);
}
"#,
    ));
}

#[test]
fn whole_value_equality_uses_the_same_nominal_layout_and_snapshots() {
    compile(&analyze_one(
        r#"#![pack(namespace = "maps", minecraft = "26.2")]
struct Point { x: number, text: string, values: Vec<[number; 2]> }
fn copy(point: Point) -> Point { return point; }
fn run(point: Point, values: [string; 2]) {
    let same = point == copy(point);
    let changed = point != Point { x: 3, text: "p", values: [[1, 2]] };
    let arrays = values == ["a", "b"];
    let empty: [number; 0] = []; if empty == [] { }
}
"#,
    ));
    for source in [
        "let a = Point { x: 1 }; let b = Other { x: 1 }; if a == b {}",
        "let a = Point { x: 1 }; if a < a {}",
        "let a = [1, 2]; if a == [1] {}",
    ] {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"maps\")] struct Point {{ x: number }} struct Other {{ x: number }} fn run() {{ {source} }}"
        ));
        assert!(analysis.has_errors(), "accepted {source}");
    }
}

#[test]
fn ore_journal_builds_with_the_bundled_library() {
    let library = include_str!("../../examples/lang_maps/collections.sand");
    assert_eq!(library, Library::Collections.source());
    let analysis = with_library(include_str!("../../examples/lang_maps/pack.sand"));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    compile(&analysis);
}
