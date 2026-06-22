# Dialogs

Dialogs are typed datapack components and require a Minecraft version that
supports dialog JSON.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

#[function]
pub fn start() {
    cmd::tellraw(Selector::self_(), Text::new("Starting...").green());
}

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice_local("welcome")
        .title(Text::new("Welcome").gold())
        .body(DialogBody::text(Text::new("Choose your next action.")))
        .button(
            DialogButton::new(Text::new("Start").green())
                .action(DialogAction::run_function(start))
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

Gate version-sensitive output with `VersionProfile::supports_dialogs()` in
tools that emit optional dialog content.
