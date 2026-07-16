#![forbid(unsafe_code)]

//! # sand-macros
//!
//! Procedural macros for the [Sand](https://github.com/ThatOneToast/sand)
//! Minecraft datapack toolkit.
//!
//! Provides five attribute macros and one procedural macro:
//!
//! - **`#[function]`** — turns a Rust function into a `.mcfunction` file,
//!   automatically registered via `inventory` at link time.
//! - **`#[component]`** — registers a datapack component (advancement, recipe,
//!   loot table, etc.) or hooks a function into `Tick`/`Load`/custom tags.
//! - **`#[event]`** — wires a handler function to either a stateless
//!   `AdvancementEvent` (`Event<T>` handler context) or an advanced custom
//!   `SandEvent` (concrete marker parameter). See [`event`] for the canonical
//!   split.
//! - **`#[schedule]`** — defines a function that runs for N ticks (with an
//!   optional interval), triggered at runtime via generated `_start`/`_stop` functions.
//! - **`#[item]`** — reads a `CustomItem`-returning function and generates a typed
//!   struct with `BASE`, `PREDICATE`, `CUSTOM_DATA_KEY` constants and an `item()` method.
//! - **`run_fn!`** — defines an inline function and returns the
//!   `cmd::function(...)` call to invoke it.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_macros::{component, function, run_fn};
//!
//! #[function]
//! pub fn greet() {
//!     cmd::tellraw(Selector::all_players(), Text::new("Hello from Sand").gold());
//! }
//!
//! #[component(Tick)]
//! pub fn tick() {
//!     cmd::say("Tick from Sand");
//! }
//!
//! #[component(Load)]
//! pub fn on_load() {
//!     cmd::say("Sand datapack loaded");
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, parse_macro_input, token};

// ── Body transformation ───────────────────────────────────────────────────────

/// Convert a `#[function]` / `#[component(Tick|Load|Tag)]` block into the
/// `Vec<String>` construction the build pipeline expects.
///
/// All expressions — with or without a trailing `;` — and macro invocations are
/// routed through [`IntoCommands::into_commands`](::sand_core::IntoCommands),
/// which accepts:
///
/// - `String` / `&str` → single command
/// - `Vec<String>` → extends with all commands (call a helper fn directly)
/// - typed command builders from `sand_core::cmd` / `sand_commands`
/// - `mcfunction![…]` → extends with all commands the macro produces for
///   advanced command collection
///
/// Attribute functions reject raw string literals directly. Use typed commands
/// for normal code, or `cmd::raw(...)` when an escape hatch is intentional.
///
/// ```rust,ignore
/// #[function]
/// pub fn load() {
///     init_scoreboards();       // fn returning Vec<String> — commands extended
///     cmd::say("pack loaded");  // typed command expression
///     cmd::raw("function other_pack:api/run"); // explicit escape hatch
/// }
/// ```
fn command_body_expr(expr: &syn::Expr) -> syn::Result<proc_macro2::TokenStream> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) => Err(syn::Error::new_spanned(
            expr,
            "raw string commands are not accepted directly inside #[function] or #[component(Tick|Load)]. Use a typed command such as cmd::say(\"hello\"), or use cmd::raw(\"say hello\") for an explicit escape hatch.",
        )),
        syn::Expr::Lit(_) => Err(syn::Error::new_spanned(
            expr,
            "this expression is not a Sand command. Attribute function body expressions must produce a typed command, Vec<String>, or another IntoCommands value.",
        )),
        syn::Expr::If(_) | syn::Expr::Match(_) => Err(syn::Error::new_spanned(
            expr,
            "Rust if/match statements are not supported directly inside Sand attribute functions yet. Use TypedExecute/typed condition helpers for Minecraft conditionals, or mcfunction! for advanced command collection.",
        )),
        syn::Expr::Return(_) => Err(syn::Error::new_spanned(
            expr,
            "do not return from a Sand attribute function body. Write typed command expressions as statements; Sand collects them into the generated .mcfunction output.",
        )),
        _ => Ok(quote! {
            __cmds.extend(
                ::sand_core::IntoCommands::into_commands(#expr)
            );
        }),
    }
}

fn build_cmd_body(block: &syn::Block) -> syn::Result<proc_macro2::TokenStream> {
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
                pieces.push(command_body_expr(expr)?);
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

    Ok(quote! {
        let mut __cmds: ::std::vec::Vec<::std::string::String> =
            ::std::vec::Vec::new();
        #(#pieces)*
        __cmds
    })
}

/// Registers a free-standing function as a datapack `.mcfunction` file.
///
/// Write typed command expressions directly in the function body. Sand collects
/// each expression into the generated command list. Use `mcfunction!` only for
/// advanced command grouping or migration code.
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
/// use sand_core::prelude::*;
///
/// #[function]
/// fn hello_world() {
///     cmd::tellraw(
///         Selector::all_players(),
///         Text::new("Welcome!").gold().bold(true),
///     );
///     cmd::say("Enjoy your stay!");
/// }
/// ```
#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_function_attr(attr);
    let func = parse_macro_input!(item as ItemFn);

    match attr.and_then(|path| expand_function(func, path)) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Expansion ─────────────────────────────────────────────────────────────────

fn parse_function_attr(attr: TokenStream) -> syn::Result<Option<String>> {
    if attr.is_empty() {
        return Ok(None);
    }

    let path = syn::parse::<LitStr>(attr)?;
    Ok(Some(path.value()))
}

fn function_descriptor_path(fn_name: &syn::Ident, explicit: Option<String>) -> String {
    explicit
        .and_then(|value| {
            value
                .split_once(':')
                .map(|(_, path)| path.to_string())
                .or(Some(value))
        })
        .unwrap_or_else(|| fn_name.to_string())
}

