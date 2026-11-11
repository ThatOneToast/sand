//! Typed gameplay lowering with local, deterministic helper allocation.
use super::{diagnostic::Diagnostic, limits, model::*};
use crate::compiler::export::{
    functions, identities, lifecycle,
    records::{self, ComponentRecord},
    tags,
};
use crate::ir::{Cmd, NumericWrite};
use std::collections::BTreeMap;

/// Portable resource bytes and the same runtime provenance used by Rust export.
pub(super) struct LoweredProgram {
    pub(super) resources: BTreeMap<String, Vec<u8>>,
    pub(super) runtime_report: crate::compiler::runtime::RuntimeReport,
}

pub(super) fn lower(
    program: &Program,
    target: &crate::version::VersionProfile,
) -> Result<LoweredProgram, Vec<Diagnostic>> {
    let _registry =
        crate::function::ExportFunctionRegistryScope::enter_namespace(&program.pack.namespace);
    let error = |error: String| vec![Diagnostic::error("SAND_PROGRAM_LOWER", "", "", error)];
    let states: Vec<_> = program
        .modules
        .iter()
        .flat_map(|module| module.states.iter().cloned())
        .collect();
    let mut automatic = crate::state::owned::lifecycle(&states).map_err(|message| {
        // The canonical Rust lifecycle API reports its logical owner in the
        // message. Retain a portable source location for that owner rather
        // than losing the module boundary when adapting its diagnostic.
        for module in &program.modules {
            for (index, state) in module.states.iter().enumerate() {
                if message.contains(&state.id.to_string()) {
                    return vec![Diagnostic::error(
                        "SAND_PROGRAM_STATE",
                        &module.id.to_string(),
                        &format!("/states/{index}"),
                        message,
                    )];
                }
            }
        }
        error(message)
    })?;
    for module in &program.modules {
        for owners in [
            &mut automatic.load_origins,
            &mut automatic.init_origins,
            &mut automatic.tick_origins,
        ] {
            if module
                .states
                .iter()
                .any(|state| owners.contains(&state.id.to_string()))
            {
                owners.insert(module.id.to_string());
            }
        }
    }
    let init = (!automatic.player_init_commands.is_empty())
        .then(|| format!("{}:__sand_lifecycle_init", program.pack.namespace));
    let owners: Vec<String> = program
        .modules
        .iter()
        .flat_map(|module| {
            module
                .functions
                .iter()
                .map(|function| function.id.to_string())
                .chain([
                    format!("module/{}/load", module.id),
                    format!("module/{}/tick", module.id),
                ])
        })
        .collect();
    let helper_keys = identities::allocate_collision_safe_keys(
        owners.iter().map(String::as_str),
        |owner, attempt| {
            sand_commands::scoreboard::hash_objective_name(&format!("{owner}#{attempt}"))
        },
        |_, _| Ok(()),
        |owner, previous, key| {
            vec![Diagnostic::error(
                "SAND_PROGRAM_COLLISION",
                "",
                "",
                format!("portable helper key `{key}` collides between `{owner}` and `{previous}`"),
            )]
        },
    )?;
    let mut lower = Lower {
        namespace: program.pack.namespace.clone(),
        holders: states
            .iter()
            .map(|state| (state.id.to_string(), crate::state::owned::holder(state)))
            .collect(),
        bounds: states
            .iter()
            .flat_map(|state| {
                state.fields.iter().filter_map(|field| {
                    field
                        .bounds
                        .map(|bounds| (format!("{}.{}", state.id, field.name), bounds))
                })
            })
            .collect(),
        helper_keys,
        runtime: crate::compiler::runtime::RuntimePlan::default(),
        records: Vec::new(),
        init,
        bytes: 0,
        profile: sand_commands::CommandProfile::new(target.resolved_name(), false),
        constants: std::collections::BTreeSet::new(),
        custom_items: BTreeMap::new(),
        flags: std::collections::BTreeSet::new(),
        entity_states: states
            .iter()
            .filter(|state| state.scope == "entity")
            .map(|state| {
                (
                    state.id.to_string(),
                    state
                        .fields
                        .iter()
                        .map(|field| (field.name.clone(), field.default))
                        .collect(),
                )
            })
            .collect(),
    };
    for module in &program.modules {
        for item in &module.items {
            let custom = custom_item(item, &lower.holders);
            lower.custom_items.insert(item.id.to_string(), custom);
        }
    }
    let mut entries = BTreeMap::new();
    let mut load = Vec::new();
    let mut tick = Vec::new();
    let mut memberships = Vec::new();
    let mut modules: Vec<_> = program.modules.iter().collect();
    modules.sort_by_key(|module| module.id.to_string());
    for module in modules {
        let owner = module.id.to_string();
        let mut functions: Vec<_> = module.functions.iter().enumerate().collect();
        functions.sort_by_key(|(_, function)| function.id.to_string());
        for (index, function) in functions {
            let body = lower.body(
                &function.body,
                &function.id.to_string(),
                "",
                &owner,
                &format!("/functions/{index}/body"),
            )?;
            lower.function(
                &function.id.to_string(),
                body,
                &owner,
                &format!("/functions/{index}"),
            )?;
            entries.insert(function.id.to_string(), function.context);
        }
        for (phase, body, output) in [
            ("load", &module.load, &mut load),
            ("tick", &module.tick, &mut tick),
        ] {
            // Module identity owns lifecycle contribution order and helper IDs.
            let body = lower.body(
                body,
                &format!("module/{owner}/{phase}"),
                "",
                &owner,
                &format!("/{phase}"),
            )?;
            output.extend(
                body.into_iter()
                    .enumerate()
                    .map(|(index, line)| (owner.clone(), index, line)),
            );
        }
        for tag in &module.tags {
            let tag_id = tag.tag.to_string();
            let function = reference_id(&tag.function);
            lower
                .runtime
                .contribute_tag(tag_id.clone(), function.clone(), owner.clone());
            memberships.push((tag_id, function));
        }
    }
    let event_owner = format!("{}:__sand_program/events", program.pack.namespace);
    let mut event_load = Vec::new();
    for (index, line) in lower.events(program)?.into_iter().enumerate() {
        match line {
            EventLine::Load(command) => event_load.push((event_owner.clone(), index, command)),
            EventLine::Tick(command) => tick.push((event_owner.clone(), index, command)),
        }
    }
    if !lower.constants.is_empty() {
        let objective = constants_objective(&program.pack.namespace);
        event_load.push((
            event_owner.clone(),
            event_load.len(),
            format!("scoreboard objectives add {objective} dummy"),
        ));
        for value in lower.constants.clone() {
            event_load.push((
                event_owner.clone(),
                event_load.len(),
                format!(
                    "scoreboard players set {} {objective} {value}",
                    constant_holder(value)
                ),
            ));
        }
    }
    for module in &program.modules {
        for bar in &module.boss_bars {
            for line in lower.boss_bar_setup(bar) {
                event_load.push((event_owner.clone(), event_load.len(), line));
            }
        }
    }
    // Objectives must exist before any module load body reads them.
    load.splice(0..0, event_load);
    let field_defaults: BTreeMap<_, _> = states
        .iter()
        .flat_map(|state| {
            state
                .fields
                .iter()
                .map(|field| (format!("{}.{}", state.id, field.name), field.default))
        })
        .collect();
    let mut flow_locations = BTreeMap::new();
    let mut flows = Vec::new();
    for module in &program.modules {
        for (index, flow) in module.flows.iter().enumerate() {
            flow_locations.insert(
                flow.id.to_string(),
                (module.id.to_string(), format!("/flows/{index}")),
            );
            flows.push(flow_registration(flow, &lower.holders, &field_defaults));
        }
    }
    let flow_plan = crate::state::flow::resolve_state_flow_plan(&program.pack.namespace, flows)
        .map_err(|message| {
            let location = flow_locations
                .iter()
                .find(|(id, _)| message.contains(&format!("Flow: {id}\n")));
            let (module, ptr) = location
                .map(|(_, (module, ptr))| (module.as_str(), ptr.as_str()))
                .unwrap_or(("", ""));
            vec![Diagnostic::error("SAND_PROGRAM_FLOW", module, ptr, message)]
        })?;
    lifecycle::validate_private_objectives(&automatic, &load, &flow_plan.private_objectives)
        .map_err(error)?;
    for generated in flow_plan.functions {
        let (module, ptr) = &flow_locations[&generated.tracker_id];
        let id = format!("{}:{}", program.pack.namespace, generated.path);
        lower.function(&id, generated.commands, module, ptr)?;
        lower.runtime.generated_function(id, generated.origins);
    }
    automatic.load_commands.extend(flow_plan.load_commands);
    for (id, (module, _)) in &flow_locations {
        automatic.load_origins.extend([id.clone(), module.clone()]);
    }
    if !flow_locations.is_empty() {
        lower.runtime.generated_lifecycle(
            format!("{}:__sand_lifecycle_tick", program.pack.namespace),
            "tick",
            flow_locations
                .iter()
                .flat_map(|(id, (module, _))| [id.clone(), module.clone()])
                .map(crate::function::GeneratedOrigin::Logical),
        );
    }
    let mut tag_map = BTreeMap::new();
    let lifecycle = lifecycle::assemble_lifecycle(
        &program.pack.namespace,
        &mut lower.records,
        &mut tag_map,
        automatic,
        load,
        tick,
        flow_plan.global_tick_commands,
    )
    .map_err(|e| error(e.to_string()))?;
    lifecycle.attribute(&program.pack.namespace, &mut lower.runtime);
    if !lifecycle.load_owners.is_empty() {
        lower.runtime.lifecycle_owners.insert(
            format!("{}:__sand_lifecycle_load", program.pack.namespace),
            lifecycle.load_owners,
        );
    }
    if !lifecycle.tick_owners.is_empty() {
        lower.runtime.lifecycle_owners.insert(
            format!("{}:__sand_lifecycle_tick", program.pack.namespace),
            lifecycle.tick_owners,
        );
    }
    lifecycle::initialize_player_entries(
        &mut lower.records,
        &entries,
        lifecycle.player_initializer.as_deref(),
    );
    tags::assemble_tags(
        &program.pack.namespace,
        &mut lower.records,
        tag_map,
        memberships,
    );
    functions::drain_generated_resources_into(
        &mut lower.records,
        &program.pack.namespace,
        None,
        &mut lower.runtime,
    )
    .map_err(|e| error(e.to_string()))?;
    for record in &mut lower.records {
        record.content = functions::resolve_local_refs(&record.content, &program.pack.namespace);
    }
    let caps = target.caps();
    let ctx = crate::compiler::export::ExportCtx {
        caps: &caps,
        requested_version: target.resolved_name(),
        is_fallback: target.is_fallback(),
    };
    for module in &program.modules {
        let owner = module.id.to_string();
        for (index, recipe) in module.recipes.iter().enumerate() {
            let fail = |message: String| {
                vec![Diagnostic::error(
                    "SAND_PROGRAM_RESOURCE",
                    &owner,
                    &format!("/recipes/{index}"),
                    message,
                )]
            };
            let component = recipe_component(recipe, &lower.custom_items).map_err(fail)?;
            let record = records::component_to_record(component.as_ref(), Some(&ctx))
                .map_err(|e| fail(e.to_string()))?;
            lower.records.push(record);
        }
        for (index, table) in module.loot_tables.iter().enumerate() {
            let fail = |message: String| {
                vec![Diagnostic::error(
                    "SAND_PROGRAM_RESOURCE",
                    &owner,
                    &format!("/loot_tables/{index}"),
                    message,
                )]
            };
            let component = loot_table_component(table);
            let record = records::component_to_record(&component, Some(&ctx))
                .map_err(|e| fail(e.to_string()))?;
            lower.records.push(record);
        }
        for (index, tag) in module.resource_tags.iter().enumerate() {
            let fail = |message: String| {
                vec![Diagnostic::error(
                    "SAND_PROGRAM_RESOURCE",
                    &owner,
                    &format!("/resource_tags/{index}"),
                    message,
                )]
            };
            let component = tag_component(tag).map_err(fail)?;
            let record = records::component_to_record(component.as_ref(), Some(&ctx))
                .map_err(|e| fail(e.to_string()))?;
            lower.records.push(record);
        }
    }
    for flag in lower.flags.clone() {
        let component = flag_predicate(&program.pack.namespace, flag).map_err(error)?;
        let record = records::component_to_record(&component, Some(&ctx))
            .map_err(|e| error(e.to_string()))?;
        lower.records.push(record);
    }
    crate::compiler::export::lower_for_target(&mut lower.records, &ctx)
        .map_err(|e| error(e.to_string()))?;
    records::validate_unique_output_identities(&lower.records).map_err(|e| error(e.to_string()))?;
    records::validate_objective_definitions(&lower.records).map_err(|e| error(e.to_string()))?;
    let mut output = BTreeMap::new();
    let metadata =
        sand_components::pack_metadata::base(&program.pack.description, target.data_pack_format());
    insert_output(
        &mut output,
        "pack.mcmeta".into(),
        serde_json::to_vec_pretty(&metadata).expect("metadata serializes"),
    )?;
    let runtime_report =
        crate::compiler::runtime::RuntimeReport::collect(&lower.records, &lower.runtime);
    for record in lower.records {
        insert_output(
            &mut output,
            format!(
                "data/{}/{}/{}.{}",
                record.namespace, record.dir, record.path, record.ext
            ),
            record.content.into_bytes(),
        )?;
    }
    Ok(LoweredProgram {
        resources: output,
        runtime_report,
    })
}

