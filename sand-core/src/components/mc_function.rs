use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::resource_location::ResourceLocation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::IntoCommands",
    module = "sand::component",
    summary = "Trait for types that can be converted into a list of Minecraft commands.",
    context = "Trait for types that can be converted into a list of Minecraft commands. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::IntoCommands;",
)]
/// Trait for types that can be converted into a list of Minecraft commands.
pub trait IntoCommands {
    /// Convert this value into a vector of command strings.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::IntoCommands::into_commands",
        module = "sand::component",
        summary = "Convert this value into a vector of command strings.",
        context = "Convert this value into a vector of command strings. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The ordered values produced to convert this value into a vector of command strings.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::component::IntoCommands>(into_commands_value: T)  {\n    let values = into_commands_value.into_commands();\n}",
    )]
    fn into_commands(self) -> Vec<String>;
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
    commands: Vec<String>,
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
        returns = "A newly constructed `McFunction` configured to create a new function with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let mc_function = sand::component::McFunction::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            commands: Vec::new(),
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
        params(cmd = "`cmd` supplies the cmd value used to add a single command to this function."),
        returns = "The `McFunction` value with the documented change applied to add a single command to this function.",
        example = "use sand::prelude::*;\n\nfn demonstrate(mc_function_value: sand::component::McFunction, cmd: impl Into < String >)  {\n    let updated_mc_function = mc_function_value.command(cmd);\n}",
    )]
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.commands.push(cmd.into());
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
        params(cmds = "`cmds` supplies the cmds value used to add multiple commands to this function."),
        returns = "The `McFunction` value with the documented change applied to add multiple commands to this function.",
        example = "use sand::prelude::*;\n\nfn demonstrate(mc_function_value: sand::component::McFunction, cmds: impl IntoIterator < Item = impl Into < String > >)  {\n    let updated_mc_function = mc_function_value.commands(cmds);\n}",
    )]
    pub fn commands(mut self, cmds: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.commands.extend(cmds.into_iter().map(|c| c.into()));
        self
    }
}

impl IntoCommands for String {
    fn into_commands(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoCommands for &str {
    fn into_commands(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IntoCommands for McFunction {
    fn into_commands(self) -> Vec<String> {
        self.commands
    }
}

impl IntoCommands for Vec<String> {
    fn into_commands(self) -> Vec<String> {
        self
    }
}

impl IntoCommands for Vec<&str> {
    fn into_commands(self) -> Vec<String> {
        self.into_iter().map(|s| s.to_string()).collect()
    }
}

impl IntoCommands for sand_commands::RawCommand {
    fn into_commands(self) -> Vec<String> {
        vec![self.into_inner()]
    }
}

impl<T: crate::cmd::Command> IntoCommands for T {
    fn into_commands(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl DatapackComponent for McFunction {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        Value::Array(
            self.commands
                .iter()
                .map(|c| Value::String(c.clone()))
                .collect(),
        )
    }

    fn content(&self) -> ComponentContent {
        ComponentContent::Text(self.commands.join("\n"))
    }

    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
}