fn expand_function(
    func: ItemFn,
    explicit_path: Option<String>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = function_descriptor_path(fn_name, explicit_path.clone());
    // Store the full path (with namespace if given) for IntoFunctionRef resolution.
    let ptr_path_str = explicit_path.unwrap_or_else(|| fn_name.to_string());
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
    let type_id_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{}_type_id", fn_name),
        proc_macro2::Span::call_site(),
    );

    let body = build_cmd_body(&func.block)?;

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

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #type_id_ident() -> ::std::any::TypeId {
            ::std::any::Any::type_id(&#fn_name)
        }

        ::sand_core::inventory::submit!(
            ::sand_core::FunctionDescriptor {
                path: #fn_name_str,
                make: #factory_ident,
            }
        );

        ::sand_core::inventory::submit!(
            ::sand_core::FunctionPointerEntry {
                ptr: #fn_name as fn() -> ::std::vec::Vec<::std::string::String>,
                path: #ptr_path_str,
            }
        );

        ::sand_core::inventory::submit!(
            ::sand_core::FunctionPointerTypeEntry {
                type_id: #type_id_ident,
                path: #ptr_path_str,
            }
        );

        ::sand_core::inventory::submit!(
            ::sand_core::sand_components::dialog::DialogFunctionPointerEntry {
                ptr: #fn_name as fn() -> ::std::vec::Vec<::std::string::String>,
                path: #ptr_path_str,
            }
        );

        ::sand_core::inventory::submit!(
            ::sand_core::sand_components::dialog::DialogFunctionPointerTypeEntry {
                type_id: #type_id_ident,
                path: #ptr_path_str,
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
///     TIMER.tick(Selector::all_players());
/// }
///
/// #[component(Load)]
/// pub fn on_load() {
///     TIMER.define();
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
///     cmd::say("player died");
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

    let body = build_cmd_body(&func.block)?;

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

// ── #[event] ─────────────────────────────────────────────────────────────────

/// Turns a function into a Sand event handler.
///
/// Sand has two event-definition families:
///
/// - `AdvancementEvent` is a stateless definition of one vanilla advancement
///   trigger. Its handler receives `Event<T>`, a generated runtime context that
///   exposes the triggering player; Sand does not construct `T` or copy fields
///   declared on `T` into the context.
/// - `SandEvent` defines advanced custom dispatch such as typed tick polling,
///   owned observation lifecycle, generic event families, same-cycle chaining,
///   and explicit persistent conditions. Current `SandEvent` handlers use the
///   concrete marker type as their single parameter.
///
/// # Advancement-backed event
///
/// ```rust,ignore
/// use sand_core::prelude::*;
/// use sand_macros::event;
///
/// pub struct AteGoldenApple;
///
/// impl AdvancementEvent for AteGoldenApple {
///     type Trigger = ConsumeItemTrigger;
///
///     fn trigger() -> Self::Trigger {
///         ConsumeItemTrigger::new()
///             .item(ItemPredicate::id("minecraft:golden_apple"))
///     }
/// }
///
/// #[event]
/// pub fn on_ate(event: Event<AteGoldenApple>) {
///     cmd::tellraw(event.player(), Text::new("Golden apple eaten"));
/// }
/// ```
///
/// `Event<AteGoldenApple>` is the runtime handler context. `AteGoldenApple`
/// remains a type-level trigger definition; ordinary Rust fields on it would
/// not be event-time values.
///
/// # Advanced custom event
///
/// ```rust,ignore
/// use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
/// use sand_core::prelude::*;
/// use sand_macros::event;
///
/// static JUMPS: ScoreVar<i32> = ScoreVar::new("jumps");
/// static PREVIOUS_JUMPS: ScoreVar<i32> = ScoreVar::new("previous_jumps");
///
/// pub struct PlayerJumped;
///
/// impl SandEvent for PlayerJumped {
///     fn dispatch() -> SandEventDispatch {
///         SandEventDispatch::tick()
///             .as_players()
///             .when(PREVIOUS_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
///             .into()
///     }
///
///     fn setup() -> EventSetup {
///         EventSetup {
///             objectives: vec![
///                 "scoreboard objectives add jumps minecraft.custom:minecraft.jump".into(),
///                 "scoreboard objectives add previous_jumps dummy".into(),
///             ],
///             pre_observation: vec![],
///             post_observation: vec![
///                 "execute as @a run scoreboard players operation @s previous_jumps = @s jumps"
///                     .into(),
///             ],
///         }
///     }
/// }
///
/// #[event]
/// pub fn on_jump(_event: PlayerJumped) {
///     cmd::say("Jumped!");
/// }
/// ```
///
/// `SandEventDispatch::chain::<Parent>()` also supports implemented
/// same-cycle parent-to-child dispatch. It reuses the parent's detector and
/// preserves that cycle's player and position. Chained children can add an
/// explicit persistent condition with `.while_::<E>()`. Multi-parent
/// `after_any`/`after_all`, bounded `.within(...)`
/// correlation, advancement-backed graph parents, and participant-rich
/// contexts are planned; they are not accepted by this macro today.
///
/// Generic `SandEvent` definitions are supported and each concrete
/// monomorphization has distinct dispatch identity. A `#[event]` handler must
/// still name a constructible concrete marker parameter; use a unit adapter
/// type when the generic definition itself stores `PhantomData`.
///
/// # Attributes
///
/// `#[event]` takes exactly one handler parameter. Flat attributes such as
/// `id = "namespace:path"`, `slot = Head`, `item = "minecraft:stick"`,
/// and `custom_data = "{key:1b}"` are supported where the selected event
/// family uses them. `dispatch = "advancement"` is retained only as a
/// compatibility selector for older unit-style advancement handlers; new
/// advancement events should use `Event<T>`.
///
/// Reset behavior belongs to `AdvancementEvent::reset()` (or the compatibility
/// `SandEvent::revoke()` hook), not to an event attribute.
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    match expand_event(attr, func) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── New event attribute ───────────────────────────────────────────────────────

/// Flat key=value attributes for the new-style `#[event]` macro.
struct FlatEventAttr {
    /// `slot = Head | Chest | Legs | Feet | Offhand | Mainhand`
    slot: Option<syn::Ident>,
    /// `item = "namespace:item_id"`
    item: Option<syn::LitStr>,
    /// `custom_data = "{key:1b}"`
    custom_data: Option<syn::LitStr>,
    /// `id = "ns:path"` — override advancement resource location
    id_override: Option<syn::LitStr>,
    /// `dispatch = "advancement"` — use `AdvancementEvent` trait (instead of `SandEvent`)
    dispatch: Option<syn::LitStr>,
}

impl syn::parse::Parse for FlatEventAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut slot = None;
        let mut item = None;
        let mut custom_data = None;
        let mut id_override = None;
        let mut dispatch = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            match key.to_string().as_str() {
                "slot" => {
                    slot = Some(input.parse::<syn::Ident>()?);
                }
                "item" => {
                    item = Some(input.parse::<syn::LitStr>()?);
                }
                "custom_data" => {
                    custom_data = Some(input.parse::<syn::LitStr>()?);
                }
                "id" => {
                    id_override = Some(input.parse::<syn::LitStr>()?);
                }
                "dispatch" => {
                    dispatch = Some(input.parse::<syn::LitStr>()?);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!(
                            "unknown #[event] filter `{other}`; \
                             allowed: slot, item, custom_data, id, dispatch"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                let _comma: syn::Token![,] = input.parse()?;
            }
        }

        Ok(FlatEventAttr {
            slot,
            item,
            custom_data,
            id_override,
            dispatch,
        })
    }
}

// ── Event expansion ──────────────────────────────────────────────────────────

fn expand_event(attr: TokenStream, func: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &func.vis;
    let fn_attrs = &func.attrs;

    // Reject method receivers.
    if let Some(recv) = func.sig.inputs.iter().find_map(|a| {
        if let syn::FnArg::Receiver(r) = a {
            Some(r)
        } else {
            None
        }
    }) {
        return Err(syn::Error::new_spanned(
            recv,
            "#[event] cannot be applied to methods — use a free-standing `fn`",
        ));
    }

    // Exactly one typed parameter required.
    if func.sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[event] functions must take exactly one parameter: the event type \
             (e.g. `event: OnJoinEvent`)",
        ));
    }

    enum EventParam {
        Context {
            event_type_name: String,
            event_type_tokens: proc_macro2::TokenStream,
            binding_tokens: proc_macro2::TokenStream,
        },
        Legacy {
            event_type_name: String,
            param_type_tokens: proc_macro2::TokenStream,
            binding_tokens: proc_macro2::TokenStream,
        },
    }

    fn extract_event_context_type(
        ty: &syn::Type,
        tp: &syn::TypePath,
    ) -> syn::Result<Option<(String, proc_macro2::TokenStream, bool)>> {
        let Some(segment) = tp.path.segments.last() else {
            return Ok(None);
        };
        let is_damage_context = segment.ident == "DamageEvent";
        if segment.ident != "Event" && !is_damage_context {
            return Ok(None);
        }

        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return Err(syn::Error::new_spanned(
                ty,
                "#[event] generic handlers must specify the event type, e.g. `Event<MyEvent>` or `DamageEvent<MyDamageEvent>`",
            ));
        };

        if args.args.len() != 1 {
            return Err(syn::Error::new_spanned(
                args,
                "#[event] generic handlers must use exactly one generic argument, e.g. `Event<MyEvent>`",
            ));
        }

        let Some(syn::GenericArgument::Type(event_ty)) = args.args.first() else {
            return Err(syn::Error::new_spanned(
                args,
                "#[event] generic handlers must use a type argument, e.g. `Event<MyEvent>`",
            ));
        };

        let event_type_name = match event_ty {
            syn::Type::Path(event_tp) => event_tp
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
                .unwrap_or_else(|| "Event".to_string()),
            _ => "Event".to_string(),
        };

        Ok(Some((
            event_type_name,
            quote! { #event_ty },
            is_damage_context,
        )))
    }

    // Extract the handler parameter model. `Event<T>` is the primary typed
    // advancement path; bare event params stay available for legacy built-ins.
    let event_param: EventParam = {
        let param = func.sig.inputs.first().unwrap();
        match param {
            syn::FnArg::Typed(pt) => match pt.ty.as_ref() {
                syn::Type::Path(tp) => {
                    let ty = pt.ty.as_ref();
                    if let Some((event_type_name, event_type_tokens, is_damage_context)) =
                        extract_event_context_type(ty, tp)?
                    {
                        let binding_tokens = if is_damage_context {
                            quote! {
                                ::sand_core::event::DamageEvent::<#event_type_tokens>::context()
                            }
                        } else {
                            quote! {
                                ::sand_core::event::Event::<#event_type_tokens>::context()
                            }
                        };
                        EventParam::Context {
                            event_type_name,
                            event_type_tokens,
                            binding_tokens,
                        }
                    } else {
                        let name = tp.path.segments.last().unwrap().ident.to_string();
                        let param_type_tokens = quote! { #ty };
                        EventParam::Legacy {
                            event_type_name: name,
                            binding_tokens: quote! { #param_type_tokens },
                            param_type_tokens,
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "#[event] parameter type must be a path (e.g. `OnJoinEvent`)",
                    ));
                }
            },
            syn::FnArg::Receiver(r) => {
                return Err(syn::Error::new_spanned(r, "#[event] cannot be a method"));
            }
        }
    };

    let (event_type_name, dispatch_type_tokens, event_binding_tokens, is_event_context) =
        match &event_param {
            EventParam::Context {
                event_type_name,
                event_type_tokens,
                binding_tokens,
                ..
            } => (
                event_type_name.clone(),
                event_type_tokens.clone(),
                binding_tokens.clone(),
                true,
            ),
            EventParam::Legacy {
                event_type_name,
                param_type_tokens,
                binding_tokens,
            } => (
                event_type_name.clone(),
                param_type_tokens.clone(),
                binding_tokens.clone(),
                false,
            ),
        };

    // Parse the flat attribute: slot=, item=, custom_data=, id=
    let flat_attr: FlatEventAttr = if attr.is_empty() {
        FlatEventAttr {
            slot: None,
            item: None,
            custom_data: None,
            id_override: None,
            dispatch: None,
        }
    } else {
        syn::parse::<FlatEventAttr>(attr)?
    };

    let fn_make_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{}_make", fn_name),
        proc_macro2::Span::call_site(),
    );

    // Strip the event parameter from the generated function — the body is
    // unchanged but the actual runtime function takes no args.
    let body = build_cmd_body(&func.block)?;

    let id_override_tokens = match &flat_attr.id_override {
        Some(s) => quote! { ::std::option::Option::Some(#s) },
        None => quote! { ::std::option::Option::None },
    };

    // ── Shared preamble: emit the body function + hidden factory ──────────────
    // The `let event` binding enables `event.player()` inside handler bodies.
    let preamble = quote! {
        #(#fn_attrs)*
        #vis fn #fn_name() -> ::std::vec::Vec<::std::string::String> {
            let event = #event_binding_tokens;
            #body
        }

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #fn_make_ident() -> ::std::vec::Vec<::std::string::String> {
            #fn_name()
        }
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Map a slot ident string to `::sand_core::ArmorSlot::*` tokens.
    fn slot_to_armor_slot_tokens(slot: &syn::Ident) -> syn::Result<proc_macro2::TokenStream> {
        match slot.to_string().as_str() {
            "Head" | "Chest" | "Legs" | "Feet" | "Offhand" => {
                Ok(quote! { ::sand_core::ArmorSlot::#slot })
            }
            other => Err(syn::Error::new_spanned(
                slot,
                format!("invalid slot `{other}`; expected Head, Chest, Legs, Feet, or Offhand"),
            )),
        }
    }

    fn item_id_expr(item: &Option<syn::LitStr>) -> proc_macro2::TokenStream {
        match item {
            Some(lit) => {
                let s = lit.value();
                quote! { ::std::option::Option::Some(#s) }
            }
            None => quote! { ::std::option::Option::None },
        }
    }

    fn custom_data_expr(cd: &Option<syn::LitStr>) -> proc_macro2::TokenStream {
        match cd {
            Some(lit) => {
                let s = lit.value();
                quote! { ::std::option::Option::Some(#s) }
            }
            None => quote! { ::std::option::Option::None },
        }
    }

    // ── Dispatch selection ────────────────────────────────────────────────────
    let dispatch_tokens = match event_type_name.as_str() {
        "PlayerStartSneakingEvent" | "PlayerStartsSneaking" => {
            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::Tracked(
                        ::sand_core::TrackedTransition::new(
                            "player_sneaking",
                            ::sand_core::events::PLAYER_SNEAKING_TRACKED_SOURCE,
                            ::sand_core::TransitionKind::BecameTrue,
                        )
                    ),
                });
            }
        }

        "PlayerStopSneakingEvent" | "PlayerStopsSneaking" => {
            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::Tracked(
                        ::sand_core::TrackedTransition::new(
                            "player_sneaking",
                            ::sand_core::events::PLAYER_SNEAKING_TRACKED_SOURCE,
                            ::sand_core::TransitionKind::BecameFalse,
                        )
                    ),
                });
            }
        }

        // OnJoinEvent / OnJoin — scoreboard-backed join detection (fires after load/reload / new player mid-session)
        //
        // Uses JoinTick dispatch: `__sand_join_check` runs handlers for any online
        // player whose `__sand_join` score is not 1 (cleared on minecraft:load),
        // then sets the score to 1. Vanilla limitation: mid-session disconnect
        // → reconnect without /reload does not re-fire.
        "OnJoinEvent" | "OnJoin" => {
            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::JoinTick,
                });
            }
        }

        // FirstJoinEvent / FirstJoin — Advancement + Tick + no revoke (fires once ever)
        "FirstJoinEvent" | "FirstJoin" => {
            let trigger_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_trigger", fn_name),
                proc_macro2::Span::call_site(),
            );
            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #trigger_ident() -> ::sand_core::AdvancementTrigger {
                    ::sand_core::AdvancementTrigger::Tick
                }

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::Advancement {
                        make_trigger: #trigger_ident,
                        revoke: (|| false) as fn() -> bool,
                        guard: ::std::option::Option::None,
                    },
                });
            }
        }

        // PlayerLevelUpEvent / PlayerLevelsUp — tick-backed XP level-up detection
        //
        // Vanilla has no `minecraft:leveled_up` trigger. Sand generates its own
        // scoreboard objectives (__sand_xp_lvl, __sand_xp_prev, __sand_xp_delta,
        // __sand_xp_seen) and a `__sand_xp_check` tick function that fires all
        // registered handlers when a player's XP level increases.
        "PlayerLevelUpEvent" | "PlayerLevelsUp" => {
            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::XpLevelUp,
                });
            }
        }

        // OnDeathEvent / OnDeath — deathCount scoreboard tick loop
        "OnDeathEvent" | "OnDeath" => {
            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::DeathTick,
                });
            }
        }

        // OnRespawnEvent / OnRespawn — tick poll after death tag
        "OnRespawnEvent" | "OnRespawn" => {
            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::RespawnTick,
                });
            }
        }

        // ArmorEquipEvent — tick armor-tag equip detection
        "ArmorEquipEvent" => {
            let slot_ident = flat_attr.slot.as_ref().ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "ArmorEquipEvent requires `slot = Head|Chest|Legs|Feet|Offhand`",
                )
            })?;
            let slot_tokens = slot_to_armor_slot_tokens(slot_ident)?;
            let item_tok = item_id_expr(&flat_attr.item);
            let cd_tok = custom_data_expr(&flat_attr.custom_data);

            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::ArmorEquip {
                        slot: #slot_tokens,
                        item_id: #item_tok,
                        custom_data_snbt: #cd_tok,
                    },
                });
            }
        }

        // ArmorUnequipEvent — tick armor-tag unequip detection
        "ArmorUnequipEvent" => {
            let slot_ident = flat_attr.slot.as_ref().ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "ArmorUnequipEvent requires `slot = Head|Chest|Legs|Feet|Offhand`",
                )
            })?;
            let slot_tokens = slot_to_armor_slot_tokens(slot_ident)?;
            let item_tok = item_id_expr(&flat_attr.item);
            let cd_tok = custom_data_expr(&flat_attr.custom_data);

            quote! {
                #preamble

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::ArmorUnequip {
                        slot: #slot_tokens,
                        item_id: #item_tok,
                        custom_data_snbt: #cd_tok,
                    },
                });
            }
        }

        // HoldingItemEvent — tick poll on weapon.mainhand / weapon.offhand
        "HoldingItemEvent" => {
            let item_str = flat_attr
                .item
                .as_ref()
                .ok_or_else(|| {
                    syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "HoldingItemEvent requires `item = \"namespace:item_id\"`",
                    )
                })?
                .value();

            let slot_str = match flat_attr.slot.as_ref().map(|s| s.to_string()).as_deref() {
                Some("Offhand") => "weapon.offhand",
                None | Some("Mainhand") => "weapon.mainhand",
                Some(other) => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!(
                            "HoldingItemEvent `slot` must be `Mainhand` or `Offhand`, got `{other}`"
                        ),
                    ));
                }
            };

            let condition = match &flat_attr.custom_data {
                Some(cd) => {
                    let cd_str = cd.value();
                    format!("items entity @s {slot_str} {item_str}[minecraft:custom_data~{cd_str}]")
                }
                None => format!("items entity @s {slot_str} {item_str}"),
            };

            let cond_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_condition", fn_name),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #cond_ident() -> ::std::string::String {
                    #condition.to_string()
                }

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::TickPoll {
                        make_condition: #cond_ident,
                    },
                });
            }
        }

        // CurrentlyWearingEvent — tick poll on armor.<slot>
        "CurrentlyWearingEvent" => {
            let slot_str = match flat_attr.slot.as_ref().map(|s| s.to_string()).as_deref() {
                Some("Head") => "armor.head",
                Some("Chest") => "armor.chest",
                Some("Legs") => "armor.legs",
                Some("Feet") => "armor.feet",
                None => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "CurrentlyWearingEvent requires `slot = Head|Chest|Legs|Feet`",
                    ));
                }
                Some(other) => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!(
                            "CurrentlyWearingEvent `slot` must be Head, Chest, Legs, or Feet, \
                         got `{other}`"
                        ),
                    ));
                }
            };

            let item_str = flat_attr
                .item
                .as_ref()
                .ok_or_else(|| {
                    syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "CurrentlyWearingEvent requires `item = \"namespace:item_id\"`",
                    )
                })?
                .value();

            let condition = match &flat_attr.custom_data {
                Some(cd) => {
                    let cd_str = cd.value();
                    format!("items entity @s {slot_str} {item_str}[minecraft:custom_data~{cd_str}]")
                }
                None => format!("items entity @s {slot_str} {item_str}"),
            };

            let cond_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_condition", fn_name),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #cond_ident() -> ::std::string::String {
                    #condition.to_string()
                }

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::TickPoll {
                        make_condition: #cond_ident,
                    },
                });
            }
        }

        // Event<T> and compatibility `dispatch = "advancement"` handlers use the
        // typed AdvancementEvent path. This path never emits legacy string guards.
        _ if is_event_context
            || flat_attr.dispatch.as_ref().map(|s| s.value()).as_deref() == Some("advancement") =>
        {
            let trigger_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_trigger", fn_name),
                proc_macro2::Span::call_site(),
            );
            let revoke_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_revoke", fn_name),
                proc_macro2::Span::call_site(),
            );

            let guard_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_guard", fn_name),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #trigger_ident() -> ::sand_core::AdvancementTrigger {
                    <#dispatch_type_tokens as ::sand_core::event::AdvancementEvent>::trigger().into()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #revoke_ident() -> bool {
                    <#dispatch_type_tokens as ::sand_core::event::AdvancementEvent>::reset().should_revoke()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #guard_ident() -> ::std::option::Option<::sand_core::condition::Condition> {
                    <#dispatch_type_tokens as ::sand_core::event::AdvancementEvent>::guard()
                }

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::Advancement {
                        make_trigger: #trigger_ident,
                        revoke: #revoke_ident,
                        guard: ::std::option::Option::Some(#guard_ident),
                    },
                });

                // Register event type → handler path mapping for EventHandle<E>.revoke/grant.
                ::sand_core::inventory::submit!(::sand_core::EventPathEntry {
                    type_id: ::std::any::TypeId::of::<#dispatch_type_tokens>(),
                    path: #fn_name_str,
                });
            }
        }

        // Unknown type — must implement SandEvent.
        _ => {
            let trigger_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_trigger", fn_name),
                proc_macro2::Span::call_site(),
            );
            let cond_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_condition", fn_name),
                proc_macro2::Span::call_site(),
            );
            let tick_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_tick", fn_name),
                proc_macro2::Span::call_site(),
            );
            let chain_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_chain", fn_name),
                proc_macro2::Span::call_site(),
            );
            let revoke_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_revoke", fn_name),
                proc_macro2::Span::call_site(),
            );
            let type_id_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_type_id", fn_name),
                proc_macro2::Span::call_site(),
            );
            let type_name_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_type_name", fn_name),
                proc_macro2::Span::call_site(),
            );
            let setup_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_setup", fn_name),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #trigger_ident() -> ::std::option::Option<::sand_core::AdvancementTrigger> {
                    let dispatch: ::sand_core::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand_core::events::SandEvent>::dispatch().into();
                    match dispatch {
                        ::sand_core::events::SandEventDispatch::AdvancementTrigger(t) => {
                            ::std::option::Option::Some(t)
                        }
                        ::sand_core::events::SandEventDispatch::TickCondition(_)
                        | ::sand_core::events::SandEventDispatch::Tick(_)
                        | ::sand_core::events::SandEventDispatch::Chain(_) => {
                            ::std::option::Option::None
                        }
                    }
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #cond_ident() -> ::std::option::Option<::std::string::String> {
                    let dispatch: ::sand_core::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand_core::events::SandEvent>::dispatch().into();
                    match dispatch {
                        ::sand_core::events::SandEventDispatch::TickCondition(s) => {
                            ::std::option::Option::Some(s)
                        }
                        ::sand_core::events::SandEventDispatch::AdvancementTrigger(_)
                        | ::sand_core::events::SandEventDispatch::Tick(_)
                        | ::sand_core::events::SandEventDispatch::Chain(_) => {
                            ::std::option::Option::None
                        }
                    }
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #tick_ident() -> ::std::option::Option<::sand_core::events::TickEventDispatch> {
                    let dispatch: ::sand_core::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand_core::events::SandEvent>::dispatch().into();
                    match dispatch {
                        ::sand_core::events::SandEventDispatch::Tick(t) => {
                            ::std::option::Option::Some(t)
                        }
                        ::sand_core::events::SandEventDispatch::AdvancementTrigger(_)
                        | ::sand_core::events::SandEventDispatch::TickCondition(_)
                        | ::sand_core::events::SandEventDispatch::Chain(_) => {
                            ::std::option::Option::None
                        }
                    }
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #chain_ident() -> ::std::option::Option<::sand_core::events::ChainEventDispatch> {
                    let dispatch: ::sand_core::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand_core::events::SandEvent>::dispatch().into();
                    match dispatch {
                        ::sand_core::events::SandEventDispatch::Chain(c) => {
                            ::std::option::Option::Some(c)
                        }
                        ::sand_core::events::SandEventDispatch::AdvancementTrigger(_)
                        | ::sand_core::events::SandEventDispatch::TickCondition(_)
                        | ::sand_core::events::SandEventDispatch::Tick(_) => {
                            ::std::option::Option::None
                        }
                    }
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #revoke_ident() -> bool {
                    <#dispatch_type_tokens as ::sand_core::events::SandEvent>::revoke()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #type_id_ident() -> ::std::any::TypeId {
                    ::std::any::TypeId::of::<#dispatch_type_tokens>()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #type_name_ident() -> &'static str {
                    ::std::any::type_name::<#dispatch_type_tokens>()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #setup_ident() -> ::sand_core::events::EventSetup {
                    <#dispatch_type_tokens as ::sand_core::events::SandEvent>::setup()
                }

                ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand_core::EventDispatch::Custom {
                        make_trigger: #trigger_ident,
                        make_condition: #cond_ident,
                        make_tick: #tick_ident,
                        make_chain: #chain_ident,
                        revoke: #revoke_ident,
                        event_type_id: #type_id_ident,
                        event_type_name: #type_name_ident,
                        make_setup: #setup_ident,
                    },
                });
            }
        }
    };

    Ok(dispatch_tokens)
}

