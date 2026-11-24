//! Map emitted commands and portable locations to authored source.
use super::*;
use serde_json::{Value, json};

impl Analysis {
    /// A versioned source map for the lowered program.
    ///
    /// Entries identify portable module pointers and emitted command ranges.
    /// Source offsets are UTF-8 bytes. Generated line numbers are one-based.
    /// Build failures report ordinary source diagnostics.
    pub fn source_map(&self) -> Result<Value, Vec<Diagnostic>> {
        let Some(program) = &self.program else {
            return Err(self.diagnostics.clone());
        };
        let (_, commands) =
            pc::Compiler::compile_with_command_sources(program).map_err(|errors| {
                errors
                    .iter()
                    .map(|error| self.map_backend(error))
                    .collect::<Vec<_>>()
            })?;
        let directory = self
            .package
            .root
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""));
        let files: Vec<_> = self
            .package
            .files
            .iter()
            .map(|file| {
                file.path
                    .strip_prefix(directory)
                    .unwrap_or(&file.path)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        let locations: BTreeMap<_, _> = self
            .pointers
            .iter()
            .map(|(module, pointer, file, span)| {
                ((module.as_str(), pointer.as_str()), (*file, *span))
            })
            .collect();
        let mut entries = Vec::new();
        for (module, pointer, file, span) in &self.pointers {
            entries.push(json!({
                "module": module,
                "pointer": pointer,
                "source": origin(*file, *span),
            }));
        }
        let mut functions = Vec::new();
        for module in &program.modules {
            let id = module.id.to_string();
            map_body(&mut entries, &id, "/load", &module.load);
            map_body(&mut entries, &id, "/tick", &module.tick);
            for (index, function) in module.functions.iter().enumerate() {
                let pointer = format!("/functions/{index}");
                let source = locations
                    .get(&(id.as_str(), pointer.as_str()))
                    .map(|(file, span)| origin(*file, *span));
                functions.push(json!({
                    "id": function.id,
                    "module": id,
                    "pointer": pointer,
                    "resource": format!("data/{}/function/{}.mcfunction", function.id.namespace(), function.id.path()),
                    "source": source,
                }));
                map_body(
                    &mut entries,
                    &id,
                    &format!("{pointer}/body"),
                    &function.body,
                );
            }
        }
        let source_by_pointer: BTreeMap<_, _> = entries
            .iter()
            .filter_map(|entry| {
                Some((
                    (entry["module"].as_str()?, entry["pointer"].as_str()?),
                    &entry["source"],
                ))
            })
            .collect();
        let commands: BTreeMap<_, _> = commands
            .functions
            .into_iter()
            .filter_map(|(resource, locations)| {
                let ranges: Vec<_> = locations
                    .into_iter()
                    .filter_map(|location| {
                        let source = source_by_pointer
                            .get(&(location.module.as_str(), location.pointer.as_str()))?;
                        Some(json!({
                            "lines": location.lines,
                            "module": location.module,
                            "pointer": location.pointer,
                            "source": source,
                        }))
                    })
                    .collect();
                (!ranges.is_empty()).then_some((resource, ranges))
            })
            .collect();
        Ok(json!({
            "format": "sand.source_map",
            "format_version": 2,
            "files": files,
            "functions": functions,
            "entries": entries,
            "commands": commands,
        }))
    }
}

fn map_body(entries: &mut Vec<Value>, module: &str, parent: &str, body: &[pc::Operation]) {
    for (index, operation) in body.iter().enumerate() {
        let pointer = format!("{parent}/{index}");
        if let Some(source) = &operation.origin {
            entries.push(json!({ "module": module, "pointer": pointer, "source": source }));
        }
        match &operation.action {
            pc::Action::Branch {
                then, otherwise, ..
            } => {
                map_body(entries, module, &format!("{pointer}/action/then"), then);
                map_body(
                    entries,
                    module,
                    &format!("{pointer}/action/otherwise"),
                    otherwise,
                );
            }
            pc::Action::Players { body }
            | pc::Action::Entities { body, .. }
            | pc::Action::WithEntity { body, .. }
            | pc::Action::While { body, .. }
            | pc::Action::Suspend { body, .. } => {
                map_body(entries, module, &format!("{pointer}/action/body"), body);
            }
            _ => {}
        }
    }
}
