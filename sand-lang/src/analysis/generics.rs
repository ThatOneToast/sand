//! Generic templates have authored identities; concrete instances have separate
//! slots, contexts and call-graph nodes, with references retaining their origins.
use super::*;
use std::collections::{BTreeSet, VecDeque};

type TypeBindings = BTreeMap<String, (Ty, DefId)>;

#[derive(Clone)]
pub(super) struct GenericImpl {
    contract: Option<DefId>,
    span: Span,
    module: ModuleId,
    arguments: Vec<Type>,
    parameters: Vec<(Ident, DefId)>,
    methods: Vec<(DefId, Item)>,
}

pub(super) struct PendingBody {
    def: DefId,
    source: DefId,
    item: Item,
    bindings: TypeBindings,
}

#[derive(Default)]
pub(super) struct Generics {
    pub records: BTreeMap<DefId, (StructItem, Vec<DefId>)>,
    pub record_instances: BTreeMap<(DefId, Vec<Ty>), DefId>,
    pub record_arguments: BTreeMap<DefId, (DefId, Vec<Ty>)>,
    pub record_depth: usize,
    pub templates: BTreeMap<DefId, (Item, Vec<DefId>)>,
    pub instances: BTreeMap<(DefId, Vec<Ty>), DefId>,
    pub implementations: BTreeMap<DefId, Vec<GenericImpl>>,
    pub bases: BTreeMap<DefId, TypeBindings>,
    pub pending: VecDeque<PendingBody>,
    pub bindings: BTreeMap<String, (Ty, DefId)>,
    pub active: Option<DefId>,
    pub origins: BTreeMap<DefId, DefId>,
    pub source_defs: BTreeMap<(DefId, FileId, Span), DefId>,
}

impl Cx {
    pub(super) fn register_generic(&mut self, def: DefId, item: &Item, function: &FnItem) {
        let owner = self.defs[def.0].clone();
        if let Some(receiver) = &function.receiver
            && !self.methods.contains_key(&def)
        {
            self.error(
                "receiver",
                owner.file,
                receiver.span,
                "`self` parameters belong inside an impl block",
            );
        }
        let parameters = self.generic_parameters(def, &function.generics);
        self.collect_trait_bounds(def, function);
        for attr in &item.attrs {
            if attr.name() != "player" {
                self.error(
                    "generic",
                    owner.file,
                    attr.span,
                    "generic functions are called with concrete types and cannot be entry points",
                );
            }
        }
        if let DefInfo::Function { signature, .. } = &mut self.defs[def.0].info {
            let text = &self.package.files[owner.file.0].text;
            *signature = text
                .get(function.name.span.end..function.body.span.start)
                .unwrap_or("")
                .trim()
                .to_owned();
        }
        self.generics
            .templates
            .insert(def, (item.clone(), parameters));
    }

    pub(super) fn generic_parameters(&mut self, def: DefId, declared: &[Ident]) -> Vec<DefId> {
        let owner = self.defs[def.0].clone();
        self.generic_parameters_at(def, declared, owner.module, owner.file, owner.span)
    }

    fn generic_parameters_at(
        &mut self,
        def: DefId,
        declared: &[Ident],
        module: ModuleId,
        file: FileId,
        scope: Span,
    ) -> Vec<DefId> {
        let mut names = HashSet::new();
        let mut parameters = Vec::new();
        for name in declared {
            if matches!(
                name.name.as_str(),
                "number" | "float" | "bool" | "string" | "EntityRef" | "Self" | "Vec"
            ) {
                self.error(
                    "generic",
                    file,
                    name.span,
                    "a type parameter cannot shadow a builtin type",
                );
            }
            if !names.insert(name.name.clone()) {
                self.error("generic", file, name.span, "duplicate type parameter");
            }
            let parameter = self.push_def(Def {
                kind: DefKind::TypeParameter,
                name: name.name.clone(),
                module,
                file,
                name_span: name.span,
                span: scope,
                docs: Vec::new(),
                is_pub: false,
                parent: Some(def),
                info: DefInfo::TypeParameter,
            });
            let source = self.generics.origins.get(&def).copied().unwrap_or(def);
            let key = (source, file, name.span);
            if let Some(origin) = self.generics.source_defs.get(&key).copied() {
                self.generics.origins.insert(parameter, origin);
            } else {
                self.generics.source_defs.insert(key, parameter);
            }
            parameters.push(parameter);
        }
        parameters
    }

