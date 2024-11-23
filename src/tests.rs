#[cfg(test)]
mod tests {
    use crate::{
        commands::{
            clear, effect, gamemode::GameMode, kill, teleport, Distance, PlayerCommands,
            TargetFilter,
        },
        selector::{EntityTargets, TargetSelector},
        status_effects,
    };

    #[test]
    fn test_gamemode() {
        let command = PlayerCommands::Gamemode(GameMode::Creative(None));
        assert_eq!(command.to_string(), "/gamemode creative @s[]");
        let command = PlayerCommands::Gamemode(GameMode::Survival(Some(TargetSelector {
            selector: EntityTargets::AllPlayers,
            filter: TargetFilter {
                name: Some("TheOneTrueToast".to_string()),
                ..Default::default()
            },
        })));
        assert_eq!(
            command.to_string(),
            "/gamemode survival @a[name=TheOneTrueToast]"
        );
        let command = PlayerCommands::Gamemode(GameMode::Spectator(Some(TargetSelector {
            selector: EntityTargets::AllPlayers,
            filter: TargetFilter {
                level: Some(10),
                name: Some("TheOneTrueToast".to_string()),
                ..Default::default()
            },
        })));
        assert_eq!(
            command.to_string(),
            "/gamemode spectator @a[level=10,name=TheOneTrueToast]"
        );
    }

    #[test]
    fn test_teleport() {
        let command = PlayerCommands::Teleport(teleport::Teleport::PlayerToPlayer(
            "test".to_string(),
            "toast".to_string(),
        ));
        assert_eq!(command.to_string(), "/tp test toast");
        let command = PlayerCommands::Teleport(teleport::Teleport::AllPlayersTo(
            33.0,
            100.5,
            55.67,
            Some(TargetFilter {
                distance: Some(Distance::Max(50.0)),
                ..Default::default()
            }),
        ));
        assert_eq!(command.to_string(), "/tp @a[distance=..50] 33 100.5 55.67")
    }

    #[test]
    fn test_kill() {
        let command = PlayerCommands::Kill(kill::Kill(TargetSelector {
            selector: EntityTargets::AllPlayers,
            filter: TargetFilter {
                name: Some("TheOneTrueToast".to_string()),
                level: Some(10),
                ..Default::default()
            },
        }));
        assert_eq!(
            command.to_string(),
            "/kill @a[level=10,name=TheOneTrueToast]"
        );
    }

    #[test]
    fn test_effect() {
        let command = PlayerCommands::Effect(effect::Effect::Give(
            TargetSelector {
                selector: EntityTargets::AllPlayers,
                filter: TargetFilter {
                    name: Some("TheOneTrueToast".to_string()),
                    ..Default::default()
                },
            },
            status_effects::StatusEffects::Blindness,
            30,
            0,
        ));
        assert_eq!(
            command.to_string(),
            "/effect give @a[name=TheOneTrueToast] minecraft:blindness 30 0"
        );
        let command = PlayerCommands::Effect(effect::Effect::Clear(TargetSelector {
            selector: EntityTargets::AllPlayers,
            filter: TargetFilter {
                name: Some("TheOneTrueToast".to_string()),
                ..Default::default()
            },
        }));
        assert_eq!(
            command.to_string(),
            "/effect clear @a[name=TheOneTrueToast]"
        );
    }

    #[test]
    fn test_clear() {
        let command = PlayerCommands::Clear(clear::Clear(Some(TargetSelector {
            selector: EntityTargets::AllPlayers,
            filter: TargetFilter {
                name: Some("TheOneTrueToast".to_string()),
                ..Default::default()
            },
        })));
        assert_eq!(command.to_string(), "/clear @a[name=TheOneTrueToast]");
    }
}
