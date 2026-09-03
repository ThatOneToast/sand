//! Explicit raw escape hatch types for datapack interop.
//!
//! These types make raw/unsafe datapack values **visibly named** at every call
//! site.  Normal typed APIs should never accept or return these types — they
//! exist solely for advanced users who need to target:
//!
//! - Modded additions not yet modelled in Sand's typed API.
//! - Features added to Minecraft after a given Sand release.
//! - Quick one-off experiments before a proper typed API is available.
//!
//! # When to use
//!
//! Prefer typed APIs.  Reach for a raw type only when no typed alternative
//! exists **and** you can document why at the call site.
//!
//! ```rust,ignore
//! use sand_components::raw::RawJson;
//! use serde_json::json;
//!
//! // Good — explicitly opting in to raw JSON for a modded entity predicate
//! let cond = LootCondition::EntityProperties {
//!     entity: "this".into(),
//!     predicate: EntityPredicate::raw(RawJson::new(json!({ "type": "mymod:custom_entity" }))),
//! };
//! ```
//!
//! # Types
//!
//! | Type | Use for |
//! |---|---|
//! | [`RawJson`] | Raw `serde_json::Value` — arbitrary JSON objects |
//! | [`RawSnbt`] | Raw SNBT string — NBT that has no typed builder |
//! | [`RawCommand`] | Raw Minecraft command string |
//! | [`RawComponent`] | Raw item component `key=snbt` string |

use serde::{Serialize, Serializer};
use serde_json::Value;
use std::fmt;

// ── RawJson ───────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::RawJson",
    aliases = ["sand::prelude::RawJson"],
    module = "sand::component",
    summary = "A raw `serde_json::Value` used as an explicit escape hatch in datapack APIs.",
    context = "A raw `serde_json::Value` used as an explicit escape hatch in datapack APIs. Prefer typed predicate/component APIs.  Use this only for modded or future Minecraft features not yet covered by Sand's typed model.",
    minecraft = "Prefer typed predicate/component APIs.  Use this only for modded or future Minecraft features not yet covered by Sand's typed model.",
    use_when = ["Prefer typed predicate/component APIs.  Use this only for modded or future Minecraft features not yet covered by Sand's typed model."],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::RawJson;",
)]
/// A raw `serde_json::Value` used as an explicit escape hatch in datapack APIs.
///
/// Prefer typed predicate/component APIs.  Use this only for modded or
/// future Minecraft features not yet covered by Sand's typed model.
///
/// # Example
/// ```rust
/// use sand_components::raw::RawJson;
/// use serde_json::json;
///
/// let raw = RawJson::new(json!({ "type": "mymod:custom" }));
/// assert_eq!(raw.as_value()["type"], "mymod:custom");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RawJson(Value);

impl RawJson {
    /// Wrap an arbitrary JSON value as an explicit raw escape hatch.
    ///
    /// # Example
    /// ```rust
    /// use sand_components::RawJson;
    ///
    /// let raw = RawJson::new(serde_json::json!({"modded": true}));
    /// assert_eq!(raw.as_value()["modded"], true);
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawJson::new",
        aliases = ["sand::prelude::RawJson::new"],
        module = "sand::component",
        kind = "method",
        summary = "Wrap an arbitrary JSON value as an explicit raw escape hatch.",
        context = "Wrap an arbitrary JSON value as an explicit raw escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the v wrapped when creating an arbitrary JSON value as an explicit raw escape hatch."),
        returns = "A `RawJson` wrapping an arbitrary JSON value as an explicit raw escape hatch.",
        example = "use sand::component::RawJson;\nlet raw = RawJson::new(serde_json::json!({\"modded\": true}));\nassert_eq!(raw.as_value()[\"modded\"], true);",
    )]
    pub fn new(v: Value) -> Self {
        Self(v)
    }

    /// Access the inner `serde_json::Value`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawJson::as_value",
        aliases = ["sand::prelude::RawJson::as_value"],
        module = "sand::component",
        kind = "method",
        summary = "Access the inner `serde_json::Value`.",
        context = "Access the inner `serde_json::Value`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `& Value` value produced to acces the inner `serde_json::Value`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_json_value: &sand::component::RawJson)  {\n    let as_value = raw_json_value.as_value();\n}",
    )]
    pub fn as_value(&self) -> &Value {
        &self.0
    }

    /// Consume and return the inner `serde_json::Value`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawJson::into_value",
        aliases = ["sand::prelude::RawJson::into_value"],
        module = "sand::component",
        kind = "method",
        summary = "Consume and return the inner `serde_json::Value`.",
        context = "Consume and return the inner `serde_json::Value`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `Value` value produced to consume and return the inner `serde_json::Value`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_json_value: sand::component::RawJson)  {\n    let into_value = raw_json_value.into_value();\n}",
    )]
    pub fn into_value(self) -> Value {
        self.0
    }
}

