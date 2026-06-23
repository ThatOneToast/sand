# Derive SandStorage

`#[derive(SandStorage)]` generates `SCHEMA` and one typed accessor per named field.

```rust
#[derive(SandStorage)]
#[sand(storage = "arcane:data", root = "config")]
struct Config { #[sand(path = "max")] max_mana: i32 }
```

It requires a named struct and `storage`/`root` attributes. Rust types document intended values but NBT remains runtime data. See [Storage And NBT](storage.md).