// ── run_fn! ───────────────────────────────────────────────────────────────────

/// Returns a `cmd::function(...)` call and optionally registers an inline body
/// as a named `.mcfunction` file.
///
/// # Named with body — define + call inline
///
/// The body is registered as a named datapack function and the macro expands
/// to the `cmd::function(...)` call in one step:
///
/// ```rust,ignore
/// use sand_macros::{function, run_fn};
/// use sand_core::prelude::*;
///
/// static VISITS: ScoreVar<i32> = ScoreVar::new("visits");
///
/// #[function]
/// fn my_fn() {
///     Execute::new()
///         .as_(Selector::all_players())
///         .run(run_fn!("hello_world:greet" {
///             cmd::say("Welcome!");
///             VISITS.add(Selector::self_(), 1);
///         }));
/// }
/// ```
///
/// # Anonymous with body — one-off inline function
///
/// When no name is given, the namespace is read from `sand.toml` and a unique
/// function name is generated automatically. Perfect for one-off inline
/// functions that don't need to be referenced elsewhere:
///
/// ```rust,ignore
/// Execute::new()
///     .as_(Selector::all_players())
///     .run(run_fn!({
///         cmd::say("One-off greeting!");
///     }));
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
    /// `None` when the user writes `run_fn! { … }` (anonymous).
    name: Option<LitStr>,
    body: Option<syn::Block>,
}

