//! Typed dialogs.

use sand_core::prelude::*;
use sand_macros::component;

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice_local("welcome")
        .title(Text::new("Welcome").gold())
        .body(DialogBody::text(Text::new("Choose your next action.").aqua()))
        .button(
            DialogButton::new(Text::new("Start").green()).action(DialogAction::run_function(
                ResourceLocation::new("example", "start").unwrap(),
            )),
        )
}
