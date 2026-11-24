mod support;

use support::*;

const SCENE: &str = include_str!("../../examples/lang_signal_scene/pack.sand");

#[test]
fn scenes_emit_native_displays_and_interpolation_through_the_same_runtime() {
    let analysis = analyze_one(SCENE);
    let pack = compile(&analysis);
    for expected in [
        "execute summon minecraft:block_display",
        "execute summon minecraft:item_display",
        "execute summon minecraft:text_display",
        "interpolation_duration:10",
        "translation:[-0.25f,0f,-0.25f]",
        "minecraft:enchantment_glint_override",
    ] {
        assert!(
            pack.resources.iter().any(|(name, content)| {
                name.ends_with(".mcfunction") && String::from_utf8_lossy(content).contains(expected)
            }),
            "missing {expected}"
        );
    }
    let portable = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&portable, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn scenes_check_content_names_and_poses() {
    for (from, to, expected) in [
        (
            "name: \"icon\", item: SupplyIcon",
            "name: \"icon\", item: SupplyIcon, block: Block::Stone",
            "needs exactly one",
        ),
        ("part: \"icon\"", "part: \"missing\"", "unknown part"),
        ("at: 5, ", "", "needs `at`"),
        (
            "scale: [0.5, 0.5, 0.5]",
            "scale: [-1, 0.5, 0.5]",
            "nonnegative scale",
        ),
    ] {
        let analysis = analyze_one(&SCENE.replace(from, to));
        let errors = analysis.compile().expect_err("invalid scene compiled");
        assert!(
            errors
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{errors:?}"
        );
    }
}
