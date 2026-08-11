use std::path::PathBuf;

fn main() {
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"));
    let rust = output.join("registries.rs");
    let provider = output.join("registries.api.json");

    std::fs::write(
        &rust,
        r#"
pub enum Item {
    Stone,
    Uncontracted,
}
impl Item {
    pub fn resource_location(&self) -> &'static str {
        match self {
            Self::Stone => "minecraft:stone",
            Self::Uncontracted => "minecraft:uncontracted",
        }
    }

    pub fn uncontracted_method(&self) {}
}
pub enum UncontractedRegistry {}
"#,
    )
    .unwrap();
    std::fs::write(
        &provider,
        r#"{
  "schema_version": 1,
  "provider": "generated_registries",
  "minecraft_version": "fixture",
  "entries": [
    {
      "definition_identity": "fixture_core::generated::Item",
      "definition_kind": "enum",
      "contract": {
        "canonical_path": "sand::vanilla::Item",
        "canonical_module": "sand::vanilla",
        "kind": "enum",
        "signature": "pub enum Item",
        "summary": "Identifies entries in Minecraft's item registry.",
        "context": "Generated from the selected Minecraft registry report.",
        "minecraft": "Each variant renders an exact vanilla item identifier.",
        "use_when": ["Selecting a vanilla item"],
        "avoid_when": ["Selecting custom content"],
        "example": "let item = sand::vanilla::Item::Stone;",
        "availability": ["minecraft = fixture"]
      }
    },
    {
      "definition_identity": "fixture_core::generated::Item::Stone",
      "definition_kind": "variant",
      "parent_identity": "fixture_core::generated::Item",
      "member_name": "Stone",
      "contract": {
        "canonical_path": "sand::vanilla::Item::Stone",
        "canonical_module": "sand::vanilla",
        "kind": "variant",
        "signature": "Stone",
        "summary": "Selects the vanilla registry entry `minecraft:stone`.",
        "context": "Generated from Minecraft's item registry.",
        "minecraft": "Serializes as `minecraft:stone`.",
        "use_when": ["Referring to vanilla stone"],
        "avoid_when": ["Referring to custom content"],
        "example": "let item = sand::vanilla::Item::Stone;",
        "availability": ["minecraft = fixture"]
      }
    },
    {
      "definition_identity": "fixture_core::generated::Item::resource_location",
      "definition_kind": "method",
      "parent_identity": "fixture_core::generated::Item",
      "member_name": "resource_location",
      "contract": {
        "canonical_path": "sand::vanilla::Item::resource_location",
        "canonical_module": "sand::vanilla",
        "kind": "method",
        "signature": "pub fn resource_location(&self) -> &'static str",
        "summary": "Returns the exact Minecraft item identifier.",
        "context": "Generated variants retain their registry key.",
        "minecraft": "Returns the identifier serialized into Minecraft data.",
        "use_when": ["Calling an untyped integration"],
        "avoid_when": ["A typed Sand API accepts Item"],
        "returns": "The static namespaced item identifier.",
        "example": "let id = sand::vanilla::Item::Stone.resource_location();",
        "availability": ["minecraft = fixture"]
      }
    }
  ]
}
"#,
    )
    .unwrap();

    let catalog = sand_build::read_api_provider(&provider).unwrap();
    sand_build::validate_api_provider_source(
        &catalog,
        &rust,
        "fixture_core::generated",
    )
    .expect("generated registry contracts must exactly cover emitted Rust");
}
