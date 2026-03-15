//! # sand-macros
//!
//! Procedural macros for the [Sand](https://github.com/ThatOneToast/sand)
//! Minecraft datapack toolkit.
//!
//! Provides three macros:
//!
//! - **`#[function]`** — turns a Rust function into a `.mcfunction` file,
//!   automatically registered via `inventory` at link time.
//! - **`#[component]`** — registers a datapack component (advancement, recipe,
//!   loot table, etc.) or hooks a function into `Tick`/`Load`/custom tags.
//! - **`run_fn!`** — defines an inline function and returns the
//!   `cmd::function(...)` call to invoke it.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::mcfunction;
//! use sand_macros::{component, function, run_fn};
//!
//! #[function]
//! pub fn greet() {
//!     mcfunction! { "say Hello from Sand!"; }
//! }
//!
//! #[component(Tick)]
//! pub fn tick() {
//!     mcfunction! { "scoreboard players add @a timer 1"; }
//! }
//!
//! #[component(Load)]
//! pub fn on_load() {
//!     mcfunction! { "scoreboard objectives add timer dummy"; }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, parse_macro_input, token};

// ── Body transformation ───────────────────────────────────────────────────────

/// Convert a `#[function]` / `#[component(Tick|Load|Tag)]` block into the
/// `Vec<String>` construction the build pipeline expects.
///
/// All expressions — with or without a trailing `;` — and all macro
/// invocations are routed through
/// [`IntoCommands::into_commands`](::sand_core::IntoCommands), which accepts:
///
/// - `String` / `&str` → single command
/// - `Vec<String>` → extends with all commands (call a helper fn directly)
/// - `mcfunction![…]` → extends with all commands the macro produces
///
/// This means plain helper functions returning `Vec<String>` work directly
/// alongside individual command strings — no wrapping in `mcfunction!` needed:
///
/// ```rust,ignore
/// #[function]
/// pub fn load() {
///     init_scoreboards();       // fn returning Vec<String> — commands extended
///     "say pack loaded";        // &str — single command
///     mcfunction!["say ready"]; // Vec<String> — commands extended
/// }
/// ```
fn build_cmd_body(block: &syn::Block) -> proc_macro2::TokenStream {
    let mut pieces: Vec<proc_macro2::TokenStream> = Vec::new();

    for stmt in &block.stmts {
        match stmt {
            // `let` bindings and item definitions pass through unchanged.
            syn::Stmt::Local(local) => {
                pieces.push(quote! { #local });
            }
            syn::Stmt::Item(item) => {
                pieces.push(quote! { #item });
            }
            // Every expression (with or without `;`) goes through IntoCommands.
            // This handles String, &str, Vec<String>, and any custom type.
            syn::Stmt::Expr(expr, _semi) => {
                pieces.push(quote! {
                    __cmds.extend(
                        ::sand_core::IntoCommands::into_commands(#expr)
                    );
                });
            }
            // Every macro invocation goes through IntoCommands so that
            // `mcfunction![…]` (returns Vec<String>) extends the list and
            // single-command macros still work.
            syn::Stmt::Macro(mac) => {
                let inner = &mac.mac;
                pieces.push(quote! {
                    __cmds.extend(
                        ::sand_core::IntoCommands::into_commands(#inner)
                    );
                });
            }
        }
    }

    quote! {
        let mut __cmds: ::std::vec::Vec<::std::string::String> =
            ::std::vec::Vec::new();
        #(#pieces)*
        __cmds
    }
}

/// Registers a free-standing function as a datapack `.mcfunction` file.
///
/// The function body must return a `Vec<String>` of commands, typically via
/// the [`mcfunction!`] macro which accepts any `Display`-implementing expression
/// (string literals or command builder values).
///
/// The function is automatically registered via [`inventory`] at program startup —
/// no manual collection or wiring is needed.
///
/// The resource location *path* is derived from the function name
/// (e.g. `fn hello_world` → path `"hello_world"`). The namespace is applied
/// by `sand build` from your `sand.toml`.
///
/// # Example
/// ```rust,ignore
/// use sand_macros::function;
/// use sand_core::cmd::{Execute, Selector, cmd};
///
/// #[function]
/// fn hello_world() {
///     mcfunction! {
///         r#"tellraw @a {"text":"Welcome!","color":"gold","bold":true}"#;
///         cmd::say("Enjoy your stay!");
///         Execute::new().as_(Selector::all_players()).run(cmd::kill(Selector::self_()));
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr; // #[function] takes no arguments
    let func = parse_macro_input!(item as ItemFn);

    match expand_function(func) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Expansion ─────────────────────────────────────────────────────────────────

fn expand_function(func: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &func.vis;
    let attrs = &func.attrs;

    // Validate: no `self` receiver (must be free-standing).
    if let Some(recv) = func.sig.inputs.iter().find_map(|a| {
        if let syn::FnArg::Receiver(r) = a {
            Some(r)
        } else {
            None
        }
    }) {
        return Err(syn::Error::new_spanned(
            recv,
            "#[function] cannot be applied to methods — use a free-standing `fn`",
        ));
    }

    // Validate: no parameters.
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[function] functions must take no parameters",
        ));
    }

    let factory_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{}_make", fn_name),
        proc_macro2::Span::call_site(),
    );

    let body = build_cmd_body(&func.block);

    Ok(quote! {
        #(#attrs)*
        #vis fn #fn_name() -> ::std::vec::Vec<::std::string::String> {
            #body
        }

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #factory_ident() -> ::std::vec::Vec<::std::string::String> {
            #fn_name()
        }

        ::sand_core::inventory::submit!(
            ::sand_core::FunctionDescriptor {
                path: #fn_name_str,
                make: #factory_ident,
            }
        );
    })
}

