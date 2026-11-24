//! Connect portable function continuations to the canonical owned lifetime runtime.
use super::*;
use crate::compiler::export::ExportCtx;
use crate::entity::kind::AnyEntity;
use crate::gameplay::Lifetime;
use crate::registration::{DatapackRegistration, IntoDatapack, LifecyclePhase};
use sand_components::{ResourceLocation, effect::Ticks};

pub(super) struct Continuation {
    id: ResourceLocation,
    registration: DatapackRegistration,
    module: String,
    pointer: String,
}

impl Lower<'_> {
    pub(super) fn suspend(
        &mut self,
        location: (&str, &str, &str, &str),
        ticks: u32,
        fields: &[FieldReference],
        body: &[Operation],
    ) -> Result<Vec<Cmd>, Vec<Diagnostic>> {
        let (owner, path, module, pointer) = location;
        self.continuation_count += 1;
        if self.continuation_count > 64 {
            return Err(vec![Diagnostic::error(
                "SAND_PROGRAM_LIMIT",
                module,
                pointer,
                "a program supports at most 64 distinct wait sites",
            )]);
        }

        let continuation = helper(
            &self.namespace,
            &self.helper_keys[owner],
            path,
            "continuation",
        );
        let resume = helper(&self.namespace, &self.helper_keys[owner], path, "resume");
        let id: ResourceLocation = helper(&self.namespace, &self.helper_keys[owner], path, "wait")
            .parse()
            .expect("valid wait identity");
        let lines = self.body(
            body,
            owner,
            &format!("{path}/wait"),
            module,
            &format!("{pointer}/action/body"),
        )?;
        self.function(&continuation, lines, module, pointer)?;

        let storage = frame_storage(&self.namespace);
        let append_payload =
            format!("data modify storage {storage} stack append from entity @s data.sand_wait");
        let discard_payload = format!("data remove storage {storage} stack[-1]");
        let push = self.frame_commands(fields, true);
        let pop = self.frame_commands(fields, false);
        // The marker appends its payload. Preserve the live slots in a frame
        // under a transient copy of the payload so a synchronous call from the
        // continuation can use the same stack and slots safely.
        let mut resume_body = push.iter().map(Cmd::render).collect::<Vec<_>>();
        resume_body.push(format!(
            "data modify storage {storage} stack append from storage {storage} stack[-2]"
        ));
        resume_body.extend(pop.iter().map(Cmd::render));
        resume_body.push(Cmd::Function(continuation).render());
        resume_body.extend(pop.iter().map(Cmd::render));
        self.function(&resume, resume_body, module, pointer)?;

        let store =
            format!("data modify entity @s data.sand_wait set from storage {storage} stack[-1]");
        let discard_tick = discard_payload.clone();
        let duration = Ticks::new(ticks);
        let lifetime = Lifetime::<AnyEntity>::new(id.clone(), duration)
            .at_invocation_position()
            .every(duration)
            .on_start(move |_| Cmd::Raw(store.clone()))
            .on_tick(move |ctx| {
                let mut actions = crate::ir::Actions(vec![
                    Cmd::Raw("execute unless data entity @s data.sand_wait run return 0".into()),
                    Cmd::Raw(append_payload.clone()),
                ]);
                actions.extend([ctx.owner_at_instance(Cmd::Function(resume.clone()))]);
                actions.0.push(Cmd::Raw(discard_tick.clone()));
                actions
            });
        let mut commands = push;
        commands.extend(lifetime.start().0);
        commands.push(Cmd::Raw(discard_payload));
        self.continuations.push(Continuation {
            id,
            registration: lifetime.into_datapack(),
            module: module.into(),
            pointer: pointer.into(),
        });
        Ok(commands)
    }

    pub(super) fn register_continuations(
        &mut self,
        context: &ExportCtx<'_>,
        load: &mut Vec<(String, usize, String)>,
        tick: &mut Vec<(String, usize, String)>,
    ) -> Result<(), Vec<Diagnostic>> {
        let mut cancellation = Vec::new();
        for continuation in std::mem::take(&mut self.continuations) {
            use crate::gameplay::identity;
            let Continuation {
                id,
                registration,
                module,
                pointer,
            } = continuation;
            let (components, lifecycle, tags) = registration.into_parts();
            debug_assert!(tags.is_empty());
            for component in components {
                let record = records::component_to_record(component.as_ref(), Some(context))
                    .map_err(|error| {
                        vec![Diagnostic::error(
                            "SAND_PROGRAM_COMMAND",
                            &module,
                            &pointer,
                            error.to_string(),
                        )]
                    })?;
                self.runtime.own_function(
                    format!("{}:{}", record.namespace, record.path),
                    module.clone(),
                );
                self.records.push(record);
            }
            for (index, action) in lifecycle.into_iter().enumerate() {
                let output = match action.phase() {
                    LifecyclePhase::Load => &mut *load,
                    LifecyclePhase::Tick => &mut *tick,
                };
                // This key is a private lifecycle owner, outside valid resource IDs.
                output.push((format!("wait {id}"), index, action.command().into()));
            }
            cancellation.extend(
                identity::call(&id, "cancel")
                    .0
                    .into_iter()
                    .map(|command| command.render()),
            );
        }
        if self.cancel_waits {
            let id = self.cancellation_function();
            self.function(&id, cancellation, "", "")?;
        }
        Ok(())
    }

    pub(super) fn cancellation_function(&self) -> String {
        format!("{}:__sand_program/cancel_waits", self.namespace)
    }
}
