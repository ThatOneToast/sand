use std::collections::HashMap;
use std::fmt::Display;

use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

// ── Ingredient ───────────────────────────────────────────────────────────────

/// Represents a recipe ingredient that can be specified by item ID or item tag.
pub struct Ingredient {
    pub item: Option<String>,
    pub tag: Option<String>,
}

impl Ingredient {
    /// Creates an ingredient specified by a single item ID.
    pub fn item(id: impl Display) -> Self {
        Self {
            item: Some(id.to_string()),
            tag: None,
        }
    }

    /// Creates an ingredient specified by an item tag (matches all items in the tag).
    pub fn tag(id: impl Display) -> Self {
        Self {
            item: None,
            tag: Some(id.to_string()),
        }
    }
}

impl Serialize for Ingredient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.item.is_some() as usize + self.tag.is_some() as usize;
        let mut map = serializer.serialize_map(Some(count))?;
        if let Some(ref item) = self.item {
            map.serialize_entry("item", item)?;
        }
        if let Some(ref tag) = self.tag {
            map.serialize_entry("tag", tag)?;
        }
        map.end()
    }
}

// ── RecipeResult ─────────────────────────────────────────────────────────────

/// Represents the output of a recipe, including the item ID and quantity produced.
pub struct RecipeResult {
    pub id: String,
    pub count: u32,
}

impl RecipeResult {
    /// Creates a new recipe result with the given item ID and quantity.
    pub fn new(id: impl Display, count: u32) -> Self {
        Self {
            id: id.to_string(),
            count,
        }
    }
}

impl Serialize for RecipeResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("count", &self.count)?;
        map.end()
    }
}

// ── CookingType ──────────────────────────────────────────────────────────────

/// Specifies the type of cooking recipe (smelting, blasting, smoking, or campfire cooking).
pub enum CookingType {
    Smelting,
    Blasting,
    Smoking,
    CampfireCooking,
}

impl CookingType {
    /// Returns the Minecraft recipe type identifier string.
    pub fn type_str(&self) -> &'static str {
        match self {
            CookingType::Smelting => "minecraft:smelting",
            CookingType::Blasting => "minecraft:blasting",
            CookingType::Smoking => "minecraft:smoking",
            CookingType::CampfireCooking => "minecraft:campfire_cooking",
        }
    }
}

// ── ShapedRecipe ─────────────────────────────────────────────────────────────

/// Represents a shaped crafting recipe where items must be placed in specific grid positions.
pub struct ShapedRecipe {
    pub location: ResourceLocation,
    pub category: Option<String>,
    pub group: Option<String>,
    pub pattern: Vec<String>,
    pub key: HashMap<char, Ingredient>,
    pub result: RecipeResult,
    pub show_notification: bool,
}

impl ShapedRecipe {
    /// Creates a new shaped recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            category: None,
            group: None,
            pattern: Vec::new(),
            key: HashMap::new(),
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
            show_notification: true,
        }
    }

    /// Sets the crafting pattern rows (e.g., 3x3 grid layout).
    pub fn pattern(mut self, rows: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pattern = rows.into_iter().map(|r| r.into()).collect();
        self
    }

    /// Maps a character to an ingredient in the recipe pattern.
    pub fn key(mut self, ch: char, ingredient: Ingredient) -> Self {
        self.key.insert(ch, ingredient);
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe category for organization.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }

    /// Sets whether a notification is shown when the recipe is unlocked.
    pub fn show_notification(mut self, v: bool) -> Self {
        self.show_notification = v;
        self
    }
}

impl DatapackComponent for ShapedRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Value::String("minecraft:crafting_shaped".to_string()));

        if let Some(ref category) = self.category {
            map.insert("category".to_string(), Value::String(category.clone()));
        }
        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert(
            "pattern".to_string(),
            Value::Array(self.pattern.iter().map(|r| Value::String(r.clone())).collect()),
        );

        let key_map: serde_json::Map<String, Value> = self
            .key
            .iter()
            .map(|(ch, ing)| {
                (
                    ch.to_string(),
                    serde_json::to_value(ing).unwrap(),
                )
            })
            .collect();
        map.insert("key".to_string(), Value::Object(key_map));

        map.insert("result".to_string(), serde_json::to_value(&self.result).unwrap());
        map.insert("show_notification".to_string(), Value::Bool(self.show_notification));

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "recipe" }
}

// ── CookingRecipe ────────────────────────────────────────────────────────────

/// Represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).
pub struct CookingRecipe {
    pub location: ResourceLocation,
    pub recipe_type: CookingType,
    pub category: Option<String>,
    pub group: Option<String>,
    pub ingredient: Ingredient,
    pub result: RecipeResult,
    pub experience: f32,
    pub cooking_time: u32,
}

impl CookingRecipe {
    /// Creates a new cooking recipe with the given location and cooking type.
    pub fn new(location: ResourceLocation, recipe_type: CookingType) -> Self {
        Self {
            location,
            recipe_type,
            category: None,
            group: None,
            ingredient: Ingredient {
                item: None,
                tag: None,
            },
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
            experience: 0.0,
            cooking_time: 200,
        }
    }

    /// Sets the ingredient for this cooking recipe.
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient = ingredient;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the amount of experience awarded for completing this recipe.
    pub fn experience(mut self, experience: f32) -> Self {
        self.experience = experience;
        self
    }

    /// Sets the cooking time in ticks required for this recipe.
    pub fn cooking_time(mut self, cooking_time: u32) -> Self {
        self.cooking_time = cooking_time;
        self
    }