// ── #[component] ─────────────────────────────────────────────────────────────

/// Registers a free-standing function as a datapack component.
///
/// ## Plain `#[component]`
///
/// The function must take no parameters and return a type that implements
/// [`sand_core::DatapackComponent`]. It is automatically collected via
/// [`inventory`] — no manual wiring needed.
///
/// ```rust,ignore
/// #[component]
/// fn player_join() -> sand_core::Advancement {
///     Advancement::new("my_pack:player_join".parse().unwrap())
///         .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
/// }
/// ```
///
/// ## `#[component(Tick)]` / `#[component(Load)]`
///
/// The function body becomes an `.mcfunction` file **and** the function is
/// added to `data/minecraft/tags/functions/tick.json` (or `load.json`),
/// making it run every tick / once on load automatically.
///
/// ```rust,ignore
/// #[component(Tick)]
/// pub fn my_tick() {
///     mcfunction! {
///         "scoreboard players add @a timer 1";
///     }
/// }
///
/// #[component(Load)]
/// pub fn on_load() {
///     mcfunction! {
///         "scoreboard objectives add timer dummy";
///     }
/// }
/// ```
///
/// ## `#[component(Tag = "ns:name")]`
///
/// Like `Tick`/`Load` but targets any function tag you choose — useful for
/// hooking into other datapacks' APIs or creating your own tags.
///
/// ```rust,ignore
/// #[component(Tag = "my_lib:on_player_death")]
/// pub fn handle_death() {
///     mcfunction! { "say player died"; }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    match parse_component_flag(attr).and_then(|flag| expand_component(func, flag)) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Component flag parsing ────────────────────────────────────────────────────

enum ComponentFlag {
    /// Plain `#[component]` — returns a DatapackComponent.
    None,
    /// `#[component(Tick)]` — registers in `minecraft:tick`.
    Tick,
    /// `#[component(Load)]` — registers in `minecraft:load`.
    Load,
    /// `#[component(Tag = "ns:name")]` — registers in a custom function tag.
    Tag(String),
}

