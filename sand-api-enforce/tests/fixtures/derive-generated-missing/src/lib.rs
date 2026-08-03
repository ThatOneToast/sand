use sand::SandStorage;

#[derive(SandStorage)]
#[sand(storage = "fixture:players", root = "magic")]
#[allow(dead_code)]
pub struct PlayerMagic {
    mana: i32,
}

// These type checks prove the real derive expansion ran in the successful
// fixture configuration. The build-time provider reads the same field
// declaration to obtain the generated `mana` identity.
const _: sand::__private::state::StorageSchema<PlayerMagic> = PlayerMagic::SCHEMA;
const _: fn() -> sand::__private::state::StorageField<PlayerMagic, i32> = PlayerMagic::mana;

include!(concat!(env!("OUT_DIR"), "/api_enforcement.rs"));
