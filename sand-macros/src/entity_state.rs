use std::collections::{BTreeMap, BTreeSet};

use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprLit, ExprPath, Fields, GenericArgument, Lit, LitInt, LitStr, Path,
    PathArguments, Type, ext::IdentExt, parse_quote,
};

pub(crate) fn derive_state(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "State schemas cannot be generic",
        ));
    }
    let config = parse_schema_config(&input)?;
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &data.fields,
                    "State requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "State can only be derived for a struct",
            ));
        }
    };

    let mut seen = BTreeSet::new();
    let mut constants = Vec::new();
    let mut descriptors = Vec::new();
    let mut bound_fields = Vec::new();
    let mut bound_values = Vec::new();
    let mut lifecycle_submissions = Vec::new();
    for field in fields {
        let ident = field.ident.as_ref().expect("named field");
        let field_name = ident.unraw().to_string();
        if !seen.insert(field_name.clone()) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("duplicate State field `{field_name}`"),
            ));
        }
        let attrs = parse_field_config(field)?;
        let wrapper = parse_wrapper(&field.ty)?;
        if !matches!(&wrapper, Wrapper::Score(_))
            && (attrs.criterion.is_some() || attrs.display_name.is_some())
        {
            return Err(syn::Error::new_spanned(
                field,
                "`criterion` and `display_name` are only valid on EntityScore fields",
            ));
        }
        if attrs.auto_tick && !matches!(&wrapper, Wrapper::Timer | Wrapper::Cooldown) {
            return Err(syn::Error::new_spanned(
                field,
                "`auto_tick` is only valid on EntityTimer and EntityCooldown fields",
            ));
        }
        if matches!(config.scope, Scope::Entity | Scope::Living) {
            if let Some(criterion) = &attrs.criterion {
                return Err(syn::Error::new_spanned(
                    criterion,
                    "`criterion` is only supported on player/global State score fields; entity/living criteria require later archetype dirty-observer integration",
                ));
            }
            if let Some(display_name) = &attrs.display_name {
                return Err(syn::Error::new_spanned(
                    display_name,
                    "`display_name` is only supported on player/global State score fields; entity/living display names require later archetype objective lowering",
                ));
            }
            if attrs.auto_tick {
                return Err(syn::Error::new_spanned(
                    field,
                    "`auto_tick` is only supported on player/global State fields; entity/living ticking is owned by archetype reconciliation",
                ));
            }
        }
        let namespace = &config.namespace;
        let schema_name = &config.name;
        let default = attrs.default.clone();
        let bounds = match (attrs.min, attrs.max) {
            (None, None) => quote!(None),
            (Some(min), Some(max)) => {
                if min > max {
                    return Err(syn::Error::new_spanned(
                        field,
                        "state field minimum must not exceed its maximum",
                    ));
                }
                quote!(Some((#min, #max)))
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    field,
                    "`min` and `max` must be specified together",
                ));
            }
        };

        let (constant, descriptor, accessor, lifecycle_default) = match wrapper {
            Wrapper::Score(ty) => {
                if attrs.kind.is_some() && !is_i32(&ty) {
                    return Err(syn::Error::new_spanned(
                        &field.ty,
                        "`kind = \"version\"` and `kind = \"dirty\"` require EntityScore<i32>",
                    ));
                }
                let kind = match attrs.kind.as_deref() {
                    None => quote!(::sand::__private::StateFieldKind::Score),
                    Some("version") => quote!(::sand::__private::StateFieldKind::Version),
                    Some("dirty") => quote!(::sand::__private::StateFieldKind::Dirty),
                    Some(other) => {
                        return Err(syn::Error::new_spanned(
                            field,
                            format!("unknown state field kind `{other}`"),
                        ));
                    }
                };
                let default = numeric_default(default, field)?;
                (
                    quote! {
                        #[doc = concat!("Typed handle for the `", #field_name, "` State field.")]
                        pub const #ident: ::sand::__private::EntityScore<#ty> =
                            ::sand::__private::EntityScore::__new(
                                #namespace, #schema_name, #field_name, #kind, #default, #bounds
                            );
                    },
                    quote! {
                        ::sand::__private::StateFieldDescriptor::new(
                            #field_name, #kind, #default, #bounds
                        )
                    },
                    quote!(::sand::__private::EntityScoreAccessor<#ty>),
                    quote!(#default),
                )
            }
            Wrapper::Flag => {
                reject_kind_or_bounds(&attrs, field)?;
                let default = boolean_default(default, field)?;
                (
                    quote! {
                        #[doc = concat!("Typed handle for the `", #field_name, "` State field.")]
                        pub const #ident: ::sand::__private::EntityFlag =
                            ::sand::__private::EntityFlag::new(
                                #namespace, #schema_name, #field_name, #default
                            );
                    },
                    quote! {
                        ::sand::__private::StateFieldDescriptor::new(
                            #field_name,
                            ::sand::__private::StateFieldKind::Flag,
                            #default as i32,
                            Some((0, 1)),
                        )
                    },
                    quote!(::sand::__private::EntityFlagAccessor),
                    quote!(#default as i32),
                )
            }
            Wrapper::Enum(ty) => {
                reject_kind_or_bounds(&attrs, field)?;
                let default = enum_default(default, &ty)?;
                (
                    quote! {
                        #[doc = concat!("Typed handle for the `", #field_name, "` State field.")]
                        pub const #ident: ::sand::__private::EntityEnum<#ty> =
                            ::sand::__private::EntityEnum::new(
                                #namespace, #schema_name, #field_name, #default
                            );
                    },
                    quote! {
                        ::sand::__private::StateFieldDescriptor::new(
                            #field_name,
                            ::sand::__private::StateFieldKind::Enum(
                                <#ty as ::sand::__private::EntityEnumValue>::ENCODINGS
                            ),
                            #default,
                            None,
                        )
                    },
                    quote!(::sand::__private::EntityEnumAccessor<#ty>),
                    quote!(#default),
                )
            }
            Wrapper::Timer => {
                reject_kind_or_bounds(&attrs, field)?;
                let default = numeric_default(default, field)?;
                if default < 0 {
                    return Err(syn::Error::new_spanned(
                        field,
                        "timer default cannot be negative",
                    ));
                }
                (
                    quote! {
                        #[doc = concat!("Typed handle for the `", #field_name, "` State field.")]
                        pub const #ident: ::sand::__private::EntityTimer =
                            ::sand::__private::EntityTimer::new(
                                #namespace, #schema_name, #field_name, #default
                            );
                    },
                    quote! {
                        ::sand::__private::StateFieldDescriptor::new(
                            #field_name,
                            ::sand::__private::StateFieldKind::Timer,
                            #default,
                            Some((0, i32::MAX)),
                        )
                    },
                    quote!(::sand::__private::EntityTimerAccessor),
                    quote!(#default),
                )
            }
            Wrapper::Cooldown => {
                reject_kind_or_bounds(&attrs, field)?;
                if default
                    .as_ref()
                    .map(parse_i32)
                    .transpose()?
                    .is_some_and(|value| value != 0)
                {
                    return Err(syn::Error::new_spanned(
                        field,
                        "EntityCooldown must be initialized ready; its default can only be zero",
                    ));
                }
                (
                    quote! {
                        #[doc = concat!("Typed handle for the `", #field_name, "` State field.")]
                        pub const #ident: ::sand::__private::EntityCooldown =
                            ::sand::__private::EntityCooldown::new(
                                #namespace, #schema_name, #field_name
                            );
                    },
                    quote! {
                        ::sand::__private::StateFieldDescriptor::new(
                            #field_name,
                            ::sand::__private::StateFieldKind::Cooldown,
                            0,
                            Some((0, i32::MAX)),
                        )
                    },
                    quote!(::sand::__private::EntityCooldownAccessor),
                    quote!(0),
                )
            }
        };
        constants.push(constant);
        descriptors.push(descriptor);
        bound_fields.push(quote! {
            #[doc = concat!("Bound accessor for the `", #field_name, "` state field.")]
            pub #ident: #accessor
        });
        let track_dirty = matches!(config.scope, Scope::Entity | Scope::Living);
        bound_values.push(quote! {
            #ident: ::sand::__private::EntityStateField::bind_to(
                Self::#ident,
                holder,
                #track_dirty,
            )
        });

        if matches!(config.scope, Scope::Player | Scope::Global) {
            let logical_objective = LitStr::new(
                &format!(
                    "{}:{}.{}",
                    namespace.value(),
                    schema_name.value(),
                    field_name
                ),
                ident.span(),
            );
            let criterion = attrs
                .criterion
                .as_ref()
                .map(|criterion| quote!(.criterion(#criterion)))
                .unwrap_or_default();
            let display_name = attrs
                .display_name
                .as_ref()
                .map(|display_name| quote!(.display_name(#display_name)))
                .unwrap_or_default();
            let lifecycle = if attrs.auto_tick {
                quote! {
                    ::sand::__private::StateLifecycle::score(#logical_objective)
                        #criterion
                        #display_name
                        .default(#lifecycle_default)
                        .auto_tick()
                }
            } else {
                quote! {
                    ::sand::__private::StateLifecycle::score(#logical_objective)
                        #criterion
                        #display_name
                        .default(#lifecycle_default)
                }
            };
            let lifecycle = if matches!(config.scope, Scope::Global) {
                let holder = LitStr::new(
                    &global_holder(&namespace.value(), &schema_name.value()),
                    ident.span(),
                );
                quote! { (#lifecycle).global(#holder) }
            } else {
                lifecycle
            };
            lifecycle_submissions.push(quote! {
                const _: () = {
                    ::sand::__private::inventory::submit! {
                        ::sand::__private::StateDescriptor::new(#lifecycle)
                    }
                };
            });
        }
    }

    let ident = &input.ident;
    let visibility = &input.vis;
    let bound_ident = quote::format_ident!("{}Bound", ident);
    let namespace = &config.namespace;
    let name = &config.name;
    let version = config.version;
    let binding_method = match config.scope {
        Scope::Player => quote! {
            /// Bind this schema to the current execution-scoped player.
            pub fn on(
                _target: ::sand::__private::EntityContext<::sand::__private::PlayerKind>
            ) -> #bound_ident {
                Self::__sand_bind_to("@s")
            }
        },
        Scope::Entity => quote! {
            /// Bind this schema to the current execution-scoped entity.
            pub fn on<K: ::sand::__private::EntityKind>(
                _target: ::sand::__private::EntityContext<K>
            ) -> #bound_ident {
                Self::__sand_bind_to("@s")
            }
        },
        Scope::Living => quote! {
            /// Bind this schema to the current execution-scoped living entity.
            pub fn on<K: ::sand::__private::LivingEntityKind>(
                _target: ::sand::__private::EntityContext<K>
            ) -> #bound_ident {
                Self::__sand_bind_to("@s")
            }
        },
        Scope::Global => {
            let holder = LitStr::new(
                &global_holder(&namespace.value(), &name.value()),
                ident.span(),
            );
            quote! {
                /// Bind this schema to its deterministic global fake-player holder.
                pub fn global() -> #bound_ident {
                    Self::__sand_bind_to(#holder)
                }
            }
        }
    };
    Ok(quote! {
        #[derive(Debug, Clone, Copy)]
        #visibility struct #bound_ident {
            #(#bound_fields),*
        }

        #[allow(non_upper_case_globals)]
        impl #ident {
            #(#constants)*

            /// Field metadata in declaration order.
            pub const FIELDS: &'static [::sand::__private::StateFieldDescriptor] = &[
                #(#descriptors),*
            ];

            fn __sand_bind_to(holder: &'static str) -> #bound_ident {
                #bound_ident {
                    #(#bound_values),*
                }
            }

            #binding_method
        }

        impl ::sand::__private::EntityState for #ident {
            fn schema() -> ::sand::__private::StateSchema {
                ::sand::__private::StateSchema {
                    namespace: #namespace,
                    name: #name,
                    version: #version,
                    fields: Self::FIELDS,
                }
            }
        }

        #(#lifecycle_submissions)*
    })
}

pub(crate) fn derive_enum(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "EntityStateEnum cannot be generic",
        ));
    }
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "EntityStateEnum can only be derived for an enum",
            ));
        }
    };
    if variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "EntityStateEnum requires at least one variant",
        ));
    }

    let mut next = 0i32;
    let mut scores = BTreeMap::new();
    let mut encodings = Vec::new();
    let mut arms = Vec::new();
    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "EntityStateEnum variants must not have fields",
            ));
        }
        let score = if let Some((_, expression)) = &variant.discriminant {
            parse_i32(expression)?
        } else {
            next
        };
        if let Some(previous) = scores.insert(score, variant.ident.to_string()) {
            return Err(syn::Error::new_spanned(
                variant,
                format!("duplicate EntityStateEnum score {score}; already used by `{previous}`"),
            ));
        }
        next = score.checked_add(1).ok_or_else(|| {
            syn::Error::new_spanned(
                variant,
                "implicit EntityStateEnum discriminant overflows i32",
            )
        })?;
        let variant_ident = &variant.ident;
        let name = variant_ident.to_string();
        encodings.push(quote! {
            ::sand::__private::EnumEncoding { name: #name, score: #score }
        });
        arms.push(quote!(Self::#variant_ident => #score));
    }
    let ident = &input.ident;
    Ok(quote! {
        impl ::sand::__private::EntityEnumValue for #ident {
            const ENCODINGS: &'static [::sand::__private::EnumEncoding] = &[
                #(#encodings),*
            ];

            fn encode(self) -> i32 {
                match self {
                    #(#arms),*
                }
            }
        }
    })
}

