use sand_version::DEFAULT_CODEGEN_VERSION;

fn main() {
    let manifest = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
    );
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let version =
        std::env::var("SAND_MC_VERSION").unwrap_or_else(|_| DEFAULT_CODEGEN_VERSION.to_owned());
    let source = manifest.join("src/registry.rs");
    let provider = out_dir.join("registry_ids.api.json");

    sand_build::registry_id_contract_provider(&source, &version)
        .and_then(|catalog| catalog.write_json(&provider))
        .unwrap_or_else(|error| panic!("failed to generate registry-ID API contracts: {error}"));

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-env-changed=SAND_MC_VERSION");
    println!("cargo::metadata=api_provider_file={}", provider.display());
}
