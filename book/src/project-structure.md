# Project Structure

A typical Sand project has:

```text
my_pack/
  Cargo.toml
  sand.toml
  src/lib.rs
```

`src/lib.rs` contains attribute functions and typed components. `sand.toml`
sets the namespace, target Minecraft version, and optional resource-pack output.
`sand build` collects inventory-registered functions and components into the
final datapack layout under `dist/`.

The repository keeps reference docs in `docs/` and this user-facing guide in
`book/`.