fn parse_component_flag(attr: TokenStream) -> syn::Result<ComponentFlag> {
    if attr.is_empty() {
        return Ok(ComponentFlag::None);
    }
    let meta = syn::parse::<syn::Meta>(attr)?;
    match &meta {
        syn::Meta::Path(path) => {
            let name = path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            match name.as_str() {
                "Tick" => Ok(ComponentFlag::Tick),
                "Load" => Ok(ComponentFlag::Load),
                _ => Err(syn::Error::new_spanned(
                    path,
                    format!(
                        "unknown flag `{name}`; expected `Tick`, `Load`, or `Tag = \"ns:name\"`"
                    ),
                )),
            }
        }
        syn::Meta::NameValue(nv) => {
            let name = nv
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            if name == "Tag" {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    Ok(ComponentFlag::Tag(s.value()))
                } else {
                    Err(syn::Error::new_spanned(
                        &nv.value,
                        "expected a string literal, e.g. `Tag = \"minecraft:tick\"`",
                    ))
                }
            } else {
                Err(syn::Error::new_spanned(
                    &nv.path,
                    "expected `Tag = \"ns:name\"`",
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            &meta,
            "expected `Tick`, `Load`, or `Tag = \"ns:name\"`",
        )),
    }
}

// ── Component expansion ───────────────────────────────────────────────────────

fn expand_component(func: ItemFn, flag: ComponentFlag) -> syn::Result<proc_macro2::TokenStream> {
    // Validate: no self receiver
    if let Some(recv) = func.sig.inputs.iter().find_map(|a| {
        if let syn::FnArg::Receiver(r) = a {
            Some(r)
        } else {
            None
        }
    }) {
        return Err(syn::Error::new_spanned(
            recv,
            "#[component] cannot be applied to methods — use a free-standing `fn`",
        ));
    }

    // Validate: no parameters
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[component] functions must take no parameters",
        ));
    }

    match flag {
        ComponentFlag::None => expand_component_plain(func),
        ComponentFlag::Tick => expand_component_tag(func, "minecraft:tick"),
        ComponentFlag::Load => expand_component_tag(func, "minecraft:load"),
        ComponentFlag::Tag(tag) => expand_component_tag(func, &tag),
    }
}

/// Plain `#[component]` — returns a `DatapackComponent`.
fn expand_component_plain(func: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    let factory_ident = proc_macro2::Ident::new(
        &format!("__sand_comp_{}_make", fn_name),
        proc_macro2::Span::call_site(),
    );

    Ok(quote! {
        #(#attrs)*
        #vis #sig #block

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #factory_ident() -> ::std::boxed::Box<dyn ::sand_core::DatapackComponent> {
            ::std::boxed::Box::new(#fn_name())
        }

        ::sand_core::inventory::submit!(
            ::sand_core::ComponentFactory { make: #factory_ident }
        );
    })
}

/// `#[component(Tick)]` / `#[component(Load)]` / `#[component(Tag = "...")]` —
/// registers the function body as an `.mcfunction` file AND adds it to `tag`.
fn expand_component_tag(func: ItemFn, tag: &str) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &func.vis;
    let attrs = &func.attrs;
    let tag_lit = LitStr::new(tag, proc_macro2::Span::call_site());

    let fn_make_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{}_make", fn_name),
        proc_macro2::Span::call_site(),
    );

    let body = build_cmd_body(&func.block);

    Ok(quote! {
        #(#attrs)*
        #vis fn #fn_name() -> ::std::vec::Vec<::std::string::String> {
            #body
        }

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #fn_make_ident() -> ::std::vec::Vec<::std::string::String> {
            #fn_name()
        }

        ::sand_core::inventory::submit!(
            ::sand_core::FunctionDescriptor {
                path: #fn_name_str,
                make: #fn_make_ident,
            }
        );

        ::sand_core::inventory::submit!(
            ::sand_core::FunctionTagDescriptor {
                tag: #tag_lit,
                function_path: #fn_name_str,
            }
        );
    })
}

// ── run_fn! ───────────────────────────────────────────────────────────────────

