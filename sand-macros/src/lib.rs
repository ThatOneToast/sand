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
//! - **`#[datapack_component]`** — registers a datapack component (advancement, recipe,
//!   loot table, etc.) or hooks a function into `Tick`/`Load`/custom tags.
//! - **`#[on_event]`** — wires a handler function to either a stateless
//!   `AdvancementEvent` (`Event<T>` handler context) or an advanced custom
//!   `SandEvent` (concrete marker parameter). See [`on_event`] for the canonical
//!   split.
//! - **`#[schedule]`** — defines a function that runs for N ticks (with an
//!   optional interval), triggered at runtime via generated `_start`/`_stop` functions.
//! - **`#[custom_item]`** — reads a `CustomItem`-returning function and generates a typed
//!   struct with `BASE`, `PREDICATE`, `CUSTOM_DATA_KEY` constants and an `item()` method.
//! - **`run_fn!`** — defines an inline function and returns the
//!   `cmd::function(...)` call to invoke it.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_macros::{datapack_component, function, run_fn};
//!
//! #[function]
//! pub fn greet() {
//!     cmd::tellraw(Target::players(), Text::new("Hello from Sand").gold());
//! }
//!
//! #[datapack_component(Tick)]
//! pub fn tick() {
//!     cmd::say("Tick from Sand");
//! }
//!
//! #[datapack_component(Load)]
//! pub fn on_load() {
//!     cmd::say("Sand datapack loaded");
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use sand_api_contract::syntax::{
    GeneratedApiContract, GeneratedApiKind, render_generated_rustdoc, validate_generated_expansion,
};
use syn::fold::{self, Fold};
use syn::visit::{self, Visit};
use syn::{ItemFn, LitStr, parse_macro_input, token};

mod api_contract;
mod entity_state;

/// Capability policy for macros which replace one author declaration or emit
/// only registration plumbing. The annotated public function may remain, but
/// a future implementation change cannot silently add another public Rust API.
fn validate_preserved_public_surface(
    expansion: &proc_macro2::TokenStream,
    expected_function: Option<String>,
) -> syn::Result<()> {
    let file: syn::File = syn::parse2(expansion.clone())?;
    let mut actual = Vec::new();
    collect_exported_macro_surface(&file.items, &mut actual);
    for item in &file.items {
        if let syn::Item::Impl(item_impl) = item
            && item_impl.trait_.is_none()
        {
            for impl_item in &item_impl.items {
                let identity = match impl_item {
                    syn::ImplItem::Const(item)
                        if matches!(item.vis, syn::Visibility::Public(_)) =>
                    {
                        Some(format!("associated const {}", item.ident))
                    }
                    syn::ImplItem::Fn(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                        Some(format!("associated fn {}", item.sig.ident))
                    }
                    syn::ImplItem::Type(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                        Some(format!("associated type {}", item.ident))
                    }
                    _ => None,
                };
                if let Some(identity) = identity {
                    actual.push(identity);
                }
            }
        }
        let identity = match item {
            syn::Item::Const(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("const {}", item.ident))
            }
            syn::Item::Enum(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("enum {}", item.ident))
            }
            syn::Item::ExternCrate(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("extern crate {}", item.ident))
            }
            syn::Item::Fn(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("fn {}", item.sig.ident))
            }
            syn::Item::Mod(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("mod {}", item.ident))
            }
            syn::Item::Static(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("static {}", item.ident))
            }
            syn::Item::Struct(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("struct {}", item.ident))
            }
            syn::Item::Trait(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("trait {}", item.ident))
            }
            syn::Item::TraitAlias(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("trait alias {}", item.ident))
            }
            syn::Item::Type(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("type {}", item.ident))
            }
            syn::Item::Union(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some(format!("union {}", item.ident))
            }
            syn::Item::Use(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                Some("use declaration".to_owned())
            }
            _ => None,
        };
        if let Some(identity) = identity {
            actual.push(identity);
        }
    }
    let expected = expected_function
        .into_iter()
        .map(|name| format!("fn {name}"))
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "shape-preserving macro public-surface drift: expected {expected:?}, emitted {actual:?}; add first-class generated API contracts before exposing new declarations"
            ),
        ));
    }
    Ok(())
}

fn collect_exported_macro_surface(items: &[syn::Item], actual: &mut Vec<String>) {
    for item in items {
        match item {
            syn::Item::Macro(item)
                if item
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("macro_export")) =>
            {
                actual.push(format!(
                    "exported macro {}",
                    item.ident
                        .as_ref()
                        .map_or_else(|| "<anonymous>".to_owned(), ToString::to_string)
                ));
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    collect_exported_macro_surface(items, actual);
                }
            }
            _ => {}
        }
    }
}

fn expected_public_function(function: &ItemFn) -> Option<String> {
    matches!(function.vis, syn::Visibility::Public(_)).then(|| function.sig.ident.to_string())
}

/// Defines a Sand API's semantic contract and generates its local Rustdoc and
/// machine-readable catalog entry from that declaration.
///
/// Apply this to a public module, type, function, method, associated item, or
/// named macro. The declaration requires a summary, context, Minecraft
/// behavior, use/avoid guidance, and an example; callable parameters and return
/// values and public fields or variants must also be described. Existing `///`
/// documentation is preserved after the generated contract sections.
///
/// Examples on facade-defined APIs are emitted as compile-checked `no_run`
/// doctests. Sand's implementation crates retain facade-shaped examples for
/// users, but mark those copies ignored because depending back on `sand` would
/// create a Cargo cycle; downstream facade fixtures compile representative
/// examples instead.
///
/// The emitted registration powers `sand api search`, `sand api show`, export,
/// and drift enforcement. Use `registry = sand_api_contract` in Sand's
/// implementation crates; downstream users normally use the facade default.
///
/// # Example
///
/// ```rust,ignore
/// #[sand::api(
///     path = "my_pack::heal",
///     module = "my_pack",
///     summary = "Restore health to the selected player.",
///     context = "A gameplay helper used by healing systems.",
///     minecraft = "Emits the vanilla instant-health effect command.",
///     use_when = ["A system needs immediate healing"],
///     avoid_when = ["Health should regenerate over time"],
///     params(player = "The single player to heal."),
///     example = "heal(Target::current_player());",
/// )]
/// pub fn heal(player: sand::command::Target) {}
/// ```
#[proc_macro_attribute]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_contract::expand(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Emits one typed registry identifier and, when requested, its definition-owned
/// API contract Rustdoc from the same semantic declaration.
#[proc_macro]
pub fn registry_id(input: TokenStream) -> TokenStream {
    sand_api_contract::syntax::registry_id::expand(input.into())
        .map(|expansion| expansion.rust)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive a typed state schema, its field constants, and a concrete bound view.
///
/// Every scope owns dependency provisioning and an independent presence/version
/// marker. Player state is observed automatically, entity/living state is
/// initialized explicitly through attachment or adoption, and global state is
/// bound to one deterministic singleton holder. Entity, living, and player
/// State can be queried directly from `#[system]`; global State remains a
/// singleton accessed through its generated `global()` method.
///
/// Generated APIs include definition-owned typed field handles, a sibling
/// `NameBound` view with holder-bound accessors, binding methods appropriate to
/// the selected scope (`on` or `global`), schema metadata, and lifecycle hooks.
/// Field attributes select score, fixed-point, flag, enum, timer, cooldown, or
/// NBT storage semantics; invalid scope/type combinations fail at compile time.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(sand::State)]
/// #[state(namespace = "game", scope = "player")]
/// struct PlayerState {
///     #[state(default = 20, min = 0)]
///     health: sand::state::Score<i32>,
/// }
/// let player = PlayerState::on(sand::command::Target::current_player());
/// player.health.set(20);
/// ```
#[proc_macro_derive(State, attributes(state))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    entity_state::derive_state(parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive a concrete, nestable holder-bound view over existing State schemas.
///
/// Each public field must be a `State` or another `StateBundle`. The generated
/// binding API binds all fields to one owner while retaining their concrete
/// types; it does not create new scoreboard storage or lifecycle ownership.
///
/// A non-global bundle can be used directly as a `#[system]` query, requiring
/// every flattened component presence marker.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(sand::StateBundle)]
/// struct CombatState {
///     damageable: DamageableEntity,
///     effects: ActiveEffects,
/// }
/// ```
#[proc_macro_derive(StateBundle)]
pub fn derive_state_bundle(input: TokenStream) -> TokenStream {
    entity_state::derive_bundle(parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive a concrete presence-filtered query over State schemas and bundles.
///
/// `#[required]` fields must be attached, `#[optional]` fields are exposed when
/// present, and `#[without]`/`#[forbidden]` fields exclude owners. The generated
/// query lowers these presence rules to Sand's State marker objectives and
/// provides typed bound views to the query body.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(sand::StateQuery)]
/// struct AlivePlayers {
///     #[required]
///     health: PlayerHealth,
///     #[without]
///     eliminated: Eliminated,
/// }
/// ```
#[proc_macro_derive(
    StateQuery,
    attributes(require, required, optional, without, forbidden, state, query)
)]
pub fn derive_state_query(input: TokenStream) -> TokenStream {
    entity_state::derive_query(parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive the stable scoreboard encoding used by `EntityEnum<T>`.
#[proc_macro_derive(EntityStateEnum)]
pub fn derive_entity_state_enum(input: TokenStream) -> TokenStream {
    entity_state::derive_enum(parse_macro_input!(input as syn::DeriveInput))
        .and_then(|tokens| validate_preserved_public_surface(&tokens, None).map(|()| tokens))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive the canonical stable scoreboard encoding used by `StateEnum<T>`.
///
/// Variants receive deterministic integer encodings in declaration order and
/// generate conversion metadata used by typed State reads, writes, and
/// conditions. The enum must contain only unit variants; reordering variants
/// changes their stored scoreboard representation.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone, Copy, sand::StateEnum)]
/// enum Phase { Lobby, Playing, Finished }
/// ```
#[proc_macro_derive(StateEnum)]
pub fn derive_state_enum(input: TokenStream) -> TokenStream {
    entity_state::derive_enum(parse_macro_input!(input as syn::DeriveInput))
        .and_then(|tokens| validate_preserved_public_surface(&tokens, None).map(|()| tokens))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Register one optional `StateLifecycle` implementation.
///
/// Apply it to an `impl StateLifecycle for MyState` block without arguments.
/// Sand records the schema's provisioning, initialization, tick,
/// reconciliation, cleanup, and migration callbacks and invokes them at the
/// matching generated lifecycle points. The state type must also implement
/// `State`/`EntityState`.
///
/// # Example
///
/// ```rust,ignore
/// #[sand::state_lifecycle]
/// impl sand::state::StateLifecycle for PlayerState {
///     // Override only the lifecycle hooks this schema needs.
/// }
/// ```
#[proc_macro_attribute]
pub fn state_lifecycle(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[state_lifecycle] does not accept arguments",
        )
        .into_compile_error()
        .into();
    }
    let implementation = parse_macro_input!(item as syn::ItemImpl);
    let Some((_, trait_path, _)) = &implementation.trait_ else {
        return syn::Error::new_spanned(
            &implementation.self_ty,
            "#[state_lifecycle] requires `impl StateLifecycle for YourState`",
        )
        .into_compile_error()
        .into();
    };
    if trait_path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "StateLifecycle")
    {
        return syn::Error::new_spanned(
            trait_path,
            "#[state_lifecycle] requires an implementation of StateLifecycle",
        )
        .into_compile_error()
        .into();
    }
    let self_ty = &implementation.self_ty;
    quote! {
        #implementation
        const _: () = {
            fn schema() -> ::sand::__private::StateSchema {
                <#self_ty as ::sand::__private::EntityState>::schema()
            }
            fn provision() -> Vec<String> {
                <#self_ty as ::sand::__private::StateLifecycle>::provision(
                    ::sand::__private::StateProvision::default(),
                )
            }
            fn initialize(holder: &'static str) -> Vec<String> {
                <#self_ty as ::sand::__private::StateLifecycle>::initialize(
                    ::sand::__private::StateInit::new(holder),
                )
            }
            fn tick(holder: &'static str) -> Vec<String> {
                <#self_ty as ::sand::__private::StateLifecycle>::tick(
                    ::sand::__private::StateTick::new(holder),
                )
            }
            fn reconcile(holder: &'static str) -> Vec<String> {
                <#self_ty as ::sand::__private::StateLifecycle>::reconcile(
                    ::sand::__private::StateReconcile::new(holder),
                )
            }
            fn cleanup(holder: &'static str) -> Vec<String> {
                <#self_ty as ::sand::__private::StateLifecycle>::cleanup(
                    ::sand::__private::StateCleanup::new(holder),
                )
            }
            fn migrate(holder: &'static str, from: u32, to: u32) -> Vec<String> {
                <#self_ty as ::sand::__private::StateLifecycle>::migrate(
                    ::sand::__private::StateMigrate::new(holder, from, to),
                )
            }
            ::sand::__private::inventory::submit! {
                ::sand::__private::StateHookDescriptor {
                    schema,
                    provision,
                    initialize,
                    tick,
                    reconcile,
                    cleanup,
                    migrate,
                }
            }
        };
    }
    .into()
}