impl syn::parse::Parse for RunFnInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // If the first token is a string literal → named form.
        // If the first token is `{` → anonymous form.
        if input.peek(LitStr) {
            let name: LitStr = input.parse()?;
            let body = if input.peek(token::Brace) {
                Some(input.parse::<syn::Block>()?)
            } else {
                None
            };
            Ok(RunFnInput {
                name: Some(name),
                body,
            })
        } else if input.peek(token::Brace) {
            let body: syn::Block = input.parse()?;
            Ok(RunFnInput {
                name: None,
                body: Some(body),
            })
        } else {
            Err(input.error("expected a string literal (e.g. \"ns:path\") or a block { … }"))
        }
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
/// | `texture` | `&str` or `create!(…)` | PNG path **or** programmatic color spec |
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
/// # `create!(…)` — programmatic texture
///
/// | `create!` field | Type | Required | Description |
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
///     texture: create!(color: 0x00000080),
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
        // Positive integer literal: 14
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(n),
            ..
        })) => Ok(quote! { #n }),
        // Negative integer literal: -10
        // In Rust's syntax tree `-10` is UnaryOp(Neg, Lit::Int(10)), not Lit::Int(-10).
        Some(syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        })) if matches!(
            expr.as_ref(),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(_),
                ..
            })
        ) =>
        {
            Ok(quote! { -(#expr) })
        }
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
#[allow(dead_code)]
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
        let frame_width_num = match gen_fields.get("frame_width") {
            Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            })) => n.base10_parse::<u32>().unwrap_or(0),
            _ => 0u32,
        };
        let height_num = match fields.get("height") {
            Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            })) => n.base10_parse::<u32>().unwrap_or(0),
            _ => 0u32,
        };
        // effective_frame_width: explicit value or 2 × height (default pill ratio).
        let effective_fw = if frame_width_num == 0 {
            height_num * 2
        } else {
            frame_width_num
        };
        let frame_width_ts = proc_macro2::Literal::u32_suffixed(frame_width_num);
        let effective_fw_lit = proc_macro2::Literal::u32_suffixed(effective_fw);

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
                name:        #name,
                steps:       #steps,
                font:        #font_ts,
                frame_width: #effective_fw_lit,
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
                name:        #name,
                steps:       #steps,
                font:        #font_ts,
                frame_width: 0u32,  // unknown for user-supplied PNGs
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
            if mac_name != "create" {
                return Err(syn::Error::new_spanned(
                    &m.mac,
                    "expected `create!(color: …)` or a string literal for `texture`",
                ));
            }
            m.mac.tokens.clone()
        } else {
            unreachable!()
        };

        let gen_fields = parse_kv_fields(gen_tokens)?;
        let color = require_lit_int(&gen_fields, "color", "create")?;
        let width_num = match gen_fields.get("width") {
            Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            })) => n.base10_parse::<u32>().unwrap_or(0),
            _ => 0u32,
        };
        let elem_height_num = match fields.get("height") {
            Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            })) => n.base10_parse::<u32>().unwrap_or(0),
            _ => 0u32,
        };
        // effective_char_width: explicit or height (square default).
        let effective_cw = if width_num == 0 {
            elem_height_num
        } else {
            width_num
        };
        let width_ts = proc_macro2::Literal::u32_suffixed(width_num);
        let effective_cw_lit = proc_macro2::Literal::u32_suffixed(effective_cw);

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
                name:       #name,
                font:       #font_ts,
                char_width: #effective_cw_lit,
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
                name:       #name,
                font:       #font_ts,
                char_width: 0u32,  // unknown for user-supplied PNGs
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