struct SchemaConfig {
    namespace: LitStr,
    name: LitStr,
    version: u32,
    scope: Scope,
}

#[derive(Clone, Copy)]
enum Scope {
    Player,
    Entity,
    Living,
    Global,
}

fn parse_schema_config(input: &DeriveInput) -> syn::Result<SchemaConfig> {
    let attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("state"))
        .collect();
    if attrs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "exactly one #[state(namespace = \"...\", scope = player|entity|living|global, ...)] schema attribute is required",
        ));
    }
    let mut namespace = None;
    let mut name = None;
    let mut version = None;
    let mut scope = None;
    attrs[0].parse_nested_meta(|meta| {
        if meta.path.is_ident("namespace") {
            set_once(&mut namespace, meta.value()?.parse()?, &meta.path)
        } else if meta.path.is_ident("name") {
            set_once(&mut name, meta.value()?.parse()?, &meta.path)
        } else if meta.path.is_ident("version") {
            let literal: LitInt = meta.value()?.parse()?;
            set_once(&mut version, literal.base10_parse()?, &meta.path)
        } else if meta.path.is_ident("scope") {
            let value: syn::Ident = meta.value()?.parse()?;
            let parsed = match value.to_string().as_str() {
                "player" => Scope::Player,
                "entity" => Scope::Entity,
                "living" => Scope::Living,
                "global" => Scope::Global,
                _ => {
                    return Err(syn::Error::new_spanned(
                        value,
                        "invalid state scope; expected player, entity, living, or global",
                    ));
                }
            };
            set_once(&mut scope, parsed, &meta.path)
        } else {
            Err(meta.error("unknown state schema option"))
        }
    })?;
    let namespace: LitStr = namespace.ok_or_else(|| {
        syn::Error::new_spanned(attrs[0], "state schema requires `namespace = \"...\"`")
    })?;
    let name = name.unwrap_or_else(|| {
        LitStr::new(
            &to_snake_case(&input.ident.unraw().to_string()),
            input.ident.span(),
        )
    });
    let scope = scope.ok_or_else(|| {
        syn::Error::new_spanned(
            attrs[0],
            "state schema requires `scope = player|entity|living|global`",
        )
    })?;
    validate_resource_part(&namespace, false)?;
    validate_resource_part(&name, true)?;
    let version = version.unwrap_or(1);
    if version == 0 {
        return Err(syn::Error::new_spanned(
            attrs[0],
            "State schema version zero is reserved for uninitialized entities",
        ));
    }
    Ok(SchemaConfig {
        namespace,
        name,
        version,
        scope,
    })
}