fn insert_output(
    output: &mut BTreeMap<String, Vec<u8>>,
    path: String,
    bytes: Vec<u8>,
) -> Result<(), Vec<Diagnostic>> {
    if path.len() > limits::RESOURCE_PATH_BYTES
        || path
            .split('/')
            .any(|segment| segment.len() > limits::RESOURCE_SEGMENT_BYTES)
    {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_PATH",
            "",
            "",
            format!(
                "generated resource path exceeds the {}-byte path or {}-byte segment limit: `{path}`",
                limits::RESOURCE_PATH_BYTES,
                limits::RESOURCE_SEGMENT_BYTES
            ),
        )]);
    }
    if let Some(segment) = path.split('/').find(|segment| !portable_segment(segment)) {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_PATH",
            "",
            "",
            format!("generated resource path contains non-portable segment `{segment}`: `{path}`"),
        )]);
    }
    if output.len() >= limits::RESOURCES
        || bytes.len() > limits::DOCUMENT_BYTES
        || output.values().map(Vec::len).sum::<usize>() + bytes.len() > limits::OUTPUT_BYTES
    {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            "",
            "",
            "generated output limit exceeded",
        )]);
    }
    let prefix = format!("{path}/");
    if output.contains_key(&path)
        || output.keys().any(|existing| {
            existing.starts_with(&prefix) || path.starts_with(&format!("{existing}/"))
        })
    {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_COLLISION",
            "",
            "",
            format!("generated resource file/directory collision at `{path}`"),
        )]);
    }
    output.insert(path, bytes);
    Ok(())
}

