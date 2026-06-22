# Dialogs

Dialogs are typed datapack components and require a Minecraft version that
supports dialog JSON.

```rust
#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice("example:welcome")
        .title("Welcome")
        .body(DialogBody::text("Choose your next action."))
        .button(DialogButton::new("Start").action(DialogAction::run_command(
            cmd::function(ResourceLocation::new("example", "start").unwrap()),
        )))
}
```

Gate version-sensitive output with `VersionProfile::supports_dialogs()` in
tools that emit optional dialog content.
