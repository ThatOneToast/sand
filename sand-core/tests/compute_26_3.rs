//! Maintained generated-command fixture consumed by validate_compute_26_3.py.
#![cfg(sand_codegen_26_3)]
use sand_core::cmd::{self, BlockPos, Target};

#[test]
fn generated_compute_commands_and_runtime_fixture() {
    let sum = cmd::compute_default_integer(r#"{type:"minecraft:add",inputs:[7,5]}"#).to_string();
    let float = cmd::compute_default_float(r#"{type:"minecraft:constant",value:-1.25}"#)
        .scale(10.0)
        .to_string();
    let root = cmd::compute_default_float(r#"{type:"minecraft:sqrt",input:81.0}"#).to_string();
    let score = cmd::compute_entity_integer(
        Target::self_(),
        r#"{type:"minecraft:score",target:{type:"minecraft:context",target:"target_entity"},score:"compute_probe"}"#,
    ).to_string();
    let block =
        cmd::compute_block_integer(BlockPos::here(), r#"{type:"minecraft:constant",value:3}"#)
            .to_string();
    let fail = cmd::compute_default_integer(r#"{type:"minecraft:div",left:1,right:0}"#).to_string();
    assert_eq!(
        sum,
        r#"compute default integer {type:"minecraft:add",inputs:[7,5]}"#
    );
    assert_eq!(
        float,
        r#"compute default float {type:"minecraft:constant",value:-1.25} 10"#
    );
    assert_eq!(
        block,
        r#"compute block ~ ~ ~ integer {type:"minecraft:constant",value:3}"#
    );
    let lines = vec![
        // Forceload requests are asynchronous; elapsed ticks do not prove that
        // the target chunk is available on a cold CI server.
        "execute unless loaded 0 64 0 run return run schedule function compute_probe:run 1t".into(),
        "scoreboard objectives add compute_probe dummy".into(),
        "kill @e[type=minecraft:marker,tag=compute_probe]".into(),
        "summon minecraft:marker 0 64 0 {Tags:[\"compute_probe\",\"probe_a\"]}".into(),
        "summon minecraft:marker 0 64 0 {Tags:[\"compute_probe\",\"probe_b\"]}".into(),
        "scoreboard players set @e[tag=probe_a] compute_probe 11".into(),
        "scoreboard players set @e[tag=probe_b] compute_probe 29".into(),
        format!("execute store result score #sum compute_probe run {sum}"),
        format!("execute store result score #float compute_probe run {float}"),
        format!("execute store result score #root compute_probe run {root}"),
        format!("execute positioned 0 64 0 store result score #block compute_probe run {block}"),
        format!("execute as @e[type=minecraft:marker,tag=compute_probe] store result score @s compute_probe run {score}"),
        "scoreboard players set #success compute_probe 99".into(),
        format!("execute store success score #success compute_probe run {fail}"),
        "execute if score #sum compute_probe matches 12 if score #float compute_probe matches -13 if score #root compute_probe matches 9 if score #block compute_probe matches 3 if score #success compute_probe matches 0 if entity @e[tag=probe_a,scores={compute_probe=11}] if entity @e[tag=probe_b,scores={compute_probe=29}] run say __SAND_COMPUTE_PASS__".into(),
        "execute unless score #sum compute_probe matches 12 run say COMPUTE_BAD_SUM".into(),
        "execute unless score #float compute_probe matches -13 run say COMPUTE_BAD_FLOAT".into(),
        "execute unless score #root compute_probe matches 9 run say COMPUTE_BAD_ROOT".into(),
        "execute unless score #block compute_probe matches 3 run say COMPUTE_BAD_BLOCK".into(),
        "execute unless score #success compute_probe matches 0 run say COMPUTE_BAD_FAILURE".into(),
        "execute unless entity @e[tag=probe_a,scores={compute_probe=11}] run say COMPUTE_BAD_ENTITY_A".into(),
        "execute unless entity @e[tag=probe_b,scores={compute_probe=29}] run say COMPUTE_BAD_ENTITY_B".into(),
        "kill @e[type=minecraft:marker,tag=compute_probe]".into(),
        "scoreboard objectives remove compute_probe".into(),
        "say __SAND_COMPUTE_END__".into(),
    ];
    if let Some(output) = std::env::var_os("SAND_COMPUTE_PROOF_PACK") {
        let root = std::path::PathBuf::from(output);
        let functions = root.join("data/compute_probe/function");
        std::fs::create_dir_all(&functions).unwrap();
        std::fs::write(root.join("pack.mcmeta"), r#"{"pack":{"description":"Sand compute validation","min_format":121,"max_format":121}}"#).unwrap();
        std::fs::write(functions.join("run.mcfunction"), lines.join("\n")).unwrap();
        std::fs::write(
            functions.join("loaded.mcfunction"),
            "say __SAND_COMPUTE_LOADED__",
        )
        .unwrap();
        let tags = root.join("data/minecraft/tags/function");
        std::fs::create_dir_all(&tags).unwrap();
        std::fs::write(
            tags.join("load.json"),
            r#"{"values":["compute_probe:loaded"]}"#,
        )
        .unwrap();
    }
}
