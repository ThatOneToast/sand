//! Dynamic function collection phase of the export pipeline.
//!
//! Drains anonymous/branch functions registered while user factories ran and
//! resolves the `__sand_local` namespace sentinel to the real pack namespace.

use super::records::ComponentRecord;

/// Drain all dynamically-registered branch/anonymous functions into `records`.
///
/// Loops until the registry is empty so that branches registered *by* other
/// branches (nested mcfunction! blocks) are also captured.
pub(crate) fn drain_dynamic_functions_into(records: &mut Vec<ComponentRecord>, namespace: &str) {
    loop {
        let drained = crate::drain_dyn_fns();
        if drained.is_empty() {
            break;
        }
        for (path, commands) in drained {
            records.push(ComponentRecord {
                namespace: namespace.to_string(),
                dir: "function".to_string(),
                path,
                ext: "mcfunction".to_string(),
                content_type: "text".to_string(),
                content: commands.join("\n"),
            });
        }
    }
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
