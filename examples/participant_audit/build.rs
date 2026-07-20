fn main() {
    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if let Err(err) = sand_build::generate("26.2") {
        if strict {
            panic!("participant_audit codegen failed: {err}");
        }
        println!(
            "cargo:warning=participant_audit codegen skipped: {err}. \
             Continuing because SAND_STRICT_CODEGEN is not enabled."
        );
    }
}
