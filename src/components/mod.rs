
use serde::{Deserialize, Serialize};

// Implement all of the components from:  https://minecraft.wiki/w/Data_component_format

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ComponentBundle {
    #[serde(rename = "minecraft:attribute_modifiers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_attribute_modifiers: Option<MinecraftAttributeModifiers>,
    #[serde(rename = "minecraft:banner_patterns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_banner_patterns: Option<Vec<MinecraftBannerPattern>>,
    #[serde(rename = "minecraft:base_color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_base_color: Option<String>,
    #[serde(rename = "minecraft:bees")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_bees: Option<Vec<MinecraftBee>>,
    #[serde(rename = "minecraft:block_entity_data")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_block_entity_data: Option<MinecraftBlockEntityData>,
    #[serde(rename = "minecraft:block_state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_block_state: Option<MinecraftBlockState>,
    #[serde(rename = "minecraft:bucket_entity_data")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_bucket_entity_data: Option<MinecraftBucketEntityData>,
    #[serde(rename = "minecraft:can_destroy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_can_destroy: Option<Vec<String>>,
    #[serde(rename = "minecraft:can_place_on")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_can_place_on: Option<Vec<String>>,
    #[serde(rename = "minecraft:custom_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_custom_name: Option<String>,
    #[serde(rename = "minecraft:damage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_damage: Option<i64>,
    #[serde(rename = "minecraft:durability")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_durability: Option<MinecraftDurability>,
    #[serde(rename = "minecraft:dye_color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_dye_color: Option<String>,
    #[serde(rename = "minecraft:enchantments")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_enchantments: Option<MinecraftEnchantments>,
    #[serde(rename = "minecraft:firework_rocket")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_firework_rocket: Option<MinecraftFireworkRocket>,
    #[serde(rename = "minecraft:firework_star")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_firework_star: Option<MinecraftFireworkStar>,
    #[serde(rename = "minecraft:food")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_food: Option<MinecraftFood>,
    #[serde(rename = "minecraft:friction")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_friction: Option<f64>,
    #[serde(rename = "minecraft:instrument")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_instrument: Option<String>,
    #[serde(rename = "minecraft:is_wearable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_is_wearable: Option<bool>,
    #[serde(rename = "minecraft:keep_on_death")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_keep_on_death: Option<bool>,
    #[serde(rename = "minecraft:lock")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_lock: Option<String>,
    #[serde(rename = "minecraft:loot_table")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_loot_table: Option<String>,
    #[serde(rename = "minecraft:map_color")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_map_color: Option<String>,
    #[serde(rename = "minecraft:max_stack_size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_max_stack_size: Option<i64>,
    #[serde(rename = "minecraft:pickup_delay")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_pickup_delay: Option<i64>,
    #[serde(rename = "minecraft:repair_cost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_repair_cost: Option<i64>,
    #[serde(rename = "minecraft:spawn_entity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_spawn_entity: Option<MinecraftSpawnEntity>,
    #[serde(rename = "minecraft:tinted")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_tinted: Option<bool>,
    #[serde(rename = "minecraft:unbreakable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_unbreakable: Option<bool>,
    #[serde(rename = "minecraft:written_book")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_written_book: Option<MinecraftWrittenBook>,
    #[serde(rename = "minecraft:charged_projectiles")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_charged_projectiles: Option<MinecraftChargedProjectiles>,
    #[serde(rename = "minecraft:potion_effects")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_potion_effects: Option<MinecraftPotionEffects>,
    #[serde(rename = "minecraft:suspicious_stew_effects")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_suspicious_stew_effects: Option<MinecraftSuspiciousStewEffects>,
    #[serde(rename = "minecraft:compass_lodestone")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_compass_lodestone: Option<MinecraftCompassLodestone>,
    #[serde(rename = "minecraft:bundle_contents")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_bundle_contents: Option<MinecraftBundleContents>,
}

impl Default for ComponentBundle {
    fn default() -> Self {
        Self {
            minecraft_attribute_modifiers: None,
            minecraft_banner_patterns: None,
            minecraft_base_color: None,
            minecraft_bees: None,
            minecraft_block_entity_data: None,
            minecraft_block_state: None,
            minecraft_bucket_entity_data: None,
            minecraft_can_destroy: None,
            minecraft_can_place_on: None,
            minecraft_custom_name: None,
            minecraft_damage: None,
            minecraft_durability: None,
            minecraft_dye_color: None,
            minecraft_enchantments: None,
            minecraft_firework_rocket: None,
            minecraft_firework_star: None,
            minecraft_food: None,
            minecraft_friction: None,
            minecraft_instrument: None,
            minecraft_is_wearable: None,
            minecraft_keep_on_death: None,
            minecraft_lock: None,
            minecraft_loot_table: None,
            minecraft_map_color: None,
            minecraft_max_stack_size: None,
            minecraft_pickup_delay: None,
            minecraft_repair_cost: None,
            minecraft_spawn_entity: None,
            minecraft_tinted: None,
            minecraft_unbreakable: None,
            minecraft_written_book: None,
            minecraft_potion_effects: None,
            minecraft_bundle_contents: None,
            minecraft_charged_projectiles: None,
            minecraft_compass_lodestone: None,
            minecraft_suspicious_stew_effects: None,
        }
    }
}

