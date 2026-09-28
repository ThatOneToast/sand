//! Whole-program validation before any output allocation or publication.
use super::{diagnostic::Diagnostic, limits, model::*};
use sand_components::ResourceLocation;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    program: &Program,
) -> Result<crate::version::VersionProfile, Vec<Diagnostic>> {
    let mut errors = Vec::new();
    if program.format != "sand.program" || program.format_version != 1 {
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
        if ![
            "player_state",
            "functions",
            "score",
            "branch",
            "players",
            "calls",
            "lifecycle",
            "function_tags",
            "raw_commands",
        ]
        .contains(&capability.as_str())
        {
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
            if state.scope != "player" || state.revision != 1 {
                errors.push(Diagnostic::error(
                    "SAND_PROGRAM_STATE",
                    &owner,
                    &ptr,
                    "v1 requires player State revision 1",
                ));
            }
            let mut fields = BTreeSet::new();
            for (index, field) in state.fields.iter().enumerate() {
                if field.default != 0
                    || matches!(
                        field.name.as_str(),
                        "presence" | "suppressed" | "numeric_scratch"
                    )
                    || field.name.is_empty()
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
                        "fields require unique simple names, zero integer defaults, and cannot claim State ownership objectives",
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
    let mut validator = Validator {
        states: states
            .into_iter()
            .map(|(id, state)| {
                (
                    id,
                    state
                        .fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect(),
                )
            })
            .collect(),
        functions,
        operations: 0,
        raw: program.requires.iter().any(|v| v == "raw_commands"),
        errors: &mut errors,
    };
    for module in &program.modules {
        let owner = module.id.to_string();
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
    states: BTreeMap<String, BTreeSet<&'a str>>,
    functions: BTreeMap<String, ExecutionContext>,
    operations: usize,
    raw: bool,
    errors: &'e mut Vec<Diagnostic>,
}
impl Validator<'_, '_> {
    fn score(&mut self, score: &FieldReference, context: ExecutionContext, owner: &str, ptr: &str) {
        if context != ExecutionContext::Player {
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
            .is_some_and(|fields| fields.contains(score.field.as_str()))
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
                    self.score(
                        &condition.score,
                        context,
                        owner,
                        &format!("{ptr}/action/condition/score"),
                    );
                    if condition
                        .min
                        .zip(condition.max)
                        .is_some_and(|(min, max)| min > max)
                    {
                        self.errors.push(Diagnostic::error(
                            "SAND_PROGRAM_RANGE",
                            owner,
                            &format!("{ptr}/action/condition"),
                            "comparison minimum exceeds maximum",
                        ));
                    }
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