/// Register typed State systems from a free function or grouped inherent impl.
///
/// The declared parameter remains its concrete source type: a scoped `State`,
/// `StateBundle`, or derived `StateQuery`. With `sand::prelude::*` in scope,
/// completion on `query.` exposes `StateQueryOperations::each` and
/// `StateQueryOperations::current`, and the callback item is the generated
/// concrete bound view rather than a generic ECS tuple.
///
/// # Free tick systems
///
/// Empty `#[system]` and `#[system(tick)]` both run every tick.
/// `#[system(tick, every = 20)]` runs every twentieth server tick:
///
/// ```rust,ignore
/// use sand::prelude::*;
///
/// #[system(tick, every = 20)]
/// fn regenerate(query: Health) {
///     query.each(|health| health.current.add(1));
/// }
/// ```
///
/// A free tick system has exactly one simply named query parameter. Write
/// command-producing expressions as statements. The outer system body is
/// declarative and returns nothing; an `each` or `current` closure returns the
/// `Vec<String>` produced by typed State operations (or explicitly builds and
/// returns one when it combines several operations). The query parameter name
/// cannot be shadowed inside the system body because it identifies the typed
/// query lowered into the export adapter. This includes bindings introduced by
/// patterns and local value items such as constants, statics, functions, and
/// unit or tuple structs.
///
/// # Grouped tick and event systems
///
/// `#[system]` on an inherent impl is organization only and creates no runtime
/// object. Methods cannot take `self`:
///
/// ```rust,ignore
/// use sand::prelude::*;
///
/// struct CombatSystems;
///
/// #[system]
/// impl CombatSystems {
///     #[tick]
///     fn update(query: Combatants) {
///         query.each(|fighter| fighter.health.current.add(1));
///     }
///
///     #[event(PlayerAttack)]
///     fn attack(_event: PlayerAttack, query: Combatants) {
///         query.current(|fighter| fighter.health.current.add(1));
///     }
/// }
/// ```
///
/// Event methods take the event first and optionally one query second. The
/// type in `#[event(Type)]` must equal the first parameter type. Event dispatch
/// has already selected its owner as `@s`; `current` checks required and
/// forbidden component presence on that executor without another scan, while
/// `each` deliberately starts a new scope-wide scan.
///
/// # Scheduling and Minecraft semantics
///
/// Systems run in deterministic registration identity order. Adjacent systems
/// with the same cadence and compatible outer selector can share one scan;
/// different cadence, scope, required filters, or intervening opaque work
/// prevents sharing. Generated function identities are deterministic and
/// identical command bodies are deduplicated. Queries see only loaded entities
/// because Minecraft selectors do; Sand does not keep an in-memory world or a
/// Rust borrowing/mutation model.
///
/// Required membership is selected when an `each` iteration starts. Optional
/// and forbidden checks are emitted as ordered runtime command guards, so an
/// earlier attach/detach can affect a later guarded command without retroactively
/// changing the already-selected iteration. See the State chapter in the Sand
/// book for the full lifecycle and scan-sharing model.
#[proc_macro_attribute]
pub fn system(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match parse_system_tick_attr(attr, true) {
        Ok(attr) => attr,
        Err(error) => return error.into_compile_error().into(),
    };
    let tokens: proc_macro2::TokenStream = item.into();
    if let Ok(function) = syn::parse2::<ItemFn>(tokens.clone()) {
        return expand_state_system_function(function, attr)
            .unwrap_or_else(syn::Error::into_compile_error)
            .into();
    }
    if let Ok(implementation) = syn::parse2::<syn::ItemImpl>(tokens.clone()) {
        return expand_state_system_impl(implementation)
            .unwrap_or_else(syn::Error::into_compile_error)
            .into();
    }
    syn::Error::new_spanned(tokens, "#[system] requires a function or inherent impl")
        .into_compile_error()
        .into()
}

#[derive(Clone, Copy)]
struct SystemTickAttr {
    every: u32,
}

fn parse_system_tick_attr(attr: TokenStream, allow_empty: bool) -> syn::Result<SystemTickAttr> {
    let tokens: proc_macro2::TokenStream = attr.into();
    if tokens.is_empty() {
        if allow_empty {
            return Ok(SystemTickAttr { every: 1 });
        }
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "tick system requires #[tick] or #[tick(every = N)]",
        ));
    }
    struct Parsed(SystemTickAttr);
    impl syn::parse::Parse for Parsed {
        fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
            if input.peek(syn::Ident) {
                let fork = input.fork();
                let ident: syn::Ident = fork.parse()?;
                if ident == "tick" {
                    let _: syn::Ident = input.parse()?;
                    if input.peek(syn::Token![,]) {
                        let _: syn::Token![,] = input.parse()?;
                    }
                }
            }
            let mut every = 1;
            if !input.is_empty() {
                let key: syn::Ident = input.parse()?;
                if key != "every" {
                    return Err(syn::Error::new_spanned(
                        key,
                        "expected `tick` or `every = N`",
                    ));
                }
                let _: syn::Token![=] = input.parse()?;
                let value: syn::LitInt = input.parse()?;
                every = value.base10_parse()?;
                if every == 0 {
                    return Err(syn::Error::new_spanned(
                        value,
                        "[SAND-SYSTEM-CADENCE] system tick cadence must be at least one",
                    ));
                }
            }
            if !input.is_empty() {
                return Err(input.error("unexpected system cadence arguments"));
            }
            Ok(Self(SystemTickAttr { every }))
        }
    }
    syn::parse2::<Parsed>(tokens).map(|parsed| parsed.0)
}

fn state_system_body(function: &ItemFn) -> syn::Result<(syn::Ident, syn::Type, syn::Block)> {
    if !matches!(function.sig.output, syn::ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &function.sig.output,
            "[SAND-SYSTEM-RETURN] systems do not declare a Rust return type; write command-producing expressions as statements and return Vec<String> only from query closures",
        ));
    }
    if function.sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &function.sig.inputs,
            "[SAND-SYSTEM-PARAM] free #[system] functions require exactly one scoped State, StateBundle, or StateQuery parameter",
        ));
    }
    let argument = function.sig.inputs.first().expect("one argument");
    let syn::FnArg::Typed(argument) = argument else {
        return Err(syn::Error::new_spanned(
            argument,
            "[SAND-SYSTEM-PARAM] free systems cannot use a self receiver",
        ));
    };
    let syn::Pat::Ident(pattern) = argument.pat.as_ref() else {
        return Err(syn::Error::new_spanned(
            &argument.pat,
            "[SAND-SYSTEM-PARAM] system query parameters must use a simple identifier",
        ));
    };
    let query_ident = pattern.ident.clone();
    let query_ty = (*argument.ty).clone();
    let block = lower_state_system_block(
        (*function.block).clone(),
        query_ident.clone(),
        query_ty.clone(),
    )?;
    Ok((query_ident, query_ty, block))
}

fn lower_state_system_block(
    block: syn::Block,
    query_ident: syn::Ident,
    query_ty: syn::Type,
) -> syn::Result<syn::Block> {
    let mut shadowing = StateSystemQueryShadowing {
        query_ident: &query_ident,
        shadow: None,
    };
    shadowing.visit_block(&block);
    if let Some(shadow) = shadowing.shadow {
        return Err(syn::Error::new_spanned(
            shadow,
            "[SAND-SYSTEM-PARAM] the system query parameter cannot be shadowed inside its body",
        ));
    }
    Ok(StateSystemQueryLowering {
        query_ident,
        query_ty,
    }
    .fold_block(block))
}

struct StateSystemQueryShadowing<'a> {
    query_ident: &'a syn::Ident,
    shadow: Option<syn::Ident>,
}

/// Make the preserved Rust endpoint resolve the same canonical trait method as
/// its export adapter, even if the schema type declares an inherent method with
/// the same name.
struct StateSystemQueryQualification {
    query_ident: syn::Ident,
}

