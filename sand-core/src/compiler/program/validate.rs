//! Whole-program validation before any output allocation or publication.
use super::{diagnostic::Diagnostic, limits, model::*};
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
                        .map(|state| state.fields.len())
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
                || state.revision != 1
            {
                errors.push(Diagnostic::error(
                    "SAND_PROGRAM_STATE",
                    &owner,
                    &ptr,
                    "v1 requires player, entity, or global State revision 1",
                ));
            }
            let mut fields = BTreeSet::new();
            for (index, field) in state.fields.iter().enumerate() {
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
        functions,
        operations: 0,
        flow_bytes: 0,
        raw: program.requires.iter().any(|v| v == "raw_commands"),
        errors: &mut errors,
    };
    for module in &program.modules {
        let owner = module.id.to_string();
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
        for (index, hook) in module.events.iter().enumerate() {
            let ptr = format!("/events/{index}");
            if let Some((id, kind)) = hook.event.filter() {
                validator.resource(id, &owner, &format!("{ptr}/event"), kind);
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
        let item_id = |id: &str| {
            id.strip_prefix('#').is_none()
                && id.parse::<ResourceLocation>().is_ok()
                && id.contains(':')
        };
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
            if item
                .name
                .iter()
                .chain(item.lore.iter().flatten())
                .any(|part| !matches!(part, TextPart::Text { .. }))
            {
                push(
                    ptr.clone(),
                    "item names and lore are static text; scores and player names are not allowed"
                        .into(),
                );
            }
            if item.max_stack.is_some_and(|n| !(1..=99).contains(&n)) {
                push(
                    format!("{ptr}/max_stack"),
                    "max stack must be 1..=99".into(),
                );
            }
            for (enchantment, level) in &item.enchantments {
                if !item_id(enchantment) || !(1..=255).contains(level) {
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
                    result
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
                    result
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
                    result
                }
                Recipe::Stonecutting {
                    ingredient, result, ..
                } => {
                    inputs.push((format!("{ptr}/ingredient"), ingredient));
                    result
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
                } else if !ingredient(item) {
                    push(input_ptr, format!("`{item}` is not an item id or `#tag`"));
                }
            }
            if !(item_id(&result.item)) || !(1..=99).contains(&result.count) {
                push(
                    format!("{ptr}/result"),
                    "a recipe result is an item id with a count of 1..=99".into(),
                );
            }
        }
        for (index, tag) in module.resource_tags.iter().enumerate() {
            let ptr = format!("/resource_tags/{index}");
            if !resource_ids.insert(format!("tag {:?} {}", tag.kind, tag.id)) {
                push(format!("{ptr}/id"), format!("duplicate tag `{}`", tag.id));
            }
            for (i, value) in tag.values.iter().enumerate() {
                if !ingredient(value) || custom_items.contains(value.as_str()) {
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

struct Validator<'a, 'e> {
    states: BTreeMap<String, (&'a str, BTreeSet<&'a str>)>,
    functions: BTreeMap<String, ExecutionContext>,
    operations: usize,
    flow_bytes: usize,
    raw: bool,
    errors: &'e mut Vec<Diagnostic>,
}
impl Validator<'_, '_> {
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

    fn resource(&mut self, id: &str, owner: &str, ptr: &str, kind: &str) {
        if id.parse::<ResourceLocation>().is_err() || !id.contains(':') {
            self.errors.push(Diagnostic::error(
                "SAND_PROGRAM_ID",
                owner,
                ptr,
                format!("`{id}` is not a namespaced {kind} identity such as `minecraft:example`"),
            ));
        }
    }

    fn score(&mut self, score: &FieldReference, context: ExecutionContext, owner: &str, ptr: &str) {
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
                    channel: _,
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
                    for (index, part) in text.iter().enumerate() {
                        let part_ptr = format!("{ptr}/action/text/{index}");
                        match part {
                            TextPart::Score { score, .. } => {
                                self.score(score, context, owner, &format!("{part_ptr}/score"))
                            }
                            TextPart::PlayerName { .. } => self.player(
                                context,
                                owner,
                                &part_ptr,
                                "the executing player's name",
                            ),
                            TextPart::Text { .. } => {}
                        }
                    }
                }
                Action::Sound {
                    sound,
                    volume,
                    pitch,
                } => {
                    self.player(context, owner, &ptr, "a sound for the executing player");
                    self.resource(sound, owner, &format!("{ptr}/action/sound"), "sound");
                    let valid_volume = volume.is_finite() && *volume >= 0.0;
                    let valid_pitch = pitch.is_finite() && (0.0..=2.0).contains(pitch);
                    if !(valid_volume && valid_pitch) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action"),
                            "sound volume must be at least 0 and pitch must be 0.0..=2.0",
                        ));
                    }
                }
                Action::Particle {
                    particle,
                    spread,
                    speed,
                    ..
                } => {
                    self.resource(
                        particle,
                        owner,
                        &format!("{ptr}/action/particle"),
                        "particle",
                    );
                    if !(spread.is_finite() && *spread >= 0.0 && speed.is_finite() && *speed >= 0.0)
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action"),
                            "particle spread and speed must be finite and non-negative",
                        ));
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
                    if *count == 0 || *count > 6400 {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/count"),
                            "item count must be 1..=6400",
                        ));
                    }
                }
                Action::ClearItem { item, .. } => {
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "clearing the executing player's items",
                    );
                    if let Some(item) = item {
                        self.resource(item, owner, &format!("{ptr}/action/item"), "item");
                    }
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
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "reading the executing player's attribute",
                    );
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
                    self.player(
                        context,
                        owner,
                        &ptr,
                        "setting the executing player's attribute",
                    );
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
                Action::Kill => self.executor(context, owner, &ptr, "killing the executor"),
                Action::Damage { amount } => {
                    self.executor(context, owner, &ptr, "damaging the executor");
                    if !(amount.is_finite() && *amount > 0.0) {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/amount"),
                            "damage must be finite and positive",
                        ));
                    }
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
                Action::Teleport { destination } => {
                    self.executor(context, owner, &ptr, "teleporting the executor");
                    let finite = match destination {
                        TeleportDestination::Position { x, y, z }
                        | TeleportDestination::Offset { x, y, z } => {
                            x.is_finite() && y.is_finite() && z.is_finite()
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
