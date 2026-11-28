//! Whole-program validation before any output allocation or publication.
use super::{diagnostic::Diagnostic, limits, model::*};
use sand_commands::ClickEvent;
use sand_components::ResourceLocation;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    program: &Program,
) -> Result<crate::version::VersionProfile, Vec<Diagnostic>> {
    let mut errors = Vec::new();
    if program.format != "sand.program" {
        errors.push(Diagnostic::error(
            "SAND_PROGRAM_VERSION",
            "",
            "/format",
            "expected sand.program format",
        ));
    }
    if program.format_version != 1 {
        errors.push(Diagnostic::error(
            "SAND_PROGRAM_VERSION",
            "",
            "/format_version",
            "expected sand.program protocol revision 1",
        ));
    }
    let target =
        crate::version::MinecraftVersion::parse(&program.target.minecraft).and_then(|version| {
            if version.is_latest() {
                return Err(crate::version::VersionError::ParseError(
                    program.target.minecraft.clone(),
                ));
            }
            crate::version::VersionProfile::resolve_strict(&version)
        });
    let target = match target {
        Ok(target) => Some(target),
        Err(error) => {
            errors.push(Diagnostic::error(
                "SAND_PROGRAM_TARGET",
                "",
                "/target/minecraft",
                error,
            ));
            None
        }
    };
    if ResourceLocation::new(&program.pack.namespace, "pack").is_err()
        || !safe_path(&program.pack.namespace)
    {
        errors.push(Diagnostic::error(
            "SAND_PROGRAM_ID",
            "",
            "/pack/namespace",
            "invalid pack namespace",
        ));
    }
    for (index, capability) in program.requires.iter().enumerate() {
        if !super::CAPABILITIES.contains(&capability.as_str()) {
            errors.push(Diagnostic::error(
                "SAND_PROGRAM_CAPABILITY",
                "",
                &format!("/requires/{index}"),
                format!("unsupported capability {capability}"),
            ));
        }
    }
    if program.modules.len() > limits::MODULES {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            "",
            "/modules",
            "more than 64 modules",
        )]);
    }
    let declarations = program
        .modules
        .iter()
        .try_fold(0usize, |count, module| {
            count
                .checked_add(module.states.len())?
                .checked_add(module.functions.len())?
                .checked_add(module.tags.len())?
                .checked_add(module.flows.len())?
                .checked_add(
                    module
                        .flows
                        .iter()
                        .map(|flow| flow.transitions.len().saturating_add(flow.hooks.len()))
                        .sum::<usize>(),
                )?
                .checked_add(
                    module
                        .states
                        .iter()
                        .map(|state| state.fields.len().saturating_add(state.data_fields.len()))
                        .sum::<usize>(),
                )
        })
        .unwrap_or(usize::MAX);
    if declarations > limits::DECLARATIONS {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            "",
            "/modules",
            "declaration limit exceeded",
        )]);
    }
    let mut modules = BTreeSet::new();
    let mut flow_ids = BTreeSet::new();
    let mut states = BTreeMap::new();
    let mut functions = BTreeMap::new();
    for module in &program.modules {
        let owner = module.id.to_string();
        if !valid_id(&module.id) || !modules.insert(owner.clone()) {
            errors.push(Diagnostic::error(
                "SAND_PROGRAM_ID",
                &owner,
                "/id",
                "invalid or duplicate module identity",
            ));
        }
        for (index, state) in module.states.iter().enumerate() {
            let ptr = format!("/states/{index}");
            if !valid_id(&state.id) || states.insert(state.id.to_string(), state).is_some() {
                errors.push(Diagnostic::error(
                    "SAND_PROGRAM_ID",
                    &owner,
                    &format!("{ptr}/id"),
                    "invalid or duplicate State identity",
                ));
            }
            if !matches!(state.scope.as_str(), "player" | "entity" | "global")
                || state.migrations.len() > 32
                || state.revision as usize != state.migrations.len() + 1
            {
                errors.push(Diagnostic::error(
                    "SAND_PROGRAM_STATE",
                    &owner,
                    &ptr,
                    "State requires player, entity, or global scope and one consecutive migration per revision after 1 (up to 32 migrations)",
                ));
            }
            let mut fields = BTreeSet::new();
            for (index, field) in state.fields.iter().enumerate() {
                if field.timer && field.bounds.is_some() {
                    errors.push(Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &module.id.to_string(),
                        &format!("{ptr}/fields/{index}/bounds"),
                        "timer fields count down on their own and cannot have bounds",
                    ));
                }
                if field.bounds.is_some_and(|(min, max)| min > max) {
                    errors.push(Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &owner,
                        &format!("{ptr}/fields/{index}/bounds"),
                        "State field minimum must not exceed its maximum",
                    ));
                }
                if matches!(
                    field.name.as_str(),
                    "presence" | "suppressed" | "numeric_scratch"
                ) || field.name.is_empty()
                    || !field
                        .name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    || !fields.insert(&field.name)
                {
                    errors.push(Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &owner,
                        &format!("{ptr}/fields/{index}"),
                        "fields require unique simple names and cannot claim State ownership objectives",
                    ));
                }
            }
            for (index, field) in state.data_fields.iter().enumerate() {
                let field_ptr = format!("{ptr}/data_fields/{index}");
                if field.name.is_empty()
                    || matches!(
                        field.name.as_str(),
                        "presence" | "suppressed" | "numeric_scratch"
                    )
                    || !field
                        .name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    || !fields.insert(&field.name)
                {
                    errors.push(Diagnostic::error("SAND_PROGRAM_STATE", &owner, &field_ptr,
                        "storage fields require unique simple names and share the score field namespace"));
                }
                let profile = sand_commands::CommandProfile::new(&program.target.minecraft, false);
                let command = sand_commands::DataTarget::storage("sand:validate")
                    .path("value")
                    .set(field.default.clone());
                if serde_json::to_value(&field.default).is_err() {
                    errors.push(Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &owner,
                        &field_ptr,
                        "portable storage defaults require typed NBT; raw SNBT is unsupported",
                    ));
                } else if let Err(error) = command.try_render(&profile) {
                    errors.push(Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &owner,
                        &field_ptr,
                        error,
                    ));
                }
            }
        }
        for (index, function) in module.functions.iter().enumerate() {
            if !valid_id(&function.id)
                || function.id.path().starts_with("__sand_")
                || functions
                    .insert(function.id.to_string(), function.context)
                    .is_some()
            {
                errors.push(Diagnostic::error(
                    "SAND_PROGRAM_ID",
                    &owner,
                    &format!("/functions/{index}/id"),
                    "invalid, duplicate, or compiler-reserved function identity",
                ));
            }
        }
    }
    let field_bounds: BTreeMap<_, _> = program
        .modules
        .iter()
        .flat_map(|module| &module.states)
        .flat_map(|state| {
            state
                .fields
                .iter()
                .map(|field| (format!("{}.{}", state.id, field.name), field.bounds))
        })
        .collect();
    let mut validator = Validator {
        dialogs_supported: target.as_ref().is_some_and(|profile| {
            profile
                .caps()
                .supports(sand_version::ComponentFeature::Dialogs)
        }),
        menus: program
            .modules
            .iter()
            .flat_map(|module| &module.menus)
            .map(|menu| menu.id.to_string())
            .collect(),
        progressions: program
            .modules
            .iter()
            .flat_map(|module| &module.progressions)
            .map(|progression| progression.id.to_string())
            .collect(),
        combats: program
            .modules
            .iter()
            .flat_map(|module| &module.combats)
            .map(|combat| combat.id.to_string())
            .collect(),
        abilities: program
            .modules
            .iter()
            .flat_map(|module| &module.abilities)
            .map(|ability| ability.id.to_string())
            .collect(),
        scenes: program
            .modules
            .iter()
            .flat_map(|module| &module.scenes)
            .map(|scene| scene.id.to_string())
            .collect(),
        jobs: program
            .modules
            .iter()
            .flat_map(|module| &module.jobs)
            .map(|job| job.id.to_string())
            .collect(),
        sessions: program
            .modules
            .iter()
            .flat_map(|module| &module.sessions)
            .map(|session| session.id.to_string())
            .collect(),
        projectiles: program
            .modules
            .iter()
            .flat_map(|module| &module.projectiles)
            .map(|projectile| projectile.id.to_string())
            .collect(),
        huds: program
            .modules
            .iter()
            .flat_map(|module| &module.huds)
            .map(|hud| hud.id.to_string())
            .collect(),
        transfers: program
            .modules
            .iter()
            .flat_map(|module| &module.inventory_transfers)
            .map(|transfer| transfer.id.to_string())
            .collect(),
        advancements: program
            .modules
            .iter()
            .flat_map(|module| &module.advancements)
            .map(|item| (item.id.to_string(), item.criteria.keys().cloned().collect()))
            .collect(),
        target: &program.target.minecraft,
        boss_bars: program
            .modules
            .iter()
            .flat_map(|m| m.boss_bars.iter().map(|b| b.id.to_string()))
            .collect(),
        loot_tables: program
            .modules
            .iter()
            .flat_map(|m| m.loot_tables.iter().map(|t| t.id.to_string()))
            .collect(),
        custom_items: program
            .modules
            .iter()
            .flat_map(|m| m.items.iter().map(|i| i.id.to_string()))
            .collect(),
        states: states
            .into_iter()
            .map(|(id, state)| {
                (
                    id,
                    (
                        state.scope.as_str(),
                        state
                            .fields
                            .iter()
                            .map(|field| field.name.as_str())
                            .collect(),
                    ),
                )
            })
            .collect(),
        data_fields: program
            .modules
            .iter()
            .flat_map(|module| &module.states)
            .map(|state| {
                (
                    state.id.to_string(),
                    state
                        .data_fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect(),
                )
            })
            .collect(),
        functions,
        identity_unsafe: program
            .modules
            .iter()
            .flat_map(|m| &m.states)
            .flat_map(|state| {
                state
                    .fields
                    .iter()
                    .filter(|field| field.timer || field.bounds.is_some())
                    .map(|field| format!("{}.{}", state.id, field.name))
            })
            .collect(),
        operations: 0,
        flow_bytes: 0,
        raw: program.requires.iter().any(|v| v == "raw_commands"),
        errors: &mut errors,
    };
    let migration_functions: BTreeMap<_, _> = program
        .modules
        .iter()
        .flat_map(|module| &module.functions)
        .map(|function| (function.id.to_string(), function))
        .collect();
    for module in &program.modules {
        let owner = module.id.to_string();
        for (index, state) in module.states.iter().enumerate() {
            let context = match state.scope.as_str() {
                "player" => ExecutionContext::Player,
                "entity" => ExecutionContext::Entity,
                _ => ExecutionContext::Server,
            };
            for (step, id) in state.migrations.iter().enumerate() {
                let ptr = format!("/states/{index}/migrations/{step}");
                validator.call(
                    &FunctionReference::Internal { id: id.clone() },
                    context,
                    &owner,
                    &ptr,
                );
                if migration_functions
                    .get(&id.to_string())
                    .is_some_and(|function| !is_state_migration(&function.body))
                {
                    validator.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &owner,
                        &ptr,
                        "a State migration may only transform values synchronously without calling other functions or changing executor",
                    ));
                }
            }
        }
        for (index, flow) in module.flows.iter().enumerate() {
            let ptr = format!("/flows/{index}");
            if !validator.reserve_flow_expansion(flow, &owner, &ptr) {
                continue;
            }
            if !valid_id(&flow.id) || !flow_ids.insert(flow.id.to_string()) {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_ID",
                    &owner,
                    &format!("{ptr}/id"),
                    "invalid or duplicate flow identity",
                ));
            }
            if validator
                .states
                .get(&flow.state.state.to_string())
                .is_some_and(|(scope, _)| *scope == "entity")
            {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_FLOW",
                    &owner,
                    &format!("{ptr}/state"),
                    "flows run over player or global State, not entity State",
                ));
                continue;
            }
            let context = if validator
                .states
                .get(&flow.state.state.to_string())
                .is_some_and(|(scope, _)| *scope == "player")
            {
                ExecutionContext::Player
            } else {
                ExecutionContext::Server
            };
            validator.score(&flow.state, context, &owner, &format!("{ptr}/state"));
            let bounds = field_bounds
                .get(&format!("{}.{}", flow.state.state, flow.state.field))
                .copied()
                .flatten();
            if flow.transitions.is_empty() && flow.hooks.is_empty() {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_FLOW",
                    &owner,
                    &ptr,
                    "flow requires a transition or hook",
                ));
            }
            for (index, transition) in flow.transitions.iter().enumerate() {
                let ptr = format!("{ptr}/transitions/{index}");
                if transition.from == transition.to
                    || bounds.is_some_and(|(min, max)| !(min..=max).contains(&transition.to))
                {
                    validator.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_FLOW",
                        &owner,
                        &ptr,
                        "destination must differ from source and satisfy State bounds",
                    ));
                }
                if let Some(condition) = &transition.when {
                    validator.comparison(condition, context, &owner, &format!("{ptr}/when"));
                }
            }
            for (index, hook) in flow.hooks.iter().enumerate() {
                let ptr = format!("{ptr}/hooks/{index}");
                validator.call(&hook.function, context, &owner, &format!("{ptr}/function"));
                if hook.cadence == 0
                    || hook.cadence > i32::MAX as u32
                    || (hook.phase != FlowHookPhase::Tick && hook.cadence != 1)
                {
                    validator.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_FLOW",
                        &owner,
                        &format!("{ptr}/cadence"),
                        "tick cadence must be 1..=i32::MAX; enter/exit cadence must be 1",
                    ));
                }
            }
        }
        for (index, function) in module.functions.iter().enumerate() {
            validator.body(
                &function.body,
                function.context,
                &owner,
                &format!("/functions/{index}/body"),
                0,
            );
        }
        validator.body(&module.load, ExecutionContext::Server, &owner, "/load", 0);
        validator.body(&module.tick, ExecutionContext::Server, &owner, "/tick", 0);
        for (index, tag) in module.tags.iter().enumerate() {
            let ptr = format!("/tags/{index}");
            if !valid_id(&tag.tag) {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_ID",
                    &owner,
                    &ptr,
                    "invalid tag identity",
                ));
            }
            let context = if matches!(
                tag.tag.to_string().as_str(),
                "minecraft:load" | "minecraft:tick"
            ) {
                ExecutionContext::Server
            } else {
                // Custom tags inherit their caller's context. Player members
                // retain their explicit initialization at function entry.
                ExecutionContext::Player
            };
            validator.call(&tag.function, context, &owner, &format!("{ptr}/function"));
        }
        for (index, menu) in module.menus.iter().enumerate() {
            let ptr = format!("/menus/{index}");
            let context = ExecutionContext::Player;
            if let Some(score) = &menu.while_open {
                validator.score(score, context, &owner, &format!("{ptr}/while_open"));
            }
            for (choices, path) in menu.choice_lists() {
                for (choice_index, choice) in choices.iter().enumerate() {
                    let ptr = format!("{ptr}/{path}/{choice_index}");
                    if let Some(action) = &choice.action {
                        validator.call(action, context, &owner, &format!("{ptr}/action"));
                    }
                    if let Some(score) = &choice.when {
                        validator.score(score, context, &owner, &format!("{ptr}/when"));
                    }
                }
            }
        }
        for (index, progression) in module.progressions.iter().enumerate() {
            let ptr = format!("/progressions/{index}");
            let context = ExecutionContext::Player;
            let score = &progression.progress;
            validator.score(score, context, &owner, &format!("{ptr}/progress"));
            if validator
                .states
                .get(&score.state.to_string())
                .is_none_or(|(scope, _)| *scope != "player")
            {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_STATE",
                    &owner,
                    &format!("{ptr}/progress"),
                    "progression requires a player State number",
                ));
            }
            for (index, objective) in progression.objectives.iter().enumerate() {
                if let Some(score) = &objective.when {
                    validator.score(
                        score,
                        context,
                        &owner,
                        &format!("{ptr}/objectives/{index}/when"),
                    );
                }
                if let Some(reward) = &objective.reward {
                    validator.call(
                        reward,
                        context,
                        &owner,
                        &format!("{ptr}/objectives/{index}/reward"),
                    );
                }
            }
            for (index, branch) in progression.branches.iter().enumerate() {
                validator.score(
                    &branch.when,
                    context,
                    &owner,
                    &format!("{ptr}/branches/{index}/when"),
                );
            }
        }
        for (index, combat) in module.combats.iter().enumerate() {
            let ptr = format!("/combats/{index}");
            let context = ExecutionContext::Player;
            if let Some(critical) = &combat.critical {
                validator.score(
                    &critical.when,
                    context,
                    &owner,
                    &format!("{ptr}/critical/when"),
                );
            }
            for (field, callback) in [
                ("hit", &combat.hit),
                ("defeat", &combat.defeat),
                ("healed", &combat.healed),
            ] {
                if let Some(callback) = callback {
                    validator.call(callback, context, &owner, &format!("{ptr}/{field}"));
                }
            }
        }
        for (index, ability) in module.abilities.iter().enumerate() {
            let ptr = format!("/abilities/{index}");
            let context = ExecutionContext::Player;
            for (field, score) in [
                (
                    "cost/balance",
                    ability.cost.as_ref().map(|cost| &cost.balance),
                ),
                (
                    "cooldown/timer",
                    ability.cooldown.as_ref().map(|cooldown| &cooldown.timer),
                ),
                ("when", ability.when.as_ref()),
            ] {
                if let Some(score) = score {
                    let location = format!("{ptr}/{field}");
                    validator.score(score, context, &owner, &location);
                    if field != "when"
                        && validator
                            .states
                            .get(&score.state.to_string())
                            .is_none_or(|(scope, _)| *scope != "player")
                    {
                        validator.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_STATE",
                            &owner,
                            &location,
                            "ability requires a player State field",
                        ));
                    }
                }
            }
            if let Some(start) = &ability.start {
                validator.call(start, context, &owner, &format!("{ptr}/start"));
            }
            if let Some(projectile) = &ability.projectile
                && !validator.projectiles.contains(&projectile.to_string())
            {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_REFERENCE",
                    &owner,
                    &format!("{ptr}/projectile"),
                    format!("unknown projectile `{projectile}`"),
                ));
            }
        }
        for (index, scene) in module.scenes.iter().enumerate() {
            for (part_index, part) in scene.parts.iter().enumerate() {
                let ptr = format!("/scenes/{index}/parts/{part_index}/content");
                match &part.content {
                    SceneContent::Block { block } => {
                        validator.resource(
                            &block.to_string(),
                            &owner,
                            &format!("{ptr}/block"),
                            "block",
                        );
                    }
                    SceneContent::Item { item } => {
                        validator.resource(item, &owner, &format!("{ptr}/item"), "item");
                    }
                    SceneContent::Text { .. } => {}
                }
            }
        }
        for (index, job) in module.jobs.iter().enumerate() {
            let ptr = format!("/jobs/{index}");
            validator.score(
                &job.index,
                ExecutionContext::Server,
                &owner,
                &format!("{ptr}/index"),
            );
            for (field, callback) in std::iter::once(("step", &job.step))
                .chain(job.complete.as_ref().map(|callback| ("complete", callback)))
            {
                validator.call(
                    callback,
                    ExecutionContext::Player,
                    &owner,
                    &format!("{ptr}/{field}"),
                );
            }
        }
        for (index, session) in module.sessions.iter().enumerate() {
            if !validator
                .progressions
                .contains(&session.progression.to_string())
            {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_REFERENCE",
                    &owner,
                    &format!("/sessions/{index}/progression"),
                    format!("unknown progression `{}`", session.progression),
                ));
            }
        }
        for (index, projectile) in module.projectiles.iter().enumerate() {
            if let Some(hit) = &projectile.hit {
                validator.call(
                    hit,
                    ExecutionContext::Player,
                    &owner,
                    &format!("/projectiles/{index}/hit"),
                );
            }
        }
        for (index, hud) in module.huds.iter().enumerate() {
            if let Some(HudBossBar {
                value: ScoreOperand::Field(score),
                ..
            }) = &hud.bossbar
            {
                validator.score(
                    score,
                    ExecutionContext::Player,
                    &owner,
                    &format!("/huds/{index}/bossbar/value"),
                );
            }
        }
        for (index, transfer) in module.inventory_transfers.iter().enumerate() {
            for (field, callback) in [
                ("success", &transfer.success),
                ("rejected", &transfer.rejected),
            ] {
                if let Some(callback) = callback {
                    validator.call(
                        callback,
                        ExecutionContext::Player,
                        &owner,
                        &format!("/inventory_transfers/{index}/{field}"),
                    );
                }
            }
        }
        for (index, hook) in module.events.iter().enumerate() {
            let ptr = format!("/events/{index}");
            if let Some((id, kind)) = hook.event.filter() {
                validator.resource(id, &owner, &format!("{ptr}/event"), kind);
            }
            if let PlayerEvent::AdvancementCompleted { advancement } = &hook.event
                && !validator
                    .advancements
                    .contains_key(&advancement.to_string())
            {
                validator.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_RESOURCE",
                    &owner,
                    &format!("{ptr}/event/advancement"),
                    format!("advancement completion requires an authored advancement, found `{advancement}`"),
                ));
            }
            if let Some(predicate) = &hook.when {
                validator.resource(
                    &predicate.to_string(),
                    &owner,
                    &format!("{ptr}/when"),
                    "predicate",
                );
            }
            // Handlers run as the observed player.
            validator.call(
                &hook.function,
                ExecutionContext::Player,
                &owner,
                &format!("{ptr}/function"),
            );
        }
    }
    let custom_items: BTreeSet<String> = program
        .modules
        .iter()
        .flat_map(|m| m.items.iter().map(|i| i.id.to_string()))
        .collect();
    let mut resource_ids = BTreeSet::new();
    for module in &program.modules {
        let owner = module.id.to_string();
        let mut push = |ptr: String, message: String| {
            errors.push(Diagnostic::error(
                "SAND_PROGRAM_RESOURCE",
                &owner,
                &ptr,
                message,
            ));
        };
        let syntax_id = |id: &str| {
            id.strip_prefix('#').is_none()
                && id.parse::<ResourceLocation>().is_ok()
                && id.contains(':')
        };
        let available = |id: &str, registry: &str| {
            !id.starts_with("minecraft:")
                || sand_version::registry::contains(&program.target.minecraft, registry, id)
                    != Some(false)
        };
        let item_id = |id: &str| syntax_id(id) && available(id, "minecraft:item");
        let block_id = |id: &str| syntax_id(id) && available(id, "minecraft:block");
        let ingredient = |id: &str| {
            id.strip_prefix('#')
                .unwrap_or(id)
                .parse::<ResourceLocation>()
                .is_ok()
                && id.contains(':')
        };
        for (index, item) in module.items.iter().enumerate() {
            let ptr = format!("/items/{index}");
            if !resource_ids.insert(format!("item {}", item.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate custom item `{}`", item.id),
                );
            }
            if item.id.namespace() == "minecraft" {
                push(
                    format!("{ptr}/id"),
                    "custom items must not use the `minecraft` namespace".into(),
                );
            }
            if !item_id(&item.base) || custom_items.contains(&item.base) {
                push(
                    format!("{ptr}/base"),
                    format!("`{}` is not a vanilla base item", item.base),
                );
            }
            let item_text =
                std::iter::once(item.name.as_slice()).chain(item.lore.iter().map(Vec::as_slice));
            for parts in item_text {
                if let Some(error) = static_text_error(parts, None) {
                    push(ptr.clone(), error);
                }
            }
            if let Some(book) = &item.book {
                if item.base != "minecraft:written_book" {
                    push(
                        format!("{ptr}/book"),
                        "book content needs a minecraft:written_book base item".into(),
                    );
                }
                for (index, page) in book.pages.iter().enumerate() {
                    if let Some(error) = static_text_error(page, Some(book.pages.len())) {
                        push(format!("{ptr}/book/pages/{index}"), error);
                    }
                }
            }
            if item.max_stack.is_some_and(|n| !(1..=99).contains(&n)) {
                push(
                    format!("{ptr}/max_stack"),
                    "max stack must be 1..=99".into(),
                );
            }
            for (enchantment, level) in &item.enchantments {
                if !available(enchantment, "minecraft:enchantment") {
                    push(
                        format!("{ptr}/enchantments"),
                        format!(
                            "no enchantment `{enchantment}` in Minecraft {}",
                            program.target.minecraft
                        ),
                    );
                } else if !syntax_id(enchantment) || !(1..=255).contains(level) {
                    push(
                        format!("{ptr}/enchantments"),
                        format!("enchantment `{enchantment}` needs an id and a level of 1..=255"),
                    );
                }
            }
        }
        for (index, recipe) in module.recipes.iter().enumerate() {
            let ptr = format!("/recipes/{index}");
            if !resource_ids.insert(format!("recipe {}", recipe.id())) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate recipe `{}`", recipe.id()),
                );
            }
            let mut inputs: Vec<(String, &String)> = Vec::new();
            let result = match recipe {
                Recipe::Shaped {
                    pattern,
                    key,
                    result,
                    ..
                } => {
                    let width = pattern.first().map(|r| r.chars().count()).unwrap_or(0);
                    if pattern.is_empty()
                        || pattern.len() > 3
                        || width == 0
                        || width > 3
                        || pattern.iter().any(|r| r.chars().count() != width)
                    {
                        push(format!("{ptr}/pattern"), "a shaped pattern has one to three rows of equal width, at most three wide".into());
                    }
                    let used: BTreeSet<char> = pattern
                        .iter()
                        .flat_map(|r| r.chars())
                        .filter(|c| *c != ' ')
                        .collect();
                    for c in &used {
                        if !key.contains_key(&c.to_string()) {
                            push(
                                format!("{ptr}/key"),
                                format!("pattern character `{c}` has no key"),
                            );
                        }
                    }
                    for (symbol, item) in key {
                        let mut chars = symbol.chars();
                        let single = chars.next().filter(|c| *c != ' ' && chars.next().is_none());
                        match single {
                            None => push(
                                format!("{ptr}/key"),
                                format!("key `{symbol}` must be one character other than space"),
                            ),
                            Some(c) if !used.contains(&c) => push(
                                format!("{ptr}/key"),
                                format!("key `{symbol}` is not used in the pattern"),
                            ),
                            Some(_) => {}
                        }
                        inputs.push((format!("{ptr}/key/{symbol}"), item));
                    }
                    Some(result)
                }
                Recipe::Shapeless {
                    ingredients,
                    result,
                    ..
                } => {
                    if ingredients.is_empty() || ingredients.len() > 9 {
                        push(
                            format!("{ptr}/ingredients"),
                            "a shapeless recipe has one to nine ingredients".into(),
                        );
                    }
                    inputs.extend(
                        ingredients
                            .iter()
                            .enumerate()
                            .map(|(i, item)| (format!("{ptr}/ingredients/{i}"), item)),
                    );
                    Some(result)
                }
                Recipe::Cooking {
                    ingredient,
                    result,
                    experience,
                    time,
                    ..
                } => {
                    if !(experience.is_finite() && *experience >= 0.0) || *time == Some(0) {
                        push(
                            ptr.clone(),
                            "cooking experience must be non-negative and time positive".into(),
                        );
                    }
                    inputs.push((format!("{ptr}/ingredient"), ingredient));
                    Some(result)
                }
                Recipe::Stonecutting {
                    ingredient, result, ..
                } => {
                    inputs.push((format!("{ptr}/ingredient"), ingredient));
                    Some(result)
                }
                Recipe::Smithing {
                    template,
                    base,
                    addition,
                    result,
                    ..
                } => {
                    inputs.push((format!("{ptr}/template"), template));
                    inputs.push((format!("{ptr}/base"), base));
                    inputs.push((format!("{ptr}/addition"), addition));
                    Some(result)
                }
                Recipe::SmithingTrim {
                    template,
                    base,
                    addition,
                    pattern,
                    ..
                } => {
                    inputs.push((format!("{ptr}/template"), template));
                    inputs.push((format!("{ptr}/base"), base));
                    inputs.push((format!("{ptr}/addition"), addition));
                    if !available(&pattern.to_string(), "minecraft:trim_pattern") {
                        push(
                            format!("{ptr}/pattern"),
                            format!(
                                "no trim pattern `{pattern}` in Minecraft {}",
                                program.target.minecraft
                            ),
                        );
                    }
                    None
                }
            };
            for (input_ptr, item) in inputs {
                if custom_items.contains(item.as_str()) {
                    push(
                        input_ptr,
                        format!(
                            "custom item `{item}` cannot be a recipe ingredient: vanilla recipes match only by item id or tag"
                        ),
                    );
                } else if !ingredient(item)
                    || (!item.starts_with('#') && !available(item, "minecraft:item"))
                {
                    push(input_ptr, format!("`{item}` is not an item id or `#tag`"));
                }
            }
            if let Some(result) = result
                && (!item_id(&result.item) || !(1..=99).contains(&result.count))
            {
                push(
                    format!("{ptr}/result"),
                    "a recipe result is an item id with a count of 1..=99".into(),
                );
            }
        }
        for (index, menu) in module.menus.iter().enumerate() {
            let ptr = format!("/menus/{index}");
            if !resource_ids.insert(format!("menu {}", menu.id)) {
                push(format!("{ptr}/id"), format!("duplicate menu `{}`", menu.id));
            }
            if !(1..=i32::MAX as u32).contains(&menu.ticks) {
                push(
                    format!("{ptr}/ticks"),
                    "menu timeout must be in 1..=2147483647".into(),
                );
            }
            if menu
                .page_size
                .is_some_and(|value| !(1..=64).contains(&value))
            {
                push(
                    format!("{ptr}/page_size"),
                    "menu page size must be in 1..=64".into(),
                );
            }
            let mut names = BTreeSet::from(["main"]);
            for (index, view) in menu.views.iter().enumerate() {
                let ptr = format!("{ptr}/views/{index}");
                if view.name.is_empty()
                    || !view
                        .name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    || !names.insert(&view.name)
                {
                    push(format!("{ptr}/name"), "expected a unique menu view name using letters, digits, or underscores; main is reserved".into());
                }
                if let Some(error) = static_text_error(&view.title, None) {
                    push(format!("{ptr}/title"), error);
                }
            }
            if menu
                .start
                .as_deref()
                .is_some_and(|name| !names.contains(name))
            {
                push(format!("{ptr}/start"), "unknown starting menu view".into());
            }
            if menu
                .choice_lists()
                .map(|(choices, _)| choices.len())
                .sum::<usize>()
                > 256
            {
                push(
                    format!("{ptr}/choices"),
                    "a menu supports at most 256 choices across all views".into(),
                );
            }
            if let Some(error) = static_text_error(&menu.title, None) {
                push(format!("{ptr}/title"), error);
            }
            for (choices, path) in menu.choice_lists() {
                if choices.is_empty() {
                    push(
                        format!("{ptr}/{path}"),
                        "each menu view needs between 1 and 256 choices".into(),
                    );
                }
                for (choice_index, choice) in choices.iter().enumerate() {
                    let ptr = format!("{ptr}/{path}/{choice_index}");
                    if choice.action.is_none() && choice.next.is_none() {
                        push(
                            ptr.clone(),
                            "a menu choice needs an action or destination".into(),
                        );
                    }
                    if choice
                        .next
                        .as_deref()
                        .is_some_and(|name| !names.contains(name))
                    {
                        push(
                            format!("{ptr}/next"),
                            "unknown destination menu view".into(),
                        );
                    }
                    if let Some(error) = static_text_error(&choice.label, None) {
                        push(format!("{ptr}/label"), error);
                    }
                }
            }
        }
        for (index, progression) in module.progressions.iter().enumerate() {
            let ptr = format!("/progressions/{index}");
            if !resource_ids.insert(format!("progression {}", progression.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate progression `{}`", progression.id),
                );
            }
            let count = progression.objectives.len();
            if !(1..=256).contains(&count) {
                push(
                    format!("{ptr}/objectives"),
                    "a progression needs between 1 and 256 objectives".into(),
                );
            }
            if progression.branches.len() > 256 {
                push(
                    format!("{ptr}/branches"),
                    "a progression supports at most 256 branches".into(),
                );
            }
            for (index, branch) in progression.branches.iter().enumerate() {
                if branch.from >= branch.to || branch.to as usize > count {
                    push(
                        format!("{ptr}/branches/{index}"),
                        "branch must move to a later objective or the completion index".into(),
                    );
                }
            }
        }
        for (index, combat) in module.combats.iter().enumerate() {
            let ptr = format!("/combats/{index}");
            if !resource_ids.insert(format!("combat {}", combat.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate combat policy `{}`", combat.id),
                );
            }
            if !combat.max.is_finite() || combat.max <= 0.0 {
                push(
                    format!("{ptr}/max"),
                    "combat maximum must be finite and positive".into(),
                );
            }
            if combat
                .resistance
                .is_some_and(|value| !(0.0..=1.0).contains(&value))
            {
                push(
                    format!("{ptr}/resistance"),
                    "combat resistance must be between zero and one".into(),
                );
            }
            if combat.shield.as_ref() == Some(&combat.health) {
                push(
                    format!("{ptr}/shield"),
                    "combat shield and health must use distinct fields".into(),
                );
            }
            if combat.critical.as_ref().is_some_and(|critical| {
                !critical.multiplier.is_finite() || critical.multiplier < 1.0
            }) {
                push(
                    format!("{ptr}/critical/multiplier"),
                    "critical multiplier must be finite and at least one".into(),
                );
            }
            if combat
                .immunity
                .as_ref()
                .is_some_and(|immunity| !(1..=i32::MAX as u32).contains(&immunity.ticks))
            {
                push(
                    format!("{ptr}/immunity/ticks"),
                    "combat immunity must last 1..=2147483647 ticks".into(),
                );
            }
            for (field, score, timer) in [
                ("health", Some(&combat.health), false),
                ("shield", combat.shield.as_ref(), false),
                (
                    "immunity/timer",
                    combat.immunity.as_ref().map(|immunity| &immunity.timer),
                    true,
                ),
            ] {
                if let Some(score) = score {
                    let valid = program
                        .modules
                        .iter()
                        .flat_map(|module| &module.states)
                        .find(|state| state.scope == "player" && state.id == score.state)
                        .and_then(|state| {
                            state.fields.iter().find(|field| field.name == score.field)
                        })
                        .is_some_and(|field| {
                            field.timer == timer
                                && field.bounds.is_none_or(|(min, max)| min <= 0 && max > 0)
                        });
                    if !valid {
                        push(format!("{ptr}/{field}"), if timer {
                            "combat immunity needs a player State timer"
                        } else {
                            "combat needs a non-ticking player score with bounds admitting zero and positive values"
                        }.into());
                    }
                }
            }
        }
        for (index, ability) in module.abilities.iter().enumerate() {
            let ptr = format!("/abilities/{index}");
            if !resource_ids.insert(format!("ability {}", ability.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate ability `{}`", ability.id),
                );
            }
            if !(1..=i32::MAX as u32).contains(&ability.ticks) {
                push(
                    format!("{ptr}/ticks"),
                    "ability duration must be in 1..=2147483647".into(),
                );
            }
            if ability
                .cost
                .as_ref()
                .is_some_and(|cost| cost.amount > i32::MAX as u32)
            {
                push(
                    format!("{ptr}/cost/amount"),
                    "ability cost must not exceed 2147483647".into(),
                );
            }
            if ability
                .cooldown
                .as_ref()
                .is_some_and(|cooldown| !(1..=i32::MAX as u32).contains(&cooldown.ticks))
            {
                push(
                    format!("{ptr}/cooldown/ticks"),
                    "ability cooldown must be in 1..=2147483647".into(),
                );
            }
        }
        for (index, scene) in module.scenes.iter().enumerate() {
            let ptr = format!("/scenes/{index}");
            if !resource_ids.insert(format!("scene {}", scene.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate scene `{}`", scene.id),
                );
            }
            if !(1..=i32::MAX as u32).contains(&scene.ticks) {
                push(
                    format!("{ptr}/ticks"),
                    "scene lifetime must be in 1..=2147483647 ticks".into(),
                );
            }
            if scene.keyframes.len() > 256 {
                push(
                    format!("{ptr}/keyframes"),
                    "a scene supports at most 256 keyframes".into(),
                );
            }
            for (index, part) in scene.parts.iter().enumerate() {
                let ptr = format!("{ptr}/parts/{index}/content");
                match &part.content {
                    SceneContent::Item { item } if !item_id(item) => {
                        push(
                            format!("{ptr}/item"),
                            "expected a plain item identity".into(),
                        );
                    }
                    SceneContent::Text { text } => {
                        if let Some(error) = static_text_error(text, None) {
                            push(format!("{ptr}/text"), error);
                        }
                    }
                    _ => {}
                }
            }
        }
        for (index, job) in module.jobs.iter().enumerate() {
            let ptr = format!("/jobs/{index}");
            if !resource_ids.insert(format!("job {}", job.id)) {
                push(format!("{ptr}/id"), format!("duplicate job `{}`", job.id));
            }
            for (field, value, max) in [
                ("ticks", job.ticks, i32::MAX as u32),
                ("count", job.count, i32::MAX as u32),
                ("per_tick", job.per_tick.unwrap_or(1), 64),
                ("capacity", job.capacity.unwrap_or(128), i32::MAX as u32),
            ] {
                if !(1..=max).contains(&value) {
                    push(
                        format!("{ptr}/{field}"),
                        format!("job {field} must be in 1..={max}"),
                    );
                }
            }
            let valid = program
                .modules
                .iter()
                .flat_map(|module| &module.states)
                .find(|state| state.scope == "global" && state.id == job.index.state)
                .and_then(|state| {
                    state
                        .fields
                        .iter()
                        .find(|field| field.name == job.index.field)
                })
                .is_some_and(|field| {
                    !field.timer
                        && field.bounds.is_none_or(|(min, max)| {
                            min <= 0 && i64::from(max) >= i64::from(job.count) - 1
                        })
                });
            if !valid {
                push(
                    format!("{ptr}/index"),
                    "job index needs a non-ticking global score that can represent all indices"
                        .into(),
                );
            }
        }
        for (index, session) in module.sessions.iter().enumerate() {
            let ptr = format!("/sessions/{index}");
            if !resource_ids.insert(format!("session {}", session.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate session `{}`", session.id),
                );
            }
            for (field, value) in std::iter::once(("ticks", session.ticks))
                .chain(session.every.map(|every| ("every", every)))
            {
                if !(1..=i32::MAX as u32).contains(&value) {
                    push(
                        format!("{ptr}/{field}"),
                        "session ticks must be in 1..=2147483647".into(),
                    );
                }
            }
        }
        for (index, projectile) in module.projectiles.iter().enumerate() {
            let ptr = format!("/projectiles/{index}");
            if !resource_ids.insert(format!("projectile {}", projectile.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate projectile `{}`", projectile.id),
                );
            }
            if !(1..=i32::MAX as u32).contains(&projectile.ticks) {
                push(
                    format!("{ptr}/ticks"),
                    "projectile ticks must be in 1..=2147483647".into(),
                );
            }
            if let Some(damage) = projectile.damage
                && let Err(error) = sand_commands::DamageAmount::try_fixed(damage)
            {
                push(format!("{ptr}/damage"), error.to_string());
            }
            if let Some(target) = &projectile.target
                && sand_version::registry::contains(
                    &program.target.minecraft,
                    "minecraft:entity_type",
                    &target.to_string(),
                ) == Some(false)
            {
                push(
                    format!("{ptr}/target"),
                    format!("unknown entity type `{target}`"),
                );
            }
        }
        for (index, hud) in module.huds.iter().enumerate() {
            let ptr = format!("/huds/{index}");
            if !resource_ids.insert(format!("hud {}", hud.id)) {
                push(format!("{ptr}/id"), format!("duplicate HUD `{}`", hud.id));
            }
            for (field, value) in
                std::iter::once(("ticks", hud.ticks)).chain(hud.every.map(|every| ("every", every)))
            {
                if !(1..=i32::MAX as u32).contains(&value) {
                    push(
                        format!("{ptr}/{field}"),
                        "HUD ticks must be in 1..=2147483647".into(),
                    );
                }
            }
            if hud.actionbar.is_empty() && hud.bossbar.is_none() {
                push(ptr.clone(), "a HUD needs an actionbar or a boss bar".into());
            }
            if let Some(error) = static_text_error(&hud.actionbar, None) {
                push(format!("{ptr}/actionbar"), error);
            }
            if let Some(bar) = &hud.bossbar {
                if !(1..=i32::MAX as u32).contains(&bar.max) {
                    push(
                        format!("{ptr}/bossbar/max"),
                        "HUD bar maximum must be in 1..=2147483647".into(),
                    );
                }
                if let Some(error) = static_text_error(&bar.title, None) {
                    push(format!("{ptr}/bossbar/title"), error);
                }
            }
        }
        for (index, transfer) in module.inventory_transfers.iter().enumerate() {
            let ptr = format!("/inventory_transfers/{index}");
            if !resource_ids.insert(format!("inventory transfer {}", transfer.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate inventory transfer `{}`", transfer.id),
                );
            }
            if transfer.from == transfer.to {
                push(
                    format!("{ptr}/to"),
                    "source and destination must be distinct numbered player slots".into(),
                );
            }
            if transfer
                .amount
                .is_some_and(|amount| !(1..=99).contains(&amount))
            {
                push(
                    format!("{ptr}/amount"),
                    "inventory transfers require 1..=99 items".into(),
                );
            }
        }
        for (index, template) in module.structure_templates.iter().enumerate() {
            use sand_components::component::DatapackComponent;
            let ptr = format!("/structure_templates/{index}");
            let id = template.resource_location();
            if !resource_ids.insert(format!("structure template {id}")) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate structure template `{id}`"),
                );
            }
            if let Err(error) = template.validate() {
                push(format!("{ptr}/source"), error.to_string());
                continue;
            }
            let source = template.source_path();
            match program.structure_assets.get(source) {
                Some(bytes) if bytes.len() >= 18 && bytes.starts_with(&[0x1f, 0x8b, 8]) => {}
                Some(_) => push(
                    format!("{ptr}/source"),
                    format!("structure asset `{source}` must be a gzip NBT file"),
                ),
                None => push(
                    format!("{ptr}/source"),
                    format!("missing structure asset `{source}`"),
                ),
            }
        }
        for (index, biome) in module.biomes.iter().enumerate() {
            let ptr = format!("/biomes/{index}");
            if !resource_ids.insert(format!("biome {}", biome.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate biome `{}`", biome.id),
                );
            }
            if biome
                .downfall
                .is_some_and(|value| !(0.0..=1.0).contains(&value))
            {
                push(
                    format!("{ptr}/downfall"),
                    "downfall must be between zero and one".into(),
                );
            }
            for (step, features) in &biome.features {
                for (index, feature) in features.iter().enumerate() {
                    if !available(&feature.to_string(), "minecraft:worldgen/placed_feature") {
                        push(
                            format!("{ptr}/features/{}/{index}", step.as_str()),
                            format!("unknown placed feature `{feature}`"),
                        );
                    }
                }
            }
            let spawn_entity_exists = |id: &str| {
                syntax_id(id)
                    && sand_version::registry::contains(
                        &program.target.minecraft,
                        "minecraft:entity_type",
                        id,
                    ) != Some(false)
            };
            for (category, entries) in &biome.spawners {
                for (index, entry) in entries.iter().enumerate() {
                    let entity = entry.entity_type().to_string();
                    if !spawn_entity_exists(&entity) {
                        push(
                            format!("{ptr}/spawners/{}/{index}/entity_type", category.as_str()),
                            format!("unknown entity type `{entity}`"),
                        );
                    }
                }
            }
            for entity in biome.spawn_costs.keys() {
                if !spawn_entity_exists(entity) {
                    push(
                        format!("{ptr}/spawn_costs/{entity}"),
                        format!("unknown entity type `{entity}`"),
                    );
                }
            }
        }
        for (index, dimension_type) in module.dimension_types.iter().enumerate() {
            use sand_components::component::DatapackComponent;
            let id = dimension_type.resource_location();
            let ptr = format!("/dimension_types/{index}");
            if !resource_ids.insert(format!("dimension type {id}")) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate dimension type `{id}`"),
                );
            }
            if let Err(error) = dimension_type.validate() {
                push(ptr, error.to_string());
            }
        }
        for (index, dimension) in module.dimensions.iter().enumerate() {
            let ptr = format!("/dimensions/{index}");
            let id = dimension.id();
            if !resource_ids.insert(format!("dimension {id}")) {
                push(format!("{ptr}/id"), format!("duplicate dimension `{id}`"));
            }
            let dimension_type = match dimension {
                DimensionDefinition::Flat(dimension) => &dimension.dimension_type,
                DimensionDefinition::Noise(dimension) => &dimension.dimension_type,
            };
            if !available(&dimension_type.to_string(), "minecraft:dimension_type") {
                push(
                    format!("{ptr}/dimension_type"),
                    format!("unknown dimension type `{dimension_type}`"),
                );
            }
            let check_biome = |biome: &sand_components::BiomeId| {
                (!available(&biome.to_string(), "minecraft:worldgen/biome"))
                    .then(|| format!("unknown biome `{biome}`"))
            };
            match dimension {
                DimensionDefinition::Flat(dimension) => {
                    for (layer_index, layer) in dimension.flat.layers.iter().enumerate() {
                        let block = layer.block.to_string();
                        if !block_id(&block) {
                            push(
                                format!("{ptr}/flat/layers/{layer_index}/block"),
                                format!("unknown block `{block}`"),
                            );
                        }
                    }
                    if let Some(error) = check_biome(&dimension.flat.biome_id) {
                        push(format!("{ptr}/flat/biome_id"), error);
                    }
                }
                DimensionDefinition::Noise(dimension) => {
                    let settings = dimension.noise_settings.to_string();
                    if !available(&settings, "minecraft:worldgen/noise_settings") {
                        push(
                            format!("{ptr}/noise_settings"),
                            format!("unknown noise settings `{settings}`"),
                        );
                    }
                    match &dimension.biome_source {
                        sand_components::BiomeSource::Fixed(biome) => {
                            if let Some(error) = check_biome(biome) {
                                push(format!("{ptr}/biome_source/fixed"), error);
                            }
                        }
                        sand_components::BiomeSource::Checkerboard { biomes, .. } => {
                            for (index, biome) in biomes.iter().enumerate() {
                                if let Some(error) = check_biome(biome) {
                                    push(
                                        format!("{ptr}/biome_source/checkerboard/biomes/{index}"),
                                        error,
                                    );
                                }
                            }
                        }
                        sand_components::BiomeSource::Vanilla => {}
                        sand_components::BiomeSource::Raw(_) => push(
                            format!("{ptr}/biome_source"),
                            "portable noise dimensions require a typed biome source".into(),
                        ),
                    }
                }
            }
        }
        for (index, dialog) in module.dialogs.iter().enumerate() {
            let ptr = format!("/dialogs/{index}");
            if !resource_ids.insert(format!("dialog {}", dialog.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate dialog `{}`", dialog.id),
                );
            }
            let text_blocks = std::iter::once(("title".to_owned(), &dialog.title))
                .chain(
                    dialog
                        .body
                        .iter()
                        .enumerate()
                        .map(|(i, text)| (format!("body/{i}"), text)),
                )
                .chain(std::iter::once(("button".to_owned(), &dialog.button)));
            for (field, text) in text_blocks {
                if let Some(error) = static_text_error(text, None) {
                    push(format!("{ptr}/{field}"), error);
                }
            }
        }
        for (index, advancement) in module.advancements.iter().enumerate() {
            let ptr = format!("/advancements/{index}");
            if !resource_ids.insert(format!("advancement {}", advancement.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate advancement `{}`", advancement.id),
                );
            }
            if advancement.criteria.is_empty() {
                push(
                    format!("{ptr}/criteria"),
                    "an advancement needs at least one criterion".into(),
                );
            }
            if advancement.experience < 0 {
                push(
                    format!("{ptr}/experience"),
                    "advancement experience must be nonnegative".into(),
                );
            }
            for (name, criterion) in &advancement.criteria {
                let criteria_path = format!(
                    "{ptr}/criteria/{}",
                    name.replace('~', "~0").replace('/', "~1")
                );
                if name.trim().is_empty() || name.chars().any(char::is_control) {
                    push(
                        criteria_path.clone(),
                        "criterion names must be nonblank and contain no control characters".into(),
                    );
                }
                let (id, registry, field) = match criterion {
                    // DimensionId validates the identity and may reference a world supplied by another pack.
                    AdvancementCriterion::ArriveDimension { .. } => continue,
                    AdvancementCriterion::InventoryItem { item }
                    | AdvancementCriterion::ConsumeItem { item } => {
                        (item, "minecraft:item", "item")
                    }
                    AdvancementCriterion::PlaceBlock { block } => {
                        (block, "minecraft:block", "block")
                    }
                    AdvancementCriterion::KillEntity { entity } => {
                        (entity, "minecraft:entity_type", "entity")
                    }
                };
                if !syntax_id(id)
                    || !available(id, registry)
                    || (field == "item" && custom_items.contains(id))
                {
                    push(
                        format!("{criteria_path}/{field}"),
                        format!("`{id}` is not a vanilla {field} identity"),
                    );
                }
            }
            if let Some(display) = &advancement.display {
                if !item_id(&display.icon) || custom_items.contains(&display.icon) {
                    push(
                        format!("{ptr}/display/icon"),
                        "an advancement icon needs a vanilla item".into(),
                    );
                }
                for (field, text) in [
                    ("title", &display.title),
                    ("description", &display.description),
                ] {
                    if let Some(error) = static_text_error(text, None) {
                        push(format!("{ptr}/display/{field}"), error);
                    }
                }
            }
        }
        for (index, predicate) in module.predicates.iter().enumerate() {
            let ptr = format!("/predicates/{index}");
            if !resource_ids.insert(format!("predicate {}", predicate.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate predicate `{}`", predicate.id),
                );
            }
            if predicate.biome.is_none()
                && predicate.dimension.is_none()
                && predicate.block.is_none()
                && predicate.weather.is_none()
                && predicate.chance.is_none()
            {
                push(
                    ptr.clone(),
                    "a world predicate needs at least one condition".into(),
                );
            }
            if let Some(biome) = &predicate.biome
                && !(syntax_id(biome) && available(biome, "minecraft:worldgen/biome"))
            {
                push(
                    format!("{ptr}/biome"),
                    format!(
                        "`{biome}` is not a biome in Minecraft {}",
                        program.target.minecraft
                    ),
                );
            }
            if predicate
                .block
                .as_ref()
                .is_some_and(|block| !block_id(block))
            {
                push(
                    format!("{ptr}/block"),
                    "predicate block must exist in the selected target".into(),
                );
            }
            if predicate
                .chance
                .is_some_and(|chance| !(0.0..=1.0).contains(&chance))
            {
                push(
                    format!("{ptr}/chance"),
                    "predicate chance must be between zero and one".into(),
                );
            }
        }
        for (index, damage_type) in module.damage_types.iter().enumerate() {
            let ptr = format!("/damage_types/{index}");
            use sand_components::DatapackComponent;
            let id = damage_type.resource_location();
            if !resource_ids.insert(format!("damage_type {id}")) {
                push(format!("{ptr}/id"), format!("duplicate damage type `{id}`"));
            }
            if let Err(error) = damage_type.validate() {
                push(ptr, error.to_string());
            }
        }
        for (index, table) in module.loot_tables.iter().enumerate() {
            let ptr = format!("/loot_tables/{index}");
            if !resource_ids.insert(format!("loot {}", table.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate loot table `{}`", table.id),
                );
            }
            if table.pools.is_empty() {
                push(
                    format!("{ptr}/pools"),
                    "a loot table needs at least one pool".into(),
                );
            }
            for (index, pool) in table.pools.iter().enumerate() {
                let ptr = format!("{ptr}/pools/{index}");
                if !(1..=1000).contains(&pool.rolls) || pool.items.is_empty() {
                    push(
                        ptr.clone(),
                        "a loot pool needs 1..=1000 rolls and at least one item".into(),
                    );
                }
                if pool
                    .chance
                    .is_some_and(|chance| !(0.0..=1.0).contains(&chance))
                {
                    push(
                        format!("{ptr}/chance"),
                        "loot pool chance must be between zero and one".into(),
                    );
                }
                for (index, item) in pool.items.iter().enumerate() {
                    let ptr = format!("{ptr}/items/{index}");
                    if !item_id(&item.item) || custom_items.contains(&item.item) {
                        push(
                            format!("{ptr}/item"),
                            format!("`{}` is not a vanilla item id", item.item),
                        );
                    }
                    if !(1..=i32::MAX as u32).contains(&item.weight)
                        || !(1..=99).contains(&item.count)
                    {
                        push(
                            ptr,
                            "loot weight must be a positive scoreboard integer and count 1..=99"
                                .into(),
                        );
                    }
                }
            }
        }
        for (index, bar) in module.boss_bars.iter().enumerate() {
            let ptr = format!("/boss_bars/{index}");
            if !resource_ids.insert(format!("bossbar {}", bar.id)) {
                push(
                    format!("{ptr}/id"),
                    format!("duplicate boss bar `{}`", bar.id),
                );
            }
            if bar.max == 0 || bar.max > i32::MAX as u32 {
                push(format!("{ptr}/max"), "boss bar max must be positive".into());
            }
            if let Some(error) = static_text_error(&bar.title, None) {
                push(format!("{ptr}/title"), error);
            }
        }
        for (index, tag) in module.resource_tags.iter().enumerate() {
            let ptr = format!("/resource_tags/{index}");
            if !resource_ids.insert(format!("tag {:?} {}", tag.kind, tag.id)) {
                push(format!("{ptr}/id"), format!("duplicate tag `{}`", tag.id));
            }
            for (i, value) in tag.values.iter().enumerate() {
                let registry = match tag.kind {
                    TagKind::Item => "minecraft:item",
                    TagKind::Block => "minecraft:block",
                    TagKind::EntityType => "minecraft:entity_type",
                };
                if !ingredient(value)
                    || custom_items.contains(value.as_str())
                    || (!value.starts_with('#') && !available(value, registry))
                {
                    push(
                        format!("{ptr}/values/{i}"),
                        format!("`{value}` is not a vanilla id or `#tag`"),
                    );
                }
            }
        }
    }
    if errors.is_empty() {
        let value = serde_json::to_value(program).expect("owned program serializes");
        let mut pending = vec![(&value, 0usize)];
        while let Some((value, depth)) = pending.pop() {
            if depth > limits::JSON_DEPTH {
                errors.push(Diagnostic::error(
                    "SAND_PROGRAM_LIMIT",
                    "",
                    "",
                    "constructed program exceeds JSON nesting limit",
                ));
                break;
            }
            match value {
                serde_json::Value::Array(values) => {
                    pending.extend(values.iter().map(|value| (value, depth + 1)))
                }
                serde_json::Value::Object(values) => {
                    pending.extend(values.values().map(|value| (value, depth + 1)))
                }
                _ => {}
            }
        }
        match serde_json::to_vec(&value) {
            Ok(bytes) if bytes.len() <= limits::INPUT_BYTES => {}
            _ => errors.push(Diagnostic::error(
                "SAND_PROGRAM_LIMIT",
                "",
                "",
                "constructed program exceeds 8 MiB",
            )),
        }
    }
    if errors.is_empty() {
        Ok(target.expect("validated target"))
    } else {
        Err(errors)
    }
}