// Windows device names remain reserved with any extension. Reject them on
// every host so check/compile results remain portable and deterministic.
fn portable_segment(segment: &str) -> bool {
    let basename = segment
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    !segment.ends_with(['.', ' '])
        && !matches!(basename.as_str(), "con" | "prn" | "aux" | "nul")
        && !((basename.starts_with("com") || basename.starts_with("lpt"))
            && basename.len() == 4
            && matches!(basename.as_bytes()[3], b'1'..=b'9'))
}

/// Adapt declarative flows into the same owned plan consumed by Rust collection.
fn flow_registration(
    flow: &Flow,
    holders: &BTreeMap<String, sand_commands::ScoreHolder>,
    defaults: &BTreeMap<String, i32>,
) -> crate::state::flow::FlowRegistration {
    use crate::state::flow::{
        FlowHook as OwnedHook, FlowRegistration, FlowTransition as OwnedTransition,
    };
    let holder = holders[&flow.state.state.to_string()].to_string();
    FlowRegistration {
        id: flow.id.to_string(),
        state_type: format!("State {}.{}", flow.state.state, flow.state.field),
        objective: objective(&flow.state),
        default_score: Some(defaults[&format!("{}.{}", flow.state.state, flow.state.field)]),
        subjects: (holder == "@s").then(|| "@a".to_owned()),
        holder,
        transitions: flow
            .transitions
            .iter()
            .enumerate()
            .map(|(order, transition)| OwnedTransition {
                from: transition.from,
                to: transition.to,
                priority: transition.priority,
                order,
                guard: transition.when.as_ref().map(|condition| {
                    use crate::condition::ScoreRange;
                    let range = match (condition.min, condition.max) {
                        (Some(min), Some(max)) if min == max => ScoreRange::Eq(min),
                        (Some(min), Some(max)) => ScoreRange::Between(Some(min), Some(max)),
                        (Some(min), None) => ScoreRange::Gte(min),
                        (None, Some(max)) => ScoreRange::Lte(max),
                        (None, None) => ScoreRange::Between(Some(i32::MIN), Some(i32::MAX)),
                    };
                    crate::condition::Condition::score(
                        holders[&condition.score.state.to_string()].to_string(),
                        objective(&condition.score),
                        range,
                    )
                }),
            })
            .collect(),
        hooks: flow
            .hooks
            .iter()
            .enumerate()
            .map(|(order, hook)| OwnedHook {
                kind: hook.phase,
                state: hook.state,
                cadence: hook.cadence,
                order,
                commands: crate::ir::Actions(vec![Cmd::Function(reference_id(&hook.function))]),
            })
            .collect(),
    }
}

fn reference_id(reference: &FunctionReference) -> String {
    match reference {
        FunctionReference::Internal { id } | FunctionReference::External { id, .. } => {
            id.to_string()
        }
    }
}
fn objective(score: &FieldReference) -> String {
    sand_commands::ObjectiveName::logical(format!("{}.{}", score.state, score.field))
        .as_str()
        .to_owned()
}
fn helper(namespace: &str, key: &str, path: &str, role: &str) -> String {
    // Export-local allocation bounds this segment while detecting and resolving
    // hash collisions independently of declaration order.
    format!("{namespace}:__sand_program/{key}/{path}/{role}")
}
/// The canonical Rust custom item for a portable declaration.
fn custom_item(
    item: &CustomItem,
    holders: &BTreeMap<String, sand_commands::ScoreHolder>,
) -> sand_components::CustomItem {
    let mut custom = sand_components::CustomItem::new(&item.base).custom_data(item.id.to_string());
    if !item.name.is_empty() {
        custom = custom.custom_name(text_component(&item.name, holders));
    }
    if !item.lore.is_empty() {
        custom = custom.lore(
            item.lore
                .iter()
                .map(|line| text_component(line, holders))
                .collect(),
        );
    }
    if let Some(rarity) = item.rarity {
        custom = custom.rarity(match rarity {
            Rarity::Common => sand_components::ItemRarity::Common,
            Rarity::Uncommon => sand_components::ItemRarity::Uncommon,
            Rarity::Rare => sand_components::ItemRarity::Rare,
            Rarity::Epic => sand_components::ItemRarity::Epic,
        });
    }
    if let Some(glint) = item.glint {
        custom = custom.enchantment_glint_override(glint);
    }
    if let Some(size) = item.max_stack {
        custom = custom.max_stack_size(size);
    }
    if item.unbreakable {
        custom = custom.unbreakable(true);
    }
    for (enchantment, level) in &item.enchantments {
        custom = custom.enchantment(enchantment.clone(), *level);
    }
    if let Some(data) = item.custom_model_data {
        custom = custom.custom_model_data(data);
    }
    custom
}

fn ingredient(id: &str) -> sand_components::recipe::Ingredient {
    match id.strip_prefix('#') {
        Some(tag) => sand_components::recipe::Ingredient::tag(tag),
        None => sand_components::recipe::Ingredient::item(id),
    }
}

fn recipe_output(
    output: &RecipeOutput,
    custom_items: &BTreeMap<String, sand_components::CustomItem>,
) -> Result<sand_components::recipe::RecipeResult, String> {
    match custom_items.get(&output.item) {
        Some(custom) => {
            sand_components::recipe::RecipeResult::from_custom_item(custom, output.count)
                .map_err(|e| e.to_string())
        }
        None => Ok(sand_components::recipe::RecipeResult::new(
            &output.item,
            output.count,
        )),
    }
}

/// Build the typed recipe component that Rust authors would write.
fn recipe_component(
    recipe: &Recipe,
    custom_items: &BTreeMap<String, sand_components::CustomItem>,
) -> Result<Box<dyn sand_components::DatapackComponent>, String> {
    use sand_components::recipe::*;
    Ok(match recipe {
        Recipe::Shaped {
            id,
            pattern,
            key,
            result,
        } => {
            let mut built = ShapedRecipe::new(id.clone()).pattern(pattern.iter().cloned());
            for (symbol, item) in key {
                let symbol = symbol.chars().next().expect("validated single character");
                built = built.key(symbol, ingredient(item));
            }
            Box::new(built.result(recipe_output(result, custom_items)?))
        }
        Recipe::Shapeless {
            id,
            ingredients,
            result,
        } => {
            let mut built = ShapelessRecipe::new(id.clone());
            for item in ingredients {
                built = built.ingredient(ingredient(item));
            }
            Box::new(built.result(recipe_output(result, custom_items)?))
        }
        Recipe::Smithing {
            id,
            template,
            base,
            addition,
            result,
        } => Box::new(
            SmithingTransformRecipe::new(id.clone())
                .template(ingredient(template))
                .base(ingredient(base))
                .addition(ingredient(addition))
                .result(recipe_output(result, custom_items)?),
        ),
        Recipe::Cooking {
            id,
            method,
            ingredient: input,
            result,
            experience,
            time,
        } => {
            let kind = match method {
                CookingMethod::Smelting => CookingType::Smelting,
                CookingMethod::Blasting => CookingType::Blasting,
                CookingMethod::Smoking => CookingType::Smoking,
                CookingMethod::Campfire => CookingType::CampfireCooking,
            };
            let mut built = CookingRecipe::new(id.clone(), kind)
                .ingredient(ingredient(input))
                .result(recipe_output(result, custom_items)?)
                .experience(*experience);
            if let Some(time) = time {
                built = built.cooking_time(*time);
            }
            Box::new(built)
        }
        Recipe::Stonecutting {
            id,
            ingredient: input,
            result,
        } => Box::new(
            StonecuttingRecipe::new(id.clone())
                .ingredient(ingredient(input))
                .result(recipe_output(result, custom_items)?)
                .count(result.count),
        ),
    })
}

