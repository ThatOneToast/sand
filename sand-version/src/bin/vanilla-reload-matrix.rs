fn main() {
    println!(
        r#"{{"include":[{{"target":"latest","version":"{}","java":"{}"}}]}}"#,
        sand_version::LATEST_KNOWN,
        sand_version::CI_LATEST_JAVA_VERSION,
    );
}
