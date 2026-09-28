//! Strict, bounded decoding and explicit in-memory module resolution.
use super::{diagnostic::Diagnostic, limits, model::*};
use serde::Deserialize;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::collections::{BTreeMap, BTreeSet};

struct StrictValue<'a> {
    depth: usize,
    pointer: String,
    error_pointer: &'a std::cell::RefCell<String>,
}
impl StrictValue<'_> {
    fn child(&self, part: &str) -> StrictValue<'_> {
        StrictValue {
            depth: self.depth + 1,
            pointer: format!(
                "{}/{}",
                self.pointer,
                part.replace('~', "~0").replace('/', "~1")
            ),
            error_pointer: self.error_pointer,
        }
    }
}
impl<'de> DeserializeSeed<'de> for StrictValue<'_> {
    type Value = serde_json::Value;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        if self.depth > limits::JSON_DEPTH {
            *self.error_pointer.borrow_mut() = self.pointer.clone();
            return Err(serde::de::Error::custom("JSON depth limit exceeded"));
        }
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for StrictValue<'_> {
    type Value = serde_json::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("strict JSON without duplicate keys")
    }
    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(v)
            .map(Into::into)
            .ok_or_else(|| E::custom("invalid number"))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(v.into())
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Null)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(self.child(&values.len().to_string()))? {
            values.push(value);
        }
        Ok(values.into())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                *self.error_pointer.borrow_mut() = self.child(&key).pointer;
                return Err(serde::de::Error::custom(format!(
                    "duplicate object key: {key}"
                )));
            }
            let value = map.next_value_seed(self.child(&key))?;
            values.insert(key, value);
        }
        Ok(values.into())
    }
}

fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8], name: &str) -> Result<T, Vec<Diagnostic>> {
    let fail = |message: String| vec![Diagnostic::error("SAND_PROGRAM_DECODE", name, "", message)];
    if bytes.len() > limits::DOCUMENT_BYTES {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            name,
            "",
            "document exceeds 1 MiB",
        )]);
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let error_pointer = std::cell::RefCell::new(String::new());
    let value = StrictValue {
        depth: 0,
        pointer: String::new(),
        error_pointer: &error_pointer,
    }
    .deserialize(&mut decoder)
    .map_err(|error| {
        vec![Diagnostic::error(
            if error.to_string().contains("depth limit") {
                "SAND_PROGRAM_LIMIT"
            } else {
                "SAND_PROGRAM_DECODE"
            },
            name,
            &error_pointer.borrow(),
            error,
        )]
    })?;
    decoder.end().map_err(|e| fail(e.to_string()))?;
    decode_value(value, name)
}

fn decode_value<T: for<'de> Deserialize<'de>>(
    value: serde_json::Value,
    name: &str,
) -> Result<T, Vec<Diagnostic>> {
    // A path-aware deserializer retains the exact field location after strict
    // duplicate-key detection has finished.
    serde_path_to_error::deserialize(&value).map_err(|error| {
        let pointer = error
            .path()
            .iter()
            .map(|part| match part {
                serde_path_to_error::Segment::Seq { index } => format!("/{index}"),
                serde_path_to_error::Segment::Map { key } => {
                    format!("/{}", key.replace('~', "~0").replace('/', "~1"))
                }
                _ => String::new(),
            })
            .collect::<String>();
        let mut diagnostic =
            Diagnostic::error("SAND_PROGRAM_DECODE", name, &pointer, error.inner());
        let mut parent = pointer.as_str();
        loop {
            if let Some(origin) = value.pointer(parent).and_then(|node| node.get("origin")) {
                diagnostic.origin = Some(origin.clone());
                break;
            }
            let Some((prefix, _)) = parent.rsplit_once('/') else {
                break;
            };
            parent = prefix;
        }
        vec![diagnostic]
    })
}

pub(super) fn resolve(
    bytes: &[u8],
    modules: &BTreeMap<String, Vec<u8>>,
) -> Result<Program, Vec<Diagnostic>> {
    if bytes
        .len()
        .saturating_add(modules.values().map(Vec::len).sum::<usize>())
        > limits::INPUT_BYTES
    {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            "",
            "",
            "program exceeds 8 MiB",
        )]);
    }
    let input: Program<serde_json::Value> = decode(bytes, "")?;
    if input.modules.len() > limits::MODULES {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            "",
            "/modules",
            "more than 64 modules",
        )]);
    }
    let mut resolved = Vec::new();
    let mut paths = BTreeSet::new();
    for (index, source) in input.modules.into_iter().enumerate() {
        match source {
            value @ serde_json::Value::Object(_) => {
                let owner = value
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                resolved.push(decode_value(value, &owner)?);
            }
            serde_json::Value::String(path) => {
                if !safe_module_path(&path) || !paths.insert(path.clone()) {
                    return Err(vec![Diagnostic::error(
                        "SAND_PROGRAM_MODULE",
                        "",
                        &format!("/modules/{index}"),
                        "unsafe or duplicate module path",
                    )]);
                }
                let bytes = modules.get(&path).ok_or_else(|| {
                    vec![Diagnostic::error(
                        "SAND_PROGRAM_MODULE",
                        &path,
                        "",
                        "missing explicitly supplied module",
                    )]
                })?;
                resolved.push(decode(bytes, &path)?);
            }
            _ => {
                return Err(vec![Diagnostic::error(
                    "SAND_PROGRAM_DECODE",
                    "",
                    &format!("/modules/{index}"),
                    "module must be an object or local path",
                )]);
            }
        }
    }
    Ok(Program {
        format: input.format,
        format_version: input.format_version,
        target: input.target,
        pack: input.pack,
        requires: input.requires,
        modules: resolved,
    })
}

pub(crate) fn safe_module_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

pub(super) fn module_paths(bytes: &[u8]) -> Result<Vec<String>, Vec<Diagnostic>> {
    let input: Program<serde_json::Value> = decode(bytes, "")?;
    if input.modules.len() > limits::MODULES {
        return Err(vec![Diagnostic::error(
            "SAND_PROGRAM_LIMIT",
            "",
            "/modules",
            "more than 64 modules",
        )]);
    }
    let mut paths = std::collections::BTreeSet::new();
    let mut result = Vec::new();
    for (index, module) in input.modules.into_iter().enumerate() {
        if let serde_json::Value::String(path) = module {
            if !safe_module_path(&path) || !paths.insert(path.clone()) {
                return Err(vec![Diagnostic::error(
                    "SAND_PROGRAM_MODULE",
                    "",
                    &format!("/modules/{index}"),
                    "unsafe or duplicate module path",
                )]);
            }
            result.push(path);
        }
    }
    Ok(result)
}
