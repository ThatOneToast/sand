//! Payload enum construction reuses the owned aggregate layout and copy paths.
use super::text::LeafDefault;
use super::values::child;
use super::*;

impl Cx {
    pub(super) fn enum_type(&self, def: DefId) -> Ty {
        if matches!(
            self.defs[def.0].info,
            DefInfo::Enum {
                layout: Some(_),
                ..
            }
        ) {
            Ty::Struct(def)
        } else {
            Ty::Enum(def)
        }
    }

    pub(super) fn variant_slot(
        &mut self,
        parent: DefId,
        span: Span,
        name: String,
        ty: Ty,
    ) -> DefId {
        let owner = &self.defs[parent.0];
        self.push_def(Def {
            kind: DefKind::Field,
            name,
            module: owner.module,
            file: owner.file,
            name_span: Span::default(),
            span,
            docs: Vec::new(),
            is_pub: false,
            parent: Some(parent),
            info: DefInfo::Field {
                ty,
                default: None,
                bounds: None,
                timer: false,
            },
        })
    }

    pub(super) fn evaluate_variant_payloads(&mut self) {
        let variants: Vec<_> = self.variant_src.keys().copied().collect();
        for variant in variants {
            let parent = self.defs[variant.0].parent.expect("variant owner");
            if self.generics.enums.contains_key(&parent) {
                continue;
            }
            self.resolve_variant_fields(variant, variant);
        }
    }

    fn resolve_variant_fields(&mut self, source: DefId, concrete: DefId) {
        let fields = self.variant_fields(concrete).to_vec();
        let source = &self.variant_src[&source];
        let (module, payload) = (source.module, source.payload.clone());
        for (field, annotation) in fields.into_iter().zip(payload) {
            let ty = self.resolve_type(module, &annotation);
            if let DefInfo::Field { ty: slot, .. } = &mut self.defs[field.0].info {
                *slot = ty;
            }
        }
    }

    pub(super) fn instantiate_variant_layout(&mut self, template: DefId, instance: DefId) {
        let DefInfo::Enum {
            variants: sources, ..
        } = self.defs[template.0].info.clone()
        else {
            unreachable!("enum template");
        };
        let tag = self.variant_slot(instance, Span::default(), "tag".into(), Ty::Enum(instance));
        let mut layout = vec![tag];
        let mut members = BTreeMap::new();
        let mut variants = Vec::new();
        for source in &sources {
            let mut variant = self.defs[source.0].clone();
            variant.parent = Some(instance);
            let name = variant.name.clone();
            let concrete = self.push_def(variant);
            self.generics.origins.insert(concrete, *source);
            let fields: Vec<_> = self
                .variant_fields(*source)
                .to_vec()
                .into_iter()
                .map(|field| {
                    let authored = &self.defs[field.0];
                    self.variant_slot(concrete, authored.span, authored.name.clone(), Ty::Unknown)
                })
                .collect();
            layout.extend(fields.iter().copied());
            let value = self.eval_def(*source);
            self.defs[concrete.0].info = DefInfo::Variant { value, fields };
            members.insert(name, concrete);
            variants.push(concrete);
        }
        self.members.insert(instance, members);
        self.defs[instance.0].info = DefInfo::Enum {
            variants: variants.clone(),
            layout: Some(layout),
        };
        for (source, concrete) in sources.into_iter().zip(variants) {
            self.resolve_variant_fields(source, concrete);
        }
    }

    pub(super) fn variant_tag(&self, ty: Ty) -> Option<&str> {
        let Ty::Struct(def) = ty else { return None };
        let DefInfo::Enum {
            layout: Some(layout),
            ..
        } = &self.defs[def.0].info
        else {
            return None;
        };
        layout.first().map(|tag| self.defs[tag.0].name.as_str())
    }

    pub(super) fn variant_path(&mut self, module: ModuleId, path: &Path) -> Option<DefId> {
        let (variant, _) = self.resolve_path(module, path).ok()?;
        (self.defs[variant.0].kind == DefKind::Variant).then_some(variant)
    }

    pub(super) fn variant_type(&self, variant: DefId) -> Ty {
        self.enum_type(self.defs[variant.0].parent.expect("variant owner"))
    }

    pub(super) fn variant_in_type(&self, variant: DefId, ty: Ty) -> Option<DefId> {
        let owner = self.defs[variant.0].parent.expect("variant owner");
        if !self.generics.enums.contains_key(&owner) {
            return Some(variant);
        }
        let Ty::Struct(instance) = ty else {
            return None;
        };
        let (template, _) = self.generics.record_arguments.get(&instance)?;
        if *template != owner {
            return None;
        }
        self.members
            .get(&instance)?
            .get(&self.defs[variant.0].name)
            .copied()
    }

    pub(super) fn variant_type_from_args(
        &mut self,
        body: &BodyCx,
        variant: DefId,
        args: &[Arg],
        span: Span,
    ) -> Ty {
        let owner = self.defs[variant.0].parent.expect("variant owner");
        let Some((params, _)) = self.generics.enums.get(&owner).cloned() else {
            return self.variant_type(variant);
        };
        let source = &self.variant_src[&variant];
        let (module, payload) = (source.module, source.payload.clone());
        let mut inferred = BTreeMap::new();
        for (annotation, arg) in payload.iter().zip(args) {
            if let ArgValue::Expr(expr) = &arg.value {
                let ty = self.type_of(body, expr);
                self.infer_type_arguments(module, annotation, ty, &params, &mut inferred);
            }
        }
        let Some(types) = params
            .iter()
            .map(|param| inferred.get(&param.name).copied())
            .collect::<Option<Vec<_>>>()
        else {
            return self.variant_type(variant);
        };
        self.instantiate_aggregate(owner, types, body.file, span)
            .unwrap_or(Ty::Unknown)
    }

