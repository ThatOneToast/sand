//! Export-local, collision-safe allocation for generated Minecraft identities.
//!
//! Callers supply the resource-specific base name, validation, and diagnostic.
//! This module supplies the part every resource kind must agree on: owners are
//! deduplicated and sorted, the unsalted candidate is tried first, collisions
//! use deterministic probes, and no process-global allocator state survives an
//! export or test.

use std::collections::{BTreeMap, BTreeSet};

use super::records::ExportResult;
use crate::component::ComponentExportError;

/// Number of deterministic candidates available to one logical owner.
pub(crate) const IDENTITY_PROBE_LIMIT: u32 = 64;

/// Allocate one unique generated key for each logical owner.
pub(crate) fn allocate_collision_safe_keys<'a, I, S, V, C>(
    owners: I,
    source: S,
    validate: V,
    collision: C,
) -> ExportResult<BTreeMap<String, String>>
where
    I: IntoIterator<Item = &'a str>,
    S: Fn(&str, u32) -> String,
    V: Fn(&str, &str) -> ExportResult<()>,
    C: Fn(&str, &str, &str) -> ComponentExportError,
{
    let unique: BTreeSet<&str> = owners.into_iter().collect();
    let mut claimed: BTreeMap<String, &str> = BTreeMap::new();
    let mut allocated = BTreeMap::new();

    for owner in unique {
        let mut chosen = None;
        let mut first_collision = None;
        for attempt in 0..IDENTITY_PROBE_LIMIT {
            let key = source(owner, attempt);
            validate(owner, &key)?;
            if let Some(previous) = claimed.get(&key) {
                first_collision.get_or_insert((*previous, key));
            } else {
                chosen = Some(key);
                break;
            }
        }
        let key = chosen.ok_or_else(|| {
            let (previous, key) = first_collision.expect("the probe limit is non-zero");
            collision(owner, previous, &key)
        })?;
        claimed.insert(key.clone(), owner);
        allocated.insert(owner.to_owned(), key);
    }
    Ok(allocated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(owner: &str, previous: &str, key: &str) -> ComponentExportError {
        ComponentExportError::ComponentValidation {
            location: sand_components::ResourceLocation::new("sand", "identity").unwrap(),
            kind: "identity".into(),
            field: key.into(),
            message: format!("{owner} conflicts with {previous}"),
        }
    }

    #[test]
    fn allocation_is_order_independent_and_export_local() {
        let source = |owner: &str, attempt| {
            if attempt == 0 {
                "same".into()
            } else {
                format!("{owner}-{attempt}")
            }
        };
        let validate = |_: &str, _: &str| Ok(());
        let forward =
            allocate_collision_safe_keys(["alpha", "beta"], source, validate, error).unwrap();
        let reverse =
            allocate_collision_safe_keys(["beta", "alpha"], source, validate, error).unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(forward["alpha"], "same");
        assert_eq!(forward["beta"], "beta-1");
    }
}
