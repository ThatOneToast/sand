//! Structured panic payload for infallible participant accessors that
//! resolve to [`crate::participant::availability::ParticipantAvailability::Unavailable`]
//! (#280 item 2).
//!
//! [`plan::EventParticipantPlan::require_entity`](super::plan::EventParticipantPlan::require_entity)/
//! `require_item` panic with [`MissingParticipantPanic`] as the payload
//! (via [`std::panic::panic_any`]) instead of a formatted string, so the
//! export pipeline's handler-invocation boundary
//! (`sand-core/src/compiler/export/pipeline.rs`'s `invoke_event_handler_body`)
//! can downcast it and convert it into a structured `SAND-EVENT-PARTICIPANT`
//! diagnostic — never letting a raw, unhandled panic/backtrace reach a
//! `sand build` user. The boundary also installs a scoped panic hook that
//! silences the default panic printer for exactly this payload type (any
//! other panic — a genuine bug — still prints and propagates normally).
//!
//! This is the internal safety net described in `require_entity`'s own doc:
//! `sand build`'s graph/participant-transport validation
//! (`sand-core/src/compiler/export/participant_transport.rs`) is expected to
//! catch a missing/ambiguous participant declaration earlier, before any
//! handler body is even evaluated; this module exists for the case a
//! mistake reaches codegen without going through that path.

/// Which accessor family panicked — determines the diagnostic's suggested
/// [`crate::participant::ParticipantBuilder`] method names and role type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParticipantAccessorKind {
    Entity,
    Item,
}

impl ParticipantAccessorKind {
    fn role_type_name(self) -> &'static str {
        match self {
            Self::Entity => "EntityParticipantRole",
            Self::Item => "ItemParticipantRole",
        }
    }

    fn builder_observe_method(self) -> &'static str {
        match self {
            Self::Entity => "observe_entity",
            Self::Item => "observe_item",
        }
    }

    fn builder_inherit_method(self) -> &'static str {
        match self {
            Self::Entity => "inherit_entity",
            Self::Item => "inherit_item",
        }
    }

    /// Extra trailing constructor arguments a `ParticipantBuilder` call for
    /// this kind needs beyond the role itself (items also need a hand).
    fn extra_builder_args(self) -> &'static str {
        match self {
            Self::Entity => "",
            Self::Item => ", ParticipantHand::MainHand",
        }
    }
}

/// The panic payload [`std::panic::panic_any`] carries. Always `'static`
/// (owned fields only) so it satisfies `catch_unwind`'s bound regardless of
/// the borrowed-string lifetimes at the panic call site.
#[derive(Debug, Clone)]
pub(crate) struct MissingParticipantPanic {
    pub kind: ParticipantAccessorKind,
    pub event_label: String,
    pub role_debug: String,
    pub reason: &'static str,
}

/// `PascalCase` → `snake_case`, e.g. `"InteractedEntity"` → `"interacted_entity"`.
/// Best-effort accessor-method-name guess for the diagnostic's `Accessor:`
/// line, matching the named shorthand methods (`Event<E>::killer`/`.victim`/
/// `.weapon`/…, mirrored by `SandEventParticipants`) — cosmetic only, never
/// used for actual resolution.
fn pascal_case_to_snake_case(input: &str) -> String {
    let mut out = String::new();
    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

impl MissingParticipantPanic {
    /// Render the full `SAND-EVENT-PARTICIPANT` structured diagnostic body
    /// (everything after the `error[...]:` summary line) for `handler_path`
    /// — the one piece of context only the export pipeline's call site
    /// knows (the panic itself has no notion of which `#[event]` handler
    /// was executing when it fired).
    pub(crate) fn render(&self, handler_path: &str) -> String {
        let role_type = self.kind.role_type_name();
        let observe_method = self.kind.builder_observe_method();
        let inherit_method = self.kind.builder_inherit_method();
        let extra_args = self.kind.extra_builder_args();
        let accessor = pascal_case_to_snake_case(&self.role_debug);

        format!(
            "error[SAND-EVENT-PARTICIPANT]: unavailable event participant\n\n\
             Event: {event_label}\n\
             Handler: {handler_path}\n\
             Accessor: {accessor}\n\
             Required role: {role_type}::{role_debug}\n\n\
             This event does not observe or inherit the requested participant ({reason}).\n\n\
             Declare it with ParticipantBuilder, for example:\n\n\
             \x20   ParticipantBuilder::new()\n\
             \x20       .{observe_method}({role_type}::{role_debug}{extra_args})\n\
             \x20       .build()\n\n\
             or, if a same-cycle ancestor event already captures it:\n\n\
             \x20   ParticipantBuilder::new()\n\
             \x20       .{inherit_method}::<ParentEvent>({role_type}::{role_debug}{extra_args})\n\
             \x20       .build()",
            event_label = self.event_label,
            role_debug = self.role_debug,
            reason = self.reason,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_conversion_handles_multi_word_roles() {
        assert_eq!(pascal_case_to_snake_case("Killer"), "killer");
        assert_eq!(
            pascal_case_to_snake_case("InteractedEntity"),
            "interacted_entity"
        );
    }

    #[test]
    fn render_includes_all_required_fields() {
        let panic = MissingParticipantPanic {
            kind: ParticipantAccessorKind::Entity,
            event_label: "my_crate::SomeSandEvent".to_string(),
            role_debug: "Killer".to_string(),
            reason: "this role does not apply to this event",
        };
        let rendered = panic.render("invalid_sand");
        assert!(rendered.contains("error[SAND-EVENT-PARTICIPANT]"));
        assert!(rendered.contains("Event: my_crate::SomeSandEvent"));
        assert!(rendered.contains("Handler: invalid_sand"));
        assert!(rendered.contains("Accessor: killer"));
        assert!(rendered.contains("Required role: EntityParticipantRole::Killer"));
        assert!(rendered.contains("ParticipantBuilder::new()"));
        assert!(rendered.contains(".inherit_entity::<ParentEvent>(EntityParticipantRole::Killer)"));
    }

    #[test]
    fn render_item_kind_includes_hand_argument() {
        let panic = MissingParticipantPanic {
            kind: ParticipantAccessorKind::Item,
            event_label: "my_crate::SomeSandEvent".to_string(),
            role_debug: "Weapon".to_string(),
            reason: "this role does not apply to this event",
        };
        let rendered = panic.render("invalid_item_handler");
        assert!(rendered.contains("Required role: ItemParticipantRole::Weapon"));
        assert!(
            rendered
                .contains(".observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)")
        );
    }
}
