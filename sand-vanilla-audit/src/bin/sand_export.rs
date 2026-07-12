fn main() {
    let version = std::env::var("SAND_EXPORT_MC_VERSION").unwrap_or_else(|_| {
        eprintln!("SAND_EXPORT_MC_VERSION is required");
        std::process::exit(1);
    });
    sand_vanilla_audit::export("sand_audit", &version);
}