/// Build the typed registry tag that Rust authors would write.
fn tag_component(tag: &ResourceTag) -> Result<Box<dyn sand_components::DatapackComponent>, String> {
    use sand_components::{BlockId, EntityTypeId, ItemId, TagId, TypedTag};
    fn build<T>(tag: &ResourceTag) -> Result<TypedTag<T>, String>
    where
        T: sand_components::TagRegistry + std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let location: TagId<T> = tag.id.to_string().parse().map_err(|e| format!("{e}"))?;
        let mut built = TypedTag::new(location).replace(tag.replace);
        for value in &tag.values {
            built = match value.strip_prefix('#') {
                Some(reference) => {
                    let reference: TagId<T> = reference.parse().map_err(|e| format!("{e}"))?;
                    built.tag_ref(reference)
                }
                None => built.entry(value.parse::<T>().map_err(|e| e.to_string())?),
            };
        }
        Ok(built)
    }
    Ok(match tag.kind {
        TagKind::Item => Box::new(build::<ItemId>(tag)?),
        TagKind::Block => Box::new(build::<BlockId>(tag)?),
        TagKind::EntityType => Box::new(build::<EntityTypeId>(tag)?),
    })
}

/// Build the typed loot table that Rust authors would write.
fn loot_table_component(table: &LootTable) -> sand_components::LootTable {
    use sand_components::{LootEntry, LootFunction, LootPool};
    let mut pool = LootPool::new().rolls(table.rolls as i32);
    for item in &table.items {
        let LootEntry::Item {
            name,
            quality,
            conditions,
            ..
        } = LootEntry::item_raw(&item.item)
        else {
            unreachable!("item_raw builds an item entry");
        };
        pool = pool.entry(LootEntry::Item {
            name,
            weight: Some(item.weight as i32),
            quality,
            functions: if item.count > 1 {
                vec![LootFunction::SetCount {
                    count: (item.count as i32).into(),
                    add: false,
                }]
            } else {
                Vec::new()
            },
            conditions,
        });
    }
    sand_components::LootTable::new(table.id.clone()).pool(pool)
}

/// The generated predicate that tests one entity flag.
fn flag_predicate(namespace: &str, flag: EntityFlag) -> Result<sand_components::Predicate, String> {
    use sand_components::{
        EntityFlags, EntityPredicate, EntityPredicateTarget, Predicate, PredicateRoot,
    };
    let flags = EntityFlags::new();
    let flags = match flag {
        EntityFlag::Sneaking => flags.sneaking(true),
        EntityFlag::Sprinting => flags.sprinting(true),
        EntityFlag::Swimming => flags.swimming(true),
        EntityFlag::OnFire => flags.on_fire(true),
    };
    let location = flag_predicate_id(namespace, flag)
        .parse()
        .map_err(|e| format!("{e}"))?;
    Ok(Predicate::new(
        location,
        PredicateRoot::entity_properties(
            EntityPredicateTarget::This,
            EntityPredicate::new().flags(flags),
        ),
    ))
}

fn flag_predicate_id(namespace: &str, flag: EntityFlag) -> String {
    let name = match flag {
        EntityFlag::Sneaking => "sneaking",
        EntityFlag::Sprinting => "sprinting",
        EntityFlag::Swimming => "swimming",
        EntityFlag::OnFire => "on_fire",
    };
    format!("{namespace}:__sand_program/flag/{name}")
}

fn block_position(position: BlockPosition) -> sand_commands::BlockPos {
    use sand_commands::{BlockPos, Coord};
    match position {
        BlockPosition::Absolute { x, y, z } => BlockPos::new(
            Coord::Absolute(f64::from(x)),
            Coord::Absolute(f64::from(y)),
            Coord::Absolute(f64::from(z)),
        ),
        BlockPosition::Relative { x, y, z } => BlockPos::new(
            Coord::Relative(f64::from(x)),
            Coord::Relative(f64::from(y)),
            Coord::Relative(f64::from(z)),
        ),
    }
}

/// The tag marking players who see a boss bar.
fn boss_bar_tag(bar: &sand_components::ResourceLocation) -> String {
    format!(
        "sand.{}",
        sand_commands::scoreboard::hash_objective_name(&format!("{bar}#viewer"))
    )
}

fn constants_objective(namespace: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{namespace}.__sand_program.constants"))
        .as_str()
        .to_owned()
}

fn constant_holder(value: i32) -> sand_commands::ScoreHolder {
    sand_commands::ScoreHolder::fake(format!("#c{value}"))
}

/// One generated lifecycle line for portable player events.
enum EventLine {
    Load(String),
    Tick(String),
}

fn text_component(
    parts: &[TextPart],
    holders: &BTreeMap<String, sand_commands::ScoreHolder>,
) -> sand_commands::TextComponent {
    use sand_commands::{ChatColor, TextComponent};
    let styled = |mut component: TextComponent, style: &TextStyle| {
        if let Some(color) = style.color {
            component = component.color(match color {
                TextColor::Black => ChatColor::Black,
                TextColor::DarkBlue => ChatColor::DarkBlue,
                TextColor::DarkGreen => ChatColor::DarkGreen,
                TextColor::DarkAqua => ChatColor::DarkAqua,
                TextColor::DarkRed => ChatColor::DarkRed,
                TextColor::DarkPurple => ChatColor::DarkPurple,
                TextColor::Gold => ChatColor::Gold,
                TextColor::Gray => ChatColor::Gray,
                TextColor::DarkGray => ChatColor::DarkGray,
                TextColor::Blue => ChatColor::Blue,
                TextColor::Green => ChatColor::Green,
                TextColor::Aqua => ChatColor::Aqua,
                TextColor::Red => ChatColor::Red,
                TextColor::LightPurple => ChatColor::LightPurple,
                TextColor::Yellow => ChatColor::Yellow,
                TextColor::White => ChatColor::White,
            });
        }
        // Parts are children of an unstyled root, so unset properties stay
        // unset instead of being restated as `false`.
        if style.bold {
            component = component.bold(true);
        }
        if style.italic {
            component = component.italic(true);
        }
        if style.underlined {
            component = component.underlined(true);
        }
        component
    };
    let mut root = TextComponent::literal("");
    for part in parts {
        let component = match part {
            TextPart::Text { text, style } => styled(TextComponent::literal(text.clone()), style),
            TextPart::Score { score, style } => styled(
                TextComponent::score(
                    holders[&score.state.to_string()].to_string(),
                    objective(score),
                ),
                style,
            ),
            TextPart::PlayerName { style } => styled(TextComponent::selector("@s"), style),
        };
        root = root.then(component);
    }
    root
}

/// Whether a body can end its enclosing function frame with `Return`.
///
/// `Players` bodies are their own frame: a return there ends one player's
/// turn, so it does not propagate.
fn returns(body: &[Operation]) -> bool {
    body.iter().any(|operation| match &operation.action {
        Action::Return => true,
        Action::Branch {
            then, otherwise, ..
        } => returns(then) || returns(otherwise),
        Action::While { body, .. } => returns(body),
        _ => false,
    })
}

/// Call a helper; when it can return early, propagate that return.
///
/// Helpers signal an early return by returning 1. `execute if function`
/// succeeds only for a nonzero result, so a helper that finishes normally
/// (with no result) lets the caller continue.
fn call_helper(id: String, propagate: bool) -> Cmd {
    if propagate {
        Cmd::Execute {
            operations: vec![sand_commands::ExecuteOp::If(
                sand_commands::ConditionIr::Function(id),
            )],
            run: Box::new(Cmd::Return(1)),
        }
    } else {
        Cmd::Function(id)
    }
}

