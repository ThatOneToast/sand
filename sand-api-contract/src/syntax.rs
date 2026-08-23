//! Shared `#[api]` syntax parsing and item-shape validation.

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::{
    Expr, ExprArray, FnArg, ItemEnum, ItemFn, ItemStruct, LitStr, Pat, ReturnType, Signature,
    Visibility,
};

/// Return the first complete prose sentence without treating punctuation in
/// common abbreviations, versions, or resource filenames as a boundary.
///
/// Contract generators use this only to preserve author-written Rustdoc; it
/// never invents semantic prose from an identifier.
pub fn first_prose_sentence(documentation: &str) -> &str {
    let bytes = documentation.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'.' {
            continue;
        }
        let next = bytes.get(index + 1).copied();
        if next.is_some_and(|next| !next.is_ascii_whitespace()) {
            continue;
        }
        let prefix = documentation[..=index].trim_end().to_ascii_lowercase();
        if ["e.g.", "i.e.", "etc.", "vs.", "mr.", "mrs.", "dr."]
            .iter()
            .any(|abbreviation| prefix.ends_with(abbreviation))
        {
            continue;
        }
        return documentation[..index].trim();
    }
    documentation.trim().trim_end_matches('.').trim()
}

pub mod registry_id;

/// Public associated names emitted by `#[derive(SandStorage)]`.
///
/// The derive expansion and build-time provider share this function, so a
/// field change cannot alter the generated Rust API without altering the
/// enforced provider surface during the same compilation.
pub fn sand_storage_generated_member_names(input: &syn::DeriveInput) -> syn::Result<Vec<String>> {
    let syn::Data::Struct(structure) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "SandStorage can only be derived for a struct",
        ));
    };
    let syn::Fields::Named(fields) = &structure.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "SandStorage requires named fields",
        ));
    };
    if fields.named.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "SandStorage requires at least one named field",
        ));
    }
    Ok(std::iter::once("SCHEMA".to_owned())
        .chain(fields.named.iter().map(|field| {
            field
                .ident
                .as_ref()
                .expect("named fields have identifiers")
                .unraw()
                .to_string()
        }))
        .collect())
}

/// Public declarations emitted by `#[derive(State)]`.
///
/// This intentionally models only the Rust API shape. The State derive remains
/// responsible for validating its complete schema semantics; this shared view
/// lets source reachability and downstream enforcement agree on every public
/// declaration which a valid derive invocation adds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateGeneratedSurface {
    /// Sibling bound-view struct, for example `PlayerStateBound`.
    pub bound_type: String,
    /// Public fields of the bound-view struct.
    pub bound_fields: Vec<String>,
    /// Public inherent associated constants and the binding method.
    pub associated: Vec<StateGeneratedAssociated>,
}

/// One public inherent declaration emitted by `#[derive(State)]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateGeneratedAssociated {
    pub name: String,
    pub kind: StateGeneratedAssociatedKind,
}

/// The Rust item kind for a generated State associated declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateGeneratedAssociatedKind {
    Const,
    Method,
}

/// Kind of one author-visible declaration emitted by a procedural macro.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GeneratedApiKind {
    Struct,
    Enum,
    Function,
    Constant,
    Field,
    Variant,
    AssociatedConst,
    AssociatedType,
    Method,
}

/// Semantic contract supplied by a generator for one emitted declaration.
///
/// Structural facts deliberately do not live here. They are extracted from
/// the emitted Rust by [`validate_generated_expansion`], preventing generator
/// code and its contract metadata from maintaining duplicate signatures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedApiContract {
    pub target: String,
    pub kind: GeneratedApiKind,
    pub summary: String,
    pub context: String,
    pub minecraft: String,
    pub use_when: Vec<String>,
    pub avoid_when: Vec<String>,
    pub parameters: BTreeMap<String, String>,
    pub returns: Option<String>,
    pub example: String,
}

/// Exact structural facts discovered from one emitted declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedApiShape {
    pub target: String,
    pub kind: GeneratedApiKind,
    pub signature: String,
    pub parameters: BTreeMap<String, String>,
    pub return_type: Option<String>,
}

