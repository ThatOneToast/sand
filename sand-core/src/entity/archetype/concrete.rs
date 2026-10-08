//! Compiler support for concrete archetype declarations. Composition, native
//! bindings and lifecycle remain owned by the canonical archetype compiler.
use super::{ArchetypeDefinition, EntityArchetype, generated_root, initialized_tag};
use crate::entity::{EntityDiagnostic, KnownEntityKind, SummonableEntityKind};
use crate::ir::{Cmd, ConditionIr, ExecuteOp};
#[doc(hidden)]
pub use sand_commands::Selector;
use sand_commands::{TargetArgument, Vec3};
use sand_components::ResourceLocation;

/// Check that native configuration retained the declaration's identity and
/// exact component composition before exposing it to the existing compiler.
pub fn configured_definition<K: KnownEntityKind>(
    declared: EntityArchetype<K>,
    configured: EntityArchetype<K>,
) -> Result<ArchetypeDefinition, EntityDiagnostic> {
    let declared = declared.definition();
    let configured = configured.definition();
    let same_components = declared.components.len() == configured.components.len()
        && declared
            .components
            .iter()
            .zip(&configured.components)
            .all(|(left, right)| {
                left.identities == right.identities && left.has_same_metadata(right)
            });
    if declared.id != configured.id || !same_components {
        return Err(EntityDiagnostic::InvalidArchetypeDeclaration {
            archetype: declared.id.to_string(),
            detail: format!(
                "native configuration must retain the declared identity and component fields; returned identity `{}`",
                configured.id
            ),
        });
    }
    Ok(configured)
}

fn lifecycle(id: &ResourceLocation, phase: &str) -> Cmd {
    Cmd::Function(format!(
        "{}:{}/{phase}",
        id.namespace(),
        generated_root(&id.to_string())
    ))
}

/// Summon at the authored position and initialize exactly the new executor.
pub(crate) fn summon<K: SummonableEntityKind>(
    id: &ResourceLocation,
    position: Vec3,
) -> Vec<String> {
    vec![
        Cmd::Execute {
            operations: vec![
                ExecuteOp::Positioned(position),
                ExecuteOp::Summon(K::entity_type().to_string()),
            ],
            run: Box::new(lifecycle(id, "initialize")),
        }
        .render(),
    ]
}

/// Attach the declaration to a statically proven current entity kind.
pub(crate) fn attach<K: KnownEntityKind>(id: &ResourceLocation) -> Vec<String> {
    vec![
        Cmd::Execute {
            operations: vec![
                ExecuteOp::If(ConditionIr::Entity(
                    Selector::self_().entity_type(K::entity_type()),
                )),
                ExecuteOp::Unless(ConditionIr::Entity(
                    Selector::self_().tag(initialized_tag(&id.to_string())),
                )),
            ],
            run: Box::new(lifecycle(id, "initialize")),
        }
        .render(),
    ]
}

/// Run canonical cleanup, including shared-component ownership guards.
pub(crate) fn detach<K: KnownEntityKind>(id: &ResourceLocation) -> Vec<String> {
    vec![
        Cmd::Execute {
            operations: vec![ExecuteOp::If(ConditionIr::Entity(selection::<K>(
                id,
                Selector::self_(),
            )))],
            run: Box::new(lifecycle(id, "cleanup")),
        }
        .render(),
    ]
}

/// Select the existing archetype membership without introducing a new query
/// representation. State query lowering adds component presence constraints.
pub fn selection<K: KnownEntityKind>(id: &ResourceLocation, selector: Selector) -> Selector {
    selector
        .entity_type(K::entity_type())
        .tag(initialized_tag(&id.to_string()))
}

/// Adopt selected entities once, preserving the author's selection and checking
/// each executor's actual kind instead of replacing a conflicting type filter.
pub(crate) fn adopt<K: KnownEntityKind>(
    id: &ResourceLocation,
    targets: impl TargetArgument,
) -> Vec<String> {
    vec![
        Cmd::Execute {
            operations: vec![
                ExecuteOp::As(targets.into_target_selector()),
                ExecuteOp::At(Selector::self_()),
                ExecuteOp::If(ConditionIr::Entity(
                    Selector::self_().entity_type(K::entity_type()),
                )),
                ExecuteOp::Unless(ConditionIr::Entity(
                    Selector::self_().tag(initialized_tag(&id.to_string())),
                )),
            ],
            run: Box::new(lifecycle(id, "initialize")),
        }
        .render(),
    ]
}

/// Type-level identity supplied by Archetype derive for shared typed operations.
#[doc(hidden)]
pub trait Declaration {
    type Kind: KnownEntityKind;
    fn archetype_id() -> ResourceLocation;
}
