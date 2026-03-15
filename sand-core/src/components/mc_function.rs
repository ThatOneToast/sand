use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::resource_location::ResourceLocation;

pub trait IntoCommands {
    fn into_commands(self) -> Vec<String>;
}

/// A Minecraft function file (.mcfunction) that contains a list of commands to be executed.
pub struct McFunction {
    pub location: ResourceLocation,
    pub commands: Vec<String>,
}

impl McFunction {
    /// Creates a new McFunction with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            commands: Vec::new(),
        }
    }

    /// Adds a single command to this function.
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.commands.push(cmd.into());
        self
    }

    /// Adds multiple commands to this function.
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
