# Dialog Example

Dialogs are components. Commands triggered by buttons should use typed command
builders when possible.

```rust
#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice("example:welcome")
        .title("Welcome")
        .body(DialogBody::text("Ready to begin?"))
        .button(DialogButton::new("Start").action(DialogAction::run_command(
            cmd::function(ResourceLocation::new("example", "start").unwrap()),
        )))
}
```
