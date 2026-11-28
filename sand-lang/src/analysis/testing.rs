//! Test entry points share normal function lowering and typed return slots.
use super::*;

/// A discovered Sand test and its command entry point in a test build.
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Source-qualified test name.
    pub name: String,
    /// Generated server-context wrapper, present in test builds only.
    pub runner: String,
    /// Declaration file.
    pub file: FileId,
    /// Declaration name span.
    pub span: Span,
}

impl Cx {
    pub(super) fn declare_test(&mut self, def: DefId, item: &Item, function: &FnItem) {
        let Some(attribute) = item.attr("test") else {
            return;
        };
        let file = self.defs[def.0].file;
        if item.attrs.len() != 1 || !attribute.args.is_empty() {
            self.error(
                "test",
                file,
                attribute.span,
                "`#[test]` takes no arguments or other function attributes",
            );
        }
        if !function.params.is_empty()
            || self.methods.contains_key(&def)
            || !function.generics.is_empty()
            || self
                .fn_slots
                .get(&def)
                .and_then(|slots| slots.ret.as_ref())
                .is_none_or(|(_, ty)| *ty != Ty::Bool)
        {
            self.error(
                "test",
                file,
                function.name.span,
                "a test must be a plain function with signature `fn name() -> bool`",
            );
        }
        self.test_functions.insert(def);
    }

    pub(super) fn discover_tests(&mut self, lowerer: &Lowering) {
        for (owner, facts) in &lowerer.facts {
            if self.test_functions.contains(owner) {
                continue;
            }
            for (callee, span) in facts.all_calls.iter().chain(&facts.schedules) {
                if self.test_functions.contains(callee) {
                    self.error(
                        "test",
                        facts.file,
                        *span,
                        "test functions can only be called by other tests",
                    );
                }
            }
        }
        for def in self.test_functions.clone() {
            let d = self.defs[def.0].clone();
            if !matches!(
                d.info,
                DefInfo::Function {
                    context: Context::Server,
                    ..
                }
            ) {
                self.error(
                    "test",
                    d.file,
                    d.name_span,
                    "a test must run without a player or entity executor",
                );
            }
            let name = format!(
                "{}::{}",
                self.package.module(d.module).display_path(),
                d.name
            );
            let runner = format!("{}:sand.lang_test/{}", self.namespace(), self.tests.len());
            self.tests.insert(
                def,
                TestCase {
                    name,
                    runner,
                    file: d.file,
                    span: d.name_span,
                },
            );
        }
    }

    pub(super) fn test_wrapper(
        &self,
        def: DefId,
        test: &TestCase,
        index: usize,
    ) -> Option<pc::Function> {
        let (returned, Ty::Bool) = self.fn_slots.get(&def)?.ret.as_ref()? else {
            return None;
        };
        let score = pc::FieldReference {
            state: self.locals_state_id().parse().ok()?,
            field: returned.clone(),
        };
        let at = |action| op_at(action, test.file, test.span);
        let mark = |result| {
            at(pc::Action::Raw {
                command: format!("say __SAND_TEST_{index}_{result}__"),
            })
        };
        Some(pc::Function {
            id: test.runner.parse().ok()?,
            context: pc::ExecutionContext::Server,
            body: vec![
                at(pc::Action::ScoreSet {
                    score: score.clone(),
                    value: 0,
                }),
                at(pc::Action::Call {
                    function: self.function_reference(def)?,
                }),
                at(pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score,
                        min: Some(1),
                        max: Some(1),
                    }),
                    then: vec![mark("PASS")],
                    otherwise: vec![mark("FAIL")],
                }),
            ],
        })
    }
}
