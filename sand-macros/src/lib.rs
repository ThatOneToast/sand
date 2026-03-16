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

// ── #[event] ─────────────────────────────────────────────────────────────────

/// Turns a function into a Minecraft advancement-driven event handler.
///
/// Registers the function as a `.mcfunction` file **and** auto-generates an
/// `Advancement` that calls it as its reward — no manual advancement setup
/// needed.
///
/// # Options
///
/// ```rust,ignore
/// #[event(EventType)]                                  // basic
/// #[event(EventType, revoke = true)]                   // re-arm on next trigger
/// #[event(EventType, id = "my_pack:custom/id")]        // override advancement ID
/// #[event(EventType { field = "value" })]              // with filter
/// #[event(EventType { f1 = "v1", f2 = "v2" }, revoke = true)]
/// ```
///
/// ## `revoke = true`
///
/// Prepends `advancement revoke @s only <id>` as the first command in the
/// generated function, so the advancement can fire again the next time the
/// trigger fires. Without this, the advancement grants only once per player
/// (until manually revoked).
///
/// **Join detection pattern** — use `revoke = false` (default) and add this to
/// your `#[component(Load)]` to reset for every world load/reload:
/// ```rust,ignore
/// "advancement revoke @a only my_pack:on_join";
/// ```
///
/// # Event types
///
/// ---
///
/// ## `Join` — player enters the world
///
/// Uses `minecraft:tick`. Fires once per player session (until revoked). The
/// standard pattern for initialization logic.
///
/// ```rust,ignore
/// #[event(Join)]
/// pub fn on_join() {
///     "scoreboard players set @s player_mana 100";
///     "say Welcome!";
/// }
/// ```
///
/// ---
///
/// ## `Death` — a player is killed
///
/// Fires when an entity kills the player. Optional filters:
/// - `entity = "minecraft:zombie"` — only when killed by this entity type
/// - `killing_blow = "minecraft:arrow"` — only when killed by this damage type
///
/// ```rust,ignore
/// // Any death
/// #[event(Death, revoke = true)]
/// pub fn on_death() { "say A player died!"; }
///
/// // Only zombie kills
/// #[event(Death { entity = "minecraft:zombie" }, revoke = true)]
/// pub fn on_zombie_death() { "give @s minecraft:apple 1"; }
///
/// // Complex predicate — pass a raw JSON object (must start with `{`)
/// #[event(Death { killing_blow = r#"{"direct_entity":{"type":"minecraft:arrow"}}"# })]
/// pub fn on_arrow_death() { }
/// ```
///
/// ---
///
/// ## `Kill` — player kills an entity
///
/// Fires when the player kills something. Same filters as `Death`.
///
/// ```rust,ignore
/// #[event(Kill { entity = "minecraft:skeleton" }, revoke = true)]
/// pub fn on_skeleton_kill() { "give @s minecraft:bone 1"; }
/// ```
///
/// ---
///
/// ## `BlockPlaced` — player places a block
///
/// Fires when a block is placed. Optional filters:
/// - `block = "minecraft:diamond_ore"` — specific block ID
/// - `item = "minecraft:diamond_ore"` — item used to place (pass raw JSON for predicates)
///
/// ```rust,ignore
/// #[event(BlockPlaced { block = "minecraft:tnt" }, revoke = true)]
/// pub fn on_tnt_placed() { "say TNT placed!"; }
/// ```
///
/// ---
///
/// ## `ItemUsed` — player finishes using an item (right-click complete)
///
/// Fires when an item use action completes (e.g. eating, drinking, shooting).
/// Optional filter: `item = "minecraft:bow"`.
///
/// ```rust,ignore
/// #[event(ItemUsed { item = "minecraft:crossbow" }, revoke = true)]
/// pub fn on_crossbow_fire() { }
/// ```
///
/// ---
///
/// ## `ItemConsumed` — player consumes a food or potion
///
/// ```rust,ignore
/// #[event(ItemConsumed { item = "minecraft:golden_apple" }, revoke = true)]
/// pub fn on_golden_apple() { "say Golden apple consumed!"; }
/// ```
///
/// ---
///
/// ## `UsingItem` — player is actively holding right-click on an item
///
/// Fires every tick the player is mid-use (e.g. drawing a bow).
///
/// ```rust,ignore
/// #[event(UsingItem { item = "minecraft:bow" })]
/// pub fn on_bow_draw() { }
/// ```
///
/// ---
///
/// ## `RecipeUnlocked` — player unlocks a recipe
///
/// `recipe` is required and must be a full resource location.
///
/// ```rust,ignore
/// #[event(RecipeUnlocked { recipe = "minecraft:diamond_sword" }, revoke = true)]
/// pub fn on_diamond_sword_recipe() { "give @s minecraft:diamond 1"; }
/// ```
///
/// ---
///
/// ## `EnterBlock` — player steps inside a block
///
/// Optional filter: `block = "minecraft:water"`.
///
/// ```rust,ignore
/// #[event(EnterBlock { block = "minecraft:lava" }, revoke = true)]
/// pub fn on_enter_lava() { "say Lava!"; }
/// ```
///
/// ---
///
/// ## `EnchantedItem` — player enchants an item
///
/// Optional filters: `item`, `levels` (raw JSON predicates).
///
/// ```rust,ignore
/// #[event(EnchantedItem, revoke = true)]
/// pub fn on_enchant() { }
/// ```
///
/// ---
///
/// ## `TamedAnimal` — player tames an animal
///
/// Optional filter: `entity = "minecraft:wolf"`.
///
/// ```rust,ignore
/// #[event(TamedAnimal { entity = "minecraft:wolf" }, revoke = true)]
/// pub fn on_wolf_tamed() { "give @s minecraft:bone 5"; }
/// ```
///
/// ---
///
/// ## `SummonedEntity` — player summons an entity
///
/// ```rust,ignore
/// #[event(SummonedEntity { entity = "minecraft:wither" }, revoke = true)]
/// pub fn on_wither_summon() { "say Wither summoned!"; }
/// ```
///
/// ---
///
/// ## `BredAnimals` — player breeds two animals
///
/// Optional filters: `parent`, `partner`, `child` (entity type strings or JSON predicates).
///
/// ```rust,ignore
/// #[event(BredAnimals { child = "minecraft:cow" }, revoke = true)]
/// pub fn on_breed_cow() { }
/// ```
///
/// ---
///
/// ## `InteractEntity` — player right-clicks an entity
///
/// Optional filters: `entity`, `item`.
///
/// ```rust,ignore
/// #[event(InteractEntity { entity = "minecraft:villager" }, revoke = true)]
/// pub fn on_villager_interact() { }
/// ```
///
/// ---
///
/// ## `Location` — player reaches a location
///
/// Optional filter: `location` — a raw JSON location predicate.
///
/// ```rust,ignore
/// #[event(Location { location = r#"{"biome":"minecraft:desert"}"# }, revoke = true)]
/// pub fn on_desert_enter() { "say You entered the desert!"; }
/// ```
///
/// ---
///
/// ## `NetherTravel` — player travels through the nether
///
/// Optional filters: `entered`, `exited`, `distance` (raw JSON predicates).
///
/// ```rust,ignore
/// #[event(NetherTravel, revoke = true)]
/// pub fn on_nether_travel() { }
/// ```
///
/// ---
///
/// ## `InventoryChanged` — player's inventory contents change
///
/// Optional filter: `items` — a **JSON array string** of item predicates.
/// Each entry is `{"id":"minecraft:..."}` or a richer predicate.
///
/// ```rust,ignore
/// // Fire when player picks up any diamond
/// #[event(InventoryChanged { items = r#"[{"id":"minecraft:diamond"}]"# }, revoke = true)]
/// pub fn on_get_diamond() { "say You got a diamond!"; }
///
/// // No filter — fires on any inventory change
/// #[event(InventoryChanged, revoke = true)]
/// pub fn on_any_inventory_change() { }
/// ```
///
/// ---
///
/// ## `Custom` — any Minecraft advancement trigger by ID
///
/// Use for triggers not covered by the named variants, or version-specific ones.
/// `trigger` is required; `conditions` is an optional raw JSON object string.
///
/// ```rust,ignore
/// // Bare trigger, no conditions
/// #[event(Custom { trigger = "minecraft:tick" })]
/// pub fn on_tick_custom() { }
///
/// // With a JSON conditions object
/// #[event(Custom {
///     trigger = "minecraft:player_interacted_with_entity",
///     conditions = r#"{"player":{"gamemode":"survival"}}"#
/// }, revoke = true)]
/// pub fn on_survival_interact() { }
/// ```
///
/// ---
///
/// ## `Tick` / `Impossible`
///
/// Raw triggers. `Tick` fires every tick (use `Join` instead for join
/// detection). `Impossible` never fires — useful as a placeholder.
///
/// ```rust,ignore
/// #[event(Tick)]
/// pub fn every_tick_once() { }   // fires once, then stays granted
///
/// #[event(Impossible)]
/// pub fn never_fires() { }
/// ```
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    match parse_event_attr(attr).and_then(|ea| expand_event(func, ea)) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── EventAttr parsing ─────────────────────────────────────────────────────────

