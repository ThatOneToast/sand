//! Explicitly refresh the committed offline vanilla registry catalog.
fn main() {
    let mut args = std::env::args().skip(1);
    let output = args
        .next()
        .expect("usage: refresh_vanilla_catalog OUTPUT VERSION...");
    let versions: Vec<String> = args.collect();
    assert!(!versions.is_empty(), "supply explicit Minecraft versions");
    sand_build::refresh_vanilla_catalog(&versions, std::path::Path::new(&output)).unwrap();
}
