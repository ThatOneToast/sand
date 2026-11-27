//! Tick bounded server-owned continuation queues without an entity executor.
use super::*;
use crate::state::registry::AutomaticLifecycle;

pub(super) struct ServerWait {
    queue: String,
    drain: String,
    group: Option<String>,
}

impl Lower<'_> {
    fn server_wait_objective(&self) -> String {
        sand_commands::ObjectiveName::logical(format!("{}:sand.server_waits", self.namespace))
            .to_string()
    }

    pub(super) fn suspend_server(
        &mut self,
        location: (&str, &str, &str, &str),
        continuation: &str,
        resume: &str,
        ticks: &ScoreOperand,
        push: Vec<Cmd>,
        group: Option<&str>,
    ) -> Result<Vec<Cmd>, Vec<Diagnostic>> {
        let (owner, path, module, pointer) = location;
        let key = &self.helper_keys[owner];
        let admit = helper(&self.namespace, key, path, "server_wait_admit");
        let drain = helper(&self.namespace, key, path, "server_wait_drain");
        let deliver = helper(&self.namespace, key, path, "server_wait_deliver");
        let frames = frame_storage(&self.namespace);
        let queue = format!("{}:__sand_server_waits sites.\"{admit}\"", self.namespace);
        let objective = self.server_wait_objective();
        let mut commands = Vec::new();
        if let ScoreOperand::Field(score) = ticks {
            let holder = &self.holders[&score.state.to_string()];
            let score = state_objective(&score.state.to_string(), &score.field);
            commands.push(Cmd::Raw(format!(
                "execute unless score {holder} {score} matches 1.. run return run function {continuation}"
            )));
        }
        commands.extend(push);
        commands.push(Cmd::Function(admit.clone()));
        commands.push(Cmd::Raw(format!("data remove storage {frames} stack[-1]")));

        let mut enqueue = vec![
            format!(
                "execute store result score #pending {objective} run data get storage {queue}.pending"
            ),
            format!(
                "execute store result score #draining {objective} run data get storage {queue}.draining"
            ),
            format!("scoreboard players operation #pending {objective} += #draining {objective}"),
            format!("execute if score #pending {objective} matches 128.. run return 0"),
        ];
        let initial_ticks = match ticks {
            ScoreOperand::Value(ticks) => *ticks,
            ScoreOperand::Field(_) => 1,
        };
        enqueue.push(format!(
            "data modify storage {queue}.pending append value {{ticks:{initial_ticks}}}"
        ));
        if let ScoreOperand::Field(score) = ticks {
            let holder = &self.holders[&score.state.to_string()];
            let score = state_objective(&score.state.to_string(), &score.field);
            enqueue.push(format!(
                "execute store result storage {queue}.pending[-1].ticks int 1 run scoreboard players get {holder} {score}"
            ));
        }
        enqueue.push(format!(
            "data modify storage {queue}.pending[-1].frame set from storage {frames} stack[-1]"
        ));
        self.function(&admit, enqueue, module, pointer)?;

        self.function(&deliver, vec![
            format!("data modify storage {frames} stack append from storage {queue}.draining[0].frame"),
            format!("data remove storage {queue}.draining[0]"),
            format!("function {resume}"),
            format!("data remove storage {frames} stack[-1]"),
        ], module, pointer)?;
        // Commit the dequeue before calling the continuation. It may cancel
        // this queue or enqueue another invocation at the same site.
        self.function(&drain, vec![
            format!("execute unless data storage {queue}.draining[0] run return 0"),
            format!("execute store result score #remaining {objective} run data get storage {queue}.draining[0].ticks"),
            format!("scoreboard players remove #remaining {objective} 1"),
            format!("execute if score #remaining {objective} matches 1.. store result storage {queue}.draining[0].ticks int 1 run scoreboard players get #remaining {objective}"),
            format!("execute if score #remaining {objective} matches 1.. run data modify storage {queue}.pending append from storage {queue}.draining[0]"),
            format!("execute if score #remaining {objective} matches 1.. run data remove storage {queue}.draining[0]"),
            format!("execute if score #remaining {objective} matches ..0 run function {deliver}"),
            format!("execute if data storage {queue}.draining[0] run function {drain}"),
        ], module, pointer)?;
        self.server_waits.push(ServerWait {
            queue,
            drain,
            group: group.map(str::to_owned),
        });
        Ok(commands)
    }

    pub(super) fn register_server_waits(
        &mut self,
        automatic: &mut AutomaticLifecycle,
        cancellation: &mut BTreeMap<(bool, Option<String>), Vec<String>>,
    ) -> Result<(), Vec<Diagnostic>> {
        if self.server_waits.is_empty() {
            return Ok(());
        }
        let objective = self.server_wait_objective();
        if automatic
            .private_objectives
            .insert(
                objective.clone(),
                (self.namespace.clone(), "server waits".into()),
            )
            .is_some()
        {
            return Err(vec![Diagnostic::error(
                "SAND_PROGRAM_COLLISION",
                "",
                "",
                "server wait objective collides with State lifecycle",
            )]);
        }
        automatic
            .load_commands
            .push(format!("scoreboard objectives add {objective} dummy"));
        automatic.global_init_commands.push(format!(
            "data modify storage {}:__sand_server_waits sites set value {{}}",
            self.namespace
        ));
        let mut drain = Vec::new();
        for ServerWait {
            queue,
            drain: function,
            group,
        } in std::mem::take(&mut self.server_waits)
        {
            let clear = format!("data modify storage {queue} set value {{pending:[],draining:[]}}");
            automatic.global_init_commands.push(clear.clone());
            for (key, commands) in cancellation.iter_mut() {
                if key.0 && (key.1.is_none() || key.1 == group) {
                    commands.push(clear.clone());
                }
            }
            // Snapshot all queues before any continuation can schedule new work.
            automatic.global_tick_commands.extend([
                format!("data modify storage {queue}.draining set from storage {queue}.pending"),
                format!("data modify storage {queue}.pending set value []"),
            ]);
            drain.push(format!(
                "execute if data storage {queue}.draining[0] run function {function}"
            ));
        }
        automatic.global_tick_commands.extend(drain);
        Ok(())
    }
}
