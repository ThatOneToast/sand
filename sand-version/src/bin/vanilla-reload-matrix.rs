fn main() {
    println!(
        r#"{{"include":[{{"target":"stable","version":"{}","java":"{}"}},{{"target":"latest","version":"{}","java":"{}"}}]}}"#,
        sand_version::CI_STABLE_CODEGEN_VERSION,
        sand_version::CI_STABLE_JAVA_VERSION,
        sand_version::LATEST_KNOWN,
        sand_version::CI_LATEST_JAVA_VERSION,
    );
}
