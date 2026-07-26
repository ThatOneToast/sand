use std::process::Command;

use sha2::{Digest, Sha256};

fn export() -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_sand_export"))
        .env("SAND_EXPORT_MC_VERSION", "26.2")
        .output()
        .expect("sand_export must run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn repeated_exports_are_byte_identical() {
    let first = export();
    let second = export();
    assert_eq!(first, second);
    assert_eq!(
        format!("{:x}", Sha256::digest(&first)),
        format!("{:x}", Sha256::digest(&second))
    );
}

#[test]
fn export_contains_the_vertical_runtime() {
    let records: Vec<serde_json::Value> =
        serde_json::from_slice(&export()).expect("valid component JSON");
    let functions: Vec<_> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .collect();
    for fragment in [
        "/provision",
        "/initialize",
        "/derive_refresh",
        "/refresh",
        "/transitions",
        "/migrate",
        "/cleanup",
    ] {
        assert!(
            functions.iter().any(|record| record["path"]
                .as_str()
                .unwrap_or_default()
                .ends_with(fragment)),
            "missing generated entity function ending in {fragment}"
        );
    }
    let all_commands = functions
        .iter()
        .filter_map(|record| record["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all_commands.contains("attribute @s minecraft:max_health base set"));
    assert!(all_commands.contains("attribute @s minecraft:attack_damage base set"));
    assert!(all_commands.contains("data modify entity @s CustomName set value"));
    assert!(all_commands.contains("data remove storage rpg:__sand_entity"));
}