// ── armor_event ───────────────────────────────────────────────────────────────

struct ArmorEventAttr {
    kind_ident: syn::Ident,
    slot_ident: syn::Ident,
    /// String literal or path expression (e.g. `MyItem::BASE`).
    item: Option<syn::Expr>,
    /// String literal or path expression (e.g. `MyItem::CUSTOM_DATA_SNBT`).
    custom_data: Option<syn::Expr>,
}

impl syn::parse::Parse for ArmorEventAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // 1. Parse kind ident (Equip or Unequip)
        let kind_ident: syn::Ident = input.parse()?;
        let kind_str = kind_ident.to_string();
        if kind_str != "Equip" && kind_str != "Unequip" {
            return Err(syn::Error::new_spanned(
                &kind_ident,
                format!("expected `Equip` or `Unequip`, got `{kind_str}`"),
            ));
        }

        // 2. Expect `,`
        let _comma: token::Comma = input.parse()?;

        // 3. Parse `slot = Ident`
        let slot_key: syn::Ident = input.parse()?;
        if slot_key != "slot" {
            return Err(syn::Error::new_spanned(
                &slot_key,
                "expected `slot = <Slot>` after event kind",
            ));
        }
        let _eq: token::Eq = input.parse()?;
        let slot_ident: syn::Ident = input.parse()?;
        let slot_str = slot_ident.to_string();
        if !matches!(
            slot_str.as_str(),
            "Head" | "Chest" | "Legs" | "Feet" | "Offhand"
        ) {
            return Err(syn::Error::new_spanned(
                &slot_ident,
                format!(
                    "expected one of `Head`, `Chest`, `Legs`, `Feet`, `Offhand`, got `{slot_str}`"
                ),
            ));
        }

        // 4. Parse optional key = value pairs
        let mut item: Option<syn::Expr> = None;
        let mut custom_data: Option<syn::Expr> = None;

        while input.peek(token::Comma) {
            let _comma: token::Comma = input.parse()?;
            if input.is_empty() {
                break;
            }
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            let _eq: token::Eq = input.parse()?;
            match key_str.as_str() {
                "item" => {
                    item = Some(input.parse::<syn::Expr>()?);
                }
                "custom_data" => {
                    // Accept either a string literal ("key") or a path expression
                    // (e.g. MyItem::CUSTOM_DATA_KEY).
                    custom_data = Some(input.parse::<syn::Expr>()?);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("unknown key `{other}`; allowed keys are `item`, `custom_data`"),
                    ));
                }
            }
        }

        Ok(ArmorEventAttr {
            kind_ident,
            slot_ident,
            item,
            custom_data,
        })
    }
}

