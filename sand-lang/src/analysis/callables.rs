//! Callable values carry a checked target and an owned environment. Dispatch only invokes
//! checked targets from this compilation and never substitutes a storage path
//! or function name from user-controlled data.
use super::*;
use sand::data::{NbtCompound, NbtValue};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Signature {
    pub context: Context,
    pub parameters: Vec<Ty>,
    pub returns: Option<Ty>,
}

#[derive(Default)]
pub(super) struct Callables {
    types: BTreeMap<Signature, DefId>,
    targets: BTreeMap<DefId, BTreeMap<String, CallableTarget>>,
    dispatchers: BTreeMap<DefId, DefId>,
    closures: BTreeMap<(ModuleId, Span, String, DefId), String>,
}

#[derive(Clone)]
struct CallableTarget {
    def: DefId,
    captures: Vec<Ty>,
    file: FileId,
    span: Span,
}

impl Cx {
    pub(super) fn callable_type(&mut self, module: ModuleId, signature: Signature) -> Ty {
        if let Some(def) = self.callables.types.get(&signature) {
            return Ty::Callable(*def);
        }
        let prefix = match signature.context {
            Context::Server => "",
            Context::Player => "player ",
            Context::Entity => "entity ",
        };
        let mut name = format!(
            "{prefix}fn({})",
            signature
                .parameters
                .iter()
                .map(|ty| self.ty_name(*ty))
                .collect::<Vec<_>>()
                .join(", ")
        );
        if let Some(ty) = signature.returns {
            name.push_str(&format!(" -> {}", self.ty_name(ty)));
        }
        let def = self.push_def(Def {
            kind: DefKind::Struct,
            name,
            module,
            file: self.package.module(module).file,
            name_span: Span::default(),
            span: Span::default(),
            docs: Vec::new(),
            is_pub: false,
            parent: Some(DefId(self.defs.len())),
            info: DefInfo::Callable {
                context: signature.context,
                parameters: signature.parameters.clone(),
                returns: signature.returns,
            },
        });
        self.callables.types.insert(signature, def);
        Ty::Callable(def)
    }

    pub(super) fn callable_signature(&self, ty: Ty) -> Option<Signature> {
        let Ty::Callable(def) = ty else { return None };
        let DefInfo::Callable {
            context,
            parameters,
            returns,
        } = &self.defs[def.0].info
        else {
            return None;
        };
        Some(Signature {
            context: *context,
            parameters: parameters.clone(),
            returns: *returns,
        })
    }

    pub(super) fn named_function_type(&mut self, module: ModuleId, def: DefId) -> Ty {
        self.named_function_type_in(module, def, Context::Server)
    }

    fn named_function_type_in(&mut self, module: ModuleId, def: DefId, context: Context) -> Ty {
        let Some(slots) = self.fn_slots.get(&def) else {
            return Ty::Unknown;
        };
        self.callable_type(
            module,
            Signature {
                context,
                parameters: slots.params.iter().map(|(_, _, ty)| *ty).collect(),
                returns: slots.ret.as_ref().map(|(_, ty)| *ty),
            },
        )
    }

    pub(super) fn emit_function_value(
        &mut self,
        body: &BodyCx,
        dest: &pc::FieldReference,
        expected: Ty,
        path: &Path,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let function =
            self.expect_path(body.module, path, &[DefKind::Function], "a named function")?;
        let DefInfo::Function { id, .. } = &self.defs[function.0].info else {
            return None;
        };
        let id = id.clone();
        if !path.arguments().is_empty()
            || self.generics.origins.contains_key(&function)
            || self.generics.templates.contains_key(&function)
            || self.test_functions.contains(&function)
            || self
                .methods
                .get(&function)
                .is_some_and(|method| method.receiver.is_some())
        {
            self.error("callable", body.file, path.span, "a function value must name an ordinary non-generic function or associated function");
            return None;
        }
        let context = self.callable_signature(expected)?.context;
        let found = self.named_function_type_in(body.module, function, context);
        let Ty::Callable(signature) = found else {
            return None;
        };
        if !self.types_agree(body.file, path.span, expected, found) {
            return None;
        }
        let key = format!("{id} {}", self.ty_name(found));
        self.register_callable_target(
            signature,
            key.clone(),
            CallableTarget {
                def: function,
                captures: Vec::new(),
                file: body.file,
                span: path.span,
            },
        )?;
        ops.push(op_at(
            Self::initialize_leaf(
                dest.clone(),
                text::LeafDefault::Data(Self::callable_payload(key)),
            ),
            body.file,
            path.span,
        ));
        Some(())
    }