/// Returns a `cmd::function(...)` call and optionally registers an inline body
/// as a named `.mcfunction` file.
///
/// # With body — define + call inline
///
/// The body is registered as a named datapack function and the macro expands
/// to the `cmd::function(...)` call in one step:
///
/// ```rust,ignore
/// use sand_macros::{function, run_fn};
/// use sand_core::cmd::{Execute, Selector};
///
/// #[function]
/// fn my_fn() {
///     Execute::new()
///         .as_(Selector::all_players())
///         .run(run_fn!("hello_world:greet" {
///             cmd::say("Welcome!");
///             "scoreboard players add @s visits 1";
///         }));
/// }
/// ```
///
/// # Without body — shorthand for `cmd::function(...)`
///
/// ```rust,ignore
/// Execute::new()
///     .as_(Selector::all_players())
///     .run(run_fn!("hello_world:on_player_join"))
/// ```
#[proc_macro]
pub fn run_fn(input: TokenStream) -> TokenStream {
    match expand_run_fn(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct RunFnInput {
    name: LitStr,
    body: Option<syn::Block>,
}

impl syn::parse::Parse for RunFnInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        let body = if input.peek(token::Brace) {
            Some(input.parse::<syn::Block>()?)
        } else {
            None
        };
        Ok(RunFnInput { name, body })
    }
}

// ── Resource pack macros ──────────────────────────────────────────────────────

