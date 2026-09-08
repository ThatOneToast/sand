//! Canonical typed references to datapack functions.

use crate::registry::FunctionId;

/// Compiler registration mapping a function item's type to its resource path.
#[doc(hidden)]
pub struct FunctionItemTypeEntry {
    pub type_id: fn() -> std::any::TypeId,
    pub path: &'static str,
}
inventory::collect!(FunctionItemTypeEntry);

fn registered_path_for_function_value<F>(_value: F) -> Option<&'static str>
where
    F: Copy + 'static,
{
    let type_id = std::any::TypeId::of::<F>();
    for entry in inventory::iter::<FunctionItemTypeEntry>() {
        if (entry.type_id)() == type_id {
            return Some(entry.path);
        }
    }

    None
}

/// A typed handle accepted wherever Sand needs a datapack function.
///
/// Implemented by [`FunctionId`] and by Rust function items registered with
/// `#[function]`. Plain strings are intentionally excluded; APIs that need an
/// unsupported raw function token expose a separately named `_raw` method.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::FunctionRef",
    aliases = ["sand::command::FunctionRef", "sand::cmd::FunctionRef", "sand::prelude::FunctionRef", "sand::prelude::cmd::FunctionRef"],
    module = "sand",
    summary = "Canonical typed handle for a datapack function.",
    context = "Registered `#[function]` items and validated `FunctionId` values implement one capability shared by commands, callbacks, dialogs, and event rewards.",
    minecraft = "Resolves to the function resource identifier emitted in commands and generated datapack resources.",
    use_when = ["Writing a generic API that accepts a datapack function"],
    avoid_when = ["Accepting unchecked function text; expose an explicit `_raw` boundary"],
    example = "fn callback(function: impl sand::FunctionRef) {}",
)]
pub trait FunctionRef {
    /// Resolves this handle to the canonical function resource identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::FunctionRef::function_id",
        aliases = ["sand::command::FunctionRef::function_id", "sand::cmd::FunctionRef::function_id", "sand::prelude::FunctionRef::function_id", "sand::prelude::cmd::FunctionRef::function_id"],
        module = "sand",
        summary = "Resolves a typed function handle to its canonical FunctionId.",
        context = "Macro-generated function items and explicit identifiers converge here before lowering.",
        minecraft = "Returns the validated function resource location used in Minecraft output.",
        use_when = ["Lowering a typed function handle"],
        avoid_when = ["Parsing a raw function string"],
        returns = "The canonical function identifier.",
        example = "let id = callback.function_id();",
    )]
    fn function_id(self) -> FunctionId;
}

impl FunctionRef for FunctionId {
    fn function_id(self) -> FunctionId {
        self
    }
}

impl FunctionRef for &FunctionId {
    fn function_id(self) -> FunctionId {
        self.clone()
    }
}

impl<F> FunctionRef for F
where
    F: Fn() -> Vec<String> + Copy + 'static,
{
    fn function_id(self) -> FunctionId {
        let path = registered_path_for_function_value(self).unwrap_or_else(|| {
            panic!(
                "unregistered function item: the function must be annotated with \
                 #[function] or #[function(\"path\")] before it can be referenced"
            )
        });
        if path.contains(':') {
            path.parse()
                .expect("#[function] validates namespaced function paths")
        } else {
            FunctionId::local(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FunctionRef;

    #[test]
    fn pointer_sized_capturing_closure_panics_without_interpreting_capture_as_pointer() {
        let captured = 7_usize;
        let closure = move || vec![captured.to_string()];
        assert_eq!(
            std::mem::size_of_val(&closure),
            std::mem::size_of::<fn() -> Vec<String>>()
        );

        let result = std::panic::catch_unwind(|| closure.function_id());
        assert!(result.is_err());
    }
}
