# Testing

Run the normal checks before committing framework changes:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
```

Use focused golden tests for exact command output, component JSON, function
paths, and load/tick tag records.