impl From<Value> for RawJson {
    fn from(v: Value) -> Self {
        Self(v)
    }
}

impl From<RawJson> for Value {
    fn from(r: RawJson) -> Self {
        r.0
    }
}

impl Serialize for RawJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl fmt::Display for RawJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── RawSnbt ───────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::RawSnbt",
    aliases = ["sand::prelude::RawSnbt"],
    module = "sand::component",
    summary = "A raw SNBT (Stringified NBT) string used as an explicit escape hatch.",
    context = "A raw SNBT (Stringified NBT) string used as an explicit escape hatch. Use this when no typed NBT builder covers the compound or list you need. It is passed verbatim into generated commands — no validation is performed.",
    minecraft = "Use this when no typed NBT builder covers the compound or list you need. It is passed verbatim into generated commands — no validation is performed.",
    use_when = ["Use this when no typed NBT builder covers the compound or list you need. It is passed verbatim into generated commands — no validation is performed."],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::RawSnbt;",
)]
/// A raw SNBT (Stringified NBT) string used as an explicit escape hatch.
///
/// Use this when no typed NBT builder covers the compound or list you need.
/// It is passed verbatim into generated commands — no validation is performed.
///
/// # Example
/// ```rust
/// use sand_components::raw::RawSnbt;
///
/// let snbt = RawSnbt::new("{CustomModelData:42,display:{Name:\"Foo\"}}");
/// assert!(snbt.as_str().contains("CustomModelData"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawSnbt(String);

impl RawSnbt {
    /// Wrap a raw SNBT string as an explicit escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawSnbt::new",
        aliases = ["sand::prelude::RawSnbt::new"],
        module = "sand::component",
        kind = "method",
        summary = "Wrap a raw SNBT string as an explicit escape hatch.",
        context = "Wrap a raw SNBT string as an explicit escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(s = "`s` provides the s wrapped when creating a raw SNBT string as an explicit escape hatch."),
        returns = "A `RawSnbt` wrapping a raw SNBT string as an explicit escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(s: impl Into < String >)  {\n    let raw_snbt = sand::component::RawSnbt::new(s);\n}",
    )]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Access the inner SNBT string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawSnbt::as_str",
        aliases = ["sand::prelude::RawSnbt::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "Access the inner SNBT string.",
        context = "Access the inner SNBT string. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to acces the inner SNBT string.",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_snbt_value: &sand::component::RawSnbt)  {\n    let as_str = raw_snbt_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<RawSnbt> for String {
    fn from(r: RawSnbt) -> Self {
        r.0
    }
}

impl fmt::Display for RawSnbt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── RawCommand ────────────────────────────────────────────────────────────────

/// A raw Minecraft command string used as an explicit escape hatch.
///
/// Prefer generated typed command builders.  Use this for commands not yet
/// modelled in Sand's API or for one-off experiments. The text bypasses typed
/// construction, while export still enforces single-line file integrity and
/// conservative checks for malformed foundational syntax.
///
/// # Example
/// ```rust
/// use sand_components::raw::RawCommand;
///
/// let cmd = RawCommand::new("say Hello from raw command");
/// assert_eq!(cmd.as_str(), "say Hello from raw command");
/// ```
/// Compatibility path for the canonical command escape hatch.
///
/// The type is defined by `sand-commands`; this re-export keeps existing
/// `sand_components::raw::RawCommand` code source-compatible without a second
/// representation or conversion layer.
pub use sand_commands::RawCommand;

// ── RawComponent ──────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::RawComponent",
    aliases = ["sand::prelude::RawComponent"],
    module = "sand::component",
    summary = "A raw item component string (`key=snbt_value`) used as an explicit escape hatch.",
    context = "A raw item component string (`key=snbt_value`) used as an explicit escape hatch. Use this for item components not yet covered by Sand's typed `CustomItem` API. The string is appended verbatim to the generated item component list.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Use this for item components not yet covered by Sand's typed `CustomItem` API. The string is appended verbatim to the generated item component list."],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::RawComponent;",
)]
/// A raw item component string (`key=snbt_value`) used as an explicit escape hatch.
///
/// Use this for item components not yet covered by Sand's typed `CustomItem` API.
/// The string is appended verbatim to the generated item component list.
///
/// # Example
/// ```rust
/// use sand_components::raw::RawComponent;
///
/// let comp = RawComponent::new("bundle_contents", "{items:[]}");
/// assert_eq!(comp.key(), "bundle_contents");
/// assert_eq!(comp.value(), "{items:[]}");
/// assert_eq!(comp.to_string(), "bundle_contents={items:[]}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawComponent {
    key: String,
    value: String,
}