/// Registers a function as an armor slot equip/unequip event handler.
///
/// Fires when a player equips or unequips an item from an armor or offhand slot.
/// Uses a tick-based NBT check — no advancement required.
///
/// # Syntax
///
/// ```rust,ignore
/// #[armor_event(Equip, slot = Feet)]
/// #[armor_event(Equip, slot = Feet, item = "minecraft:leather_boots")]
/// #[armor_event(Unequip, slot = Head, item = "minecraft:diamond_helmet")]
/// #[armor_event(Equip, slot = Feet, item = "minecraft:diamond_sword", custom_data = "{mana_boots:true}")]
/// ```
///
/// ## Slots
///
/// | Slot | Covers |
/// |---|---|
/// | `Head` | Helmet slot |
/// | `Chest` | Chestplate slot |
/// | `Legs` | Leggings slot |
/// | `Feet` | Boots slot |
/// | `Offhand` | Offhand slot |
///
/// ## Item filter
///
/// Omit `item` to match any item in the slot. Add `custom_data` to match
/// a specific `minecraft:custom_data` component tag (SNBT format):
///
/// ```rust,ignore
/// static MANA_REGEN_BOOST: Flag = Flag::new("mana_regen_boost");
///
/// // Fire when any custom "mana boots" item is equipped in the feet slot
/// #[armor_event(Equip, slot = Feet, item = "minecraft:leather_boots",
///               custom_data = "{mana_boots:true}")]
/// pub fn on_mana_boots_equip() {
///     MANA_REGEN_BOOST.enable(Selector::self_());
/// }
///
/// #[armor_event(Unequip, slot = Feet, item = "minecraft:leather_boots",
///               custom_data = "{mana_boots:true}")]
/// pub fn on_mana_boots_unequip() {
///     MANA_REGEN_BOOST.disable(Selector::self_());
/// }
/// ```
///
/// ## How it works
///
/// All `#[armor_event]` functions are combined into a single
/// `__sand_armor_check` mcfunction registered to `minecraft:tick`.
/// Each watch uses a scoreboard tag (`__armor_*`) to track previous state
/// and detect equip/unequip transitions.
#[proc_macro_attribute]
pub fn armor_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_attr = parse_macro_input!(attr as ArmorEventAttr);
    let func = parse_macro_input!(item as ItemFn);

    match expand_armor_event(parsed_attr, func) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_armor_event(attr: ArmorEventAttr, func: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &func.vis;
    let attrs = &func.attrs;

    // Validate: no parameters.
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[armor_event] functions must take no parameters",
        ));
    }

    let factory_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{}_make", fn_name),
        proc_macro2::Span::call_site(),
    );

    let body = build_cmd_body(&func.block)?;

    // Map slot ident to ::sand_core::ArmorSlot::*
    let slot_ident = &attr.slot_ident;
    let slot_expr = quote! { ::sand_core::ArmorSlot::#slot_ident };

    // Map kind ident to ::sand_core::ArmorEventKind::*
    let kind_ident = &attr.kind_ident;
    let kind_expr = quote! { ::sand_core::ArmorEventKind::#kind_ident };

    // item_id: Option<&'static str>
    let item_id_expr = match &attr.item {
        Some(lit) => quote! { ::std::option::Option::Some(#lit) },
        None => quote! { ::std::option::Option::None },
    };

    // custom_data_snbt: Option<&'static str>
    let custom_data_expr = match &attr.custom_data {
        Some(lit) => quote! { ::std::option::Option::Some(#lit) },
        None => quote! { ::std::option::Option::None },
    };

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

        ::sand_core::inventory::submit!(::sand_core::ArmorEventDescriptor {
            path: #fn_name_str,
            make: #factory_ident,
            slot: #slot_expr,
            kind: #kind_expr,
            item_id: #item_id_expr,
            custom_data_snbt: #custom_data_expr,
        });
    })
}

// ── run_fn! ───────────────────────────────────────────────────────────────────

/// Read the `[pack].namespace` value from `sand.toml` next to `CARGO_MANIFEST_DIR`.
fn read_sand_namespace() -> Option<String> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let path = std::path::Path::new(&dir).join("sand.toml");
    let content = std::fs::read_to_string(path).ok()?;
    // Simple parse: find `namespace` key under `[pack]`.
    let mut in_pack = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_pack = trimmed == "[pack]";
            continue;
        }
        if in_pack && let Some(rest) = trimmed.strip_prefix("namespace") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let val = rest.trim().trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

/// Global counter for generating unique anonymous function names.
static ANON_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn expand_run_fn(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let RunFnInput { name, body } = syn::parse::<RunFnInput>(input)?;

    // Resolve the full resource location string (e.g. "ns:path").
    let (name_val, span) = match &name {
        Some(lit) => (lit.value(), lit.span()),
        None => {
            // Anonymous: read namespace from sand.toml, generate unique path.
            let ns = read_sand_namespace().ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "could not read [pack].namespace from sand.toml; \
                     provide an explicit name or ensure sand.toml exists",
                )
            })?;
            let id = ANON_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let anon_path = format!("{ns}:__anon/fn_{id}");
            (anon_path, proc_macro2::Span::call_site())
        }
    };

    // Extract the path part (after ":") for the FunctionDescriptor path.
    let path_part = match name_val.find(':') {
        Some(i) => &name_val[i + 1..],
        None => &name_val[..],
    };

    let name_lit = LitStr::new(&name_val, span);
    let fn_call = quote! {
        ::sand_core::cmd::function(
            #name_lit.parse::<::sand_core::ResourceLocation>().unwrap()
        )
    };

    if let Some(block) = body {
        let path_lit = LitStr::new(path_part, span);
        let cmd_body = build_cmd_body(&block)?;

        if name.is_some() {
            // Named run_fn!("ns:path" { ... }) — no captures expected; use inventory.
            let mangled = path_part.replace(['/', ':'], "_");
            let fn_ident = proc_macro2::Ident::new(
                &format!("__sand_run_fn_{mangled}"),
                proc_macro2::Span::call_site(),
            );
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
            // Anonymous run_fn!({ ... }) — body is evaluated immediately so local
            // variable captures work. Registered via runtime registry instead of
            // inventory, so the component builder picks it up after user fns run.
            Ok(quote! {
                {
                    ::sand_core::register_dyn_fn(
                        #path_lit.to_string(),
                        { #cmd_body },
                    );

                    #fn_call
                }
            })
        }
    } else {
        Ok(fn_call)
    }
}

// ── #[schedule] ───────────────────────────────────────────────────────────────

/// Defines a scheduled function that runs for a fixed number of ticks.
///
/// The body is called repeatedly while the schedule is active. Start and stop
/// the schedule at runtime by calling the generated companion functions:
///
/// | Function | Effect |
/// |---|---|
/// | `<name>_start` | Start/restart the schedule for `@s` |
/// | `<name>_stop` | Cancel the schedule for `@s` |
///
/// # Parameters
/// - `ticks` (**required**) — total duration in ticks (e.g. `60` = 3 seconds).
/// - `every` *(optional, default `1`)* — execute body every N ticks.
///   `every = 1` fires on every tick; `every = 3` fires on ticks 1, 4, 7, …
///
/// # Example
/// ```rust,ignore
/// use sand_macros::schedule;
/// use sand_core::{cmd::*, mcfunction};
///
/// /// Flame aura: runs every tick for 3 seconds.
/// #[schedule(ticks = 60)]
/// pub fn flame_aura() {
///     mcfunction! {
///         for cmd in &ParticleBuilder::new(Particle::named("minecraft:flame"))
///             .circle(1.5, 1.0, 24) { cmd; }
///     }
/// }
///
/// /// Pulse effect: runs every 5 ticks for 4 seconds.
/// #[schedule(ticks = 80, every = 5)]
/// pub fn pulse_effect() {
///     mcfunction! {
///         for cmd in &ParticleBuilder::new(Particle::dust_hex(0xFF4400, 1.5))
///             .sphere(2.0, 1.0, 48) { cmd; }
///     }
/// }
///
/// // Trigger from another function:
/// // cmd::function("mypack:flame_aura_start".parse().unwrap())
/// // cmd::function("mypack:flame_aura_stop".parse().unwrap())
/// ```
#[proc_macro_attribute]
pub fn schedule(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    match parse_schedule_attr(attr).and_then(|sa| expand_schedule(func, sa)) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct ScheduleAttr {
    ticks: u32,
    every: u32,
}

fn parse_schedule_attr(attr: TokenStream) -> syn::Result<ScheduleAttr> {
    struct Parsed {
        ticks: u32,
        every: u32,
    }

    impl syn::parse::Parse for Parsed {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut ticks: Option<u32> = None;
            let mut every: u32 = 1;

            while !input.is_empty() {
                let key: syn::Ident = input.parse()?;
                let _eq: syn::Token![=] = input.parse()?;
                let val: syn::LitInt = input.parse()?;
                match key.to_string().as_str() {
                    "ticks" => ticks = Some(val.base10_parse()?),
                    "every" => every = val.base10_parse()?,
                    other => {
                        return Err(syn::Error::new_spanned(
                            &key,
                            format!("unknown parameter `{other}`; expected `ticks` or `every`"),
                        ));
                    }
                }
                if input.peek(syn::Token![,]) {
                    let _: syn::Token![,] = input.parse()?;
                }
            }

            let ticks = ticks.ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "#[schedule] requires `ticks = <n>`, e.g. `#[schedule(ticks = 60)]`",
                )
            })?;

            Ok(Parsed { ticks, every })
        }
    }

    let parsed = syn::parse::<Parsed>(attr)?;
    Ok(ScheduleAttr {
        ticks: parsed.ticks,
        every: parsed.every.max(1),
    })
}

