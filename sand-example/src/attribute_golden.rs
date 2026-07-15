//! Golden coverage for attribute-first datapack authoring.

use sand_core::prelude::*;
use sand_macros::{component, function};

static GOLDEN_MANA: ScoreVar<i32> = ScoreVar::new("golden_mana");
static GOLDEN_CASTING: Flag = Flag::new("golden_casting");
static GOLDEN_DASH: Cooldown = Cooldown::new("golden_dash", Ticks::seconds(3));
static GOLDEN_SETTINGS: StorageVar<i32> = StorageVar::new("golden:settings", "players.self.mana");

#[component(Load)]
pub fn golden_load() {
    GOLDEN_MANA.define();
    GOLDEN_CASTING.define();
    GOLDEN_DASH.define();
    GOLDEN_SETTINGS.set_int(100);
}

#[component(Tick)]
pub fn golden_tick() {
    GOLDEN_DASH.tick_all_players();
    TypedExecute::as_players()
        .when(all![
            GOLDEN_MANA.of("@s").gte(25),
            any![GOLDEN_DASH.ready("@s"), GOLDEN_SETTINGS.exists()],
            GOLDEN_CASTING.of("@s").is_false(),
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));
}

#[function("golden:nested_or")]
pub fn nested_or() {
    TypedExecute::as_players()
        .when(all![
            GOLDEN_MANA.of("@s").gte(25),
            any![GOLDEN_DASH.ready("@s"), GOLDEN_SETTINGS.exists()],
        ])
        .run(cmd::function(
            ResourceLocation::new("golden", "cast_dash").unwrap(),
        ));
}

#[function]
pub fn golden_text() {
    cmd::tellraw(Selector::all_players(), Text::new("Hello").gold());
    Title::of(Selector::self_())
        .title(Text::new("Dash").aqua())
        .subtitle(Text::new("Ready").green())
        .build();
}

#[function]
pub fn golden_storage() {
    GOLDEN_SETTINGS.set_int(100);
    GOLDEN_SETTINGS.as_path().key("enabled").set_bool(true);
}

#[function]
pub fn golden_interop() {
    cmd::raw("function other_pack:api/do_special_thing");
}

pub fn golden_welcome_dialog() -> Dialog {
    Dialog::multi_action_local("welcome")
        .title("Welcome")
        .body(DialogBody::text("Dash is ready."))
        .button(DialogButton::new("Start").action(DialogAction::run_command(
            cmd::function(ResourceLocation::new("golden", "start").unwrap()).to_string(),
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_core::{FunctionDescriptor, FunctionTagDescriptor, inventory};
    use serde_json::Value;

    fn commands_for(path: &str) -> Vec<String> {
        let descriptor = inventory::iter::<FunctionDescriptor>()
            .find(|d| d.path == path)
            .unwrap_or_else(|| panic!("{path} descriptor not registered"));
        (descriptor.make)()
    }

    #[test]
    fn golden_load_commands_and_tag() {
        assert_eq!(
            golden_load(),
            vec![
                "scoreboard objectives add golden_mana dummy",
                "scoreboard objectives add golden_casting dummy",
                "scoreboard objectives add golden_dash dummy",
                "data modify storage golden:settings players.self.mana set value 100",
            ]
        );

        let tag_entry = inventory::iter::<FunctionTagDescriptor>()
            .find(|d| d.function_path == "golden_load")
            .expect("golden_load not registered in function tags");
        assert_eq!(tag_entry.tag, "minecraft:load");
    }

    #[test]
    fn golden_tick_commands_and_tag() {
        assert_eq!(commands_for("golden_tick"), golden_tick());
        assert_eq!(
            golden_tick(),
            vec![
                "execute as @a if score @s golden_dash matches 1.. run scoreboard players remove @s golden_dash 1",
                "execute as @a if score @s golden_mana matches 25.. if score @s golden_dash matches 0 if score @s golden_casting matches 0 run title @s actionbar {\"bold\":true,\"color\":\"aqua\",\"text\":\"Dash ready\"}",
                "execute as @a if score @s golden_mana matches 25.. if data storage golden:settings players.self.mana if score @s golden_casting matches 0 run title @s actionbar {\"bold\":true,\"color\":\"aqua\",\"text\":\"Dash ready\"}",
            ]
        );

        let tag_entry = inventory::iter::<FunctionTagDescriptor>()
            .find(|d| d.function_path == "golden_tick")
            .expect("golden_tick not registered in function tags");
        assert_eq!(tag_entry.tag, "minecraft:tick");
    }

    #[test]
    fn golden_nested_or_output() {
        assert_eq!(
            commands_for("nested_or"),
            vec![
                "execute as @a if score @s golden_mana matches 25.. if score @s golden_dash matches 0 run function golden:cast_dash",
                "execute as @a if score @s golden_mana matches 25.. if data storage golden:settings players.self.mana run function golden:cast_dash",
            ]
        );
    }

    #[test]
    fn golden_text_output() {
        assert_eq!(
            golden_text(),
            vec![
                "tellraw @a {\"color\":\"gold\",\"text\":\"Hello\"}",
                "title @s times 10 70 20",
                "title @s subtitle {\"color\":\"green\",\"text\":\"Ready\"}",
                "title @s title {\"color\":\"aqua\",\"text\":\"Dash\"}",
            ]
        );
    }

    #[test]
    fn golden_storage_output() {
        assert_eq!(
            golden_storage(),
            vec![
                "data modify storage golden:settings players.self.mana set value 100",
                "data modify storage golden:settings players.self.mana.enabled set value 1b",
            ]
        );
    }

    #[test]
    fn golden_raw_interop_is_explicit() {
        assert_eq!(
            golden_interop(),
            vec!["function other_pack:api/do_special_thing"]
        );
    }

    #[test]
    fn golden_dialog_component_json() {
        let json = golden_welcome_dialog().to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:multi_action");
        assert_eq!(json["title"]["text"], Value::String("Welcome".to_string()));
        assert_eq!(
            json["body"][0]["contents"]["text"],
            Value::String("Dash is ready.".to_string())
        );
        assert_eq!(
            json["actions"][0]["action"]["command"],
            Value::String("function golden:start".to_string())
        );
    }
}