/// Registers a bitmap-font progress bar as a resource pack component.
///
/// Unicode codepoints are **assigned automatically** from the component name —
/// you never need to manage `\uXXXX` values by hand.
///
/// The macro generates a `pub const NAME: BarHandle` alongside the component
/// registration, where `NAME` is the uppercased `name` field. Use the handle
/// to display the bar in commands.
///
/// # Required fields
///
/// | Field | Type | Description |
/// |---|---|---|
/// | `name` | `&str` | Unique identifier; also used for auto-unicode derivation |
/// | `texture` | `&str` or `create!(…)` | PNG path **or** programmatic color spec |
/// | `steps` | `u32` | Number of frames in the sprite strip |
/// | `height` | `i32` | Rendered glyph height in pixels — increase to make the bar larger |
/// | `ascent` | `i32` | Vertical offset from baseline to glyph top — set equal to `height` for normal positioning |
///
/// # Optional fields
///
/// | Field | Type | Default | Description |
/// |---|---|---|---|
/// | `font` | `&str` | `"default"` | Font file name (without `.json`) |
/// | `texture_dest` | `&str` | `"font/<name>"` | Destination sub-path inside `assets/<ns>/textures/` |
/// | `unicode_start` | `char` | auto | Override the first codepoint (advanced use only) |
///
/// # `create!(…)` — programmatic pill-shaped texture
///
/// Use `create!(…)` in the `texture` field to have Sand generate a pill-shaped
/// sprite strip at build time — no external PNG needed.
///
/// | `create!` field | Type | Required | Description |
/// |---|---|---|---|
/// | `fill` | `u32` (`0xRRGGBBAA`) | yes | Filled-portion color |
/// | `empty` | `u32` (`0xRRGGBBAA`) | yes | Empty/background color |
/// | `frame_width` | `u32` | no | Width per frame in px (default = `2 × height`) |
///
/// # Sizing
///
/// `height` controls the rendered pixel height of the bar. `ascent` should
/// normally equal `height` so the top of the bar sits at the baseline.
/// Increase both to make the bar larger (e.g. `height: 14, ascent: 14`).
///
/// # Horizontal positioning
///
/// The actionbar is center-aligned. Use the generated handle's `show_at` or
/// `display_commands_at` to offset from center:
///
/// ```rust,ignore
/// // Shift 40 px left of center
/// HEALTH.show_at("@a", frame, "my_pack", -40);
/// HEALTH.display_commands_at("@s", "hp_frame", "my_pack", -40);
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use sand_macros::hud_bar;
///
/// // From a user-supplied PNG
/// hud_bar!(
///     name: "health",
///     texture: "src/assets/health_bar.png",
///     steps: 10,
///     height: 14,
///     ascent: 14,
/// );
///
/// // Programmatically generated pill-shaped sprite strip
/// hud_bar!(
///     name: "mana",
///     texture: create!(fill: 0x4444FFFF, empty: 0x222244FF),
///     steps: 10,
///     height: 14,
///     ascent: 14,
///     font: "hud",
/// );
/// ```
///
/// # Displaying the bar
///
/// ```rust,ignore
/// // Fixed frame (e.g. always full)
/// HEALTH.show("@a", HEALTH.steps - 1, "my_pack");
///
/// // Dynamic frame from a scoreboard value
/// HEALTH.display_commands("@s", "hp_frame", "my_pack");
/// ```
#[cfg(feature = "resourcepack")]
#[proc_macro]
pub fn hud_bar(input: TokenStream) -> TokenStream {
    match expand_hud_bar(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Registers a static single-character HUD overlay as a resource pack component.
///
/// Unicode codepoints are **assigned automatically** from the component name —
/// you never need to manage `\uXXXX` values by hand.
///
/// # Required fields
///
/// | Field | Type | Description |
/// |---|---|---|
/// | `name` | `&str` | Unique identifier; also used for auto-unicode derivation |
/// | `texture` | `&str` or `gen!(…)` | PNG path **or** programmatic color spec |
/// | `height` | `i32` | Rendered glyph height in pixels |
/// | `ascent` | `i32` | Vertical offset from baseline (negative = below baseline) |
///
/// # Optional fields
///
/// | Field | Type | Default | Description |
/// |---|---|---|---|
/// | `font` | `&str` | `"default"` | Font file name (without `.json`) |
/// | `texture_dest` | `&str` | `"font/<name>"` | Destination sub-path inside `assets/<ns>/textures/` |
/// | `unicode` | `char` | auto | Override the codepoint (advanced use only) |
///
/// # `gen!(…)` — programmatic texture
///
/// | `gen!` field | Type | Required | Description |
/// |---|---|---|---|
/// | `color` | `u32` (`0xRRGGBBAA`) | yes | Solid fill color |
/// | `width` | `u32` | no | Pixel width (default = `height`) |
///
/// # Examples
///
/// ```rust,ignore
/// use sand_macros::hud_element;
///
/// // From a user-supplied PNG
/// hud_element!(
///     name: "hotbar_bg",
///     texture: "src/assets/hotbar.png",
///     height: 22,
///     ascent: -10,
/// );
///
/// // Programmatically generated solid-color texture
/// hud_element!(
///     name: "dark_overlay",
///     texture: gen!(color: 0x00000080),
///     height: 22,
///     ascent: -10,
///     font: "hud",
/// );
/// ```
///
/// # Displaying the element
///
/// ```rust,ignore
/// HOTBAR_BG.show("@a", "my_pack");
///
/// // Shifted 40 px right of center
/// HOTBAR_BG.show_at("@a", "my_pack", 40);
/// ```
#[cfg(feature = "resourcepack")]
#[proc_macro]
pub fn hud_element(input: TokenStream) -> TokenStream {
    match expand_hud_element(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Registers a raw texture copy as a resource pack component.
///
/// The macro submits a [`sand_resourcepack::RawTexture`] descriptor via
/// [`inventory::submit!`] at link time.
///
/// # Required fields
///
/// | Field | Type | Description |
/// |---|---|---|
/// | `id` | `&str` | Resource location `<namespace>:<sub_path>` for the texture |
/// | `path` | `&str` | Project-root-relative path to the source PNG |
///
/// The `id` namespace determines the asset namespace (use `"minecraft:…"` to
/// override a vanilla texture). The sub-path is the path within `textures/`.
///
/// # Example
///
/// ```rust,ignore
/// use sand_macros::texture;
///
/// texture!(
///     id: "my_pack:item/custom_sword",
///     path: "src/assets/custom_sword.png",
/// );
/// ```
#[cfg(feature = "resourcepack")]
#[proc_macro]
pub fn texture(input: TokenStream) -> TokenStream {
    match expand_texture(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Resource pack expansion helpers ──────────────────────────────────────────

#[cfg(feature = "resourcepack")]
fn parse_kv_fields(
    input: proc_macro2::TokenStream,
) -> syn::Result<std::collections::HashMap<String, syn::Expr>> {
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;

    let pairs = Punctuated::<syn::ExprAssign, syn::Token![,]>::parse_terminated.parse2(input)?;

    let mut map = std::collections::HashMap::new();
    for pair in pairs {
        let key = match pair.left.as_ref() {
            syn::Expr::Path(p) => p
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new_spanned(&p.path, "expected a simple field name"))?
                .to_string(),
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected a simple field name, e.g. `name: \"value\"`",
                ));
            }
        };
        map.insert(key, *pair.right);
    }
    Ok(map)
}

#[cfg(feature = "resourcepack")]
fn require_lit_str(
    map: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
    macro_name: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match map.get(key) {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => Ok(quote! { #s }),
        Some(other) => Err(syn::Error::new_spanned(
            other,
            format!("`{key}` must be a string literal in {macro_name}!"),
        )),
        None => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("`{key}` is required in {macro_name}!"),
        )),
    }
}

#[cfg(feature = "resourcepack")]
fn require_lit_int(
    map: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
    macro_name: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match map.get(key) {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(n),
            ..
        })) => Ok(quote! { #n }),
        Some(other) => Err(syn::Error::new_spanned(
            other,
            format!("`{key}` must be an integer literal in {macro_name}!"),
        )),
        None => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("`{key}` is required in {macro_name}!"),
        )),
    }
}

