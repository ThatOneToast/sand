mod support;
use support::*;

#[test]
fn motion_reads_and_guarded_vector_write_use_native_entity_data() {
    let pack = compile(&analyze_one(include_str!("fixtures/entity_motion.sand")));
    let commands = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, body)| std::str::from_utf8(body).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for expected in [
        "data get entity @s Motion[0] 1000",
        "data get entity @s Motion[1] 1000",
        "data get entity @s Motion[2] 1000",
        "execute if entity @s[type=!minecraft:player] run data modify entity @s Motion set value",
    ] {
        assert!(commands.contains(expected), "missing {expected}");
    }
}

#[test]
fn invalid_literal_motion_has_a_source_diagnostic() {
    let analysis =
        analyze_one("#![pack(namespace = \"motion\")] fn f() { set_velocity(0, 10.5, 0); }");
    assert!(
        errors(&analysis)
            .iter()
            .any(|message| message.contains("from -10 to 10 blocks per tick"))
    );
}
