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
use syn::{parse_macro_input, token, ItemFn, LitStr};

// ── Body transformation ───────────────────────────────────────────────────────

/// Convert a `#[function]` / `#[component(Tick|Load|Tag)]` block into the
/// `Vec<String>` construction the build pipeline expects.
///
/// Rules (applied to top-level statements only):
/// - `let` bindings → passed through unchanged.
/// - `expr;` (semicolon-terminated) → `__cmds.push(expr.to_string())`.
/// - trailing `expr` without `;` → `__cmds.extend(expr)`, so a bare
///   `mcfunction![…]` or `vec![…]` still works as a return value.
fn build_cmd_body(block: &syn::Block) -> proc_macro2::TokenStream {
    let n = block.stmts.len();
    let mut pieces: Vec<proc_macro2::TokenStream> = Vec::new();

    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_last = i + 1 == n;
        match stmt {
            syn::Stmt::Local(local) => {
                pieces.push(quote! { #local });
            }
            syn::Stmt::Expr(expr, None) if is_last => {
                // Trailing expression — extend so both Vec<String> and
                // any other IntoIterator<Item = String> work.
                pieces.push(quote! { __cmds.extend(#expr); });
            }
            syn::Stmt::Expr(expr, _semi) => {
                // Any other expression (with or without `;`): append as a command.
                pieces.push(quote! { __cmds.push((#expr).to_string()); });
            }
            syn::Stmt::Item(item) => {
                pieces.push(quote! { #item });
            }
            syn::Stmt::Macro(mac) => {
                let inner = &mac.mac;
                if is_last && mac.semi_token.is_none() {
                    // Trailing macro without `;` — extend (e.g. `mcfunction![…]`).
                    pieces.push(quote! { __cmds.extend(#inner); });
                } else {
                    // Macro statement with `;` — push result as a command string.
                    pieces.push(quote! { __cmds.push((#inner).to_string()); });
                }
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
        if let syn::FnArg::Receiver(r) = a { Some(r) } else { None }
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
            let name = path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
            match name.as_str() {
                "Tick" => Ok(ComponentFlag::Tick),
                "Load" => Ok(ComponentFlag::Load),
                _ => Err(syn::Error::new_spanned(
                    path,
                    format!("unknown flag `{name}`; expected `Tick`, `Load`, or `Tag = \"ns:name\"`"),
                )),
            }
        }
        syn::Meta::NameValue(nv) => {
            let name = nv.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
            if name == "Tag" {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    Ok(ComponentFlag::Tag(s.value()))
                } else {
                    Err(syn::Error::new_spanned(&nv.value, "expected a string literal, e.g. `Tag = \"minecraft:tick\"`"))
                }
            } else {
                Err(syn::Error::new_spanned(&nv.path, "expected `Tag = \"ns:name\"`"))
            }
        }
        _ => Err(syn::Error::new_spanned(&meta, "expected `Tick`, `Load`, or `Tag = \"ns:name\"`")),
    }
}

// ── Component expansion ───────────────────────────────────────────────────────

fn expand_component(func: ItemFn, flag: ComponentFlag) -> syn::Result<proc_macro2::TokenStream> {
    // Validate: no self receiver
    if let Some(recv) = func.sig.inputs.iter().find_map(|a| {
        if let syn::FnArg::Receiver(r) = a { Some(r) } else { None }
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