#[cfg(feature = "resourcepack")]
fn require_lit_char(
    map: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
    macro_name: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match map.get(key) {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Char(c),
            ..
        })) => Ok(quote! { #c }),
        Some(other) => Err(syn::Error::new_spanned(
            other,
            format!("`{key}` must be a char literal in {macro_name}!"),
        )),
        None => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("`{key}` is required in {macro_name}!"),
        )),
    }
}

#[cfg(feature = "resourcepack")]
fn opt_lit_str<'a>(
    map: &'a std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> Option<&'a syn::LitStr> {
    match map.get(key) {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => Some(s),
        _ => None,
    }
}

#[cfg(feature = "resourcepack")]
fn opt_lit_char<'a>(
    map: &'a std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> Option<&'a syn::LitChar> {
    match map.get(key) {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Char(c),
            ..
        })) => Some(c),
        _ => None,
    }
}

#[cfg(feature = "resourcepack")]
fn expand_hud_bar(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let fields = parse_kv_fields(proc_macro2::TokenStream::from(input))?;

    let name = require_lit_str(&fields, "name", "hud_bar")?;
    let steps = require_lit_int(&fields, "steps", "hud_bar")?;
    let height = require_lit_int(&fields, "height", "hud_bar")?;
    let ascent = require_lit_int(&fields, "ascent", "hud_bar")?;

    let font_ts = match opt_lit_str(&fields, "font") {
        Some(s) => quote! { #s },
        None => quote! { "default" },
    };

    let tex_dest_ts = match opt_lit_str(&fields, "texture_dest") {
        Some(s) => quote! { #s },
        None => quote! { ::std::concat!("font/", #name) },
    };

    let name_str = match fields.get("name") {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => s.value(),
        _ => unreachable!(),
    };
    let factory_ident = proc_macro2::Ident::new(
        &format!("__sand_rp_bar_{}_make", name_str.replace(['-', ' '], "_")),
        proc_macro2::Span::call_site(),
    );
    let handle_ident = proc_macro2::Ident::new(
        &name_str.to_uppercase().replace(['-', ' '], "_"),
        proc_macro2::Span::call_site(),
    );

    // Optional unicode_start override.
    let uni_start_ts = match opt_lit_char(&fields, "unicode_start") {
        Some(c) => quote! { Some(#c) },
        None => quote! { None },
    };

    // Detect create!(…) vs string-literal texture.
    let is_gen = matches!(fields.get("texture"), Some(syn::Expr::Macro(_)));

    if is_gen {
        // Parse create!(fill: 0x…, empty: 0x…, frame_width: N)
        let gen_tokens = if let Some(syn::Expr::Macro(m)) = fields.get("texture") {
            let mac_name = m
                .mac
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            if mac_name != "create" {
                return Err(syn::Error::new_spanned(
                    &m.mac,
                    "expected `create!(fill = …, empty = …)` or a string literal for `texture`",
                ));
            }
            m.mac.tokens.clone()
        } else {
            unreachable!()
        };

        let gen_fields = parse_kv_fields(gen_tokens)?;
        let fill = require_lit_int(&gen_fields, "fill", "create")?;
        let empty = require_lit_int(&gen_fields, "empty", "create")?;
        let frame_width_ts = match gen_fields.get("frame_width") {
            Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            })) => quote! { #n },
            _ => quote! { 0u32 },
        };

        Ok(quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand_resourcepack::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand_resourcepack::GenHudBar {
                    name:          #name,
                    texture_dest:  #tex_dest_ts,
                    unicode_start: #uni_start_ts,
                    steps:         #steps,
                    height:        #height,
                    ascent:        #ascent,
                    font:          #font_ts,
                    fill:          #fill as u32,
                    empty:         #empty as u32,
                    frame_width:   #frame_width_ts,
                })
            }

            pub const #handle_ident: ::sand_resourcepack::BarHandle = ::sand_resourcepack::BarHandle {
                name:  #name,
                steps: #steps,
                font:  #font_ts,
            };

            ::sand_resourcepack::inventory::submit!(
                ::sand_resourcepack::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        })
    } else {
        let texture = require_lit_str(&fields, "texture", "hud_bar")?;

        Ok(quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand_resourcepack::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand_resourcepack::HudBar {
                    name:          #name,
                    texture_src:   #texture,
                    texture_dest:  #tex_dest_ts,
                    unicode_start: #uni_start_ts,
                    steps:         #steps,
                    height:        #height,
                    ascent:        #ascent,
                    font:          #font_ts,
                })
            }

            pub const #handle_ident: ::sand_resourcepack::BarHandle = ::sand_resourcepack::BarHandle {
                name:  #name,
                steps: #steps,
                font:  #font_ts,
            };

            ::sand_resourcepack::inventory::submit!(
                ::sand_resourcepack::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        })
    }
}

