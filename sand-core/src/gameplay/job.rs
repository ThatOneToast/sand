use super::{Lifetime, TaskContext, identity, runtime};
use crate::{
    IntoCommands,
    entity::{EntityKind, PlayerKind, StatCurve},
    ir::Actions,
    registration::{DatapackRegistration, IntoDatapack},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use std::sync::Arc;

/// Resumable indexed work owned by a lifetime. Each instance processes at most
/// `per_tick` steps on each loaded tick, in ascending index order. A step may
/// author arbitrary work: this is a callback-count budget, not a time budget.
/// Expiry and cancellation stop unfinished work without invoking completion.
/// Existing lifetime callbacks and their cadence are preserved; jobs run after
/// lifetime validation and tick callbacks. State belongs on the lifetime.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Job", module = "sand::systems",
summary = "Resumable indexed work with a per-instance tick budget.", context = "Owns a zero-based cursor through Lifetime. Completion runs once after the last step; cancellation or expiry discards unfinished work.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
example = "use sand::systems::Job;",
)]
pub struct Job<K: EntityKind = PlayerKind> {
    pub(super) lifetime: Lifetime<K>,
    count: u32,
    budget: u32,
    step: Arc<dyn Fn(TaskContext<K>, StatCurve) -> Actions>,
    pub(super) ready: Option<super::lifetime::Callback<K>>,
    complete: Option<super::lifetime::Callback<K>>,
}
impl<K: EntityKind> Job<K> {
    /// Create a job with one step per loaded tick.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Job::new", module = "sand::systems", kind = "method",
summary = "Create a job with one step per loaded tick.", context = "Uses Lifetime admission and cleanup. Per-instance cursors resume in ascending order on loaded ticks; no completion callback is run on cancellation or expiry.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(lifetime = "Ownership, deadline, capacity and cleanup.", count = "Positive total number of steps, at most i32::MAX.", step = "Authors one step; index is a zero-based runtime expression in instance context."),
returns = "The configured definition or retained runtime actions.", example = "use sand::command::{Actionbar, Target};\nuse sand::text::Text;\nuse sand::systems::{Job, Lifetime};\nfn demonstrate(lifetime: Lifetime) { let _job = Job::new(lifetime, 20, |ctx, _index| ctx.owner(|_| Actionbar::show(Target::self_(), Text::new(\"Working\")))); }",
)]
    pub fn new<R: IntoCommands>(
        lifetime: Lifetime<K>,
        count: u32,
        step: impl Fn(TaskContext<K>, StatCurve) -> R + 'static,
    ) -> Self {
        Self {
            lifetime,
            count,
            budget: 1,
            step: Arc::new(move |ctx, index| step(ctx, index).into_commands()),
            complete: None,
            ready: None,
        }
    }
    /// Set the maximum steps per instance per loaded tick.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Job::per_tick", module = "sand::systems", kind = "method",
summary = "Set the maximum steps per instance per loaded tick.", context = "Uses Lifetime admission and cleanup. Per-instance cursors resume in ascending order on loaded ticks; no completion callback is run on cancellation or expiry.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(steps = "Callback-count budget in 1..=64."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Job,Lifetime};\nfn demonstrate(job: Job) { let _job = job.per_tick(4); }",
)]
    pub fn per_tick(mut self, steps: u32) -> Self {
        self.budget = steps;
        self
    }
    /// Run once after the last step, before lifetime cleanup.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Job::on_complete", module = "sand::systems", kind = "method",
summary = "Run once after the last step, before lifetime cleanup.", context = "Uses Lifetime admission and cleanup. Per-instance cursors resume in ascending order on loaded ticks; no completion callback is run on cancellation or expiry.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(body = "Instance-context completion callback. May resolve the owner through TaskContext."),
returns = "The configured definition or retained runtime actions.", example = "use sand::command::{Actionbar, Target};\nuse sand::text::Text;\nuse sand::systems::{Job, Lifetime};\nfn demonstrate(job: Job) { let _job = job.on_complete(|ctx| ctx.owner(|_| Actionbar::show(Target::self_(), Text::new(\"Done\")))); }",
)]
    pub fn on_complete<R: IntoCommands>(
        mut self,
        body: impl Fn(TaskContext<K>) -> R + 'static,
    ) -> Self {
        self.complete = Some(Arc::new(move |ctx| body(ctx).into_commands()));
        self
    }
    /// Start an independent job for the current owner.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Job::start", module = "sand::systems", kind = "method",
