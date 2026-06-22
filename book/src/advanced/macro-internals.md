# Macro Internals

`#[function]`, `#[component(Load)]`, and `#[component(Tick)]` rewrite function
bodies into command collection functions and register descriptors through
inventory.

Attribute bodies collect typed command expressions via `IntoCommands`. Raw
string literals are rejected in attribute bodies; use `cmd::raw(...)` or the
advanced `mcfunction!` collection macro when intentional.
