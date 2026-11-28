//! Exercise both formatter entry points and package-wide failure behavior.
use std::fs;
use std::process::Command;

#[test]
fn formatter_checks_all_modules_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("pack.sand");
    let module = dir.path().join("logic.sand");
    fs::write(&root, "#![pack(namespace=\"demo\")]\nmod logic;\n").unwrap();
    fs::write(&module, "pub fn run(){let n=2;}\n").unwrap();
    let run = |prefix: &[&str], check: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sand"));
        cmd.args(prefix).arg(dir.path());
        if check {
            cmd.arg("--check");
        }
        cmd.output().unwrap()
    };
    let original = fs::read(&root).unwrap();
    assert!(!run(&["fmt"], true).status.success());
    assert_eq!(fs::read(&root).unwrap(), original);
    assert!(run(&["lang", "fmt"], false).status.success());
    assert!(
        fs::read_to_string(&module)
            .unwrap()
            .contains("    let n = 2;")
    );
    assert!(run(&["fmt"], true).status.success());
    let original = fs::read(&root).unwrap();
    fs::write(&module, "fn broken( {").unwrap();
    assert!(!run(&["fmt"], false).status.success());
    assert_eq!(fs::read(&root).unwrap(), original);
    assert_eq!(fs::read_to_string(module).unwrap(), "fn broken( {");
}
