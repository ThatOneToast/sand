# Release Process

Release validation mirrors repository validation:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
mdbook build
```

If `mdbook` is unavailable locally, install it with:

```sh
cargo install mdbook
```

Release notes should identify stable typed APIs, experimental APIs, supported
Minecraft versions, and any intentional raw escape hatches.
