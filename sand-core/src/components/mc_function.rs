use crate::ir::{Actions, Cmd};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::resource_location::ResourceLocation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::IntoCommands",
    module = "sand::component",
    summary = "Collects typed operations and explicit command text into authored actions.",
    context = "Function macros, query callbacks, and McFunction share this collection protocol. Actions retain structured nodes; strings are accepted as raw interoperability input.",
    minecraft = "The exporter validates and lowers collected actions into ordered mcfunction lines.",
    use_when = ["Accepting a gameplay helper result or command in an action-collecting API"],
    avoid_when = ["Inspecting rendered command text before export validation"],
    example = "use sand::component::IntoCommands;",
)]
/// Collects typed operations and explicit command text into authored actions.
pub trait IntoCommands {
    /// Collect this value into an ordered sequence of compiler actions.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::IntoCommands::into_commands",
        module = "sand::component",
        summary = "Collect this value into an ordered sequence of compiler actions.",
        context = "Collection preserves operation order and structured nodes until the export boundary. Constructing actions does not execute Minecraft commands.",
        minecraft = "The exporter validates and lowers collected actions into ordered mcfunction lines.",
        use_when = ["Accepting a gameplay helper result or command in an action-collecting API"],
        avoid_when = ["Inspecting rendered command text before export validation"],
        returns = "An ordered action collection retaining the input operations.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::component::IntoCommands>(into_commands_value: T)  {\n    let values = into_commands_value.into_commands();\n}",
    )]
    fn into_commands(self) -> Actions;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::McFunction",
    module = "sand::component",
    summary = "A Minecraft function file (.mcfunction) that contains a list of commands to be executed.",
    context = "A Minecraft function file (.mcfunction) that contains a list of commands to be executed. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::McFunction;",
)]
/// A Minecraft function file (.mcfunction) that contains a list of commands to be executed.
pub struct McFunction {
    location: ResourceLocation,
    commands: Actions,
}

impl McFunction {
    /// Create a new function with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::McFunction::new",
        module = "sand::component",
        kind = "method",
        summary = "Create a new function with the given resource location.",
        context = "Create a new function with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new function with the given resource location."),
        returns = "A `McFunction` representing a new function with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let mc_function = sand::component::McFunction::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            commands: Actions::default(),
        }
    }

    /// Add a single command to this function.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::McFunction::command",
        module = "sand::component",
        kind = "method",
        summary = "Add a single command to this function.",
        context = "Add a single command to this function. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cmd = "`cmd` provides the cmd added when building a single command to this function."),
        returns = "The `McFunction` value with the documented change applied to add a single command to this function.",
        example = "use sand::prelude::*;\n\nfn demonstrate(mc_function_value: sand::component::McFunction, cmd: impl sand::component::IntoCommands)  {\n    let updated_mc_function = mc_function_value.command(cmd);\n}",
    )]
    pub fn command(mut self, cmd: impl IntoCommands) -> Self {
        self.commands.extend([cmd]);
        self
    }

    /// Add multiple commands to this function.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::McFunction::commands",
        module = "sand::component",
        kind = "method",
        summary = "Add multiple commands to this function.",
        context = "Add multiple commands to this function. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cmds = "`cmds` provides the cmds added when building multiple commands to this function."),
        returns = "The `McFunction` value with the documented change applied to add multiple commands to this function.",
        example = "use sand::prelude::*;\n\nfn demonstrate(mc_function_value: sand::component::McFunction, cmds: impl IntoIterator < Item = impl sand::component::IntoCommands >)  {\n    let updated_mc_function = mc_function_value.commands(cmds);\n}",
    )]
    pub fn commands(mut self, cmds: impl IntoIterator<Item = impl IntoCommands>) -> Self {
        self.commands.extend(cmds);
        self
    }
}

impl IntoCommands for Actions {
    fn into_commands(self) -> Actions {
        self
    }
}

impl IntoCommands for String {
    fn into_commands(self) -> Actions {
        Actions(vec![Cmd::Raw(self)])
    }
}

impl IntoCommands for &str {
    fn into_commands(self) -> Actions {
        self.to_owned().into_commands()
    }
}

impl IntoCommands for McFunction {
    fn into_commands(self) -> Actions {
        self.commands.into_commands()
    }
}

impl IntoCommands for Vec<String> {
    fn into_commands(self) -> Actions {
        Actions(self.into_iter().map(Cmd::Raw).collect())
    }
}

impl IntoCommands for Vec<&str> {
    fn into_commands(self) -> Actions {
        self.into_iter().map(IntoCommands::into_commands).collect()
    }
}

impl IntoCommands for sand_commands::RawCommand {
    fn into_commands(self) -> Actions {
        self.into_inner().into_commands()
    }
}

impl<T: crate::cmd::Command> IntoCommands for T {
    fn into_commands(self) -> Actions {
        self.to_string().into_commands()
    }
}

impl DatapackComponent for McFunction {
    fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
        Ok(ComponentContent::Text(
            self.commands.lower(&self.location)?.join("\n"),
        ))
    }

    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        Value::Array(
            self.commands
                .lower(&self.location)
                .expect("function actions must validate before infallible serialization")
                .iter()
                .map(|c| Value::String(c.clone()))
                .collect(),
        )
    }

    fn content(&self) -> ComponentContent {
        self.try_content()
            .expect("function actions must validate before infallible serialization")
    }

    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
}

#[cfg(test)]
mod action_export_tests {
    use super::*;

    #[test]
    fn structured_actions_and_raw_interop_export_in_authored_order() {
        let structured = Actions(vec![Cmd::ScoreDefine {
            objective: "energy".into(),
            criterion: "dummy".into(),
        }]);
        let mut body = Actions::default();
        body.extend([structured]);
        body.extend(["say ready".into_commands()]);
        let resource = McFunction::new("game:initialize".parse().unwrap()).commands(body);
        let ComponentContent::Text(content) = resource.try_content().unwrap() else {
            panic!("expected function text")
        };
        assert_eq!(content, "scoreboard objectives add energy dummy\nsay ready");
    }

    #[test]
    fn invalid_structured_action_reports_function_and_action_without_panicking() {
        let body = Actions(vec![
            Cmd::Raw("say first".into()),
            Cmd::Execute {
                operations: vec![],
                run: Box::new(Cmd::Raw("say invalid".into())),
            },
        ]);
        let resource = McFunction::new("game:invalid".parse().unwrap()).commands(body);
        let error = resource.try_content().unwrap_err().to_string();
        assert!(error.contains("game:invalid"), "{error}");
        assert!(error.contains("actions[1].operations"), "{error}");
        assert!(error.contains("SAND-COMMAND-EXECUTE-EMPTY"), "{error}");
    }
}