/// Validate a generator's actual emitted public surface against its semantic
/// contracts and return the source-derived structural metadata.
///
/// `external_owners` names pre-existing public types which the expansion may
/// extend with inherent items. Public sibling types emitted by the expansion
/// are discovered automatically. Every discovered declaration must have one
/// contract, every contract must resolve, callable parameter and return
/// semantics must match the emitted signature, and the defining item must
/// contain the complete API Contract Rustdoc schema.
pub fn validate_generated_expansion(
    expansion: TokenStream,
    external_owners: impl IntoIterator<Item = String>,
    contracts: &[GeneratedApiContract],
) -> syn::Result<Vec<GeneratedApiShape>> {
    let file: syn::File = syn::parse2(expansion)?;
    let mut public_owners = external_owners.into_iter().collect::<BTreeSet<_>>();
    for item in &file.items {
        match item {
            syn::Item::Struct(item) if public_visibility(&item.vis) => {
                public_owners.insert(item.ident.unraw().to_string());
            }
            syn::Item::Enum(item) if public_visibility(&item.vis) => {
                public_owners.insert(item.ident.unraw().to_string());
            }
            _ => {}
        }
    }

    let mut discovered = BTreeMap::<String, (GeneratedApiShape, Vec<syn::Attribute>)>::new();
    for item in &file.items {
        match item {
            syn::Item::Struct(item) if public_visibility(&item.vis) => {
                let owner = item.ident.unraw().to_string();
                insert_generated_shape(
                    &mut discovered,
                    GeneratedApiShape {
                        target: owner.clone(),
                        kind: GeneratedApiKind::Struct,
                        signature: format!(
                            "{} struct {} {}",
                            item.vis.to_token_stream(),
                            item.ident,
                            item.generics.to_token_stream()
                        )
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                        parameters: BTreeMap::new(),
                        return_type: None,
                    },
                    item.attrs.clone(),
                    item,
                )?;
                for field in &item.fields {
                    if !public_visibility(&field.vis) {
                        continue;
                    }
                    let Some(name) = &field.ident else {
                        return Err(syn::Error::new_spanned(
                            field,
                            "generated public tuple fields are not supported by contract validation",
                        ));
                    };
                    let target = format!("{owner}::{}", name.unraw());
                    insert_generated_shape(
                        &mut discovered,
                        GeneratedApiShape {
                            target,
                            kind: GeneratedApiKind::Field,
                            signature: field.to_token_stream().to_string(),
                            parameters: BTreeMap::new(),
                            return_type: Some(field.ty.to_token_stream().to_string()),
                        },
                        field.attrs.clone(),
                        field,
                    )?;
                }
            }
            syn::Item::Enum(item) if public_visibility(&item.vis) => {
                let owner = item.ident.unraw().to_string();
                insert_generated_shape(
                    &mut discovered,
                    GeneratedApiShape {
                        target: owner.clone(),
                        kind: GeneratedApiKind::Enum,
                        signature: format!("{} enum {}", item.vis.to_token_stream(), item.ident),
                        parameters: BTreeMap::new(),
                        return_type: None,
                    },
                    item.attrs.clone(),
                    item,
                )?;
                for variant in &item.variants {
                    insert_generated_shape(
                        &mut discovered,
                        GeneratedApiShape {
                            target: format!("{owner}::{}", variant.ident.unraw()),
                            kind: GeneratedApiKind::Variant,
                            signature: variant.to_token_stream().to_string(),
                            parameters: BTreeMap::new(),
                            return_type: None,
                        },
                        variant.attrs.clone(),
                        variant,
                    )?;
                }
            }
            syn::Item::Fn(item) if public_visibility(&item.vis) => {
                let (parameters, return_type) = callable_shape(&item.sig)?;
                insert_generated_shape(
                    &mut discovered,
                    GeneratedApiShape {
                        target: item.sig.ident.unraw().to_string(),
                        kind: GeneratedApiKind::Function,
                        signature: item.sig.to_token_stream().to_string(),
                        parameters,
                        return_type,
                    },
                    item.attrs.clone(),
                    item,
                )?;
            }
            syn::Item::Const(item) if public_visibility(&item.vis) => {
                insert_generated_shape(
                    &mut discovered,
                    GeneratedApiShape {
                        target: item.ident.unraw().to_string(),
                        kind: GeneratedApiKind::Constant,
                        signature: format!(
                            "{} const {} : {}",
                            item.vis.to_token_stream(),
                            item.ident,
                            item.ty.to_token_stream()
                        ),
                        parameters: BTreeMap::new(),
                        return_type: Some(item.ty.to_token_stream().to_string()),
                    },
                    item.attrs.clone(),
                    item,
                )?;
            }
            syn::Item::Impl(item) if item.trait_.is_none() => {
                let syn::Type::Path(owner) = item.self_ty.as_ref() else {
                    continue;
                };
                let Some(owner) = owner.path.segments.last() else {
                    continue;
                };
                let owner = owner.ident.unraw().to_string();
                if !public_owners.contains(&owner) {
                    continue;
                }
                for member in &item.items {
                    match member {
                        syn::ImplItem::Const(member) if public_visibility(&member.vis) => {
                            insert_generated_shape(
                                &mut discovered,
                                GeneratedApiShape {
                                    target: format!("{owner}::{}", member.ident.unraw()),
                                    kind: GeneratedApiKind::AssociatedConst,
                                    signature: format!(
                                        "{} const {} : {}",
                                        member.vis.to_token_stream(),
                                        member.ident,
                                        member.ty.to_token_stream()
                                    ),
                                    parameters: BTreeMap::new(),
                                    return_type: Some(member.ty.to_token_stream().to_string()),
                                },
                                member.attrs.clone(),
                                member,
                            )?;
                        }
                        syn::ImplItem::Type(member) if public_visibility(&member.vis) => {
                            insert_generated_shape(
                                &mut discovered,
                                GeneratedApiShape {
                                    target: format!("{owner}::{}", member.ident.unraw()),
                                    kind: GeneratedApiKind::AssociatedType,
                                    signature: member.to_token_stream().to_string(),
                                    parameters: BTreeMap::new(),
                                    return_type: Some(member.ty.to_token_stream().to_string()),
                                },
                                member.attrs.clone(),
                                member,
                            )?;
                        }
                        syn::ImplItem::Fn(member) if public_visibility(&member.vis) => {
                            let (parameters, return_type) = callable_shape(&member.sig)?;
                            insert_generated_shape(
                                &mut discovered,
                                GeneratedApiShape {
                                    target: format!("{owner}::{}", member.sig.ident.unraw()),
                                    kind: GeneratedApiKind::Method,
                                    signature: member.sig.to_token_stream().to_string(),
                                    parameters,
                                    return_type,
                                },
                                member.attrs.clone(),
                                member,
                            )?;
                        }
                        _ => {}
                    }
                }
            }
            item if public_item_visibility(item).is_some_and(public_visibility) => {
                return Err(syn::Error::new_spanned(
                    item,
                    "generated expansion contains an unsupported public item kind",
                ));
            }
            _ => {}
        }
    }

    let mut declared = BTreeMap::new();
    for contract in contracts {
        validate_contract_semantics(&ContractSemantics {
            summary: Some(&contract.summary),
            context: Some(&contract.context),
            minecraft: Some(&contract.minecraft),
            use_when: Some(&contract.use_when),
            avoid_when: Some(&contract.avoid_when),
            example: Some(&contract.example),
        })
        .map_err(|message| syn::Error::new(proc_macro2::Span::call_site(), message))?;
        if declared
            .insert(contract.target.as_str(), contract)
            .is_some()
        {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("duplicate generated API contract `{}`", contract.target),
            ));
        }
    }

    for (target, (shape, attrs)) in &discovered {
        let Some(contract) = declared.remove(target.as_str()) else {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("generated public API `{target}` has no semantic contract"),
            ));
        };
        if contract.kind != shape.kind {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "generated API `{target}` has kind {:?}, but its contract declares {:?}",
                    shape.kind, contract.kind
                ),
            ));
        }
        let parameter_names = shape.parameters.keys().cloned().collect::<BTreeSet<_>>();
        let described_parameters = contract.parameters.keys().cloned().collect::<BTreeSet<_>>();
        if parameter_names != described_parameters {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "generated API `{target}` parameter contract drift: emitted {parameter_names:?}, described {described_parameters:?}"
                ),
            ));
        }
        if shape.return_type.is_some() != contract.returns.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "generated API `{target}` return contract drift: emitted {:?}, described {:?}",
                    shape.return_type, contract.returns
                ),
            ));
        }
        validate_generated_rustdoc(target, attrs, contract)?;
    }
    if let Some(target) = declared.keys().next() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("generated API contract `{target}` does not resolve to emitted public Rust"),
        ));
    }
    Ok(discovered.into_values().map(|(shape, _)| shape).collect())
}

