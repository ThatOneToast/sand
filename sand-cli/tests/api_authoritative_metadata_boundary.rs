//! Validates the authoritative-metadata boundary claimed for `sand api`
//! (Phase 6 of the API-discovery work): it answers queries entirely from
//! metadata compiled into the binary, with no runtime dependency on a
//! checked-out Sand source tree and no network access.
//!
//! `installed_generated_export_is_byte_deterministic` (sand-cli/src/api_cmd.rs
//! unit tests) already covers export determinism from within the repo. This
//! file covers the property that unit test can't: running the binary from a
//! location where no Sand source, or network, is reachable at all.

use std::process::Command;

/// Running `sand api export` from a directory with no Sand workspace nearby,
/// and with common proxy env vars pointed at an address nothing listens on,
/// must still succeed and produce the same catalog as running from the repo.
/// If the CLI parsed Rust source or made a network call at query time, one
/// of these would fail or hang instead of returning instantly.
#[test]
fn api_export_needs_no_source_tree_or_network_access() {
    let outside_dir = std::env::temp_dir();
    assert!(
        !outside_dir.join("Cargo.toml").exists(),
        "test assumption violated: {} unexpectedly contains a Cargo.toml",
        outside_dir.display()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["api", "export"])
        .current_dir(&outside_dir)
        // Point every common proxy variable at a port nothing listens on.
        // A real network call would fail fast (connection refused) rather
        // than silently succeeding, making this a meaningful negative check.
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("http_proxy", "http://127.0.0.1:1")
        .env("https_proxy", "http://127.0.0.1:1")
        .output()
        .expect("run `sand api export` outside any Sand source tree");
    assert!(
        output.status.success(),
        "`sand api export` failed outside a Sand source tree:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let from_outside = String::from_utf8(output.stdout).expect("export is UTF-8");

    let from_repo = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["api", "export"])
        .output()
        .expect("run `sand api export` from the repo");
    assert!(from_repo.status.success());
    let from_repo = String::from_utf8(from_repo.stdout).expect("export is UTF-8");

    assert_eq!(
        from_outside, from_repo,
        "the installed API catalog differs depending on the working directory; \
         it should be entirely determined by what's compiled into the binary"
    );
}

/// `sand api search`/`show` must likewise work identically regardless of
/// working directory, confirming the whole CLI surface (not just export)
/// stays inside the authoritative-metadata boundary.
#[test]
fn api_search_and_show_need_no_source_tree() {
    let outside_dir = std::env::temp_dir();

    let search = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["api", "search", "nearby entities", "--limit", "3"])
        .current_dir(&outside_dir)
        .output()
        .expect("run `sand api search` outside any Sand source tree");
    assert!(search.status.success());
    assert!(
        String::from_utf8_lossy(&search.stdout).contains("sand::entity::EntityQuery::nearby"),
        "search from outside the repo should find the same results as from inside it"
    );

    let show = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["api", "show", "sand::entity::EntityQuery::nearby"])
        .current_dir(&outside_dir)
        .output()
        .expect("run `sand api show` outside any Sand source tree");
    assert!(show.status.success());
}
