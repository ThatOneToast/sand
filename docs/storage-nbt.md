# Storage And NBT

Use scoreboards for compact integer state and `StorageVar<T>` for structured
NBT payloads.

```rust
use sand_core::prelude::*;

static DATA: StorageVar<i32> = StorageVar::new("example:data", "players.self.mana");

mcfunction! {
    DATA.set_int(100);
    DATA.as_path().key("flags").key("has_dash").set_bool(true);
}
```

Storage paths can also become typed conditions:

```rust
let commands = TypedExecute::as_players()
    .when(DATA.exists())
    .run(Actionbar::show(Selector::self_(), Text::new("Data loaded").green()));
```