    /// Sets the recipe category for organization.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for CookingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Value::String(self.recipe_type.type_str().to_string()));

        if let Some(ref category) = self.category {
            map.insert("category".to_string(), Value::String(category.clone()));
        }
        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert("ingredient".to_string(), serde_json::to_value(&self.ingredient).unwrap());
        map.insert("result".to_string(), serde_json::to_value(&self.result).unwrap());
        map.insert("experience".to_string(), serde_json::to_value(self.experience).unwrap());
        map.insert("cookingtime".to_string(), serde_json::to_value(self.cooking_time).unwrap());

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "recipe" }
}

// ── ShapelessRecipe ──────────────────────────────────────────────────────────

/// Represents a shapeless crafting recipe where ingredient order and position don't matter.
pub struct ShapelessRecipe {
    pub location: ResourceLocation,
    pub category: Option<String>,
    pub group: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub result: RecipeResult,
}

impl ShapelessRecipe {
    /// Creates a new shapeless recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            category: None,
            group: None,
            ingredients: Vec::new(),
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
        }
    }

    /// Adds an ingredient to the recipe.
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe category for organization.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for ShapelessRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Value::String("minecraft:crafting_shapeless".to_string()));

        if let Some(ref category) = self.category {
            map.insert("category".to_string(), Value::String(category.clone()));
        }
        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        let ingredients: Vec<Value> = self
            .ingredients
            .iter()
            .map(|i| serde_json::to_value(i).unwrap())
            .collect();
        map.insert("ingredients".to_string(), Value::Array(ingredients));
        map.insert("result".to_string(), serde_json::to_value(&self.result).unwrap());

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "recipe" }
}

// ── StonecuttingRecipe ───────────────────────────────────────────────────────

/// Represents a stonecutter recipe for cutting stone blocks into other shapes.
pub struct StonecuttingRecipe {
    pub location: ResourceLocation,
    pub group: Option<String>,
    pub ingredient: Ingredient,
    pub result: RecipeResult,
    pub count: u32,
}

impl StonecuttingRecipe {
    /// Creates a new stonecutter recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            ingredient: Ingredient {
                item: None,
                tag: None,
            },
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
            count: 1,
        }
    }

    /// Sets the ingredient to be cut by the stonecutter.
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient = ingredient;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the quantity of the result produced.
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for StonecuttingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Value::String("minecraft:stonecutting".to_string()));

        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert("ingredient".to_string(), serde_json::to_value(&self.ingredient).unwrap());
        map.insert("result".to_string(), serde_json::to_value(&self.result).unwrap());
        map.insert("count".to_string(), serde_json::to_value(self.count).unwrap());

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "recipe" }
}

// ── SmithingTransformRecipe ──────────────────────────────────────────────────

/// Represents a smithing table recipe that transforms items using a template, base, and addition.
pub struct SmithingTransformRecipe {
    pub location: ResourceLocation,
    pub group: Option<String>,
    pub template: Ingredient,
    pub base: Ingredient,
    pub addition: Ingredient,
    pub result: RecipeResult,
}

impl SmithingTransformRecipe {
    /// Creates a new smithing transform recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            template: Ingredient { item: None, tag: None },
            base: Ingredient { item: None, tag: None },
            addition: Ingredient { item: None, tag: None },
            result: RecipeResult { id: String::new(), count: 1 },
        }
    }

    /// Sets the template ingredient (e.g., netherite upgrade template).
    pub fn template(mut self, template: Ingredient) -> Self {
        self.template = template;
        self
    }

    /// Sets the base ingredient to be upgraded.
    pub fn base(mut self, base: Ingredient) -> Self {
        self.base = base;
        self
    }

    /// Sets the addition ingredient (e.g., netherite ingot).
    pub fn addition(mut self, addition: Ingredient) -> Self {
        self.addition = addition;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for SmithingTransformRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Value::String("minecraft:smithing_transform".to_string()));

        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert("template".to_string(), serde_json::to_value(&self.template).unwrap());
        map.insert("base".to_string(), serde_json::to_value(&self.base).unwrap());
        map.insert("addition".to_string(), serde_json::to_value(&self.addition).unwrap());
        map.insert("result".to_string(), serde_json::to_value(&self.result).unwrap());

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "recipe" }
}

// ── SmithingTrimRecipe ───────────────────────────────────────────────────────

/// Represents a smithing table recipe that applies decorative trim to armor.
pub struct SmithingTrimRecipe {
    pub location: ResourceLocation,
    pub group: Option<String>,
    pub template: Ingredient,
    pub base: Ingredient,
    pub addition: Ingredient,
}

impl SmithingTrimRecipe {
    /// Creates a new smithing trim recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            template: Ingredient { item: None, tag: None },
            base: Ingredient { item: None, tag: None },
            addition: Ingredient { item: None, tag: None },
        }
    }

    /// Sets the trim template ingredient.
    pub fn template(mut self, template: Ingredient) -> Self {
        self.template = template;
        self
    }

    /// Sets the armor piece to be trimmed.
    pub fn base(mut self, base: Ingredient) -> Self {
        self.base = base;
        self
    }

    /// Sets the trim material ingredient (e.g., amethyst shard).
    pub fn addition(mut self, addition: Ingredient) -> Self {
        self.addition = addition;
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for SmithingTrimRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Value::String("minecraft:smithing_trim".to_string()));

        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert("template".to_string(), serde_json::to_value(&self.template).unwrap());
        map.insert("base".to_string(), serde_json::to_value(&self.base).unwrap());
        map.insert("addition".to_string(), serde_json::to_value(&self.addition).unwrap());

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "recipe" }
}