fn public_visibility(visibility: &Visibility) -> bool {
    matches!(visibility, Visibility::Public(_))
}

fn public_item_visibility(item: &syn::Item) -> Option<&Visibility> {
    match item {
        syn::Item::Const(item) => Some(&item.vis),
        syn::Item::ExternCrate(item) => Some(&item.vis),
        syn::Item::Mod(item) => Some(&item.vis),
        syn::Item::Static(item) => Some(&item.vis),
        syn::Item::Trait(item) => Some(&item.vis),
        syn::Item::TraitAlias(item) => Some(&item.vis),
        syn::Item::Type(item) => Some(&item.vis),
        syn::Item::Union(item) => Some(&item.vis),
        syn::Item::Use(item) => Some(&item.vis),
        _ => None,
    }
}

fn insert_generated_shape(
    discovered: &mut BTreeMap<String, (GeneratedApiShape, Vec<syn::Attribute>)>,
    shape: GeneratedApiShape,
    attrs: Vec<syn::Attribute>,
    source: &impl ToTokens,
) -> syn::Result<()> {
    if discovered
        .insert(shape.target.clone(), (shape, attrs))
        .is_some()
    {
        Err(syn::Error::new_spanned(
            source,
            "generated expansion emits a duplicate public API identity",
        ))
    } else {
        Ok(())
    }
}

fn callable_shape(
    signature: &Signature,
) -> syn::Result<(BTreeMap<String, String>, Option<String>)> {
    let mut parameters = BTreeMap::new();
    for input in &signature.inputs {
        match input {
            FnArg::Receiver(_) => {}
            FnArg::Typed(argument) => {
                let Pat::Ident(name) = argument.pat.as_ref() else {
                    return Err(syn::Error::new_spanned(
                        &argument.pat,
                        "generated public callable parameters must use identifier patterns",
                    ));
                };
                parameters.insert(
                    name.ident.unraw().to_string(),
                    argument.ty.to_token_stream().to_string(),
                );
            }
        }
    }
    let return_type = match &signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) if matches!(ty.as_ref(), syn::Type::Tuple(tuple) if tuple.elems.is_empty()) => {
            None
        }
        ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
    };
    Ok((parameters, return_type))
}

fn validate_generated_rustdoc(
    target: &str,
    attrs: &[syn::Attribute],
    contract: &GeneratedApiContract,
) -> syn::Result<()> {
    let docs = attrs
        .iter()
        .filter_map(|attribute| {
            let syn::Meta::NameValue(value) = &attribute.meta else {
                return None;
            };
            if !value.path.is_ident("doc") {
                return None;
            }
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) = &value.value
            else {
                return None;
            };
            Some(text.value())
        })
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "# API Contract",
        "**Context:**",
        "**Minecraft behavior:**",
        "**Use when:**",
        "**Avoid when:**",
        "**Example:**",
        contract.summary.as_str(),
        contract.context.as_str(),
        contract.minecraft.as_str(),
        contract.example.as_str(),
    ] {
        if !docs.contains(required) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "generated public API `{target}` is missing API Contract Rustdoc material `{required}`"
                ),
            ));
        }
    }
    for value in contract
        .use_when
        .iter()
        .chain(&contract.avoid_when)
        .chain(contract.parameters.values())
        .chain(contract.returns.iter())
    {
        if !docs.contains(value) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("generated public API `{target}` Rustdoc omits contract text `{value}`"),
            ));
        }
    }
    Ok(())
}

