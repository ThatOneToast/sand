//! Export-owned identities shared by built-in player detectors and typed reads.

/// Logical detector storage; physical objective names are resolved per export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DetectorScore {
    JoinLeaves,
    JoinPrevious,
    DeathCount,
    RespawnPhase,
    TimeSinceDeath,
    XpCurrent,
    XpPrevious,
    XpDelta,
    XpSeen,
}

impl DetectorScore {
    /// Keep each detector's related fields under one private lifecycle owner.
    pub(crate) fn owner(self, namespace: &str) -> String {
        let family = match self {
            Self::JoinLeaves | Self::JoinPrevious => "player_join",
            Self::DeathCount | Self::RespawnPhase | Self::TimeSinceDeath => "player_death",
            Self::XpCurrent | Self::XpPrevious | Self::XpDelta | Self::XpSeen => "player_xp",
        };
        format!("{namespace}:{family}")
    }

    /// Preserve the full namespace between a fixed family prefix and role suffix.
    pub(crate) fn objective(self, namespace: &str) -> String {
        let (prefix, role) = match self {
            Self::JoinLeaves => ("__sj_", 'l'),
            Self::JoinPrevious => ("__sj_", 'p'),
            Self::DeathCount => ("__sd_", 'c'),
            Self::RespawnPhase => ("__sd_", 'p'),
            Self::TimeSinceDeath => ("__sd_", 't'),
            Self::XpCurrent => ("__sx_", 'c'),
            Self::XpPrevious => ("__sx_", 'p'),
            Self::XpDelta => ("__sx_", 'd'),
            Self::XpSeen => ("__sx_", 's'),
        };
        format!("{prefix}{namespace}.{role}")
    }
}

/// Built-in tick sources use the same private resources and player lifecycle as joins.
pub(super) fn resolve(
    namespace: &str,
    deaths: &[&str],
    respawns: &[&str],
    levels: &[&str],
) -> super::TransitionPlan {
    use DetectorScore::*;
    let mut plan = super::TransitionPlan::default();
    if !deaths.is_empty() || !respawns.is_empty() {
        let death_origins =
            super::callback_origins(namespace, deaths.iter().chain(respawns.iter()).copied());
        let respawn_origins = super::callback_origins(namespace, respawns.iter().copied());
        declare(&mut plan, namespace, DeathCount, "deathCount");
        let count = DeathCount.objective(namespace);
        let phase = RespawnPhase.objective(namespace);
        let since = TimeSinceDeath.objective(namespace);
        let mut check = Vec::new();
        let mut dispatch = vec![format!("scoreboard players set @s {count} 0")];
        if !respawns.is_empty() {
            declare(&mut plan, namespace, RespawnPhase, "dummy");
            declare(
                &mut plan,
                namespace,
                TimeSinceDeath,
                "minecraft.custom:minecraft.time_since_death",
            );
            // Complete the old respawn phase before observing this tick's death.
            check.push(format!("function {namespace}:__sand_respawn_check"));
            dispatch.push(format!("scoreboard players set @s {phase} 1"));
            add_function(
                &mut plan,
                namespace,
                DeathCount,
                "__sand_respawn_check",
                &respawn_origins,
                vec![format!(
                    "execute if score @s {phase} matches 1 if score @s {since} matches 1.. run function {namespace}:__sand_respawn_dispatch"
                )],
            );
            let mut respawn = vec![format!("scoreboard players set @s {phase} 0")];
            respawn.extend(callbacks(namespace, respawns));
            add_function(
                &mut plan,
                namespace,
                DeathCount,
                "__sand_respawn_dispatch",
                &respawn_origins,
                respawn,
            );
        }
        check.push(format!(
            "execute if score @s {count} matches 1.. run function {namespace}:__sand_death_dispatch"
        ));
        dispatch.extend(callbacks(namespace, deaths));
        add_function(
            &mut plan,
            namespace,
            DeathCount,
            "__sand_death_dispatch",
            &death_origins,
            dispatch,
        );
        add_function(
            &mut plan,
            namespace,
            DeathCount,
            "__sand_death_check",
            &death_origins,
            check,
        );
        plan.tick_commands
            .push(format!("function {namespace}:__sand_death_check"));
    }
    if !levels.is_empty() {
        let level_origins = super::callback_origins(namespace, levels.iter().copied());
        for score in [XpCurrent, XpPrevious, XpDelta, XpSeen] {
            declare(&mut plan, namespace, score, "dummy");
        }
        let current = XpCurrent.objective(namespace);
        let previous = XpPrevious.objective(namespace);
        let delta = XpDelta.objective(namespace);
        let seen = XpSeen.objective(namespace);
        let check = vec![
            format!("execute store result score @s {current} run experience query @s levels"),
            format!(
                "execute unless score @s {seen} matches 1 run scoreboard players operation @s {previous} = @s {current}"
            ),
            format!("scoreboard players set @s {seen} 1"),
            format!("scoreboard players operation @s {delta} = @s {current}"),
            format!("scoreboard players operation @s {delta} -= @s {previous}"),
            format!(
                "execute if score @s {delta} matches 1.. run function {namespace}:__sand_xp_dispatch"
            ),
            format!("scoreboard players operation @s {previous} = @s {current}"),
        ];
        add_function(
            &mut plan,
            namespace,
            XpCurrent,
            "__sand_xp_check",
            &level_origins,
            check,
        );
        add_function(
            &mut plan,
            namespace,
            XpCurrent,
            "__sand_xp_dispatch",
            &level_origins,
            callbacks(namespace, levels),
        );
        plan.tick_commands
            .push(format!("function {namespace}:__sand_xp_check"));
    }
    plan
}

