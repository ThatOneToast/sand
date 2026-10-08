//! Shared base metadata for in-memory and Rust-project packaging.
/// Build metadata for one exact pack format, including modern required bounds.
pub fn base(description: &str, pack_format: u32) -> serde_json::Value {
    let mut value =
        serde_json::json!({"pack": {"description": description, "pack_format": pack_format}});
    if pack_format > 81 {
        value["pack"]["min_format"] = pack_format.into();
        value["pack"]["max_format"] = pack_format.into();
    }
    value
}