    pub(super) fn variant_fields(&self, variant: DefId) -> &[DefId] {
        match &self.defs[variant.0].info {
            DefInfo::Variant { fields, .. } => fields,
            _ => unreachable!("expected a variant"),
        }
    }

    pub(super) fn variant_discriminant(&self, variant: DefId) -> Option<i32> {
        match self.defs[variant.0].info {
            DefInfo::Variant { value, .. } => value.map(|value| value as i32),
            _ => None,
        }
    }

    pub(super) fn check_variant_arity(
        &mut self,
        file: FileId,
        span: Span,
        variant: DefId,
        given: usize,
    ) -> bool {
        let expected = self.variant_fields(variant).len();
        if expected == given {
            return true;
        }
        self.error(
            "variant",
            file,
            span,
            format!(
                "variant `{}` expects {expected} payload values, found {given}",
                self.defs[variant.0].name,
            ),
        );
        false
    }

    fn variant_values<'a>(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &'a Expr,
        ty: Ty,
    ) -> Option<(DefId, Vec<&'a Expr>)> {
        let (path, args) = match expr {
            Expr::Path(path) => (path, &[][..]),
            Expr::Call { path, args, .. } => (path, args.as_slice()),
            _ => {
                self.error(
                    "variant",
                    file,
                    expr.span(),
                    "expected an enum variant constructor",
                );
                return None;
            }
        };
        let variant = self.expect_path(module, path, &[DefKind::Variant], "an enum variant")?;
        let Some(variant) = self.variant_in_type(variant, ty) else {
            self.error(
                "generic",
                file,
                path.span,
                "this variant needs explicit type arguments or a matching enum type annotation",
            );
            return None;
        };
        if !self.types_agree(file, path.span, ty, self.variant_type(variant))
            || !self.check_variant_arity(file, expr.span(), variant, args.len())
        {
            return None;
        }
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            if let (None, ArgValue::Expr(value)) = (&arg.name, &arg.value) {
                values.push(value);
            } else {
                self.error(
                    "variant",
                    file,
                    arg.span,
                    "variant constructors take positional values",
                );
                return None;
            }
        }
        Some((variant, values))
    }

    pub(super) fn variant_constant(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: Option<&Expr>,
        ty: Ty,
        def: DefId,
    ) -> Option<Vec<LeafDefault>> {
        let (variant, args) = if let Some(expr) = expr {
            self.variant_values(module, file, expr, ty)?
        } else {
            let DefInfo::Enum { variants, .. } = &self.defs[def.0].info else {
                unreachable!("expected an enum");
            };
            (*variants.first()?, Vec::new())
        };
        let active: BTreeMap<_, _> = self
            .variant_fields(variant)
            .iter()
            .copied()
            .zip(args)
            .collect();
        let mut leaves = vec![LeafDefault::Score(self.variant_discriminant(variant)?)];
        for field in self.struct_fields(def).into_iter().skip(1) {
            let next = self.record_constant(
                module,
                file,
                active.get(&field).copied(),
                self.field_ty(field),
            )?;
            if !self.default_size(
                file,
                expr.map(Expr::span).unwrap_or_default(),
                text::defaults_cost(&leaves).saturating_add(text::defaults_cost(&next)),
            ) {
                return None;
            }
            leaves.extend(next);
        }
        Some(leaves)
    }

    pub(super) fn emit_variant(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        def: DefId,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let ty = Ty::Struct(def);
        let (variant, args) = self.variant_values(body.module, body.file, expr, ty)?;
        let span = expr.span();
        let discr = self.variant_discriminant(variant)?;
        let tag = self.variant_tag(ty)?.to_owned();
        let depth = body.temp_depth;
        let snapshot = self.temp(body);
        let fields = self.variant_fields(variant).to_vec();
        let mut init = Vec::new();
        for (field, arg) in fields.iter().zip(args) {
            let slot = child(&snapshot, &self.defs[field.0].name);
            if self
                .emit(body, &slot, self.field_ty(*field), arg, &mut init)
                .is_none()
            {
                body.temp_depth = depth;
                return None;
            }
        }
        init.push(op_at(
            pc::Action::ScoreSet {
                score: child(&snapshot, &tag),
                value: discr,
            },
            body.file,
            span,
        ));
        for field in self
            .struct_fields(def)
            .into_iter()
            .skip(1)
            .filter(|field| !fields.contains(field))
        {
            let base = child(&snapshot, &self.defs[field.0].name);
            let ty = self.field_ty(field);
            let defaults = self.record_constant(body.module, body.file, None, ty)?;
            for ((slot, _), value) in self.value_slots(&base, ty).into_iter().zip(defaults) {
                init.push(op_at(Self::initialize_leaf(slot, value), body.file, span));
            }
        }
        ops.extend(init);
        self.copy_value(dest, &snapshot, ty, body.file, span, ops);
        body.temp_depth = depth;
        Some(())
    }
}
