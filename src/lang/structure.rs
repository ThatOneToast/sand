use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SandConfig {
    pub name: String,
    pub version: String,
}

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml;

#[derive(Debug, Clone)]
pub struct Object {
    pub constructor: SuperConstructor,
    pub name: String,
    pub fields: Vec<Field>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Number(i64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone)]
pub enum SuperConstructor {
    ItemStack,
    Entities,
    LootTables,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Int,
    Float,
    Boolean,
}
