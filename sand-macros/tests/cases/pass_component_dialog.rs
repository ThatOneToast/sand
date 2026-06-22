use sand_core::DatapackComponent;
use sand_core::prelude::*;
use sand_macros::{component, function};

#[function]
pub fn start() {
    cmd::say("start");
}

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice_local("welcome")
        .title(Text::new("Welcome").gold())
        .body(DialogBody::text(Text::new("Choose an action.").aqua()))
        .button(
            DialogButton::new(Text::new("Start").green())
                .tooltip(Text::new("Begin").yellow())
                .action(DialogAction::run_function(start)),
        )
            .button(
                DialogButton::new(Text::new("Rules").yellow())
                    .action(DialogAction::open_dialog(DialogRef::local("rules"))),
            )
    }

fn main() {
    let dialog = welcome_dialog();
    assert_eq!(dialog.resource_location().path(), "welcome");
    assert_eq!(dialog.component_dir(), "dialog");
    let json = dialog.to_json();
    assert_eq!(json["title"]["color"], "gold");
    assert_eq!(json["body"][0]["contents"]["color"], "aqua");
    assert_eq!(json["buttons"][0]["label"]["color"], "green");
    assert_eq!(
        json["buttons"][0]["action"]["command"],
        "/function __sand_local:start"
    );
    assert_eq!(json["buttons"][1]["action"]["dialog"], "__sand_local:rules");
}