struct EventAttr {
    event_type: syn::Ident,
    filters: std::collections::HashMap<String, syn::Expr>,
    revoke: bool,
    id_override: Option<syn::LitStr>,
}

fn parse_event_attr(attr: TokenStream) -> syn::Result<EventAttr> {
    // We parse a custom token stream rather than using syn::Meta so that we
    // can handle the mixed `EventType { field = … } , revoke = true` syntax.
    struct ParsedEventAttr {
        event_type: syn::Ident,
        filters: std::collections::HashMap<String, syn::Expr>,
        revoke: bool,
        id_override: Option<syn::LitStr>,
    }

    impl syn::parse::Parse for ParsedEventAttr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            // Parse the event type identifier (e.g. `Join`, `Death`, `Custom`)
            let event_type: syn::Ident = input.parse()?;

            // Optionally parse filter fields inside braces: `{ field = "value", … }`
            let mut filters = std::collections::HashMap::new();
            if input.peek(syn::token::Brace) {
                let brace_content;
                syn::braced!(brace_content in input);
                // Parse comma-separated `key = value` pairs
                loop {
                    if brace_content.is_empty() {
                        break;
                    }
                    let key: syn::Ident = brace_content.parse()?;
                    let _eq: syn::Token![=] = brace_content.parse()?;
                    let val: syn::Expr = brace_content.parse()?;
                    filters.insert(key.to_string(), val);
                    if brace_content.is_empty() {
                        break;
                    }
                    let _comma: syn::Token![,] = brace_content.parse()?;
                }
            }

            // Parse optional trailing `, revoke = true` and/or `, id = "..."` options
            let mut revoke = false;
            let mut id_override: Option<syn::LitStr> = None;

            while input.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = input.parse()?;
                if input.is_empty() {
                    break;
                }
                let key: syn::Ident = input.parse()?;
                let _eq: syn::Token![=] = input.parse()?;
                match key.to_string().as_str() {
                    "revoke" => {
                        let val: syn::LitBool = input.parse()?;
                        revoke = val.value;
                    }
                    "id" => {
                        let val: syn::LitStr = input.parse()?;
                        id_override = Some(val);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            &key,
                            format!("unknown option `{other}`; expected `revoke` or `id`"),
                        ));
                    }
                }
            }

            Ok(ParsedEventAttr {
                event_type,
                filters,
                revoke,
                id_override,
            })
        }
    }

    let parsed = syn::parse::<ParsedEventAttr>(attr)?;
    Ok(EventAttr {
        event_type: parsed.event_type,
        filters: parsed.filters,
        revoke: parsed.revoke,
        id_override: parsed.id_override,
    })
}