#[derive(Default)]
struct FieldConfig {
    default: Option<Expr>,
    min: Option<i32>,
    max: Option<i32>,
    kind: Option<String>,
    criterion: Option<LitStr>,
    display_name: Option<LitStr>,
    auto_tick: bool,
}

fn parse_field_config(field: &syn::Field) -> syn::Result<FieldConfig> {
    let mut result = FieldConfig::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("state") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                let expression: Expr = meta.value()?.parse()?;
                set_once(&mut result.default, expression, &meta.path)
            } else if meta.path.is_ident("min") || meta.path.is_ident("max") {
                let expression: Expr = meta.value()?.parse()?;
                let value = parse_i32(&expression)?;
                if meta.path.is_ident("min") {
                    set_once(&mut result.min, value, &meta.path)
                } else {
                    set_once(&mut result.max, value, &meta.path)
                }
            } else if meta.path.is_ident("kind") {
                let value: LitStr = meta.value()?.parse()?;
                set_once(&mut result.kind, value.value(), &meta.path)
            } else if meta.path.is_ident("criterion") {
                let value: LitStr = meta.value()?.parse()?;
                validate_criterion(&value)?;
                set_once(&mut result.criterion, value, &meta.path)
            } else if meta.path.is_ident("display_name") {
                let value: LitStr = meta.value()?.parse()?;
                if value.value().chars().any(char::is_control) {
                    return Err(syn::Error::new_spanned(
                        value,
                        "state objective display name must not contain control characters",
                    ));
                }
                set_once(&mut result.display_name, value, &meta.path)
            } else if meta.path.is_ident("auto_tick") {
                if result.auto_tick {
                    Err(meta.error("duplicate option"))
                } else {
                    result.auto_tick = true;
                    Ok(())
                }
            } else {
                Err(meta.error("unknown state field option"))
            }
        })?;
    }
    Ok(result)
}

