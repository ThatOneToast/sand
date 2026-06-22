//! Explicit raw escape hatches for interop.
//!
//! Keep raw commands out of normal examples. Use them only when Sand cannot
//! model the command because it belongs to another datapack, a mod, a snapshot,
//! a future feature, or a debugging workflow.

use sand_core::prelude::*;
use sand_macros::function;

#[function]
pub fn call_other_pack_api() {
    mcfunction! {
        // Escape hatch: this is another datapack's documented public contract.
        cmd::raw("function other_pack:api/do_special_thing");
    }
}
