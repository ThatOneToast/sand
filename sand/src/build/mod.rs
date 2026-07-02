mod config;
pub mod package;
pub mod records;
mod resourcepack;
pub mod validate;
pub mod validate_output;
pub mod write;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use colored::Colorize;

use crate::config::SandConfig;
use crate::pack_format::pack_format_for;

use config::{cargo_target_dir, resolve_mc_version};
use package::zip_dir;
use records::ComponentRecord;
use resourcepack::build_resourcepack;
use validate::validate_component_records_for_project;
use write::{write_component, write_pack_mcmeta};

pub fn run(release: bool, resourcepack: bool) -> Result<()> {
    // 1. Read sand.toml
    let project_root = std::env::current_dir()?;
    let config_path = project_root.join("sand.toml");
    if !config_path.exists() {
        bail!("sand.toml not found — run `sand build` from your project root");
    }
    let config: SandConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)
        .context("failed to parse sand.toml")?;

    // Resolve mc_version ("latest" → actual version from Mojang manifest)
    let mc_version = resolve_mc_version(&config.pack.mc_version);

    // Resolve pack format: explicit override in sand.toml wins; otherwise derive
    // from the version profile.  If the version is not in the known table the
    // profile is a conservative fallback and we warn the user.
    let (pack_format, format_is_fallback) = {
        use sand_core::version::{MinecraftVersion, VersionProfile};
        if let Some(explicit) = config.pack.pack_format {
            (explicit, false)
        } else if let Ok(v) = MinecraftVersion::parse(&mc_version) {
            let p = VersionProfile::resolve(&v).unwrap_or_else(|_| {
                VersionProfile::resolve(
                    &MinecraftVersion::parse(sand_core::version::LATEST_KNOWN).unwrap(),
                )
                .unwrap()
            });
            let meta = p.datapack_metadata();
            (meta.pack_format, meta.is_fallback)
        } else {
            (pack_format_for(&mc_version), false)
        }
    };

    if format_is_fallback {
        eprintln!(
            "{} Minecraft version '{}' is not in Sand's known version table. \
             Using pack_format {} as a conservative fallback. \
             Add `pack_format = {}` to [pack] in sand.toml to silence this warning.",
            "warning:".yellow().bold(),
            mc_version,
            pack_format,
            pack_format
        );
    }

    println!(
        "{} {} (Minecraft {}, pack_format {})...",
        "Building".cyan().bold(),
        config.pack.namespace.as_str().white().bold(),
        mc_version.yellow(),
        pack_format.to_string().yellow()
    );

    // 2. Compile the export binary
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "--bin", "sand_export"]);
    if release {
        cmd.arg("--release");
    }
    // Suppress all compiler warnings during the build — the export binary is a
    // build-time tool, not user-facing code, so warning noise is unhelpful here.
    cmd.env("RUSTFLAGS", "-Awarnings");
    let status = cmd.status().context("failed to invoke `cargo build`")?;
    if !status.success() {
        bail!("`cargo build` failed");
    }

    // 3. Run the export binary
    let target_dir = cargo_target_dir()?;
    let profile = if release { "release" } else { "debug" };
    let binary = target_dir.join(profile).join("sand_export");
    let output = std::process::Command::new(&binary)
        .output()
        .with_context(|| format!("failed to run '{}'", binary.display()))?;
    if !output.status.success() {
        bail!(
            "export binary failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // 4. Parse component records
    let records: Vec<ComponentRecord> = serde_json::from_slice(&output.stdout).map_err(|e| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let hint = if stdout.contains("export_resourcepack_json") {
            "\n\nHint: it looks like __sand_export is calling \
             export_resourcepack_json. Resource pack output must go in \
             __sand_resource_export (src/bin/sand_resource_export.rs), \
             not in the datapack export. Remove the \
             sand_resourcepack::export_resourcepack_json call from \
             __sand_export in src/lib.rs."
        } else if stdout.trim_start().starts_with('[') && stdout.matches('[').count() > 1 {
            "\n\nHint: the export binary printed more than one JSON value. \
             __sand_export must print exactly one JSON array \
             (from sand_core::export_components_json). Resource pack \
             output belongs in __sand_resource_export instead."
        } else {
            ""
        };
        anyhow::anyhow!("failed to parse component export JSON: {}{}", e, hint)
    })?;

    // 5. Validate every record before creating the output directory.  A build
    // must fail before it produces a partially valid datapack.
    let dist = PathBuf::from("dist").join(config.pack.namespace.as_str());
    validate_component_records_for_project(&dist, &project_root, &records)?;

    // 6. Write pack.mcmeta
    std::fs::create_dir_all(&dist)?;
    write_pack_mcmeta(
        &dist,
        config.pack.namespace.as_str(),
        &config.pack.description,
        pack_format,
    )?;

    // 7. Write each component file
    for record in &records {
        write_component(&dist, &project_root, record)?;
    }

    println!(
        "{} {} component(s) written to {}",
        "Done!".green().bold(),
        records.len().to_string().white().bold(),
        format!("dist/{}/", config.pack.namespace.as_str())
            .white()
            .bold()
    );

    // 8. Zip if --release, otherwise hint how to install manually.
    if release {
        let zip_path = zip_dir(&dist, config.pack.namespace.as_str())?;
        println!(
            "  {} {}",
            "zip:".dimmed(),
            zip_path.display().to_string().white().bold()
        );
        println!(
            "  {} drop {} into your world's datapacks/ folder",
            "install:".dimmed(),
            format!("dist/{}.zip", config.pack.namespace.as_str())
                .white()
                .bold()
        );
    } else {
        println!(
            "  {} copy the {} folder into your world's datapacks/ folder, \
             or run `sand build --release` to produce a zip",
            "install:".dimmed(),
            format!("dist/{}/", config.pack.namespace.as_str())
                .white()
                .bold()
        );
    }

    // 9. Resource pack build (optional, --resourcepack flag)
    if resourcepack {
        build_resourcepack(&config, &mc_version, release, &target_dir)?;
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::package::zip_dir;
    use super::records::{
        ComponentContentType, ComponentRecord, ContentType, OutputExt, ResourcePackRecord,
    };
    use super::validate::{
        component_output_path, validate_component_records, validate_component_records_for_project,
        validate_function_tag, validate_resourcepack_records,
        validate_resourcepack_records_for_project,
    };
    use super::write::{write_component, write_pack_mcmeta, write_resourcepack_mcmeta};
    use sand_components::registry_coverage::REGISTRY_COVERAGE;

    /// Construct a valid ComponentRecord from parts via JSON deserialization.
    ///
    /// Uses "audit" as the namespace. Panics if the inputs are invalid (which
    /// makes test failures obvious at the point of construction).
    fn record(dir: &str, path: &str, ext: &str, content: &str) -> ComponentRecord {
        serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": dir,
            "path": path,
            "ext": ext,
            "content": content,
        }))
        .unwrap_or_else(|e| panic!("invalid test record ({dir}/{path}.{ext}): {e}"))
    }

    fn resourcepack_record(path: &str, content_type: &str, content: &str) -> ResourcePackRecord {
        serde_json::from_value(serde_json::json!({
            "path": path,
            "content_type": content_type,
            "content": content,
        }))
        .unwrap_or_else(|e| panic!("invalid resource-pack test record ({path}): {e}"))
    }

    // ── sand.toml namespace validation at config parse time ───────────────────

    fn parse_config(namespace: &str) -> Result<crate::config::SandConfig, toml::de::Error> {
        let toml = format!(
            "[pack]\nnamespace = {namespace:?}\ndescription = \"test\"\nmc_version = \"1.21\"\n"
        );
        toml::from_str(&toml)
    }

    #[test]
    fn valid_config_namespace_parses() {
        for ns in ["my_pack", "test-pack", "ns.v2", "a", "abc123"] {
            assert!(
                parse_config(ns).is_ok(),
                "namespace '{ns}' should be valid in sand.toml"
            );
        }
    }

    #[test]
    fn invalid_config_namespace_rejected_at_parse() {
        for ns in [
            "",
            "MyPack",
            "has space",
            "upper/slash",
            "UPPER",
            "../escape",
        ] {
            assert!(
                parse_config(ns).is_err(),
                "namespace '{ns}' should be rejected when parsing sand.toml"
            );
        }
    }

    // ── Record validation ─────────────────────────────────────────────────────

    #[test]
    fn validates_component_records_before_writing() {
        let dist = Path::new("dist/audit");
        assert!(
            validate_component_records(
                dist,
                &[record(
                    "recipe",
                    "valid",
                    "json",
                    "{\"type\":\"minecraft:crafting_shaped\"}"
                )]
            )
            .is_ok()
        );
        assert!(
            validate_component_records(dist, &[record("recipe", "invalid", "json", "{")]).is_err()
        );
        assert!(
            validate_component_records(
                dist,
                &[record("function", "null", "mcfunction", "say hi\0")]
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_duplicate_component_outputs() {
        let dist = Path::new("dist/audit");
        assert!(
            validate_component_records(
                dist,
                &[
                    record("recipe", "same", "json", "{}"),
                    record("recipe", "same", "json", "{}"),
                ]
            )
            .is_err()
        );
    }

    // ── Newtype boundary validation ───────────────────────────────────────────

    #[test]
    fn path_traversal_rejected_at_deserialization() {
        let bad: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "recipe",
            "path": "../escape",
            "ext": "json",
            "content": "{}",
        }));
        assert!(
            bad.is_err(),
            "path traversal must be rejected at deserialization"
        );

        let abs: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "recipe",
            "path": "/etc/passwd",
            "ext": "json",
            "content": "{}",
        }));
        assert!(
            abs.is_err(),
            "absolute path must be rejected at deserialization"
        );
    }

    #[test]
    fn invalid_namespace_rejected_at_deserialization() {
        for bad_ns in ["", "My_Pack", "has space", "upper/slash", "UPPER"] {
            let result: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
                "namespace": bad_ns,
                "dir": "function",
                "path": "load",
                "ext": "mcfunction",
                "content": "",
            }));
            assert!(
                result.is_err(),
                "namespace '{bad_ns}' must be rejected at deserialization"
            );
        }
    }

    #[test]
    fn unsupported_component_dir_rejected_at_deserialization() {
        for bad_dir in ["assets", "data", "META-INF", "../data", "unknown_dir"] {
            let result: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
                "namespace": "audit",
                "dir": bad_dir,
                "path": "test",
                "ext": "json",
                "content": "{}",
            }));
            assert!(
                result.is_err(),
                "dir '{bad_dir}' must be rejected at deserialization"
            );
        }
    }

    #[test]
    fn registry_coverage_component_dirs_are_supported() {
        for entry in REGISTRY_COVERAGE {
            let datapack_record = record(entry.datapack_dir, "sample", "json", "{}");
            assert_eq!(datapack_record.dir.as_str(), entry.datapack_dir);

            if let Some(tag_dir) = entry.tag_dir {
                let tag_record = record(tag_dir, "sample", "json", "{}");
                assert_eq!(tag_record.dir.as_str(), tag_dir);
            }
        }
    }

    #[test]
    fn missing_registry_raw_json_component_passes_build_validation() {
        let dist = Path::new("dist/audit");
        let record = record(
            "enchantment_provider",
            "bonus_enchants",
            "json",
            r#"{"type":"minecraft:single_enchantment","enchantment":"minecraft:sharpness"}"#,
        );

        assert!(validate_component_records(dist, &[record]).is_ok());
    }

    // ── Datapack / resource-pack separation ───────────────────────────────────

    #[test]
    fn separates_datapack_and_resourcepack_roots() {
        // 'assets' is not a valid ComponentDirectory — caught at deserialization
        let bad_dir: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "assets",
            "path": "escaped",
            "ext": "json",
            "content": "{}",
        }));
        assert!(
            bad_dir.is_err(),
            "'assets' dir must be rejected at deserialization"
        );

        let rp_ok: ResourcePackRecord = serde_json::from_value(serde_json::json!({
            "path": "assets/audit/models/item/test.json",
            "content_type": "json",
            "content": "{}",
        }))
        .unwrap();
        assert!(validate_resourcepack_records(&[rp_ok]).is_ok());

        let rp_bad: ResourcePackRecord = serde_json::from_value(serde_json::json!({
            "path": "data/audit/recipe/test.json",
            "content_type": "json",
            "content": "{}",
        }))
        .unwrap();
        assert!(
            validate_resourcepack_records(&[rp_bad]).is_err(),
            "data/ paths must be rejected for resource pack records"
        );
    }

    #[test]
    fn validates_resourcepack_copy_source_paths_before_writing() {
        for bad_source in ["", "../escape.png", "/tmp/escape.png", "assets\0bad.png"] {
            let record =
                resourcepack_record("assets/audit/textures/item/test.png", "copy", bad_source);
            let err = validate_resourcepack_records(&[record]).unwrap_err();
            assert!(
                err.to_string()
                    .contains("unsafe resource-pack copy source path"),
                "error should identify unsafe source path: {err}"
            );
        }
    }

    #[test]
    fn validates_resourcepack_copy_source_files_before_writing() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path();

        let missing = resourcepack_record(
            "assets/audit/textures/item/missing.png",
            "copy",
            "assets/src/missing.png",
        );
        let err = validate_resourcepack_records_for_project(project_root, &[missing]).unwrap_err();
        assert!(
            err.to_string().contains("resource-pack asset not found"),
            "missing source should be reported before writing: {err}"
        );

        std::fs::create_dir_all(project_root.join("assets/src/dir.png")).unwrap();
        let directory = resourcepack_record(
            "assets/audit/textures/item/dir.png",
            "copy",
            "assets/src/dir.png",
        );
        let err =
            validate_resourcepack_records_for_project(project_root, &[directory]).unwrap_err();
        assert!(
            err.to_string()
                .contains("resource-pack asset is not a file"),
            "directory source should be rejected before writing: {err}"
        );

        std::fs::create_dir_all(project_root.join("assets/src")).unwrap();
        std::fs::write(project_root.join("assets/src/ok.png"), b"png").unwrap();
        let valid = resourcepack_record(
            "assets/audit/textures/item/ok.png",
            "copy",
            "assets/src/ok.png",
        );
        assert!(validate_resourcepack_records_for_project(project_root, &[valid]).is_ok());
    }

    #[test]
    fn validates_resourcepack_bytes_before_writing() {
        let invalid = resourcepack_record(
            "assets/audit/textures/item/bad.bin",
            "bytes",
            "not valid base64",
        );
        let err = validate_resourcepack_records(&[invalid]).unwrap_err();
        assert!(
            err.to_string().contains("invalid base64 bytes"),
            "invalid bytes should fail during validation: {err}"
        );

        let valid = resourcepack_record("assets/audit/textures/item/ok.bin", "bytes", "cG5n");
        assert!(validate_resourcepack_records(&[valid]).is_ok());
    }

    // ── Pack metadata and zip ─────────────────────────────────────────────────

    #[test]
    fn pack_metadata_and_release_zip_stay_with_their_pack_root() {
        let temp = tempfile::tempdir().unwrap();
        let datapack = temp.path().join("audit");
        let resourcepack = temp.path().join("audit-resources");
        std::fs::create_dir_all(datapack.join("data/audit/function")).unwrap();
        std::fs::create_dir_all(resourcepack.join("assets/audit/models/item")).unwrap();
        write_pack_mcmeta(&datapack, "audit", "data", 71).unwrap();
        write_resourcepack_mcmeta(&resourcepack, "resources", 48).unwrap();
        std::fs::write(
            datapack.join("data/audit/function/load.mcfunction"),
            "say loaded",
        )
        .unwrap();
        std::fs::write(
            resourcepack.join("assets/audit/models/item/test.json"),
            "{}",
        )
        .unwrap();

        let data_meta: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(datapack.join("pack.mcmeta")).unwrap())
                .unwrap();
        let resource_meta: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(resourcepack.join("pack.mcmeta")).unwrap(),
        )
        .unwrap();
        assert_eq!(data_meta["pack"]["pack_format"], 71);
        assert_eq!(resource_meta["pack"]["pack_format"], 48);

        let zip_path = zip_dir(&datapack, "audit").unwrap();
        let mut zip = zip::ZipArchive::new(std::fs::File::open(zip_path).unwrap()).unwrap();
        assert!(zip.by_name("pack.mcmeta").is_ok());
        assert!(zip.by_name("data/audit/function/load.mcfunction").is_ok());
        assert!(zip.by_name("assets/audit/models/item/test.json").is_err());
    }

    // ── Component output path computation ─────────────────────────────────────

    #[test]
    fn locks_modern_singular_datapack_component_paths() {
        let dist = Path::new("dist/audit");
        let cases = [
            (
                "function",
                "load",
                "mcfunction",
                "data/audit/function/load.mcfunction",
            ),
            (
                "tags/function",
                "load",
                "json",
                "data/audit/tags/function/load.json",
            ),
            (
                "advancement",
                "test",
                "json",
                "data/audit/advancement/test.json",
            ),
            ("recipe", "test", "json", "data/audit/recipe/test.json"),
            (
                "predicate",
                "test",
                "json",
                "data/audit/predicate/test.json",
            ),
            (
                "loot_table",
                "test",
                "json",
                "data/audit/loot_table/test.json",
            ),
            (
                "item_modifier",
                "test",
                "json",
                "data/audit/item_modifier/test.json",
            ),
            (
                "damage_type",
                "test",
                "json",
                "data/audit/damage_type/test.json",
            ),
            (
                "enchantment",
                "test",
                "json",
                "data/audit/enchantment/test.json",
            ),
            (
                "banner_pattern",
                "test",
                "json",
                "data/audit/banner_pattern/test.json",
            ),
            (
                "painting_variant",
                "test",
                "json",
                "data/audit/painting_variant/test.json",
            ),
            (
                "trim_material",
                "test",
                "json",
                "data/audit/trim_material/test.json",
            ),
            (
                "trim_pattern",
                "test",
                "json",
                "data/audit/trim_pattern/test.json",
            ),
            (
                "chat_type",
                "test",
                "json",
                "data/audit/chat_type/test.json",
            ),
            (
                "wolf_variant",
                "test",
                "json",
                "data/audit/wolf_variant/test.json",
            ),
            (
                "jukebox_song",
                "test",
                "json",
                "data/audit/jukebox_song/test.json",
            ),
            (
                "worldgen/biome",
                "test",
                "json",
                "data/audit/worldgen/biome/test.json",
            ),
            (
                "worldgen/noise_settings",
                "test",
                "json",
                "data/audit/worldgen/noise_settings/test.json",
            ),
            (
                "worldgen/placed_feature",
                "test",
                "json",
                "data/audit/worldgen/placed_feature/test.json",
            ),
        ];
        for (dir, path, ext, expected) in cases {
            let output = component_output_path(dist, &record(dir, path, ext, "{}")).unwrap();
            let actual = output
                .strip_prefix(dist)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            assert_eq!(actual, expected, "wrong directory for {dir}");
        }

        let minecraft_tag: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags/function",
            "path": "tick",
            "ext": "json",
            "content": "{}",
        }))
        .unwrap();
        assert_eq!(
            component_output_path(dist, &minecraft_tag)
                .unwrap()
                .strip_prefix(dist)
                .unwrap(),
            PathBuf::from("data/minecraft/tags/function/tick.json")
        );
    }

    // ── OutputExt / ContentType deserialization ───────────────────────────────

    #[test]
    fn typed_output_ext_deserializes_from_json() {
        let json = r#"{"namespace":"ns","dir":"function","path":"load","ext":"mcfunction","content":"say hi"}"#;
        let rec: ComponentRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.ext, OutputExt::Mcfunction);
        assert_eq!(rec.content_type, ComponentContentType::Text);

        let json2 =
            r#"{"namespace":"ns","dir":"recipe","path":"test","ext":"json","content":"{}"}"#;
        let rec2: ComponentRecord = serde_json::from_str(json2).unwrap();
        assert_eq!(rec2.ext, OutputExt::Json);

        let json3 = r#"{"namespace":"ns","dir":"structure","path":"rooms/start","ext":"nbt","content_type":"copy","content":"structures/start.nbt"}"#;
        let rec3: ComponentRecord = serde_json::from_str(json3).unwrap();
        assert_eq!(rec3.ext, OutputExt::Nbt);
        assert_eq!(rec3.content_type, ComponentContentType::Copy);
    }

    #[test]
    fn unknown_ext_rejected_at_deserialize() {
        let json = r#"{"namespace":"ns","dir":"function","path":"load","ext":"lua","content":""}"#;
        assert!(serde_json::from_str::<ComponentRecord>(json).is_err());
    }

    #[test]
    fn typed_content_type_deserializes_from_json() {
        let json = r#"{"path":"assets/ns/font/hud.json","content_type":"json","content":"{}"}"#;
        let rec: ResourcePackRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.content_type, ContentType::Json);

        let json2 =
            r#"{"path":"assets/ns/textures/a.png","content_type":"copy","content":"src/a.png"}"#;
        let rec2: ResourcePackRecord = serde_json::from_str(json2).unwrap();
        assert_eq!(rec2.content_type, ContentType::Copy);

        let json3 =
            r#"{"path":"assets/ns/textures/b.png","content_type":"bytes","content":"AAAA"}"#;
        let rec3: ResourcePackRecord = serde_json::from_str(json3).unwrap();
        assert_eq!(rec3.content_type, ContentType::Bytes);
    }

    #[test]
    fn unknown_content_type_rejected_at_deserialize() {
        let json = r#"{"path":"assets/ns/a.png","content_type":"binary","content":""}"#;
        assert!(serde_json::from_str::<ResourcePackRecord>(json).is_err());
    }

    #[test]
    fn validates_structure_template_copy_records() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let dist = temp.path().join("dist/audit");
        let src = project_root.join("src/structures/start.nbt");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, [0x0a, 0x00, 0x00]).unwrap();

        let good: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content_type": "copy",
            "content": "src/structures/start.nbt",
        }))
        .unwrap();
        assert!(validate_component_records_for_project(&dist, &project_root, &[good]).is_ok());

        let unsafe_source: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content_type": "copy",
            "content": "../start.nbt",
        }))
        .unwrap();
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[unsafe_source]).is_err()
        );

        let missing_source: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/missing",
            "ext": "nbt",
            "content_type": "copy",
            "content": "src/structures/missing.nbt",
        }))
        .unwrap();
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[missing_source])
                .is_err()
        );
        assert!(
            !dist.exists(),
            "copy-backed structure preflight must not create output"
        );

        let wrong_ext: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "json",
            "content": "{}",
        }))
        .unwrap();
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[wrong_ext]).is_err(),
            "structure outputs must use .nbt"
        );

        let text_nbt: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content": "not binary content",
        }))
        .unwrap();
        assert!(validate_component_records_for_project(&dist, &project_root, &[text_nbt]).is_err());
    }

    #[test]
    fn writes_and_zips_structure_template_assets() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let dist = temp.path().join("dist").join("audit");
        let src = project_root.join("src/structures/start.nbt");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, [0x0a, 0x00, 0x00]).unwrap();

        let record: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content_type": "copy",
            "content": "src/structures/start.nbt",
        }))
        .unwrap();

        validate_component_records_for_project(&dist, &project_root, std::slice::from_ref(&record))
            .unwrap();
        write_component(&dist, &project_root, &record).unwrap();

        let output = dist.join("data/audit/structure/rooms/start.nbt");
        assert_eq!(std::fs::read(&output).unwrap(), [0x0a, 0x00, 0x00]);

        let zip_path = zip_dir(&dist, "audit").unwrap();
        let zip_file = std::fs::File::open(zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        let mut file = archive
            .by_name("data/audit/structure/rooms/start.nbt")
            .unwrap();
        let mut bytes = Vec::new();
        use std::io::Read as _;
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, [0x0a, 0x00, 0x00]);
    }

    // ── Function tag validation ───────────────────────────────────────────────

    #[test]
    fn function_tag_accepts_valid_load_tick_tags() {
        // Typical load tag
        let load = r#"{"values":["my_pack:load"]}"#;
        assert!(validate_function_tag("load", load).is_ok());

        // Typical tick tag with multiple entries
        let tick = r#"{"values":["my_pack:tick","other_pack:tick"]}"#;
        assert!(validate_function_tag("tick", tick).is_ok());

        // Empty values array is valid (no functions registered)
        let empty = r#"{"values":[]}"#;
        assert!(validate_function_tag("load", empty).is_ok());

        // Tag reference (#-prefixed) with valid resource location
        let tag_ref = "{\"values\":[\"#minecraft:some_tag\"]}";
        assert!(validate_function_tag("load", tag_ref).is_ok());

        // Object form with valid resource location and required=false
        let optional = r#"{"values":[{"id":"my_pack:optional","required":false}]}"#;
        assert!(validate_function_tag("load", optional).is_ok());

        // Object form with id only (required is optional)
        let id_only = r#"{"values":[{"id":"my_pack:fn"}]}"#;
        assert!(validate_function_tag("load", id_only).is_ok());

        // Paths with subdirectories are valid
        let subdir = r#"{"values":["my_pack:subfolder/load"]}"#;
        assert!(validate_function_tag("load", subdir).is_ok());
    }

    #[test]
    fn function_tag_rejects_invalid_structures() {
        // Not an object
        assert!(validate_function_tag("load", r#"[]"#).is_err());

        // Missing values key
        assert!(validate_function_tag("load", r#"{}"#).is_err());

        // values is not an array
        assert!(validate_function_tag("load", r#"{"values":"my_pack:load"}"#).is_err());

        // String entry missing ':' entirely
        assert!(validate_function_tag("load", r#"{"values":["no_colon_here"]}"#).is_err());

        // Uppercase namespace is rejected
        assert!(validate_function_tag("load", r#"{"values":["Bad:load"]}"#).is_err());

        // Empty namespace
        assert!(validate_function_tag("load", r#"{"values":[":load"]}"#).is_err());

        // Empty path
        assert!(validate_function_tag("load", r#"{"values":["minecraft:"]}"#).is_err());

        // Object entry missing 'id'
        assert!(validate_function_tag("load", r#"{"values":[{"required":false}]}"#).is_err());

        // Object id is not a string
        assert!(validate_function_tag("load", r#"{"values":[{"id":42}]}"#).is_err());

        // Object id is not a valid resource location
        assert!(validate_function_tag("load", r#"{"values":[{"id":"not_a_location"}]}"#).is_err());

        // Object id with uppercase namespace
        assert!(validate_function_tag("load", r#"{"values":[{"id":"Bad:load"}]}"#).is_err());

        // required is not a boolean
        assert!(
            validate_function_tag(
                "load",
                r#"{"values":[{"id":"my_pack:fn","required":"yes"}]}"#
            )
            .is_err()
        );

        // Invalid JSON
        assert!(validate_function_tag("load", r#"{"values": ["#).is_err());
    }

    #[test]
    fn function_tag_validation_applies_to_generic_tags_dir() {
        // A record with dir="tags" and path="function/load" should also be
        // validated as a function tag by validate_component_records.
        let dist = std::path::Path::new("dist/audit");

        // Valid function tag via the generic dir="tags" form
        let good: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags",
            "path": "function/load",
            "ext": "json",
            "content": r#"{"values":["my_pack:load"]}"#,
        }))
        .unwrap();
        assert!(validate_component_records(dist, &[good]).is_ok());

        // Malformed function tag via the generic dir="tags" form should fail
        let bad: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags",
            "path": "function/load",
            "ext": "json",
            "content": r#"{"values":["BadNamespace:load"]}"#,
        }))
        .unwrap();
        assert!(
            validate_component_records(dist, &[bad]).is_err(),
            "invalid resource location in tags dir+function/ path must be caught"
        );
    }

    // ── Golden fixture ────────────────────────────────────────────────────────

    /// End-to-end fixture: given a minimal set of records (functions + tags),
    /// the build pipeline writes the expected files with the expected content.
    #[test]
    fn golden_fixture_minimal_pack() {
        let temp = tempfile::tempdir().unwrap();
        let dist = temp.path().join("golden");

        let tick_tag_json = r#"{"values":["golden:tick"]}"#;
        let load_tag_json = r#"{"values":["golden:load"]}"#;

        let records: Vec<ComponentRecord> = serde_json::from_value(serde_json::json!([
            {
                "namespace": "golden",
                "dir": "function",
                "path": "load",
                "ext": "mcfunction",
                "content": "say loaded",
            },
            {
                "namespace": "golden",
                "dir": "function",
                "path": "tick",
                "ext": "mcfunction",
                "content": "say tick",
            },
            {
                "namespace": "minecraft",
                "dir": "tags/function",
                "path": "load",
                "ext": "json",
                "content": load_tag_json,
            },
            {
                "namespace": "minecraft",
                "dir": "tags/function",
                "path": "tick",
                "ext": "json",
                "content": tick_tag_json,
            },
        ]))
        .unwrap();

        // Validate before writing
        validate_component_records(&dist, &records).unwrap();

        // Validate load/tick tag structure explicitly
        validate_function_tag("load", load_tag_json).unwrap();
        validate_function_tag("tick", tick_tag_json).unwrap();

        // Write the pack
        std::fs::create_dir_all(&dist).unwrap();
        write_pack_mcmeta(&dist, "golden", "Golden fixture pack", 71).unwrap();
        for r in &records {
            write_component(&dist, temp.path(), r).unwrap();
        }

        // Verify pack.mcmeta
        let mcmeta: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dist.join("pack.mcmeta")).unwrap())
                .unwrap();
        assert_eq!(mcmeta["pack"]["pack_format"], 71);
        assert_eq!(mcmeta["pack"]["description"], "Golden fixture pack");

        // Verify functions
        assert_eq!(
            std::fs::read_to_string(dist.join("data/golden/function/load.mcfunction")).unwrap(),
            "say loaded"
        );
        assert_eq!(
            std::fs::read_to_string(dist.join("data/golden/function/tick.mcfunction")).unwrap(),
            "say tick"
        );

        // Verify function tags
        let load_tag: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dist.join("data/minecraft/tags/function/load.json")).unwrap(),
        )
        .unwrap();
        assert!(
            load_tag["values"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "golden:load"),
            "load tag must reference golden:load"
        );

        let tick_tag: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dist.join("data/minecraft/tags/function/tick.json")).unwrap(),
        )
        .unwrap();
        assert!(
            tick_tag["values"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "golden:tick"),
            "tick tag must reference golden:tick"
        );
    }
}
