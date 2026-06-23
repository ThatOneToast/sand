use sand_macros::SandStorage;

#[derive(SandStorage)]
#[sand(storage = "arcane:players", root = "player.magic")]
pub struct PlayerMagic {
    pub mana: i32,
    pub max_mana: i32,
    pub school: String,
}

fn main() {
    // SCHEMA const exists
    let _ = PlayerMagic::SCHEMA;

    // Field accessors return the right types
    let _mana: sand_core::state::StorageField<PlayerMagic, i32> = PlayerMagic::mana();
    let _max_mana: sand_core::state::StorageField<PlayerMagic, i32> = PlayerMagic::max_mana();
    let _school: sand_core::state::StorageField<PlayerMagic, String> = PlayerMagic::school();

    // Generated paths are correct
    assert_eq!(PlayerMagic::mana().field_name(), "mana");
    assert_eq!(PlayerMagic::max_mana().field_name(), "max_mana");
    assert_eq!(PlayerMagic::school().field_name(), "school");
    assert_eq!(PlayerMagic::mana().storage(), "arcane:players");
    assert_eq!(PlayerMagic::mana().root_path(), "player.magic");

    // Get/set/remove commands
    let get_cmd = PlayerMagic::mana().get();
    assert!(get_cmd.contains("data get storage arcane:players"), "got: {get_cmd}");
    assert!(get_cmd.contains("player.magic.mana"), "got: {get_cmd}");
}
