fn main() {
    println!(
        r#"{{"include":[{{"target":"latest","version":"{}","java":"{}"}},{{"target":"26.2","version":"26.2","java":"25"}}]}}"#,
        sand_version::LATEST_KNOWN,
        sand_version::CI_LATEST_JAVA_VERSION,
    );
}
