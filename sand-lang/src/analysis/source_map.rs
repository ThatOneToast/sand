//! Map portable function identities and operation pointers to authored source.
use super::*;
use serde_json::{Value, json};

impl Analysis {
    /// A versioned source map for the lowered program.
    ///
    /// Entries identify a portable module and JSON pointer, and functions
    /// include their emitted resource path. Source ranges are UTF-8 byte offsets.
    pub fn source_map(&self) -> Option<Value> {
        let program = self.program.as_ref()?;
        let directory = self.package.root.parent()?;
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
        Some(json!({
            "format": "sand.source_map",
            "format_version": 1,
            "files": files,
            "functions": functions,
            "entries": entries,
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
            | pc::Action::While { body, .. } => {
                map_body(entries, module, &format!("{pointer}/action/body"), body);
            }
            _ => {}
        }
    }
}
