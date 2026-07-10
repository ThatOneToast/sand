# Dialogs

Dialogs are typed datapack components for Minecraft versions that support the
dialog format. Check capabilities when targeting older versions.

## Button actions

### `run_function` — requires operator permissions

`DialogAction::run_function(fn_ptr)` emits `run_command: /function namespace:path` in the
dialog JSON. Minecraft executes `/function` directly from the client, which requires the
player to have OP permissions (permission level ≥ 2). This is **not safe for survival servers**.

```rust
#[component]
pub fn admin_dialog() -> Dialog {
    Dialog::multi_action_local("admin_panel")
        .title(Text::new("Admin Panel").red())
        .body(DialogBody::text(Text::new("Admin only.")))
        .button(
            DialogButton::new(Text::new("Reset world").red())
                .action(DialogAction::run_function(reset_world))  // OP only
        )
}
```

### `callback` — survival-safe (recommended)

`DialogAction::callback(fn_ptr)` emits `/trigger sand.dialog set <id>` instead.
The trigger is server-evaluated, so any player can press the button regardless of permissions.
Sand auto-generates the `__sand_dialog_init` (registers the trigger objective) and
`__sand_dialog_tick` (dispatches callbacks) functions registered in `minecraft:load`
and `minecraft:tick`.

```rust
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
```

## Opening a dialog

Use `cmd::show_dialog` to open a dialog from a function:

```rust
#[function]
pub fn open_welcome_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
}
```

`DialogRef::local("welcome")` resolves to `yourpack:welcome` at export time via the
`__sand_local:` sentinel. Dialogs can also be opened from other dialog buttons using
`DialogAction::open_dialog(DialogRef::local("other"))`.

## Pause screen and Quick Actions

Expose dialogs through vanilla dialog tags with `DialogTag`:

```rust
#[component]
pub fn quick_actions_dialogs() -> DialogTag {
    DialogTag::quick_actions().dialog(DialogRef::local("welcome"))
}

#[component]
pub fn pause_screen_dialogs() -> DialogTag {
    DialogTag::pause_screen_additions().dialog(DialogRef::local("welcome"))
}
```

The helpers emit `data/minecraft/tags/dialog/quick_actions.json` and
`data/minecraft/tags/dialog/pause_screen_additions.json`.

## Version support

```rust
let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.6").unwrap()).unwrap();
assert!(profile.supports_dialogs());
```

Gate dialog registration behind `profile.supports_dialogs()` when targeting a version
range that predates dialog support.
