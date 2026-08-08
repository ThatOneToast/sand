use sand_macros::api;

#[api(
    registry = sand_api_contract,
    path = "sand::Widget",
    aliases = ["sand::prelude::Widget"],
    summary = "Carries a checked value from an implementation crate through its facade.",
    context = "The fixture proves API-producing lower crates can register contracts without depending on the facade that re-exports them.",
    minecraft = "Represents a small validated Minecraft-facing value.",
    use_when = ["A facade deliberately exposes a lower-crate domain type"],
    avoid_when = ["The type is private compiler wiring"],
    example = "let widget = sand::Widget;"
)]
pub struct Widget;

impl Widget {
    #[api(
        registry = sand_api_contract,
        path = "sand::Widget::value",
        aliases = ["sand::prelude::Widget::value"],
        summary = "Reads the facade widget's checked value.",
        context = "The inherent method remains reachable when its lower-crate owner is re-exported through the facade.",
        minecraft = "Returns the validated numeric value used by the fixture's Minecraft model.",
        use_when = ["Reading the typed facade value"],
        avoid_when = ["An untyped implementation-only value is sufficient"],
        returns = "The checked numeric value carried by the widget.",
        example = "let value = sand::Widget.value();"
    )]
    pub fn value(&self) -> u8 {
        7
    }
}
