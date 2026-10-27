//! Owned living-entity effect queries using the shared predicate resource model.
use super::{Actions, Cmd};
use sand_commands::{
    CommandError, CommandProfile, CommandResult, ConditionIr, ExecuteOp, Selector,
};
use sand_components::predicate::{EntityPredicateTarget, Predicate, PredicateRoot};
use sand_components::{EffectId, EffectPredicate, EntityPredicate, ResourceLocation};

/// Effect presence retained through boolean composition and repeated export.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct EffectPresence {
    pub(crate) target: Selector,
    pub(crate) effect: EffectId,
}

impl EffectPresence {
    pub(crate) fn register(&self) -> String {
        self.reference(true)
    }

    /// Compute the same owned identity without registering discarded Actions.
    pub(crate) fn reference(&self, register: bool) -> String {
        let body = Actions(vec![Cmd::EffectPresence(Box::new(self.clone()))]);
        let prefix = "sand/conditions/effects";
        let path = if register {
            crate::function::register_dyn_fn_dedup(prefix, body)
        } else {
            crate::function::generated_function_path(prefix, &body)
        };
        format!("{}:{path}", crate::function::SAND_LOCAL_NS)
    }

    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let error =
            |detail| CommandError::new("effect presence", "target", format!("{owner}: {detail}"));
        sand_commands::ScoreHolder::entity(self.target.clone())
            .validate_single(&CommandProfile::unprofiled())
            .map_err(|e| error(e.to_string()))?;
        let path =
            crate::function::generated_resource_path("sand/effects", &self.effect.to_string());
        let location =
            ResourceLocation::new(owner.namespace(), &path).map_err(|e| error(e.to_string()))?;
        let predicate = Predicate::new(
            location.clone().into(),
            PredicateRoot::entity_properties(
                EntityPredicateTarget::This,
                EntityPredicate::new().effect(self.effect.clone(), EffectPredicate::new()),
            ),
        );
        crate::function::register_generated_component(predicate);
        Ok(vec![
            Cmd::Execute {
                operations: vec![
                    ExecuteOp::As(self.target.clone()),
                    ExecuteOp::If(ConditionIr::Predicate(location.to_string())),
                ],
                run: Box::new(Cmd::Return(1)),
            },
            Cmd::Return(0),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_presence_rejects_plural_owners_before_registering_resources() {
        let query = EffectPresence {
            target: Selector::all_entities(),
            effect: EffectId::Speed,
        };
        let error = query
            .lower(&"game:check".parse().unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains("game:check"), "{error}");
        assert!(error.contains("single") || error.contains("one"), "{error}");
    }
}