impl Fold for StateSystemQueryQualification {
    fn fold_expr(&mut self, expression: syn::Expr) -> syn::Expr {
        let syn::Expr::MethodCall(call) = &expression else {
            return fold::fold_expr(self, expression);
        };
        let syn::Expr::Path(receiver) = call.receiver.as_ref() else {
            return fold::fold_expr(self, expression);
        };
        if !receiver.path.is_ident(&self.query_ident)
            || (call.method != "each" && call.method != "current")
        {
            return fold::fold_expr(self, expression);
        }

        let receiver = receiver.clone();
        let arguments = call
            .args
            .clone()
            .into_iter()
            .map(|argument| self.fold_expr(argument))
            .collect::<syn::punctuated::Punctuated<_, syn::Token![,]>>();
        if call.method == "each" {
            syn::parse_quote_spanned! {call.method.span()=>
                ::sand::prelude::StateQueryOperations::each(#receiver, #arguments)
            }
        } else {
            syn::parse_quote_spanned! {call.method.span()=>
                ::sand::prelude::StateQueryOperations::current(#receiver, #arguments)
            }
        }
    }
}

fn qualify_state_system_block(block: syn::Block, query_ident: syn::Ident) -> syn::Block {
    StateSystemQueryQualification { query_ident }.fold_block(block)
}

impl<'ast> Visit<'ast> for StateSystemQueryShadowing<'_> {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        if self.shadow.is_none() && pattern.ident == *self.query_ident {
            self.shadow = Some(pattern.ident.clone());
        }
        visit::visit_pat_ident(self, pattern);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        self.record_value_item(&item.ident);
        visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        self.record_value_item(&item.ident);
        visit::visit_item_static(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.record_value_item(&item.sig.ident);
        visit::visit_item_fn(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if matches!(item.fields, syn::Fields::Unit | syn::Fields::Unnamed(_)) {
            self.record_value_item(&item.ident);
        }
        visit::visit_item_struct(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.visit_use_binding(&item.tree);
        visit::visit_item_use(self, item);
    }
}

impl StateSystemQueryShadowing<'_> {
    fn record_value_item(&mut self, ident: &syn::Ident) {
        if self.shadow.is_none() && ident == self.query_ident {
            self.shadow = Some(ident.clone());
        }
    }

    fn visit_use_binding(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => self.visit_use_binding(&path.tree),
            syn::UseTree::Name(name) => self.record_value_item(&name.ident),
            syn::UseTree::Rename(rename) => self.record_value_item(&rename.rename),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.visit_use_binding(tree);
                }
            }
            syn::UseTree::Glob(_) => {}
        }
    }
}

/// Re-target the editor-visible query operations only in the private export
/// adapter. The authored endpoint remains untouched and type-checks against
/// `StateQueryOperations`; the adapter needs no fabricated runtime value.
struct StateSystemQueryLowering {
    query_ident: syn::Ident,
    query_ty: syn::Type,
}

impl Fold for StateSystemQueryLowering {
    fn fold_expr(&mut self, expression: syn::Expr) -> syn::Expr {
        let syn::Expr::MethodCall(call) = &expression else {
            return fold::fold_expr(self, expression);
        };
        let syn::Expr::Path(receiver) = call.receiver.as_ref() else {
            return fold::fold_expr(self, expression);
        };
        if !receiver.path.is_ident(&self.query_ident)
            || (call.method != "each" && call.method != "current")
        {
            return fold::fold_expr(self, expression);
        }

        let arguments = call
            .args
            .clone()
            .into_iter()
            .map(|argument| self.fold_expr(argument))
            .collect::<syn::punctuated::Punctuated<_, syn::Token![,]>>();
        let query_ty = &self.query_ty;
        if call.method == "each" {
            syn::parse_quote_spanned! {call.method.span()=>
                ::sand::__private::lower_system_query_each::<#query_ty, _>(#arguments)
            }
        } else {
            syn::parse_quote_spanned! {call.method.span()=>
                ::sand::__private::lower_system_query_current::<#query_ty, _>(#arguments)
            }
        }
    }
}

fn expand_state_system_function(
    function: ItemFn,
    attr: SystemTickAttr,
) -> syn::Result<proc_macro2::TokenStream> {
    let (query_ident, query_ty, block) = state_system_body(&function)?;
    let mut authored = function.clone();
    *authored.block = qualify_state_system_block((*function.block).clone(), query_ident);
    authored
        .attrs
        .push(syn::parse_quote!(#[allow(dead_code, unused_must_use, unused_variables)]));
    let ident = &function.sig.ident;
    let factory = quote::format_ident!("__sand_system_{}_make", ident);
    let body = build_cmd_body(&block)?;
    let every = attr.every;
    Ok(quote! {
        #authored

        const _: fn() = ::sand::__private::assert_system_query_parameter::<#query_ty>;

        #[doc(hidden)]
        fn #factory() -> Vec<String> {
            let _: ::std::marker::PhantomData<#query_ty> = ::std::marker::PhantomData;
            #body
        }

        ::sand::__private::inventory::submit! {
            ::sand::__private::StateSystemDescriptor {
                id: concat!(module_path!(), "::", stringify!(#ident)),
                every: #every,
                make: #factory,
            }
        }
    })
}

fn expand_state_system_impl(
    mut implementation: syn::ItemImpl,
) -> syn::Result<proc_macro2::TokenStream> {
    if implementation.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            implementation.impl_token,
            "#[system] grouped form requires an inherent impl",
        ));
    }
    let self_ty = &implementation.self_ty;
    let mut registrations = Vec::new();
    for item in &mut implementation.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        let tick_index = method
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident("tick"));
        let Some(tick_index) = tick_index else {
            if let Some(event_index) = method
                .attrs
                .iter()
                .position(|attr| attr.path().is_ident("event"))
            {
                let event_attr = method.attrs.remove(event_index);
                if !matches!(method.sig.output, syn::ReturnType::Default) {
                    return Err(syn::Error::new_spanned(
                        &method.sig.output,
                        "[SAND-SYSTEM-RETURN] event systems do not declare a Rust return type; write command-producing expressions as statements and return Vec<String> only from query closures",
                    ));
                }
                let expected_ty: syn::Type = match event_attr.meta {
                    syn::Meta::List(list) => syn::parse2(list.tokens)?,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            event_attr,
                            "use #[event(EventType)] on a grouped system method",
                        ));
                    }
                };
                if !(1..=2).contains(&method.sig.inputs.len()) {
                    return Err(syn::Error::new_spanned(
                        &method.sig.inputs,
                        "[SAND-SYSTEM-EVENT] event systems accept `(event)` or `(event, query)`; found a different parameter count",
                    ));
                }
                let syn::FnArg::Typed(argument) = method.sig.inputs.first().expect("one argument")
                else {
                    return Err(syn::Error::new_spanned(
                        &method.sig.inputs,
                        "[SAND-SYSTEM-EVENT] grouped event systems cannot use self",
                    ));
                };
                let actual_ty = &argument.ty;
                if quote!(#expected_ty).to_string() != quote!(#actual_ty).to_string() {
                    return Err(syn::Error::new_spanned(
                        &argument.ty,
                        "[SAND-SYSTEM-EVENT] #[event(EventType)] must match the first method parameter type",
                    ));
                }
                let mut event_block = method.block.clone();
                let mut event_query_ty = None;
                if method.sig.inputs.len() == 2 {
                    let query_argument = method.sig.inputs.iter().nth(1).expect("two inputs");
                    let syn::FnArg::Typed(query_argument) = query_argument else {
                        return Err(syn::Error::new_spanned(
                            query_argument,
                            "[SAND-SYSTEM-EVENT] grouped event systems cannot use self",
                        ));
                    };
                    let syn::Pat::Ident(query_pattern) = query_argument.pat.as_ref() else {
                        return Err(syn::Error::new_spanned(
                            &query_argument.pat,
                            "[SAND-SYSTEM-PARAM] event query parameters must use a simple identifier",
                        ));
                    };
                    let query_ty = (*query_argument.ty).clone();
                    event_block = lower_state_system_block(
                        event_block,
                        query_pattern.ident.clone(),
                        query_ty.clone(),
                    )?;
                    method.block = qualify_state_system_block(
                        method.block.clone(),
                        query_pattern.ident.clone(),
                    );
                    event_query_ty = Some(query_ty);
                }
                let mut signature = method.sig.clone();
                while signature.inputs.len() > 1 {
                    signature.inputs.pop();
                }
                method.attrs.push(
                    syn::parse_quote!(#[allow(dead_code, unused_must_use, unused_variables)]),
                );
                let original = &method.sig.ident;
                signature.ident = quote::format_ident!("__sand_system_event_{}", original);
                let function = ItemFn {
                    attrs: method.attrs.clone(),
                    vis: syn::Visibility::Inherited,
                    sig: signature,
                    block: Box::new(event_block),
                };
                let event_adapter = expand_event(TokenStream::new(), function)?;
                registrations.push(if let Some(query_ty) = event_query_ty {
                    quote! {
                        const _: fn() =
                            ::sand::__private::assert_system_query_parameter::<#query_ty>;
                        #event_adapter
                    }
                } else {
                    event_adapter
                });
            }
            continue;
        };
        let tick_attr = method.attrs.remove(tick_index);
        let cadence = match &tick_attr.meta {
            syn::Meta::Path(_) => SystemTickAttr { every: 1 },
            syn::Meta::List(list) => parse_system_tick_attr(list.tokens.clone().into(), true)?,
            syn::Meta::NameValue(_) => {
                return Err(syn::Error::new_spanned(
                    tick_attr,
                    "[SAND-SYSTEM-CADENCE] use #[tick] or #[tick(every = N)]",
                ));
            }
        };
        let function = ItemFn {
            attrs: Vec::new(),
            vis: method.vis.clone(),
            sig: method.sig.clone(),
            block: Box::new(method.block.clone()),
        };
        let (query_ident, query_ty, block) = state_system_body(&function)?;
        let body = build_cmd_body(&block)?;
        method.block = qualify_state_system_block(method.block.clone(), query_ident);
        method
            .attrs
            .push(syn::parse_quote!(#[allow(dead_code, unused_must_use, unused_variables)]));
        let method_ident = &method.sig.ident;
        let factory = quote::format_ident!("__sand_system_{}_make", method_ident);
        let every = cadence.every;
        registrations.push(quote! {
            const _: fn() = ::sand::__private::assert_system_query_parameter::<#query_ty>;

            #[doc(hidden)]
            fn #factory() -> Vec<String> {
                let _: ::std::marker::PhantomData<#query_ty> = ::std::marker::PhantomData;
                #body
            }
            ::sand::__private::inventory::submit! {
                ::sand::__private::StateSystemDescriptor {
                    id: concat!(module_path!(), "::", stringify!(#self_ty), "::", stringify!(#method_ident)),
                    every: #every,
                    make: #factory,
                }
            }
        });
    }
    Ok(quote! { #implementation #(#registrations)* })
}

/// Register a zero-argument component-first `EntityArchetype<K>` factory for export.
///
/// The annotated function remains callable by author code. Each export calls
/// it afresh, so registration does not retain process-global mutable builder
/// state between exports or tests.
/// The function must take no parameters and return a concrete typed
/// `EntityArchetype`; Sand invokes it during export to provision the
/// archetype's scoreboards, tags, attributes, equipment, and lifecycle
/// functions. Use `#[entity_archetype("path")]` to override the generated
/// resource path when the default function name is unsuitable.
///
/// # Example
///
/// ```rust,ignore
/// use sand::prelude::*;
///
/// #[entity_archetype]
/// fn zombie_guard() -> EntityArchetype<ZombieKind> {
///     EntityArchetype::new("demo:zombie_guard".parse().unwrap())
///         .components::<GuardState>()
///         .health(HealthBinding::new(GuardState::max_health))
/// }
/// ```
#[proc_macro_attribute]
pub fn entity_archetype(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[entity_archetype] does not accept arguments",
        )
        .into_compile_error()
        .into();
    }
    let function = parse_macro_input!(item as ItemFn);
    let expected = expected_public_function(&function);
    expand_entity_archetype(function)
        .and_then(|tokens| validate_preserved_public_surface(&tokens, expected).map(|()| tokens))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_entity_archetype(function: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    if !function.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &function.sig.inputs,
            "#[entity_archetype] functions must take no parameters",
        ));
    }
    if !function.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &function.sig.generics,
            "#[entity_archetype] functions cannot be generic",
        ));
    }
    if function.sig.asyncness.is_some()
        || function.sig.constness.is_some()
        || function.sig.unsafety.is_some()
        || function.sig.abi.is_some()
        || function.sig.variadic.is_some()
    {
        return Err(syn::Error::new_spanned(
            &function.sig,
            "#[entity_archetype] requires an ordinary synchronous safe Rust function",
        ));
    }
    let name = &function.sig.ident;
    let factory = proc_macro2::Ident::new(
        &format!("__sand_entity_archetype_{}_make", name),
        name.span(),
    );
    Ok(quote! {
        #function

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #factory() -> ::std::result::Result<
            ::sand::__private::entity::ArchetypeDefinition,
            ::sand::__private::entity::EntityDiagnostic,
        > {
            ::std::result::Result::Ok(
                ::sand::__private::entity::archetype::registered_definition(&#name())
            )
        }

        ::sand::__private::inventory::submit!(
            ::sand::__private::entity::EntityArchetypeDescriptor {
                make: #factory,
            }
        );
    })
}

