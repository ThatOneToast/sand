# Storage And NBT

Use `StorageVar<T>` and `NbtPath` for structured datapack state.

```rust
static SETTINGS: StorageVar<i32> = StorageVar::new("example:data", "settings.mana");

#[component(Load)]
pub fn load_storage() {
    SETTINGS.set_int(100);
    SETTINGS.as_path().key("regen").set_bool(true);
}
```

Storage paths can be used as execute conditions:

```rust
TypedExecute::as_players()
    .when(SETTINGS.exists())
    .run(Actionbar::show(Selector::self_(), Text::new("Storage ready").green()));
```
