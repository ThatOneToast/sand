use std::fs;
use std::process::Command;

#[test]
fn player_data_feature_exposes_its_lifecycle_dependency_and_compatibility_alias() {
    let project = tempfile::tempdir().expect("create feature-closure fixture");
    let target = tempfile::tempdir().expect("create isolated feature-closure target");
    let sand = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir(project.path().join("src")).expect("create fixture source directory");
    fs::write(
        project.path().join("Cargo.toml"),
        format!(
            "[package]\nname = \"player-data-feature-closure\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\n[workspace]\n\n[dependencies]\nsand = {{ path = {:?}, default-features = false, features = [\"systems-player-data\"] }}\n",
            sand
        ),
    )
    .expect("write fixture manifest");
    fs::write(
        project.path().join("src/lib.rs"),
        r#"
use sand::prelude::PlayerSchema as PreludePlayerSchema;
use sand::systems::lifecycle::{FirstJoinCommands, RespawnCommands};
use sand::systems::player_data::PlayerSchema as ModulePlayerSchema;

pub fn compatibility_schemas() -> (PreludePlayerSchema, ModulePlayerSchema) {
    (PreludePlayerSchema::new("prelude"), ModulePlayerSchema::new("module"))
}

pub fn lifecycle_helpers() -> (FirstJoinCommands, RespawnCommands) {
    (FirstJoinCommands::new("joined"), RespawnCommands::new("dead"))
}
"#,
    )
    .expect("write fixture source");
    let output = Command::new(env!("CARGO"))
        .current_dir(project.path())
        .env("CARGO_TARGET_DIR", target.path())
        .args(["check", "--offline"])
        .output()
        .expect("check player-data feature closure fixture");

    assert!(
        output.status.success(),
        "systems-player-data feature closure failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