/// Derive the complete public Rust shape of a valid `State` invocation.
pub fn state_generated_surface(input: &syn::DeriveInput) -> syn::Result<StateGeneratedSurface> {
    let syn::Data::Struct(structure) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "State can only be derived for a struct",
        ));
    };
    let syn::Fields::Named(fields) = &structure.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "State requires a struct with named fields",
        ));
    };

    let state_attributes = input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("state"))
        .collect::<Vec<_>>();
    if state_attributes.len() != 1 {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "exactly one #[state(...)] schema attribute is required",
        ));
    }
    let mut binding_method = None;
    state_attributes[0].parse_nested_meta(|meta| {
        if meta.path.is_ident("scope") {
            let scope: syn::Ident = meta.value()?.parse()?;
            binding_method = Some(match scope.to_string().as_str() {
                "player" | "entity" | "living" => "on",
                "global" => "global",
                _ => {
                    return Err(syn::Error::new_spanned(
                        scope,
                        "invalid state scope; expected player, entity, living, or global",
                    ));
                }
            });
        } else if meta.input.peek(syn::Token![=]) {
            // The derive validates the complete schema attribute. This shape
            // model only needs `scope`, but it must consume the remaining
            // literal options so syn can continue to the next entry.
            let _: Expr = meta.value()?.parse()?;
        }
        Ok(())
    })?;
    let binding_method = binding_method.ok_or_else(|| {
        syn::Error::new_spanned(
            state_attributes[0],
            "state schema requires `scope = player|entity|living|global`",
        )
    })?;

    let field_names = fields
        .named
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .expect("named fields have identifiers")
                .unraw()
                .to_string()
        })
        .collect::<Vec<_>>();
    let associated = std::iter::once(StateGeneratedAssociated {
        name: "FIELDS".to_owned(),
        kind: StateGeneratedAssociatedKind::Const,
    })
    .chain(
        field_names
            .iter()
            .cloned()
            .map(|name| StateGeneratedAssociated {
                name,
                kind: StateGeneratedAssociatedKind::Const,
            }),
    )
    .chain(std::iter::once(StateGeneratedAssociated {
        name: binding_method.to_owned(),
        kind: StateGeneratedAssociatedKind::Method,
    }))
    .collect();
    Ok(StateGeneratedSurface {
        bound_type: format!("{}Bound", input.ident.unraw()),
        bound_fields: field_names,
        associated,
    })
}

/// Public declarations emitted by `#[custom_item]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomItemGeneratedSurface {
    /// Sibling typed item reference struct.
    pub type_name: String,
    /// Public inherent constants in declaration order.
    pub constants: Vec<String>,
    /// Public inherent helper methods in declaration order.
    pub methods: Vec<String>,
}

/// Derive the complete public Rust shape of a valid `custom_item` invocation.
///
/// The macro's item identity comes from its explicit `name` argument or its
/// literal `custom_data` key. Keeping this parser deliberately literal is a
/// safety property: a future macro extension that accepts a dynamic name must
/// add an equally explicit provider model instead of silently escaping
/// consumer-build enforcement.
pub fn custom_item_generated_surface(function: &ItemFn) -> syn::Result<CustomItemGeneratedSurface> {
    let attribute = function
        .attrs
        .iter()
        .find(|attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "custom_item")
        })
        .ok_or_else(|| syn::Error::new_spanned(function, "missing #[custom_item] attribute"))?;
    let mut explicit_name = None;
    let mut data_constants = Vec::new();
    if !matches!(attribute.meta, syn::Meta::Path(_)) {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                explicit_name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("data") {
                let value = meta.value()?;
                let content;
                syn::bracketed!(content in value);
                while !content.is_empty() {
                    let name: syn::Ident = content.parse()?;
                    content.parse::<syn::Token![:]>()?;
                    content.parse::<syn::Type>()?;
                    content.parse::<syn::Token![=]>()?;
                    content.parse::<Expr>()?;
                    data_constants.push(name.unraw().to_string());
                    if content.peek(syn::Token![,]) {
                        content.parse::<syn::Token![,]>()?;
                    }
                }
            } else {
                return Err(meta.error("unknown #[custom_item] option"));
            }
            Ok(())
        })?;
    }

    let mut custom_data = None;
    for statement in &function.block.stmts {
        custom_item_data_in_statement(statement, &mut custom_data);
    }
    let type_name = explicit_name
        .or_else(|| custom_data.as_deref().map(custom_item_pascal_case))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &function.sig,
                "#[custom_item] needs name = \"TypeName\" or a literal .custom_data(\"key\") call",
            )
        })?;
    let mut constants = vec!["BASE".to_owned(), "PREDICATE".to_owned()];
    if custom_data.is_some() {
        constants.extend(["CUSTOM_DATA_KEY".to_owned(), "CUSTOM_DATA_SNBT".to_owned()]);
    }
    constants.extend(data_constants);
    Ok(CustomItemGeneratedSurface {
        type_name,
        constants,
        methods: vec![
            "if_wearing".to_owned(),
            "unless_wearing".to_owned(),
            "item".to_owned(),
        ],
    })
}

fn custom_item_pascal_case(value: &str) -> String {
    value
        .split(['_', '-'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut characters = segment.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect()
}

fn custom_item_data_in_statement(statement: &syn::Stmt, custom_data: &mut Option<String>) {
    match statement {
        syn::Stmt::Expr(expression, _) => custom_item_data_in_expression(expression, custom_data),
        syn::Stmt::Local(local) => {
            if let Some(initializer) = &local.init {
                custom_item_data_in_expression(&initializer.expr, custom_data);
            }
        }
        syn::Stmt::Item(_) | syn::Stmt::Macro(_) => {}
    }
}

fn custom_item_data_in_expression(expression: &Expr, custom_data: &mut Option<String>) {
    match expression {
        Expr::MethodCall(call) => {
            if call.method == "custom_data"
                && let Some(Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                })) = call.args.first()
            {
                *custom_data = Some(value.value());
            }
            custom_item_data_in_expression(&call.receiver, custom_data);
            for argument in &call.args {
                custom_item_data_in_expression(argument, custom_data);
            }
        }
        Expr::Call(call) => {
            custom_item_data_in_expression(&call.func, custom_data);
            for argument in &call.args {
                custom_item_data_in_expression(argument, custom_data);
            }
        }
        Expr::Block(block) => {
            for statement in &block.block.stmts {
                custom_item_data_in_statement(statement, custom_data);
            }
        }
        Expr::Return(returned) => {
            if let Some(expression) = &returned.expr {
                custom_item_data_in_expression(expression, custom_data);
            }
        }
        _ => {}
    }
}

/// One explicitly described function parameter or nested API member.
#[derive(Clone)]
pub struct Description {
    pub name: syn::Ident,
    pub text: LitStr,
}