    pub(super) fn empty_callable() -> NbtValue {
        NbtValue::Compound(NbtCompound::new())
    }

    fn callable_payload(id: String) -> NbtValue {
        let mut data = NbtCompound::new();
        data.insert("target", NbtValue::String(id));
        data.insert("captures", NbtValue::Compound(NbtCompound::new()));
        NbtValue::Compound(data)
    }

    fn register_callable_target(
        &mut self,
        signature: DefId,
        key: String,
        candidate: CallableTarget,
    ) -> Option<()> {
        let targets = self.callables.targets.entry(signature).or_default();
        if let Some(target) = targets.get(&key) {
            if target.def != candidate.def {
                self.diagnostics.push(
                    Diagnostic::error(
                        "callable",
                        candidate.file,
                        candidate.span,
                        "these callable target identities collide",
                    )
                    .with_note(
                        target.file,
                        target.span,
                        "other callable target",
                    ),
                );
                return None;
            }
        } else {
            if targets.len() >= 256 {
                self.error(
                    "callable",
                    candidate.file,
                    candidate.span,
                    "a callable signature supports at most 256 targets",
                );
                return None;
            }
            targets.insert(key, candidate);
        }
        Some(())
    }

    pub(super) fn emit_callable(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let span = expr.span();
        match expr {
            Expr::Closure(closure) => self.emit_closure(body, dest, ty, closure, ops),
            Expr::Path(path) => {
                if let Some(local) = Self::local_of(body, expr) {
                    if !self.types_agree(body.file, span, ty, self.local_ty(local)) {
                        return None;
                    }
                    body.used.insert(local);
                    self.references.push(Reference {
                        file: body.file,
                        span: path.last().span,
                        def: local,
                    });
                    let source = self.local_ref(local);
                    self.copy_value(dest, &source, ty, body.file, span, ops);
                    Some(())
                } else {
                    self.emit_function_value(body, dest, ty, path, ops)
                }
            }
            Expr::Place(place) => {
                let source = self.place_value(body, place, false)?;
                if !self.types_agree(body.file, span, ty, source.ty) {
                    return None;
                }
                self.copy_value(dest, &source.reference, ty, body.file, span, ops);
                Some(())
            }
            Expr::Call { path, args, .. } => self.emit_call(body, dest, ty, path, args, span, ops),
            Expr::Method {
                receiver,
                name,
                type_arguments,
                args,
                ..
            } => {
                let receiver_ty = self.type_of(body, receiver);
                if self.list_element(receiver_ty).is_some() {
                    return self.emit_list_method(body, dest, ty, expr, ops);
                }
                if self.array_shape(receiver_ty).is_some() {
                    return self.emit_array_method(body, dest, ty, expr, ops);
                }
                self.emit_method(
                    body,
                    dest,
                    ty,
                    receiver,
                    name,
                    type_arguments,
                    args,
                    span,
                    ops,
                )
            }
            Expr::Project(value, field, _) => {
                self.emit_projection(body, dest, ty, value, field, ops)
            }
            Expr::Index(..) => self.emit_index(body, dest, ty, expr, ops),
            _ => {
                self.error("callable", body.file, span, "expected a function value");
                None
            }
        }
    }