fn validate_criterion(value: &LitStr) -> syn::Result<()> {
    let criterion = value.value();
    if criterion.is_empty()
        || !criterion
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '-'))
    {
        return Err(syn::Error::new_spanned(
            value,
            "state objective criterion must be non-empty and contain only ASCII letters, digits, `_`, `.`, `:`, or `-`",
        ));
    }
    Ok(())
}

fn to_snake_case(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}

fn global_holder(namespace: &str, schema: &str) -> String {
    let logical = format!("{namespace}:{schema}");
    let clean: String = logical
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
                ch
            } else {
                '_'
            }
        })
        .take(24)
        .collect();
    let mut hash = 14_695_981_039_346_656_037u64;
    for byte in logical.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("#sand_{clean}_{:08x}", hash as u32)
}

enum Wrapper {
    Score(Type),
    Flag,
    Enum(Type),
    Timer,
    Cooldown,
}

fn parse_wrapper(ty: &Type) -> syn::Result<Wrapper> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "state field type must be an EntityScore, EntityFlag, EntityEnum, EntityTimer, or EntityCooldown",
        ));
    };
    let segment = type_path.path.segments.last().expect("type path segment");
    match segment.ident.to_string().as_str() {
        "EntityScore" => Ok(Wrapper::Score(single_type_argument(
            segment,
            parse_quote!(i32),
        )?)),
        "EntityEnum" => Ok(Wrapper::Enum(required_type_argument(segment)?)),
        "EntityFlag" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Flag)
        }
        "EntityTimer" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Timer)
        }
        "EntityCooldown" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Cooldown)
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "unsupported State field wrapper; expected EntityScore<T>, EntityFlag, EntityEnum<T>, EntityTimer, or EntityCooldown",
        )),
    }
}

