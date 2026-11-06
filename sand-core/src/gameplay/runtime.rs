//! Shared export plumbing for systems that extend an owned lifetime.
use super::{Lifetime, TaskContext, identity};
use crate::{
    IntoCommands, McFunction,
    entity::EntityKind,
    ir::{Actions, Cmd},
};
use sand_commands::{CommandError, CommandResult, NbtCompound, NbtValue};
use sand_components::{ComponentContent, DatapackComponent, ResourceLocation};
use std::sync::Arc;

/// Register the canonical native count operation used by guarded inventory workflows.
pub(super) fn count_modifier(
    id: ResourceLocation,
    score: &sand_commands::ObjectiveName,
    holder: &str,
) -> ResourceLocation {
    crate::function::register_generated_component(
        sand_components::ItemModifier::new(id.clone()).function(
            sand_components::LootFunction::SetCount {
                count: sand_components::NumberProvider::Score {
                    target: serde_json::json!({"type":"minecraft:fixed", "name":holder}),
                    score: score.to_string(),
                    scale: None,
                },
                add: false,
            },
        ),
    );
    id
}

/// Lower already-validated canonical component JSON into native structured NBT.
/// Component text permits controls such as book/display newlines, so string
/// values and control-bearing keys need JSON/SNBT escapes rather than the
/// generic authoring NBT string formatter. No author literal can become a new
/// physical command line. Native NBT has neither null nor unsigned integers.
pub(super) fn component_nbt(value: &serde_json::Value) -> CommandResult<NbtValue> {
    Ok(match value {
        serde_json::Value::Null => {
            return Err(CommandError::new(
                "component NBT",
                "data",
                "native NBT has no null value",
            ));
        }
        serde_json::Value::Bool(value) => NbtValue::Bool(*value),
        serde_json::Value::Number(value) => match value.as_i64() {
            Some(value) => match i32::try_from(value) {
                Ok(value) => NbtValue::Int(value),
                Err(_) => NbtValue::Long(value),
            },
            None if value.is_u64() => {
                return Err(CommandError::new(
                    "component NBT",
                    "data",
                    "native NBT integer exceeds i64::MAX",
                ));
            }
            None => NbtValue::Double(value.as_f64().expect("JSON finite number")),
        },
        serde_json::Value::String(value) => {
            NbtValue::raw(serde_json::to_string(value).expect("Rust string JSON"))
        }
        serde_json::Value::Array(values) => NbtValue::List(
            values
                .iter()
                .map(component_nbt)
                .collect::<CommandResult<_>>()?,
        ),
        serde_json::Value::Object(values) => {
            let fields = values
                .iter()
                .map(|(key, value)| Ok((key.clone(), component_nbt(value)?)))
                .collect::<CommandResult<Vec<_>>>()?;
            if values.keys().any(|key| key.chars().any(char::is_control)) {
                // NbtCompound's general key formatter intentionally does not
                // encode control characters. Quote these component keys with
                // the same safe literal encoder used for their values.
                NbtValue::raw(format!(
                    "{{{}}}",
                    fields
                        .iter()
                        .map(|(key, value)| format!(
                            "{}:{value}",
                            serde_json::to_string(key).expect("Rust string JSON")
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                ))
            } else {
                fields
                    .into_iter()
                    .fold(NbtCompound::new(), |compound, (key, value)| {
                        compound.field(key, value)
                    })
                    .into()
            }
        }
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_strings_and_keys_preserve_controls_without_splitting_commands() {
        let value = serde_json::json!({"key\nline": ["first\r\nsecond\t\u{0}", {"quote\"\\\rkey": "line\nline"}]});
        let encoded = component_nbt(&value).unwrap().to_string();
        assert!(!encoded.contains(['\n', '\r', '\t', '\0']), "{encoded:?}");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
            value
        );
    }

    #[test]
    fn component_numbers_preserve_signed_nbt_bounds() {
        assert_eq!(
            component_nbt(&serde_json::json!(i64::from(i32::MAX) + 1))
                .unwrap()
                .to_string(),
            "2147483648L"
        );
        for value in [serde_json::Value::Null, serde_json::json!(u64::MAX)] {
            assert_eq!(component_nbt(&value).unwrap_err().field, "data");
        }
    }
}