    /// Resolve a callable, specializing a template with inferred or explicit types.
    pub(super) fn resolve_call(
        &mut self,
        body: &BodyCx,
        path: &Path,
        args: &[Arg],
        expected: Option<Ty>,
        report: bool,
    ) -> Option<DefId> {
        let def = if report {
            self.expect_path(
                body.module,
                path,
                &[DefKind::Function, DefKind::ExternFunction],
                "a function",
            )?
        } else {
            self.resolve_path(body.module, path).ok()?.0
        };
        self.specialize_call(body, def, path, args, expected, report)
    }

    pub(super) fn specialize_call(
        &mut self,
        body: &BodyCx,
        def: DefId,
        path: &Path,
        args: &[Arg],
        expected: Option<Ty>,
        report: bool,
    ) -> Option<DefId> {
        let Some((item, parameters)) = self.generics.templates.get(&def).cloned() else {
            if !path.arguments().is_empty() && report {
                self.error(
                    "generic",
                    body.file,
                    path.span,
                    "this function has no type parameters",
                );
                return None;
            }
            return Some(def);
        };
        let ItemKind::Fn(function) = &item.kind else {
            return None;
        };
        let mut inferred = BTreeMap::new();
        if !path.arguments().is_empty() {
            if path.arguments().len() != parameters.len() {
                if report {
                    self.error(
                        "generic",
                        body.file,
                        path.span,
                        format!(
                            "expected {} type arguments, found {}",
                            parameters.len(),
                            path.arguments().len()
                        ),
                    );
                }
                return None;
            }
            for (name, annotation) in function.generics.iter().zip(path.arguments()) {
                let ty = self.resolve_type(body.module, annotation);
                inferred.insert(name.name.clone(), ty);
            }
        }
        for (param, arg) in function.params.iter().zip(args) {
            if let ArgValue::Expr(value) = &arg.value {
                let ty = self.type_of(body, value);
                self.infer_type_arguments(
                    self.defs[def.0].module,
                    &param.ty,
                    ty,
                    &function.generics,
                    &mut inferred,
                );
            }
        }
        if let (Some(ret), Some(expected)) = (&function.ret, expected) {
            self.infer_type_arguments(
                self.defs[def.0].module,
                ret,
                expected,
                &function.generics,
                &mut inferred,
            );
        }
        let mut types = Vec::new();
        for name in &function.generics {
            let ty = inferred.get(&name.name).copied().unwrap_or(Ty::Unknown);
            if ty == Ty::Unknown {
                if report {
                    self.error(
                        "generic",
                        body.file,
                        path.span,
                        format!(
                            "cannot infer `{}`; supply explicit type arguments with `::<...>`",
                            name.name
                        ),
                    );
                }
                return None;
            }
            types.push(ty);
        }
        if let Some(bounds) = self.traits.function_bounds.get(&def) {
            for &(index, contract) in bounds {
                let ty = types[index];
                if !self.trait_bound_holds(ty, contract) {
                    if report {
                        self.error(
                            "trait",
                            body.file,
                            path.span,
                            format!(
                                "`{}` does not implement trait `{}` required by `{}`",
                                self.ty_name(ty),
                                self.defs[contract.0].name,
                                function.generics[index].name
                            ),
                        );
                    }
                    return None;
                }
            }
        }
        if let Some(instance) = self.generics.instances.get(&(def, types.clone())) {
            let instance = *instance;
            if report {
                self.specialized_reference(body, path, def, instance);
            }
            return Some(instance);
        }
        if self.generics.instances.len() >= 256 {
            if report {
                self.error(
                    "generic",
                    body.file,
                    path.span,
                    "a package supports at most 256 generic function specializations",
                );
            }
            return None;
        }
        let mut bindings = self.generics.bases.get(&def).cloned().unwrap_or_default();
        bindings.extend(
            function
                .generics
                .iter()
                .zip(types.iter().copied())
                .zip(parameters)
                .map(|((name, ty), parameter)| (name.name.clone(), (ty, parameter)))
                .collect::<BTreeMap<_, _>>(),
        );
        let mut concrete = self.defs[def.0].clone();
        if let DefInfo::Function { id, .. } = &mut concrete.info {
            *id = format!("{}.generic/{}", id, self.generics.instances.len());
        }
        let source = self.generics.origins.get(&def).copied().unwrap_or(def);
        let active = self.generics.active.replace(source);
        let saved = std::mem::replace(&mut self.generics.bindings, bindings.clone());
        let instance = self.push_def(concrete);
        self.generics.origins.insert(instance, source);
        self.generics.instances.insert((def, types), instance);
        if let Some(method) = self.methods.get(&def).cloned() {
            self.methods.insert(instance, method);
        }
        self.declare_slots(
            self.defs[def.0].module,
            self.defs[def.0].file,
            instance,
            &function.params,
            function.ret.as_ref(),
        );
        self.generics.bindings = saved;
        self.generics.active = active;
        self.generics.pending.push_back(PendingBody {
            def: instance,
            source,
            item: item.clone(),
            bindings,
        });
        if report {
            self.specialized_reference(body, path, def, instance);
        }
        Some(instance)
    }

