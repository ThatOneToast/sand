//! Shared export plumbing for systems that extend an owned lifetime.
use super::{Lifetime, TaskContext, identity};
use crate::{
    IntoCommands, McFunction,
    entity::EntityKind,
    ir::{Actions, Cmd},
};
use sand_commands::CommandResult;
use sand_components::{ComponentContent, DatapackComponent, ResourceLocation};
use std::sync::Arc;

pub(super) fn hooks<K: EntityKind>(
    lifetime: &mut Lifetime<K>,
    start: &'static str,
    stop: &'static str,
    step: &'static str,
) {
    lifetime.step = Some(step);
    let old_start = lifetime.start.take();
    lifetime.start = Some(Arc::new(move |ctx| {
        let mut actions = identity::call(&ctx.scope, start);
        if let Some(callback) = &old_start {
            actions.extend([isolated(callback(ctx))]);
        }
        actions
    }));
    let old_stop = lifetime.stop.take();
    lifetime.stop = Some(Arc::new(move |ctx| {
        let mut actions = Actions::default();
        if let Some(callback) = &old_stop {
            actions.extend([isolated(callback(ctx.clone()))]);
        }
        actions.extend([identity::call(&ctx.scope, stop)]);
        actions
    }));
}
pub(super) fn isolated(body: Actions) -> Actions {
    Cmd::AnonymousFunction {
        prefix: "function".into(),
        body,
    }
    .into_commands()
}
pub(super) struct Function {
    location: ResourceLocation,
    build: Box<dyn Fn() -> CommandResult<Actions>>,
}
impl Function {
    pub(super) fn new(
        scope: &ResourceLocation,
        part: &str,
        build: impl Fn() -> CommandResult<Actions> + 'static,
    ) -> Self {
        Self {
            location: identity::resource(scope, part),
            build: Box::new(build),
        }
    }
}
impl DatapackComponent for Function {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }
    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
    fn to_json(&self) -> serde_json::Value {
        match self.try_content().expect("system must validate") {
            ComponentContent::Text(s) => serde_json::Value::String(s),
            ComponentContent::Json(v) => v,
        }
    }
    fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
        let actions =
            (self.build)().map_err(|error| sand_components::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "owned system".into(),
                field: error.field.clone(),
                message: error.to_string(),
            })?;
        McFunction::new(self.location.clone())
            .commands(actions)
            .try_content()
    }
}
pub(super) fn context<K: EntityKind>(lifetime: &Lifetime<K>) -> TaskContext<K> {
    TaskContext::new(lifetime.id.clone())
}

/// Reacquire the live entity after an authored callback. Vanilla dimension
/// transfer copies non-player entities; the old command executor can survive
/// as a detached object. Stable instance identity selects the current copy.
pub(super) fn resolve(scope: &ResourceLocation, destination: &str) -> CommandResult<Actions> {
    let tag = identity::task_tag(scope)?;
    let owner = identity::objective(scope)?;
    Ok(format!(
        "$execute as @e[type=minecraft:marker,tag={tag},scores={{{owner}=$(instance)}},limit=1] at @s rotated as @s run function {} with entity @s data",
        identity::resource(scope, destination)
    ).into_commands())
}
