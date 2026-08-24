use sand::prelude::PlayerSchema as PreludePlayerSchema;
use sand::systems::lifecycle::{FirstJoinCommands, RespawnCommands};
use sand::systems::player_data::PlayerSchema as ModulePlayerSchema;

pub fn compatibility_schemas() -> (PreludePlayerSchema, ModulePlayerSchema) {
    (
        PreludePlayerSchema::new("prelude"),
        ModulePlayerSchema::new("module"),
    )
}

pub fn lifecycle_helpers() -> (FirstJoinCommands, RespawnCommands) {
    (
        FirstJoinCommands::new("joined"),
        RespawnCommands::new("dead"),
    )
}