/// One explicitly described field carried by an enum variant.
///
/// Tuple fields use their zero-based position as a string (for example,
/// `ParseError = ["..."]` documents `ParseError::0`). Named fields use the
/// field identifier (`UnknownVersion(requested = "...", hint = "...")`).
#[derive(Clone)]
pub struct VariantFieldDescription {
    pub variant: syn::Ident,
    pub name: String,
    pub text: LitStr,
}

/// Parsed `#[api(...)]` arguments, retaining spans for precise diagnostics.
#[derive(Default)]
pub struct ContractArgs {
    /// Narrow item-kind hint for syntax that is ambiguous outside its enclosing impl.
    pub kind: Option<LitStr>,
    /// Rust path used only to transport the link-time registration.
    ///
    /// Facade users normally omit this and register through Sand's hidden
    /// re-export. API-defining implementation crates set it to their direct
    /// `sand-api-contract` dependency, avoiding a dependency on the facade.
    pub registry: Option<syn::Path>,
    pub path: Option<LitStr>,
    pub module: Option<LitStr>,
    pub aliases: Option<Vec<LitStr>>,
    pub summary: Option<LitStr>,
    pub context: Option<LitStr>,
    pub minecraft: Option<LitStr>,
    pub use_when: Option<Vec<LitStr>>,
    pub avoid_when: Option<Vec<LitStr>>,
    pub params: Option<Vec<Description>>,
    pub returns: Option<LitStr>,
    pub example: Option<LitStr>,
    pub availability: Option<Vec<LitStr>>,
    pub variants: Option<Vec<Description>>,
    pub fields: Option<Vec<Description>>,
    pub variant_fields: Option<Vec<VariantFieldDescription>>,
}

/// The declaration shape whose contract is being validated.
pub enum ContractTarget<'a> {
    Function {
        ident: &'a syn::Ident,
        signature: &'a Signature,
    },
    Struct(&'a ItemStruct),
    Enum(&'a ItemEnum),
    Plain {
        ident: &'a syn::Ident,
    },
}

/// Span-free semantic projection used by every contract producer, including
/// facade registrations that are resolved at build time rather than expanded
/// as an attribute on the defining item.
pub struct ContractSemantics<'a> {
    pub summary: Option<&'a str>,
    pub context: Option<&'a str>,
    pub minecraft: Option<&'a str>,
    pub use_when: Option<&'a [String]>,
    pub avoid_when: Option<&'a [String]>,
    pub example: Option<&'a str>,
}

/// Validate the required semantic schema independently of its Rust syntax.
pub fn validate_contract_semantics(semantics: &ContractSemantics<'_>) -> Result<(), String> {
    for (name, value) in [
        ("summary", semantics.summary),
        ("context", semantics.context),
        ("minecraft", semantics.minecraft),
        ("example", semantics.example),
    ] {
        let value = value.ok_or_else(|| format!("missing required API contract field `{name}`"))?;
        if value.trim().is_empty() {
            return Err(format!("API contract field `{name}` cannot be empty"));
        }
    }
    for (name, values) in [
        ("use_when", semantics.use_when),
        ("avoid_when", semantics.avoid_when),
    ] {
        let values =
            values.ok_or_else(|| format!("missing required API contract field `{name}`"))?;
        if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
            return Err(format!(
                "API contract field `{name}` must contain non-empty strings"
            ));
        }
    }
    Ok(())
}

impl ContractTarget<'_> {
    pub fn ident(&self) -> &syn::Ident {
        match self {
            Self::Function { ident, .. } | Self::Plain { ident } => ident,
            Self::Struct(item) => &item.ident,
            Self::Enum(item) => &item.ident,
        }
    }
}

/// Parse one attribute's comma-separated argument tokens.
pub fn parse_contract_args(tokens: TokenStream) -> syn::Result<ContractArgs> {
    let mut args = ContractArgs::default();
    let parser = syn::meta::parser(|meta| {
        let key = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("API contract keys must be identifiers"))?
            .to_string();
        match key.as_str() {
            "kind" => set_once(&mut args.kind, meta.value()?.parse()?, &meta, "kind"),
            "registry" => set_once(
                &mut args.registry,
                meta.value()?.parse()?,
                &meta,
                "registry",
            ),
            "path" => set_once(&mut args.path, meta.value()?.parse()?, &meta, "path"),
            "module" => set_once(&mut args.module, meta.value()?.parse()?, &meta, "module"),
            "summary" => set_once(&mut args.summary, meta.value()?.parse()?, &meta, "summary"),
            "context" => set_once(&mut args.context, meta.value()?.parse()?, &meta, "context"),
            "minecraft" => set_once(
                &mut args.minecraft,
                meta.value()?.parse()?,
                &meta,
                "minecraft",
            ),
            "returns" => set_once(&mut args.returns, meta.value()?.parse()?, &meta, "returns"),
            "example" => set_once(&mut args.example, meta.value()?.parse()?, &meta, "example"),
            "aliases" => parse_array_field(&mut args.aliases, &meta, "aliases"),
            "use_when" => parse_array_field(&mut args.use_when, &meta, "use_when"),
            "avoid_when" => parse_array_field(&mut args.avoid_when, &meta, "avoid_when"),
            "availability" => parse_array_field(&mut args.availability, &meta, "availability"),
            "params" => parse_descriptions(&mut args.params, &meta, "params", "parameter"),
            "variants" => parse_descriptions(&mut args.variants, &meta, "variants", "variant"),
            "fields" => parse_descriptions(&mut args.fields, &meta, "fields", "field"),
            "variant_fields" => parse_variant_fields(&mut args.variant_fields, &meta),
            _ => Err(meta.error(format!("unknown API contract field `{key}`"))),
        }
    });
    parser.parse2(tokens)?;
    Ok(args)
}

