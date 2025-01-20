use sand_commands::prelude::MinecraftCommand;

/// Creates a Datapack Function from a list of minecraft commands.
#[macro_export]
macro_rules! dp_func {
    ($name:expr, $($command:expr),* $(,)?) => {{
        let mut commands: Vec<Box<dyn sand_commands::prelude::MinecraftCommand>> = Vec::new();
        $(
            let command = $command;
            let new_command: Box<dyn sand_commands::prelude::MinecraftCommand> = Box::new(command.clone());
            commands.push(new_command);
        )*
        crate::datapack::builder::DatapackFunction {
            name: $name.to_string(),
            commands,
        }
    }};

    ($name:expr) => {{
        DatapackFunction {
            name: $name.to_string(),
            commands: Vec::new(),
        }
    }};
}

/// Creates a Datapack Function from a vector of miencraft commands.
#[macro_export]
macro_rules! dp_func_from_vec {
    ($name:expr, $commands:expr) => {{
        DatapackFunction {
            name: $name.to_string(),
            commands: $commands,
        }
    }};
}
