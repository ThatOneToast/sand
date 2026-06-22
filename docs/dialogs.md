# Dialogs

Dialogs are typed datapack components for Minecraft versions that support the
dialog format. Check capabilities when targeting older versions.

```rust
use sand_core::prelude::*;
use sand_macros::component;

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice("example:welcome")
        .title("Welcome")
        .body(DialogBody::text("Choose your next action."))
        .button(
            DialogButton::new("Start")
                .action(DialogAction::run_command(cmd::function(
                    ResourceLocation::new("example", "start").unwrap(),
                )))
        )
}
```

Version support:

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.5").unwrap()).unwrap();
assert!(profile.supports_dialogs());
```