// ── Event expansion ───────────────────────────────────────────────────────────

fn expand_event(func: ItemFn, attr: EventAttr) -> syn::Result<proc_macro2::TokenStream> {
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &func.vis;
    let attrs = &func.attrs;

    // Validate: no `self` receiver (must be free-standing)
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

    // Validate: no parameters
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[event] functions must take no parameters",
        ));
    }

    let fn_make_ident = proc_macro2::Ident::new(
        &format!("__sand_fn_{}_make", fn_name),
        proc_macro2::Span::call_site(),
    );
    let trigger_ident = proc_macro2::Ident::new(
        &format!("__sand_event_{}_trigger", fn_name),
        proc_macro2::Span::call_site(),
    );

    let body = build_cmd_body(&func.block);

    // Build the trigger expression tokens
    let trigger_expr = build_trigger_expr(&attr.event_type, &attr.filters)?;

    // id_override token
    let id_override_tokens = match &attr.id_override {
        Some(s) => quote! { ::std::option::Option::Some(#s) },
        None => quote! { ::std::option::Option::None },
    };

    let revoke_val = attr.revoke;

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

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #trigger_ident() -> ::sand_core::AdvancementTrigger {
            #trigger_expr
        }

        ::sand_core::inventory::submit!(::sand_core::EventDescriptor {
            path: #fn_name_str,
            id_override: #id_override_tokens,
            make_trigger: #trigger_ident,
            make: #fn_make_ident,
            revoke: #revoke_val,
        });
    })
}