fn parse_variant_fields(
    slot: &mut Option<Vec<VariantFieldDescription>>,
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<()> {
    if slot.is_some() {
        return Err(meta.error("duplicate API contract field `variant_fields`"));
    }
    let mut descriptions = Vec::new();
    let mut names = BTreeSet::new();
    meta.parse_nested_meta(|variant| {
        let variant_name = variant
            .path
            .get_ident()
            .cloned()
            .ok_or_else(|| variant.error("variant-field owners must be identifiers"))?;
        if variant.input.peek(syn::Token![=]) {
            let values = variant.value()?.parse::<ExprArray>()?;
            for (index, value) in values.elems.into_iter().enumerate() {
                let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                }) = value
                else {
                    return Err(syn::Error::new_spanned(value, "expected a string literal"));
                };
                if text.value().trim().is_empty() {
                    return Err(syn::Error::new_spanned(
                        text,
                        "variant field description cannot be empty",
                    ));
                }
                let name = index.to_string();
                if !names.insert(format!("{variant_name}::{name}")) {
                    return Err(variant.error(format!(
                        "duplicate variant field documentation for `{variant_name}::{name}`"
                    )));
                }
                descriptions.push(VariantFieldDescription {
                    variant: variant_name.clone(),
                    name,
                    text,
                });
            }
            return Ok(());
        }
        variant.parse_nested_meta(|field| {
            let name = field
                .path
                .get_ident()
                .map(ToString::to_string)
                .ok_or_else(|| field.error("named variant fields must be identifiers"))?;
            let text: LitStr = field.value()?.parse()?;
            if text.value().trim().is_empty() {
                return Err(syn::Error::new_spanned(
                    text,
                    "variant field description cannot be empty",
                ));
            }
            if !names.insert(format!("{variant_name}::{name}")) {
                return Err(field.error(format!(
                    "duplicate variant field documentation for `{variant_name}::{name}`"
                )));
            }
            descriptions.push(VariantFieldDescription {
                variant: variant_name.clone(),
                name,
                text,
            });
            Ok(())
        })
    })?;
    *slot = Some(descriptions);
    Ok(())
}

fn parse_array_field(
    slot: &mut Option<Vec<LitStr>>,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
) -> syn::Result<()> {
    let value = parse_string_array(meta)?;
    set_once(slot, value, meta, name)
}

fn parse_descriptions(
    slot: &mut Option<Vec<Description>>,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
    member_kind: &str,
) -> syn::Result<()> {
    if slot.is_some() {
        return Err(meta.error(format!("duplicate API contract field `{name}`")));
    }
    let mut descriptions = Vec::new();
    let mut names = BTreeSet::new();
    meta.parse_nested_meta(|member| {
        let ident = member
            .path
            .get_ident()
            .cloned()
            .ok_or_else(|| member.error(format!("{member_kind} names must be identifiers")))?;
        if !names.insert(ident.to_string()) {
            return Err(member.error(format!(
                "duplicate {member_kind} documentation for `{ident}`"
            )));
        }
        let text: LitStr = member.value()?.parse()?;
        if text.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(
                &text,
                format!("{member_kind} description cannot be empty"),
            ));
        }
        descriptions.push(Description { name: ident, text });
        Ok(())
    })?;
    *slot = Some(descriptions);
    Ok(())
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
) -> syn::Result<()> {
    if slot.replace(value).is_some() {
        Err(meta.error(format!("duplicate API contract field `{name}`")))
    } else {
        Ok(())
    }
}

fn parse_string_array(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Vec<LitStr>> {
    let array = meta.value()?.parse::<ExprArray>()?;
    array
        .elems
        .into_iter()
        .map(|element| match element {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(value),
                ..
            }) => Ok(value),
            other => Err(syn::Error::new_spanned(other, "expected a string literal")),
        })
        .collect()
}

/// Validate required prose, paths, parameters, returns, and nested members.
pub fn validate_contract(args: &ContractArgs, target: &ContractTarget<'_>) -> syn::Result<()> {
    let ident = target.ident();
    let use_when = args
        .use_when
        .as_ref()
        .map(|values| values.iter().map(LitStr::value).collect::<Vec<_>>());
    let avoid_when = args
        .avoid_when
        .as_ref()
        .map(|values| values.iter().map(LitStr::value).collect::<Vec<_>>());
    validate_contract_semantics(&ContractSemantics {
        summary: args.summary.as_ref().map(LitStr::value).as_deref(),
        context: args.context.as_ref().map(LitStr::value).as_deref(),
        minecraft: args.minecraft.as_ref().map(LitStr::value).as_deref(),
        use_when: use_when.as_deref(),
        avoid_when: avoid_when.as_deref(),
        example: args.example.as_ref().map(LitStr::value).as_deref(),
    })
    .map_err(|message| syn::Error::new(ident.span(), message))?;
    for (name, value) in [
        ("summary", args.summary.as_ref()),
        ("context", args.context.as_ref()),
        ("minecraft", args.minecraft.as_ref()),
        ("example", args.example.as_ref()),
    ] {
        let value = value.ok_or_else(|| {
            syn::Error::new(
                ident.span(),
                format!("missing required API contract field `{name}`"),
            )
        })?;
        if value.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(
                value,
                format!("API contract field `{name}` cannot be empty"),
            ));
        }
    }
    for (name, values) in [
        ("use_when", args.use_when.as_ref()),
        ("avoid_when", args.avoid_when.as_ref()),
    ] {
        let values = values.ok_or_else(|| {
            syn::Error::new(
                ident.span(),
                format!("missing required API contract field `{name}`"),
            )
        })?;
        if values.is_empty() || values.iter().any(|value| value.value().trim().is_empty()) {
            return Err(syn::Error::new(
                ident.span(),
                format!("API contract field `{name}` must contain non-empty strings"),
            ));
        }
    }
    if let Some(path) = &args.path {
        validate_path(path, "path")?;
    }
    if let Some(kind) = &args.kind {
        match kind.value().as_str() {
            "method" | "associated_const" | "associated_type" => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    kind,
                    "`kind` must be `method`, `associated_const`, or `associated_type`",
                ));
            }
        }
    }
    if let Some(module) = &args.module {
        validate_path(module, "module")?;
    }
    let mut aliases = BTreeSet::new();
    for alias in args.aliases.as_deref().unwrap_or_default() {
        validate_path(alias, "alias")?;
        if !aliases.insert(alias.value()) {
            return Err(syn::Error::new_spanned(
                alias,
                "duplicate API contract alias",
            ));
        }
    }

    match target {
        ContractTarget::Function { signature, .. } => {
            reject_members(args, ident)?;
            validate_function(args, signature)
        }
        ContractTarget::Struct(item) => {
            reject_variant_docs(args, ident)?;
            validate_struct_fields(args, item)
        }
        ContractTarget::Enum(item) => {
            reject_field_docs(args, ident)?;
            validate_enum_variants(args, item)
        }
        ContractTarget::Plain { .. } => {
            reject_members(args, ident)?;
            reject_parameters_and_returns(args, ident)
        }
    }
}

