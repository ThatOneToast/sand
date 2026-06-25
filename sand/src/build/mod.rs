mod config;
mod package;
mod records;
mod resourcepack;
mod validate;
mod write;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use colored::Colorize;

use crate::config::SandConfig;
use crate::pack_format::pack_format_for;

use config::{cargo_target_dir, resolve_mc_version};
use package::zip_dir;
use records::ComponentRecord;
use resourcepack::build_resourcepack;
use validate::{validate_component_records, validate_namespace};
use write::{write_component, write_pack_mcmeta};

pub fn run(release: bool, resourcepack: bool) -> Result<()> {
    // 1. Read sand.toml
    let config_path = std::env::current_dir()?.join("sand.toml");
    if !config_path.exists() {
        bail!("sand.toml not found — run `sand build` from your project root");
    }
    let config: SandConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)
        .context("failed to parse sand.toml")?;

    validate_namespace(&config.pack.namespace)?;

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
        config.pack.namespace.white().bold(),
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
    let dist = PathBuf::from("dist").join(&config.pack.namespace);
    validate_component_records(&dist, &records)?;

    // 6. Write pack.mcmeta
    std::fs::create_dir_all(&dist)?;
    write_pack_mcmeta(
        &dist,
        &config.pack.namespace,
        &config.pack.description,
        pack_format,
    )?;

    // 7. Write each component file
    for record in &records {
        write_component(&dist, record)?;
    }

    println!(
        "{} {} component(s) written to {}",
        "Done!".green().bold(),
        records.len().to_string().white().bold(),
        format!("dist/{}/", config.pack.namespace).white().bold()
    );

    // 8. Zip if --release, otherwise hint how to install manually.
    if release {
        let zip_path = zip_dir(&dist, &config.pack.namespace)?;
        println!(
            "  {} {}",
            "zip:".dimmed(),
            zip_path.display().to_string().white().bold()
        );
        println!(
            "  {} drop {} into your world's datapacks/ folder",
            "install:".dimmed(),
            format!("dist/{}.zip", config.pack.namespace).white().bold()
        );
    } else {
        println!(
            "  {} copy the {} folder into your world's datapacks/ folder, \
             or run `sand build --release` to produce a zip",
            "install:".dimmed(),
            format!("dist/{}/", config.pack.namespace).white().bold()
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
    use super::records::{ComponentRecord, ContentType, OutputExt, ResourcePackRecord};
    use super::validate::{
        component_output_path, validate_component_records, validate_function_tag,
        validate_resourcepack_records,
    };
    use super::write::{write_component, write_pack_mcmeta, write_resourcepack_mcmeta};

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

        let json2 =
            r#"{"namespace":"ns","dir":"recipe","path":"test","ext":"json","content":"{}"}"#;
        let rec2: ComponentRecord = serde_json::from_str(json2).unwrap();
        assert_eq!(rec2.ext, OutputExt::Json);
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

        // Tag reference (#-prefixed)
        let tag_ref = "{\"values\":[\"#minecraft:some_tag\"]}";
        assert!(validate_function_tag("load", tag_ref).is_ok());

        // Object form with required=false
        let optional = r#"{"values":[{"id":"my_pack:optional","required":false}]}"#;
        assert!(validate_function_tag("load", optional).is_ok());
    }

    #[test]
    fn function_tag_rejects_invalid_structures() {
        // Not an object
        assert!(validate_function_tag("load", r#"[]"#).is_err());

        // Missing values key
        assert!(validate_function_tag("load", r#"{}"#).is_err());

        // values is not an array
        assert!(validate_function_tag("load", r#"{"values":"my_pack:load"}"#).is_err());

        // Entry missing ':' in resource location
        assert!(validate_function_tag("load", r#"{"values":["no_colon_here"]}"#).is_err());

        // Object entry missing 'id'
        assert!(validate_function_tag("load", r#"{"values":[{"required":false}]}"#).is_err());

        // Invalid JSON
        assert!(validate_function_tag("load", r#"{"values": ["#).is_err());
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
            write_component(&dist, r).unwrap();
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