impl RawComponent {
    /// Create a raw item component from a `key` and its SNBT `value`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawComponent::new",
        aliases = ["sand::prelude::RawComponent::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a raw item component from a `key` and its SNBT `value`.",
        context = "Create a raw item component from a `key` and its SNBT `value`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "Create a raw item component from a `key` and its SNBT `value`.", value = "Create a raw item component from a `key` and its SNBT `value`."),
        returns = "A `RawComponent` representing a raw item component from a `key` and its SNBT `value`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(key: impl Into < String >, value: impl Into < String >)  {\n    let raw_component = sand::component::RawComponent::new(key, value);\n}",
    )]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// The component key (e.g. `"bundle_contents"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawComponent::key",
        aliases = ["sand::prelude::RawComponent::key"],
        module = "sand::component",
        kind = "method",
        summary = "The component key (e.g. `\"bundle_contents\"`).",
        context = "The component key (e.g. `\"bundle_contents\"`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the component key (e.g. `\"bundle_contents\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_component_value: &sand::component::RawComponent)  {\n    let key = raw_component_value.key();\n}",
    )]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The raw SNBT value string (e.g. `"{items:[]}"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::RawComponent::value",
        aliases = ["sand::prelude::RawComponent::value"],
        module = "sand::component",
        kind = "method",
        summary = "The raw SNBT value string (e.g. `\"{items:[]}\"`).",
        context = "The raw SNBT value string (e.g. `\"{items:[]}\"`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the raw SNBT value string (e.g. `\"{items:[]}\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(raw_component_value: &sand::component::RawComponent)  {\n    let value = raw_component_value.value();\n}",
    )]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for RawComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn raw_json_roundtrip() {
        let v = json!({"type": "mymod:custom", "level": 3});
        let raw = RawJson::new(v.clone());
        assert_eq!(raw.as_value(), &v);
        assert_eq!(raw.into_value(), v);
    }

    #[test]
    fn raw_json_from_value() {
        let raw: RawJson = json!({"k": 1}).into();
        assert_eq!(raw.as_value()["k"], 1);
    }

    #[test]
    fn raw_json_serialize() {
        let raw = RawJson::new(json!({"x": true}));
        let s = serde_json::to_string(&raw).unwrap();
        assert_eq!(s, r#"{"x":true}"#);
    }

    #[test]
    fn raw_snbt_roundtrip() {
        let snbt = RawSnbt::new("{CustomModelData:42}");
        assert_eq!(snbt.as_str(), "{CustomModelData:42}");
        assert_eq!(snbt.to_string(), "{CustomModelData:42}");
        let s: String = snbt.into();
        assert_eq!(s, "{CustomModelData:42}");
    }

    #[test]
    fn raw_command_roundtrip() {
        let cmd = RawCommand::new("say hello");
        assert_eq!(cmd.as_str(), "say hello");
        assert_eq!(cmd.to_string(), "say hello");
    }

    #[test]
    fn raw_component_display() {
        let c = RawComponent::new("bundle_contents", "{items:[]}");
        assert_eq!(c.key(), "bundle_contents");
        assert_eq!(c.value(), "{items:[]}");
        assert_eq!(c.to_string(), "bundle_contents={items:[]}");
    }
}