    fn specialized_reference(
        &mut self,
        body: &BodyCx,
        path: &Path,
        template: DefId,
        instance: DefId,
    ) {
        if let Some(reference) = self
            .references
            .iter_mut()
            .rev()
            .find(|r| r.file == body.file && r.span == path.last().span && r.def == template)
        {
            reference.def = instance;
        }
    }

    pub(super) fn register_generic_impl(
        &mut self,
        module: ModuleId,
        file: FileId,
        target: DefId,
        implementation: ImplItem,
        scope: Span,
        contract: Option<DefId>,
    ) {
        let Some((record, _)) = self.generics.records.get(&target) else {
            return;
        };
        if implementation.target.arguments().len() != record.generics.len() {
            self.error(
                "generic",
                file,
                implementation.target.span,
                format!(
                    "this impl needs {} target type arguments",
                    record.generics.len()
                ),
            );
            return;
        }
        for parameter in &implementation.generics {
            if !implementation
                .target
                .arguments()
                .iter()
                .any(|ty| mentions_parameter(ty, &parameter.name))
            {
                self.error(
                    "generic",
                    file,
                    parameter.span,
                    "impl type parameter is not determined by the target type",
                );
            }
        }
        let parameter_ids =
            self.generic_parameters_at(target, &implementation.generics, module, file, scope);
        let mut methods = Vec::new();
        for method in implementation.methods {
            if let Some((def, item)) = self.collect_method(module, file, target, false, method) {
                if let ItemKind::Fn(function) = &item.kind
                    && !function.generics.is_empty()
                {
                    self.register_generic(def, &item, function);
                }
                methods.push((def, item));
            }
        }
        self.generics
            .implementations
            .entry(target)
            .or_default()
            .push(GenericImpl {
                contract,
                span: scope,
                module,
                arguments: implementation.target.arguments().to_vec(),
                parameters: implementation
                    .generics
                    .into_iter()
                    .zip(parameter_ids)
                    .collect(),
                methods,
            });
    }

    fn instantiate_record_methods(&mut self, template: DefId, instance: DefId) {
        let implementations = self
            .generics
            .implementations
            .get(&template)
            .cloned()
            .unwrap_or_default();
        let (_, actual) = self.generics.record_arguments[&instance].clone();
        for implementation in &implementations {
            let names: Vec<_> = implementation
                .parameters
                .iter()
                .map(|(name, _)| name.clone())
                .collect();
            let mut inferred = BTreeMap::new();
            for (annotation, ty) in implementation.arguments.iter().zip(&actual) {
                self.infer_type_arguments(
                    implementation.module,
                    annotation,
                    *ty,
                    &names,
                    &mut inferred,
                );
            }
            let mut bindings = BTreeMap::new();
            let mut missing = false;
            for (name, parameter) in &implementation.parameters {
                if let Some(ty) = inferred.get(&name.name) {
                    bindings.insert(name.name.clone(), (*ty, *parameter));
                } else {
                    self.error(
                        "generic",
                        self.defs[template.0].file,
                        name.span,
                        "impl type parameter is not determined by the target type",
                    );
                    missing = true;
                }
            }
            if missing {
                continue;
            }
            bindings.insert("Self".into(), (Ty::Struct(instance), template));
            let saved = std::mem::replace(&mut self.generics.bindings, bindings.clone());
            let resolved: Vec<_> = implementation
                .arguments
                .iter()
                .map(|ty| self.resolve_type(implementation.module, ty))
                .collect();
            if resolved != actual {
                self.generics.bindings = saved;
                continue;
            }
            if let Some(contract) = implementation.contract {
                self.register_trait_impl(
                    contract,
                    instance,
                    self.package.module(implementation.module).file,
                    implementation.span,
                );
            }
            for (source, item) in &implementation.methods {
                let ItemKind::Fn(function) = &item.kind else {
                    continue;
                };
                let mut concrete = self.defs[source.0].clone();
                concrete.parent = Some(instance);
                if let DefInfo::Function { id, .. } = &mut concrete.info {
                    *id = format!("{id}.type/{}", instance.0);
                }
                let active = self.generics.active.replace(*source);
                let def = self.push_def(concrete);
                self.generics.origins.insert(def, *source);
                let mut method = self.methods[source].clone();
                method.target = instance;
                self.methods.insert(def, method);
                self.members
                    .get_mut(&instance)
                    .expect("record members")
                    .insert(function.name.name.clone(), def);
                if function.generics.is_empty() {
                    self.declare_slots(
                        implementation.module,
                        self.defs[source.0].file,
                        def,
                        &function.params,
                        function.ret.as_ref(),
                    );
                    self.generics.pending.push_back(PendingBody {
                        def,
                        source: *source,
                        item: item.clone(),
                        bindings: bindings.clone(),
                    });
                } else {
                    self.register_generic(def, item, function);
                    self.generics.bases.insert(def, bindings.clone());
                }
                self.generics.active = active;
            }
            self.generics.bindings = saved;
        }
    }

