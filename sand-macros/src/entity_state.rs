use std::collections::{BTreeMap, BTreeSet};

use quote::quote;
use sand_api_contract::syntax::{
    GeneratedApiContract, GeneratedApiKind, render_generated_rustdoc, validate_generated_expansion,
};
use syn::{
    Data, DeriveInput, Expr, ExprLit, ExprPath, Fields, GenericArgument, Lit, LitInt, LitStr, Path,
    PathArguments, Type, ext::IdentExt, parse_quote, spanned::Spanned,
};

pub(crate) fn derive_state(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "State schemas cannot be generic",
        ));
    }
    let config = parse_schema_config(&input)?;
    let fields: Vec<&syn::Field> = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter().collect(),
            Fields::Unit => Vec::new(),
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &data.fields,
                    "State requires named fields or a unit struct marker",
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
    let owner_ident = &input.ident;
    let bound_ident = quote::format_ident!("{}Bound", owner_ident);

    let mut seen = BTreeSet::new();
    let mut constants = Vec::new();
    let mut descriptors = Vec::new();
    let mut bound_fields = Vec::new();
    let mut bound_values = Vec::new();
    let mut lifecycle_fields = Vec::new();
    let mut auto_tick_objectives = Vec::new();
    let mut data_descriptors = Vec::new();
    let mut generated_contracts = Vec::new();
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
        if !matches!(&wrapper, Wrapper::Score(_) | Wrapper::Fixed)
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
        if attrs.scale.is_some() && !matches!(&wrapper, Wrapper::Fixed) {
            return Err(syn::Error::new_spanned(
                field,
                "`scale` is only valid on FixedScore State fields",
            ));
        }
        let namespace = &config.namespace;
        let schema_name = &config.name;
        if let Wrapper::Data(ty) = &wrapper {
            reject_kind_or_bounds(&attrs, field)?;
            let default_snbt = data_default_snbt(&attrs, field)?;
            let storage = LitStr::new(&format!("{}:state", namespace.value()), field.span());
            let path = LitStr::new(
                &format!(
                    "components.{}.{}",
                    quoted_nbt_key(&schema_name.value()),
                    field_name
                ),
                field.span(),
            );
            let keyed = !matches!(config.scope, Scope::Global);
            let handle = if keyed {
                quote!(::sand::__private::KeyedData<#ty>)
            } else {
                quote!(::sand::__private::Data<#ty>)
            };
            let constructor = if keyed {
                quote!(::sand::__private::KeyedData::new(#storage, #path))
            } else {
                quote!(::sand::__private::Data::new(#storage, #path))
            };
            let descriptor = if keyed {
                quote!(::sand::__private::StateDataFieldDescriptor::keyed(
                    #storage,
                    #path,
                    #default_snbt
                ))
            } else {
                quote!(::sand::__private::StateDataFieldDescriptor::new(
                    #storage,
                    #path,
                    #default_snbt
                ))
            };
            let constant_contract = generated_contract(
                format!("{}::{field_name}", owner_ident.unraw()),
                GeneratedApiKind::AssociatedConst,
                format!("Provides the typed Data handle for the `{field_name}` State field."),
                "The handle addresses one component-owned typed command-storage path, keyed by owner UUID outside global scope.",
                "Reads and writes isolated command storage without using unreliable custom entity NBT.",
                &["Accessing structured State data through typed command storage"],
                &["Storing arbitrary custom data on an entity's top-level native NBT"],
                &[],
                Some("The definition-owned typed storage field handle."),
                format!("let field = {}::{field_name};", owner_ident.unraw()),
            );
            let constant_docs = generated_contract_docs(&constant_contract);
            generated_contracts.push(constant_contract);
            constants.push(quote! {
                #constant_docs
                pub const #ident: #handle = #constructor;
            });
            data_descriptors.push(descriptor);
            let bound_contract = generated_contract(
                format!("{}Bound::{field_name}", owner_ident.unraw()),
                GeneratedApiKind::Field,
                format!("Provides the bound typed Data accessor for `{field_name}`."),
                "State data retains its deterministic component-owned path and, when scoped, the current owner's UUID key.",
                "Operations lower to typed command-storage commands for the isolated path.",
                &["Reading or replacing structured State data"],
                &["Using entity-native NBT as custom persistent component storage"],
                &[],
                Some("The typed component-owned storage accessor."),
                format!("let value = bound.{field_name};"),
            );
            let bound_docs = generated_contract_docs(&bound_contract);
            generated_contracts.push(bound_contract);
            bound_fields.push(quote! {
                #bound_docs
                pub #ident: #handle
            });
            bound_values.push(quote!(#ident: Self::#ident));
            continue;
        }
        if attrs.default_snbt.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "`default_snbt` is only valid on Data<T> State fields",
            ));
        }
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

        let (wrapper_name, wrapper_behavior) = match &wrapper {
            Wrapper::Score(_) => (
                "score",
                "Reads and writes the schema field through its validated scoreboard objective.",
            ),
            Wrapper::Fixed => (
                "fixed score",
                "Encodes decimal values as deterministic scaled scoreboard integers.",
            ),
            Wrapper::Flag => (
                "flag",
                "Stores the schema field as a boolean scoreboard value with zero and one encodings.",
            ),
            Wrapper::Enum(_) => (
                "enum",
                "Stores the schema field as the stable scoreboard encoding derived for its Rust enum.",
            ),
            Wrapper::Timer => (
                "timer",
                "Stores the schema field as a non-negative scoreboard timer value.",
            ),
            Wrapper::Cooldown => (
                "cooldown",
                "Stores the schema field as a non-negative scoreboard cooldown whose zero value is ready.",
            ),
            Wrapper::Data(_) => unreachable!("Data fields return before scoreboard lowering"),
        };
        let (constant, descriptor, accessor, _lifecycle_default) = match wrapper {
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
                            ::sand::__private::entity_score_new(
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
            Wrapper::Fixed => {
                if attrs.kind.is_some() {
                    return Err(syn::Error::new_spanned(
                        field,
                        "FixedScore does not accept an internal state field kind",
                    ));
                }
                let scale = attrs.scale.unwrap_or(1_000);
                if scale == 0 || scale > i32::MAX as u32 {
                    return Err(syn::Error::new_spanned(
                        field,
                        "FixedScore scale must be between 1 and 2147483647",
                    ));
                }
                let scale = scale as i32;
                let default = fixed_default(default, scale, field)?;
                let fixed_bounds = match (attrs.min, attrs.max) {
                    (None, None) => quote!(None),
                    (Some(min), Some(max)) => {
                        if min > max {
                            return Err(syn::Error::new_spanned(
                                field,
                                "state field minimum must not exceed its maximum",
                            ));
                        }
                        let min = scale_fixed_integer(min, scale, field)?;
                        let max = scale_fixed_integer(max, scale, field)?;
                        quote!(Some((#min, #max)))
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            field,
                            "`min` and `max` must be specified together",
                        ));
                    }
                };
                (
                    quote! {
                        #[doc = concat!("Typed fixed-point handle for the `", #field_name, "` State field.")]
                        pub const #ident: ::sand::__private::FixedScore =
                            ::sand::__private::fixed_score_new(
                                #namespace, #schema_name, #field_name, #scale, #default, #fixed_bounds
                            );
                    },
                    quote! {
                        ::sand::__private::StateFieldDescriptor::new(
                            #field_name,
                            ::sand::__private::StateFieldKind::Fixed(#scale),
                            #default,
                            #fixed_bounds,
                        )
                    },
                    quote!(::sand::__private::FixedScoreAccessor),
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
            Wrapper::Data(_) => unreachable!("Data fields return before scoreboard lowering"),
        };
        let constant_contract = generated_contract(
            format!("{}::{field_name}", owner_ident.unraw()),
            GeneratedApiKind::AssociatedConst,
            format!("Provides the typed {wrapper_name} handle for the `{field_name}` State field."),
            "Use this definition-owned handle to compose state operations before binding them to a concrete entity or global holder.",
            wrapper_behavior,
            &[
                "Referring to this State field in typed conditions, commands, or archetype configuration",
            ],
            &[
                "Accessing a concrete entity value; bind the schema first and use the bound accessor",
            ],
            &[],
            Some("The definition-owned typed field handle."),
            format!("let field = {}::{field_name};", owner_ident.unraw()),
        );
        let constant_docs = generated_contract_docs(&constant_contract);
        generated_contracts.push(constant_contract);
        constants.push(quote!(#constant_docs #constant));
        descriptors.push(descriptor.clone());
        let bound_contract = generated_contract(
            format!("{}Bound::{field_name}", owner_ident.unraw()),
            GeneratedApiKind::Field,
            format!(
                "Provides the bound {wrapper_name} accessor for the `{field_name}` State field."
            ),
            "This accessor carries the concrete scoreboard holder selected by State::on or State::global, so reads and writes target the intended entity or global schema instance.",
            wrapper_behavior,
            &[
                "Reading, updating, or testing this field for the schema instance that was just bound",
            ],
            &[
                "Declaring schema metadata without a concrete holder; use the definition-owned field handle",
            ],
            &[],
            Some("The holder-bound typed field accessor."),
            format!("let value = bound.{field_name};"),
        );
        let bound_docs = generated_contract_docs(&bound_contract);
        generated_contracts.push(bound_contract);
        bound_fields.push(quote! {
            #bound_docs
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
            .map(|value| quote!(.criterion(#value)))
            .unwrap_or_default();
        let display_name = attrs
            .display_name
            .as_ref()
            .map(|value| quote!(.display_name(#value)))
            .unwrap_or_default();
        let auto_tick = if attrs.auto_tick {
            auto_tick_objectives.push(quote!(
                ::sand::__private::resolve_state_objective(#logical_objective)
            ));
            quote!(.auto_tick())
        } else {
            quote!()
        };
        lifecycle_fields.push(quote! {
            ::sand::__private::StateLifecycleDescriptor::new(#logical_objective, #descriptor)
                #criterion
                #display_name
                #auto_tick
        });
    }

    let ident = owner_ident;
    let visibility = &input.vis;
    let namespace = &config.namespace;
    let name = &config.name;
    let version = config.version;
    let component_id = LitStr::new(
        &format!("{}:{}", namespace.value(), name.value()),
        input.ident.span(),
    );
    let presence_objective = LitStr::new(
        &format!("{}:{}.presence", namespace.value(), name.value()),
        input.ident.span(),
    );
    let suppression_objective = LitStr::new(
        &format!("{}:{}.suppressed", namespace.value(), name.value()),
        input.ident.span(),
    );
    let lifecycle_scope = match config.scope {
        Scope::Player => quote!(::sand::__private::StateScope::Player),
        Scope::Entity => quote!(::sand::__private::StateScope::Entity),
        Scope::Living => quote!(::sand::__private::StateScope::Living),
        Scope::Global => {
            let holder = LitStr::new(
                &global_holder(&namespace.value(), &name.value()),
                input.ident.span(),
            );
            quote!(::sand::__private::StateScope::Global(#holder))
        }
    };
    let scope_marker = match config.scope {
        Scope::Player => quote!(::sand::__private::PlayerStateScope),
        Scope::Entity => quote!(::sand::__private::EntityStateScope),
        Scope::Living => quote!(::sand::__private::LivingStateScope),
        Scope::Global => quote!(::sand::__private::GlobalStateScope),
    };
    let default_member_holder = if matches!(config.scope, Scope::Global) {
        let holder = LitStr::new(
            &global_holder(&namespace.value(), &name.value()),
            input.ident.span(),
        );
        quote!(#holder)
    } else {
        quote!("@s")
    };
    let suppress_player_observation = matches!(config.scope, Scope::Player);
    let migration_steps = config
        .migrations
        .iter()
        .map(|(from, to)| quote!(::sand::__private::StateMigrationDescriptor::new(#from, #to)));
    let lifecycle_registration = quote! {
        const _: () = {
            ::sand::__private::inventory::submit! {
                ::sand::__private::StateDescriptor::new(
                    #component_id,
                    #version,
                    #lifecycle_scope,
                    #presence_objective,
                    #suppression_objective,
                    #ident::__SAND_LIFECYCLE_FIELDS,
                    #ident::__SAND_MIGRATIONS,
                    #ident::__SAND_DATA_FIELDS,
                )
            }
        };
    };
    let (binding_contract, binding_method) = match config.scope {
        Scope::Player => {
            let contract = generated_contract(
                format!("{}::on", ident.unraw()),
                GeneratedApiKind::Method,
                "Binds this schema to the current execution-scoped player.",
                "The returned bound view makes every generated field accessor target the player represented by the supplied typed context.",
                "Uses the current player selector as the scoreboard holder for all schema field operations.",
                &[
                    "Reading or mutating persistent player state inside player-scoped command generation",
                ],
                &[
                    "Binding entity, living, or global state; derive the schema with the matching scope",
                ],
                &[(
                    "_target",
                    "The typed current-player context that proves player scope.",
                )],
                Some("The schema view bound to the current player."),
                format!("let bound = {}::on(player);", ident.unraw()),
            );
            let docs = generated_contract_docs(&contract);
            (
                contract,
                quote! {
                    #docs
                    /// Bind this schema to the current execution-scoped player.
                    pub fn on(
                        _target: ::sand::__private::EntityContext<::sand::__private::PlayerKind>
                    ) -> #bound_ident {
                        Self::__sand_bind_to("@s")
                    }
                },
            )
        }
        Scope::Entity => {
            let contract = generated_contract(
                format!("{}::on", ident.unraw()),
                GeneratedApiKind::Method,
                "Binds this schema to the current execution-scoped entity.",
                "The returned bound view carries the current entity holder while retaining the caller's typed EntityKind evidence.",
                "Uses the current entity selector as the scoreboard holder and enables dirty tracking for archetype reconciliation.",
                &["Reading or mutating state on the current typed entity"],
                &["Binding player-only or global state; derive the schema with the matching scope"],
                &[(
                    "_target",
                    "The typed entity context that proves the current execution target.",
                )],
                Some("The schema view bound to the current entity."),
                format!("let bound = {}::on(entity);", ident.unraw()),
            );
            let docs = generated_contract_docs(&contract);
            (
                contract,
                quote! {
                    #docs
                    /// Bind this schema to the current execution-scoped entity.
                    pub fn on<K: ::sand::__private::EntityKind>(
                        _target: ::sand::__private::EntityContext<K>
                    ) -> #bound_ident {
                        Self::__sand_bind_to("@s")
                    }
                },
            )
        }
        Scope::Living => {
            let contract = generated_contract(
                format!("{}::on", ident.unraw()),
                GeneratedApiKind::Method,
                "Binds this schema to the current execution-scoped living entity.",
                "The returned bound view carries the current living-entity holder and preserves the LivingEntityKind capability required by this schema scope.",
                "Uses the current living entity as the scoreboard holder and enables dirty tracking for archetype reconciliation.",
                &["Reading or mutating state that requires a living entity context"],
                &[
                    "Binding non-living entity or global state; derive the schema with the matching scope",
                ],
                &[(
                    "_target",
                    "The typed living-entity context that proves the current target capability.",
                )],
                Some("The schema view bound to the current living entity."),
                format!("let bound = {}::on(living);", ident.unraw()),
            );
            let docs = generated_contract_docs(&contract);
            (
                contract,
                quote! {
                    #docs
                    /// Bind this schema to the current execution-scoped living entity.
                    pub fn on<K: ::sand::__private::LivingEntityKind>(
                        _target: ::sand::__private::EntityContext<K>
                    ) -> #bound_ident {
                        Self::__sand_bind_to("@s")
                    }
                },
            )
        }
        Scope::Global => {
            let holder = LitStr::new(
                &global_holder(&namespace.value(), &name.value()),
                ident.span(),
            );
            let contract = generated_contract(
                format!("{}::global", ident.unraw()),
                GeneratedApiKind::Method,
                "Binds this schema to its deterministic global holder.",
                "The returned bound view targets one schema-wide fake player, making the generated fields shared across every executing entity.",
                "Uses Sand's deterministic fake-player holder for the schema's scoreboard-backed values.",
                &["Reading or mutating datapack-wide persistent state"],
                &[
                    "Each player or entity requires an independent value; derive a scoped schema instead",
                ],
                &[],
                Some("The schema view bound to its global fake-player holder."),
                format!("let bound = {}::global();", ident.unraw()),
            );
            let docs = generated_contract_docs(&contract);
            (
                contract,
                quote! {
                    #docs
                    /// Bind this schema to its deterministic global fake-player holder.
                    pub fn global() -> #bound_ident {
                        Self::__sand_bind_to(#holder)
                    }
                },
            )
        }
    };
    generated_contracts.push(binding_contract);
    let mut presence_docs = Vec::new();
    let presence_parameters: &[(&str, &str)] = if matches!(config.scope, Scope::Global) {
        &[]
    } else {
        &[(
            "_target",
            "The typed current owner context for this component.",
        )]
    };
    for (method, summary) in [
        (
            "attach",
            "Attaches and idempotently initializes this State component.",
        ),
        (
            "detach",
            "Detaches this State component without touching unrelated state.",
        ),
        (
            "is_attached",
            "Tests whether this State component is present at its current version.",
        ),
    ] {
        let contract = generated_contract(
            format!("{}::{method}", ident.unraw()),
            GeneratedApiKind::Method,
            summary,
            "The generated component lifecycle uses its independent version marker and ownership boundary.",
            "Emits or tests scoreboard state for the bound owner; dependencies are provisioned by Sand's load lifecycle.",
            &["Managing component presence through the canonical State lifecycle"],
            &["Removing shared objectives or state owned by another component"],
            presence_parameters,
            Some("Generated commands or a typed runtime presence condition."),
            format!("let _ = {}::{method}(...);", ident.unraw()),
        );
        presence_docs.push(generated_contract_docs(&contract));
        generated_contracts.push(contract);
    }
    let attach_docs = &presence_docs[0];
    let detach_docs = &presence_docs[1];
    let attached_docs = &presence_docs[2];
    let track_dirty = matches!(config.scope, Scope::Entity | Scope::Living);
    let presence_methods = match config.scope {
        Scope::Player => quote! {
            #attach_docs
            /// Attach this component to the current player.
            pub fn attach(_target: ::sand::__private::EntityContext<::sand::__private::PlayerKind>) -> Vec<String> {
                ::sand::__private::state_attach_commands::<Self>("@s", #presence_objective, #suppression_objective, true)
            }
            #detach_docs
            /// Explicitly detach this component and suppress automatic re-observation.
            pub fn detach(_target: ::sand::__private::EntityContext<::sand::__private::PlayerKind>) -> Vec<String> {
                ::sand::__private::state_detach_commands::<Self>("@s", #presence_objective, #suppression_objective, false, true)
            }
            #attached_docs
            /// Test whether this component is attached.
            pub fn is_attached(_target: ::sand::__private::EntityContext<::sand::__private::PlayerKind>) -> ::sand::__private::condition::Condition {
                ::sand::__private::state_attached_condition::<Self>("@s", #presence_objective)
            }
        },
        Scope::Entity => quote! {
            #attach_docs
            /// Attach this component to the current entity.
            pub fn attach<K: ::sand::__private::EntityKind>(_target: ::sand::__private::EntityContext<K>) -> Vec<String> {
                ::sand::__private::state_attach_commands::<Self>("@s", #presence_objective, #suppression_objective, false)
            }
            #detach_docs
            /// Detach this component from the current entity.
            pub fn detach<K: ::sand::__private::EntityKind>(_target: ::sand::__private::EntityContext<K>) -> Vec<String> {
                ::sand::__private::state_detach_commands::<Self>("@s", #presence_objective, #suppression_objective, #track_dirty, false)
            }
            #attached_docs
            /// Test whether this component is attached.
            pub fn is_attached<K: ::sand::__private::EntityKind>(_target: ::sand::__private::EntityContext<K>) -> ::sand::__private::condition::Condition {
                ::sand::__private::state_attached_condition::<Self>("@s", #presence_objective)
            }
        },
        Scope::Living => quote! {
            #attach_docs
            /// Attach this component to the current living entity.
            pub fn attach<K: ::sand::__private::LivingEntityKind>(_target: ::sand::__private::EntityContext<K>) -> Vec<String> {
                ::sand::__private::state_attach_commands::<Self>("@s", #presence_objective, #suppression_objective, false)
            }
            #detach_docs
            /// Detach this component from the current living entity.
            pub fn detach<K: ::sand::__private::LivingEntityKind>(_target: ::sand::__private::EntityContext<K>) -> Vec<String> {
                ::sand::__private::state_detach_commands::<Self>("@s", #presence_objective, #suppression_objective, #track_dirty, false)
            }
            #attached_docs
            /// Test whether this component is attached.
            pub fn is_attached<K: ::sand::__private::LivingEntityKind>(_target: ::sand::__private::EntityContext<K>) -> ::sand::__private::condition::Condition {
                ::sand::__private::state_attached_condition::<Self>("@s", #presence_objective)
            }
        },
        Scope::Global => {
            let holder = LitStr::new(
                &global_holder(&namespace.value(), &name.value()),
                input.ident.span(),
            );
            quote! {
                #attach_docs
                /// Attach and initialize this global component.
                pub fn attach() -> Vec<String> {
                    ::sand::__private::state_attach_commands::<Self>(#holder, #presence_objective, #suppression_objective, false)
                }
                #detach_docs
                /// Detach this global component's owned values.
                pub fn detach() -> Vec<String> {
                    ::sand::__private::state_detach_commands::<Self>(#holder, #presence_objective, #suppression_objective, false, false)
                }
                #attached_docs
                /// Test whether this global component is attached.
                pub fn is_attached() -> ::sand::__private::condition::Condition {
                    ::sand::__private::state_attached_condition::<Self>(#holder, #presence_objective)
                }
            }
        }
    };
    let bound_example = match config.scope {
        Scope::Player => format!("let bound = {}::on(player);", ident.unraw()),
        Scope::Entity => format!("let bound = {}::on(entity);", ident.unraw()),
        Scope::Living => format!("let bound = {}::on(living);", ident.unraw()),
        Scope::Global => format!("let bound = {}::global();", ident.unraw()),
    };
    let bound_type_contract = generated_contract(
        bound_ident.unraw().to_string(),
        GeneratedApiKind::Struct,
        format!(
            "Provides holder-bound accessors for the `{}` State schema.",
            ident.unraw()
        ),
        "State binding methods construct this view so every field operation shares one concrete player, entity, or global scoreboard holder.",
        "Each accessor lowers typed reads, writes, and conditions against the schema's generated scoreboard objectives.",
        &["Using several fields from one State schema against the same bound holder"],
        &["Declaring schema-level handles before a target has been selected"],
        &[],
        None,
        bound_example,
    );
    let bound_type_docs = generated_contract_docs(&bound_type_contract);
    generated_contracts.push(bound_type_contract);
    let fields_contract = generated_contract(
        format!("{}::FIELDS", ident.unraw()),
        GeneratedApiKind::AssociatedConst,
        "Lists this State schema's field descriptors in declaration order.",
        "Sand uses these descriptors for schema registration, objective provisioning, defaults, bounds, and lifecycle behavior.",
        "Each descriptor identifies the scoreboard representation and initialization policy for one generated field.",
        &["Inspecting schema metadata for diagnostics or integration tooling"],
        &["Reading or writing a concrete field value; use a typed generated handle instead"],
        &[],
        Some("The static ordered field-descriptor slice for this schema."),
        format!("let fields = {}::FIELDS;", ident.unraw()),
    );
    let fields_docs = generated_contract_docs(&fields_contract);
    generated_contracts.push(fields_contract);
    let system_query_parameter_impl = (!matches!(config.scope, Scope::Global)).then(|| {
        quote! {
            impl ::sand::__private::GeneratedSystemQueryParameter for #ident {}
        }
    });
    let expanded = quote! {
        #bound_type_docs
        #[doc = concat!("Typed bound view generated for `", module_path!(), "::", stringify!(#ident), "`.")]
        #[derive(Debug, Clone, Copy)]
        #visibility struct #bound_ident {
            #(#bound_fields),*
        }

        #[allow(non_upper_case_globals)]
        impl #ident {
            #(#constants)*

            #[doc(hidden)]
            const __SAND_LIFECYCLE_FIELDS: &'static [::sand::__private::StateLifecycleDescriptor] = &[
                #(#lifecycle_fields),*
            ];

            #[doc(hidden)]
            const __SAND_MIGRATIONS: &'static [::sand::__private::StateMigrationDescriptor] = &[
                #(#migration_steps),*
            ];

            #[doc(hidden)]
            const __SAND_DATA_FIELDS: &'static [::sand::__private::StateDataFieldDescriptor] = &[
                #(#data_descriptors),*
            ];

            #fields_docs
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
            #presence_methods
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

            fn data_fields() -> &'static [::sand::__private::StateDataFieldDescriptor] {
                Self::__SAND_DATA_FIELDS
            }
        }

        impl ::sand::__private::StateBundleMember for #ident {
            type Bound = #bound_ident;
            type Scope = #scope_marker;
            const COMPONENT_TREE: ::sand::__private::StateBundleTree =
                ::sand::__private::StateBundleTree::Component(#component_id);

            fn bind_member(holder: &'static str) -> Self::Bound {
                Self::__sand_bind_to(holder)
            }

            fn bind_global_member() -> Self::Bound {
                Self::__sand_bind_to(#default_member_holder)
            }

            fn attach_member(holder: &'static str) -> Vec<String> {
                ::sand::__private::state_attach_commands::<Self>(
                    holder,
                    #presence_objective,
                    #suppression_objective,
                    #suppress_player_observation,
                )
            }

            fn attach_global_member() -> Vec<String> {
                ::sand::__private::state_attach_commands::<Self>(
                    #default_member_holder,
                    #presence_objective,
                    #suppression_objective,
                    false,
                )
            }

            fn detach_member(holder: &'static str) -> Vec<String> {
                ::sand::__private::state_detach_commands::<Self>(
                    holder,
                    #presence_objective,
                    #suppression_objective,
                    #track_dirty,
                    #suppress_player_observation,
                )
            }

            fn detach_global_member() -> Vec<String> {
                ::sand::__private::state_detach_commands::<Self>(
                    #default_member_holder,
                    #presence_objective,
                    #suppression_objective,
                    false,
                    false,
                )
            }

            fn presence_requirements() -> Vec<(String, u32)> {
                vec![(::sand::__private::resolve_state_objective(#presence_objective), #version)]
            }

            fn component_schemas() -> Vec<::sand::__private::StateSchema> {
                vec![<Self as ::sand::__private::EntityState>::schema()]
            }

            fn component_lifecycles() -> Vec<::sand::__private::StateComponentLifecycle> {
                vec![::sand::__private::StateComponentLifecycle::__new(
                    <Self as ::sand::__private::StateBundleMember>::presence_requirements(),
                    <Self as ::sand::__private::StateBundleMember>::component_schemas(),
                    vec![#(#auto_tick_objectives),*],
                    <Self as ::sand::__private::StateBundleMember>::attach_member,
                    <Self as ::sand::__private::StateBundleMember>::detach_member,
                )]
            }
        }

        #system_query_parameter_impl

        #lifecycle_registration
    };
    if matches!(input.vis, syn::Visibility::Public(_)) {
        validate_generated_expansion(
            expanded.clone(),
            [input.ident.unraw().to_string()],
            &generated_contracts,
        )?;
    }
    Ok(expanded)
}

pub(crate) fn derive_bundle(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "State bundles cannot be generic",
        ));
    }
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "StateBundle can only be derived for a struct",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &data.fields,
            "StateBundle requires named component or bundle fields",
        ));
    };
    if fields.named.is_empty() {
        return Err(syn::Error::new_spanned(
            &data.fields,
            "StateBundle requires at least one component",
        ));
    }

    let ident = &input.ident;
    let bound_ident = quote::format_ident!("{}Bound", ident);
    let visibility = &input.vis;
    let mut seen = BTreeSet::new();
    let mut bound_fields = Vec::new();
    let mut bound_values = Vec::new();
    let mut attach = Vec::new();
    let mut detach = Vec::new();
    let mut global_bound_values = Vec::new();
    let mut global_attach = Vec::new();
    let mut global_detach = Vec::new();
    let mut component_trees = Vec::new();
    let mut presence = Vec::new();
    let mut schemas = Vec::new();
    let mut lifecycles = Vec::new();
    let mut member_types = Vec::new();
    let mut contracts = Vec::new();

    for field in &fields.named {
        let field_ident = field.ident.as_ref().expect("named field");
        let field_name = field_ident.unraw().to_string();
        if !seen.insert(field_name.clone()) {
            return Err(syn::Error::new_spanned(
                field_ident,
                format!("duplicate StateBundle field `{field_name}`"),
            ));
        }
        let ty = &field.ty;
        if let syn::Type::Path(path) = ty
            && path.qself.is_none()
            && path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == *ident)
        {
            return Err(syn::Error::new_spanned(
                ty,
                "StateBundle cannot contain itself; bundle nesting must be acyclic",
            ));
        }
        member_types.push(ty);
        let contract = generated_contract(
            format!("{}Bound::{field_name}", ident.unraw()),
            GeneratedApiKind::Field,
            format!("Provides the bound `{field_name}` component or nested bundle view."),
            "Bundle fields retain the original component identity and expose its concrete generated members.",
            "Emits no storage of its own; operations lower through the referenced component backend.",
            &["Navigating a named State bundle with field completion"],
            &["Creating a second physical copy of a component"],
            &[],
            Some("The nested concrete bound view."),
            format!("let component = bundle.{field_name};"),
        );
        let docs = generated_contract_docs(&contract);
        contracts.push(contract);
        bound_fields.push(quote! {
            #docs
            pub #field_ident: <#ty as ::sand::__private::StateBundleMember>::Bound
        });
        bound_values.push(quote! {
            #field_ident: <#ty as ::sand::__private::StateBundleMember>::bind_member(holder)
        });
        global_bound_values.push(quote! {
            #field_ident: <#ty as ::sand::__private::StateBundleMember>::bind_global_member()
        });
        attach.push(quote! {
            commands.extend(<#ty as ::sand::__private::StateBundleMember>::attach_member(holder));
        });
        detach.push(quote! {
            commands.extend(<#ty as ::sand::__private::StateBundleMember>::detach_member(holder));
        });
        global_attach.push(quote! {
            commands.extend(<#ty as ::sand::__private::StateBundleMember>::attach_global_member());
        });
        global_detach.push(quote! {
            commands.extend(<#ty as ::sand::__private::StateBundleMember>::detach_global_member());
        });
        component_trees.push(quote! {
            <#ty as ::sand::__private::StateBundleMember>::COMPONENT_TREE
        });
        presence.push(quote! {
            objectives.extend(<#ty as ::sand::__private::StateBundleMember>::presence_requirements());
        });
        schemas.push(quote! {
            schemas.extend(<#ty as ::sand::__private::StateBundleMember>::component_schemas());
        });
        lifecycles.push(quote! {
            lifecycles.extend(<#ty as ::sand::__private::StateBundleMember>::component_lifecycles());
        });
    }
    detach.reverse();
    global_detach.reverse();

    let type_contract = generated_contract(
        bound_ident.unraw().to_string(),
        GeneratedApiKind::Struct,
        format!("Provides the concrete bound view for `{}`.", ident.unraw()),
        "The view preserves every named component and nested bundle without merging their schemas.",
        "Binding itself emits no commands; operations use each component's existing backend identity.",
        &["Working with a recurring named composition of State components"],
        &["Declaring a new component identity or version"],
        &[],
        None,
        format!("let bundle = {}::on(entity);", ident.unraw()),
    );
    let type_docs = generated_contract_docs(&type_contract);
    contracts.push(type_contract);
    let mut method_docs = Vec::new();
    for (method, summary) in [
        ("on", "Binds every nested component to the current entity."),
        ("attach", "Attaches every unique component in this bundle."),
        (
            "detach",
            "Detaches every unique component in reverse composition order.",
        ),
    ] {
        let contract = generated_contract(
            format!("{}::{method}", ident.unraw()),
            GeneratedApiKind::Method,
            summary,
            "Nested bundles flatten through the compiler-facing composition contract.",
            "Repeated component commands are deduplicated while preserving deterministic declaration order.",
            &["Using bundle composition through one named operation"],
            &["Allocating independent storage for a bundle"],
            &[(
                "_target",
                "The typed current entity context for this bundle.",
            )],
            Some("A concrete bound view or component lifecycle commands."),
            format!("let _ = {}::{method}(entity);", ident.unraw()),
        );
        method_docs.push(generated_contract_docs(&contract));
        contracts.push(contract);
    }
    let on_docs = &method_docs[0];
    let attach_docs = &method_docs[1];
    let detach_docs = &method_docs[2];
    let first_ty = member_types[0];
    let scope_bounds = member_types
        .iter()
        .skip(1)
        .map(|ty| {
            quote! {
                <#first_ty as ::sand::__private::StateBundleMember>::Scope:
                    ::sand::__private::SameStateScope<<#ty as ::sand::__private::StateBundleMember>::Scope>
            }
        })
        .collect::<Vec<_>>();
    let expanded = quote! {
        #type_docs
        #[derive(Debug, Clone, Copy)]
        #visibility struct #bound_ident {
            #(#bound_fields),*
        }

        impl #ident {
            #on_docs
            pub fn on<K: ::sand::__private::EntityKind>(
                _target: ::sand::__private::EntityContext<K>,
            ) -> #bound_ident
            where
                <#first_ty as ::sand::__private::StateBundleMember>::Scope:
                    ::sand::__private::StateBundleTarget<K>,
            {
                <Self as ::sand::__private::StateBundleMember>::bind_member("@s")
            }

            #attach_docs
            pub fn attach<K: ::sand::__private::EntityKind>(
                _target: ::sand::__private::EntityContext<K>,
            ) -> Vec<String>
            where
                <#first_ty as ::sand::__private::StateBundleMember>::Scope:
                    ::sand::__private::StateBundleTarget<K>,
            {
                <Self as ::sand::__private::StateBundleMember>::attach_member("@s")
            }

            #detach_docs
            pub fn detach<K: ::sand::__private::EntityKind>(
                _target: ::sand::__private::EntityContext<K>,
            ) -> Vec<String>
            where
                <#first_ty as ::sand::__private::StateBundleMember>::Scope:
                    ::sand::__private::StateBundleTarget<K>,
            {
                <Self as ::sand::__private::StateBundleMember>::detach_member("@s")
            }

        }

        impl ::sand::__private::StateBundleMember for #ident
        where
            #(#scope_bounds,)*
        {
            type Bound = #bound_ident;
            type Scope = <#first_ty as ::sand::__private::StateBundleMember>::Scope;
            const COMPONENT_TREE: ::sand::__private::StateBundleTree =
                ::sand::__private::StateBundleTree::Bundle(&[#(#component_trees),*]);

            fn bind_member(holder: &'static str) -> Self::Bound {
                #bound_ident { #(#bound_values),* }
            }

            fn bind_global_member() -> Self::Bound {
                #bound_ident { #(#global_bound_values),* }
            }

            fn attach_member(holder: &'static str) -> Vec<String> {
                let mut commands = Vec::new();
                #(#attach)*
                let mut seen = ::std::collections::BTreeSet::new();
                commands.retain(|command| seen.insert(command.clone()));
                commands
            }

            fn attach_global_member() -> Vec<String> {
                let mut commands = Vec::new();
                #(#global_attach)*
                let mut seen = ::std::collections::BTreeSet::new();
                commands.retain(|command| seen.insert(command.clone()));
                commands
            }

            fn detach_member(holder: &'static str) -> Vec<String> {
                let mut commands = Vec::new();
                #(#detach)*
                let mut seen = ::std::collections::BTreeSet::new();
                commands.retain(|command| seen.insert(command.clone()));
                commands
            }

            fn detach_global_member() -> Vec<String> {
                let mut commands = Vec::new();
                #(#global_detach)*
                let mut seen = ::std::collections::BTreeSet::new();
                commands.retain(|command| seen.insert(command.clone()));
                commands
            }

            fn presence_requirements() -> Vec<(String, u32)> {
                let mut objectives = Vec::new();
                #(#presence)*
                objectives.sort();
                objectives.dedup();
                objectives
            }

            fn component_schemas() -> Vec<::sand::__private::StateSchema> {
                let mut schemas = Vec::new();
                #(#schemas)*
                let mut seen = ::std::collections::BTreeSet::new();
                schemas.retain(|schema| seen.insert(schema.id()));
                schemas
            }

            fn component_lifecycles() -> Vec<::sand::__private::StateComponentLifecycle> {
                let mut lifecycles = Vec::new();
                #(#lifecycles)*
                lifecycles
            }
        }

        impl ::sand::__private::GeneratedSystemQueryParameter for #ident {}

    };
    if matches!(input.vis, syn::Visibility::Public(_)) {
        validate_generated_expansion(
            expanded.clone(),
            [input.ident.unraw().to_string()],
            &contracts,
        )?;
    }
    Ok(expanded)
}

pub(crate) fn derive_query(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "State queries cannot be generic",
        ));
    }
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "StateQuery can only be derived for a struct",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &data.fields,
            "StateQuery requires named component or bundle fields",
        ));
    };

    let ident = &input.ident;
    let query_scope = parse_query_scope(&input)?;
    let item_ident = quote::format_ident!("{}Item", ident);
    let visibility = &input.vis;
    let mut item_fields = Vec::new();
    let mut item_values = Vec::new();
    let mut required = Vec::new();
    let mut forbidden = Vec::new();
    let mut optional_methods = Vec::new();
    let mut contracts = Vec::new();
    let mut component_modes = BTreeMap::<String, QueryFieldMode>::new();
    let mut component_types = Vec::new();
    let mut required_types = Vec::new();
    let mut forbidden_types = Vec::new();
    let mut optional_types = Vec::new();

    for field in &fields.named {
        let field_ident = field.ident.as_ref().expect("named field");
        let ty = &field.ty;
        let mode = parse_query_field_mode(field)?;
        component_types.push(ty);
        let component_key = quote!(#ty).to_string();
        if let Some(previous) = component_modes.insert(component_key.clone(), mode)
            && previous != mode
        {
            return Err(syn::Error::new_spanned(
                field,
                format!(
                    "StateQuery component `{component_key}` has contradictory presence requirements"
                ),
            ));
        }
        match mode {
            QueryFieldMode::Required => {
                required_types.push(ty);
                let contract = generated_contract(
                    format!("{}::{field_ident}", item_ident.unraw()),
                    GeneratedApiKind::Field,
                    format!("Provides the bound required `{field_ident}` component or bundle."),
                    "The query selector proves that every component represented by this field is attached at its current version.",
                    "The concrete view binds its component operations to the current generated Minecraft iteration executor.",
                    &["Accessing required State data inside a typed query callback"],
                    &["Retaining the execution-scoped view after the generated callback"],
                    &[],
                    Some("A concrete holder-bound State component or bundle view."),
                    format!("let component = item.{field_ident};"),
                );
                let docs = generated_contract_docs(&contract);
                contracts.push(contract);
                item_fields.push(quote! {
                    #docs
                    pub #field_ident: <#ty as ::sand::__private::StateBundleMember>::Bound
                });
                item_values.push(quote! {
                    #field_ident: <#ty as ::sand::__private::StateBundleMember>::bind_member("@s")
                });
                required.push(quote! {
                    requirements.extend(<#ty as ::sand::__private::StateBundleMember>::presence_requirements());
                });
            }
            QueryFieldMode::Forbidden => {
                forbidden_types.push(ty);
                forbidden.push(quote! {
                    forbidden.extend(<#ty as ::sand::__private::StateBundleMember>::presence_requirements());
                });
            }
            QueryFieldMode::Optional => {
                optional_types.push(ty);
                let contract = generated_contract(
                    format!("{}::{field_ident}", item_ident.unraw()),
                    GeneratedApiKind::Method,
                    format!("Runs a callback when optional `{field_ident}` State is present."),
                    "Optional access is represented by generated Minecraft presence guards, not a Rust Option fabricated during compilation.",
                    "Each returned command executes only when every component in the optional component or bundle is attached at its current version.",
                    &["Conditionally using an optional State component inside a query callback"],
                    &["Assuming optional State is present outside the guarded callback"],
                    &[(
                        "body",
                        "Builds commands against the concrete optional bound view.",
                    )],
                    Some("Commands guarded by the optional component presence conditions."),
                    format!("let commands = item.{field_ident}(|state| state.value.add(1));"),
                );
                let docs = generated_contract_docs(&contract);
                contracts.push(contract);
                optional_methods.push(quote! {
                    #docs
                    pub fn #field_ident(
                        &self,
                        body: impl FnOnce(<#ty as ::sand::__private::StateBundleMember>::Bound) -> Vec<String>,
                    ) -> Vec<String> {
                        let requirements = <#ty as ::sand::__private::StateBundleMember>::presence_requirements();
                        let commands = body(<#ty as ::sand::__private::StateBundleMember>::bind_member("@s"));
                        commands
                            .into_iter()
                            .map(|command| {
                                let guards = requirements
                                    .iter()
                                    .map(|(objective, version)| format!("if score @s {objective} matches {version}"))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                format!("execute {guards} run {command}")
                            })
                            .collect()
                    }
                });
            }
        }
    }

    let item_contract = generated_contract(
        item_ident.unraw().to_string(),
        GeneratedApiKind::Struct,
        format!(
            "Provides concrete bound State access for one `{}` query match.",
            ident.unraw()
        ),
        "The value exists only while generating the callback body for one Minecraft iteration executor.",
        "Required component presence is filtered in the selector and optional access emits runtime execute guards.",
        &["Using named component fields inside a StateQuery callback"],
        &["Treating a Minecraft entity as a persistent Rust reference"],
        &[],
        None,
        format!("{}::each(|item| Vec::new());", ident.unraw()),
    );
    let item_docs = generated_contract_docs(&item_contract);
    contracts.push(item_contract);
    let each_contract = generated_contract(
        format!("{}::each", ident.unraw()),
        GeneratedApiKind::Method,
        "Runs a generated Minecraft iteration over entities matching this State query.",
        "Required components become typed score filters, forbidden components become runtime absence guards, and callback order is preserved.",
        "Emits one execute-as/at iteration calling a deterministic generated function body.",
        &["Processing loaded entities selected by State component presence"],
        &["Performing a single-holder read without narrowing the query"],
        &[(
            "body",
            "Builds commands for the concrete query item bound to @s.",
        )],
        Some("The command that invokes the generated query iteration."),
        format!("let commands = {}::each(|item| Vec::new());", ident.unraw()),
    );
    let each_docs = generated_contract_docs(&each_contract);
    contracts.push(each_contract);
    let current_contract = generated_contract(
        format!("{}::current", ident.unraw()),
        GeneratedApiKind::Method,
        "Runs a State query against the current event or function executor.",
        "The caller must already execute as the intended owner; required and forbidden component presence is checked at Minecraft runtime.",
        "Emits guarded commands directly and never introduces an entity scan.",
        &["Filtering an event executor before accessing its State components"],
        &["Iterating a collection; use each for generated Minecraft iteration"],
        &[(
            "body",
            "Builds commands for the concrete query item bound to @s.",
        )],
        Some("Commands guarded by the complete query presence predicate."),
        format!(
            "let commands = {}::current(|item| Vec::new());",
            ident.unraw()
        ),
    );
    let current_docs = generated_contract_docs(&current_contract);
    contracts.push(current_contract);
    let query_scope_value = if matches!(query_scope, Some(Scope::Player)) {
        quote!(::sand::__private::StateQueryScope::Player)
    } else {
        quote!(::sand::__private::StateQueryScope::Entity)
    };
    let scope_assertions: Vec<_> = query_scope
        .map(|scope| {
            let marker = match scope {
                Scope::Player => quote!(::sand::__private::PlayerStateScope),
                Scope::Entity => quote!(::sand::__private::EntityStateScope),
                Scope::Living => quote!(::sand::__private::LivingStateScope),
                Scope::Global => quote!(::sand::__private::GlobalStateScope),
            };
            component_types
                .iter()
                .map(|ty| {
                    quote!(let _ = assert_same::<#marker, <#ty as ::sand::__private::StateBundleMember>::Scope>;)
                })
                .collect()
        })
        .unwrap_or_default();
    let mut contradiction_assertions = Vec::new();
    for (left_group, right_group) in [
        (&required_types, &forbidden_types),
        (&required_types, &optional_types),
        (&optional_types, &forbidden_types),
    ] {
        for left in left_group {
            for right in right_group {
                contradiction_assertions.push(quote! {
                    if ::sand::__private::state_bundle_trees_overlap(
                        &<#left as ::sand::__private::StateBundleMember>::COMPONENT_TREE,
                        &<#right as ::sand::__private::StateBundleMember>::COMPONENT_TREE,
                    ) {
                        panic!("StateQuery members with different presence modes overlap after flattening nested bundles");
                    }
                });
            }
        }
    }

    let expanded = quote! {
        const _: () = {
            fn assert_same<L, R>()
            where
                L: ::sand::__private::StateScopeMarker + ::sand::__private::SameStateScope<R>,
                R: ::sand::__private::StateScopeMarker,
            {}
            #(#scope_assertions)*
            #(#contradiction_assertions)*
        };

        #item_docs
        #[derive(Debug, Clone, Copy)]
        #visibility struct #item_ident {
            #(#item_fields),*
        }

        impl #item_ident {
            #(#optional_methods)*
        }

        impl #ident {
            #each_docs
            pub fn each(body: impl FnOnce(#item_ident) -> Vec<String>) -> Vec<String> {
                let mut requirements: Vec<(String, u32)> = Vec::new();
                #(#required)*
                let mut forbidden: Vec<(String, u32)> = Vec::new();
                #(#forbidden)*
                ::sand::__private::lower_state_query_each(
                    #query_scope_value,
                    requirements,
                    forbidden,
                    #item_ident { #(#item_values),* },
                    body,
                )
            }


            #current_docs
            pub fn current(body: impl FnOnce(#item_ident) -> Vec<String>) -> Vec<String> {
                let mut requirements: Vec<(String, u32)> = Vec::new();
                #(#required)*
                let mut forbidden: Vec<(String, u32)> = Vec::new();
                #(#forbidden)*
                ::sand::__private::lower_state_query_current(
                    requirements,
                    forbidden,
                    #item_ident { #(#item_values),* },
                    body,
                )
            }
        }

        impl ::sand::__private::StateQuerySpec for #ident {
            type Item = #item_ident;

            fn each(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
                Self::each(body)
            }


            fn current(body: impl FnOnce(Self::Item) -> Vec<String>) -> Vec<String> {
                Self::current(body)
            }
        }

        impl ::sand::__private::GeneratedSystemQueryParameter for #ident {}
    };
    if matches!(input.vis, syn::Visibility::Public(_)) {
        validate_generated_expansion(
            expanded.clone(),
            [input.ident.unraw().to_string()],
            &contracts,
        )?;
    }
    Ok(expanded)
}

fn parse_query_scope(input: &DeriveInput) -> syn::Result<Option<Scope>> {
    let attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("query"))
        .collect();
    if attrs.len() > 1 {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "StateQuery accepts at most one #[query(scope = ...)] attribute",
        ));
    }
    let Some(attr) = attrs.first() else {
        return Ok(None);
    };
    let mut scope = None;
    attr.parse_nested_meta(|meta| {
        if !meta.path.is_ident("scope") {
            return Err(meta.error("unknown StateQuery option; expected `scope`"));
        }
        let value: syn::Ident = meta.value()?.parse()?;
        let parsed = match value.to_string().as_str() {
            "player" => Scope::Player,
            "entity" => Scope::Entity,
            "living" => Scope::Living,
            "global" => {
                return Err(syn::Error::new_spanned(
                    value,
                    "global State is a singleton resource and cannot be iterated by StateQuery",
                ));
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    value,
                    "invalid query scope; expected player, entity, or living",
                ));
            }
        };
        set_once(&mut scope, parsed, &meta.path)
    })?;
    scope
        .ok_or_else(|| syn::Error::new_spanned(attr, "StateQuery scope is required"))
        .map(Some)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryFieldMode {
    Required,
    Optional,
    Forbidden,
}

fn parse_query_field_mode(field: &syn::Field) -> syn::Result<QueryFieldMode> {
    let mut selected = None;
    for attr in &field.attrs {
        let mode = if attr.path().is_ident("require") || attr.path().is_ident("required") {
            Some(QueryFieldMode::Required)
        } else if attr.path().is_ident("optional") {
            Some(QueryFieldMode::Optional)
        } else if attr.path().is_ident("without") || attr.path().is_ident("forbidden") {
            Some(QueryFieldMode::Forbidden)
        } else if attr.path().is_ident("state") {
            let mut nested = None;
            attr.parse_nested_meta(|meta| {
                let candidate = if meta.path.is_ident("required") || meta.path.is_ident("require") {
                    QueryFieldMode::Required
                } else if meta.path.is_ident("optional") {
                    QueryFieldMode::Optional
                } else if meta.path.is_ident("forbidden") || meta.path.is_ident("without") {
                    QueryFieldMode::Forbidden
                } else {
                    return Err(meta.error("expected `required`, `optional`, or `forbidden`"));
                };
                if nested.replace(candidate).is_some() {
                    return Err(meta.error("a StateQuery field can declare only one presence mode"));
                }
                Ok(())
            })?;
            nested
        } else {
            None
        };
        if let Some(mode) = mode
            && selected.replace(mode).is_some()
        {
            return Err(syn::Error::new_spanned(
                attr,
                "a StateQuery field cannot combine required, optional, and forbidden modes",
            ));
        }
    }
    Ok(selected.unwrap_or(QueryFieldMode::Required))
}

#[allow(clippy::too_many_arguments)]
fn generated_contract(
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

fn generated_contract_docs(contract: &GeneratedApiContract) -> proc_macro2::TokenStream {
    let lines = render_generated_rustdoc(contract);
    let lines = lines
        .iter()
        .map(|line| LitStr::new(line, proc_macro2::Span::call_site()));
    quote!(#(#[doc = #lines])*)
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
    migrations: Vec<(u32, u32)>,
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
    let mut migrations = Vec::new();
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
        } else if meta.path.is_ident("migrate") {
            let mut from = None;
            let mut to = None;
            meta.parse_nested_meta(|nested| {
                if nested.path.is_ident("from") {
                    let value: LitInt = nested.value()?.parse()?;
                    set_once(&mut from, value.base10_parse()?, &nested.path)
                } else if nested.path.is_ident("to") {
                    let value: LitInt = nested.value()?.parse()?;
                    set_once(&mut to, value.base10_parse()?, &nested.path)
                } else {
                    Err(nested.error("expected `from = N` or `to = N`"))
                }
            })?;
            let from = from.ok_or_else(|| meta.error("migration requires `from = N`"))?;
            let to = to.ok_or_else(|| meta.error("migration requires `to = N`"))?;
            migrations.push((from, to));
            Ok(())
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
    migrations.sort_unstable();
    for (index, &(from, to)) in migrations.iter().enumerate() {
        if from == 0 || to != from + 1 || to > version {
            return Err(syn::Error::new_spanned(
                attrs[0],
                "State migrations must be contiguous `from = N, to = N + 1` transitions ending at or before the schema version",
            ));
        }
        if index > 0 && migrations[index - 1].1 != from {
            return Err(syn::Error::new_spanned(
                attrs[0],
                "State migration declarations contain a gap or conflicting transition",
            ));
        }
    }
    if !migrations.is_empty()
        && (migrations.first().is_none_or(|step| step.0 != 1)
            || migrations.last().is_none_or(|step| step.1 != version))
    {
        return Err(syn::Error::new_spanned(
            attrs[0],
            "State migration declarations must cover every transition from version 1 through the current version",
        ));
    }
    Ok(SchemaConfig {
        namespace,
        name,
        version,
        scope,
        migrations,
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
    default_snbt: Option<LitStr>,
    scale: Option<u32>,
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
            } else if meta.path.is_ident("default_snbt") {
                let value: LitStr = meta.value()?.parse()?;
                set_once(&mut result.default_snbt, value, &meta.path)
            } else if meta.path.is_ident("scale") {
                let expression: Expr = meta.value()?.parse()?;
                let value = parse_i32(&expression)?;
                let value = u32::try_from(value)
                    .map_err(|_| meta.error("FixedScore scale must be positive"))?;
                set_once(&mut result.scale, value, &meta.path)
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

fn quoted_nbt_key(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('\"', "\\\""))
}

enum Wrapper {
    Score(Type),
    Fixed,
    Flag,
    Enum(Type),
    Timer,
    Cooldown,
    Data(Type),
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
        "Score" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Score(parse_quote!(i32)))
        }
        "FixedScore" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Fixed)
        }
        "EntityEnum" | "StateEnum" => Ok(Wrapper::Enum(required_type_argument(segment)?)),
        "EntityFlag" | "Flag" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Flag)
        }
        "EntityTimer" | "Timer" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Timer)
        }
        "EntityCooldown" | "Cooldown" => {
            reject_arguments(segment)?;
            Ok(Wrapper::Cooldown)
        }
        "Data" => Ok(Wrapper::Data(required_type_argument(segment)?)),
        _ => Err(syn::Error::new_spanned(
            ty,
            "unsupported State field wrapper; expected Score, FixedScore, Flag, StateEnum<T>, Timer, Cooldown, Data<T>, or an advanced Entity-prefixed form",
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

fn fixed_default(default: Option<Expr>, scale: i32, field: &syn::Field) -> syn::Result<i32> {
    let Some(default) = default else {
        return Ok(0);
    };
    let value = parse_f64(&default).map_err(|_| {
        syn::Error::new_spanned(field, "FixedScore default must be a finite numeric literal")
    })?;
    encode_fixed_literal(value, scale, field)
}

fn parse_f64(expression: &Expr) -> syn::Result<f64> {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Float(value),
            ..
        }) => value.base10_parse(),
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_parse(),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => Ok(-parse_f64(&unary.expr)?),
        _ => Err(syn::Error::new_spanned(
            expression,
            "expected a numeric literal",
        )),
    }
}

fn encode_fixed_literal(value: f64, scale: i32, field: &syn::Field) -> syn::Result<i32> {
    if !value.is_finite() {
        return Err(syn::Error::new_spanned(
            field,
            "FixedScore values must be finite",
        ));
    }
    let scaled = value * f64::from(scale);
    if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
        return Err(syn::Error::new_spanned(
            field,
            "FixedScore value does not fit the signed 32-bit scoreboard backend at this scale",
        ));
    }
    Ok(scaled.round() as i32)
}

fn scale_fixed_integer(value: i32, scale: i32, field: &syn::Field) -> syn::Result<i32> {
    value.checked_mul(scale).ok_or_else(|| {
        syn::Error::new_spanned(
            field,
            "FixedScore bound does not fit the signed 32-bit scoreboard backend at this scale",
        )
    })
}

fn data_default_snbt(config: &FieldConfig, field: &syn::Field) -> syn::Result<LitStr> {
    if config.default.is_some() && config.default_snbt.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "Data<T> fields cannot combine `default` and `default_snbt`",
        ));
    }
    if let Some(value) = &config.default_snbt {
        if value.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(
                value,
                "default SNBT must not be empty",
            ));
        }
        return Ok(value.clone());
    }
    let Some(default) = &config.default else {
        return Err(syn::Error::new_spanned(
            field,
            "Data<T> fields require `default = <integer|bool|string>` or `default_snbt = \"...\"`",
        ));
    };
    let rendered = match default {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_digits().to_owned(),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            format!("-{}", parse_i32(&unary.expr)?)
        }
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) => if value.value { "1b" } else { "0b" }.to_owned(),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => format!("{:?}", value.value()),
        _ => {
            return Err(syn::Error::new_spanned(
                default,
                "typed Data default must be an integer, bool, or string literal; use `default_snbt` for compound/list SNBT",
            ));
        }
    };
    Ok(LitStr::new(&rendered, default.span()))
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

#[cfg(test)]
mod generated_surface_tests {
    use super::*;

    fn state_input() -> DeriveInput {
        syn::parse_quote! {
            #[state(namespace = "demo", name = "stats", scope = player)]
            pub struct Stats { health: EntityScore<i32> }
        }
    }

    fn fixture_contract(
        target: &str,
        kind: GeneratedApiKind,
        parameters: &[(&str, &str)],
        returns: Option<&str>,
    ) -> GeneratedApiContract {
        generated_contract(
            target.to_owned(),
            kind,
            "Reads the fixture scoreboard state for validation.",
            "The fixture binds one declared scoreboard value for validator coverage.",
            "Minecraft stores the fixture value in the selected scoreboard objective.",
            &["Using the generated fixture API"],
            &["The fixture API is not applicable"],
            parameters,
            returns,
            "let value = fixture();".to_owned(),
        )
    }

    #[test]
    fn validator_extracts_exact_callable_signature_parameters_and_return() {
        let contract = fixture_contract(
            "Stats::on",
            GeneratedApiKind::Method,
            &[("_target", "The target context.")],
            Some("The bound view."),
        );
        let docs = generated_contract_docs(&contract);
        let expansion = quote! {
            impl Stats {
                #docs
                pub fn on<K: EntityKind>(_target: EntityContext<K>) -> StatsBound { todo!() }
            }
        };
        let shapes = validate_generated_expansion(expansion, ["Stats".to_owned()], &[contract])
            .expect("exact generated contract");
        let shape = &shapes[0];
        assert_eq!(shape.target, "Stats::on");
        assert!(shape.signature.contains("EntityContext < K >"));
        assert_eq!(shape.parameters["_target"], "EntityContext < K >");
        assert_eq!(shape.return_type.as_deref(), Some("StatsBound"));
    }

    #[test]
    fn validator_extracts_bound_field_and_owner_const_types() {
        let bound = fixture_contract("StatsBound", GeneratedApiKind::Struct, &[], None);
        let field = fixture_contract(
            "StatsBound::health",
            GeneratedApiKind::Field,
            &[],
            Some("The bound accessor."),
        );
        let constant = fixture_contract(
            "Stats::health",
            GeneratedApiKind::AssociatedConst,
            &[],
            Some("The definition-owned handle."),
        );
        let bound_docs = generated_contract_docs(&bound);
        let field_docs = generated_contract_docs(&field);
        let constant_docs = generated_contract_docs(&constant);
        let expansion = quote! {
            #bound_docs
            pub struct StatsBound {
                #field_docs
                pub health: EntityScoreAccessor<i32>
            }
            impl Stats {
                #constant_docs
                pub const health: EntityScore<i32> = todo!();
            }
        };
        let shapes = validate_generated_expansion(
            expansion,
            ["Stats".to_owned()],
            &[bound, field, constant],
        )
        .expect("field and const contracts");
        let field = shapes
            .iter()
            .find(|shape| shape.target == "StatsBound::health")
            .unwrap();
        assert_eq!(
            field.return_type.as_deref(),
            Some("EntityScoreAccessor < i32 >")
        );
        let constant = shapes
            .iter()
            .find(|shape| shape.target == "Stats::health")
            .unwrap();
        assert_eq!(constant.return_type.as_deref(), Some("EntityScore < i32 >"));
    }

    #[test]
    fn validator_rejects_uncontracted_siblings_and_bound_members() {
        let bound = fixture_contract("StatsBound", GeneratedApiKind::Struct, &[], None);
        let bound_docs = generated_contract_docs(&bound);
        let expansion = quote! {
            #bound_docs
            pub struct StatsBound;
            pub struct Escape;
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[bound])
                .unwrap_err()
                .to_string()
                .contains("Escape")
        );

        let bound = fixture_contract("StatsBound", GeneratedApiKind::Struct, &[], None);
        let bound_docs = generated_contract_docs(&bound);
        let expansion = quote! {
            #bound_docs
            pub struct StatsBound;
            impl StatsBound { pub fn escape() {} }
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[bound])
                .unwrap_err()
                .to_string()
                .contains("StatsBound::escape")
        );
    }

    #[test]
    fn validator_rejects_parameter_return_kind_and_rustdoc_drift() {
        let contract = fixture_contract(
            "Stats::on",
            GeneratedApiKind::Method,
            &[("_target", "The target context.")],
            Some("The bound view."),
        );
        let docs = generated_contract_docs(&contract);
        let expansion = quote! {
            impl Stats {
                #docs
                pub fn on(_target: EntityContext, extra: i32) -> StatsBound { todo!() }
            }
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[contract])
                .unwrap_err()
                .to_string()
                .contains("parameter contract drift")
        );

        let contract = fixture_contract(
            "Stats::on",
            GeneratedApiKind::Method,
            &[("_target", "The target context.")],
            Some("The bound view."),
        );
        let docs = generated_contract_docs(&contract);
        let expansion = quote! {
            impl Stats {
                #docs
                pub fn on(_target: EntityContext) {}
            }
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[contract])
                .unwrap_err()
                .to_string()
                .contains("return contract drift")
        );

        let wrong_kind = fixture_contract(
            "Stats::on",
            GeneratedApiKind::AssociatedConst,
            &[("_target", "The target context.")],
            Some("The bound view."),
        );
        let docs = generated_contract_docs(&wrong_kind);
        let expansion = quote! {
            impl Stats {
                #docs
                pub fn on(_target: EntityContext) -> StatsBound { todo!() }
            }
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[wrong_kind])
                .unwrap_err()
                .to_string()
                .contains("declares AssociatedConst")
        );

        let contract = fixture_contract(
            "Stats::on",
            GeneratedApiKind::Method,
            &[("_target", "The target context.")],
            Some("The bound view."),
        );
        let expansion = quote! {
            impl Stats {
                pub fn on(_target: EntityContext) -> StatsBound { todo!() }
            }
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[contract])
                .unwrap_err()
                .to_string()
                .contains("contract-derived Rustdoc")
        );

        let mut contract = fixture_contract(
            "Stats::on",
            GeneratedApiKind::Method,
            &[("_target", "The target context.")],
            Some("The bound view."),
        );
        contract.summary.clear();
        let expansion = quote! {
            impl Stats {
                pub fn on(_target: EntityContext) -> StatsBound { todo!() }
            }
        };
        assert!(
            validate_generated_expansion(expansion, ["Stats".to_owned()], &[contract])
                .unwrap_err()
                .to_string()
                .contains("cannot be empty")
        );
    }

    #[test]
    fn real_state_expansion_contracts_every_generated_public_item() {
        let expansion = derive_state(state_input()).unwrap().to_string();
        assert!(expansion.matches("# Minecraft behavior").count() >= 5);
        assert!(expansion.matches("# Example").count() >= 5);
        assert!(!expansion.contains("sand api show"));
        assert!(expansion.contains("StatsBound"));
        assert!(expansion.contains("FIELDS"));
        assert!(expansion.contains("on"));
    }
}