/// A canonical State objective, matching Rust's `state::objective_name`.
fn state_objective(state: &str, suffix: &str) -> String {
    sand_commands::ObjectiveName::logical(format!("{state}.{suffix}"))
        .as_str()
        .to_owned()
}

fn query_selector(query: &EntityQuery) -> sand_commands::Selector {
    let mut selector = sand_commands::Selector::all_entities();
    if let Some(entity_type) = &query.entity_type {
        selector = selector.entity_type_raw(entity_type.clone());
    }
    if let Some(tag) = &query.tag {
        selector = selector.tag(tag.clone());
    }
    if let Some(state) = &query.with_state {
        // Attached at revision 1, as `has_state` tests.
        selector = selector
            .score_typed(
                sand_commands::ObjectiveName::logical(format!("{state}.presence")),
                sand_commands::ScoreRange::exact(1),
            )
            .expect("canonical presence objective with a single score filter");
    }
    selector = match (query.min_distance, query.max_distance) {
        (Some(min), Some(max)) => selector.distance_range(min, max),
        (Some(min), None) => selector.distance_min(min),
        (None, Some(max)) => selector.distance_max(max),
        (None, None) => selector,
    };
    if let Some(sort) = query.sort {
        selector = selector.sort(match sort {
            EntitySort::Nearest => sand_commands::SortOrder::Nearest,
            EntitySort::Furthest => sand_commands::SortOrder::Furthest,
            EntitySort::Random => sand_commands::SortOrder::Random,
            EntitySort::Arbitrary => sand_commands::SortOrder::Arbitrary,
        });
    }
    if let Some(limit) = query.limit {
        selector = selector.limit(limit as i32);
    }
    selector
}

struct Lower {
    runtime: crate::compiler::runtime::RuntimePlan,
    namespace: String,
    holders: BTreeMap<String, sand_commands::ScoreHolder>,
    bounds: BTreeMap<String, (i32, i32)>,
    helper_keys: BTreeMap<String, String>,
    records: Vec<ComponentRecord>,
    init: Option<String>,
    bytes: usize,
    profile: sand_commands::CommandProfile,
    /// Constant operands materialised on the compiler-owned constant holder.
    constants: std::collections::BTreeSet<i32>,
    /// Declared custom items by id.
    custom_items: BTreeMap<String, sand_components::CustomItem>,
    /// Entity flags tested anywhere; each gets one generated predicate.
    flags: std::collections::BTreeSet<EntityFlag>,
    /// Entity States by id, with their field defaults.
    entity_states: BTreeMap<String, Vec<(String, i32)>>,
}
impl Lower {
    fn function(
        &mut self,
        id: &str,
        body: Vec<String>,
        module: &str,
        ptr: &str,
    ) -> Result<(), Vec<Diagnostic>> {
        let content = body.join("\n");
        if self.records.len() >= limits::RESOURCES
            || content.len() > limits::DOCUMENT_BYTES
            || self.bytes.saturating_add(content.len()) > limits::OUTPUT_BYTES
        {
            return Err(vec![Diagnostic::error(
                "SAND_PROGRAM_LIMIT",
                module,
                ptr,
                "generated output limit exceeded",
            )]);
        }
        self.bytes += content.len();
        self.runtime.own_function(id.to_owned(), module.to_owned());
        let (namespace, path) = id.split_once(':').expect("validated/generated identity");
        self.records.push(ComponentRecord {
            namespace: namespace.into(),
            dir: "function".into(),
            path: path.into(),
            ext: "mcfunction".into(),
            content_type: "text".into(),
            content,
        });
        Ok(())
    }
    /// Lower player event observation: one namespace-owned objective or tag per
    /// distinct event, a tick probe that runs a dispatcher as each observed
    /// player, and a dispatcher that consumes the observation before handlers.
    fn events(&mut self, program: &Program) -> Result<Vec<EventLine>, Vec<Diagnostic>> {
        let mut groups: BTreeMap<String, (PlayerEvent, Vec<(String, String)>)> = BTreeMap::new();
        let mut modules: Vec<_> = program.modules.iter().collect();
        modules.sort_by_key(|module| module.id.to_string());
        for module in modules {
            for (index, hook) in module.events.iter().enumerate() {
                groups
                    .entry(hook.event.key())
                    .or_insert_with(|| (hook.event.clone(), Vec::new()))
                    .1
                    .push((
                        reference_id(&hook.function),
                        format!("{}#{index}", module.id),
                    ));
            }
        }
        let mut lines = Vec::new();
        for (key, (event, handlers)) in groups {
            let base = format!("{}.__sand_event.{key}", self.namespace);
            let dispatch = format!("{}:__sand_program/event/{key}", self.namespace);
            let objective = sand_commands::ObjectiveName::logical(base.clone())
                .as_str()
                .to_owned();
            let tag = format!(
                "sand.{}",
                sand_commands::scoreboard::hash_objective_name(&base)
            );
            let mut body = Vec::new();
            if let Some(init) = &self.init {
                body.push(Cmd::Function(init.clone()).render());
            }
            let criteria = event.criterion();
            if let Some(criteria) = &criteria {
                lines.push(EventLine::Load(format!(
                    "scoreboard objectives add {objective} {criteria}"
                )));
                lines.push(EventLine::Tick(format!(
                    "execute as @a[scores={{{objective}=1..}}] at @s run function {dispatch}"
                )));
                body.push(format!("scoreboard players reset @s {objective}"));
            }
            if matches!(event, PlayerEvent::Join | PlayerEvent::FirstJoin) {
                // A persistent tag marks players already observed once; joins
                // after that are seen through the leave_game statistic.
                lines.push(EventLine::Tick(format!(
                    "execute as @a[tag=!{tag}] at @s run function {dispatch}"
                )));
                body.push(format!("tag @s add {tag}"));
            }
            for (function, _) in &handlers {
                body.push(Cmd::Function(function.clone()).render());
            }
            let owner = handlers
                .first()
                .map(|(_, owner)| owner.clone())
                .unwrap_or_default();
            self.function(&dispatch, body, &owner, "/events")?;
        }
        Ok(lines)
    }

    /// Commands that create a boss bar and refresh its display at load.
    fn boss_bar_setup(&self, bar: &BossBar) -> Vec<String> {
        let id = &bar.id;
        let title = text_component(&bar.title, &self.holders);
        let color = match bar.color {
            BossBarColor::Blue => "blue",
            BossBarColor::Green => "green",
            BossBarColor::Pink => "pink",
            BossBarColor::Purple => "purple",
            BossBarColor::Red => "red",
            BossBarColor::White => "white",
            BossBarColor::Yellow => "yellow",
        };
        let style = match bar.style {
            BossBarStyle::Progress => "progress",
            BossBarStyle::Notched6 => "notched_6",
            BossBarStyle::Notched10 => "notched_10",
            BossBarStyle::Notched12 => "notched_12",
            BossBarStyle::Notched20 => "notched_20",
        };
        let name = title.to_string();
        vec![
            format!("bossbar add {id} {name}"),
            format!("bossbar set {id} name {name}"),
            format!("bossbar set {id} color {color}"),
            format!("bossbar set {id} style {style}"),
            format!("bossbar set {id} max {}", bar.max),
        ]
    }

    /// A custom item as a command item argument with its identity components.
    fn item_argument(&self, item: &str) -> Result<String, String> {
        let Some(custom) = self.custom_items.get(item) else {
            return Ok(item.to_owned());
        };
        let predicate = custom.item_predicate().map_err(|e| e.to_string())?;
        let lowered = sand_components::predicates::lower_item_predicate_for_command(&predicate)?;
        let base = lowered
            .items
            .as_ref()
            .and_then(|items| items.first())
            .map(|id| id.to_string())
            .unwrap_or_default();
        Ok(format!("{base}[{}]", lowered.constraints.join(",")))
    }

