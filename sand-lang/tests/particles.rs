mod support;

use support::*;

#[test]
fn runtime_particles_use_typed_macro_arguments_and_guard_every_operand() {
    let analysis = analyze_one(
        r#"#![pack(namespace = "effects", minecraft = "26.2")]
fn run(count: number, spread: float, speed: float) {
    spawn_particle(Particle::Heart, count = count, spread = spread, speed = speed);
}
"#,
    );
    let pack = compile(&analysis);
    assert!(pack.resources.values().any(|bytes| {
        String::from_utf8_lossy(bytes).contains(
            "$particle minecraft:heart ~ ~1 ~ $(spread) $(spread) $(spread) $(speed) $(count)",
        )
    }));
    let caller = function(&pack, "effects/function/run");
    assert!(caller.contains("storage effects:sand.args count int 1"));
    assert!(caller.contains("storage effects:sand.args spread double 0.001"));
    assert!(caller.contains("storage effects:sand.args speed double 0.001"));
    let guard = caller
        .lines()
        .find(|line| line.contains("with storage effects:sand.args"))
        .unwrap();
    assert!(guard.contains("matches 0..100000 "));
    assert_eq!(guard.matches("matches 0..1000000").count(), 2);
}

#[test]
fn particles_reject_out_of_range_literals_and_fractional_counts() {
    for options in [
        "count = -1",
        "count = 100001",
        "count = 2.5",
        "spread = -1.0",
        "speed = 1001",
    ] {
        let source = format!(
            r#"#![pack(namespace = "effects")] fn f() {{ spawn_particle(Particle::Heart, {options}); }}"#
        );
        assert!(analyze_one(&source).has_errors(), "{options}");
    }
}