fn safe_path(value: &str) -> bool {
    super::decode::safe_module_path(value)
}
fn valid_id(id: &ResourceLocation) -> bool {
    safe_path(id.namespace()) && safe_path(id.path()) && id.namespace() != "__sand_local"
}

fn text_shape_error(
    text: &[TextPart],
    depth: usize,
    interactive: bool,
    book_pages: Option<usize>,
) -> Option<String> {
    for part in text {
        if depth >= TextPart::MAX_NESTING_DEPTH
            && matches!(
                part,
                TextPart::Translation { .. } | TextPart::Interactive { .. }
            )
        {
            return Some("text exceeds the component nesting limit".into());
        }
        match part {
            TextPart::Translation { key, arguments, .. } => {
                if key.trim().is_empty() || key.chars().any(char::is_control) {
                    return Some(
                        "translation keys must be nonblank and contain no control characters"
                            .into(),
                    );
                }
                if let Some(error) = arguments
                    .iter()
                    .find_map(|parts| text_shape_error(parts, depth + 1, interactive, book_pages))
                {
                    return Some(error);
                }
            }
            TextPart::Interactive {
                content,
                tooltip,
                click,
            } => {
                if !interactive {
                    return Some(
                        "interactive text is supported in chat messages and written-book pages"
                            .into(),
                    );
                }
                if content.is_empty() || (click.is_none() && tooltip.is_empty()) {
                    return Some(
                        "interactive text needs text and a tooltip or click action".into(),
                    );
                }
                if let Some(click) = click {
                    match click {
                        ClickEvent::RunCommand(_) => {
                            return Some(
                                "portable text can suggest a command without running it".into(),
                            );
                        }
                        ClickEvent::ChangePage(page)
                            if book_pages.is_none_or(|count| (*page as usize) > count) =>
                        {
                            return Some(
                                "page navigation needs a valid page in the same written book"
                                    .into(),
                            );
                        }
                        _ => {}
                    }
                    if let Err(error) = click.validate() {
                        return Some(error.to_string());
                    }
                }
                if let Some(error) = text_shape_error(content, depth + 1, interactive, book_pages)
                    .or_else(|| text_shape_error(tooltip, depth + 1, interactive, book_pages))
                {
                    return Some(error);
                }
            }
            _ => {}
        }
    }
    None
}