fn single_type_argument(segment: &syn::PathSegment, default: Type) -> syn::Result<Type> {
    match &segment.arguments {
        PathArguments::None => Ok(default),
        PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
            match args.args.first().expect("one argument") {
                GenericArgument::Type(ty) => Ok(ty.clone()),
                other => Err(syn::Error::new_spanned(other, "expected a type argument")),
            }
        }
        args => Err(syn::Error::new_spanned(
            args,
            "expected exactly one type argument",
        )),
    }
}

fn required_type_argument(segment: &syn::PathSegment) -> syn::Result<Type> {
    match &segment.arguments {
        PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
            match args.args.first().expect("one argument") {
                GenericArgument::Type(ty) => Ok(ty.clone()),
                other => Err(syn::Error::new_spanned(
                    other,
                    "expected an enum type argument",
                )),
            }
        }
        args => Err(syn::Error::new_spanned(
            args,
            "EntityEnum requires exactly one enum type argument",
        )),
    }
}

fn reject_arguments(segment: &syn::PathSegment) -> syn::Result<()> {
    if matches!(segment.arguments, PathArguments::None) {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            &segment.arguments,
            "this state field wrapper does not accept type arguments",
        ))
    }
}

fn is_i32(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "i32" && segment.arguments.is_empty())
}