fn reject_members(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    reject_field_docs(args, ident)?;
    reject_variant_docs(args, ident)
}

fn reject_field_docs(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    if let Some(fields) = &args.fields {
        return Err(syn::Error::new_spanned(
            fields.first().map_or_else(
                || ident.to_token_stream(),
                |field| field.name.to_token_stream(),
            ),
            "field descriptions are only valid on structs with public named fields",
        ));
    }
    Ok(())
}

fn reject_variant_docs(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    if let Some(variants) = &args.variants {
        return Err(syn::Error::new_spanned(
            variants.first().map_or_else(
                || ident.to_token_stream(),
                |variant| variant.name.to_token_stream(),
            ),
            "variant descriptions are only valid on enums",
        ));
    }
    if let Some(fields) = &args.variant_fields {
        return Err(syn::Error::new_spanned(
            fields.first().map_or_else(
                || ident.to_token_stream(),
                |field| field.variant.to_token_stream(),
            ),
            "variant-field descriptions are only valid on enums",
        ));
    }
    Ok(())
}

fn reject_parameters_and_returns(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    if let Some(parameters) = &args.params {
        return Err(syn::Error::new_spanned(
            parameters.first().map_or_else(
                || ident.to_token_stream(),
                |parameter| parameter.name.to_token_stream(),
            ),
            "parameter descriptions are only valid on functions and methods",
        ));
    }
    if let Some(returns) = &args.returns {
        return Err(syn::Error::new_spanned(
            returns,
            "`returns` is only valid on functions and methods",
        ));
    }
    Ok(())
}

fn validate_function(args: &ContractArgs, signature: &Signature) -> syn::Result<()> {
    let mut actual = Vec::new();
    for argument in &signature.inputs {
        match argument {
            FnArg::Receiver(_) => {}
            FnArg::Typed(argument) => match argument.pat.as_ref() {
                Pat::Ident(ident) => actual.push(ident.ident.to_string()),
                pattern => {
                    return Err(syn::Error::new_spanned(
                        pattern,
                        "contracted public API parameters must use simple identifier patterns",
                    ));
                }
            },
        }
    }
    let documented = args
        .params
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|description| (description.name.to_string(), &description.name))
        .collect::<BTreeMap<_, _>>();
    for name in &actual {
        if !documented.contains_key(name) {
            return Err(syn::Error::new_spanned(
                signature,
                format!("missing API contract documentation for parameter `{name}`"),
            ));
        }
    }
    for (name, ident) in &documented {
        if !actual.contains(name) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("API contract documents nonexistent parameter `{name}`"),
            ));
        }
    }
    match (&signature.output, &args.returns) {
        (ReturnType::Default, Some(value)) => Err(syn::Error::new_spanned(
            value,
            "`returns` is not valid on a function without a return value",
        )),
        (ReturnType::Type(_, _), None) => Err(syn::Error::new_spanned(
            signature,
            "missing required API contract field `returns`",
        )),
        _ => Ok(()),
    }
}

fn validate_struct_fields(args: &ContractArgs, item: &ItemStruct) -> syn::Result<()> {
    reject_parameters_and_returns(args, &item.ident)?;
    let mut actual = BTreeMap::new();
    for field in &item.fields {
        if matches!(field.vis, syn::Visibility::Public(_)) && !doc_hidden(&field.attrs) {
            let Some(ident) = &field.ident else {
                return Err(syn::Error::new_spanned(
                    field,
                    "contracted public tuple fields are unsupported; use named fields or make them private",
                ));
            };
            actual.insert(ident.to_string(), ident);
        }
    }
    validate_members(
        "field",
        args.fields.as_deref().unwrap_or_default(),
        &actual,
        &item.ident,
    )
}

fn validate_enum_variants(args: &ContractArgs, item: &ItemEnum) -> syn::Result<()> {
    reject_parameters_and_returns(args, &item.ident)?;
    let actual = item
        .variants
        .iter()
        .filter(|variant| !doc_hidden(&variant.attrs))
        .map(|variant| (variant.ident.to_string(), &variant.ident))
        .collect::<BTreeMap<_, _>>();
    validate_members(
        "variant",
        args.variants.as_deref().unwrap_or_default(),
        &actual,
        &item.ident,
    )?;
    let mut actual_fields = BTreeMap::new();
    for variant in &item.variants {
        if doc_hidden(&variant.attrs) {
            continue;
        }
        for (index, field) in variant.fields.iter().enumerate() {
            if doc_hidden(&field.attrs) {
                continue;
            }
            let name = field
                .ident
                .as_ref()
                .map_or_else(|| index.to_string(), ToString::to_string);
            actual_fields.insert(
                format!("{}::{name}", variant.ident),
                (variant.ident.clone(), name, field),
            );
        }
    }
    let documented = args.variant_fields.as_deref().unwrap_or_default();
    let mut docs = BTreeMap::new();
    for doc in documented {
        let key = format!("{}::{}", doc.variant, doc.name);
        docs.insert(key, doc);
    }
    for (name, (_, _, field)) in &actual_fields {
        if !docs.contains_key(name) {
            return Err(syn::Error::new_spanned(
                field,
                format!("missing API contract documentation for variant field `{name}`"),
            ));
        }
    }
    for (name, doc) in docs {
        if !actual_fields.contains_key(&name) {
            return Err(syn::Error::new_spanned(
                &doc.variant,
                format!(
                    "API contract documents nonexistent variant field `{name}` on `{}`",
                    item.ident
                ),
            ));
        }
    }
    Ok(())
}

