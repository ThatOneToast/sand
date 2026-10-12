//! Dynamic function collection phase of the export pipeline.
//!
//! Drains anonymous/branch functions registered while user factories ran and
//! resolves the `__sand_local` namespace sentinel to the real pack namespace.

use super::records::ComponentRecord;

/// Drain all dynamically-registered branch/anonymous functions into `records`.
///
/// Loops until the registry is empty so that branches registered *by* other
/// branches (nested mcfunction! blocks) are also captured.
pub(crate) fn drain_dynamic_functions_into(
    records: &mut Vec<ComponentRecord>,
    namespace: &str,
) -> super::records::ExportResult<()> {
    let mut emitted = std::collections::BTreeSet::new();
    loop {
        let drained = crate::drain_dyn_fns();
        if drained.is_empty() {
            break;
        }
        for (path, commands) in drained {
            // Lowering a parent can re-register a helper drained in an earlier
            // batch. Deduplicate only the same owned body; distinct bodies at
            // one path still reach validation and the output collision guard.
            if !emitted.insert((path.clone(), commands.identity())) {
                continue;
            }
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path: path.clone(),
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: commands
                    .lower(&sand_components::ResourceLocation::new(namespace, &path)?)?
                    .join("\n"),
            });
        }
    }
    Ok(())
}

/// Replace every `__sand_local:<path>` sentinel in an mcfunction content string
/// with `<namespace>:<path>`.
///
/// Handles both patterns:
/// - `function __sand_local:path` — bare function pointer calls
/// - `... only __sand_local:path` — advancement revoke/grant from EventHandle
pub(crate) fn resolve_local_refs(content: &str, namespace: &str) -> String {
    let sentinel = crate::function::SAND_LOCAL_NS;
    content.replace(&format!("{sentinel}:"), &format!("{namespace}:"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_with_different_ir_still_validates_each_body() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        crate::register_dyn_fn("shared".into(), "say valid");
        crate::register_dyn_fn(
            "shared".into(),
            crate::ir::Actions(vec![crate::ir::Cmd::Execute {
                operations: vec![],
                run: Box::new(crate::ir::Cmd::Raw("say invalid".into())),
            }]),
        );
        let error = drain_dynamic_functions_into(&mut Vec::new(), "test").unwrap_err();
        assert!(error.to_string().contains("SAND-COMMAND-EXECUTE-EMPTY"));
    }
}