#[cfg(feature = "resourcepack")]
fn expand_hud_element(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let fields = parse_kv_fields(proc_macro2::TokenStream::from(input))?;

    let name = require_lit_str(&fields, "name", "hud_element")?;
    let height = require_lit_int(&fields, "height", "hud_element")?;
    let ascent = require_lit_int(&fields, "ascent", "hud_element")?;

    let font_ts = match opt_lit_str(&fields, "font") {
        Some(s) => quote! { #s },
        None => quote! { "default" },
    };

    let tex_dest_ts = match opt_lit_str(&fields, "texture_dest") {
        Some(s) => quote! { #s },
        None => quote! { ::std::concat!("font/", #name) },
    };

    let name_str = match fields.get("name") {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => s.value(),
        _ => unreachable!(),
    };
    let factory_ident = proc_macro2::Ident::new(
        &format!("__sand_rp_elem_{}_make", name_str.replace(['-', ' '], "_")),
        proc_macro2::Span::call_site(),
    );
    let handle_ident = proc_macro2::Ident::new(
        &name_str.to_uppercase().replace(['-', ' '], "_"),
        proc_macro2::Span::call_site(),
    );

    // Optional unicode override.
    let unicode_ts = match opt_lit_char(&fields, "unicode") {
        Some(c) => quote! { Some(#c) },
        None => quote! { None },
    };

    // Detect gen!(…) vs string-literal texture.
    let is_gen = matches!(fields.get("texture"), Some(syn::Expr::Macro(_)));

    if is_gen {
        let gen_tokens = if let Some(syn::Expr::Macro(m)) = fields.get("texture") {
            let mac_name = m
                .mac
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            if mac_name != "gen" {
                return Err(syn::Error::new_spanned(
                    &m.mac,
                    "expected `gen!(color: …)` or a string literal for `texture`",
                ));
            }
            m.mac.tokens.clone()
        } else {
            unreachable!()
        };

        let gen_fields = parse_kv_fields(gen_tokens)?;
        let color = require_lit_int(&gen_fields, "color", "gen")?;
        let width_ts = match gen_fields.get("width") {
            Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            })) => quote! { #n },
            _ => quote! { 0u32 },
        };

        Ok(quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand_resourcepack::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand_resourcepack::GenHudElement {
                    name:         #name,
                    texture_dest: #tex_dest_ts,
                    unicode:      #unicode_ts,
                    height:       #height,
                    ascent:       #ascent,
                    font:         #font_ts,
                    color:        #color as u32,
                    width:        #width_ts,
                })
            }

            pub const #handle_ident: ::sand_resourcepack::ElementHandle = ::sand_resourcepack::ElementHandle {
                name: #name,
                font: #font_ts,
            };

            ::sand_resourcepack::inventory::submit!(
                ::sand_resourcepack::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        })
    } else {
        let texture = require_lit_str(&fields, "texture", "hud_element")?;

        Ok(quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand_resourcepack::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand_resourcepack::HudElement {
                    name:         #name,
                    texture_src:  #texture,
                    texture_dest: #tex_dest_ts,
                    unicode:      #unicode_ts,
                    height:       #height,
                    ascent:       #ascent,
                    font:         #font_ts,
                })
            }

            pub const #handle_ident: ::sand_resourcepack::ElementHandle = ::sand_resourcepack::ElementHandle {
                name: #name,
                font: #font_ts,
            };

            ::sand_resourcepack::inventory::submit!(
                ::sand_resourcepack::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        })
    }
}

