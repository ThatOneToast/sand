use sand_core::prelude::*;
use sand_macros::{SandStorage, component, event, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(2));

#[derive(SandStorage)]
#[sand(storage = "example:players", root = "player")]
pub struct PlayerStats {
    pub mana: i32,
}

pub struct AteApple;

impl AdvancementEvent for AteApple {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:apple"))
    }

    fn guard() -> Option<Condition> {
        Some(MANA.of("@s").gte(1))
    }
}

#[component(Load)]
pub fn load() {
    MANA.define();
    DASH.define();
}

#[component]
pub fn welcome_advancement() -> Advancement {
    Advancement::new("example:welcome".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}

#[function]
pub fn spend_mana() {
    MANA.remove(Selector::self_(), 1);
    DASH.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("mana spent").green());
    cmd::raw("function other_pack:bridge");
}

#[event]
pub fn on_ate_apple(event: Event<AteApple>) {
    MANA.add(event.player(), 1);
    cmd::call(spend_mana);
}

fn main() {
    let item = CustomItem::new("minecraft:diamond").custom_data("example_wand");
    assert!(item.to_string().contains("custom_data"));

    let field: StorageField<PlayerStats, i32> = PlayerStats::mana();
    assert_eq!(field.storage(), "example:players");

    assert!(load().iter().any(|cmd| cmd.contains("scoreboard objectives add")));
    assert!(spend_mana()
        .iter()
        .any(|cmd| cmd == "function other_pack:bridge"));
    assert!(on_ate_apple()
        .iter()
        .any(|cmd| cmd.contains("scoreboard players add")));
}
