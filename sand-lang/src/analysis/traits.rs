//! Trait bounds select statically checked impls through the existing method model.
use super::*;
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct Traits {
    pub declarations: BTreeMap<DefId, Vec<TraitMethod>>,
    pub implementations: BTreeMap<(DefId, DefId), (FileId, Span)>,
    pub function_bounds: BTreeMap<DefId, Vec<(usize, DefId)>>,
}

impl Cx {
    pub(super) fn collect_trait(&mut self, def: DefId, declaration: &TraitItem) {
        let file = self.defs[def.0].file;
        let mut seen = HashSet::new();
        for method in &declaration.methods {
            if !seen.insert(&method.name.name) {
                self.error("trait", file, method.name.span, "duplicate trait method");
            }
            let mut params = HashSet::new();
            for parameter in &method.params {
                if !params.insert(&parameter.name.name) {
                    self.error(
                        "trait",
                        file,
                        parameter.name.span,
                        "duplicate trait method parameter",
                    );
                }
            }
        }
        self.traits
            .declarations
            .insert(def, declaration.methods.clone());
    }

    pub(super) fn register_trait_impl(
        &mut self,
        contract: DefId,
        target: DefId,
        file: FileId,
        span: Span,
    ) {
        if let Some((prior_file, prior_span)) = self
            .traits
            .implementations
            .insert((contract, target), (file, span))
        {
            self.diagnostics.push(
                Diagnostic::error(
                    "trait",
                    file,
                    span,
                    "this trait is already implemented for this type",
                )
                .with_note(prior_file, prior_span, "previous implementation"),
            );
        }
    }

    /// Validate the authored method set even when the generic target is unused.
    pub(super) fn validate_trait_members(
        &mut self,
        contract: DefId,
        file: FileId,
        span: Span,
        methods: &mut [Item],
    ) {
        let required = self.traits.declarations[&contract].clone();
        let mut named = HashSet::new();
        for method in methods {
            let ItemKind::Fn(function) = &method.kind else {
                continue;
            };
            named.insert(function.name.name.clone());
            let Some(signature) = required.iter().find(|r| r.name.name == function.name.name)
            else {
                self.error(
                    "trait",
                    file,
                    function.name.span,
                    "method is not in this trait",
                );
                continue;
            };
            let receiver = |receiver: Option<&Receiver>| receiver.map(|r| (r.borrowed, r.mutable));
            if receiver(signature.receiver.as_ref()) != receiver(function.receiver.as_ref()) {
                self.error(
                    "trait",
                    file,
                    function.name.span,
                    "receiver must match the trait signature",
                );
            }
            if !function.generics.is_empty() {
                self.error(
                    "trait",
                    file,
                    function.name.span,
                    "trait methods cannot declare their own type parameters",
                );
            }
            // The trait controls visibility of its contract. Methods are
            // callable anywhere the implementing type and trait are visible.
            method.is_pub = self.defs[contract.0].is_pub;
        }
        for signature in required {
            if !named.contains(&signature.name.name) {
                self.error(
                    "trait",
                    file,
                    span,
                    format!("missing trait method `{}`", signature.name.name),
                );
            }
        }
    }

    pub(super) fn validate_trait_signatures(&mut self, checked: &mut BTreeSet<(DefId, DefId)>) {
        loop {
            let next = self
                .traits
                .implementations
                .iter()
                .find(|(key, _)| !checked.contains(*key));
            let Some((&(contract, target), &(file, span))) = next else {
                break;
            };
            checked.insert((contract, target));
            let signatures = self.traits.declarations[&contract].clone();
            let module = self.defs[contract.0].module;
            let mut bindings = BTreeMap::new();
            bindings.insert("Self".to_owned(), (Ty::Struct(target), target));
            let saved = std::mem::replace(&mut self.generics.bindings, bindings);
            for signature in signatures {
                let Some(method) = self.members[&target].get(&signature.name.name).copied() else {
                    continue;
                };
                let Some(slots) = self.fn_slots.get(&method).cloned() else {
                    continue;
                };
                let expected: Vec<_> = signature
                    .params
                    .iter()
                    .map(|param| self.resolve_type(module, &param.ty))
                    .collect();
                let expected_return = signature
                    .ret
                    .as_ref()
                    .map(|ty| self.resolve_type(module, ty));
                let found: Vec<_> = slots
                    .params
                    .iter()
                    .skip(usize::from(signature.receiver.is_some()))
                    .map(|(_, _, ty)| *ty)
                    .collect();
                if expected != found || expected_return != slots.ret.as_ref().map(|(_, ty)| *ty) {
                    let method = &self.defs[method.0];
                    self.diagnostics.push(
                        Diagnostic::error(
                            "trait",
                            file,
                            method.name_span,
                            format!(
                                "method `{}` has an incompatible trait signature",
                                method.name
                            ),
                        )
                        .with_note(
                            self.defs[contract.0].file,
                            signature.name.span,
                            "required signature",
                        )
                        .with_note(
                            file,
                            span,
                            "implementation for this type",
                        ),
                    );
                }
            }
            self.generics.bindings = saved;
        }
    }

    pub(super) fn collect_trait_bounds(&mut self, def: DefId, function: &FnItem) {
        let module = self.defs[def.0].module;
        let file = self.defs[def.0].file;
        let mut bounds = Vec::new();
        for (parameter, path) in &function.bounds {
            let Some(position) = function
                .generics
                .iter()
                .position(|name| name.name == parameter.name)
            else {
                continue;
            };
            let Some(contract) = self.expect_path(module, path, &[DefKind::Trait], "a trait")
            else {
                continue;
            };
            if bounds.contains(&(position, contract)) {
                self.error("trait", file, path.span, "duplicate trait bound");
            } else {
                bounds.push((position, contract));
            }
        }
        self.traits.function_bounds.insert(def, bounds);
    }

    pub(super) fn trait_bound_holds(&self, ty: Ty, contract: DefId) -> bool {
        matches!(ty, Ty::Struct(target) if self.traits.implementations.contains_key(&(contract, target)))
    }
}