#[cfg(feature = "resourcepack")]
fn expand_texture(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let fields = parse_kv_fields(proc_macro2::TokenStream::from(input))?;

    let id = require_lit_str(&fields, "id", "texture")?;
    let path = require_lit_str(&fields, "path", "texture")?;

    let id_str = match fields.get("id") {
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => s.value(),
        _ => unreachable!(),
    };
    let (asset_ns, dest_path) = match id_str.split_once(':') {
        Some((ns, p)) => (ns.to_string(), p.to_string()),
        None => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("`id` must be a resource location `namespace:path`, got `{id_str}`"),
            ));
        }
    };
    let asset_ns_lit = proc_macro2::Literal::string(&asset_ns);
    let dest_path_lit = proc_macro2::Literal::string(&dest_path);

    let mangled = id_str.replace([':', '/', '-', ' '], "_");
    let factory_ident = proc_macro2::Ident::new(
        &format!("__sand_rp_tex_{}_make", mangled),
        proc_macro2::Span::call_site(),
    );

    Ok(quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        fn #factory_ident() -> ::std::boxed::Box<dyn ::sand_resourcepack::ResourcePackComponent> {
            ::std::boxed::Box::new(::sand_resourcepack::RawTexture {
                name:            #id,
                asset_namespace: #asset_ns_lit,
                dest_path:       #dest_path_lit,
                src_path:        #path,
            })
        }

        ::sand_resourcepack::inventory::submit!(
            ::sand_resourcepack::ResourcePackDescriptor {
                name: #id,
                make: #factory_ident,
            }
        );
    })
}

// ── run_fn! ───────────────────────────────────────────────────────────────────

fn expand_run_fn(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let RunFnInput { name, body } = syn::parse::<RunFnInput>(input)?;
    let name_val = name.value();

    // Extract the path part (after ":") for the FunctionDescriptor path.
    let path_part = match name_val.find(':') {
        Some(i) => &name_val[i + 1..],
        None => &name_val[..],
    };

    let fn_call = quote! {
        ::sand_core::cmd::function(
            #name.parse::<::sand_core::ResourceLocation>().unwrap()
        )
    };

    if let Some(block) = body {
        // Mangle the path into a valid Rust identifier.
        let mangled = path_part.replace(['/', ':'], "_");
        let fn_ident = proc_macro2::Ident::new(
            &format!("__sand_run_fn_{mangled}"),
            proc_macro2::Span::call_site(),
        );
        let path_lit = LitStr::new(path_part, name.span());
        let cmd_body = build_cmd_body(&block);

        Ok(quote! {
            {
                fn #fn_ident() -> ::std::vec::Vec<::std::string::String> {
                    #cmd_body
                }

                ::sand_core::inventory::submit!(
                    ::sand_core::FunctionDescriptor {
                        path: #path_lit,
                        make: #fn_ident,
                    }
                );

                #fn_call
            }
        })
    } else {
        Ok(fn_call)
    }
}
