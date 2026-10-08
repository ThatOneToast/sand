//! Concrete game-object declarations over the canonical component-first model.
use quote::{format_ident, quote};
use sand_api_contract::syntax::{GeneratedApiKind, validate_generated_expansion};
use syn::{Data, DeriveInput, Fields, LitStr, Path, ext::IdentExt};

use crate::entity_state::{generated_contract, generated_contract_docs};

struct Declaration {
    id: LitStr,
    kind: Path,
    configure: Option<Path>,
}

impl Declaration {
    fn parse(input: &DeriveInput) -> syn::Result<Self> {
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                "[SAND-ARCHETYPE] concrete archetypes cannot be generic",
            ));
        }
        let mut id = None;
        let mut kind: Option<Path> = None;
        let mut configure = None;
        for attribute in input
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("archetype"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    if id.is_some() {
                        return Err(meta.error("duplicate archetype id"));
                    }
                    id = Some(meta.value()?.parse::<LitStr>()?);
                } else if meta.path.is_ident("entity") {
                    if kind.is_some() {
                        return Err(meta.error("duplicate archetype entity kind"));
                    }
                    kind = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("configure") {
                    if configure.is_some() {
                        return Err(meta.error("duplicate archetype configuration callback"));
                    }
                    configure = Some(meta.value()?.parse()?);
                } else {
                    return Err(meta.error("expected id, entity, or configure"));
                }
                Ok(())
            })?;
        }
        let id = id.ok_or_else(|| {
            syn::Error::new_spanned(
                &input.ident,
                "[SAND-ARCHETYPE] declare id = \"namespace:path\" once",
            )
        })?;
        let value = id.value();
        let valid = value.split_once(':').is_some_and(|(namespace, path)| {
            !namespace.is_empty()
                && !path.is_empty()
                && namespace.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
                })
                && path.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_/.-".contains(&byte)
                })
                && path
                    .split('/')
                    .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
        });
        if !valid {
            return Err(syn::Error::new_spanned(
                &id,
                "[SAND-ARCHETYPE] id must be a valid namespace:path resource identity",
            ));
        }
        let kind = kind.ok_or_else(|| {
            syn::Error::new_spanned(
                &input.ident,
                "[SAND-ARCHETYPE] declare an entity kind, such as entity = Zombie",
            )
        })?;
        let kind = if kind.is_ident("Zombie") {
            syn::parse_quote!(::sand::entity::ZombieKind)
        } else if kind.is_ident("Marker") {
            syn::parse_quote!(::sand::entity::MarkerKind)
        } else if kind.is_ident("Player") {
            syn::parse_quote!(::sand::entity::PlayerKind)
        } else {
            kind
        };
        Ok(Self {
            id,
            kind,
            configure,
        })
    }
}