impl ToString for ComponentBundle {
    fn to_string(&self) -> String {
        let mut s = String::new();
        
        if let Some(minecraft_enchantments) = &self.minecraft_enchantments {
            s.push_str(minecraft_enchantments.to_string().as_str());
            s.push(',');
        }
        
        if let Some(minecraft_enchantments) = &self.minecraft_enchantments {
            s.push_str(minecraft_enchantments.to_string().as_str());
            s.push(',');
        }
        
        if let Some(minecraft_enchantments) = &self.minecraft_enchantments {
            s.push_str(minecraft_enchantments.to_string().as_str());
            s.push(',');
        }
        
        if let Some(minecraft_unbreakable) = &self.minecraft_unbreakable {
            s.push_str("unbreakable={}");
            s.push(',');
        }
        
        if let Some(minecraft_keep_on_death) = &self.minecraft_keep_on_death {
            s.push_str(format!("keep_on_death={}", minecraft_keep_on_death).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_lock) = &self.minecraft_lock {
            s.push_str(format!("lock=\"{}\"", minecraft_lock).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_loot_table) = &self.minecraft_loot_table {
            s.push_str(format!("loot_table=\"{}\"", minecraft_loot_table).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_map_color) = &self.minecraft_map_color {
            s.push_str(format!("map_color=\"{}\"", minecraft_map_color).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_max_stack_size) = &self.minecraft_max_stack_size {
            s.push_str(format!("max_stack_size={}", minecraft_max_stack_size).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_pickup_delay) = &self.minecraft_pickup_delay {
            s.push_str(format!("pickup_delay={}", minecraft_pickup_delay).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_repair_cost) = &self.minecraft_repair_cost {
            s.push_str(format!("repair_cost={}", minecraft_repair_cost).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_spawn_entity) = &self.minecraft_spawn_entity {
            s.push_str(minecraft_spawn_entity.to_string().as_str());
            s.push(',');
        }
        
        if let Some(minecraft_tinted) = &self.minecraft_tinted {
            s.push_str(format!("tinted={}", minecraft_tinted).as_str());
            s.push(',');
        }
        
        if let Some(minecraft_tinted) = &self.minecraft_tinted {
            s.push_str(format!("tinted={}", minecraft_tinted).as_str());
            s.push(',');
        }
        
        s
    }
}

impl ComponentBundle {
    pub fn to_minecraft(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftAttributeModifiers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<Vec<Modifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_in_tooltip: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Modifier {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftBannerPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftBee {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_data: Option<EntityData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ticks_in_hive: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticks_in_hive: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EntityData {
    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MinecraftBlockEntityData {
    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_data: Option<SpawnData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_range: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SpawnData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<Entity>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Entity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "Health")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftBlockState {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_block_state_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facing: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MinecraftBucketEntityData {
    #[serde(rename = "NoAI")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_ai: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_gravity: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glowing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invulnerable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_variant_tag: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftDurability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_durability: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_durability: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftEnchantments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enchantments: Option<Vec<MinecraftEnchantment>>,
}

impl ToString for MinecraftEnchantments {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str("levels:{");
        for enchantment in self.enchantments.as_ref().unwrap() {
            s.push_str(enchantment.to_string().as_str());
        }
        s.push('}');
        s
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftEnchantment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
}

impl ToString for MinecraftEnchantment {
    fn to_string(&self) -> String {
        format!("\"{}\":{},", self.id.as_ref().unwrap(), self.level.unwrap())
    }
}
        

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftFireworkRocket {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flight_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explosions: Option<Vec<MinecraftFireworkStar>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftFireworkStar {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flicker: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fade_colors: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftFood {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nutrition: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturation_modifier: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_meat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_always_eat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Effect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amplifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chance: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftSpawnEntity {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_spawn_entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<i64>,
}


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftBundleContents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Item {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftChargedProjectiles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectiles: Option<Vec<Projectile>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Projectile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<Vec<i64>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftCompassLodestone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<Pos>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Pos {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftPotionEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<MinecraftPotionEffectsEffect>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftPotionEffectsEffect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amplifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_particles: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftSuspiciousStewEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<MinecraftSuspiciousStewEffectsEffect>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftSuspiciousStewEffectsEffect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MinecraftWrittenBook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Vec<String>>,
}

