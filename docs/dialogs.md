# Dialogs

Dialogs are typed datapack components for Minecraft versions that support the
dialog format. Check capabilities when targeting older versions.

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

Version support:

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.6").unwrap()).unwrap();
assert!(profile.supports_dialogs());
```