pub(crate) fn derive(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let declaration = Declaration::parse(&input)?;
    let fields: Vec<_> = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter().collect(),
            Fields::Unit => vec![],
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &data.fields,
                    "[SAND-ARCHETYPE] use named State or StateBundle fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "[SAND-ARCHETYPE] only concrete structs can declare archetypes",
            ));
        }
    };
    let owner = &input.ident;
    let bound = format_ident!("{}Bound", owner);
    let visibility = &input.vis;
    let factory = format_ident!("__sand_archetype_{}_make", owner);
    let Declaration {
        id,
        kind,
        configure,
    } = declaration;
    let mut contracts = Vec::new();
    let mut document = |target: String,
                        api_kind,
                        summary: String,
                        params: &[(&str, &str)],
                        returns: Option<&str>,
                        example: String| {
        let contract = generated_contract(
            target,
            api_kind,
            summary,
            "A concrete archetype owns one entity kind and a named composition of reusable State components. Bound views refer to the current Minecraft executor, not persistent Rust entities.",
            "State retains its canonical storage and lifecycle; archetype operations use the existing initialization, shared-component cleanup, and native-binding compiler.",
            &["Declaring and using a concrete gameplay object"],
            &["Retaining a bound view after its generated execution scope"],
            params,
            returns,
            example,
        );
        let docs = generated_contract_docs(&contract);
        contracts.push(contract);
        docs
    };
    let bound_docs = document(
        bound.unraw().to_string(),
        GeneratedApiKind::Struct,
        format!("Execution-scoped component view of `{owner}`."),
        &[],
        None,
        format!("let object = {owner}::on(entity);"),
    );
    let mut bound_fields = Vec::new();
    let mut values = Vec::new();
    let mut components = Vec::new();
    let mut requirements = Vec::new();
    let mut scope_bounds = Vec::new();
    for field in &fields {
        let name = field.ident.as_ref().expect("named field");
        let ty = &field.ty;
        let docs = document(
            format!("{}::{}", bound.unraw(), name.unraw()),
            GeneratedApiKind::Field,
            format!("Access the declared `{name}` State component or nested bundle."),
            &[],
            Some("The component's existing bound view."),
            format!("let component = object.{name};"),
        );
        bound_fields
            .push(quote!(#docs pub #name: <#ty as ::sand::__private::StateBundleMember>::Bound));
        values
            .push(quote!(#name: <#ty as ::sand::__private::StateBundleMember>::bind_member("@s")));
        components.push(quote!(.components::<#ty>()));
        requirements.push(quote!(requirements.extend(<#ty as ::sand::__private::StateBundleMember>::presence_requirements());));
        scope_bounds.push(quote!(<#ty as ::sand::__private::StateBundleMember>::Scope: ::sand::__private::StateBundleTarget<#kind>));
    }
    let on_docs = document(format!("{}::on", owner.unraw()), GeneratedApiKind::Method,
        "Bind the declared fields to a current entity of the declared kind; this does not attach missing State.".into(),
        &[("_entity", "The typed current executor; use an archetype query or attach it before accessing State.")], Some("The concrete bound component view."), format!("let object = {owner}::on(entity);"));
    let entity_docs = document(
        format!("{bound}::entity"),
        GeneratedApiKind::Method,
        "Return the typed current entity context underlying this bound view.".into(),
        &[],
        Some("The same execution-scoped Minecraft executor."),
        "let entity = object.entity();".into(),
    );
    let configure = configure.map_or_else(
        || quote!(base.clone()),
        |callback| quote!(#callback(base.clone())),
    );
    let mut overlaps = Vec::new();
    for field in &fields {
        let ty = &field.ty;
        let message = format!(
            "[SAND-ARCHETYPE] `{owner}` field `{}` repeats a State component inside its nested composition",
            field.ident.as_ref().unwrap()
        );
        overlaps.push(quote! {
            const _: () = assert!(!::sand::__private::entity::state::state_bundle_tree_has_duplicates(
                &<#ty as ::sand::__private::StateBundleMember>::COMPONENT_TREE,
            ), #message);
        });
    }
    for (index, left) in fields.iter().enumerate() {
        for right in fields.iter().skip(index + 1) {
            let left_type = &left.ty;
            let right_type = &right.ty;
            let message = format!(
                "[SAND-ARCHETYPE] `{owner}` fields `{}` and `{}` contain the same State component",
                left.ident.as_ref().unwrap(),
                right.ident.as_ref().unwrap()
            );
            overlaps.push(quote! {
                const _: () = assert!(!::sand::__private::state_bundle_trees_overlap(
                    &<#left_type as ::sand::__private::StateBundleMember>::COMPONENT_TREE,
                    &<#right_type as ::sand::__private::StateBundleMember>::COMPONENT_TREE,
                ), #message);
            });
        }
    }
    let requirement_body = if requirements.is_empty() {
        quote!(Vec::new())
    } else {
        quote!({ let mut requirements = Vec::new(); #(#requirements)* requirements })
    };
    let expanded = quote! {
        #bound_docs
        #[derive(Debug, Clone, Copy)]
        #visibility struct #bound { #(#bound_fields),* }
        impl #bound {
            #entity_docs
            pub fn entity(&self) -> ::sand::entity::EntityContext<#kind> { Default::default() }
        }
        impl #owner {
            fn __sand_archetype_id() -> ::sand::prelude::ResourceLocation {
                #id.parse().expect("Archetype derive validated this resource identity")
            }
            fn __sand_archetype_requirements() -> Vec<(String, u32)> {
                #requirement_body
            }
            fn __sand_archetype_definition() -> Result<::sand::__private::entity::ArchetypeDefinition, ::sand::entity::EntityDiagnostic> {
                let base = ::sand::entity::EntityArchetype::<#kind>::new(Self::__sand_archetype_id()) #(#components)*;
                let configured = #configure;
                ::sand::__private::entity::archetype::concrete::configured_definition(base, configured)
            }
            #on_docs
            pub fn on(_entity: ::sand::entity::EntityContext<#kind>) -> #bound
            where #(#scope_bounds,)*
            { #bound { #(#values),* } }

        }
        impl ::sand::__private::entity::archetype::concrete::Declaration for #owner {
            type Kind = #kind;
            fn archetype_id() -> ::sand::prelude::ResourceLocation { Self::__sand_archetype_id() }
        }
        #(#overlaps)*
        impl ::sand::__private::GeneratedSystemQueryParameter for #owner {}
        impl ::sand::__private::StateQuerySpec for #owner {
            type Item = #bound;
            fn each(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
                ::sand::__private::lower_state_query_each(
                    ::sand::__private::entity::archetype::concrete::selection::<#kind>(&Self::__sand_archetype_id(), ::sand::__private::entity::archetype::concrete::Selector::all_entities()),
                    Self::__sand_archetype_requirements(), Vec::new(), Self::on(Default::default()), body,
                )
            }
            fn current(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
                ::sand::__private::lower_state_query_current(
                    Some(::sand::__private::entity::archetype::concrete::selection::<#kind>(&Self::__sand_archetype_id(), ::sand::__private::entity::archetype::concrete::Selector::self_())),
                    Self::__sand_archetype_requirements(), Vec::new(), Self::on(Default::default()), body,
                )
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #factory() -> Result<::sand::__private::entity::ArchetypeDefinition, ::sand::entity::EntityDiagnostic> {
            #owner::__sand_archetype_definition()
        }
        ::sand::__private::inventory::submit!(::sand::__private::entity::EntityArchetypeDescriptor { make: #factory });
    };
    if matches!(input.vis, syn::Visibility::Public(_)) {
        validate_generated_expansion(expanded.clone(), [owner.unraw().to_string()], &contracts)?;
    }
    Ok(expanded)
}
