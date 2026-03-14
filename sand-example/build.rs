fn main() {
    // Use the version pinned in sand.toml. In a real project this would be
    // read from the file; here we reference it directly so the example stays
    // self-contained.
    sand_build::generate("1.21.11").unwrap();
}
