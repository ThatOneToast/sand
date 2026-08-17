use sand::{hud_bar, hud_element, texture};

hud_bar!(
    name = "health",
    texture = "src/assets/health.png",
    steps = 10,
    height = 8,
    ascent = 8,
);

hud_element!(
    name = "status icon",
    texture = "src/assets/status.png",
    height = 8,
    ascent = 8,
);

texture!(id = "fixture:item/icon", path = "src/assets/icon.png");

const _: sand::resourcepack::BarHandle = HEALTH;
const _: sand::resourcepack::ElementHandle = STATUS_ICON;

#[doc(hidden)]
pub mod __private {
    include!(concat!(env!("OUT_DIR"), "/api_enforcement.rs"));
}