    /// Instantiate a record using the existing nominal field/layout model.
    pub(super) fn instantiate_record(
        &mut self,
        template: DefId,
        types: Vec<Ty>,
        file: FileId,
        span: Span,
    ) -> Option<Ty> {
        let Some((record, parameters)) = self.generics.records.get(&template).cloned() else {
            self.error("generic", file, span, "this type has no type parameters");
            return None;
        };
        if types.len() != parameters.len() {
            self.error(
                "generic",
                file,
                span,
                format!(
                    "expected {} type arguments, found {}",
                    parameters.len(),
                    types.len()
                ),
            );
            return None;
        }
        if types.contains(&Ty::Unknown) {
            return None;
        }
        if let Some(instance) = self
            .generics
            .record_instances
            .get(&(template, types.clone()))
        {
            return Some(Ty::Struct(*instance));
        }
        if self.generics.record_instances.len() >= 256 || self.generics.record_depth >= 64 {
            self.error(
                "generic",
                file,
                span,
                "generic record expansion exceeds 256 instances or 64 nested specializations",
            );
            return None;
        }
        let owner = self.defs[template.0].clone();
        let mut concrete = owner.clone();
        concrete.name = format!(
            "{}<{}>",
            owner.name,
            types
                .iter()
                .map(|ty| self.ty_name(*ty))
                .collect::<Vec<_>>()
                .join(", ")
        );
        concrete.info = DefInfo::Struct { fields: Vec::new() };
        let active = self.generics.active.replace(template);
        let instance = self.push_def(concrete);
        self.generics.origins.insert(instance, template);
        self.generics
            .record_instances
            .insert((template, types.clone()), instance);
        self.generics
            .record_arguments
            .insert(instance, (template, types.clone()));
        let mut bindings: BTreeMap<_, _> = record
            .generics
            .iter()
            .zip(types)
            .zip(parameters)
            .map(|((name, ty), parameter)| (name.name.clone(), (ty, parameter)))
            .collect();
        bindings.insert("Self".into(), (Ty::Struct(instance), template));
        let saved = std::mem::replace(&mut self.generics.bindings, bindings);
        self.generics.record_depth += 1;
        let mut fields = Vec::new();
        let mut members = BTreeMap::new();
        for field in self.struct_fields(template) {
            let mut concrete = self.defs[field.0].clone();
            concrete.parent = Some(instance);
            let next = self.push_def(concrete);
            self.generics.origins.insert(next, field);
            members.insert(self.defs[field.0].name.clone(), next);
            fields.push(next);
        }
        self.defs[instance.0].info = DefInfo::Struct {
            fields: fields.clone(),
        };
        self.members.insert(instance, members);
        for (field, declared) in fields.into_iter().zip(&record.fields) {
            let ty = self.resolve_type(owner.module, &declared.ty);
            self.defs[field.0].info = DefInfo::Field {
                ty,
                default: None,
                bounds: None,
                timer: false,
            };
        }
        self.instantiate_record_methods(template, instance);
        self.generics.record_depth -= 1;
        self.generics.bindings = saved;
        self.generics.active = active;
        if self.layout_leaves(Ty::Struct(instance)).is_none() {
            self.error(
                "struct",
                file,
                span,
                "struct layouts must be finite, at most 32 levels deep and 1024 scalar fields",
            );
            self.defs[instance.0].info = DefInfo::Struct { fields: Vec::new() };
            return None;
        }
        Some(Ty::Struct(instance))
    }