// ── Body transformation ───────────────────────────────────────────────────────

/// Convert a `#[function]` / `#[datapack_component(Tick|Load|Tag)]` block into the
/// `Vec<String>` construction the build pipeline expects.
///
/// All expressions — with or without a trailing `;` — and macro invocations are
/// routed through [`IntoCommands::into_commands`](::sand::__private::IntoCommands),
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
            "raw string commands are not accepted directly inside #[function] or #[datapack_component(Tick|Load)]. Use a typed command such as cmd::say(\"hello\"), or use cmd::raw(\"say hello\") for an explicit escape hatch.",
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
                ::sand::__private::IntoCommands::into_commands(#expr)
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
                        ::sand::__private::IntoCommands::into_commands(#inner)
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

/// Registers a Rust function as an exported Minecraft function.
///
/// Write typed command expressions directly in the function body. Sand collects
/// each expression into the generated command list. Use `mcfunction!` only for
/// advanced command grouping or migration code.
///
/// The function is automatically registered via `inventory` at program startup —
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
///         Target::players(),
///         Text::new("Welcome!").gold().bold(true),
///     );
///     cmd::say("Enjoy your stay!");
/// }
/// ```
///
/// # Resource path syntax
///
/// An explicit path override (`#[function("path")]` or
/// `#[function("namespace:path")]`) must follow Minecraft resource-location
/// rules: namespace `[a-z0-9_.-]+`, path `[a-z0-9_./-]+`. Empty strings,
/// uppercase letters, whitespace, and multiple colons are rejected at compile
/// time with a diagnostic pointing at the offending literal.
///
/// The annotated function must not take runtime arguments: its body is compiled
/// as authoring code that produces Minecraft commands during the Sand build.
#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_function_attr(attr);
    let func = parse_macro_input!(item as ItemFn);
    let expected = expected_public_function(&func);

    match attr
        .and_then(|path| expand_function(func, path))
        .and_then(|tokens| validate_preserved_public_surface(&tokens, expected).map(|()| tokens))
    {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Expansion ─────────────────────────────────────────────────────────────────

fn validate_macro_resource_path(lit: &LitStr, context: &str) -> syn::Result<()> {
    let value = lit.value();
    if value.is_empty() {
        return Err(syn::Error::new_spanned(
            lit,
            format!("invalid resource path in {context}: must not be empty"),
        ));
    }
    let colon_count = value.chars().filter(|&c| c == ':').count();
    if colon_count > 1 {
        return Err(syn::Error::new_spanned(
            lit,
            format!(
                "invalid resource path in {context}: multiple colons are not allowed in `{value}`"
            ),
        ));
    }
    if colon_count == 1 {
        let (ns, path) = value.split_once(':').unwrap();
        if ns.is_empty() {
            return Err(syn::Error::new_spanned(
                lit,
                format!(
                    "invalid resource path in {context}: namespace must not be empty in `{value}`"
                ),
            ));
        }
        if !ns
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'))
        {
            return Err(syn::Error::new_spanned(
                lit,
                format!(
                    "invalid resource path in {context}: namespace must only contain [a-z0-9_.-] in `{value}`"
                ),
            ));
        }
        if path.is_empty() {
            return Err(syn::Error::new_spanned(
                lit,
                format!("invalid resource path in {context}: path must not be empty in `{value}`"),
            ));
        }
        if !path
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'))
        {
            return Err(syn::Error::new_spanned(
                lit,
                format!(
                    "invalid resource path in {context}: path must only contain [a-z0-9_./-] in `{value}`"
                ),
            ));
        }
    } else if !value
        .chars()
        .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'))
    {
        return Err(syn::Error::new_spanned(
            lit,
            format!(
                "invalid resource path in {context}: path must only contain [a-z0-9_./-] in `{value}`"
            ),
        ));
    }
    Ok(())
}

fn parse_function_attr(attr: TokenStream) -> syn::Result<Option<String>> {
    if attr.is_empty() {
        return Ok(None);
    }

    let path = syn::parse::<LitStr>(attr)?;
    validate_macro_resource_path(&path, "#[function(...)]")?;
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

        ::sand::__private::inventory::submit!(
            ::sand::__private::FunctionDescriptor {
                path: #fn_name_str,
                make: #factory_ident,
            }
        );

        ::sand::__private::inventory::submit!(
            ::sand::__private::FunctionPointerEntry {
                ptr: #fn_name as fn() -> ::std::vec::Vec<::std::string::String>,
                path: #ptr_path_str,
            }
        );

        ::sand::__private::inventory::submit!(
            ::sand::__private::FunctionPointerTypeEntry {
                type_id: #type_id_ident,
                path: #ptr_path_str,
            }
        );

        ::sand::__private::inventory::submit!(
            ::sand::__private::sand_components::dialog::DialogFunctionPointerEntry {
                ptr: #fn_name as fn() -> ::std::vec::Vec<::std::string::String>,
                path: #ptr_path_str,
            }
        );

        ::sand::__private::inventory::submit!(
            ::sand::__private::sand_components::dialog::DialogFunctionPointerTypeEntry {
                type_id: #type_id_ident,
                path: #ptr_path_str,
            }
        );
    })
}

// ── #[datapack_component] ─────────────────────────────────────────────────────────────

/// Registers a free-standing function as a datapack component.
///
/// ## Plain `#[datapack_component]`
///
/// The function must take no parameters and return a type that implements
/// `sand_core::DatapackComponent`. It is automatically collected via
/// `inventory` — no manual wiring needed.
///
/// ```rust,ignore
/// #[datapack_component]
/// fn player_join() -> sand_core::Advancement {
///     Advancement::new("my_pack:player_join".parse().unwrap())
///         .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
/// }
/// ```
///
/// ## `#[datapack_component(Tick)]` / `#[datapack_component(Load)]`
///
/// The function body becomes an `.mcfunction` file **and** the function is
/// added to `data/minecraft/tags/functions/tick.json` (or `load.json`),
/// making it run every tick / once on load automatically.
///
/// ```rust,ignore
/// #[datapack_component(Tick)]
/// pub fn my_tick() {
///     TIMER.tick(Target::players());
/// }
///
/// #[datapack_component(Load)]
/// pub fn on_load() {
///     TIMER.define();
/// }
/// ```
///
/// ## `#[datapack_component(Tag = "ns:name")]`
///
/// Like `Tick`/`Load` but targets any function tag you choose — useful for
/// hooking into other datapacks' APIs or creating your own tags.
///
/// ```rust,ignore
/// #[datapack_component(Tag = "my_lib:on_player_death")]
/// pub fn handle_death() {
///     cmd::say("player died");
/// }
/// ```
///
/// The tag string must be a valid resource location (`namespace:path` or
/// `path`-only). Namespace must match `[a-z0-9_.-]+`, path must match
/// `[a-z0-9_./-]+`.
#[proc_macro_attribute]
pub fn datapack_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let expected = expected_public_function(&func);
    match parse_component_flag(attr)
        .and_then(|flag| expand_component(func, flag))
        .and_then(|tokens| validate_preserved_public_surface(&tokens, expected).map(|()| tokens))
    {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Component flag parsing ────────────────────────────────────────────────────

enum ComponentFlag {
    /// Plain `#[datapack_component]` — returns a DatapackComponent.
    None,
    /// `#[datapack_component(Tick)]` — registers in `minecraft:tick`.
    Tick,
    /// `#[datapack_component(Load)]` — registers in `minecraft:load`.
    Load,
    /// `#[datapack_component(Tag = "ns:name")]` — registers in a custom function tag.
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
                    validate_macro_resource_path(s, "#[datapack_component(Tag = \"...\")]")?;
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
            "#[datapack_component] cannot be applied to methods — use a free-standing `fn`",
        ));
    }

    // Validate: no parameters
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[datapack_component] functions must take no parameters",
        ));
    }

    match flag {
        ComponentFlag::None => expand_component_plain(func),
        ComponentFlag::Tick => expand_component_tag(func, "minecraft:tick"),
        ComponentFlag::Load => expand_component_tag(func, "minecraft:load"),
        ComponentFlag::Tag(tag) => expand_component_tag(func, &tag),
    }
}