fn static_text_error(text: &[TextPart], book_pages: Option<usize>) -> Option<String> {
    text_shape_error(text, 0, book_pages.is_some(), book_pages).or_else(|| {
        text.iter()
            .any(|part| !part.is_static())
            .then(|| "static text cannot use stored values, scores or player names".into())
    })
}

fn is_state_migration(operations: &[Operation]) -> bool {
    let mut pending = vec![operations];
    while let Some(operations) = pending.pop() {
        for operation in operations {
            match &operation.action {
                Action::Branch {
                    then, otherwise, ..
                } => {
                    pending.extend([then.as_slice(), otherwise.as_slice()]);
                }
                Action::While { body, .. } => pending.push(body),
                Action::ScoreSet { .. }
                | Action::ScoreAdd { .. }
                | Action::ScoreOperation { .. }
                | Action::DataModify { .. }
                | Action::DataRemove { .. }
                | Action::DataRead { .. }
                | Action::DataEqual { .. }
                | Action::Return => {}
                _ => return false,
            }
        }
    }
    true
}

struct Validator<'a, 'e> {
    dialogs_supported: bool,
    advancements: BTreeMap<String, BTreeSet<String>>,
    transfers: BTreeSet<String>,
    huds: BTreeSet<String>,
    projectiles: BTreeSet<String>,
    progressions: BTreeSet<String>,
    scenes: BTreeSet<String>,
    jobs: BTreeSet<String>,
    sessions: BTreeSet<String>,
    abilities: BTreeSet<String>,
    combats: BTreeSet<String>,
    menus: BTreeSet<String>,
    target: &'a str,
    boss_bars: BTreeSet<String>,
    loot_tables: BTreeSet<String>,
    custom_items: BTreeSet<String>,
    states: BTreeMap<String, (&'a str, BTreeSet<&'a str>)>,
    data_fields: BTreeMap<String, BTreeSet<&'a str>>,
    functions: BTreeMap<String, ExecutionContext>,
    identity_unsafe: BTreeSet<String>,
    operations: usize,
    flow_bytes: usize,
    raw: bool,
    errors: &'e mut Vec<Diagnostic>,
}
impl Validator<'_, '_> {
    fn message_parts(
        &mut self,
        text: &[TextPart],
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        for (index, part) in text.iter().enumerate() {
            let part_ptr = format!("{ptr}/{index}");
            match part {
                TextPart::Stored { value, .. } => self.data_reference(
                    &DataReference {
                        field: value.clone(),
                        path: Vec::new(),
                    },
                    context,
                    owner,
                    &format!("{part_ptr}/value"),
                    false,
                ),
                TextPart::Score { score, .. } => {
                    self.score(score, context, owner, &format!("{part_ptr}/score"));
                }
                TextPart::PlayerName { .. } => {
                    self.player(context, owner, &part_ptr, "the executing player's name")
                }
                TextPart::Translation { arguments, .. } => {
                    for (index, parts) in arguments.iter().enumerate() {
                        self.message_parts(
                            parts,
                            context,
                            owner,
                            &format!("{part_ptr}/arguments/{index}"),
                        );
                    }
                }
                TextPart::Interactive {
                    content, tooltip, ..
                } => {
                    self.message_parts(content, context, owner, &format!("{part_ptr}/content"));
                    self.message_parts(tooltip, context, owner, &format!("{part_ptr}/tooltip"));
                }
                TextPart::Text { .. } => {}
            }
        }
    }