    /// A contextual literal can omit arguments; otherwise infer from its fields.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_literal_type(
        &mut self,
        body: Option<&BodyCx>,
        module: ModuleId,
        file: FileId,
        path: &Path,
        fields: &[(Ident, Expr)],
        expected: Option<Ty>,
        report: bool,
    ) -> Option<Ty> {
        let def = if report {
            self.expect_path(module, path, &[DefKind::Struct], "a struct")?
        } else {
            self.resolve_path(module, path).ok()?.0
        };
        let Some((record, _)) = self.generics.records.get(&def).cloned() else {
            if !path.arguments().is_empty() {
                if report {
                    self.error(
                        "generic",
                        file,
                        path.span,
                        "this record has no type parameters",
                    );
                }
                return None;
            }
            return Some(Ty::Struct(def));
        };
        if path.arguments().is_empty()
            && let Some(Ty::Struct(instance)) = expected
            && self
                .generics
                .record_arguments
                .get(&instance)
                .is_some_and(|(template, _)| *template == def)
        {
            return Some(Ty::Struct(instance));
        }
        let types = if !path.arguments().is_empty() {
            path.arguments()
                .iter()
                .map(|ty| self.resolve_type(module, ty))
                .collect()
        } else {
            let mut inferred = BTreeMap::new();
            if let Some(body) = body {
                for (name, value) in fields {
                    if let Some(field) = record
                        .fields
                        .iter()
                        .find(|field| field.name.name == name.name)
                    {
                        let ty = self.type_of(body, value);
                        self.infer_type_arguments(
                            self.defs[def.0].module,
                            &field.ty,
                            ty,
                            &record.generics,
                            &mut inferred,
                        );
                    }
                }
            }
            let mut types = Vec::new();
            for name in &record.generics {
                let Some(ty) = inferred.get(&name.name).copied() else {
                    if report {
                        self.error(
                            "generic",
                            file,
                            path.span,
                            format!(
                                "cannot infer `{}`; annotate the value or use `::<...>`",
                                name.name
                            ),
                        );
                    }
                    return None;
                };
                types.push(ty);
            }
            types
        };
        self.instantiate_record(def, types, file, path.span)
    }

    /// Unification records candidates; the ordinary concrete call checks conflicts.
    fn infer_type_arguments(
        &mut self,
        module: ModuleId,
        annotation: &Type,
        ty: Ty,
        parameters: &[Ident],
        inferred: &mut BTreeMap<String, Ty>,
    ) {
        match &annotation.kind {
            TypeKind::Named(path)
                if path.segments.len() == 1
                    && parameters.iter().any(|p| p.name == path.last().name) =>
            {
                if ty != Ty::Unknown {
                    inferred.entry(path.last().name.clone()).or_insert(ty);
                }
            }
            TypeKind::Applied { path, arguments }
                if path.last().name == "Vec" && arguments.len() == 1 =>
            {
                if let Some(element) = self
                    .list_element(ty)
                    .or_else(|| self.array_shape(ty).map(|(ty, _)| ty))
                {
                    self.infer_type_arguments(module, &arguments[0], element, parameters, inferred);
                }
            }
            TypeKind::Applied { path, arguments } => {
                if let Ty::Struct(instance) = ty
                    && let Some((template, types)) =
                        self.generics.record_arguments.get(&instance).cloned()
                    && self
                        .resolve_path(module, path)
                        .ok()
                        .is_some_and(|(def, _)| def == template)
                {
                    for (annotation, ty) in arguments.iter().zip(&types) {
                        self.infer_type_arguments(module, annotation, *ty, parameters, inferred);
                    }
                }
            }
            TypeKind::Array { element, .. } => {
                if let Some((ty, _)) = self.array_shape(ty) {
                    self.infer_type_arguments(module, element, ty, parameters, inferred);
                }
            }
            _ => {}
        }
    }

    pub(super) fn lower_generic_instances(
        &mut self,
        modules: &mut [pc::Module],
        lowerer: &mut Lowering,
    ) {
        let mut checked_traits = BTreeSet::new();
        loop {
            self.validate_trait_signatures(&mut checked_traits);
            let Some(pending) = self.generics.pending.pop_front() else {
                break;
            };
            let ItemKind::Fn(function) = &pending.item.kind else {
                continue;
            };
            let module = self.defs[pending.def.0].module;
            let file = self.defs[pending.def.0].file;
            let saved = std::mem::replace(&mut self.generics.bindings, pending.bindings);
            let active = self.generics.active.replace(pending.source);
            self.lower_fn(
                module,
                file,
                &pending.item,
                function,
                pending.def,
                &mut modules[module.0],
                lowerer,
            );
            self.generics.bindings = saved;
            self.generics.active = active;
        }
    }
}

fn mentions_parameter(ty: &Type, name: &str) -> bool {
    match &ty.kind {
        TypeKind::Named(path) => path.segments.len() == 1 && path.last().name == name,
        TypeKind::Applied { arguments, .. } => {
            arguments.iter().any(|ty| mentions_parameter(ty, name))
        }
        TypeKind::Array { element, .. } => mentions_parameter(element, name),
    }
}
