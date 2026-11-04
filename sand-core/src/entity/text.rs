//! Shared EntityText materialization. Author text is written through ordinary
//! typed text payloads; macro functions interpolate numeric scalars only.
use super::{
    property::{EntityText, EntityTextSegment},
    state::StateFieldReference,
};
use sand_commands::{CommandError, CommandResult};

pub(crate) struct MaterializedText {
    pub(crate) value: String,
    pub(crate) dynamic: bool,
    pub(crate) commands: Vec<String>,
    pub(crate) helpers: Vec<(String, Vec<String>)>,
}

/// Render one canonical template into a caller-owned temporary storage path.
/// Each segment adds bounded work and the caller owns clearing the final value.
/// Numeric scratch is local to this materialization and removed before return.
pub(crate) fn materialize(
    text: &EntityText,
    storage: &str,
    path: &str,
    helper: &str,
    holder: impl Fn(&StateFieldReference) -> CommandResult<String>,
) -> CommandResult<MaterializedText> {
    let args = "__sand_text_numbers";
    let mut rendered = Vec::new();
    let mut updates = Vec::new();
    let mut numeric = Vec::new();
    let mut dynamic = false;
    for (index, segment) in text.segments().iter().enumerate() {
        let color = match segment {
            EntityTextSegment::Canonical { .. } => String::new(),
            EntityTextSegment::Literal { color, .. }
            | EntityTextSegment::Numeric { color, .. }
            | EntityTextSegment::Enum { color, .. }
            | EntityTextSegment::Flag { color, .. } => color
                .as_ref()
                .map(|color| format!(",color:\"{color}\""))
                .unwrap_or_default(),
        };
        let destination = format!("{path}.extra[{index}].text");
        match segment {
            EntityTextSegment::Canonical { component } => {
                component.try_to_json_value()?;
                rendered.push(component.to_string());
            }
            EntityTextSegment::Literal { text, .. } => rendered.push(format!(
                "{{text:{}{color}}}",
                serde_json::to_string(text).expect("Rust string JSON")
            )),
            EntityTextSegment::Numeric {
                objective, field, ..
            } => {
                dynamic = true;
                let holder = holder(field)?;
                updates.push(format!("execute store result storage {storage} {args}.n{index} int 1 run scoreboard players get {holder} {objective}"));
                numeric.push(format!(
                    "$data modify storage {storage} {destination} set value \"$(n{index})\""
                ));
                rendered.push(format!("{{text:\"\"{color}}}"));
            }
            EntityTextSegment::Enum {
                objective,
                field,
                variants,
                ..
            } => {
                dynamic = true;
                let holder = holder(field)?;
                for (score, label) in variants {
                    updates.push(format!("execute if score {holder} {objective} matches {score} run data modify storage {storage} {destination} set value {}",serde_json::to_string(label).expect("Rust string JSON")));
                }
                rendered.push(format!("{{text:\"\"{color}}}"));
            }
            EntityTextSegment::Flag {
                objective,
                field,
                disabled,
                enabled,
                ..
            } => {
                dynamic = true;
                let holder = holder(field)?;
                for (score, label) in [(0, disabled), (1, enabled)] {
                    updates.push(format!("execute if score {holder} {objective} matches {score} run data modify storage {storage} {destination} set value {}",serde_json::to_string(label).expect("Rust string JSON")));
                }
                rendered.push(format!("{{text:\"\"{color}}}"));
            }
        }
    }
    let value = format!("{{text:\"\",extra:[{}]}}", rendered.join(","));
    let mut commands = vec![format!(
        "data modify storage {storage} {path} set value {value}"
    )];
    if !numeric.is_empty() {
        commands.push(format!("data remove storage {storage} {args}"));
    }
    commands.extend(updates);
    let mut helpers = Vec::new();
    if !numeric.is_empty() {
        commands.push(format!("function {helper} with storage {storage} {args}"));
        commands.push(format!("data remove storage {storage} {args}"));
        helpers.push((helper.to_owned(), numeric));
    }
    Ok(MaterializedText {
        value,
        dynamic,
        commands,
        helpers,
    })
}

/// Snapshot consumers must not retain unresolved canonical score/NBT/selector
/// components that would otherwise resolve only when the item is first opened.
pub(crate) fn literal_component(component: &sand_commands::TextComponent) -> CommandResult<()> {
    fn dynamic(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
                matches!(key.as_str(), "score" | "selector" | "nbt") || dynamic(value)
            }),
            serde_json::Value::Array(values) => values.iter().any(dynamic),
            _ => false,
        }
    }
    if dynamic(&component.try_to_json_value()?) {
        Err(CommandError::new(
            "text snapshot",
            "text",
            "canonical dynamic score, selector or NBT components require a typed EntityText field segment",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityFlag, EntityScore};
    #[test]
    fn dynamic_text_never_places_author_literals_in_macro_bodies() {
        let literal = "quote \" \\ newline\n $(missing)";
        let template = EntityText::new()
            .literal(literal)
            .score(EntityScore::<i32>::new(
                "text_test",
                "state",
                "count",
                0,
                None,
            ))
            .flag(
                EntityFlag::new("text_test", "state", "flag", false),
                literal,
                literal,
            );
        let plan = materialize(
            &template,
            "test:scratch",
            "page",
            "test:text_numbers",
            |_| Ok("@s".into()),
        )
        .unwrap();
        assert!(plan.commands.iter().any(|line| line.contains("$(missing)")));
        assert_eq!(plan.helpers.len(), 1);
        assert!(
            plan.helpers[0]
                .1
                .iter()
                .all(|line| !line.contains("$(missing)")
                    && !line.contains("quote")
                    && line.contains("$(n1)"))
        );
    }
}