summary = "Start an independent job for the current owner.", context = "Uses Lifetime admission and cleanup. Per-instance cursors resume in ascending order on loaded ticks; no completion callback is run on cancellation or expiry.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Job,Lifetime};\nfn demonstrate(job: Job) { let _actions = job.start(); }",
)]
    pub fn start(&self) -> Actions {
        self.lifetime.start()
    }
    /// Cancel all instances of this job belonging to the current owner.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Job::cancel", module = "sand::systems", kind = "method",
summary = "Cancel all instances of this job belonging to the current owner.", context = "Uses Lifetime admission and cleanup. Per-instance cursors resume in ascending order on loaded ticks; no completion callback is run on cancellation or expiry.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Job,Lifetime};\nfn demonstrate(job: Job) { let _actions = job.cancel(); }",
)]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> IntoDatapack for Job<K> {
    fn into_datapack(mut self) -> DatapackRegistration {
        runtime::hooks(&mut self.lifetime, "job_init", "job_cleanup", "job_batch");
        let mut registration = self.lifetime.clone().into_datapack();
        let mut parts = vec![
            "job_init",
            "job_cleanup",
            "job_batch",
            "job_step",
            "job_dispatch",
            "job_resume",
            "job_advance",
            "job_body",
            "job_complete",
        ];
        if self.ready.is_some() {
            parts.push("job_ready");
        }
        for part in parts {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.lifetime.id,
                part,
                move || definition.build(part),
            ));
        }
        registration
    }
}
impl<K: EntityKind> Job<K> {
    fn build(&self, part: &str) -> CommandResult<Actions> {
        if self.count == 0 || self.count > i32::MAX as u32 {
            return Err(CommandError::new(
                "job",
                "count",
                "must be in 1..=2147483647",
            ));
        }
        if !(1..=64).contains(&self.budget) {
            return Err(CommandError::new("job", "per_tick", "must be in 1..=64"));
        }
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let cursor = cursor(&id);
        crate::function::request_numeric_objectives([cursor.clone()])?;
        let resource = |part| identity::resource(&id, part).to_string();
        let ctx = || runtime::context(&self.lifetime);
        match part {
            "job_ready" => Ok(self.ready.as_ref().expect("registered gate")(ctx())),
            "job_body" => Ok((self.step)(
                ctx(),
                StatCurve::input_raw(&cursor.to_string()),
            )),
            "job_complete" => Ok(self.complete.as_ref().map(|f| f(ctx())).unwrap_or_default()),
            "job_init" => Ok(format!("scoreboard players set @s {cursor} 0").into_commands()),
            "job_cleanup" => Ok(format!("scoreboard players reset @s {cursor}").into_commands()),
            "job_dispatch" => runtime::resolve(&id, "job_step"),
            "job_resume" => runtime::resolve(&id, "job_advance"),
            "job_batch" => Ok((0..self.budget)
                .map(|_| format!("function {} with entity @s data", resource("job_dispatch")))
                .collect::<Vec<_>>()
                .into_commands()),
            "job_step" => {
                let mut lines = vec![];
                if self.ready.is_some() {
                    lines.push(format!(
                        "execute unless function {} run return 0",
                        resource("job_ready")
                    ));
                }
                lines.extend([
                    format!(
                        "execute unless score @s {cursor} matches 0..{} run return 0",
                        self.count - 1
                    ),
                    format!("function {}", resource("job_body")),
                    format!("function {} with entity @s data", resource("job_resume")),
                ]);
                Ok(lines.into_commands())
            }
            "job_advance" => Ok(vec![
                format!("scoreboard players add @s {cursor} 1"),
                format!(
                    "execute unless score @s {cursor} matches {} run return 0",
                    self.count
                ),
                format!("function {}", resource("job_complete")),
                format!("function {}", resource("stop")),
            ]
            .into_commands()),
            _ => unreachable!(),
        }
    }
}

/// Shared cursor model for indexed jobs and authored timelines.
pub(super) fn cursor(id: &sand_components::ResourceLocation) -> ObjectiveName {
    ObjectiveName::logical(format!("sand job {id} cursor"))
}

impl<K: EntityKind> AsRef<Lifetime<K>> for Job<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_components::effect::Ticks;
    #[test]
    fn invalid_budgets_fail_before_reserving_objectives() {
        for (count, budget, field) in [
            (0, 1, "count"),
            (u32::MAX, 1, "count"),
            (1, 0, "per_tick"),
            (1, 65, "per_tick"),
        ] {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let job = Job::new(
                Lifetime::<PlayerKind>::new("test:job".parse().unwrap(), Ticks::new(100)),
                count,
                |_, _| Actions::default(),
            )
            .per_tick(budget);
            assert_eq!(job.build("job_init").unwrap_err().field, field);
            assert!(crate::function::take_numeric_objectives().is_empty());
        }
    }
}
