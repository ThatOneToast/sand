use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Discover the complete feature-name union selected by enabling every feature
/// of a facade crate, including features forwarded to local path dependencies.
///
/// The surface extractor currently evaluates one conservative feature set for
/// all source crates. Returning the union is therefore intentional: a feature
/// name enabled in any API-producing crate must make its `cfg(feature = ...)`
/// declarations visible to the audit.
pub fn discover_facade_feature_union(manifest: &Path) -> Result<BTreeSet<String>, String> {
    let mut manifests = BTreeMap::<PathBuf, Manifest>::new();
    load_manifest(manifest, &mut manifests)?;
    let root = canonical(manifest)?;
    let mut active = BTreeSet::<(PathBuf, String)>::new();
    let mut pending = manifests[&root]
        .features
        .keys()
        .map(|feature| (root.clone(), feature.clone()))
        .collect::<Vec<_>>();

    while let Some((manifest_path, feature)) = pending.pop() {
        if !active.insert((manifest_path.clone(), feature.clone())) {
            continue;
        }
        let manifest_data = manifests.get(&manifest_path).cloned().ok_or_else(|| {
            format!(
                "feature references unknown manifest {}",
                manifest_path.display()
            )
        })?;
        let Some(values) = manifest_data.features.get(&feature) else {
            // Cargo permits dependency features that are not declared locally;
            // they are queued against the dependency manifest below instead.
            continue;
        };
        for value in values {
            if let Some((dependency, dependency_feature)) = forwarded_feature(value) {
                let Some(dependency_manifest) = manifest_data.dependencies.get(dependency) else {
                    return Err(format!(
                        "{}: feature `{feature}` forwards `{value}` to an unknown local path dependency",
                        manifest_path.display()
                    ));
                };
                load_manifest(dependency_manifest, &mut manifests)?;
                pending.push((
                    canonical(dependency_manifest)?,
                    dependency_feature.to_owned(),
                ));
            } else if !value.starts_with("dep:") && manifest_data.features.contains_key(value) {
                pending.push((manifest_path.clone(), value.clone()));
            }
        }
    }

    Ok(active.into_iter().map(|(_, feature)| feature).collect())
}

#[derive(Clone)]
struct Manifest {
    features: BTreeMap<String, Vec<String>>,
    dependencies: BTreeMap<String, PathBuf>,
}

fn load_manifest(path: &Path, manifests: &mut BTreeMap<PathBuf, Manifest>) -> Result<(), String> {
    let path = canonical(path)?;
    if manifests.contains_key(&path) {
        return Ok(());
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let value = source
        .parse::<toml::Value>()
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    let features = value
        .get("features")
        .and_then(toml::Value::as_table)
        .map(|table| {
            table
                .iter()
                .map(|(name, values)| {
                    let values = values
                        .as_array()
                        .ok_or_else(|| {
                            format!("{}: feature `{name}` must be an array", path.display())
                        })?
                        .iter()
                        .map(|value| {
                            value.as_str().map(str::to_owned).ok_or_else(|| {
                                format!(
                                    "{}: feature `{name}` contains a non-string value",
                                    path.display()
                                )
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok((name.clone(), values))
                })
                .collect::<Result<BTreeMap<_, _>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    let mut dependencies = BTreeMap::new();
    for section in ["dependencies", "build-dependencies"] {
        let Some(table) = value.get(section).and_then(toml::Value::as_table) else {
            continue;
        };
        for (name, dependency) in table {
            let Some(relative) = dependency
                .as_table()
                .and_then(|dependency| dependency.get("path"))
                .and_then(toml::Value::as_str)
            else {
                continue;
            };
            let dependency_manifest = path
                .parent()
                .expect("Cargo.toml has a parent")
                .join(relative)
                .join("Cargo.toml");
            dependencies.insert(name.clone(), dependency_manifest);
        }
    }
    manifests.insert(
        path,
        Manifest {
            features,
            dependencies,
        },
    );
    Ok(())
}

fn forwarded_feature(value: &str) -> Option<(&str, &str)> {
    let (dependency, feature) = value.split_once('/')?;
    Some((dependency.strip_suffix('?').unwrap_or(dependency), feature))
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_new_facade_features_and_forwarded_dependency_closure() {
        let temp = tempfile::tempdir().unwrap();
        let facade = temp.path().join("facade");
        let core = temp.path().join("core");
        fs::create_dir_all(&facade).unwrap();
        fs::create_dir_all(&core).unwrap();
        fs::write(
            facade.join("Cargo.toml"),
            r#"
                [package]
                name = "facade"
                version = "0.0.0"
                [dependencies]
                core = { path = "../core" }
                [features]
                newly-added = ["core/forwarded"]
            "#,
        )
        .unwrap();
        fs::write(
            core.join("Cargo.toml"),
            r#"
                [package]
                name = "core"
                version = "0.0.0"
                [features]
                forwarded = ["lower-level"]
                lower-level = []
                unsupported = []
            "#,
        )
        .unwrap();

        assert_eq!(
            discover_facade_feature_union(&facade.join("Cargo.toml")).unwrap(),
            BTreeSet::from([
                "forwarded".to_owned(),
                "lower-level".to_owned(),
                "newly-added".to_owned(),
            ])
        );
    }
}
