//! Namespace-owned player join observations, including reconnects.
use super::{GeneratedTransitionFunction, TransitionPlan, builtin::DetectorScore};
use std::collections::BTreeSet;

pub(super) fn resolve(namespace: &str, handlers: &[&str]) -> TransitionPlan {
    let mut plan = TransitionPlan::default();
    if handlers.is_empty() {
        return plan;
    }
    let owner = DetectorScore::JoinLeaves.owner(namespace);
    let leaves = DetectorScore::JoinLeaves.objective(namespace);
    let previous = DetectorScore::JoinPrevious.objective(namespace);
    let source = "minecraft.custom:minecraft.leave_game";
    for (objective, criterion) in [(&leaves, source), (&previous, "dummy")] {
        plan.load_commands
            .push(format!("scoreboard objectives add {objective} {criterion}"));
        plan.private_objectives
            .insert(objective.clone(), (owner.clone(), source.to_string()));
    }
    // Reset our baseline for offline players too, preserving reload semantics.
    plan.reload_commands
        .push(format!("scoreboard players reset * {previous}"));
    let check = "__sand_join_check";
    let dispatch = "__sand_join_dispatch";
    plan.tick_commands
        .push(format!("function {namespace}:{check}"));
    let check_commands = vec![
        // Short circuit: callbacks must not make this same check dispatch twice.
        format!(
            "execute unless score @s {previous} = @s {previous} run return run function {namespace}:{dispatch}"
        ),
        format!(
            "execute if score @s {leaves} = @s {leaves} unless score @s {leaves} = @s {previous} run function {namespace}:{dispatch}"
        ),
    ];
    // Commit before callbacks, including when vanilla has no statistic yet.
    let mut dispatch_commands = vec![
        format!("scoreboard players set @s {previous} 0"),
        format!(
            "execute if score @s {leaves} = @s {leaves} run scoreboard players operation @s {previous} = @s {leaves}"
        ),
    ];
    for handler in handlers.iter().copied().collect::<BTreeSet<_>>() {
        dispatch_commands.push(format!("execute at @s run function {namespace}:{handler}"));
    }
    for (path, commands) in [(check, check_commands), (dispatch, dispatch_commands)] {
        plan.functions.push(GeneratedTransitionFunction {
            tracker_id: owner.clone(),
            source: source.to_string(),
            path: path.to_string(),
            commands,
        });
    }
    plan
}
