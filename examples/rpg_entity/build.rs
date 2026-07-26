fn main() {
    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|value| matches!(value.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if let Err(error) = sand_build::generate("26.2") {
        if strict {
            panic!("rpg_entity code generation failed: {error}");
        }
        println!("cargo:warning=rpg_entity code generation skipped: {error}");
    }
}
