# Storage And NBT Example

Use storage for structured or persistent settings that do not fit scoreboard
state cleanly.

```rust
static SETTINGS: StorageVar<i32> = StorageVar::new("example:data", "settings.mana");

#[component(Load)]
pub fn load_settings() {
    SETTINGS.set_int(100);
    SETTINGS.as_path().key("enabled").set_bool(true);
}
```