/// Build the `AdvancementTrigger` expression for the given event type and filters.
fn build_trigger_expr(
    event_type: &syn::Ident,
    filters: &std::collections::HashMap<String, syn::Expr>,
) -> syn::Result<proc_macro2::TokenStream> {
    let event_str = event_type.to_string();

    match event_str.as_str() {
        "Join" | "Tick" => {
            check_no_unknown_filters(event_type, filters, &[])?;
            Ok(quote! { ::sand_core::AdvancementTrigger::Tick })
        }
        "Impossible" => {
            check_no_unknown_filters(event_type, filters, &[])?;
            Ok(quote! { ::sand_core::AdvancementTrigger::Impossible })
        }
        "Death" => {
            check_no_unknown_filters(event_type, filters, &["entity", "killing_blow"])?;
            let entity = build_opt_entity_filter(filters, "entity");
            let killing_blow = build_opt_value_filter(filters, "killing_blow")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::EntityKilledPlayer {
                    entity: #entity,
                    killing_blow: #killing_blow,
                }
            })
        }
        "Kill" => {
            check_no_unknown_filters(event_type, filters, &["entity", "killing_blow"])?;
            let entity = build_opt_entity_filter(filters, "entity");
            let killing_blow = build_opt_value_filter(filters, "killing_blow")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::PlayerKilledEntity {
                    entity: #entity,
                    killing_blow: #killing_blow,
                }
            })
        }
        "ItemUsed" => {
            check_no_unknown_filters(event_type, filters, &["item"])?;
            let item = build_opt_entity_filter(filters, "item");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::UsedItem { item: #item }
            })
        }
        "ItemConsumed" => {
            check_no_unknown_filters(event_type, filters, &["item"])?;
            let item = build_opt_entity_filter(filters, "item");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::ConsumeItem { item: #item }
            })
        }
        "UsingItem" => {
            check_no_unknown_filters(event_type, filters, &["item"])?;
            let item = build_opt_entity_filter(filters, "item");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::UsingItem { item: #item }
            })
        }
        "BlockPlaced" => {
            check_no_unknown_filters(event_type, filters, &["block", "item", "location", "state"])?;
            let block = build_opt_plain_string_filter(filters, "block");
            let item = build_opt_entity_filter(filters, "item");
            let location = build_opt_value_filter(filters, "location")?;
            let state = build_opt_state_map_filter(filters, "state")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::PlacedBlock {
                    block: #block,
                    item: #item,
                    location: #location,
                    state: #state,
                }
            })
        }
        "RecipeUnlocked" => {
            check_no_unknown_filters(event_type, filters, &["recipe"])?;
            let recipe = require_plain_string_filter(event_type, filters, "recipe")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::RecipeUnlocked { recipe: #recipe.to_string() }
            })
        }
        "EnterBlock" => {
            check_no_unknown_filters(event_type, filters, &["block", "state"])?;
            let block = build_opt_plain_string_filter(filters, "block");
            let state = build_opt_state_map_filter(filters, "state")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::EnterBlock {
                    block: #block,
                    state: #state,
                }
            })
        }
        "EnchantedItem" => {
            check_no_unknown_filters(event_type, filters, &["item", "levels"])?;
            let item = build_opt_entity_filter(filters, "item");
            let levels = build_opt_value_filter(filters, "levels")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::EnchantedItem {
                    item: #item,
                    levels: #levels,
                }
            })
        }
        "TamedAnimal" => {
            check_no_unknown_filters(event_type, filters, &["entity"])?;
            let entity = build_opt_entity_filter(filters, "entity");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::TamedAnimal { entity: #entity }
            })
        }
        "SummonedEntity" => {
            check_no_unknown_filters(event_type, filters, &["entity"])?;
            let entity = build_opt_entity_filter(filters, "entity");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::SummonedEntity { entity: #entity }
            })
        }
        "Location" => {
            check_no_unknown_filters(event_type, filters, &["location"])?;
            let location = build_opt_value_filter(filters, "location")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::Location { location: #location }
            })
        }
        "BredAnimals" => {
            check_no_unknown_filters(event_type, filters, &["parent", "partner", "child"])?;
            let parent = build_opt_entity_filter(filters, "parent");
            let partner = build_opt_entity_filter(filters, "partner");
            let child = build_opt_entity_filter(filters, "child");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::BredAnimals {
                    parent: #parent,
                    partner: #partner,
                    child: #child,
                }
            })
        }
        "InteractEntity" => {
            check_no_unknown_filters(event_type, filters, &["item", "entity"])?;
            let item = build_opt_entity_filter(filters, "item");
            let entity = build_opt_entity_filter(filters, "entity");
            Ok(quote! {
                ::sand_core::AdvancementTrigger::PlayerInteractedWithEntity {
                    item: #item,
                    entity: #entity,
                }
            })
        }
        "NetherTravel" => {
            check_no_unknown_filters(event_type, filters, &["entered", "exited", "distance"])?;
            let entered = build_opt_value_filter(filters, "entered")?;
            let exited = build_opt_value_filter(filters, "exited")?;
            let distance = build_opt_value_filter(filters, "distance")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::NetherTravel {
                    entered: #entered,
                    exited: #exited,
                    distance: #distance,
                }
            })
        }
        "InventoryChanged" => {
            check_no_unknown_filters(event_type, filters, &["items", "slots"])?;
            let items_tokens = build_inventory_items_filter(filters)?;
            let slots = build_opt_value_filter(filters, "slots")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::InventoryChanged {
                    slots: #slots,
                    items: #items_tokens,
                }
            })
        }
        "Custom" => {
            check_no_unknown_filters(event_type, filters, &["trigger", "conditions"])?;
            let trigger = require_plain_string_filter(event_type, filters, "trigger")?;
            let conditions = build_opt_json_object_filter(filters, "conditions")?;
            Ok(quote! {
                ::sand_core::AdvancementTrigger::Custom {
                    trigger: #trigger.to_string(),
                    conditions: #conditions,
                }
            })
        }
        _ => Err(syn::Error::new_spanned(
            event_type,
            format!(
                "unknown event type `{event_str}`; expected one of: \
                Join, Tick, Impossible, Death, Kill, ItemUsed, ItemConsumed, UsingItem, \
                BlockPlaced, RecipeUnlocked, EnterBlock, EnchantedItem, TamedAnimal, \
                SummonedEntity, Location, BredAnimals, InteractEntity, NetherTravel, \
                InventoryChanged, Custom"
            ),
        )),
    }
}

