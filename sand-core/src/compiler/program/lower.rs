//! Typed gameplay lowering with local, deterministic helper allocation.
use super::{diagnostic::Diagnostic, limits, model::*};
use crate::compiler::export::{
    identities, lifecycle,
    records::{self, ComponentRecord},
    tags,
};
use crate::ir::{Cmd, ScorePlayersOp};
use std::collections::BTreeMap;

pub(super) fn lower(
    program: &Program,
    target: &crate::version::VersionProfile,
) -> Result<BTreeMap<String, Vec<u8>>, Vec<Diagnostic>> {
    let error = |error: String| vec![Diagnostic::error("SAND_PROGRAM_LOWER", "", "", error)];
    let states: Vec<_> = program
        .modules
        .iter()
        .flat_map(|module| module.states.iter().cloned())
        .collect();
    let automatic = crate::state::owned::player_lifecycle(&states).map_err(|message| {
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
        helper_keys,
        records: Vec::new(),
        init,
        bytes: 0,
        profile: sand_commands::CommandProfile::new(target.resolved_name(), false),
    };
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
        memberships.extend(
            module
                .tags
                .iter()
                .map(|tag| (tag.tag.to_string(), reference_id(&tag.function))),
        );
    }
    let mut tag_map = BTreeMap::new();
    let player_initializer = lifecycle::assemble_lifecycle(
        &program.pack.namespace,
        &mut lower.records,
        &mut tag_map,
        automatic,
        load,
        tick,
        Vec::new(),
    )
    .map_err(|e| error(e.to_string()))?;
    lifecycle::initialize_player_entries(
        &mut lower.records,
        &entries,
        player_initializer.as_deref(),
    );
    tags::assemble_tags(
        &program.pack.namespace,
        &mut lower.records,
        tag_map,
        memberships,
    );
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
    Ok(output)
}

fn insert_output(
    output: &mut BTreeMap<String, Vec<u8>>,
    path: String,
    bytes: Vec<u8>,
) -> Result<(), Vec<Diagnostic>> {
    if path.split('/').any(|segment| segment.len() > 255) {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_PATH",
            "",
            "",
            format!("generated resource path segment exceeds 255 bytes: `{path}`"),
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
struct Lower {
    namespace: String,
    helper_keys: BTreeMap<String, String>,
    records: Vec<ComponentRecord>,
    init: Option<String>,
    bytes: usize,
    profile: sand_commands::CommandProfile,
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
            let cmd = match &operation.action {
                Action::ScoreSet { score, value } => Cmd::ScorePlayers(ScorePlayersOp::Set {
                    selector: "@s".into(),
                    objective: objective(score),
                    value: *value,
                }),
                Action::ScoreAdd { score, value } => Cmd::ScorePlayers(ScorePlayersOp::Add {
                    selector: "@s".into(),
                    objective: objective(score),
                    amount: *value,
                }),
                Action::Call { function } => Cmd::Function(reference_id(function)),
                Action::Raw { command } => Cmd::Raw(command.clone()),
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
                    Cmd::Execute {
                        operations: vec![sand_commands::ExecuteOp::As(
                            sand_commands::Selector::all_players(),
                        )],
                        run: Box::new(Cmd::Function(target)),
                    }
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
                    let range = format!(
                        "{}..{}",
                        condition.min.unwrap_or(i32::MIN),
                        condition.max.unwrap_or(i32::MAX)
                    );
                    let guard = Cmd::Execute {
                        operations: vec![sand_commands::ExecuteOp::If(
                            sand_commands::ConditionIr::ScoreMatches {
                                holder: sand_commands::ScoreHolder::self_(),
                                objective: objective(&condition.score),
                                range,
                            },
                        )],
                        run: Box::new(Cmd::ReturnRun(Box::new(Cmd::Function(yes)))),
                    };
                    self.function(
                        &decision,
                        vec![guard.render(), Cmd::Function(no).render()],
                        module,
                        &ptr,
                    )?;
                    Cmd::Function(decision)
                }
            };
            let fail = |error: String| {
                let mut error = Diagnostic::error("SAND_PROGRAM_COMMAND", module, &ptr, error);
                error.origin = operation.origin.clone();
                vec![error]
            };
            let rendered = cmd.try_render().map_err(|e| fail(e.to_string()))?;
            let rendered =
                sand_commands::render::validate_standalone_line(&rendered, &self.profile)
                    .map_err(|e| fail(e.to_string()))?;
            commands.push(rendered);
        }
        Ok(commands)
    }
}
