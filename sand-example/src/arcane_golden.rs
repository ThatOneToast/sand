//! Golden tests for the arcane_starter dogfood datapack.

use sand_core::prelude::*;
use sand_macros::{component, function};

static ARCANE_MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");
static ARCANE_DASH: Cooldown = Cooldown::new("arcane_dash", Ticks::seconds(3));
static ARCANE_DATA: StorageVar<i32> = StorageVar::new("arcane:dogfood", "player.settings");

#[component(Load)]
pub fn arcane_load() {
    ARCANE_MANA.define();
    ARCANE_DASH.define();
    ARCANE_DATA.set_int(100);
}

#[component(Tick)]
pub fn arcane_tick() {
    ARCANE_DASH.tick_all_players();
    TypedExecute::as_players()
        .when(all![ARCANE_MANA.of("@s").gte(25), ARCANE_DASH.ready("@s"),])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));
}

#[function("arcane_dogfood:cast")]
pub fn arcane_cast() {
    TypedExecute::as_players_at_self()
        .when(all![ARCANE_MANA.of("@s").gte(25), ARCANE_DASH.ready("@s")])
        .run(cmd::function(
            ResourceLocation::new("arcane_dogfood", "cast/execute").unwrap(),
        ));
}

#[function("arcane_dogfood:cast/execute")]
pub fn arcane_cast_execute() {
    ARCANE_MANA.remove(Selector::self_(), 25);
    ARCANE_DASH.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Dash cast!").gold());
}

#[function("arcane_dogfood:interop")]
pub fn arcane_interop() {
    cmd::raw("function other_pack:api/do_special_thing");
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_core::{FunctionDescriptor, inventory};

    fn commands_for(path: &str) -> Vec<String> {
        let descriptor = inventory::iter::<FunctionDescriptor>()
            .find(|d| d.path == path)
            .unwrap_or_else(|| panic!("{path} descriptor not registered"));
        (descriptor.make)()
    }

    #[test]
    fn arcane_load_commands() {
        assert_eq!(
            arcane_load(),
            vec![
                "scoreboard objectives add arcane_mana dummy",
                "scoreboard objectives add arcane_dash dummy",
                "data modify storage arcane:dogfood player.settings set value 100",
            ]
        );
    }

    #[test]
    fn arcane_tick_commands() {
        assert_eq!(
            arcane_tick(),
            vec![
                "execute as @a if score @s arcane_dash matches 1.. run scoreboard players remove @s arcane_dash 1",
                "execute as @a if score @s arcane_mana matches 25.. if score @s arcane_dash matches 0 run title @s actionbar {\"bold\":true,\"color\":\"aqua\",\"text\":\"Dash ready\"}",
            ]
        );
    }

    #[test]
    fn arcane_cast_checks_conditions() {
        assert_eq!(
            commands_for("cast"),
            vec![
                "execute as @a at @s if score @s arcane_mana matches 25.. if score @s arcane_dash matches 0 run function arcane_dogfood:cast/execute",
            ]
        );
    }

    #[test]
    fn arcane_cast_execute_applies_effects() {
        assert_eq!(
            commands_for("cast/execute"),
            vec![
                "scoreboard players remove @s arcane_mana 25",
                "scoreboard players set @s arcane_dash 60",
                "tellraw @s {\"color\":\"gold\",\"text\":\"Dash cast!\"}",
            ]
        );
    }

    #[test]
    fn arcane_interop_is_raw() {
        let cmds = commands_for("interop");
        assert_eq!(cmds, vec!["function other_pack:api/do_special_thing"]);
    }
}