    pub(super) fn indirect_call(
        &mut self,
        body: &BodyCx,
        path: &Path,
        local: DefId,
        args: &[Arg],
    ) -> Option<(DefId, Vec<Arg>)> {
        let ty = self.local_ty(local);
        let Ty::Callable(signature) = ty else {
            self.error(
                "callable",
                body.file,
                path.span,
                "this local does not hold a function",
            );
            return None;
        };
        if !path.arguments().is_empty() {
            self.error(
                "callable",
                body.file,
                path.span,
                "function values have a concrete signature",
            );
            return None;
        }
        let dispatcher = self.callable_dispatcher(body, signature, path.span);
        let mut actual = Vec::with_capacity(args.len() + 1);
        actual.push(Arg {
            name: None,
            value: ArgValue::Expr(Expr::Path(path.clone())),
            span: path.span,
        });
        actual.extend_from_slice(args);
        Some((dispatcher, actual))
    }

    fn callable_dispatcher(&mut self, body: &BodyCx, signature: DefId, span: Span) -> DefId {
        if let Some(def) = self.callables.dispatchers.get(&signature) {
            return *def;
        }
        let name = self.defs[signature.0].name.clone();
        let id = format!("{}:sand.lang_call/{}", self.namespace(), signature.0);
        let def = self.push_def(Def {
            kind: DefKind::Function,
            name: name.clone(),
            module: body.module,
            file: body.file,
            name_span: Span::default(),
            span,
            docs: Vec::new(),
            is_pub: false,
            parent: Some(signature),
            info: DefInfo::Function {
                id,
                context: Context::Server,
                player_reason: None,
                entry: Vec::new(),
                signature: name,
            },
        });
        let sig = self
            .callable_signature(Ty::Callable(signature))
            .expect("interned signature");
        let mut params = Vec::new();
        for (index, ty) in std::iter::once(Ty::Callable(signature))
            .chain(sig.parameters)
            .enumerate()
        {
            let name = format!("argument_{index}");
            let param = self.push_def(Def {
                kind: DefKind::Local,
                name: name.clone(),
                module: body.module,
                file: body.file,
                name_span: Span::default(),
                span: Span::default(),
                docs: Vec::new(),
                is_pub: false,
                parent: Some(def),
                info: DefInfo::Local {
                    mutable: false,
                    param: true,
                    owner: Some(def),
                    ty,
                },
            });
            params.push((format!("v{}_{name}", param.0), param, ty));
        }
        let ret = sig.returns.map(|ty| (format!("f{}_ret", def.0), ty));
        self.fn_slots.insert(
            def,
            FnSlots {
                params,
                ret,
                receiver_out: None,
            },
        );
        self.callables.dispatchers.insert(signature, def);
        def
    }

