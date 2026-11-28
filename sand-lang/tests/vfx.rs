mod support;
use sand::advanced::compiler::Compiler;
use support::*;

const SOURCE: &str = include_str!("fixtures/vfx.sand");

#[test]
fn named_vfx_reuses_one_ordered_spatial_function() {
    let analysis = analyze_one(SOURCE);
    let pack = compile(&analysis);
    assert_eq!(
        function(&pack, "vfx_probe/function/perform"),
        "function vfx_probe:flourish"
    );
    let effect = function(&pack, "vfx_probe/function/flourish");
    let lines = effect.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].starts_with("particle minecraft:happy_villager ~-0.5 ~1 ~"));
    assert!(lines[1].starts_with("particle minecraft:happy_villager ~0.5 ~1 ~"));
    assert!(
        lines[2].contains("playsound minecraft:entity.player.levelup master @a ~ ~ ~ 0.75 1.25")
    );
    assert!(lines[3].starts_with("particle minecraft:heart ~0 ~1 ~0 0.25 0 0.25"));
    assert!(lines[3].contains("force"));
    let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&decoded).unwrap().resources
    );
}

#[test]
fn declared_vfx_checks_shape_and_native_ranges() {
    for (old, replacement, expected) in [
        (
            "[0.5, 1.0, 0.0]",
            "[0.5, 1.0]",
            "expected three coordinates",
        ),
        ("count: 4", "count: 100001", "expected a value from 0"),
        ("pitch: 1.25", "pitch: 3.0", "pitch"),
        (
            "force: true",
            "sound: Sound::EntityPlayerLevelup",
            "either a particle or a sound",
        ),
        ("play_vfx(Flourish)", "play_vfx(perform)", "a VFX sequence"),
    ] {
        let analysis = analyze_one(&SOURCE.replacen(old, replacement, 1));
        let diagnostics = analysis.compile().unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{diagnostics:?}"
        );
    }
}
