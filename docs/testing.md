# Testing

Run the standard validation set before committing:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
```

Public APIs should include focused tests for rendered command strings, component
JSON, datapack paths, tag paths, version gates, and nested condition lowering.

For the user guide:

```sh
scripts/build-book.sh
```

The script fails with a clear install hint when `mdbook` is not available.
