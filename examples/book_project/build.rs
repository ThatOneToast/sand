fn main() {
    // Use the version pinned in sand.toml. Codegen downloads the matching
    // server jar the first time; in restricted environments the non-strict
    // fallback keeps `cargo build` working from the cached data.
    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if let Err(err) = sand_build::generate("26.2") {
        if strict {
            panic!("book_project codegen failed: {err}");
        }
        println!(
            "cargo:warning=book_project codegen skipped: {err}. \
             Continuing because SAND_STRICT_CODEGEN is not enabled."
        );
    }
}
