use sand_core::DatapackComponent;
use sand_core::prelude::*;
use sand_macros::component;

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice_local("welcome")
        .title("Welcome")
        .body(DialogBody::text("Choose an action."))
        .button(DialogButton::new("Start").action(DialogAction::close()))
}

fn main() {
    let dialog = welcome_dialog();
    assert_eq!(dialog.resource_location().path(), "welcome");
    assert_eq!(dialog.component_dir(), "dialog");
}
