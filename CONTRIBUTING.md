# Contributing

Use small, green commits. For code or public docs changes, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p sand-macros
cargo doc --workspace --all-features --no-deps
```

Or use the combined check script:

```sh
scripts/check.sh
```

## Authoring guidance

- Teach `#[function]`, `#[component(Load)]`, and `#[component(Tick)]` first.
- Use `sand_core::prelude::*` in beginner examples.
- Keep raw commands behind `cmd::raw(...)`.
- Use `mcfunction!` only for advanced command grouping, migration, or interop.
- Add exact output tests when changing command builders or macro expansion.
- Generated command builders live in `sand-core/src/cmd/_generated` (from
  `commands.json`). Hand-written builtins live in `sand-commands/src/builtins.rs`.

## Testing

- `cargo test --workspace --all-features` — workspace tests
- `cargo test -p sand-macros` — trybuild compile tests
- `cargo doc --workspace --all-features --no-deps` — rustdoc validation
- `mdbook build` — book validation (if mdbook is installed)

Do not require network-heavy Minecraft data regeneration in default CI unless
the change is specifically about generated data.
