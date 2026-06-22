//! Typed dialogs.

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