fn validate_members(
    kind: &str,
    documented: &[Description],
    actual: &BTreeMap<String, &syn::Ident>,
    parent: &syn::Ident,
) -> syn::Result<()> {
    let docs = documented
        .iter()
        .map(|doc| (doc.name.to_string(), &doc.name))
        .collect::<BTreeMap<_, _>>();
    for (name, ident) in actual {
        if !docs.contains_key(name) {
            return Err(syn::Error::new_spanned(
                *ident,
                format!("missing API contract documentation for {kind} `{name}`"),
            ));
        }
    }
    for (name, ident) in docs {
        if !actual.contains_key(&name) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("API contract documents nonexistent {kind} `{name}` on `{parent}`"),
            ));
        }
    }
    Ok(())
}

fn doc_hidden(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr
                .parse_args::<syn::Ident>()
                .is_ok_and(|ident| ident == "hidden")
    })
}

pub fn validate_path(value: &LitStr, role: &str) -> syn::Result<()> {
    let path = value.value();
    let valid = (role == "module" && path == "sand")
        || path.starts_with("sand::")
            && path.split("::").all(|segment| {
                !segment.is_empty()
                    && segment.chars().enumerate().all(|(index, ch)| {
                        ch == '_'
                            || ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit())
                    })
            });
    if valid {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("invalid canonical API {role}; expected a path beginning with `sand::`"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parses_nested_member_descriptions() {
        let args = parse_contract_args(quote!(
            summary = "Value.",
            context = "Context.",
            minecraft = "Behavior.",
            use_when = ["Useful."],
            avoid_when = ["Not useful."],
            example = "Value::A",
            variants(A = "The first state.", B = "The second state.")
        ))
        .unwrap();
        assert_eq!(args.variants.unwrap().len(), 2);
    }

    #[test]
    fn sand_storage_members_come_from_the_derive_declaration() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct PlayerMagic { mana: i32, school: String }
        };
        assert_eq!(
            sand_storage_generated_member_names(&input).unwrap(),
            ["SCHEMA", "mana", "school"]
        );
    }

    #[test]
    fn prose_sentence_preserves_abbreviations_versions_paths_and_parentheses() {
        for (documentation, expected) in [
            (
                "Typed identifier, e.g. `minecraft:stone`. More.",
                "Typed identifier, e.g. `minecraft:stone`",
            ),
            (
                "Use a typed value, i.e. not raw text. More.",
                "Use a typed value, i.e. not raw text",
            ),
            (
                "Available in Minecraft 1.21.5. More.",
                "Available in Minecraft 1.21.5",
            ),
            (
                "Writes data/demo/example.json. More.",
                "Writes data/demo/example.json",
            ),
            (
                "Selects a value (e.g. stone or dirt). More.",
                "Selects a value (e.g. stone or dirt)",
            ),
        ] {
            assert_eq!(first_prose_sentence(documentation), expected);
        }
    }

    #[test]
    fn generated_top_level_constants_are_extracted_and_require_constant_contracts() {
        let contract = GeneratedApiContract {
            target: "HEALTH_BAR".to_owned(),
            kind: GeneratedApiKind::Constant,
            summary: "Names the generated health HUD bar handle.".to_owned(),
            context: "The handle connects author code to the HUD bar declared by the macro."
                .to_owned(),
            minecraft:
                "The handle selects the generated font and display resources for this HUD bar."
                    .to_owned(),
            use_when: vec!["Updating or rendering this generated HUD bar".to_owned()],
            avoid_when: vec!["Addressing a different generated HUD element".to_owned()],
            parameters: BTreeMap::new(),
            returns: Some("The typed handle for this generated HUD bar.".to_owned()),
            example: "let bar = HEALTH_BAR;".to_owned(),
        };
        let expansion = quote! {
            #[doc = "Names the generated health HUD bar handle."]
            #[doc = "# API Contract"]
            #[doc = "**Context:** The handle connects author code to the HUD bar declared by the macro."]
            #[doc = "**Minecraft behavior:** The handle selects the generated font and display resources for this HUD bar."]
            #[doc = "**Use when:** Updating or rendering this generated HUD bar"]
            #[doc = "**Avoid when:** Addressing a different generated HUD element"]
            #[doc = "**Returns:** The typed handle for this generated HUD bar."]
            #[doc = "**Example:** `let bar = HEALTH_BAR;`"]
            pub const HEALTH_BAR: BarHandle = BarHandle::new();
        };
        let shapes = validate_generated_expansion(expansion.clone(), [], &[contract.clone()])
            .expect("top-level constant contract");
        assert_eq!(shapes[0].kind, GeneratedApiKind::Constant);
        assert_eq!(shapes[0].return_type.as_deref(), Some("BarHandle"));
        assert!(shapes[0].signature.contains("pub const HEALTH_BAR"));

        let mut wrong_kind = contract;
        wrong_kind.kind = GeneratedApiKind::AssociatedConst;
        assert!(
            validate_generated_expansion(expansion, [], &[wrong_kind])
                .unwrap_err()
                .to_string()
                .contains("declares AssociatedConst")
        );
    }
}
