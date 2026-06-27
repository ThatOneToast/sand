# Testing Datapacks

Test command generation in Rust and test output in Minecraft. Unit tests can assert exact generated strings; integration tests should build a pack, install it in a disposable world, run `/reload`, and invoke a small function.

Run `cargo test --workspace --all-features` and `scripts/build-book.sh` before publishing. The docs script builds the mdBook and validates local Markdown links. Inspect generated JSON/functions after Minecraft upgrades.
