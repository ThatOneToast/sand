# Testing Datapacks

Test command generation in Rust and test output in Minecraft. Unit tests can assert exact generated strings; integration tests should build a pack, install it in a disposable world, run `/reload`, and invoke a small function.

Run `cargo test --workspace --all-features` and `mdbook build` before publishing. Inspect generated JSON/functions after Minecraft upgrades.
