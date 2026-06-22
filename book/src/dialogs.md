# Dialogs

Dialogs are typed datapack components and require a Minecraft version that
supports dialog JSON.

## Button actions

### `run_function` — operator-only

`DialogAction::run_function(fn_ptr)` emits `run_command: /function …` in the dialog JSON.
Minecraft executes `/function` as the clicking player, requiring OP permissions
(permission level ≥ 2). **Not safe for survival mode.**

### `callback` — survival-safe (recommended)

`DialogAction::callback(fn_ptr)` emits `/trigger sand.dialog set <id>` instead.
The `/trigger` command is usable by any player regardless of permissions. Sand
auto-generates `__sand_dialog_init` and `__sand_dialog_tick` infrastructure that
dispatches each callback on the server side.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

#[function]
pub fn start() {
    cmd::tellraw(Selector::self_(), Text::new("Starting...").green());
}

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::multi_action_local("welcome")
        .title(Text::new("Welcome").gold())
        .body(DialogBody::text(Text::new("Choose your next action.")))
        .button(
            DialogButton::new(Text::new("Start").green())
                .action(DialogAction::callback(start))  // survival-safe
        )
        .button(
            DialogButton::new(Text::new("Rules").yellow())
                .action(DialogAction::open_dialog(DialogRef::local("rules")))
        )
}

#[function]
pub fn open_welcome_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
}
```

## Generated infrastructure

When any `DialogAction::callback(...)` is used, Sand generates two extra functions:

- `__sand_dialog_init` — registered in `minecraft:load`, adds the `sand.dialog` trigger
  objective and enables it for `@a`
- `__sand_dialog_tick` — registered in `minecraft:tick`, re-enables the trigger each tick,
  dispatches each registered callback by ID, then resets the score to `0`

## Version support

Gate dialog registration behind `VersionProfile::supports_dialogs()` when targeting a
version range that predates dialog support.