    fn condition_test(
        &mut self,
        condition: &Condition,
    ) -> Result<sand_commands::ConditionIr, String> {
        Ok(match condition {
            Condition::Exists(test) => {
                sand_commands::ConditionIr::Entity(query_selector(&test.exists))
            }
            Condition::Block(test) => sand_commands::ConditionIr::Block {
                position: block_position(test.at),
                block: test.block.clone(),
            },
            Condition::Holding(test) => sand_commands::ConditionIr::ItemsEntity {
                target: sand_commands::Selector::self_(),
                slot: if test.offhand {
                    sand_commands::ItemSlot::OffHand
                } else {
                    sand_commands::ItemSlot::MainHand
                },
                item: self.item_argument(&test.holding)?,
            },
            Condition::Flag(test) => {
                self.flags.insert(test.flag);
                sand_commands::ConditionIr::Predicate(flag_predicate_id(&self.namespace, test.flag))
            }
            Condition::Attached(test) => sand_commands::ConditionIr::ScoreMatches {
                holder: sand_commands::ScoreHolder::self_(),
                objective: state_objective(&test.attached.to_string(), "presence"),
                range: "1..1".into(),
            },
            Condition::Range(range) => sand_commands::ConditionIr::ScoreMatches {
                holder: self.holders[&range.score.state.to_string()].clone(),
                objective: objective(&range.score),
                range: format!(
                    "{}..{}",
                    range.min.unwrap_or(i32::MIN),
                    range.max.unwrap_or(i32::MAX)
                ),
            },
            Condition::Relation(relation) => sand_commands::ConditionIr::ScoreCompare {
                left: self.holders[&relation.left.state.to_string()].clone(),
                left_objective: objective(&relation.left),
                op: match relation.relation {
                    Relation::Lt => sand_commands::ScoreCmp::Lt,
                    Relation::Le => sand_commands::ScoreCmp::Le,
                    Relation::Eq => sand_commands::ScoreCmp::Eq,
                    Relation::Ge => sand_commands::ScoreCmp::Ge,
                    Relation::Gt => sand_commands::ScoreCmp::Gt,
                },
                right: self.holders[&relation.right.state.to_string()].clone(),
                right_objective: objective(&relation.right),
            },
        })
    }