/// Provision only scores owned by this detector's namespace and semantic role.
fn declare(
    plan: &mut super::TransitionPlan,
    namespace: &str,
    score: DetectorScore,
    criterion: &str,
) {
    let objective = score.objective(namespace);
    plan.load_commands
        .push(format!("scoreboard objectives add {objective} {criterion}"));
    plan.private_objectives.insert(
        objective,
        (score.owner(namespace), "built-in player observation".into()),
    );
}

/// Preserve callback order and the current player's position for every handler.
fn callbacks(namespace: &str, handlers: &[&str]) -> Vec<String> {
    handlers
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|path| format!("execute at @s run function {namespace}:{path}"))
        .collect()
}

/// Publish a helper through the canonical private-function collision checks.
fn add_function(
    plan: &mut super::TransitionPlan,
    namespace: &str,
    score: DetectorScore,
    path: &str,
    origins: &std::collections::BTreeSet<crate::function::GeneratedOrigin>,
    commands: Vec<String>,
) {
    plan.functions.push(super::GeneratedTransitionFunction {
        origins: origins.clone(),
        tracker_id: score.owner(namespace),
        source: "built-in player observation".into(),
        path: path.into(),
        commands,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xp_score_operations_remain_per_player_and_advance_after_callbacks() {
        use DetectorScore::*;
        let plan = resolve("xp", &[], &[], &["leveled"]);
        assert_eq!(plan, resolve("xp", &[], &[], &["leveled"]));
        let commands = &plan
            .functions
            .iter()
            .find(|function| function.path == "__sand_xp_check")
            .unwrap()
            .commands;
        let (current, previous, delta, seen) = (
            XpCurrent.objective("xp"),
            XpPrevious.objective("xp"),
            XpDelta.objective("xp"),
            XpSeen.objective("xp"),
        );
        assert_eq!(
            *commands,
            vec![
                format!("execute store result score @s {current} run experience query @s levels"),
                format!(
                    "execute unless score @s {seen} matches 1 run scoreboard players operation @s {previous} = @s {current}"
                ),
                format!("scoreboard players set @s {seen} 1"),
                format!("scoreboard players operation @s {delta} = @s {current}"),
                format!("scoreboard players operation @s {delta} -= @s {previous}"),
                "execute if score @s ".to_owned()
                    + &delta
                    + " matches 1.. run function xp:__sand_xp_dispatch",
                format!("scoreboard players operation @s {previous} = @s {current}"),
            ]
        );
        assert_eq!(plan.tick_commands, ["function xp:__sand_xp_check"]);
    }
}