fn expand_schedule(func: ItemFn, attr: ScheduleAttr) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &func.vis;
    let attrs = &func.attrs;

    if let Some(recv) = func.sig.inputs.iter().find_map(|a| {
        if let syn::FnArg::Receiver(r) = a {
            Some(r)
        } else {
            None
        }
    }) {
        return Err(syn::Error::new_spanned(
            recv,
            "#[schedule] cannot be applied to methods — use a free-standing `fn`",
        ));
    }
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[schedule] functions must take no parameters",
        ));
    }

    let fn_make_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{fn_name}_sched_make"),
        proc_macro2::Span::call_site(),
    );

    let body = build_cmd_body(&func.block)?;
    let total_ticks = attr.ticks;
    let every = attr.every;

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

        ::sand_core::inventory::submit!(::sand_core::ScheduleDescriptor {
            path: #fn_name_str,
            total_ticks: #total_ticks,
            every: #every,
            make: #fn_make_ident,
        });
    })
}

// ── #[item] ───────────────────────────────────────────────────────────────────

/// Generate a typed item struct from a `CustomItem`-producing function.
///
/// Reads `CustomItem::new("base_id")` and `.custom_data("key")` directly from
/// the function body — no duplication needed. Generates a unit struct with
/// `BASE`, `PREDICATE`, and an `item()` method that calls the original function.
///
/// The struct name is derived automatically from the `custom_data` key
/// (converted to PascalCase). Override it with `#[item(name = "MyName")]`.
/// If there is no `custom_data` call, `name` is required.
///
/// # Examples
///
/// ```rust,ignore
/// // Struct name "ManaBoots" derived from custom_data key "mana_boots"
/// #[item]
/// pub fn mana_boots() -> CustomItem {
///     CustomItem::new("minecraft:leather_boots")
///         .custom_data("mana_boots")
///         .display_name("Mana Boots")
/// }
///
/// // No custom_data — must provide name
/// #[item(name = "ShardBlade")]
/// pub fn shard_blade() -> CustomItem {
///     CustomItem::new("minecraft:diamond_sword")
///         .display_name("Shard Blade")
/// }
/// ```
///
/// Generated:
/// ```rust,ignore
/// pub struct ManaBoots;
/// impl ManaBoots {
///     pub const BASE: &'static str = "minecraft:leather_boots";
///     pub const PREDICATE: &'static str =
///         "minecraft:leather_boots[custom_data={mana_boots:1b}]";
///     pub const CUSTOM_DATA_KEY: &'static str = "mana_boots";
///     pub fn item() -> CustomItem { mana_boots() }
/// }
/// ```
///
/// Usage:
/// ```rust,ignore
/// Execute::new()
///     .as_(Selector::all_players())
///     .at(Selector::self_())
///     .if_items_entity(Selector::self_(), ItemSlot::Feet, ManaBoots::PREDICATE)
///     .run_fn("ns:on_mana_boots_tick");
/// ```
#[proc_macro_attribute]
pub fn item(attr: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    match expand_item(attr, func) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Convert `snake_case` or `kebab-case` to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|seg| !seg.is_empty())
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Recursively walk a syn `Expr` looking for:
/// - `CustomItem::new("<base>")` → returns the base string
/// - `.custom_data("<key>")` → returns the custom_data key
fn item_walk_expr(expr: &syn::Expr, base: &mut Option<String>, cd: &mut Option<String>) {
    match expr {
        syn::Expr::Call(c) => {
            // CustomItem::new("...") or new("...")
            if let syn::Expr::Path(p) = &*c.func {
                let last = p.path.segments.last().map(|s| s.ident.to_string());
                let has_custom_item = p.path.segments.iter().any(|s| s.ident == "CustomItem");
                if last.as_deref() == Some("new")
                    && has_custom_item
                    && let Some(syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    })) = c.args.first()
                {
                    *base = Some(s.value());
                }
            }
            item_walk_expr(&c.func, base, cd);
            for arg in &c.args {
                item_walk_expr(arg, base, cd);
            }
        }
        syn::Expr::MethodCall(mc) => {
            if mc.method == "custom_data"
                && let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = mc.args.first()
            {
                *cd = Some(s.value());
            }
            item_walk_expr(&mc.receiver, base, cd);
            for arg in &mc.args {
                item_walk_expr(arg, base, cd);
            }
        }
        syn::Expr::Block(b) => {
            for stmt in &b.block.stmts {
                item_walk_stmt(stmt, base, cd);
            }
        }
        syn::Expr::Return(r) => {
            if let Some(e) = &r.expr {
                item_walk_expr(e, base, cd);
            }
        }
        _ => {}
    }
}

fn item_walk_stmt(stmt: &syn::Stmt, base: &mut Option<String>, cd: &mut Option<String>) {
    match stmt {
        syn::Stmt::Expr(e, _) => item_walk_expr(e, base, cd),
        syn::Stmt::Local(l) => {
            if let Some(init) = &l.init {
                item_walk_expr(&init.expr, base, cd);
            }
        }
        _ => {}
    }
}

/// A single entry in the `data = [NAME: Type = value]` list.
struct ItemDataConst {
    name: proc_macro2::Ident,
    ty: syn::Type,
    value: syn::Expr,
}

/// Parse the attr tokens for `#[item(...)]`.
/// Accepts: `name = "..."` and/or `data = [IDENT: Type = expr, ...]`
struct ItemAttr {
    name: Option<String>,
    data: Vec<ItemDataConst>,
}

impl syn::parse::Parse for ItemAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut data: Vec<ItemDataConst> = Vec::new();

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;

            match key.to_string().as_str() {
                "name" => {
                    let val: LitStr = input.parse()?;
                    name = Some(val.value());
                }
                "data" => {
                    // Parse `[ IDENT: Type = Expr, ... ]`
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        let const_name: proc_macro2::Ident = content.parse()?;
                        let _colon: syn::Token![:] = content.parse()?;
                        let ty: syn::Type = content.parse()?;
                        let _eq2: syn::Token![=] = content.parse()?;
                        let value: syn::Expr = content.parse()?;
                        data.push(ItemDataConst {
                            name: const_name,
                            ty,
                            value,
                        });
                        if content.peek(syn::Token![,]) {
                            let _: syn::Token![,] = content.parse()?;
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!(
                            "unknown #[item] parameter `{other}`; \
                             expected `name = \"...\"` or \
                             `data = [CONST: Type = value, ...]`"
                        ),
                    ));
                }
            }

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(ItemAttr { name, data })
    }
}

