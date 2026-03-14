/// Descriptor for a datapack function registered via `#[sand_macros::function]`.
///
/// All descriptors submitted with [`inventory::submit!`] are collected at
/// program startup and iterable via [`inventory::iter::<FunctionDescriptor>`].
///
/// # Fields
/// - `path` — the resource location *path* component (e.g. `"hello_world"`,
///   `"utils/tick"`). The namespace is applied by the caller at build time.
/// - `make` — a zero-argument factory function that returns the list of
///   command strings for this function. Using a factory enables both static
///   string literals and dynamic [`crate::Command`] builder values.
pub struct FunctionDescriptor {
    pub path: &'static str,
    pub make: fn() -> Vec<String>,
}

inventory::collect!(FunctionDescriptor);

/// Registry entry for a `#[component]`-annotated function.
///
/// The `make` fn pointer is a zero-argument function that constructs the
/// component and boxes it as a trait object. Registered at link time via
/// `inventory::submit!` — no user wiring needed.
pub struct ComponentFactory {
    pub make: fn() -> Box<dyn crate::DatapackComponent>,
}
inventory::collect!(ComponentFactory);

/// Registers a function as an entry in a Minecraft function tag.
///
/// Produced by `#[component(Tick)]`, `#[component(Load)]`, and
/// `#[component(Tag = "ns:name")]`. During `sand build` all descriptors for
/// the same `tag` are merged into a single tag JSON file:
///
/// | Variant | `tag` value | Output file |
/// |---|---|---|
/// | `Tick` | `"minecraft:tick"` | `data/minecraft/tags/function/tick.json` |
/// | `Load` | `"minecraft:load"` | `data/minecraft/tags/function/load.json` |
/// | `Tag = "ns:name"` | `"ns:name"` | `data/ns/tags/function/name.json` |
pub struct FunctionTagDescriptor {
    /// Full tag resource location, e.g. `"minecraft:tick"`.
    pub tag: &'static str,
    /// Function path component (namespace applied at build time), e.g. `"my_tick"`.
    pub function_path: &'static str,
}
inventory::collect!(FunctionTagDescriptor);