    fn body(
        &mut self,
        body: &[Operation],
        owner: &str,
        path: &str,
        module: &str,
        ptr: &str,
    ) -> Result<Vec<String>, Vec<Diagnostic>> {
        let mut commands = Vec::new();
        for (index, operation) in body.iter().enumerate() {
            let path = if path.is_empty() {
                index.to_string()
            } else {
                format!("{path}/{index}")
            };
            let ptr = format!("{ptr}/{index}");
            let fail = |error: String| {
                let mut error = Diagnostic::error("SAND_PROGRAM_COMMAND", module, &ptr, error);
                error.origin = operation.origin.clone();
                vec![error]
            };
            let cmds = match &operation.action {
                Action::ScoreSet { score, value } | Action::ScoreAdd { score, value } => {
                    let bounds = self
                        .bounds
                        .get(&format!("{}.{}", score.state, score.field))
                        .copied();
                    let objective = objective(score);
                    let holder = &self.holders[&score.state.to_string()];
                    if matches!(operation.action, Action::ScoreSet { .. }) {
                        // Portable v1 assigns only integer literals to @s or a
                        // global holder. Canonical constant lowering is local:
                        // it allocates no helpers or ambient registry state.
                        NumericWrite {
                            value: crate::entity::StatCurve::from(*value),
                            holder: holder.clone(),
                            objective: sand_commands::ObjectiveName::logical(objective),
                            scale: 1,
                            fixed: crate::entity::FixedPoint::default(),
                            bounds,
                            dirty: Vec::new(),
                        }
                        .lower(&module.parse().expect("validated module identity"))
                        .map_err(|error| fail(error.to_string()))?
                    } else {
                        let mut commands =
                            crate::ir::score_delta(holder, &objective, *value, false);
                        if let Some(bounds) = bounds {
                            commands.extend(crate::ir::score_bounds(holder, &objective, bounds));
                        }
                        commands
                    }
                }
                Action::Call { function } => vec![Cmd::Function(reference_id(function))],
                Action::Raw { command } => vec![Cmd::Raw(command.clone())],
                Action::Players { body } => {
                    let target = if let [
                        Operation {
                            action:
                                Action::Call {
                                    function: function @ FunctionReference::Internal { .. },
                                },
                            ..
                        },
                    ] = body.as_slice()
                    {
                        reference_id(function)
                    } else {
                        let id =
                            helper(&self.namespace, &self.helper_keys[owner], &path, "players");
                        let mut lines = self.body(
                            body,
                            owner,
                            &format!("{path}/players"),
                            module,
                            &format!("{ptr}/action/body"),
                        )?;
                        if let Some(init) = &self.init {
                            lines.insert(0, Cmd::Function(init.clone()).render());
                        }
                        self.function(&id, lines, module, &ptr)?;
                        id
                    };
                    vec![Cmd::Execute {
                        operations: vec![sand_commands::ExecuteOp::As(
                            sand_commands::Selector::all_players(),
                        )],
                        run: Box::new(Cmd::Function(target)),
                    }]
                }
                Action::ScoreOperation {
                    score,
                    operator,
                    operand,
                } => {
                    let holder = self.holders[&score.state.to_string()].clone();
                    let target_objective = objective(score);
                    let mut commands = Vec::new();
                    let (source, source_objective) = match operand {
                        ScoreOperand::Field(field) => (
                            self.holders[&field.state.to_string()].clone(),
                            objective(field),
                        ),
                        ScoreOperand::Value(value) => {
                            self.constants.insert(*value);
                            (
                                constant_holder(*value),
                                constants_objective(&self.namespace),
                            )
                        }
                    };
                    commands.push(Cmd::ScorePlayers(crate::ir::ScorePlayersOp::Operation {
                        target: holder.to_string(),
                        target_obj: target_objective.clone(),
                        op: match operator {
                            ScoreOperator::Set => crate::ir::ScoreOpKind::Assign,
                            ScoreOperator::Add => crate::ir::ScoreOpKind::Add,
                            ScoreOperator::Sub => crate::ir::ScoreOpKind::Sub,
                            ScoreOperator::Mul => crate::ir::ScoreOpKind::Mul,
                            ScoreOperator::Div => crate::ir::ScoreOpKind::Div,
                            ScoreOperator::Mod => crate::ir::ScoreOpKind::Mod,
                            ScoreOperator::Min => crate::ir::ScoreOpKind::Min,
                            ScoreOperator::Max => crate::ir::ScoreOpKind::Max,
                        },
                        source: source.to_string(),
                        source_obj: source_objective,
                    }));
                    if let Some(bounds) = self
                        .bounds
                        .get(&format!("{}.{}", score.state, score.field))
                        .copied()
                    {
                        commands.extend(crate::ir::score_bounds(
                            &holder,
                            &target_objective,
                            bounds,
                        ));
                    }
                    commands
                }
                Action::ScoreRandom { score, min, max } => {
                    let holder = self.holders[&score.state.to_string()].clone();
                    let target_objective = objective(score);
                    let mut commands = vec![Cmd::Execute {
                        operations: vec![sand_commands::ExecuteOp::StoreResult(
                            sand_commands::ExecuteStoreTarget::Score {
                                holder: holder.clone(),
                                objective: target_objective.clone(),
                            },
                        )],
                        run: Box::new(Cmd::Raw(format!("random value {min}..{max}"))),
                    }];
                    if let Some(bounds) = self
                        .bounds
                        .get(&format!("{}.{}", score.state, score.field))
                        .copied()
                    {
                        commands.extend(crate::ir::score_bounds(
                            &holder,
                            &target_objective,
                            bounds,
                        ));
                    }
                    commands
                }
                Action::Message {
                    audience,
                    channel,
                    text,
                } => {
                    let target = match audience {
                        Audience::Player => sand_commands::Selector::self_(),
                        Audience::Everyone => sand_commands::Selector::all_players(),
                    };
                    let component = text_component(text, &self.holders);
                    let lines = match channel {
                        MessageChannel::Chat => vec![crate::cmd::tellraw(target, component)],
                        MessageChannel::Actionbar => {
                            vec![sand_commands::Actionbar::show(target, component)]
                        }
                        MessageChannel::Title => sand_commands::Title::of(target)
                            .title(component)
                            .try_build()
                            .map_err(|e| fail(e.to_string()))?,
                        MessageChannel::Subtitle => sand_commands::Title::of(target)
                            .subtitle(component)
                            .try_build()
                            .map_err(|e| fail(e.to_string()))?,
                    };
                    lines.into_iter().map(Cmd::Raw).collect()
                }
                Action::Sound {
                    sound,
                    volume,
                    pitch,
                } => vec![Cmd::Raw(
                    sand_commands::Sound::play_raw(sound.clone())
                        .to(sand_commands::Selector::self_())
                        .volume(f64::from(*volume))
                        .pitch(f64::from(*pitch))
                        .to_string(),
                )],
                Action::Particle {
                    particle,
                    count,
                    spread,
                    speed,
                } => vec![Cmd::Raw(format!(
                    "particle {particle} ~ ~1 ~ {spread} {spread} {spread} {speed} {count}"
                ))],
                Action::Effect {
                    effect,
                    seconds,
                    amplifier,
                    hide_particles,
                } => vec![Cmd::Effect(
                    sand_commands::EffectCommand::give_raw(
                        sand_commands::Selector::self_(),
                        effect.clone(),
                    )
                    .duration(match seconds {
                        Some(seconds) => sand_commands::EffectDuration::Seconds(*seconds),
                        None => sand_commands::EffectDuration::Infinite,
                    })
                    .amplifier(*amplifier)
                    .particles(!hide_particles),
                )],
                Action::ClearEffect { effect } => vec![Cmd::Raw(match effect {
                    Some(effect) => format!("effect clear @s {effect}"),
                    None => "effect clear @s".to_owned(),
                })],
                Action::GiveItem { item, count } => {
                    let inventory = sand_commands::Inventory::of(sand_commands::Selector::self_());
                    let line = match self.custom_items.get(item) {
                        Some(custom) => {
                            let stack =
                                sand_components::IntoItemStack::try_into_item_stack(custom.clone())
                                    .map_err(|e| fail(e.to_string()))?;
                            inventory.give_count(stack, *count)
                        }
                        None => inventory.give_count(item, *count),
                    };
                    vec![Cmd::Raw(line)]
                }
                Action::ClearItem { item, count } => {
                    let inventory = sand_commands::Inventory::of(sand_commands::Selector::self_());
                    // Custom items clear by their custom_data identity.
                    let item = match item {
                        Some(item) => Some(self.item_argument(item).map_err(fail)?),
                        None => None,
                    };
                    vec![Cmd::Raw(match (item, count) {
                        (Some(item), Some(count)) => inventory.clear_item_count(item, *count),
                        (Some(item), None) => inventory.clear_item(item),
                        (None, _) => inventory.clear_all(),
                    })]
                }
                Action::Schedule { function, ticks } => {
                    vec![Cmd::Raw(sand_commands::builtins::schedule_replace_raw(
                        reference_id(function),
                        format!("{ticks}t"),
                    ))]
                }
                Action::Branch {
                    condition,
                    then,
                    otherwise,
                } => {
                    let yes = helper(&self.namespace, &self.helper_keys[owner], &path, "then");
                    let no = helper(
                        &self.namespace,
                        &self.helper_keys[owner],
                        &path,
                        "otherwise",
                    );
                    let decision =
                        helper(&self.namespace, &self.helper_keys[owner], &path, "branch");
                    let yes_body = self.body(
                        then,
                        owner,
                        &format!("{path}/then"),
                        module,
                        &format!("{ptr}/action/then"),
                    )?;
                    let no_body = self.body(
                        otherwise,
                        owner,
                        &format!("{path}/otherwise"),
                        module,
                        &format!("{ptr}/action/otherwise"),
                    )?;
                    self.function(&yes, yes_body, module, &ptr)?;
                    self.function(&no, no_body, module, &ptr)?;
                    let test = self.condition_test(condition).map_err(fail)?;
                    let propagate = returns(then) || returns(otherwise);
                    let guard = Cmd::Execute {
                        operations: vec![sand_commands::ExecuteOp::If(test)],
                        run: Box::new(Cmd::ReturnRun(Box::new(Cmd::Function(yes)))),
                    };
                    // Without early returns the decision keeps its original
                    // shape; with them, both outcomes forward the helper result.
                    let otherwise_call = if propagate {
                        Cmd::ReturnRun(Box::new(Cmd::Function(no)))
                    } else {
                        Cmd::Function(no)
                    };
                    self.function(
                        &decision,
                        vec![guard.render(), otherwise_call.render()],
                        module,
                        &ptr,
                    )?;
                    vec![call_helper(decision, propagate)]
                }
                Action::Return => vec![Cmd::Return(1)],
                Action::Entities { query, body } => {
                    let id = helper(&self.namespace, &self.helper_keys[owner], &path, "entities");
                    let lines = self.body(
                        body,
                        owner,
                        &format!("{path}/entities"),
                        module,
                        &format!("{ptr}/action/body"),
                    )?;
                    self.function(&id, lines, module, &ptr)?;
                    vec![Cmd::Execute {
                        operations: vec![
                            sand_commands::ExecuteOp::As(query_selector(query)),
                            sand_commands::ExecuteOp::At(sand_commands::Selector::self_()),
                        ],
                        run: Box::new(Cmd::Function(id)),
                    }]
                }
                Action::Kill => vec![Cmd::Raw("kill @s".into())],
                Action::Teleport { destination } => vec![Cmd::Raw(match destination {
                    TeleportDestination::Position { x, y, z } => format!("tp @s {x} {y} {z}"),
                    TeleportDestination::Offset { x, y, z } => format!("tp @s ~{x} ~{y} ~{z}"),
                    TeleportDestination::Entity { query } => {
                        // Exactly one destination, nearest unless sorted.
                        let mut query = query.clone();
                        query.limit = Some(1);
                        query.sort = query.sort.or(Some(EntitySort::Nearest));
                        format!("tp @s {}", query_selector(&query))
                    }
                })],
                // Mirrors Rust's `state_attach_commands` for entity State:
                // keep existing values, default the rest, then mark present.
                Action::AttachState { state } => {
                    let id = state.to_string();
                    let mut commands: Vec<Cmd> = self.entity_states[&id]
                        .iter()
                        .map(|(field, default)| {
                            let objective = state_objective(&id, field);
                            Cmd::Raw(format!(
                                "execute unless score @s {objective} matches -2147483648.. run scoreboard players set @s {objective} {default}"
                            ))
                        })
                        .collect();
                    let presence = state_objective(&id, "presence");
                    commands.push(Cmd::Raw(format!(
                        "execute unless score @s {presence} matches 1.. run scoreboard players set @s {presence} 1"
                    )));
                    commands
                }
                // Mirrors Rust's `state_detach_commands` with dirty tracking.
                Action::DetachState { state } => {
                    let id = state.to_string();
                    let mut commands = Vec::new();
                    for (field, _) in &self.entity_states[&id] {
                        commands.push(Cmd::Raw(format!(
                            "scoreboard players reset @s {}",
                            state_objective(&id, field)
                        )));
                        commands.push(Cmd::Raw(format!(
                            "scoreboard players reset @s {}",
                            state_objective(&id, &format!("{field}.dirty"))
                        )));
                    }
                    commands.push(Cmd::Raw(format!(
                        "scoreboard players reset @s {}",
                        state_objective(&id, "reconcile_dirty")
                    )));
                    commands.push(Cmd::Raw(format!(
                        "scoreboard players reset @s {}",
                        state_objective(&id, "presence")
                    )));
                    commands
                }
                Action::SetBlock { block, at } => {
                    vec![Cmd::Raw(format!(
                        "setblock {} {block}",
                        block_position(*at)
                    ))]
                }
                Action::FillBlocks { block, from, to } => vec![Cmd::Raw(format!(
                    "fill {} {} {block}",
                    block_position(*from),
                    block_position(*to)
                ))],
                Action::AddExperience { amount, levels } => vec![Cmd::Raw(format!(
                    "xp add @s {amount} {}",
                    if *levels { "levels" } else { "points" }
                ))],
                Action::SetGameMode { mode } => vec![Cmd::Raw(format!(
                    "gamemode {} @s",
                    match mode {
                        PlayerGameMode::Survival => "survival",
                        PlayerGameMode::Creative => "creative",
                        PlayerGameMode::Adventure => "adventure",
                        PlayerGameMode::Spectator => "spectator",
                    }
                ))],
                Action::SetTime { ticks } => vec![Cmd::Raw(format!("time set {ticks}"))],
                Action::SetWeather { weather, seconds } => {
                    let weather = match weather {
                        Weather::Clear => "clear",
                        Weather::Rain => "rain",
                        Weather::Thunder => "thunder",
                    };
                    vec![Cmd::Raw(match seconds {
                        Some(seconds) => format!("weather {weather} {seconds}s"),
                        None => format!("weather {weather}"),
                    })]
                }
                Action::BossBarAudience {
                    bar,
                    audience,
                    visible,
                } => {
                    // `bossbar set players` replaces the viewer list, so a tag
                    // remembers who sees the bar across individual changes.
                    let tag = boss_bar_tag(bar);
                    let targets = match audience {
                        Audience::Player => "@s",
                        Audience::Everyone => "@a",
                    };
                    let verb = if *visible { "add" } else { "remove" };
                    vec![
                        Cmd::Raw(format!("tag {targets} {verb} {tag}")),
                        Cmd::Raw(format!("bossbar set {bar} players @a[tag={tag}]")),
                    ]
                }
                Action::SetBossBar { bar, value, max } => {
                    let property = if *max { "max" } else { "value" };
                    vec![Cmd::Raw(match value {
                        ScoreOperand::Value(value) => {
                            format!("bossbar set {bar} {property} {value}")
                        }
                        ScoreOperand::Field(field) => format!(
                            "execute store result bossbar {bar} {property} run scoreboard players get {} {}",
                            self.holders[&field.state.to_string()],
                            objective(field)
                        ),
                    })]
                }
                Action::GiveLoot { table } => {
                    vec![Cmd::Raw(format!("loot give @s loot {table}"))]
                }
                Action::DropLoot { table } => {
                    vec![Cmd::Raw(format!("loot spawn ~ ~ ~ loot {table}"))]
                }
                Action::EntityTag { tag, add } => vec![Cmd::Raw(format!(
                    "tag @s {} {tag}",
                    if *add { "add" } else { "remove" }
                ))],
                Action::Summon { entity_type, tag } => vec![Cmd::Raw(match tag {
                    Some(tag) => format!("summon {entity_type} ~ ~ ~ {{Tags:[\"{tag}\"]}}"),
                    None => format!("summon {entity_type} ~ ~ ~"),
                })],
                Action::Damage { amount } => vec![Cmd::Raw(format!("damage @s {amount}"))],
                Action::CountEntities { score, query } => vec![Cmd::Raw(format!(
                    "execute store result score {} {} if entity {}",
                    self.holders[&score.state.to_string()],
                    objective(score),
                    query_selector(query)
                ))],
                Action::While { condition, body } => {
                    let looped = helper(&self.namespace, &self.helper_keys[owner], &path, "loop");
                    let mut lines = self.body(
                        body,
                        owner,
                        &format!("{path}/loop"),
                        module,
                        &format!("{ptr}/action/body"),
                    )?;
                    let test = self.condition_test(condition).map_err(fail)?;
                    // Tail call: re-check the condition after each pass and
                    // forward an early return from deeper passes.
                    lines.push(
                        Cmd::Execute {
                            operations: vec![sand_commands::ExecuteOp::If(test.clone())],
                            run: Box::new(Cmd::ReturnRun(Box::new(Cmd::Function(looped.clone())))),
                        }
                        .render(),
                    );
                    self.function(&looped, lines, module, &ptr)?;
                    let mut operations = vec![sand_commands::ExecuteOp::If(test)];
                    let run = if returns(body) {
                        operations.push(sand_commands::ExecuteOp::If(
                            sand_commands::ConditionIr::Function(looped),
                        ));
                        Cmd::Return(1)
                    } else {
                        Cmd::Function(looped)
                    };
                    vec![Cmd::Execute {
                        operations,
                        run: Box::new(run),
                    }]
                }
                Action::AttributeGet {
                    score,
                    attribute,
                    scale,
                } => vec![Cmd::Execute {
                    operations: vec![sand_commands::ExecuteOp::StoreResult(
                        sand_commands::ExecuteStoreTarget::Score {
                            holder: self.holders[&score.state.to_string()].clone(),
                            objective: objective(score),
                        },
                    )],
                    run: Box::new(Cmd::Raw(format!(
                        "attribute @s {attribute} base get {scale}"
                    ))),
                }],
                Action::AttributeSet { attribute, value } => match value {
                    Amount::Value(value) => {
                        vec![Cmd::Raw(format!(
                            "attribute @s {attribute} base set {value}"
                        ))]
                    }
                    Amount::Field(field) => {
                        // Minecraft cannot read a score as a command argument;
                        // copy it into storage and apply it through a macro.
                        let storage = format!("{}:sand.args", self.namespace);
                        let id = helper(
                            &self.namespace,
                            &self.helper_keys[owner],
                            &path,
                            "attribute",
                        );
                        self.function(
                            &id,
                            vec![format!("$attribute @s {attribute} base set $(value)")],
                            module,
                            &ptr,
                        )?;
                        vec![
                            Cmd::Execute {
                                operations: vec![sand_commands::ExecuteOp::Raw(format!(
                                    "store result storage {storage} value double {}",
                                    1.0 / field.scale
                                ))],
                                run: Box::new(Cmd::Raw(format!(
                                    "scoreboard players get {} {}",
                                    self.holders[&field.score.state.to_string()],
                                    objective(&field.score)
                                ))),
                            },
                            Cmd::Raw(format!("function {id} with storage {storage}")),
                        ]
                    }
                },
            };
            for cmd in cmds {
                let rendered = cmd.try_render().map_err(|e| fail(e.to_string()))?;
                let rendered =
                    sand_commands::render::validate_standalone_line(&rendered, &self.profile)
                        .map_err(|e| fail(e.to_string()))?;
                commands.push(rendered);
            }
        }
        Ok(commands)
    }
}