    // Count copied hook invocations before the shared compiler materializes
    // transition bodies. Input declaration limits alone cannot bound this
    // transition × hook expansion. This is a lower bound on emitted bytes;
    // the normal output limits still check every complete generated resource.
    fn reserve_flow_expansion(&mut self, flow: &Flow, owner: &str, ptr: &str) -> bool {
        let mut enter = BTreeMap::<i32, (usize, usize)>::new();
        let mut exit = BTreeMap::<i32, (usize, usize)>::new();
        let mut calls = 0usize;
        let mut bytes = 0usize;
        for hook in &flow.hooks {
            let id = match &hook.function {
                FunctionReference::Internal { id } | FunctionReference::External { id, .. } => id,
            };
            let size = id.to_string().len().saturating_add("function ".len() + 1);
            let counters = match hook.phase {
                FlowHookPhase::Enter => enter.entry(hook.state).or_default(),
                FlowHookPhase::Exit => exit.entry(hook.state).or_default(),
                FlowHookPhase::Tick => {
                    calls = calls.saturating_add(1);
                    bytes = bytes.saturating_add(size);
                    continue;
                }
            };
            counters.0 = counters.0.saturating_add(1);
            counters.1 = counters.1.saturating_add(size);
        }
        let mut oversized = false;
        for transition in &flow.transitions {
            let before = exit.get(&transition.from).copied().unwrap_or_default();
            let after = enter.get(&transition.to).copied().unwrap_or_default();
            calls = calls.saturating_add(before.0).saturating_add(after.0);
            let body_bytes = before.1.saturating_add(after.1);
            oversized |= body_bytes > limits::DOCUMENT_BYTES;
            bytes = bytes.saturating_add(body_bytes);
        }
        self.operations = self.operations.saturating_add(calls);
        self.flow_bytes = self.flow_bytes.saturating_add(bytes);
        if oversized
            || self.operations > limits::OPERATIONS
            || self.flow_bytes > limits::OUTPUT_BYTES
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_LIMIT",
                owner,
                ptr,
                "expanded flow hooks exceed operation or generated-output limits",
            ));
            false
        } else {
            true
        }
    }

    fn comparison(
        &mut self,
        condition: &ScoreComparison,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        self.score(&condition.score, context, owner, &format!("{ptr}/score"));
        if condition
            .min
            .zip(condition.max)
            .is_some_and(|(min, max)| min > max)
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_RANGE",
                owner,
                ptr,
                "comparison minimum exceeds maximum",
            ));
        }
    }

    fn player(&mut self, context: ExecutionContext, owner: &str, ptr: &str, what: &str) {
        if context != ExecutionContext::Player {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_CONTEXT",
                owner,
                ptr,
                format!("{what} requires player execution context"),
            ));
        }
    }

    fn condition(
        &mut self,
        condition: &Condition,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        match condition {
            Condition::Range(range) => self.comparison(range, context, owner, ptr),
            Condition::Relation(relation) => {
                self.score(&relation.left, context, owner, &format!("{ptr}/left"));
                self.score(&relation.right, context, owner, &format!("{ptr}/right"));
            }
            Condition::Attached(test) => {
                self.attachable_state(&test.attached, context, owner, &format!("{ptr}/attached"))
            }
            Condition::Exists(test) => self.query(&test.exists, owner, &format!("{ptr}/exists")),
            Condition::Block(test) => {
                self.block_selector(&test.block, owner, &format!("{ptr}/block"));
            }
            Condition::Biome(test) => {
                self.resource(&test.biome, owner, &format!("{ptr}/biome"), "biome");
            }
            Condition::Dimension(test) => self.resource(
                &test.dimension.to_string(),
                owner,
                &format!("{ptr}/dimension"),
                "dimension",
            ),
            Condition::Holding(test) => {
                self.executor(context, owner, ptr, "testing a held item");
                if let Some(slot) = test.slot {
                    self.equipment_slot(slot, owner, &format!("{ptr}/slot"));
                    if test.offhand {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_OPTION",
                            owner,
                            ptr,
                            "choose either slot or offhand",
                        ));
                    }
                }
                if !self.custom_items.contains(&test.holding) {
                    self.resource(&test.holding, owner, &format!("{ptr}/holding"), "item");
                }
            }
            Condition::ProgressionCompleted {
                progression_completed,
            } => {
                self.player(
                    context,
                    owner,
                    ptr,
                    "checking the executing player's progression",
                );
                if !self
                    .progressions
                    .contains(&progression_completed.to_string())
                {
                    self.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_REFERENCE",
                        owner,
                        ptr,
                        format!("unknown progression `{progression_completed}`"),
                    ));
                }
            }
            Condition::Advancement {
                advancement,
                criterion,
            } => {
                self.player(context, owner, ptr, "advancement progress");
                let id = advancement.to_string();
                self.resource(&id, owner, &format!("{ptr}/advancement"), "advancement");
                if let Some(criterion) = criterion {
                    let error =
                        if criterion.trim().is_empty() || criterion.chars().any(char::is_control) {
                            Some(
                            "criterion names must be nonblank and contain no control characters"
                                .into(),
                        )
                        } else if self
                            .advancements
                            .get(&id)
                            .is_some_and(|names| !names.contains(criterion))
                        {
                            Some(format!("advancement `{id}` has no criterion `{criterion}`"))
                        } else {
                            None
                        };
                    if let Some(error) = error {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RESOURCE",
                            owner,
                            &format!("{ptr}/criterion"),
                            error,
                        ));
                    }
                }
            }
            Condition::Predicate { predicate } => {
                self.resource(
                    &predicate.to_string(),
                    owner,
                    &format!("{ptr}/predicate"),
                    "predicate",
                );
            }
            Condition::Flag(_) => self.executor(context, owner, ptr, "testing an entity flag"),
        }
    }

    fn equipment_slot(&mut self, slot: sand_components::EquipmentSlot, owner: &str, ptr: &str) {
        if let Err(error) = crate::item::location::equipment_item_slot(slot) {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_OPTION",
                owner,
                ptr,
                error.to_string(),
            ));
        }
    }

    /// An entity State reference, used where `@s` must be some entity.
    fn attachable_state(
        &mut self,
        state: &ResourceLocation,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        if !self.is_attachable_state(state) {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                ptr,
                format!("`{state}` is not a declared entity State"),
            ));
        }
        self.executor(context, owner, ptr, "an entity State");
    }

    fn is_attachable_state(&self, state: &ResourceLocation) -> bool {
        self.states
            .get(&state.to_string())
            .is_some_and(|(scope, _)| *scope == "entity")
    }

    fn frame_fields(
        &mut self,
        fields: &[FieldReference],
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
        global: bool,
    ) {
        let mut seen = std::collections::BTreeSet::new();
        for (index, field) in fields.iter().enumerate() {
            let field_ptr = format!("{ptr}/{index}");
            if global
                && self
                    .states
                    .get(&field.state.to_string())
                    .is_some_and(|(scope, _)| *scope != "global")
            {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_STATE",
                    owner,
                    &field_ptr,
                    "a wait retains global invocation slots; player and entity State stay live",
                ));
            }
            if self
                .data_fields
                .get(&field.state.to_string())
                .is_some_and(|fields| fields.contains(field.field.as_str()))
            {
                self.data_reference(
                    &DataReference {
                        field: field.clone(),
                        path: Vec::new(),
                    },
                    context,
                    owner,
                    &field_ptr,
                    false,
                );
            } else {
                self.score(field, context, owner, &field_ptr);
            }
            if !seen.insert((field.state.to_string(), field.field.clone())) {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_STATE",
                    owner,
                    &field_ptr,
                    "a call frame names each field once",
                ));
            }
        }
    }

    fn executor(&mut self, context: ExecutionContext, owner: &str, ptr: &str, what: &str) {
        if context == ExecutionContext::Server {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_CONTEXT",
                owner,
                ptr,
                format!("{what} requires a player or entity executor"),
            ));
        }
    }

    fn boss_bar(&mut self, bar: &ResourceLocation, owner: &str, ptr: &str) {
        if !self.boss_bars.contains(&bar.to_string()) {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                ptr,
                format!("`{bar}` is not a declared boss bar"),
            ));
        }
    }

    fn tag_name(&mut self, tag: &str, owner: &str, ptr: &str) {
        if tag.is_empty()
            || !tag
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.+-".contains(&b))
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_ID",
                owner,
                ptr,
                format!("`{tag}` is not a valid entity tag (letters, digits, `_ . + -`)"),
            ));
        }
    }

    fn wait_group(&mut self, group: Option<&str>, owner: &str, ptr: &str) {
        if let Some(group) = group
            && (!(1..=32).contains(&group.len())
                || !group
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'))
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_ID",
                owner,
                &format!("{ptr}/action/group"),
                "a wait group needs 1 to 32 lowercase letters, digits, or underscores",
            ));
        }
    }

    fn query(&mut self, query: &EntityQuery, owner: &str, ptr: &str) {
        if let Some(state) = &query.with_state
            && !self.is_attachable_state(state)
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                &format!("{ptr}/with_state"),
                format!("`{state}` is not a declared entity State"),
            ));
        }
        if let Some(entity_type) = &query.entity_type {
            self.resource(
                entity_type,
                owner,
                &format!("{ptr}/entity_type"),
                "entity type",
            );
        }
        if let Some(tag) = &query.tag {
            self.tag_name(tag, owner, &format!("{ptr}/tag"));
        }
        if let Some(filter) = &query.score {
            let field = &filter.score;
            if !self
                .states
                .get(&field.state.to_string())
                .is_some_and(|(scope, fields)| {
                    matches!(*scope, "player" | "entity") && fields.contains(field.field.as_str())
                })
            {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_REFERENCE",
                    owner,
                    &format!("{ptr}/score/score"),
                    "a query score filter requires a declared player or entity scoreboard field",
                ));
            }
            if (filter.min.is_none() && filter.max.is_none())
                || filter
                    .min
                    .zip(filter.max)
                    .is_some_and(|(min, max)| min > max)
            {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_RANGE",
                    owner,
                    &format!("{ptr}/score"),
                    "query score bounds must be ordered with at least one bound",
                ));
            }
        }
        if let Some(nbt) = &query.nbt {
            use sand_commands::Validate;
            let profile = sand_commands::CommandProfile::unprofiled();
            let selector = sand_commands::Selector::all_entities().nbt_raw(nbt.to_string());
            if let Err(error) = nbt
                .validate(&profile)
                .and_then(|()| selector.validate(&profile))
            {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_VALUE",
                    owner,
                    &format!("{ptr}/nbt"),
                    error.to_string(),
                ));
            }
        }
        let distance = |d: Option<f64>| d.is_none_or(|d| d.is_finite() && d >= 0.0);
        if !distance(query.min_distance)
            || !distance(query.max_distance)
            || query
                .min_distance
                .zip(query.max_distance)
                .is_some_and(|(min, max)| min > max)
            || query.limit == Some(0)
            || query.limit.is_some_and(|l| l > i32::MAX as u32)
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_RANGE",
                owner,
                ptr,
                "entity query distances must be non-negative and ordered, and limits positive",
            ));
        }
    }

    fn amount(
        &mut self,
        amount: &Amount,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
        bounds: Option<(f64, f64)>,
    ) -> bool {
        let valid = match amount {
            Amount::Value(value) => {
                value.is_finite() && bounds.is_none_or(|(min, max)| *value >= min && *value <= max)
            }
            Amount::Field(field) => {
                self.score(&field.score, context, owner, ptr);
                field.scale.is_finite() && field.scale > 0.0 && (1.0 / field.scale).is_finite()
            }
        };
        if !valid {
            self.errors.push(Diagnostic::error("SAND_PROGRAM_RANGE", owner, ptr, "numeric argument must be finite, within its command bounds, and use a positive finite scale"));
        }
        valid
    }

    fn resource(&mut self, id: &str, owner: &str, ptr: &str, kind: &str) {
        let registry = match kind {
            "item" => Some("minecraft:item"),
            "block" => Some("minecraft:block"),
            "biome" => Some("minecraft:worldgen/biome"),
            "damage type" => Some("minecraft:damage_type"),
            "entity type" => Some("minecraft:entity_type"),
            "sound" => Some("minecraft:sound_event"),
            "particle" => Some("minecraft:particle_type"),
            "attribute" => Some("minecraft:attribute"),
            "effect" => Some("minecraft:mob_effect"),
            "potion" => Some("minecraft:potion"),
            _ => None,
        };
        if id.starts_with("minecraft:")
            && !(kind == "item" && self.custom_items.contains(id))
            && let Some(registry) = registry
            && let Some(false) = sand_version::registry::contains(self.target, registry, id)
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_TARGET",
                owner,
                ptr,
                format!("Minecraft {} has no {kind} `{id}`", self.target),
            ));
        }

        if id.parse::<ResourceLocation>().is_err() || !id.contains(':') {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_ID",
                owner,
                ptr,
                format!("`{id}` is not a namespaced {kind} identity such as `minecraft:example`"),
            ));
        }
    }

    fn block_selector(&mut self, selector: &str, owner: &str, ptr: &str) {
        if let Some(tag) = selector.strip_prefix('#') {
            self.resource(tag, owner, ptr, "block tag");
        } else {
            self.resource(selector, owner, ptr, "block");
        }
    }

    fn field_context(
        &mut self,
        score: &FieldReference,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        if context == ExecutionContext::Server
            && self
                .states
                .get(&score.state.to_string())
                .is_some_and(|(scope, _)| *scope == "entity")
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_CONTEXT",
                owner,
                ptr,
                "entity State requires a player or entity executor",
            ));
        }
        if context != ExecutionContext::Player
            && self
                .states
                .get(&score.state.to_string())
                .is_some_and(|(scope, _)| *scope == "player")
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_CONTEXT",
                owner,
                ptr,
                "player-self State requires player execution context",
            ));
        }
    }

    fn score(&mut self, score: &FieldReference, context: ExecutionContext, owner: &str, ptr: &str) {
        self.field_context(score, context, owner, ptr);
        if !self
            .states
            .get(&score.state.to_string())
            .is_some_and(|(_, fields)| fields.contains(score.field.as_str()))
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                ptr,
                "unresolved State field",
            ));
        }
    }
    fn data_reference(
        &mut self,
        reference: &DataReference,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
        each: bool,
    ) {
        self.field_context(&reference.field, context, owner, ptr);
        if !self
            .data_fields
            .get(&reference.field.state.to_string())
            .is_some_and(|fields| fields.contains(reference.field.field.as_str()))
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                ptr,
                "unresolved State storage field",
            ));
        }
        if reference.path.len() > 32 {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_LIMIT",
                owner,
                ptr,
                "storage projections support at most 32 steps",
            ));
            return;
        }
        for (index, projection) in reference.path.iter().enumerate() {
            let location = format!("{ptr}/path/{index}");
            match projection {
                DataProjection::Member { name } => {
                    if name.is_empty()
                        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &location,
                            "storage members require nonempty simple names",
                        ));
                    }
                }
                DataProjection::Index {
                    index: ScoreOperand::Field(score),
                } => self.score(score, context, owner, &location),
                DataProjection::Index {
                    index: ScoreOperand::Value(value),
                } if *value < 0 => {
                    self.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_RANGE",
                        owner,
                        &location,
                        "storage indices must be nonnegative",
                    ));
                }
                DataProjection::Each if !each || index + 1 != reference.path.len() => {
                    self.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_REFERENCE",
                        owner,
                        &location,
                        "all-elements selection is only valid at the end of a modification source",
                    ));
                }
                _ => {}
            }
        }
    }

    fn data_operand(
        &mut self,
        value: &DataOperand,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        match value {
            DataOperand::Value { value } => self.data_literal(value, owner, ptr),
            DataOperand::Reference { value } => {
                self.data_reference(value, context, owner, ptr, true)
            }
            DataOperand::Score {
                value: ScoreOperand::Field(score),
            } => self.score(score, context, owner, ptr),
            DataOperand::Score { .. } | DataOperand::Block { .. } => {}
        }
    }

    fn data_literal(&mut self, value: &sand_commands::NbtValue, owner: &str, ptr: &str) {
        let profile = sand_commands::CommandProfile::new(self.target, false);
        let command = sand_commands::DataTarget::storage("sand:validate")
            .path("value")
            .set(value.clone());
        if serde_json::to_value(value).is_err() {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_STATE",
                owner,
                ptr,
                "portable data requires typed NBT; raw SNBT is unsupported",
            ));
        } else if let Err(error) = command.try_render(&profile) {
            self.errors
                .push(Diagnostic::error("SAND_PROGRAM_STATE", owner, ptr, error));
        }
    }

    fn call(
        &mut self,
        reference: &FunctionReference,
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
    ) {
        let (id, requirement) = match reference {
            FunctionReference::Internal { id } => {
                (id, self.functions.get(&id.to_string()).copied())
            }
            FunctionReference::External { id, context } => {
                if self.functions.contains_key(&id.to_string()) {
                    self.errors.push(Diagnostic::error(
                        "SAND_PROGRAM_REFERENCE",
                        owner,
                        ptr,
                        "external function reference resolves to a declared internal function",
                    ));
                }
                (id, Some(*context))
            }
        };
        if !valid_id(id) || id.path().starts_with("__sand_") {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                ptr,
                "invalid or compiler-private function reference",
            ));
        }
        match requirement {
            None => self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_REFERENCE",
                owner,
                ptr,
                format!("unresolved internal function {id}"),
            )),
            Some(ExecutionContext::Player) if context != ExecutionContext::Player => {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_CONTEXT",
                    owner,
                    ptr,
                    "callee requires a player executor",
                ))
            }
            Some(ExecutionContext::Entity) if context == ExecutionContext::Server => {
                self.errors.push(Diagnostic::error(
                    "SAND_PROGRAM_CONTEXT",
                    owner,
                    ptr,
                    "callee requires an entity executor",
                ))
            }
            _ => {}
        }
    }
    fn body(
        &mut self,
        body: &[Operation],
        context: ExecutionContext,
        owner: &str,
        ptr: &str,
        depth: usize,
    ) {
        if depth > limits::OP_DEPTH
            || self.operations.saturating_add(body.len()) > limits::OPERATIONS
        {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_LIMIT",
                owner,
                ptr,
                "operation depth/count limit exceeded",
            ));
            return;
        }
        self.operations += body.len();
        for (index, operation) in body.iter().enumerate() {
            let ptr = format!("{ptr}/{index}");
            let start = self.errors.len();
            let mut origin_valid = true;
            if let Some(origin) = &operation.origin {
                let mut pending = vec![(origin, 0)];
                while let Some((value, depth)) = pending.pop() {
                    if depth > limits::JSON_DEPTH {
                        origin_valid = false;
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_LIMIT",
                            owner,
                            &ptr,
                            "origin nesting limit exceeded",
                        ));
                        break;
                    }
                    match value {
                        serde_json::Value::Array(values) => {
                            pending.extend(values.iter().map(|value| (value, depth + 1)))
                        }
                        serde_json::Value::Object(values) => {
                            pending.extend(values.values().map(|value| (value, depth + 1)))
                        }
                        _ => {}
                    }
                }
            }
            match &operation.action {
                Action::DataModify { target, source, .. } => {
                    self.data_reference(
                        target,
                        context,
                        owner,
                        &format!("{ptr}/action/target"),
                        false,
                    );
                    self.data_operand(source, context, owner, &format!("{ptr}/action/source"));
                }
                Action::DataRemove { target } => self.data_reference(
                    target,
                    context,
                    owner,
                    &format!("{ptr}/action/target"),
                    false,
                ),
                Action::DataRead { score, source } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    self.data_reference(
                        source,
                        context,
                        owner,
                        &format!("{ptr}/action/source"),
                        false,
                    );
                }
                Action::DataEqual { score, left, right } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    self.data_reference(left, context, owner, &format!("{ptr}/action/left"), false);
                    self.data_reference(
                        right,
                        context,
                        owner,
                        &format!("{ptr}/action/right"),
                        false,
                    );
                }
                Action::ScoreSet { score, .. } | Action::ScoreAdd { score, .. } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"))
                }
                Action::Call { function } => {
                    self.call(function, context, owner, &format!("{ptr}/action/function"))
                }
                Action::Players { body } => self.body(
                    body,
                    ExecutionContext::Player,
                    owner,
                    &format!("{ptr}/action/body"),
                    depth + 1,
                ),
                Action::Branch {
                    condition,
                    then,
                    otherwise,
                } => {
                    let condition_ptr = format!("{ptr}/action/condition");
                    self.condition(condition, context, owner, &condition_ptr);
                    self.body(
                        then,
                        context,
                        owner,
                        &format!("{ptr}/action/then"),
                        depth + 1,
                    );
                    self.body(
                        otherwise,
                        context,
                        owner,
                        &format!("{ptr}/action/otherwise"),
                        depth + 1,
                    );
                }
                Action::ScoreOperation {
                    score,
                    operator,
                    operand,
                } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    match operand {
                        ScoreOperand::Field(field) => {
                            self.score(field, context, owner, &format!("{ptr}/action/operand"))
                        }
                        ScoreOperand::Value(0)
                            if matches!(operator, ScoreOperator::Div | ScoreOperator::Mod) =>
                        {
                            self.errors.push(Diagnostic::error(
                                "SAND_PROGRAM_RANGE",
                                owner,
                                &format!("{ptr}/action/operand"),
                                "division or modulo by the constant zero has no effect",
                            ));
                        }
                        ScoreOperand::Value(_) => {}
                    }
                }
                Action::ScoreRandom { score, min, max } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    if min > max {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action"),
                            "random minimum exceeds maximum",
                        ));
                    }
                }
                Action::Message {
                    audience,
                    channel,
                    text,
                } => {
                    if *audience == Audience::Player {
                        self.player(context, owner, &ptr, "a message to the executing player");
                    }
                    if text.is_empty() {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_TEXT",
                            owner,
                            &format!("{ptr}/action/text"),
                            "message text must contain at least one part",
                        ));
                    }
                    let text_ptr = format!("{ptr}/action/text");
                    if let Some(error) =
                        text_shape_error(text, 0, *channel == MessageChannel::Chat, None)
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_TEXT",
                            owner,
                            &text_ptr,
                            error,
                        ));
                    } else {
                        self.message_parts(text, context, owner, &text_ptr);
                    }
                }
                Action::Dialog { dialog } => {
                    self.player(context, owner, &ptr, "showing or clearing a dialog");
                    if !self.dialogs_supported {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_TARGET",
                            owner,
                            &ptr,
                            format!("Minecraft {} does not support native dialogs", self.target),
                        ));
                    }
                    if let Some(dialog) = dialog {
                        self.resource(
                            &dialog.to_string(),
                            owner,
                            &format!("{ptr}/action/dialog"),
                            "dialog",
                        );
                    }
                }
                Action::Sound {
                    sound,
                    volume,
                    pitch,
                } => {
                    self.player(context, owner, &ptr, "a sound for the executing player");
                    self.resource(sound, owner, &format!("{ptr}/action/sound"), "sound");
                    self.amount(
                        volume,
                        context,
                        owner,
                        &format!("{ptr}/action/volume"),
                        Some((0.0, f32::MAX as f64)),
                    );
                    self.amount(
                        pitch,
                        context,
                        owner,
                        &format!("{ptr}/action/pitch"),
                        Some((0.0, 2.0)),
                    );
                }
                Action::Particle {
                    particle,
                    count,
                    spread,
                    speed,
                } => {
                    self.resource(
                        particle,
                        owner,
                        &format!("{ptr}/action/particle"),
                        "particle",
                    );
                    for (name, value) in [("spread", spread), ("speed", speed)] {
                        self.amount(
                            value,
                            context,
                            owner,
                            &format!("{ptr}/action/{name}"),
                            Some((0.0, 1000.0)),
                        );
                    }
                    let count_path = format!("{ptr}/action/count");
                    match count {
                        ScoreOperand::Value(value) if !(0..=100_000).contains(value) => {
                            self.errors.push(Diagnostic::error(
                                "SAND_PROGRAM_RANGE",
                                owner,
                                &count_path,
                                "particle count must be from 0 to 100000",
                            ));
                        }
                        ScoreOperand::Field(score) => {
                            self.score(score, context, owner, &count_path)
                        }
                        _ => {}
                    }
                }
                Action::Effect {
                    effect, seconds, ..
                } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "a status effect on the executing player",
                    );
                    self.resource(effect, owner, &format!("{ptr}/action/effect"), "effect");
                    if seconds.is_some_and(|s| s == 0 || s > 1_000_000) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/seconds"),
                            "effect duration must be 1..=1000000 seconds; omit it for infinite",
                        ));
                    }
                }
                Action::ClearEffect { effect } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "clearing the executing player's effects",
                    );
                    if let Some(effect) = effect {
                        self.resource(effect, owner, &format!("{ptr}/action/effect"), "effect");
                    }
                }
                Action::GiveItem { item, count } => {
                    self.player(context, owner, &ptr, "giving items to the executing player");
                    self.resource(item, owner, &format!("{ptr}/action/item"), "item");
                    if let ScoreOperand::Field(score) = count {
                        self.score(score, context, owner, &format!("{ptr}/action/count"));
                    }
                    if matches!(count, ScoreOperand::Value(count) if *count < 1 || *count > 6400) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/count"),
                            "item count must be 1..=6400",
                        ));
                    }
                }
                Action::SetEquipment { slot, item, count } => {
                    self.executor(context, owner, &ptr, "changing equipment");
                    self.equipment_slot(*slot, owner, &format!("{ptr}/action/slot"));
                    if let Some(item) = item
                        && !self.custom_items.contains(item)
                    {
                        self.resource(item, owner, &format!("{ptr}/action/item"), "item");
                    }
                    if count.is_some_and(|count| !(1..=99).contains(&count))
                        || (item.is_none() && count.is_some())
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/count"),
                            "equipment stack count requires an item and must be 1..=99",
                        ));
                    }
                }
                Action::ClearItem { item, count } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "clearing the executing player's items",
                    );
                    if let Some(item) = item {
                        self.resource(item, owner, &format!("{ptr}/action/item"), "item");
                    }
                    let pointer = format!("{ptr}/action/count");
                    if count.is_some() && item.is_none() {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RESOURCE",
                            owner,
                            &pointer,
                            "an item-removal count requires an item",
                        ));
                    }
                    match count {
                        Some(ScoreOperand::Field(score)) => {
                            self.score(score, context, owner, &pointer);
                        }
                        Some(ScoreOperand::Value(count)) if *count < 0 => {
                            self.errors.push(Diagnostic::error(
                                "SAND_PROGRAM_RANGE",
                                owner,
                                &pointer,
                                "item-removal count must be nonnegative",
                            ));
                        }
                        _ => {}
                    }
                }
                Action::Menu { menu, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "opening or closing the executing player's menu",
                    );
                    if !self.menus.contains(&menu.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RESOURCE",
                            owner,
                            &format!("{ptr}/action/menu"),
                            format!("unknown menu `{menu}`"),
                        ));
                    }
                }
                Action::Progression { progression, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "advancing or resetting the executing player's progression",
                    );
                    if !self.progressions.contains(&progression.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/progression"),
                            format!("unknown progression `{progression}`"),
                        ));
                    }
                }
                Action::Combat { combat, amount, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "applying custom player health changes",
                    );
                    if !self.combats.contains(&combat.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/combat"),
                            format!("unknown combat policy `{combat}`"),
                        ));
                    }
                    let location = format!("{ptr}/action/amount");
                    self.amount(amount, context, owner, &location, Some((0.0, f64::MAX)));
                    if let Amount::Field(field) = amount
                        && (field.scale.fract() != 0.0 || field.scale > f64::from(i32::MAX))
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &location,
                            "combat score scale must be a positive 32-bit integer",
                        ));
                    }
                }
                Action::Ability { ability, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "controlling the executing player's ability",
                    );
                    if !self.abilities.contains(&ability.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/ability"),
                            format!("unknown ability `{ability}`"),
                        ));
                    }
                }
                Action::Scene { scene, .. } => {
                    self.player(context, owner, &ptr, "showing or cancelling an owned scene");
                    if !self.scenes.contains(&scene.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/scene"),
                            format!("unknown scene `{scene}`"),
                        ));
                    }
                }
                Action::Job { job, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "controlling the executing player's job",
                    );
                    if !self.jobs.contains(&job.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/job"),
                            format!("unknown job `{job}`"),
                        ));
                    }
                }
                Action::Session { session, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "controlling the executing player's session",
                    );
                    if !self.sessions.contains(&session.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/session"),
                            format!("unknown session `{session}`"),
                        ));
                    }
                }
                Action::Projectile { projectile, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "launching or cancelling the executing player's projectile",
                    );
                    if !self.projectiles.contains(&projectile.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RESOURCE",
                            owner,
                            &format!("{ptr}/action/projectile"),
                            format!("unknown projectile `{projectile}`"),
                        ));
                    }
                }
                Action::Hud { hud, .. } => {
                    self.player(context, owner, &ptr, "showing the executing player's HUD");
                    if !self.huds.contains(&hud.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RESOURCE",
                            owner,
                            &format!("{ptr}/action/hud"),
                            format!("unknown HUD `{hud}`"),
                        ));
                    }
                }
                Action::TransferInventory { transfer } => {
                    self.player(context, owner, &ptr, "moving the executing player's items");
                    if !self.transfers.contains(&transfer.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RESOURCE",
                            owner,
                            &format!("{ptr}/action/transfer"),
                            format!("unknown inventory transfer `{transfer}`"),
                        ));
                    }
                }
                Action::CountItems { item, score } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "counting the executing player's items",
                    );
                    self.resource(item, owner, &format!("{ptr}/action/item"), "item");
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                }
                Action::Schedule { function, ticks } => {
                    // Scheduled functions run later on the server, without an executor.
                    self.call(
                        function,
                        ExecutionContext::Server,
                        owner,
                        &format!("{ptr}/action/function"),
                    );
                    if *ticks == 0 {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/ticks"),
                            "schedule delay must be at least one tick",
                        ));
                    }
                }
                Action::Return => {}
                Action::Suspend {
                    ticks,
                    fields,
                    body,
                    group,
                    server,
                } => {
                    if !server {
                        self.executor(context, owner, &ptr, "suspending a function");
                    } else if context != ExecutionContext::Server {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_CONTEXT",
                            owner,
                            &ptr,
                            "a server wait must run in server context",
                        ));
                    }
                    self.wait_group(group.as_deref(), owner, &ptr);
                    match ticks {
                        ScoreOperand::Field(score) => {
                            self.score(score, context, owner, &format!("{ptr}/action/ticks"))
                        }
                        ScoreOperand::Value(ticks) if *ticks < 1 => {
                            self.errors.push(Diagnostic::error(
                                "SAND_PROGRAM_RANGE",
                                owner,
                                &format!("{ptr}/action/ticks"),
                                "wait delay must be a positive scoreboard integer",
                            ));
                        }
                        ScoreOperand::Value(_) => {}
                    }
                    if fields.len() > 64 {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_LIMIT",
                            owner,
                            &format!("{ptr}/action/fields"),
                            "at most 64 local slots can be retained by a wait",
                        ));
                    }
                    self.frame_fields(
                        fields,
                        context,
                        owner,
                        &format!("{ptr}/action/fields"),
                        true,
                    );
                    self.body(
                        body,
                        context,
                        owner,
                        &format!("{ptr}/action/body"),
                        depth + 1,
                    );
                }
                Action::CancelWaits { group, server } => {
                    if !server {
                        self.executor(context, owner, &ptr, "cancelling a wait");
                    }
                    self.wait_group(group.as_deref(), owner, &ptr);
                }
                Action::While { condition, body } => {
                    let condition_ptr = format!("{ptr}/action/condition");
                    self.condition(condition, context, owner, &condition_ptr);
                    self.body(
                        body,
                        context,
                        owner,
                        &format!("{ptr}/action/body"),
                        depth + 1,
                    );
                }
                Action::AttributeGet {
                    score,
                    attribute,
                    scale,
                } => {
                    self.executor(context, owner, &ptr, "reading the executor's attribute");
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    self.resource(
                        attribute,
                        owner,
                        &format!("{ptr}/action/attribute"),
                        "attribute",
                    );
                    if !(scale.is_finite() && *scale > 0.0) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/scale"),
                            "attribute scale must be finite and positive",
                        ));
                    }
                }
                Action::AttributeSet { attribute, value } => {
                    self.executor(context, owner, &ptr, "setting the executor's attribute");
                    self.resource(
                        attribute,
                        owner,
                        &format!("{ptr}/action/attribute"),
                        "attribute",
                    );
                    let valid = match value {
                        Amount::Value(value) => value.is_finite(),
                        Amount::Field(field) => {
                            self.score(
                                &field.score,
                                context,
                                owner,
                                &format!("{ptr}/action/value/score"),
                            );
                            field.scale.is_finite() && field.scale > 0.0
                        }
                    };
                    if !valid {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/value"),
                            "attribute values must be finite and scales positive",
                        ));
                    }
                }
                Action::Entities { query, body } => {
                    self.query(query, owner, &format!("{ptr}/action/query"));
                    self.body(
                        body,
                        ExecutionContext::Entity,
                        owner,
                        &format!("{ptr}/action/body"),
                        depth + 1,
                    );
                }
                Action::CaptureEntity { score, relation } => {
                    if *relation == Some(crate::entity::Relation::Passengers) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_TARGET",
                            owner,
                            &format!("{ptr}/action/relation"),
                            "retaining an entity requires a single-entity relationship",
                        ));
                    }
                    if self
                        .identity_unsafe
                        .contains(&format!("{}.{}", score.state, score.field))
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_STATE",
                            owner,
                            &format!("{ptr}/action/score"),
                            "entity identities require an unbounded, non-timer field",
                        ));
                    }
                    self.executor(context, owner, &ptr, "capturing an entity identity");
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                }
                Action::WithEntity {
                    reference,
                    players,
                    body,
                } => {
                    self.score(
                        reference,
                        context,
                        owner,
                        &format!("{ptr}/action/reference"),
                    );
                    self.body(
                        body,
                        if *players {
                            ExecutionContext::Player
                        } else {
                            ExecutionContext::Entity
                        },
                        owner,
                        &format!("{ptr}/action/body"),
                        depth + 1,
                    );
                }
                Action::Kill => self.executor(context, owner, &ptr, "killing the executor"),
                Action::Ride { vehicle } => {
                    self.executor(context, owner, &ptr, "changing the executor's vehicle");
                    if let Some(vehicle) = vehicle {
                        self.score(vehicle, context, owner, &format!("{ptr}/action/vehicle"));
                    }
                }
                Action::PushFrame { fields } | Action::PopFrame { fields } => {
                    if fields.is_empty() {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_STATE",
                            owner,
                            &format!("{ptr}/action/fields"),
                            "a call frame saves at least one field",
                        ));
                    }
                    self.frame_fields(
                        fields,
                        context,
                        owner,
                        &format!("{ptr}/action/fields"),
                        false,
                    );
                }
                Action::DataGet {
                    score,
                    value,
                    scale,
                } => {
                    if value.player_only() {
                        self.player(context, owner, &ptr, "reading player data");
                    } else {
                        self.executor(context, owner, &ptr, "reading entity data");
                    }
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    if !(scale.is_finite() && *scale > 0.0) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/scale"),
                            "data scale must be finite and positive",
                        ));
                    }
                }
                Action::Damage {
                    amount,
                    by,
                    damage_type,
                } => {
                    if let Some(damage_type) = damage_type {
                        self.resource(
                            &damage_type.to_string(),
                            owner,
                            &format!("{ptr}/action/damage_type"),
                            "damage type",
                        );
                    }
                    if let Some(by) = by {
                        self.score(by, context, owner, &format!("{ptr}/action/by"));
                    }
                    self.executor(context, owner, &ptr, "damaging the executor");
                    self.amount(
                        amount,
                        context,
                        owner,
                        &format!("{ptr}/action/amount"),
                        Some((0.01, 1_000_000.0)),
                    );
                }
                Action::EntityTag { tag, .. } => {
                    self.executor(context, owner, &ptr, "tagging the executor");
                    self.tag_name(tag, owner, &format!("{ptr}/action/tag"));
                }
                Action::Summon { entity_type, tag } => {
                    self.resource(
                        entity_type,
                        owner,
                        &format!("{ptr}/action/entity_type"),
                        "entity type",
                    );
                    if let Some(tag) = tag {
                        self.tag_name(tag, owner, &format!("{ptr}/action/tag"));
                    }
                }
                Action::CountEntities { score, query } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                    self.query(query, owner, &format!("{ptr}/action/query"));
                }
                Action::SetVelocity { x, y, z } => {
                    self.executor(context, owner, &ptr, "setting entity velocity");
                    for (axis, value) in [("x", x), ("y", y), ("z", z)] {
                        self.amount(
                            value,
                            context,
                            owner,
                            &format!("{ptr}/action/{axis}"),
                            Some((-10.0, 10.0)),
                        );
                    }
                }
                Action::Teleport { destination } => {
                    self.executor(context, owner, &ptr, "teleporting the executor");
                    let finite = match destination {
                        TeleportDestination::Position { x, y, z }
                        | TeleportDestination::Offset { x, y, z } => {
                            let x = self.amount(
                                x,
                                context,
                                owner,
                                &format!("{ptr}/action/destination/x"),
                                None,
                            );
                            let y = self.amount(
                                y,
                                context,
                                owner,
                                &format!("{ptr}/action/destination/y"),
                                None,
                            );
                            let z = self.amount(
                                z,
                                context,
                                owner,
                                &format!("{ptr}/action/destination/z"),
                                None,
                            );
                            x && y && z
                        }
                        TeleportDestination::Entity { query } => {
                            self.query(query, owner, &format!("{ptr}/action/destination/query"));
                            true
                        }
                    };
                    if !finite {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/destination"),
                            "teleport coordinates must be finite",
                        ));
                    }
                }
                Action::AttachState { state } | Action::DetachState { state } => {
                    self.attachable_state(state, context, owner, &format!("{ptr}/action/state"))
                }
                Action::SetBlock { block, .. } | Action::FillBlocks { block, .. } => {
                    self.resource(block, owner, &format!("{ptr}/action/block"), "block");
                }
                Action::CloneBlocks { masked, filter, .. } => {
                    if let Some(filter) = filter {
                        self.block_selector(filter, owner, &format!("{ptr}/action/filter"));
                        if *masked {
                            self.errors.push(Diagnostic::error(
                                "SAND_PROGRAM_TARGET",
                                owner,
                                &ptr,
                                "choose either a clone filter or a masked clone",
                            ));
                        }
                    }
                }
                Action::PlaceTemplate { template, .. } => {
                    self.resource(
                        &template.to_string(),
                        owner,
                        &format!("{ptr}/action/template"),
                        "structure template",
                    );
                }
                Action::InDimension { dimension, body } => {
                    self.resource(
                        &dimension.to_string(),
                        owner,
                        &format!("{ptr}/action/dimension"),
                        "dimension",
                    );
                    self.body(
                        body,
                        context,
                        owner,
                        &format!("{ptr}/action/body"),
                        depth + 1,
                    );
                }
                Action::ReadEntityData { score, .. } => {
                    self.executor(context, owner, &ptr, "reading entity data");
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                }
                Action::ReadBlockData { score, .. } => {
                    self.score(score, context, owner, &format!("{ptr}/action/score"));
                }
                Action::WriteBlockData { value, .. } => {
                    if let Some(value) = value {
                        self.data_operand(value, context, owner, &format!("{ptr}/action/value"));
                    }
                }
                Action::AddExperience { amount, .. } => {
                    self.player(context, owner, &ptr, "giving experience");
                    if let ScoreOperand::Field(score) = amount {
                        self.score(score, context, owner, &format!("{ptr}/action/amount"));
                    }
                }
                Action::SetGameMode { .. } => {
                    self.player(context, owner, &ptr, "changing game mode");
                }
                Action::SetTime { .. } => {}
                Action::SetWeather { seconds, .. } => {
                    if *seconds == Some(0) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/seconds"),
                            "weather duration must be positive",
                        ));
                    }
                }
                Action::BossBarAudience { bar, audience, .. } => {
                    self.boss_bar(bar, owner, &format!("{ptr}/action/bar"));
                    if *audience == Audience::Player {
                        self.player(context, owner, &ptr, "showing a boss bar to one player");
                    }
                }
                Action::SetBossBar { bar, value, .. } => {
                    self.boss_bar(bar, owner, &format!("{ptr}/action/bar"));
                    if let ScoreOperand::Field(field) = value {
                        self.score(field, context, owner, &format!("{ptr}/action/value"));
                    }
                }
                Action::GiveLoot { table } | Action::DropLoot { table } => {
                    if !self.loot_tables.contains(&table.to_string()) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_REFERENCE",
                            owner,
                            &format!("{ptr}/action/table"),
                            format!("`{table}` is not a declared loot table"),
                        ));
                    }
                    if matches!(operation.action, Action::GiveLoot { .. }) {
                        self.player(context, owner, &ptr, "giving loot");
                    }
                }
                Action::Raw { command } => {
                    if !self.raw {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_CAPABILITY",
                            owner,
                            &ptr,
                            "raw operation requires explicit raw_commands capability",
                        ));
                    }
                    if command.is_empty()
                        || command.contains(['\r', '\n', '\0'])
                        || command.starts_with('/')
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RAW",
                            owner,
                            &format!("{ptr}/action/command"),
                            "raw command must be one nonempty command line without a leading slash",
                        ));
                    }
                }
            }
            for error in &mut self.errors[start..] {
                if error.origin.is_none() && origin_valid {
                    error.origin = operation.origin.clone();
                }
            }
        }
    }
}
