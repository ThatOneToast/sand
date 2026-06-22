# Dialog Example

Dialogs are components. Commands triggered by buttons should use typed command
builders when possible.

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
        .body(DialogBody::text(Text::new("Ready to begin?")))
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
