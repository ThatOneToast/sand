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

// Compiler diagnostics are build wiring, not an author-facing generated API.
// Keeping the opaque include under `__private` makes that exclusion explicit
// to the facade surface model while still compiling it during a normal check.
#[doc(hidden)]
pub mod __private {
    include!(concat!(env!("OUT_DIR"), "/api_enforcement.rs"));
}