fn expand_item(attr: TokenStream, func: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // ── Parse attr ────────────────────────────────────────────────────────────
    let item_attr = if attr.is_empty() {
        ItemAttr {
            name: None,
            data: vec![],
        }
    } else {
        syn::parse::<ItemAttr>(attr)?
    };

    // ── Extract base and custom_data from function body ───────────────────────
    let mut base: Option<String> = None;
    let mut custom_data: Option<String> = None;
    for stmt in &func.block.stmts {
        item_walk_stmt(stmt, &mut base, &mut custom_data);
    }

    let base = base.ok_or_else(|| {
        syn::Error::new_spanned(
            &func.sig,
            "#[item] could not find `CustomItem::new(\"minecraft:...\")` in the function body. \
             Make sure the base item ID is a string literal passed directly to `CustomItem::new`.",
        )
    })?;

    // ── Determine struct name ─────────────────────────────────────────────────
    let struct_name_str = if let Some(n) = item_attr.name {
        n
    } else if let Some(ref cd) = custom_data {
        to_pascal_case(cd)
    } else {
        return Err(syn::Error::new_spanned(
            &func.sig,
            "#[item] could not find a `.custom_data(\"key\")` call to derive the struct name. \
             Either add `.custom_data(\"your_key\")` to uniquely identify this item, or \
             specify an explicit name with `#[item(name = \"YourName\")]`.",
        ));
    };

    // ── Build constants ───────────────────────────────────────────────────────
    let struct_ident = proc_macro2::Ident::new(&struct_name_str, proc_macro2::Span::call_site());
    let fn_ident = &func.sig.ident;
    let vis = &func.vis;
    let fn_attrs = &func.attrs;

    let predicate_lit = match &custom_data {
        // 1.21.2+: use ~ (partial/contains match); full namespace required in commands.
        Some(key) => format!("{base}[minecraft:custom_data~{{{key}:1b}}]"),
        None => base.clone(),
    };

    let custom_data_const = if let Some(ref key) = custom_data {
        let snbt = format!("{{{key}:1b}}");
        quote! {
            /// The raw `custom_data` key (e.g. `"mana_boots"`).
            pub const CUSTOM_DATA_KEY: &'static str = #key;

            /// SNBT form of the `custom_data` tag (e.g. `"{mana_boots:1b}"`).
            ///
            /// Use this with `#[armor_event(..., custom_data = MyItem::CUSTOM_DATA_SNBT)]`.
            pub const CUSTOM_DATA_SNBT: &'static str = #snbt;
        }
    } else {
        quote! {}
    };

    // ── User-defined data consts ──────────────────────────────────────────────
    let data_consts = item_attr.data.iter().map(|c| {
        let const_name = &c.name;
        let ty = &c.ty;
        let val = &c.value;
        quote! { pub const #const_name: #ty = #val; }
    });

    Ok(quote! {
        #(#fn_attrs)*
        #func

        /// Auto-generated item reference type produced by `#[item]`.
        ///
        /// Use [`PREDICATE`](Self::PREDICATE) with
        /// [`Execute::if_items_entity`] to detect this item in any slot, and
        /// [`item()`](Self::item) to obtain the [`CustomItem`] definition.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #vis struct #struct_ident;

        impl #struct_ident {
            /// The base Minecraft item ID (e.g. `"minecraft:leather_boots"`).
            pub const BASE: &'static str = #base;

            /// Full item predicate for `execute if items`.
            ///
            /// Includes the `custom_data` component when set, making this
            /// predicate uniquely identify this item.
            pub const PREDICATE: &'static str = #predicate_lit;

            #custom_data_const

            #(#data_consts)*

            /// Returns an `execute if items entity @s <slot> <predicate> run <cmd>` command
            /// that runs `cmd` only when `@s` has this item equipped in `slot`.
            ///
            /// # Example
            /// ```rust,ignore
            /// mcfunction! {
            ///     ManaBoots::if_wearing(ItemSlot::Feet, run_fn! { … });
            /// }
            /// ```
            pub fn if_wearing(
                slot: ::sand_core::cmd::ItemSlot,
                cmd: impl ::std::fmt::Display,
            ) -> ::std::string::String {
                ::std::format!(
                    "execute if items entity @s {slot} {} run {cmd}",
                    Self::PREDICATE,
                )
            }

            /// Returns an `execute unless items entity @s <slot> <predicate> run <cmd>` command
            /// that runs `cmd` only when `@s` does NOT have this item in `slot`.
            pub fn unless_wearing(
                slot: ::sand_core::cmd::ItemSlot,
                cmd: impl ::std::fmt::Display,
            ) -> ::std::string::String {
                ::std::format!(
                    "execute unless items entity @s {slot} {} run {cmd}",
                    Self::PREDICATE,
                )
            }

            /// Construct the [`CustomItem`] definition for this item.
            pub fn item() -> ::sand_core::CustomItem {
                #fn_ident()
            }
        }
    })
}

// ── SandStorage derive macro ──────────────────────────────────────────────────

/// Derive `StorageSchema` and typed field accessors from a Rust struct.
///
/// # Required attribute
///
/// ```rust,ignore
/// #[derive(SandStorage)]
/// #[sand(storage = "namespace:id", root = "nbt.path")]
/// pub struct MySchema {
///     pub field_one: i32,
///     pub field_two: String,
/// }
/// ```
///
/// # Generated API
///
/// ```rust,ignore
/// impl MySchema {
///     pub const SCHEMA: StorageSchema<MySchema> =
///         StorageSchema::new("namespace:id", "nbt.path");
///
///     pub fn field_one() -> StorageField<MySchema, i32> {
///         Self::SCHEMA.field("field_one")
///     }
///
///     pub fn field_two() -> StorageField<MySchema, String> {
///         Self::SCHEMA.field("field_two")
///     }
/// }
/// ```
///
/// # Custom field paths
///
/// ```rust,ignore
/// #[sand(path = "alternate.key")]
/// pub school: String,
/// ```
///
/// # Restrictions
///
/// - Named structs only; tuple structs are rejected at compile time.
/// - The `#[sand(storage = ..., root = ...)]` attribute is required.
#[proc_macro_derive(SandStorage, attributes(sand))]
pub fn sand_storage_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match sand_storage_derive_impl(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn sand_storage_derive_impl(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use proc_macro2::Span;
    use quote::quote;
    use syn::{Data, Fields, Lit, Meta};

    let struct_name = &input.ident;

    // ── Extract #[sand(storage = "...", root = "...")] from the struct ────────
    let mut storage_val: Option<String> = None;
    let mut root_val: Option<String> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("sand") {
            continue;
        }
        let nested = attr
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .map_err(|e| syn::Error::new_spanned(attr, format!("#[sand] parse error: {e}")))?;

        for meta in nested {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("storage") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                    {
                        storage_val = Some(s.value());
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("root") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                    {
                        root_val = Some(s.value());
                    }
                }
                _ => {}
            }
        }
    }

    let storage = storage_val.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[derive(SandStorage)] requires #[sand(storage = \"namespace:id\", root = \"nbt.path\")]",
        )
    })?;
    let root = root_val.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[derive(SandStorage)] requires #[sand(root = \"nbt.path\")] on the struct",
        )
    })?;

    // ── Validate named struct ─────────────────────────────────────────────────
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "#[derive(SandStorage)] does not support tuple structs; use named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "#[derive(SandStorage)] requires at least one named field",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "#[derive(SandStorage)] can only be applied to structs",
            ));
        }
    };

    // ── Build field accessor methods ─────────────────────────────────────────
    let mut methods = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().expect("named field has ident");
        let field_ty = &field.ty;

        // Check for #[sand(path = "...")] override
        let mut path_override: Option<String> = None;
        for attr in &field.attrs {
            if !attr.path().is_ident("sand") {
                continue;
            }
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            ) {
                for meta in nested {
                    if let Meta::NameValue(nv) = meta
                        && nv.path.is_ident("path")
                        && let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = &nv.value
                    {
                        path_override = Some(s.value());
                    }
                }
            }
        }

        let field_name_str = field_ident.to_string();
        let key_str: &str = path_override.as_deref().unwrap_or(field_name_str.as_str());

        methods.push(quote! {
            pub fn #field_ident() -> ::sand_core::state::StorageField<#struct_name, #field_ty> {
                Self::SCHEMA.field(#key_str)
            }
        });
    }

    let storage_lit = storage.as_str();
    let root_lit = root.as_str();

    let expanded = quote! {
        impl #struct_name {
            pub const SCHEMA: ::sand_core::state::StorageSchema<#struct_name> =
                ::sand_core::state::StorageSchema::new(#storage_lit, #root_lit);

            #( #methods )*
        }
    };

    Ok(expanded.into())
}
