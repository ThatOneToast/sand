fn main() {
    // Use the version pinned in sand.toml. In a real project this would be
    // read from the file; here we reference it directly so the example stays
    // self-contained.
    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if let Err(err) = sand_build::generate("1.21.4") {
        if strict {
            panic!("arcane-pack codegen failed: {err}");
        }
        println!(
            "cargo:warning=arcane-pack codegen skipped: {err}. \
             Continuing because SAND_STRICT_CODEGEN is not enabled."
        );
    }
}