/// Verify that all keys in `filters` are in the `allowed` list.
fn check_no_unknown_filters(
    event_type: &syn::Ident,
    filters: &std::collections::HashMap<String, syn::Expr>,
    allowed: &[&str],
) -> syn::Result<()> {
    for key in filters.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(syn::Error::new_spanned(
                event_type,
                format!(
                    "unknown filter `{key}` for event `{}`; allowed: {}",
                    event_type,
                    if allowed.is_empty() {
                        "none".to_string()
                    } else {
                        allowed.join(", ")
                    }
                ),
            ));
        }
    }
    Ok(())
}

/// Build an `Option<serde_json::Value>` expression for an entity/item filter field.
///
/// Accepts:
/// - A plain string ID: `"minecraft:zombie"` → `{"type": "minecraft:zombie"}`
/// - A JSON object string: `r#"{"type":"minecraft:zombie","nbt":"..."}"#`
/// - A typed `EntityPredicate` or `ItemPredicate` expression (serialized via serde_json)
fn build_opt_entity_filter(
    filters: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> proc_macro2::TokenStream {
    match filters.get(key) {
        None => quote! { ::std::option::Option::None },
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => {
            let val = s.value();
            if val.starts_with('{') || val.starts_with('[') {
                // Raw JSON string — parse directly.
                quote! {
                    ::std::option::Option::Some(
                        ::sand_core::serde_json::from_str::<::sand_core::serde_json::Value>(#s)
                            .expect(concat!("invalid JSON in `", #key, "` filter"))
                    )
                }
            } else {
                // Plain type ID — auto-wrap as {"type": "..."}.
                quote! {
                    ::std::option::Option::Some(
                        ::sand_core::serde_json::json!({"type": #s})
                    )
                }
            }
        }
        Some(other) => {
            // Typed Rust expression (EntityPredicate, ItemPredicate, serde_json::Value, …)
            quote! {
                ::std::option::Option::Some(
                    ::sand_core::serde_json::to_value(#other)
                        .expect(concat!("failed to serialize `", #key, "` filter"))
                )
            }
        }
    }
}

/// Build an `Option<serde_json::Value>` for a raw JSON value filter field.
/// Strings starting with `{` or `[` are parsed; anything else is treated as a
/// JSON string value.
fn build_opt_value_filter(
    filters: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match filters.get(key) {
        None => Ok(quote! { ::std::option::Option::None }),
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => {
            let val = s.value();
            if val.starts_with('{') || val.starts_with('[') {
                Ok(quote! {
                    ::std::option::Option::Some(
                        ::sand_core::serde_json::from_str::<::sand_core::serde_json::Value>(#s).unwrap()
                    )
                })
            } else {
                Ok(quote! {
                    ::std::option::Option::Some(
                        ::sand_core::serde_json::Value::String(#s.to_string())
                    )
                })
            }
        }
        Some(other) => Ok(quote! {
            ::std::option::Option::Some({
                let __v = #other;
                ::sand_core::serde_json::to_value(__v).unwrap()
            })
        }),
    }
}

/// Build an `Option<serde_json::Value>` for a JSON object field specifically.
fn build_opt_json_object_filter(
    filters: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match filters.get(key) {
        None => Ok(quote! { ::std::option::Option::None }),
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => Ok(quote! {
            ::std::option::Option::Some(
                ::sand_core::serde_json::from_str::<::sand_core::serde_json::Value>(#s).unwrap()
            )
        }),
        Some(other) => Ok(quote! {
            ::std::option::Option::Some(::sand_core::serde_json::to_value(#other).unwrap())
        }),
    }
}

/// Build an `Option<String>` expression for a plain-string filter field.
fn build_opt_plain_string_filter(
    filters: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> proc_macro2::TokenStream {
    match filters.get(key) {
        None => quote! { ::std::option::Option::None },
        Some(expr) => quote! { ::std::option::Option::Some((#expr).to_string()) },
    }
}

/// Require a plain string filter field; produce a compile error if missing.
fn require_plain_string_filter(
    event_type: &syn::Ident,
    filters: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match filters.get(key) {
        None => Err(syn::Error::new_spanned(
            event_type,
            format!("`{key}` is required for `{}` events", event_type),
        )),
        Some(expr) => Ok(quote! { #expr }),
    }
}

/// Build an `Option<HashMap<String, String>>` expression for block state maps.
/// Expects a JSON object string like `r#"{"facing":"north"}"#`.
fn build_opt_state_map_filter(
    filters: &std::collections::HashMap<String, syn::Expr>,
    key: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    match filters.get(key) {
        None => Ok(quote! { ::std::option::Option::None }),
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => Ok(quote! {
            ::std::option::Option::Some(
                ::sand_core::serde_json::from_str::<
                    ::std::collections::HashMap<::std::string::String, ::std::string::String>
                >(#s).unwrap()
            )
        }),
        Some(other) => Ok(quote! {
            ::std::option::Option::Some(
                ::sand_core::serde_json::from_str::<
                    ::std::collections::HashMap<::std::string::String, ::std::string::String>
                >(&(#other).to_string()).unwrap()
            )
        }),
    }
}

/// Build a `Vec<serde_json::Value>` expression for InventoryChanged items.
///
/// Accepts:
/// - A JSON array string: `r#"[{"id":"minecraft:diamond"}]"#`
/// - A Rust array/Vec of `ItemPredicate` (or anything `serde::Serialize`):
///   `[ItemPredicate::id("minecraft:leather_boots")]`
fn build_inventory_items_filter(
    filters: &std::collections::HashMap<String, syn::Expr>,
) -> syn::Result<proc_macro2::TokenStream> {
    match filters.get("items") {
        None => Ok(quote! { ::std::vec::Vec::new() }),
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        })) => {
            // Raw JSON string — parse as array at runtime.
            Ok(quote! {
                ::sand_core::serde_json::from_str::<
                    ::std::vec::Vec<::sand_core::serde_json::Value>
                >(#s).expect("invalid JSON array in InventoryChanged items filter")
            })
        }
        Some(other) => {
            // Typed Rust expression (e.g. `[ItemPredicate::id("...")]`).
            // Serialize via serde_json::to_value and unwrap the array.
            Ok(quote! {{
                let __items_val = ::sand_core::serde_json::to_value(#other)
                    .expect("InventoryChanged items must be serializable");
                match __items_val {
                    ::sand_core::serde_json::Value::Array(arr) => arr,
                    single => vec![single],
                }
            }})
        }
    }
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
    item: Option<syn::LitStr>,
    custom_data: Option<syn::LitStr>,
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
        let mut item: Option<syn::LitStr> = None;
        let mut custom_data: Option<syn::LitStr> = None;

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
                    item = Some(input.parse()?);
                }
                "custom_data" => {
                    custom_data = Some(input.parse()?);
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
/// // Fire when any custom "mana boots" item is equipped in the feet slot
/// #[armor_event(Equip, slot = Feet, item = "minecraft:leather_boots",
///               custom_data = "{mana_boots:true}")]
/// pub fn on_mana_boots_equip() {
///     "scoreboard players set @s mana_regen_boost 1";
/// }
///
/// #[armor_event(Unequip, slot = Feet, item = "minecraft:leather_boots",
///               custom_data = "{mana_boots:true}")]
/// pub fn on_mana_boots_unequip() {
///     "scoreboard players set @s mana_regen_boost 0";
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

    let body = build_cmd_body(&func.block);

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