/// Plain `#[datapack_component]` — returns a `DatapackComponent`.
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
        fn #factory_ident() -> ::std::boxed::Box<dyn ::sand::__private::DatapackComponent> {
            ::std::boxed::Box::new(#fn_name())
        }

        ::sand::__private::inventory::submit!(
            ::sand::__private::ComponentFactory { make: #factory_ident }
        );
    })
}

/// `#[datapack_component(Tick)]` / `#[datapack_component(Load)]` / `#[datapack_component(Tag = "...")]` —
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

        ::sand::__private::inventory::submit!(
            ::sand::__private::FunctionDescriptor {
                path: #fn_name_str,
                make: #fn_make_ident,
            }
        );

        ::sand::__private::inventory::submit!(
            ::sand::__private::FunctionTagDescriptor {
                tag: #tag_lit,
                function_path: #fn_name_str,
            }
        );
    })
}

// ── #[on_event] ─────────────────────────────────────────────────────────────────

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
/// use sand_macros::on_event;
///
/// pub struct AteGoldenApple;
///
/// impl AdvancementEvent for AteGoldenApple {
///     type Trigger = ConsumeItemTrigger;
///
///     fn trigger() -> Self::Trigger {
///         ConsumeItemTrigger::new()
///             .item(ItemPredicate::id(ItemId::minecraft("golden_apple")?))
///     }
/// }
///
/// #[on_event]
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
/// use sand_macros::on_event;
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
/// #[on_event]
/// pub fn on_jump(_event: PlayerJumped) {
///     cmd::say("Jumped!");
/// }
/// ```
///
/// `SandEventDispatch::chain::<Parent>()` also supports implemented
/// same-cycle parent-to-child dispatch. It reuses the parent's detector and
/// preserves that cycle's player and position. Chained children can add an
/// explicit persistent condition with `.while_::<E>()`. Typed two- through
/// eight-parent `after_any::<(...)>()` and `after_all::<(...)>()` clauses are
/// also supported and coalesced per subject. Bounded `.within(...)`
/// correlation, advancement-backed graph parents, and participant-rich
/// contexts remain planned; they are not accepted by this macro today.
///
/// Generic `SandEvent` definitions are supported and each concrete
/// monomorphization has distinct dispatch identity. A `#[on_event]` handler must
/// still name a constructible concrete marker parameter; use a unit adapter
/// type when the generic definition itself stores `PhantomData`.
///
/// # Attributes
///
/// `#[on_event]` takes exactly one handler parameter. Flat attributes such as
/// `id = "namespace:path"`, `slot = Head`, `item = "minecraft:stick"`,
/// and `custom_data = "{key:1b}"` are supported where the selected event
/// family uses them. `dispatch = "advancement"` is retained only as a
/// compatibility selector for older unit-style advancement handlers; new
/// advancement events should use `Event<T>`.
///
/// Reset behavior belongs to `AdvancementEvent::reset()` (or the compatibility
/// `SandEvent::revoke()` hook), not to an event attribute.
#[proc_macro_attribute]
pub fn on_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let expected = expected_public_function(&func);
    match expand_event(attr, func)
        .and_then(|tokens| validate_preserved_public_surface(&tokens, expected).map(|()| tokens))
    {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ── New event attribute ───────────────────────────────────────────────────────

/// Flat key=value attributes for the new-style `#[on_event]` macro.
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
                            "unknown #[on_event] filter `{other}`; \
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

/// Built-in event type names (#49) that reach the shared tracked-transition
/// provider backend through `SandEvent::dispatch()` rather than
/// `AdvancementEvent`, even when used as `Event<T>`.
///
/// Matches the *outer* type name only — generic markers like
/// `EffectStarted<Speed>` are covered by their base name (`EffectStarted`)
/// since the macro's `event_type_name` extraction discards generic
/// arguments; the actual dispatch resolution still sees the fully
/// monomorphized type through `dispatch_type_tokens`.
fn is_tracked_provider_event_type(name: &str) -> bool {
    matches!(
        name,
        "PlayerStartSprintingEvent"
            | "PlayerStopSprintingEvent"
            | "PlayerStartSwimmingEvent"
            | "PlayerStopSwimmingEvent"
            | "PlayerStartFlyingEvent"
            | "PlayerStopFlyingEvent"
            | "PlayerCaughtFireEvent"
            | "PlayerExtinguishedEvent"
            | "PlayerEnteredSurvivalEvent"
            | "PlayerExitedSurvivalEvent"
            | "PlayerEnteredCreativeEvent"
            | "PlayerExitedCreativeEvent"
            | "PlayerEnteredAdventureEvent"
            | "PlayerExitedAdventureEvent"
            | "PlayerEnteredSpectatorEvent"
            | "PlayerExitedSpectatorEvent"
            | "PlayerHealthChangedEvent"
            | "PlayerHealthLostEvent"
            | "PlayerHealthGainedEvent"
            | "PlayerLowHealthEvent"
            | "PlayerRecoveredHealthEvent"
            | "EffectStarted"
            | "EffectStopped"
    )
}

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
            "#[on_event] cannot be applied to methods — use a free-standing `fn`",
        ));
    }

    // Exactly one typed parameter required.
    if func.sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "#[on_event] functions must take exactly one parameter: the event type \
             (e.g. `event: OnJoinEvent`)",
        ));
    }
    let event_binding_pattern = match func.sig.inputs.first().expect("one event parameter") {
        syn::FnArg::Typed(parameter) => parameter.pat.as_ref(),
        syn::FnArg::Receiver(receiver) => {
            return Err(syn::Error::new_spanned(
                receiver,
                "#[on_event] cannot be a method",
            ));
        }
    };

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
                "#[on_event] generic handlers must specify the event type, e.g. `Event<MyEvent>` or `DamageEvent<MyDamageEvent>`",
            ));
        };

        if args.args.len() != 1 {
            return Err(syn::Error::new_spanned(
                args,
                "#[on_event] generic handlers must use exactly one generic argument, e.g. `Event<MyEvent>`",
            ));
        }

        let Some(syn::GenericArgument::Type(event_ty)) = args.args.first() else {
            return Err(syn::Error::new_spanned(
                args,
                "#[on_event] generic handlers must use a type argument, e.g. `Event<MyEvent>`",
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
                                ::sand::__private::event::DamageEvent::<#event_type_tokens>::context()
                            }
                        } else {
                            quote! {
                                ::sand::__private::event::Event::<#event_type_tokens>::context()
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
                        "#[on_event] parameter type must be a path (e.g. `OnJoinEvent`)",
                    ));
                }
            },
            syn::FnArg::Receiver(r) => {
                return Err(syn::Error::new_spanned(r, "#[on_event] cannot be a method"));
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
    // Preserve the author's parameter binding while removing it from the
    // generated zero-argument Minecraft function.
    let preamble = quote! {
        #(#fn_attrs)*
        #[allow(unused_variables)]
        #vis fn #fn_name() -> ::std::vec::Vec<::std::string::String> {
            let #event_binding_pattern = #event_binding_tokens;
            #body
        }

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #fn_make_ident() -> ::std::vec::Vec<::std::string::String> {
            #fn_name()
        }
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Map a slot ident string to `::sand::__private::ArmorSlot::*` tokens.
    fn slot_to_armor_slot_tokens(slot: &syn::Ident) -> syn::Result<proc_macro2::TokenStream> {
        match slot.to_string().as_str() {
            "Head" | "Chest" | "Legs" | "Feet" | "Offhand" => {
                Ok(quote! { ::sand::__private::ArmorSlot::#slot })
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::Tracked(
                        ::sand::__private::TrackedTransition::new(
                            "player_sneaking",
                            ::sand::__private::player_sneaking_tracked_source(),
                            ::sand::__private::TransitionKind::BecameTrue,
                        )
                    ),
                });
            }
        }

        "PlayerStopSneakingEvent" | "PlayerStopsSneaking" => {
            quote! {
                #preamble

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::Tracked(
                        ::sand::__private::TrackedTransition::new(
                            "player_sneaking",
                            ::sand::__private::player_sneaking_tracked_source(),
                            ::sand::__private::TransitionKind::BecameFalse,
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::JoinTick,
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
                fn #trigger_ident() -> ::sand::__private::AdvancementTrigger {
                    ::sand::__private::AdvancementTrigger::Tick
                }

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::Advancement {
                        make_trigger: #trigger_ident,
                        revoke: (|| false) as fn() -> bool,
                        guard: ::std::option::Option::None,
                        make_participants: (|| ::sand::__private::participant::EventParticipantPlan::none())
                            as fn() -> ::sand::__private::participant::EventParticipantPlan,
                        event_type_name: (|| ::std::any::type_name::<#dispatch_type_tokens>())
                            as fn() -> &'static str,
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::XpLevelUp,
                });
            }
        }

        // OnDeathEvent / OnDeath — deathCount scoreboard tick loop
        "OnDeathEvent" | "OnDeath" => {
            quote! {
                #preamble

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::DeathTick,
                });
            }
        }

        // OnRespawnEvent / OnRespawn — phase + time_since_death tick lifecycle
        "OnRespawnEvent" | "OnRespawn" => {
            quote! {
                #preamble

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::RespawnTick,
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::ArmorEquip {
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::ArmorUnequip {
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::TickPoll {
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

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::TickPoll {
                        make_condition: #cond_ident,
                    },
                });
            }
        }

        // Event<T> and compatibility `dispatch = "advancement"` handlers use the
        // typed AdvancementEvent path. This path never emits legacy string guards.
        //
        // Built-in tracked-provider event types (#49) are also reached as
        // `Event<T>` (see `sand-core/src/event/mod.rs`'s `impl<E> Event<E>`,
        // which is intentionally unbounded — shared by advancement-backed
        // and generated tracked events) but must NOT take this
        // `AdvancementEvent`-only path, including generic ones like
        // `EffectStarted<Speed>` where `event_type_name` only captures the
        // outer generic's name. `is_tracked_provider_event_type` excludes
        // them so they fall through to the generic `SandEvent` dispatch arm
        // below, which resolves dispatch via `SandEvent::dispatch()`
        // (including `SandEventDispatch::Tracked`) instead of requiring
        // `AdvancementEvent`.
        _ if (is_event_context
            || flat_attr.dispatch.as_ref().map(|s| s.value()).as_deref()
                == Some("advancement"))
            && !is_tracked_provider_event_type(&event_type_name) =>
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
            let participants_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_participants", fn_name),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #trigger_ident() -> ::sand::__private::AdvancementTrigger {
                    <#dispatch_type_tokens as ::sand::__private::event::AdvancementEvent>::trigger().into()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #revoke_ident() -> bool {
                    <#dispatch_type_tokens as ::sand::__private::event::AdvancementEvent>::reset().should_revoke()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #guard_ident() -> ::std::option::Option<::sand::__private::condition::Condition> {
                    <#dispatch_type_tokens as ::sand::__private::event::AdvancementEvent>::guard()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #participants_ident() -> ::sand::__private::participant::EventParticipantPlan {
                    <#dispatch_type_tokens as ::sand::__private::event::AdvancementEvent>::participants()
                }

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::Advancement {
                        make_trigger: #trigger_ident,
                        revoke: #revoke_ident,
                        guard: ::std::option::Option::Some(#guard_ident),
                        make_participants: #participants_ident,
                        event_type_name: (|| ::std::any::type_name::<#dispatch_type_tokens>())
                            as fn() -> &'static str,
                    },
                });

                // Register event type → handler path mapping for EventHandle<E>.revoke/grant.
                ::sand::__private::inventory::submit!(::sand::__private::EventPathEntry {
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
            let tracked_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_tracked", fn_name),
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
            let participants_ident = proc_macro2::Ident::new(
                &format!("__sand_event_{}_participants", fn_name),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #preamble

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #trigger_ident() -> ::std::option::Option<::sand::__private::AdvancementTrigger> {
                    let dispatch: ::sand::__private::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::dispatch().into();
                    ::sand::__private::event_dispatch_advancement(dispatch)
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #cond_ident() -> ::std::option::Option<::std::string::String> {
                    let dispatch: ::sand::__private::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::dispatch().into();
                    ::sand::__private::event_dispatch_tick_condition(dispatch)
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #tick_ident() -> ::std::option::Option<::sand::__private::events::TickEventDispatch> {
                    let dispatch: ::sand::__private::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::dispatch().into();
                    ::sand::__private::event_dispatch_tick(dispatch)
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #chain_ident() -> ::std::option::Option<::sand::__private::events::ChainEventDispatch> {
                    let dispatch: ::sand::__private::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::dispatch().into();
                    ::sand::__private::event_dispatch_chain(dispatch)
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #tracked_ident() -> ::std::option::Option<::sand::__private::TrackedTransition> {
                    let dispatch: ::sand::__private::events::SandEventDispatch =
                        <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::dispatch().into();
                    ::sand::__private::event_dispatch_tracked(dispatch)
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #revoke_ident() -> bool {
                    <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::revoke()
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
                fn #setup_ident() -> ::sand::__private::events::EventSetup {
                    <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::setup()
                }

                #[doc(hidden)]
                #[allow(dead_code)]
                fn #participants_ident() -> ::sand::__private::participant::EventParticipantPlan {
                    <#dispatch_type_tokens as ::sand::__private::events::SandEvent>::participants()
                }

                ::sand::__private::inventory::submit!(::sand::__private::EventDescriptor {
                    path: #fn_name_str,
                    id_override: #id_override_tokens,
                    make: #fn_make_ident,
                    dispatch: ::sand::__private::EventDispatch::Custom {
                        make_trigger: #trigger_ident,
                        make_condition: #cond_ident,
                        make_tick: #tick_ident,
                        make_chain: #chain_ident,
                        make_tracked: #tracked_ident,
                        make_participants: #participants_ident,
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
///         .as_(Target::players())
///         .run(run_fn!("hello_world:greet" {
///             cmd::say("Welcome!");
///             VISITS.add(Target::self_(), 1);
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
///     .as_(Target::players())
///     .run(run_fn!({
///         cmd::say("One-off greeting!");
///     }));
/// ```
///
/// # Without body — shorthand for `cmd::function(...)`
///
/// ```rust,ignore
/// Execute::new()
///     .as_(Target::players())
///     .run(run_fn!("hello_world:on_player_join"))
/// ```
///
/// The name string must be a valid resource location: either a full
/// `namespace:path`, or a bare `path`, which is resolved against the pack's
/// `[pack].namespace` from `sand.toml` at compile time — the same mechanism
/// the anonymous form above uses to pick a namespace. Namespace must match
/// `[a-z0-9_.-]+`, path must match `[a-z0-9_./-]+`. Invalid strings, and a
/// bare `path` with no resolvable `sand.toml` namespace, are rejected at
/// compile time — never at runtime.
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
/// The macro submits a `sand_resourcepack::RawTexture` descriptor via
/// `inventory::submit!` at link time.
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
    match expand_texture(input)
        .and_then(|tokens| validate_preserved_public_surface(&tokens, None).map(|()| tokens))
    {
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
    let handle_contract = generated_api_contract(
        handle_ident.to_string(),
        GeneratedApiKind::Constant,
        format!("Controls the `{name_str}` generated HUD bar."),
        "The generated handle selects frames from the registered bitmap-font bar and builds display commands.",
        "At resource-pack export, Sand writes the bar texture and font provider; datapack commands render its assigned glyphs.",
        &["Render or update this named HUD bar from author code."],
        &["Do not guess the assigned Unicode glyphs or rebuild the font component manually."],
        &[],
        Some("A BarHandle configured from this hud_bar invocation."),
        format!("{handle_ident}.show(\"@a\", 0, \"my_pack\");"),
    );
    let handle_docs = generated_api_contract_docs(&handle_contract);

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

        let expansion = quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand::__private::rp::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand::__private::rp::GenHudBar {
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

            #handle_docs
            pub const #handle_ident: ::sand::__private::rp::BarHandle = ::sand::__private::rp::BarHandle {
                name:        #name,
                steps:       #steps,
                font:        #font_ts,
                frame_width: #effective_fw_lit,
            };

            ::sand::__private::rp::__private::inventory::submit!(
                ::sand::__private::rp::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        };
        validate_generated_expansion(
            expansion.clone(),
            std::iter::empty(),
            std::slice::from_ref(&handle_contract),
        )?;
        Ok(expansion)
    } else {
        let texture = require_lit_str(&fields, "texture", "hud_bar")?;

        let expansion = quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand::__private::rp::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand::__private::rp::HudBar {
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

            #handle_docs
            pub const #handle_ident: ::sand::__private::rp::BarHandle = ::sand::__private::rp::BarHandle {
                name:        #name,
                steps:       #steps,
                font:        #font_ts,
                frame_width: 0u32,  // unknown for user-supplied PNGs
            };

            ::sand::__private::rp::__private::inventory::submit!(
                ::sand::__private::rp::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        };
        validate_generated_expansion(
            expansion.clone(),
            std::iter::empty(),
            std::slice::from_ref(&handle_contract),
        )?;
        Ok(expansion)
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
    let handle_contract = generated_api_contract(
        handle_ident.to_string(),
        GeneratedApiKind::Constant,
        format!("Controls the `{name_str}` generated HUD element."),
        "The generated handle represents one bitmap-font glyph and builds commands that display it.",
        "At resource-pack export, Sand writes the element texture and font provider; datapack commands render its assigned glyph.",
        &["Render this named HUD element from author code."],
        &["Do not guess the assigned Unicode glyph or rebuild the font component manually."],
        &[],
        Some("An ElementHandle configured from this hud_element invocation."),
        format!("{handle_ident}.show(\"@a\", \"my_pack\");"),
    );
    let handle_docs = generated_api_contract_docs(&handle_contract);

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

        let expansion = quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand::__private::rp::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand::__private::rp::GenHudElement {
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

            #handle_docs
            pub const #handle_ident: ::sand::__private::rp::ElementHandle = ::sand::__private::rp::ElementHandle {
                name:       #name,
                font:       #font_ts,
                char_width: #effective_cw_lit,
            };

            ::sand::__private::rp::__private::inventory::submit!(
                ::sand::__private::rp::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        };
        validate_generated_expansion(
            expansion.clone(),
            std::iter::empty(),
            std::slice::from_ref(&handle_contract),
        )?;
        Ok(expansion)
    } else {
        let texture = require_lit_str(&fields, "texture", "hud_element")?;

        let expansion = quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn #factory_ident() -> ::std::boxed::Box<dyn ::sand::__private::rp::ResourcePackComponent> {
                ::std::boxed::Box::new(::sand::__private::rp::HudElement {
                    name:         #name,
                    texture_src:  #texture,
                    texture_dest: #tex_dest_ts,
                    unicode:      #unicode_ts,
                    height:       #height,
                    ascent:       #ascent,
                    font:         #font_ts,
                })
            }

            #handle_docs
            pub const #handle_ident: ::sand::__private::rp::ElementHandle = ::sand::__private::rp::ElementHandle {
                name:       #name,
                font:       #font_ts,
                char_width: 0u32,  // unknown for user-supplied PNGs
            };

            ::sand::__private::rp::__private::inventory::submit!(
                ::sand::__private::rp::ResourcePackDescriptor {
                    name: #name,
                    make: #factory_ident,
                }
            );
        };
        validate_generated_expansion(
            expansion.clone(),
            std::iter::empty(),
            std::slice::from_ref(&handle_contract),
        )?;
        Ok(expansion)
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
        fn #factory_ident() -> ::std::boxed::Box<dyn ::sand::__private::rp::ResourcePackComponent> {
            ::std::boxed::Box::new(::sand::__private::rp::RawTexture {
                name:            #id,
                asset_namespace: #asset_ns_lit,
                dest_path:       #dest_path_lit,
                src_path:        #path,
            })
        }

        ::sand::__private::rp::__private::inventory::submit!(
            ::sand::__private::rp::ResourcePackDescriptor {
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
///     MANA_REGEN_BOOST.enable(Target::self_());
/// }
///
/// #[armor_event(Unequip, slot = Feet, item = "minecraft:leather_boots",
///               custom_data = "{mana_boots:true}")]
/// pub fn on_mana_boots_unequip() {
///     MANA_REGEN_BOOST.disable(Target::self_());
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
    let expected = expected_public_function(&func);

    match expand_armor_event(parsed_attr, func)
        .and_then(|tokens| validate_preserved_public_surface(&tokens, expected).map(|()| tokens))
    {
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

    // Map slot ident to ::sand::__private::ArmorSlot::*
    let slot_ident = &attr.slot_ident;
    let slot_expr = quote! { ::sand::__private::ArmorSlot::#slot_ident };

    // Map kind ident to ::sand::__private::ArmorEventKind::*
    let kind_ident = &attr.kind_ident;
    let kind_expr = quote! { ::sand::__private::ArmorEventKind::#kind_ident };

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

        ::sand::__private::inventory::submit!(::sand::__private::ArmorEventDescriptor {
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

/// Validates a `[pack].namespace` value read from `sand.toml` against the
/// same character rules `ResourceLocation` enforces, so a malformed config
/// value is reported at macro-expansion time instead of reaching an
/// `.expect()` in generated code.
fn validate_pack_namespace(ns: &str, call_site: proc_macro2::Span) -> syn::Result<()> {
    if ns.is_empty()
        || !ns
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'))
    {
        return Err(syn::Error::new(
            call_site,
            format!(
                "[pack].namespace `{ns}` in sand.toml is not a valid resource-location \
                 namespace ([a-z0-9_.-]+)"
            ),
        ));
    }
    Ok(())
}

fn expand_run_fn(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let RunFnInput { name, body } = syn::parse::<RunFnInput>(input)?;

    // Resolve the full (namespace, path) resource location. A path-only
    // named input (no `:`) is resolved against the pack namespace exactly
    // like the anonymous form below, rather than being handed to
    // `ResourceLocation`'s runtime parser — that parser requires
    // `namespace:path` and would panic on a bare path.
    let (ns_val, path_val, span) = match &name {
        Some(lit) => {
            validate_macro_resource_path(lit, "run_fn!(\"...\")")?;
            let raw = lit.value();
            match raw.split_once(':') {
                Some((ns, path)) => (ns.to_string(), path.to_string(), lit.span()),
                None => {
                    let ns = read_sand_namespace().ok_or_else(|| {
                        syn::Error::new_spanned(
                            lit,
                            format!(
                                "run_fn!(\"{raw}\") is path-only and requires [pack].namespace \
                                 in sand.toml to resolve to a full resource location; provide \
                                 an explicit `namespace:path` name instead, or add sand.toml"
                            ),
                        )
                    })?;
                    validate_pack_namespace(&ns, lit.span())?;
                    (ns, raw, lit.span())
                }
            }
        }
        None => {
            // Anonymous: read namespace from sand.toml, generate unique path.
            let ns = read_sand_namespace().ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "could not read [pack].namespace from sand.toml; \
                     provide an explicit name or ensure sand.toml exists",
                )
            })?;
            validate_pack_namespace(&ns, proc_macro2::Span::call_site())?;
            let id = ANON_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let anon_path = format!("__anon/fn_{id}");
            (ns, anon_path, proc_macro2::Span::call_site())
        }
    };

    let path_part = path_val.as_str();
    let ns_lit = LitStr::new(&ns_val, span);
    let full_path_lit = LitStr::new(&path_val, span);
    let fn_call = quote! {
        ::sand::__private::cmd::function(
            ::sand::__private::ResourceLocation::new(#ns_lit, #full_path_lit)
                .expect("run_fn! validates namespace/path syntax at compile time")
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

                    ::sand::__private::inventory::submit!(
                        ::sand::__private::FunctionDescriptor {
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
                    ::sand::__private::register_dyn_fn(
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
    let expected = expected_public_function(&func);
    match parse_schedule_attr(attr)
        .and_then(|sa| expand_schedule(func, sa))
        .and_then(|tokens| validate_preserved_public_surface(&tokens, expected).map(|()| tokens))
    {
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

        ::sand::__private::inventory::submit!(::sand::__private::ScheduleDescriptor {
            path: #fn_name_str,
            total_ticks: #total_ticks,
            every: #every,
            make: #fn_make_ident,
        });
    })
}

// ── #[custom_item] ───────────────────────────────────────────────────────────────────

/// Generate a typed item struct from a `CustomItem`-producing function.
///
/// Reads `CustomItem::new("base_id")` and `.custom_data("key")` directly from
/// the function body — no duplication needed. Generates a unit struct with
/// `BASE`, `PREDICATE`, and an `item()` method that calls the original function.
///
/// The struct name is derived automatically from the `custom_data` key
/// (converted to PascalCase). Override it with `#[custom_item(name = "MyName")]`.
/// If there is no `custom_data` call, `name` is required.
///
/// # Examples
///
/// ```rust,ignore
/// // Struct name "ManaBoots" derived from custom_data key "mana_boots"
/// #[custom_item]
/// pub fn mana_boots() -> CustomItem {
///     CustomItem::new("minecraft:leather_boots")
///         .custom_data("mana_boots")
///         .display_name("Mana Boots")
/// }
///
/// // No custom_data — must provide name
/// #[custom_item(name = "ShardBlade")]
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
///     .as_(Target::players())
///     .at(Target::self_())
///     .if_items_entity(Target::self_(), ItemSlot::Feet, ManaBoots::PREDICATE)
///     .run_fn("ns:on_mana_boots_tick");
/// ```
#[proc_macro_attribute]
pub fn custom_item(attr: TokenStream, input: TokenStream) -> TokenStream {
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

/// Parse the attr tokens for `#[custom_item(...)]`.
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
                            "unknown #[custom_item] parameter `{other}`; \
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
            "#[custom_item] could not find `CustomItem::new(\"minecraft:...\")` in the function body. \
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
            "#[custom_item] could not find a `.custom_data(\"key\")` call to derive the struct name. \
             Either add `.custom_data(\"your_key\")` to uniquely identify this item, or \
             specify an explicit name with `#[custom_item(name = \"YourName\")]`.",
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

    let custom_data_contracts = if custom_data.is_some() {
        vec![
            generated_api_contract(
                format!("{struct_ident}::CUSTOM_DATA_KEY"),
                GeneratedApiKind::AssociatedConst,
                "Names the custom-data marker that uniquely identifies this item.",
                "The generated item reference keeps its identity marker available for integrations that need the raw key.",
                "This is the key stored with byte value `1b` in the item's `minecraft:custom_data` component.",
                &[
                    "Refer to the marker key when integrating with APIs that cannot consume the complete item predicate.",
                ],
                &[
                    "Use PREDICATE for normal equipment tests instead of rebuilding the component predicate.",
                ],
                &[],
                Some("The unqualified custom-data key."),
                format!("let key = {struct_ident}::CUSTOM_DATA_KEY;"),
            ),
            generated_api_contract(
                format!("{struct_ident}::CUSTOM_DATA_SNBT"),
                GeneratedApiKind::AssociatedConst,
                "Provides the custom-data marker in SNBT object form.",
                "Armor-event filters accept this representation when selecting the generated custom item.",
                "The SNBT object stores the generated marker key with byte value `1b`.",
                &["Pass the marker to an API that explicitly requires custom-data SNBT."],
                &["Do not use this fragment as a complete item predicate; use PREDICATE instead."],
                &[],
                Some("An SNBT compound fragment containing the marker."),
                format!("let marker = {struct_ident}::CUSTOM_DATA_SNBT;"),
            ),
        ]
    } else {
        Vec::new()
    };
    let custom_data_const = if let Some(ref key) = custom_data {
        let snbt = format!("{{{key}:1b}}");
        let key_docs = generated_api_contract_docs(&custom_data_contracts[0]);
        let snbt_docs = generated_api_contract_docs(&custom_data_contracts[1]);
        quote! {
            #key_docs
            pub const CUSTOM_DATA_KEY: &'static str = #key;

            #snbt_docs
            pub const CUSTOM_DATA_SNBT: &'static str = #snbt;
        }
    } else {
        quote! {}
    };

    // ── User-defined data consts ──────────────────────────────────────────────
    let data_contracts = item_attr
        .data
        .iter()
        .map(|c| {
            let const_name = &c.name;
            generated_api_contract(
                format!("{struct_ident}::{const_name}"),
                GeneratedApiKind::AssociatedConst,
                format!("Exposes the author-declared `{const_name}` metadata for this custom item."),
                "This constant is declared in the custom_item attribute and names item-specific author metadata.",
                "Sand does not interpret this value; it remains available to the datapack's Rust authoring code.",
                &["Share item-specific immutable metadata with code that uses the generated item reference."],
                &["Do not treat the value as Sand-validated Minecraft data unless its declared type provides that validation."],
                &[],
                Some("The value and type supplied in the custom_item data declaration."),
                format!("let value = {struct_ident}::{const_name};"),
            )
        })
        .collect::<Vec<_>>();
    let data_consts = item_attr
        .data
        .iter()
        .zip(&data_contracts)
        .map(|(c, contract)| {
            let const_name = &c.name;
            let ty = &c.ty;
            let val = &c.value;
            let docs = generated_api_contract_docs(contract);
            quote! { #docs pub const #const_name: #ty = #val; }
        })
        .collect::<Vec<_>>();

    let mut contracts = vec![
        generated_api_contract(
            struct_ident.to_string(),
            GeneratedApiKind::Struct,
            format!("Identifies the `{struct_ident}` custom item in author code."),
            "The generated zero-sized type groups the item's canonical predicate, construction helper, and item-specific metadata.",
            format!(
                "Its predicate selects the Minecraft base item `{base}` together with this definition's custom-data marker."
            ),
            &[
                "Reference this custom item from equipment conditions, events, or code that needs its definition.",
            ],
            &[
                "Do not construct raw predicates when the generated reference already represents the item.",
            ],
            &[],
            None,
            format!("let item = {struct_ident}::item();"),
        ),
        generated_api_contract(
            format!("{struct_ident}::BASE"),
            GeneratedApiKind::AssociatedConst,
            "Names the vanilla item ID used as this custom item's base.",
            "The custom item definition adds components and identity data to this base item.",
            "The value is the Minecraft resource location supplied to CustomItem::new by the annotated factory.",
            &["Inspect or reuse the base identifier without constructing the complete item."],
            &[
                "Use PREDICATE when testing for the custom item, because BASE alone does not distinguish it.",
            ],
            &[],
            Some("The base item resource location as a string."),
            format!("let base = {struct_ident}::BASE;"),
        ),
        generated_api_contract(
            format!("{struct_ident}::PREDICATE"),
            GeneratedApiKind::AssociatedConst,
            "Provides the complete item-stack predicate for this custom item.",
            "The predicate combines the base item with the generated identity marker when one is configured.",
            "It is formatted for Minecraft `execute if items` and `execute unless items` matching.",
            &["Test whether an entity or container slot contains this exact custom item."],
            &["Do not use BASE alone when another item can share the same vanilla base."],
            &[],
            Some("A Minecraft item predicate string."),
            format!("let predicate = {struct_ident}::PREDICATE;"),
        ),
        generated_api_contract(
            format!("{struct_ident}::if_wearing"),
            GeneratedApiKind::Method,
            "Builds a command that runs only while the current entity wears this item.",
            "This helper applies the generated item predicate to one equipment slot on `@s`.",
            "It emits `execute if items entity @s <slot> <predicate> run <command>`.",
            &[
                "Condition one command on the current entity wearing this custom item in a known slot.",
            ],
            &[
                "Use a selector-aware command builder when the tested entity is not the current executor.",
            ],
            &[
                (
                    "slot",
                    "The equipment slot to inspect on the current entity.",
                ),
                ("cmd", "The command to run when the item predicate matches."),
            ],
            Some("The rendered Minecraft execute command."),
            format!("let command = {struct_ident}::if_wearing(ItemSlot::Feet, \"say equipped\");"),
        ),
        generated_api_contract(
            format!("{struct_ident}::unless_wearing"),
            GeneratedApiKind::Method,
            "Builds a command that runs only while the current entity does not wear this item.",
            "This helper negates the generated item predicate for one equipment slot on `@s`.",
            "It emits `execute unless items entity @s <slot> <predicate> run <command>`.",
            &[
                "Condition one command on the current entity not wearing this custom item in a known slot.",
            ],
            &["Use if_wearing when the command should run on a positive match."],
            &[
                (
                    "slot",
                    "The equipment slot to inspect on the current entity.",
                ),
                (
                    "cmd",
                    "The command to run when the item predicate does not match.",
                ),
            ],
            Some("The rendered Minecraft execute command."),
            format!(
                "let command = {struct_ident}::unless_wearing(ItemSlot::Feet, \"say missing\");"
            ),
        ),
        generated_api_contract(
            format!("{struct_ident}::item"),
            GeneratedApiKind::Method,
            "Constructs this custom item's complete Sand definition.",
            "The helper invokes the annotated factory that owns the item's components and identity data.",
            "The returned definition serializes to the configured Minecraft item stack when used by Sand commands or components.",
            &["Obtain the reusable typed definition of this custom item."],
            &["Use PREDICATE instead when only an item-stack condition is needed."],
            &[],
            Some("The CustomItem value produced by the annotated factory."),
            format!("let item = {struct_ident}::item();"),
        ),
    ];
    contracts.extend(custom_data_contracts);
    contracts.extend(data_contracts);

    let struct_docs = generated_api_contract_docs(&contracts[0]);
    let base_docs = generated_api_contract_docs(&contracts[1]);
    let predicate_docs = generated_api_contract_docs(&contracts[2]);
    let if_wearing_docs = generated_api_contract_docs(&contracts[3]);
    let unless_wearing_docs = generated_api_contract_docs(&contracts[4]);
    let item_docs = generated_api_contract_docs(&contracts[5]);

    let generated = quote! {

        #struct_docs
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #vis struct #struct_ident;

        impl #struct_ident {
            #base_docs
            pub const BASE: &'static str = #base;

            #predicate_docs
            pub const PREDICATE: &'static str = #predicate_lit;

            #custom_data_const

            #(#data_consts)*

            #if_wearing_docs
            pub fn if_wearing(
                slot: ::sand::__private::cmd::ItemSlot,
                cmd: impl ::std::fmt::Display,
            ) -> ::std::string::String {
                ::std::format!(
                    "execute if items entity @s {slot} {} run {cmd}",
                    Self::PREDICATE,
                )
            }

            #unless_wearing_docs
            pub fn unless_wearing(
                slot: ::sand::__private::cmd::ItemSlot,
                cmd: impl ::std::fmt::Display,
            ) -> ::std::string::String {
                ::std::format!(
                    "execute unless items entity @s {slot} {} run {cmd}",
                    Self::PREDICATE,
                )
            }

            #item_docs
            pub fn item() -> ::sand::__private::CustomItem {
                #fn_ident()
            }
        }
    };

    if matches!(vis, syn::Visibility::Public(_)) {
        validate_generated_expansion(generated.clone(), std::iter::empty(), &contracts)?;
    }

    Ok(quote! {
        #(#fn_attrs)*
        #func
        #generated
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
/// - Field types are phantom markers only — the struct is never
///   constructed, only its generated `SCHEMA`/per-field accessors are used.
///   Add `#[allow(dead_code)]` to the struct to silence the resulting
///   "field is never read" warning.
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
    let generated_member_names =
        sand_api_contract::syntax::sand_storage_generated_member_names(&input)?;
    let schema_ident = syn::Ident::new(&generated_member_names[0], struct_name.span());

    // ── Build field accessor methods ─────────────────────────────────────────
    let mut methods = Vec::new();
    let mut contracts = Vec::new();

    for (field, generated_name) in fields.iter().zip(generated_member_names.iter().skip(1)) {
        let field_ident = syn::Ident::new(
            generated_name,
            field.ident.as_ref().expect("named field has ident").span(),
        );
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

        let contract = generated_api_contract(
            format!("{struct_name}::{field_ident}"),
            GeneratedApiKind::Method,
            format!("Returns the typed storage field for `{key_str}`."),
            "This accessor belongs to the derived storage schema and preserves the Rust field type while selecting its configured NBT path.",
            format!(
                "Commands built from this field address `{key_str}` below the schema root in Minecraft command storage."
            ),
            &["Read, write, or modify this schema field through Sand's typed storage API."],
            &[
                "Avoid raw data commands when the generated typed field expresses the same operation.",
            ],
            &[],
            Some("A typed storage-field handle bound to this schema and field value type."),
            format!("let field = {struct_name}::{field_ident}();"),
        );
        let docs = generated_api_contract_docs(&contract);
        contracts.push(contract);
        methods.push(quote! {
            #docs
            pub fn #field_ident() -> ::sand::__private::state::StorageField<#struct_name, #field_ty> {
                Self::#schema_ident.field(#key_str)
            }
        });
    }

    let storage_lit = storage.as_str();
    let root_lit = root.as_str();

    let schema_contract = generated_api_contract(
        format!("{struct_name}::{schema_ident}"),
        GeneratedApiKind::AssociatedConst,
        "Describes the Minecraft command-storage location owned by this schema.",
        "The derived schema constant is the canonical root used by all generated field accessors.",
        format!("It addresses storage `{storage_lit}` below NBT root `{root_lit}`."),
        &[
            "Pass the whole schema to APIs that operate on its root, or use a generated field accessor for one value.",
        ],
        &["Avoid duplicating the storage ID or root as raw strings in author code."],
        &[],
        Some("A typed storage-schema descriptor for the derived Rust type."),
        format!("let schema = {struct_name}::{schema_ident};"),
    );
    let schema_docs = generated_api_contract_docs(&schema_contract);
    contracts.insert(0, schema_contract);

    let expanded = quote! {
        impl #struct_name {
            #schema_docs
            pub const #schema_ident: ::sand::__private::state::StorageSchema<#struct_name> =
                ::sand::__private::state::StorageSchema::new(#storage_lit, #root_lit);

            #( #methods )*
        }
    };

    validate_generated_expansion(expanded.clone(), [struct_name.to_string()], &contracts)?;

    Ok(expanded.into())
}

#[allow(clippy::too_many_arguments)]
fn generated_api_contract(
    target: String,
    kind: GeneratedApiKind,
    summary: impl Into<String>,
    context: impl Into<String>,
    minecraft: impl Into<String>,
    use_when: &[&str],
    avoid_when: &[&str],
    parameters: &[(&str, &str)],
    returns: Option<&str>,
    example: String,
) -> GeneratedApiContract {
    GeneratedApiContract {
        target,
        kind,
        summary: summary.into(),
        context: context.into(),
        minecraft: minecraft.into(),
        use_when: use_when.iter().map(|value| (*value).to_owned()).collect(),
        avoid_when: avoid_when.iter().map(|value| (*value).to_owned()).collect(),
        parameters: parameters
            .iter()
            .map(|(name, description)| ((*name).to_owned(), (*description).to_owned()))
            .collect(),
        returns: returns.map(ToOwned::to_owned),
        example,
    }
}

fn generated_api_contract_docs(contract: &GeneratedApiContract) -> proc_macro2::TokenStream {
    let lines = render_generated_rustdoc(contract);
    let lines = lines
        .iter()
        .map(|line| LitStr::new(line, proc_macro2::Span::call_site()));
    quote!(#(#[doc = #lines])*)
}

#[cfg(test)]
mod consumer_surface_policy_tests {
    use super::validate_preserved_public_surface;
    use quote::quote;

    #[test]
    fn preserved_function_allows_only_the_author_owned_public_identity() {
        validate_preserved_public_surface(
            &quote! {
                pub fn tick() -> Vec<String> { Vec::new() }
                #[doc(hidden)]
                fn __sand_tick_make() -> Vec<String> { tick() }
                inventory::submit! { Descriptor { make: __sand_tick_make } }
            },
            Some("tick".to_owned()),
        )
        .unwrap();
    }

    #[test]
    fn output_free_derive_allows_private_trait_plumbing() {
        validate_preserved_public_surface(
            &quote! {
                impl Encoding for Subject {
                    fn encode(&self) -> i32 { 0 }
                }
            },
            None,
        )
        .unwrap();
    }

    #[test]
    fn output_free_derive_rejects_public_inherent_members() {
        for expansion in [
            quote! {
                impl Subject {
                    pub const GENERATED: i32 = 1;
                }
            },
            quote! {
                impl Subject {
                    pub fn generated() {}
                }
            },
        ] {
            let error = validate_preserved_public_surface(&expansion, None).unwrap_err();
            let rendered = error.to_string();
            assert!(rendered.contains("public-surface drift"), "{rendered}");
            assert!(rendered.contains("associated"), "{rendered}");
        }
    }

    #[test]
    fn texture_style_output_rejects_a_public_item() {
        let expansion = quote! {
            fn __sand_rp_tex_demo_make() {}
            inventory::submit! { Descriptor { make: __sand_rp_tex_demo_make } }
            pub struct TextureHandle;
        };
        let error = validate_preserved_public_surface(&expansion, None).unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("public-surface drift"), "{rendered}");
        assert!(rendered.contains("struct TextureHandle"), "{rendered}");
    }

    #[test]
    fn output_free_expansion_rejects_exported_declarative_macros() {
        for expansion in [
            quote! {
                #[macro_export]
                macro_rules! generated_api { () => {}; }
            },
            quote! {
                mod private_plumbing {
                    #[macro_export]
                    macro_rules! generated_api { () => {}; }
                }
            },
        ] {
            let error = validate_preserved_public_surface(&expansion, None).unwrap_err();
            let rendered = error.to_string();
            assert!(rendered.contains("public-surface drift"), "{rendered}");
            assert!(
                rendered.contains("exported macro generated_api"),
                "{rendered}"
            );
        }
    }

    #[test]
    fn future_public_sibling_fails_closed() {
        let error = validate_preserved_public_surface(
            &quote! {
                pub fn tick() -> Vec<String> { Vec::new() }
                pub struct GeneratedHandle;
            },
            Some("tick".to_owned()),
        )
        .unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("public-surface drift"), "{rendered}");
        assert!(rendered.contains("struct GeneratedHandle"), "{rendered}");
    }

    #[test]
    fn renamed_or_duplicate_public_function_fails_closed() {
        for expansion in [
            quote!(
                pub fn renamed() {}
            ),
            quote!(
                pub fn tick() {}
                pub fn tick_alias() {}
            ),
        ] {
            let error =
                validate_preserved_public_surface(&expansion, Some("tick".to_owned())).unwrap_err();
            assert!(error.to_string().contains("public-surface drift"));
        }
    }
}
