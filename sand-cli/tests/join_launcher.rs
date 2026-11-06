//! Exercise the public CLI against a stand-in launcher, without opening a game.
#![cfg(unix)]

use std::{os::unix::fs::PermissionsExt, process::Command};

#[test]
fn join_launches_and_propagates_launcher_failures() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("sand.toml"),
        "[pack]\nnamespace='join_test'\ndescription='Test'\nmc_version='26.3'\n",
    )
    .unwrap();
    let launcher = root.path().join("fake prism");
    std::fs::write(
        &launcher,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > invocation\nexit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o700)).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_sand"))
            .current_dir(root.path())
            .args(["join", "--instance", "instance with spaces", "--launcher"])
            .arg(&launcher)
            .args(extra)
            .output()
            .unwrap()
    };
    assert!(run(&["--world", "literal $(world)"]).status.success());
    assert_eq!(
        std::fs::read_to_string(root.path().join("invocation")).unwrap(),
        "--launch\ninstance with spaces\n--world\nliteral $(world)\n"
    );
    assert!(!run(&["--local"]).status.success());
    std::fs::create_dir_all(root.path().join("dist/server")).unwrap();
    std::fs::write(
        root.path().join("dist/server/server.properties"),
        "server-port=25568\n",
    )
    .unwrap();
    assert!(run(&["--local"]).status.success());
    assert_eq!(
        std::fs::read_to_string(root.path().join("invocation")).unwrap(),
        "--launch\ninstance with spaces\n--server\n127.0.0.1:25568\n"
    );
    std::fs::write(&launcher, "#!/bin/sh\nexit 7\n").unwrap();
    let failed = run(&[]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("Prism launcher exited"));
    std::fs::remove_file(&launcher).unwrap();
    let missing = run(&[]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("failed to launch Prism"));
}
