# Dialogs

Dialogs are typed component JSON and can use typed function callbacks.

```rust
Dialog::notice("arcane:welcome").title("Welcome")
 .button(DialogButton::new("Start").action(DialogAction::run_function(start)));
```

Dialogs are Minecraft-version-sensitive; use `VersionProfile` when supporting multiple versions. A dialog component is not an event system.
