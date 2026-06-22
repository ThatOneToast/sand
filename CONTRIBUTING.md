# Contributing

Use small, green commits. For code or public docs changes, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
```

Authoring guidance:

- Teach `#[function]`, `#[component(Load)]`, and `#[component(Tick)]` first.
- Use `sand_core::prelude::*` in beginner examples.
- Keep raw commands behind `cmd::raw(...)`.
- Use `mcfunction!` only for advanced command grouping, migration, or interop.
- Add exact output tests when changing command builders or macro expansion.

Do not require network-heavy Minecraft data regeneration in default CI unless
the change is specifically about generated data.