fn numeric_default(default: Option<Expr>, field: &syn::Field) -> syn::Result<i32> {
    default.as_ref().map_or(Ok(0), parse_i32).map_err(|_| {
        syn::Error::new_spanned(field, "this state field requires an i32 literal default")
    })
}

fn boolean_default(default: Option<Expr>, field: &syn::Field) -> syn::Result<bool> {
    match default {
        None => Ok(false),
        Some(Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        })) => Ok(value.value),
        _ => Err(syn::Error::new_spanned(
            field,
            "EntityFlag default must be `true` or `false`",
        )),
    }
}

fn enum_default(default: Option<Expr>, ty: &Type) -> syn::Result<proc_macro2::TokenStream> {
    let Some(default) = default else {
        return Ok(quote!(<#ty as ::sand::__private::EntityEnumValue>::ENCODINGS[0].score));
    };
    let path: Path = match default {
        Expr::Path(ExprPath { path, .. }) => path,
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => value.parse()?,
        _ => {
            return Err(syn::Error::new_spanned(
                default,
                "enum default must be a variant path or quoted variant path",
            ));
        }
    };
    Ok(quote!(#path as i32))
}

fn parse_i32(expression: &Expr) -> syn::Result<i32> {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_parse(),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            let value = parse_i32(&unary.expr)?;
            value.checked_neg().ok_or_else(|| {
                syn::Error::new_spanned(expression, "integer is outside the i32 range")
            })
        }
        _ => Err(syn::Error::new_spanned(
            expression,
            "expected an i32 integer literal",
        )),
    }
}

fn reject_kind_or_bounds(config: &FieldConfig, field: &syn::Field) -> syn::Result<()> {
    if config.kind.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "`kind` is only valid on EntityScore fields",
        ));
    }
    if config.min.is_some() || config.max.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "`min` and `max` are only valid on EntityScore fields",
        ));
    }
    Ok(())
}

fn validate_resource_part(value: &LitStr, path: bool) -> syn::Result<()> {
    let text = value.value();
    let valid = !text.is_empty()
        && text.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.')
                || (path && byte == b'/')
        });
    if valid {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            value,
            if path {
                "State schema name must use lowercase resource-path characters [a-z0-9_./-]"
            } else {
                "State schema namespace must use lowercase characters [a-z0-9_.-]"
            },
        ))
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, path: &Path) -> syn::Result<()> {
    if slot.replace(value).is_some() {
        Err(syn::Error::new_spanned(path, "duplicate option"))
    } else {
        Ok(())
    }
}
