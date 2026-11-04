//! Retained owner resolution. Macro arguments are captured by Minecraft before
//! callback execution, so reentrant calls cannot overwrite an outer selection.
use crate::{
    IntoCommands,
    ir::{Actions, Cmd},
};
use sand_commands::{CommandResult, ObjectiveName};
use sand_components::ResourceLocation;

/// The compilation's pack namespace, independent of a definition's resource
/// namespace. Shared admission/transaction state must cover imported definitions
/// too, while separate exports retain isolated runtime state.
pub(crate) fn pack_namespace() -> CommandResult<String> {
    let local = ResourceLocation::new(crate::function::SAND_LOCAL_NS, "runtime_scope")
        .expect("internal resource identity");
    Ok(crate::function::resolve_export_location(&local)?
        .namespace()
        .to_owned())
}

pub(crate) fn objective(scope: &ResourceLocation) -> CommandResult<ObjectiveName> {
    let scope = crate::function::resolve_export_location(scope)?;
    let objective = ObjectiveName::logical(format!("sand runtime owner {}", scope.namespace()));
    crate::function::request_numeric_objectives([objective.clone()])?;
    Ok(objective)
}

pub(crate) fn task_tag(scope: &ResourceLocation) -> CommandResult<String> {
    let id = crate::function::resolve_export_location(scope)?.to_string();
    // Hex is injective and stays within Brigadier's unquoted selector alphabet.
    Ok(format!(
        "sand.task.{}",
        id.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    ))
}

/// Retained compiler-only context traversal from an owned runtime instance.
#[derive(Debug, Clone)]
pub struct OwnedContext {
    pub(crate) scope: ResourceLocation,
    pub(crate) kind: &'static str,
    pub(crate) field: &'static str,
    pub(crate) body: Actions,
    pub(crate) child: Option<ResourceLocation>,
    pub(crate) damage: Option<(sand_commands::DamageAmount, sand_commands::DamageKind)>,
}

impl OwnedContext {
    pub(crate) fn lower(&self, _owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let objective = objective(&self.scope)?;
        let mut lines = if let Some(child) = &self.child {
            let child = crate::function::resolve_export_location(child)?;
            let parent = crate::function::resolve_export_location(&self.scope)?;
            selectors(self.kind, &objective.to_string(), "$(owner)").into_iter().map(|selector| format!(
                "$execute as {selector} at @s run return run function {} {{ancestors:$(ancestors),parent:$(instance),parent_ledger:\"{}\"}}",
                resource(&child, "start_child"), resource(&parent, "task_ledger")
            )).collect()
        } else if let Some((amount, kind)) = &self.damage {
            let command = sand_commands::__private::damage_one(
                sand_commands::Selector::self_(),
                amount.clone(),
                *kind,
            )
            .try_run()?
            .join("\n");
            let mut lines = Vec::new();
            for target in selectors("entity", &objective.to_string(), "$(target)") {
                for owner in selectors(self.kind, &objective.to_string(), "$(owner)") {
                    lines.push(format!("$execute as {target} at @s if entity {owner} run return run {command} by {owner}"));
                }
            }
            lines
        } else {
            let body = crate::function::register_dyn_fn_dedup(
                "sand/runtime/owned_body",
                self.body.clone(),
            );
            selectors(
                self.kind,
                &objective.to_string(),
                &format!("$({})", self.field),
            )
            .into_iter()
            .map(|selector| {
                format!("$execute as {selector} at @s run return run function __sand_local:{body}")
            })
            .collect::<Vec<_>>()
        };
        lines.push("return 0".into());
        let proxy = crate::function::register_dyn_fn_dedup("sand/runtime/owned_context", lines);
        let child_guard = if self.child.is_some() {
            format!(
                " unless entity @s[tag={}.stopping] unless data entity @s data.ancestors[31]",
                task_tag(&self.scope)?
            )
        } else {
            String::new()
        };
        Ok(vec![Cmd::Raw(format!(
            "execute if entity @s[type=minecraft:marker,tag={}] if data entity @s data.{}{child_guard} run function __sand_local:{proxy} with entity @s data",
            task_tag(&self.scope)?,
            self.field
        ))])
    }
}

pub(crate) fn selectors(kind: &str, objective: &str, value: &str) -> Vec<String> {
    let score = format!("scores={{{objective}={value}}},limit=1");
    match kind {
        "player" => vec![format!("@a[{score}]")],
        "entity" => vec![
            format!("@a[{score}]"),
            format!("@e[type=!minecraft:player,distance=0..,{score}]"),
        ],
        kind => vec![format!("@e[type=minecraft:{kind},distance=0..,{score}]")],
    }
}

pub(crate) fn call(scope: &ResourceLocation, suffix: &str) -> Actions {
    Cmd::Function(resource(scope, suffix).to_string()).into_commands()
}

pub(crate) fn resource(scope: &ResourceLocation, suffix: &str) -> ResourceLocation {
    ResourceLocation::new(
        scope.namespace(),
        format!("__sand_system/lifetime/{}/{suffix}", scope.path()),
    )
    .expect("a validated scope and compiler suffix form a valid path")
}