    pub(super) fn lower_callable_dispatchers(&mut self, lowerer: &mut Lowering) {
        for (signature, def) in self.callables.dispatchers.clone() {
            let d = self.defs[def.0].clone();
            let DefInfo::Function { id, .. } = d.info else {
                continue;
            };
            let slots = self.fn_slots[&def].clone();
            let call_value = self.slot(slots.params[0].0.clone());
            self.value_slots(&call_value, Ty::Callable(signature));
            let mut target_selection = Self::data_reference(&call_value);
            target_selection.path.push(pc::DataProjection::Member {
                name: "target".into(),
            });
            let mut body = BodyCx::new(d.module, d.file, false);
            body.owner = Some(def);
            body.slot = format!("f{}", def.0);
            let mut arguments = Vec::new();
            for (_, param, _) in slots.params.iter().skip(1) {
                let name = self.defs[param.0].name.clone();
                body.scopes[0].push((name.clone(), *param));
                let path = Path {
                    segments: vec![Ident {
                        name,
                        span: Span::default(),
                    }],
                    applications: Vec::new(),
                    span: Span::default(),
                };
                arguments.push(Arg {
                    name: None,
                    value: ArgValue::Expr(Expr::Path(path)),
                    span: d.span,
                });
            }
            let targets: Vec<_> = self
                .callables
                .targets
                .get(&signature)
                .into_iter()
                .flatten()
                .map(|(key, target)| (key.clone(), target.clone()))
                .collect();
            let mut ops = Vec::new();
            if let Some((field, ty)) = &slots.ret {
                let result = self.slot(field.clone());
                if let Some(defaults) = self.record_constant(d.module, d.file, None, *ty) {
                    for ((score, _), default) in
                        self.value_slots(&result, *ty).into_iter().zip(defaults)
                    {
                        ops.push(op_at(Self::initialize_leaf(score, default), d.file, d.span));
                    }
                }
                body.ret = Some((result, *ty));
            }
            let expected_target = self.temp(&mut body);
            self.register_data_slot(&expected_target, Self::empty_callable());
            let comparison = self.temp(&mut body);
            for (key, target) in targets {
                ops.push(op_at(
                    Self::initialize_leaf(
                        expected_target.clone(),
                        text::LeafDefault::Data(NbtValue::String(key)),
                    ),
                    d.file,
                    d.span,
                ));
                ops.push(op_at(
                    pc::Action::DataEqual {
                        score: comparison.clone(),
                        left: target_selection.clone(),
                        right: Self::data_reference(&expected_target),
                    },
                    d.file,
                    d.span,
                ));
                let mut then = Vec::new();
                let mut all_arguments = Vec::new();
                for (index, ty) in target.captures.iter().enumerate() {
                    let name = format!("capture_{}", body.scopes[0].len());
                    let local = self.callback_local(def, &name, *ty);
                    body.scopes[0].push((name.clone(), local));
                    let slot = self.local_ref(local);
                    if let Some(defaults) = self.record_constant(d.module, d.file, None, *ty) {
                        for ((field, _), default) in
                            self.value_slots(&slot, *ty).into_iter().zip(defaults)
                        {
                            then.push(op_at(Self::initialize_leaf(field, default), d.file, d.span));
                        }
                    }
                    let source = Self::capture_selection(&call_value, index);
                    self.unpack_value(&mut body, &slot, source, *ty, d.span, &mut then);
                    all_arguments.push(Self::local_argument(name, d.span));
                }
                all_arguments.extend_from_slice(&arguments);
                let target = target.def;
                let name = self.defs[target.0].name.clone();
                let path = Path {
                    segments: vec![Ident { name, span: d.span }],
                    applications: Vec::new(),
                    span: d.span,
                };
                if let Some((dest, ty)) = body.ret.clone() {
                    self.emit_user_result(
                        &mut body,
                        &dest,
                        ty,
                        target,
                        &path,
                        &all_arguments,
                        d.span,
                        &mut then,
                    );
                } else {
                    let target_slots = self.fn_slots[&target].clone();
                    self.call_user(
                        &mut body,
                        target,
                        &target_slots,
                        &path,
                        &all_arguments,
                        d.span,
                        &mut then,
                    );
                }
                then.push(op_at(pc::Action::Return, d.file, d.span));
                ops.push(op_at(
                    pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score: comparison.clone(),
                            min: Some(1),
                            max: Some(1),
                        }),
                        then,
                        otherwise: Vec::new(),
                    },
                    d.file,
                    d.span,
                ));
            }
            let context = self
                .callable_signature(Ty::Callable(signature))
                .unwrap()
                .context;
            lowerer.facts.insert(
                def,
                Facts {
                    entity_direct: (context == Context::Entity)
                        .then(|| (d.span, "invokes an entity callback".into())),
                    entity_loop_calls: Vec::new(),
                    all_calls: body.all_calls,
                    schedules: Vec::new(),
                    forced: None,
                    direct: (context == Context::Player)
                        .then(|| (d.span, "invokes a player callback".into())),
                    calls: body.calls,
                    entry: Vec::new(),
                    file: d.file,
                },
            );
            lowerer.functions.push(PendingFn {
                module: d.module,
                file: d.file,
                def: Some(def),
                fixed: None,
                id,
                name_span: d.span,
                body: ops,
            });
        }
    }

    fn callback_local(&mut self, owner: DefId, name: &str, ty: Ty) -> DefId {
        let declaration = &self.defs[owner.0];
        self.push_def(Def {
            kind: DefKind::Local,
            name: name.into(),
            module: declaration.module,
            file: declaration.file,
            name_span: Span::default(),
            span: Span::default(),
            docs: Vec::new(),
            is_pub: false,
            parent: Some(owner),
            info: DefInfo::Local {
                ty,
                mutable: false,
                param: true,
                owner: Some(owner),
            },
        })
    }

    fn local_argument(name: String, span: Span) -> Arg {
        Arg {
            name: None,
            span,
            value: ArgValue::Expr(Expr::Path(Path {
                segments: vec![Ident {
                    name,
                    span: Span::default(),
                }],
                applications: Vec::new(),
                span: Span::default(),
            })),
        }
    }

    fn capture_selection(value: &pc::FieldReference, index: usize) -> pc::DataReference {
        let mut selection = Self::data_reference(value);
        selection.path.push(pc::DataProjection::Member {
            name: "captures".into(),
        });
        selection.path.push(pc::DataProjection::Member {
            name: format!("c{index}"),
        });
        selection
    }

    pub(super) fn closure_type(&mut self, module: ModuleId, closure: &ClosureExpr) -> Ty {
        self.closure_type_in(module, closure, Context::Server)
    }

    fn closure_type_in(&mut self, module: ModuleId, closure: &ClosureExpr, context: Context) -> Ty {
        let parameters = closure
            .parameters
            .iter()
            .map(|parameter| self.resolve_type(module, &parameter.ty))
            .collect();
        let returns = closure
            .returns
            .as_ref()
            .map(|ty| self.resolve_type(module, ty));
        self.callable_type(
            module,
            Signature {
                context,
                parameters,
                returns,
            },
        )
    }

    fn emit_closure(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        expected: Ty,
        closure: &ClosureExpr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let context = self.callable_signature(expected)?.context;
        let Ty::Callable(signature) = self.closure_type_in(body.module, closure, context) else {
            return None;
        };
        if !self.types_agree(body.file, closure.span, expected, Ty::Callable(signature)) {
            return None;
        }
        let mut captured = BTreeMap::new();
        let source = &self.package.files[body.file.0].text;
        let text = source
            .get(closure.body.span.start..closure.body.span.end)
            .unwrap_or_default();
        // Keep identifiers in interpolation as well as ordinary expressions.
        // Shadowed mentions may add a harmless immutable snapshot.
        let tokens: HashSet<_> = text
            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .collect();
        for scope in &body.scopes {
            for (name, local) in scope {
                if tokens.contains(name.as_str())
                    && !closure
                        .parameters
                        .iter()
                        .any(|parameter| parameter.name.name == *name)
                {
                    captured.insert(name.clone(), *local);
                }
            }
        }
        if captured.len() > 64 {
            self.error(
                "callable",
                body.file,
                closure.span,
                "a closure supports at most 64 captured locals",
            );
            return None;
        }

        let site = (body.module, closure.span, body.slot.clone(), signature);
        let target = if let Some(target) = self.callables.closures.get(&site) {
            target.clone()
        } else {
            if self.callables.closures.len() >= 256 {
                self.error(
                    "callable",
                    body.file,
                    closure.span,
                    "a package supports at most 256 concrete closures",
                );
                return None;
            }
            let target = self.declare_closure(body, closure, signature, &captured)?;
            self.callables.closures.insert(site, target.clone());
            target
        };
        let depth = body.temp_depth;
        let staged = self.temp(body);
        self.value_slots(&staged, Ty::Callable(signature));
        ops.push(op_at(
            Self::initialize_leaf(
                staged.clone(),
                text::LeafDefault::Data(Self::callable_payload(target)),
            ),
            body.file,
            closure.span,
        ));
        for (index, local) in captured.into_values().enumerate() {
            body.used.insert(local);
            let source = self.local_ref(local);
            self.pack_value(
                body,
                Self::capture_selection(&staged, index),
                &source,
                self.local_ty(local),
                closure.span,
                ops,
            );
        }
        self.copy_value(
            dest,
            &staged,
            Ty::Callable(signature),
            body.file,
            closure.span,
            ops,
        );
        body.temp_depth = depth;
        Some(())
    }

    fn declare_closure(
        &mut self,
        body: &BodyCx,
        closure: &ClosureExpr,
        signature: DefId,
        captured: &BTreeMap<String, DefId>,
    ) -> Option<String> {
        let index = self.callables.closures.len();
        let id = format!("{}:sand.lang_closure/{index}", self.namespace());
        let name = format!("closure_{index}");
        let func = self.push_def(Def {
            kind: DefKind::Function,
            name: name.clone(),
            module: body.module,
            file: body.file,
            name_span: Span::default(),
            span: closure.span,
            docs: Vec::new(),
            is_pub: false,
            parent: Some(signature),
            info: DefInfo::Function {
                id: id.clone(),
                context: Context::Server,
                player_reason: None,
                entry: Vec::new(),
                signature: String::new(),
            },
        });
        self.declare_slots(
            body.module,
            body.file,
            func,
            &closure.parameters,
            closure.returns.as_ref(),
        );
        let mut params = Vec::new();
        let mut capture_types = Vec::new();
        let mut schema = String::new();
        for (name, source) in captured {
            let ty = self.local_ty(*source);
            let local = self.callback_local(func, name, ty);
            self.generics.origins.insert(
                local,
                self.generics
                    .origins
                    .get(source)
                    .copied()
                    .unwrap_or(*source),
            );
            params.push((format!("v{}_{name}", local.0), local, ty));
            capture_types.push(ty);
            schema.push_str(&format!(" {name}:{}", self.ty_name(ty)));
        }
        let slots = self.fn_slots.get_mut(&func).expect("declared closure");
        params.append(&mut slots.params);
        slots.params = params;
        let function = FnItem {
            bounds: Vec::new(),
            generics: Vec::new(),
            receiver: None,
            name: Ident {
                name,
                span: closure.span,
            },
            params: closure.parameters.clone(),
            ret: closure.returns.clone(),
            body: closure.body.clone(),
        };
        let key = format!(
            "{id} {}:{}{schema}",
            self.package.module(body.module).display_path(),
            closure.span.start
        );
        self.register_callable_target(
            signature,
            key.clone(),
            CallableTarget {
                def: func,
                captures: capture_types,
                file: body.file,
                span: closure.span,
            },
        )?;
        self.queue_generated_body(
            func,
            Item {
                docs: Vec::new(),
                attrs: Vec::new(),
                is_pub: false,
                kind: ItemKind::Fn(function),
                span: closure.span,
            },
        );
        Some(key)
    }

    pub(super) fn validate_callable_contexts(&mut self) {
        for (signature, targets) in &self.callables.targets {
            let expected = self
                .callable_signature(Ty::Callable(*signature))
                .unwrap()
                .context;
            for target in targets.values() {
                let def = &self.defs[target.def.0];
                let DefInfo::Function {
                    context: actual, ..
                } = def.info
                else {
                    unreachable!("registered callable target is a function");
                };
                let message = match (expected, actual) {
                    (_, Context::Server)
                    | (Context::Player, _)
                    | (Context::Entity, Context::Entity) => continue,
                    (Context::Server, _) => {
                        "a function value must be callable without a player or entity executor"
                    }
                    (Context::Entity, Context::Player) => {
                        "an entity callback cannot require a player executor"
                    }
                };
                self.diagnostics.push(
                    Diagnostic::error("callable", target.file, target.span, message).with_note(
                        def.file,
                        def.span,
                        format!("this function runs as {}", actual.name()),
                    ),
                );
            }
        }
    }
}
