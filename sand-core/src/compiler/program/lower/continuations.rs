//! Connect portable function continuations to the canonical owned lifetime runtime.
use super::*;
use crate::compiler::export::ExportCtx;
use crate::entity::kind::AnyEntity;
use crate::gameplay::Lifetime;
use crate::registration::{DatapackRegistration, IntoDatapack};
use sand_components::{ResourceLocation, effect::Ticks};

pub(super) struct Continuation {
    id: ResourceLocation,
    group: Option<String>,
    registration: DatapackRegistration,
    module: String,
    pointer: String,
}

impl Lower<'_> {
    pub(super) fn suspend(
        &mut self,
        location: (&str, &str, &str, &str),
        ticks: &ScoreOperand,
        fields: &[FieldReference],
        body: &[Operation],
        group: Option<&str>,
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
        resume_body.push(Cmd::Function(continuation.clone()).render());
        resume_body.extend(pop.iter().map(Cmd::render));
        self.function(&resume, resume_body, module, pointer)?;

        let store =
            format!("data modify entity @s data.sand_wait set from storage {storage} stack[-1]");
        let discard_tick = discard_payload.clone();
        let (lifetime, invalid) = match ticks {
            ScoreOperand::Value(ticks) => {
                let duration = Ticks::new(*ticks as u32);
                (
                    Lifetime::<AnyEntity>::new(id.clone(), duration).every(duration),
                    None,
                )
            }
            ScoreOperand::Field(score) => {
                let holder = self.holders[&score.state.to_string()].clone();
                let objective = sand_commands::ObjectiveName::logical(format!(
                    "{}.{}",
                    score.state, score.field
                ));
                let invalid = Cmd::Raw(format!(
                    "execute unless score {holder} {objective} matches 1.. run return run function {continuation}"
                ));
                (
                    Lifetime::<AnyEntity>::new(id.clone(), Ticks::new(1))
                        .with_duration_score(holder, objective),
                    Some(invalid),
                )
            }
        };
        let lifetime = lifetime
            .include_dimensions(self.dimensions.iter().cloned())
            .at_invocation_position()
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
        let mut commands = Vec::new();
        commands.extend(invalid);
        commands.extend(push);
        commands.extend(lifetime.start().0);
        commands.push(Cmd::Raw(discard_payload));
        self.continuations.push(Continuation {
            id,
            group: group.map(str::to_owned),
            registration: lifetime.into_datapack(),
            module: module.into(),
            pointer: pointer.into(),
        });
        Ok(commands)
    }

    pub(super) fn register_continuations(
        &mut self,
        context: &ExportCtx<'_>,
        load: &mut Vec<LifecycleCommand>,
        tick: &mut Vec<LifecycleCommand>,
    ) -> Result<(), Vec<Diagnostic>> {
        let mut cancellation = BTreeMap::<_, Vec<_>>::from_iter(
            std::mem::take(&mut self.wait_cancellations)
                .into_iter()
                .map(|group| (group, Vec::new())),
        );
        for continuation in std::mem::take(&mut self.continuations) {
            use crate::gameplay::identity;
            let Continuation {
                id,
                group,
                registration,
                module,
                pointer,
            } = continuation;
            self.register_owned_runtime(
                registration,
                (&format!("wait {id}"), &module, &pointer),
                context,
                load,
                tick,
            )?;
            let commands = identity::call(&id, "cancel");
            if let Some(all) = cancellation.get_mut(&None) {
                all.extend(commands.0.iter().map(Cmd::render));
            }
            if group.is_some()
                && let Some(matching) = cancellation.get_mut(&group)
            {
                matching.extend(commands.0.iter().map(Cmd::render));
            }
        }
        for (group, commands) in cancellation {
            let id = self.cancellation_function(group.as_deref());
            self.function(&id, commands, "", "")?;
        }
        Ok(())
    }

    pub(super) fn cancellation_function(&self, group: Option<&str>) -> String {
        let base = format!("{}:__sand_program/cancel_waits", self.namespace);
        match group {
            Some(group) => format!("{base}/{group}"),
            None => base,
        }
    }
}
