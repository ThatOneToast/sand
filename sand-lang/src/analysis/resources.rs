//! Lowering for data declarations: custom items, recipes, and registry tags.

use super::*;
use crate::ast::{DataField, DataItem, DataKind, DataValue};
use crate::builtins;
use std::collections::BTreeSet;

impl Cx {
    pub(super) fn lower_data(
        &mut self,
        module: ModuleId,
        file: FileId,
        def: DefId,
        data: &DataItem,
        item: &Item,
        out: &mut pc::Module,
    ) {
        let DefInfo::Data { id, tag_type, .. } = self.defs[def.0].info.clone() else {
            return;
        };
        if let Some(attr) = item.attrs.first() {
            self.error(
                "attribute",
                file,
                attr.span,
                format!("`{}` declarations take no attributes", data.kind.keyword()),
            );
        }
        let Ok(location) = id.parse::<sand::ResourceLocation>() else {
            self.error(
                "id",
                file,
                data.name.span,
                format!("`{id}` is not a valid resource identity"),
            );
            return;
        };
        let fields = self.check_fields(file, data);
        let module_id = self.module_identity(module);
        let get = |name: &str| fields.get(name).copied();
        match data.kind {
            DataKind::Item => {
                let base =
                    get("base").and_then(|f| self.data_item_ref(module, file, &f.value, false));
                let name = get("name").map(|f| self.data_text(module, file, &f.value));
                let lore = get("lore").map(|f| match &f.value {
                    DataValue::Array(lines, _) => lines
                        .iter()
                        .map(|line| self.data_text(module, file, line))
                        .collect::<Option<Vec<_>>>(),
                    other => {
                        self.error(
                            "type",
                            file,
                            other.span(),
                            "expected a list of lines: `[\"...\"]`",
                        );
                        None
                    }
                });
                let rarity = get("rarity").map(|f| {
                    let expr = self.data_expr(file, &f.value, "a rarity such as `Rarity::Epic`")?;
                    self.vanilla_value(module, file, expr, "Rarity")
                        .map(|value| match value.as_str() {
                            "common" => pc::Rarity::Common,
                            "uncommon" => pc::Rarity::Uncommon,
                            "rare" => pc::Rarity::Rare,
                            _ => pc::Rarity::Epic,
                        })
                });
                let glint = get("glint").map(|f| self.data_bool(file, &f.value));
                let unbreakable = get("unbreakable").map(|f| self.data_bool(file, &f.value));
                let max_stack =
                    get("max_stack").map(|f| self.data_int(module, file, &f.value, 1, 99));
                let model = get("custom_model_data").map(|f| {
                    self.data_int(module, file, &f.value, i32::MIN as i64, i32::MAX as i64)
                });
                let enchantments =
                    get("enchantments").map(|f| self.data_enchantments(module, file, &f.value));
                let (
                    Some(base),
                    Some(name),
                    Some(lore),
                    Some(rarity),
                    Some(glint),
                    Some(unbreakable),
                    Some(max_stack),
                    Some(model),
                    Some(enchantments),
                ) = (
                    base,
                    name.map(|n| n.map(Some)).unwrap_or(Some(None)),
                    lore.map(|l| l.map(Some)).unwrap_or(Some(None)),
                    rarity.map(|r| r.map(Some)).unwrap_or(Some(None)),
                    glint.map(|g| g.map(Some)).unwrap_or(Some(None)),
                    unbreakable.unwrap_or(Some(false)),
                    max_stack.map(|m| m.map(Some)).unwrap_or(Some(None)),
                    model.map(|m| m.map(Some)).unwrap_or(Some(None)),
                    enchantments.unwrap_or_else(|| Some(BTreeMap::new())),
                )
                else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/items/{}", out.items.len()),
                    file,
                    data.name.span,
                ));
                out.items.push(pc::CustomItem {
                    id: location,
                    base,
                    name: name.unwrap_or_default(),
                    lore: lore.unwrap_or_default(),
                    glint,
                    unbreakable,
                    max_stack: max_stack.map(|m| m as u32),
                    rarity,
                    enchantments,
                    custom_model_data: model.map(|m| m as i32),
                });
            }
            DataKind::Recipe => {
                let chosen: Vec<&str> = builtins::RECIPE_TYPES
                    .iter()
                    .copied()
                    .filter(|name| fields.contains_key(name))
                    .collect();
                let kind = match chosen.as_slice() {
                    [kind] => *kind,
                    [] => {
                        self.diagnostics.push(
                            Diagnostic::error("recipe", file, data.name.span, "a recipe needs a type")
                                .with_help("add one of `shaped`, `shapeless`, `smelting`, `blasting`, `smoking`, `campfire`, `stonecutting`, `smithing`, or `smithing_trim`"),
                        );
                        return;
                    }
                    [_, second, ..] => {
                        let span = fields[second].key.span;
                        self.error("recipe", file, span, "a recipe has exactly one type");
                        return;
                    }
                };
                for field in [
                    "key",
                    "base",
                    "addition",
                    "experience",
                    "time",
                    "trim_pattern",
                    "result",
                    "count",
                ] {
                    if let Some(f) = get(field)
                        && !builtins::recipe_accepts_field(Some(kind), field)
                    {
                        self.error(
                            "recipe",
                            file,
                            f.key.span,
                            format!("`{field}` does not apply to a `{kind}` recipe"),
                        );
                    }
                }
                let register_recipe = |this: &mut Self, recipe| {
                    this.pointers.push((
                        module_id,
                        format!("/recipes/{}", out.recipes.len()),
                        file,
                        data.name.span,
                    ));
                    out.recipes.push(recipe);
                };
                if kind == "smithing_trim" {
                    let Some((template, base, addition)) = self.data_smithing_ingredients(
                        module,
                        file,
                        fields[kind],
                        get("base"),
                        get("addition"),
                    ) else {
                        return;
                    };
                    let Some(pattern) = get("trim_pattern") else {
                        self.error(
                            "recipe",
                            file,
                            data.name.span,
                            "a smithing trim recipe needs `trim_pattern`",
                        );
                        return;
                    };
                    let parsed = match &pattern.value {
                        DataValue::Expr(Expr::Str { value, .. }) => value.parse().ok(),
                        _ => None,
                    };
                    let Some(pattern_id) = parsed else {
                        self.error(
                            "recipe",
                            file,
                            pattern.value.span(),
                            "expected a quoted trim pattern id such as `\"minecraft:sentry\"`",
                        );
                        return;
                    };
                    let recipe = pc::Recipe::SmithingTrim {
                        id: location,
                        template,
                        base,
                        addition,
                        pattern: pattern_id,
                    };
                    register_recipe(self, recipe);
                    return;
                }
                let Some(result_field) = get("result") else {
                    self.error("field", file, data.name.span, "`recipe` needs `result`");
                    return;
                };
                let result_item = self.data_item_ref(module, file, &result_field.value, true);
                let count = get("count")
                    .map(|f| self.data_int(module, file, &f.value, 1, 99))
                    .unwrap_or(Some(1));
                let (Some(item), Some(count)) = (result_item, count) else {
                    return;
                };
                let result = pc::RecipeOutput {
                    item,
                    count: count as u32,
                };
                let type_field = fields[kind];
                let recipe = match kind {
                    "shaped" => {
                        let Some(pattern) = self.data_pattern(file, &type_field.value) else {
                            return;
                        };
                        let Some(key_field) = get("key") else {
                            self.error(
                                "recipe",
                                file,
                                type_field.key.span,
                                "a shaped recipe needs a `key`",
                            );
                            return;
                        };
                        let DataValue::Object(entries, _) = &key_field.value else {
                            self.error(
                                "type",
                                file,
                                key_field.value.span(),
                                "expected `{ A: Item::..., ... }`",
                            );
                            return;
                        };
                        let mut key = BTreeMap::new();
                        let used: std::collections::BTreeSet<char> = pattern
                            .iter()
                            .flat_map(|row| row.chars())
                            .filter(|c| *c != ' ')
                            .collect();
                        for entry in entries {
                            let mut chars = entry.key.name.chars();
                            match (chars.next(), chars.next()) {
                                (Some(c), None) if used.contains(&c) => {}
                                (Some(_), None) => {
                                    self.error(
                                        "recipe",
                                        file,
                                        entry.key.span,
                                        format!("`{}` is not used in the pattern", entry.key.name),
                                    );
                                    continue;
                                }
                                _ => {
                                    self.error(
                                        "recipe",
                                        file,
                                        entry.key.span,
                                        "pattern keys are single characters",
                                    );
                                    continue;
                                }
                            }
                            if let Some(ingredient) =
                                self.data_ingredient(module, file, &entry.value)
                            {
                                key.insert(entry.key.name.clone(), ingredient);
                            }
                        }
                        for c in &used {
                            if !entries.iter().any(|e| e.key.name == c.to_string()) {
                                self.error(
                                    "recipe",
                                    file,
                                    type_field.value.span(),
                                    format!("pattern character `{c}` has no `key` entry"),
                                );
                                return;
                            }
                        }
                        pc::Recipe::Shaped {
                            id: location,
                            pattern,
                            key,
                            result,
                        }
                    }
                    "shapeless" => {
                        let DataValue::Array(values, span) = &type_field.value else {
                            self.error(
                                "type",
                                file,
                                type_field.value.span(),
                                "expected a list of ingredients",
                            );
                            return;
                        };
                        if values.is_empty() || values.len() > 9 {
                            self.error(
                                "recipe",
                                file,
                                *span,
                                "a shapeless recipe has one to nine ingredients",
                            );
                            return;
                        }
                        let ingredients: Option<Vec<String>> = values
                            .iter()
                            .map(|v| self.data_ingredient(module, file, v))
                            .collect();
                        pc::Recipe::Shapeless {
                            id: location,
                            ingredients: ingredients.unwrap_or_default(),
                            result,
                        }
                    }
                    "smithing" => {
                        let Some((template, base, addition)) = self.data_smithing_ingredients(
                            module,
                            file,
                            fields[kind],
                            get("base"),
                            get("addition"),
                        ) else {
                            return;
                        };
                        pc::Recipe::Smithing {
                            id: location,
                            template,
                            base,
                            addition,
                            result,
                        }
                    }
                    "stonecutting" => {
                        let Some(ingredient) =
                            self.data_ingredient(module, file, &type_field.value)
                        else {
                            return;
                        };
                        pc::Recipe::Stonecutting {
                            id: location,
                            ingredient,
                            result,
                        }
                    }
                    cooking => {
                        let Some(ingredient) =
                            self.data_ingredient(module, file, &type_field.value)
                        else {
                            return;
                        };
                        let experience = get("experience")
                            .map(|f| self.data_number(module, file, &f.value))
                            .unwrap_or(Some(0.0));
                        let time = get("time")
                            .map(|f| {
                                self.data_int(module, file, &f.value, 1, i32::MAX as i64)
                                    .map(Some)
                            })
                            .unwrap_or(Some(None));
                        let (Some(experience), Some(time)) = (experience, time) else {
                            return;
                        };
                        pc::Recipe::Cooking {
                            id: location,
                            method: match cooking {
                                "smelting" => pc::CookingMethod::Smelting,
                                "blasting" => pc::CookingMethod::Blasting,
                                "smoking" => pc::CookingMethod::Smoking,
                                _ => pc::CookingMethod::Campfire,
                            },
                            ingredient,
                            result,
                            experience: experience as f32,
                            time: time.map(|t| t as u32),
                        }
                    }
                };
                register_recipe(self, recipe);
            }
            DataKind::BossBar => {
                let Some(title) = get("title").and_then(|f| self.data_text(module, file, &f.value))
                else {
                    return;
                };
                let color = match get("color") {
                    Some(f) => {
                        let Some(expr) = self.data_expr(file, &f.value, "a `BossBarColor`") else {
                            return;
                        };
                        let Some(value) = self.vanilla_value(module, file, expr, "BossBarColor")
                        else {
                            return;
                        };
                        match value.as_str() {
                            "blue" => pc::BossBarColor::Blue,
                            "green" => pc::BossBarColor::Green,
                            "pink" => pc::BossBarColor::Pink,
                            "purple" => pc::BossBarColor::Purple,
                            "red" => pc::BossBarColor::Red,
                            "yellow" => pc::BossBarColor::Yellow,
                            _ => pc::BossBarColor::White,
                        }
                    }
                    None => pc::BossBarColor::White,
                };
                let style = match get("style") {
                    Some(f) => {
                        let Some(expr) = self.data_expr(file, &f.value, "a `BossBarStyle`") else {
                            return;
                        };
                        let Some(value) = self.vanilla_value(module, file, expr, "BossBarStyle")
                        else {
                            return;
                        };
                        match value.as_str() {
                            "notched_6" => pc::BossBarStyle::Notched6,
                            "notched_10" => pc::BossBarStyle::Notched10,
                            "notched_12" => pc::BossBarStyle::Notched12,
                            "notched_20" => pc::BossBarStyle::Notched20,
                            _ => pc::BossBarStyle::Progress,
                        }
                    }
                    None => pc::BossBarStyle::Progress,
                };
                let max = get("max")
                    .map(|f| self.data_int(module, file, &f.value, 1, i32::MAX as i64))
                    .unwrap_or(Some(100));
                let Some(max) = max else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/boss_bars/{}", out.boss_bars.len()),
                    file,
                    data.name.span,
                ));
                out.boss_bars.push(pc::BossBar {
                    id: location,
                    title,
                    color,
                    style,
                    max: max as u32,
                });
            }
            DataKind::Predicate => {
                let option = |cx: &mut Cx, field, registry| match get(field) {
                    Some(field) => {
                        let expr = cx.data_expr(file, &field.value, "a typed predicate value")?;
                        cx.vanilla_value(module, file, expr, registry).map(Some)
                    }
                    None => Some(None),
                };
                let (Some(block), Some(biome), Some(dimension)) = (
                    option(self, "block", "Block"),
                    option(self, "biome", "Biome"),
                    option(self, "dimension", "Dimension"),
                ) else {
                    return;
                };
                let weather = match get("weather") {
                    Some(field) => {
                        let Some(weather) = self.data_weather(module, file, &field.value) else {
                            return;
                        };
                        Some(weather)
                    }
                    None => None,
                };
                let chance = match get("chance") {
                    Some(field) => {
                        let Some(chance) =
                            self.data_probability(module, file, &field.value, "predicate")
                        else {
                            return;
                        };
                        Some(chance)
                    }
                    None => None,
                };
                if block.is_none()
                    && biome.is_none()
                    && dimension.is_none()
                    && weather.is_none()
                    && chance.is_none()
                {
                    self.error(
                        "predicate",
                        file,
                        data.name.span,
                        "a world predicate needs at least one condition",
                    );
                    return;
                }
                self.pointers.push((
                    module_id,
                    format!("/predicates/{}", out.predicates.len()),
                    file,
                    data.name.span,
                ));
                out.predicates.push(pc::WorldPredicate {
                    id: location,
                    biome,
                    block,
                    dimension: dimension.and_then(|id| id.parse().ok()),
                    weather,
                    chance,
                });
            }
            DataKind::DamageType => {
                let Some(message) = get("message_id") else {
                    return;
                };
                let DataValue::Expr(Expr::Str {
                    value: message_id,
                    span,
                    ..
                }) = &message.value
                else {
                    self.error(
                        "type",
                        file,
                        message.value.span(),
                        "expected a literal death message id",
                    );
                    return;
                };
                if message_id.trim().is_empty() || message_id.chars().any(char::is_control) {
                    self.error(
                        "value",
                        file,
                        *span,
                        "a death message id must contain text without control characters",
                    );
                    return;
                }
                let exhaustion = match get("exhaustion") {
                    Some(field) => match self.data_number(module, file, &field.value) {
                        Some(value) if value >= 0.0 && (value as f32).is_finite() => value as f32,
                        Some(_) => {
                            self.error(
                                "range",
                                file,
                                field.value.span(),
                                "exhaustion must be finite and non-negative",
                            );
                            return;
                        }
                        None => return,
                    },
                    None => 0.0,
                };
                let option = |cx: &mut Cx, field, registry| {
                    let Some(field) = get(field) else {
                        return Some(serde_json::Value::Null);
                    };
                    let value = cx.data_expr(file, &field.value, "a typed damage option")?;
                    if !matches!(value, Expr::Path(_)) {
                        cx.error(
                            "type",
                            file,
                            value.span(),
                            format!("expected a `{registry}::...` value"),
                        );
                        return None;
                    }
                    cx.vanilla_value(module, file, value, registry)
                        .map(serde_json::Value::String)
                };
                let (Some(scaling), Some(effects), Some(death_message_type)) = (
                    option(self, "scaling", "DamageScaling"),
                    option(self, "effects", "DamageEffects"),
                    option(self, "death_message_type", "DeathMessageType"),
                ) else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/damage_types/{}", out.damage_types.len()),
                    file,
                    data.name.span,
                ));
                let mut damage_type = sand::component::DamageType::new(location)
                    .message_id(message_id)
                    .exhaustion(exhaustion);
                if let Some(scaling) =
                    serde_json::from_value(scaling).expect("damage scaling option")
                {
                    damage_type = damage_type.scaling(scaling);
                }
                if let Some(effects) =
                    serde_json::from_value(effects).expect("damage effect option")
                {
                    damage_type = damage_type.effects(effects);
                }
                if let Some(death_message_type) =
                    serde_json::from_value(death_message_type).expect("death message option")
                {
                    damage_type = damage_type.death_message_type(death_message_type);
                }
                out.damage_types.push(damage_type);
            }
            DataKind::Loot => {
                let Some(pools) = self.data_loot_pools(module, file, data, &fields) else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/loot_tables/{}", out.loot_tables.len()),
                    file,
                    data.name.span,
                ));
                out.loot_tables.push(pc::LootTable {
                    id: location,
                    pools,
                });
            }
            DataKind::Tag => {
                // A missing or unknown type was reported at the declaration.
                let Some(registry) = tag_type else {
                    return;
                };
                let kind = match registry {
                    "Item" => pc::TagKind::Item,
                    "Block" => pc::TagKind::Block,
                    _ => pc::TagKind::EntityType,
                };
                let Some(values_field) = get("values") else {
                    return;
                };
                let DataValue::Array(values, _) = &values_field.value else {
                    self.error(
                        "type",
                        file,
                        values_field.value.span(),
                        "expected a list of values",
                    );
                    return;
                };
                let values: Option<Vec<String>> = values
                    .iter()
                    .map(|v| self.data_tag_value(module, file, v, registry))
                    .collect();
                let replace = get("replace")
                    .map(|f| self.data_bool(file, &f.value))
                    .unwrap_or(Some(false));
                let (Some(values), Some(replace)) = (values, replace) else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/resource_tags/{}", out.resource_tags.len()),
                    file,
                    data.name.span,
                ));
                out.resource_tags.push(pc::ResourceTag {
                    kind,
                    id: location,
                    values,
                    replace,
                });
            }
        }
    }

    pub(super) fn predicate_id(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
    ) -> Option<sand::ResourceLocation> {
        if let Expr::Str { value, span, .. } = expr {
            if value.contains(':')
                && let Ok(id) = value.parse()
            {
                return Some(id);
            }
            self.error(
                "id",
                file,
                *span,
                "expected a namespaced predicate identity",
            );
            return None;
        }
        let Expr::Path(path) = expr else {
            self.error("type", file, expr.span(), "expected a declared predicate");
            return None;
        };
        let def = self.expect_path(module, path, &[DefKind::Predicate], "a predicate")?;
        let DefInfo::Data { id, .. } = &self.defs[def.0].info else {
            return None;
        };
        id.parse().ok()
    }

    fn data_weather(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::Weather> {
        let expr = self.data_expr(file, value, "a Weather value")?;
        let value = self.vanilla_value(module, file, expr, "Weather")?;
        let Ok(weather) = serde_json::from_value(serde_json::Value::String(value)) else {
            self.error("type", file, expr.span(), "expected a `Weather::...` value");
            return None;
        };
        Some(weather)
    }

    fn data_probability(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        owner: &str,
    ) -> Option<f64> {
        let chance = self.data_number(module, file, value)?;
        if !(0.0..=1.0).contains(&chance) {
            self.error(
                "range",
                file,
                value.span(),
                format!("{owner} chance must be between zero and one"),
            );
            return None;
        }
        Some(chance)
    }

    fn data_loot_pools(
        &mut self,
        module: ModuleId,
        file: FileId,
        data: &DataItem,
        fields: &BTreeMap<&str, &DataField>,
    ) -> Option<Vec<pc::LootPool>> {
        if let Some(pools) = fields.get("pools") {
            if fields.len() > 1 {
                self.error(
                    "loot",
                    file,
                    pools.span,
                    "use either pools or top-level items and rolls",
                );
                return None;
            }
            let DataValue::Array(values, span) = &pools.value else {
                self.error(
                    "type",
                    file,
                    pools.value.span(),
                    "expected a list of loot pools",
                );
                return None;
            };
            if values.is_empty() {
                self.error("loot", file, *span, "a loot table needs at least one pool");
                return None;
            }
            return values
                .iter()
                .map(|value| {
                    let DataValue::Object(fields, span) = value else {
                        self.error("type", file, value.span(), "expected a loot pool object");
                        return None;
                    };
                    self.data_loot_pool(module, file, fields, *span)
                })
                .collect();
        }
        self.data_loot_pool(module, file, &data.fields, data.name.span)
            .map(|pool| vec![pool])
    }

    fn data_loot_pool(
        &mut self,
        module: ModuleId,
        file: FileId,
        fields: &[DataField],
        span: Span,
    ) -> Option<pc::LootPool> {
        let mut pool = pc::LootPool {
            items: Vec::new(),
            rolls: 1,
            chance: None,
            weather: None,
            predicate: None,
        };
        let mut seen = BTreeSet::new();
        for field in fields {
            if !seen.insert(&field.key.name) {
                self.error(
                    "loot",
                    file,
                    field.key.span,
                    format!("duplicate loot pool field `{}`", field.key.name),
                );
                return None;
            }
            match field.key.name.as_str() {
                "items" => {
                    let DataValue::Array(values, _) = &field.value else {
                        self.error("type", file, field.value.span(), "expected a list of items");
                        return None;
                    };
                    pool.items = values
                        .iter()
                        .map(|item| self.data_loot_item(module, file, item))
                        .collect::<Option<_>>()?;
                }
                "rolls" => pool.rolls = self.data_int(module, file, &field.value, 1, 1000)? as u32,
                "chance" => {
                    pool.chance =
                        Some(self.data_probability(module, file, &field.value, "loot pool")?)
                }
                "weather" => pool.weather = Some(self.data_weather(module, file, &field.value)?),
                "when" => {
                    let expr = self.data_expr(file, &field.value, "a native predicate")?;
                    pool.predicate = Some(self.predicate_id(module, file, expr)?);
                }
                _ => {
                    self.error(
                        "field",
                        file,
                        field.key.span,
                        format!("unknown loot pool field `{}`", field.key.name),
                    );
                    return None;
                }
            }
        }
        if pool.items.is_empty() {
            self.error("loot", file, span, "a loot table needs at least one item");
            return None;
        }
        Some(pool)
    }

    /// A vanilla item, or `{ item: ..., weight: ..., count: ... }`.
    fn data_loot_item(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::LootItem> {
        let (item, mut weight, mut count) = match value {
            DataValue::Object(entries, span) => {
                let mut item = None;
                let mut weight = Some(1);
                let mut count = Some(1);
                for entry in entries {
                    match entry.key.name.as_str() {
                        "item" => {
                            item = Some(self.data_item_ref(module, file, &entry.value, false)?);
                        }
                        "weight" => {
                            weight = self.data_int(module, file, &entry.value, 1, i32::MAX as i64)
                        }
                        "count" => count = self.data_int(module, file, &entry.value, 1, 99),
                        other => {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "field",
                                    file,
                                    entry.key.span,
                                    format!("a loot item has no field `{other}`"),
                                )
                                .with_help("fields: item, weight, count"),
                            );
                            return None;
                        }
                    }
                }
                if item.is_none() {
                    self.error("field", file, *span, "a loot item needs `item`");
                    return None;
                }
                (item?, weight, count)
            }
            other => (
                self.data_item_ref(module, file, other, false)?,
                Some(1),
                Some(1),
            ),
        };
        Some(pc::LootItem {
            item,
            weight: weight.take()? as u32,
            count: count.take()? as u32,
        })
    }

    /// Index fields by name, reporting unknown, duplicate, and missing ones.
    fn check_fields<'a>(
        &mut self,
        file: FileId,
        data: &'a DataItem,
    ) -> BTreeMap<&'a str, &'a DataField> {
        let keyword = data.kind.keyword();
        let mut fields = BTreeMap::new();
        for field in &data.fields {
            let name = field.key.name.as_str();
            if builtins::data_fields(keyword).all(|doc| doc.name != name) {
                let known: Vec<&str> = builtins::data_fields(keyword).map(|doc| doc.name).collect();
                let best = known
                    .iter()
                    .min_by_key(|candidate| crate::vanilla::distance(name, candidate))
                    .filter(|candidate| crate::vanilla::distance(name, candidate) <= 2);
                let mut diagnostic = Diagnostic::error(
                    "field",
                    file,
                    field.key.span,
                    format!("`{keyword}` has no field `{name}`"),
                );
                diagnostic = match best {
                    Some(best) => diagnostic.with_help(format!("did you mean `{best}`?")),
                    None => diagnostic.with_help(format!("fields: {}", known.join(", "))),
                };
                self.diagnostics.push(diagnostic);
                continue;
            }
            if fields.insert(name, field).is_some() {
                self.error(
                    "field",
                    file,
                    field.key.span,
                    format!("field `{name}` is given more than once"),
                );
            }
        }
        for doc in builtins::data_fields(keyword).filter(|doc| doc.required) {
            if !fields.contains_key(doc.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "field",
                        file,
                        data.name.span,
                        format!("`{keyword}` needs `{}`", doc.name),
                    )
                    .with_help(doc.doc),
                );
            }
        }
        fields
    }

    fn data_expr<'a>(
        &mut self,
        file: FileId,
        value: &'a DataValue,
        what: &str,
    ) -> Option<&'a Expr> {
        match value {
            DataValue::Expr(expr) => Some(expr),
            other => {
                self.error("type", file, other.span(), format!("expected {what}"));
                None
            }
        }
    }

    /// A vanilla item or a declared custom item (only where `allow_custom`).
    fn data_item_ref(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        allow_custom: bool,
    ) -> Option<String> {
        let expr = self.data_expr(file, value, "an item such as `Item::Diamond`")?;
        if let Some((def, id)) = self.declared(module, file, expr, DefKind::CustomItem) {
            if !allow_custom {
                self.error(
                    "type",
                    file,
                    expr.span(),
                    format!(
                        "`{}` is a custom item; the base must be a vanilla item",
                        self.defs[def.0].name
                    ),
                );
                return None;
            }
            return Some(id);
        }
        self.vanilla_value(module, file, expr, "Item")
    }

    /// Resolve a path to a declaration of `kind`, recording the reference.
    pub(super) fn declared(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        kind: DefKind,
    ) -> Option<(DefId, String)> {
        let Expr::Path(path) = expr else {
            return None;
        };
        let Ok((def, _)) = self.resolve_path(module, path) else {
            return None;
        };
        if self.defs[def.0].kind != kind {
            return None;
        }
        let DefInfo::Data { id, .. } = &self.defs[def.0].info else {
            return None;
        };
        let id = id.clone();
        self.references.push(Reference {
            file,
            span: path.last().span,
            def,
        });
        Some((def, id))
    }

    fn data_smithing_ingredients(
        &mut self,
        module: ModuleId,
        file: FileId,
        template: &DataField,
        base: Option<&DataField>,
        addition: Option<&DataField>,
    ) -> Option<(String, String, String)> {
        let (Some(base), Some(addition)) = (base, addition) else {
            self.error(
                "recipe",
                file,
                template.key.span,
                "a smithing recipe needs `base` and `addition`",
            );
            return None;
        };
        let template = self.data_ingredient(module, file, &template.value);
        let base = self.data_ingredient(module, file, &base.value);
        let addition = self.data_ingredient(module, file, &addition.value);
        Some((template?, base?, addition?))
    }

    fn data_ingredient(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<String> {
        let expr = self.data_expr(file, value, "an ingredient such as `Item::Stick`")?;
        if let Some((def, id)) = self.declared(module, file, expr, DefKind::Tag) {
            if !matches!(
                self.defs[def.0].info,
                DefInfo::Data {
                    tag_type: Some("Item"),
                    ..
                }
            ) {
                self.error(
                    "type",
                    file,
                    expr.span(),
                    "recipe ingredients use `Item` tags",
                );
                return None;
            }
            return Some(format!("#{id}"));
        }
        if let Some((def, _)) = self.declared(module, file, expr, DefKind::CustomItem) {
            let name = self.defs[def.0].name.clone();
            self.diagnostics.push(
                Diagnostic::error("recipe", file, expr.span(), format!("custom item `{name}` cannot be a recipe ingredient"))
                    .with_help("vanilla recipes match ingredients only by item id or tag; use its base item or an item tag"),
            );
            return None;
        }
        self.data_tag_or(module, file, expr, "Item")
    }

    fn data_tag_value(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        registry: &'static str,
    ) -> Option<String> {
        let expr = self.data_expr(file, value, "a value")?;
        if let Some((def, id)) = self.declared(module, file, expr, DefKind::Tag) {
            if !matches!(&self.defs[def.0].info, DefInfo::Data { tag_type: Some(t), .. } if *t == registry)
            {
                self.error(
                    "type",
                    file,
                    expr.span(),
                    format!("expected a `{registry}` tag"),
                );
                return None;
            }
            return Some(format!("#{id}"));
        }
        self.data_tag_or(module, file, expr, registry)
    }

    /// `"#ns:tag"` strings, or a typed vanilla value.
    fn data_tag_or(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        registry: &'static str,
    ) -> Option<String> {
        if let Expr::Str { value, span, .. } = expr
            && let Some(tag) = value.strip_prefix('#')
        {
            if tag.parse::<sand::ResourceLocation>().is_err() || !tag.contains(':') {
                self.error(
                    "id",
                    file,
                    *span,
                    format!("`{value}` is not a tag reference such as `\"#minecraft:logs\"`"),
                );
                return None;
            }
            return Some(value.clone());
        }
        self.vanilla_value(module, file, expr, registry)
    }

    fn data_text(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<Vec<pc::TextPart>> {
        let expr = self.data_expr(file, value, "text in quotes")?.clone();
        let span = expr.span();
        let mut body = BodyCx::new(module, file, false);
        let arg = Arg {
            name: None,
            value: ArgValue::Expr(expr),
            span,
        };
        let parts = self.presentation_text(&mut body, &arg, None, 0)?;
        self.diagnostics.append(&mut body.pending);
        if parts.iter().any(|part| !part.is_static()) || body.player_use.is_some() {
            self.diagnostics.push(
                Diagnostic::error(
                    "text",
                    file,
                    span,
                    "resource text cannot read runtime values",
                )
                .with_help(
                    "use literals, styles or translations with fixed arguments; scores and `{player}` are not available",
                ),
            );
            return None;
        }
        Some(parts)
    }

    fn data_bool(&mut self, file: FileId, value: &DataValue) -> Option<bool> {
        match value {
            DataValue::Expr(Expr::Bool(b, _)) => Some(*b),
            other => {
                self.error("type", file, other.span(), "expected `true` or `false`");
                None
            }
        }
    }

    fn data_int(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        min: i64,
        max: i64,
    ) -> Option<i64> {
        let expr = self.data_expr(file, value, "an integer")?;
        let v = self.eval_i32(module, file, expr, Some(Ty::Number), None)?;
        if v < min || v > max {
            self.error(
                "range",
                file,
                expr.span(),
                format!("expected a value from {min} to {max}, found {v}"),
            );
            return None;
        }
        Some(v)
    }

    fn data_number(&mut self, module: ModuleId, file: FileId, value: &DataValue) -> Option<f64> {
        let expr = self.data_expr(file, value, "a number")?;
        match expr {
            Expr::Float(v, _) => Some(*v),
            other => self
                .eval_i32(module, file, other, Some(Ty::Number), None)
                .map(|v| v as f64),
        }
    }

    fn data_pattern(&mut self, file: FileId, value: &DataValue) -> Option<Vec<String>> {
        let DataValue::Array(rows, span) = value else {
            self.error(
                "type",
                file,
                value.span(),
                "expected rows such as `[\"AAA\", \" B \"]`",
            );
            return None;
        };
        let mut pattern = Vec::new();
        for row in rows {
            match row {
                DataValue::Expr(Expr::Str { value, .. }) => pattern.push(value.clone()),
                other => {
                    self.error("type", file, other.span(), "pattern rows are strings");
                    return None;
                }
            }
        }
        let width = pattern.first().map(|r| r.chars().count()).unwrap_or(0);
        if pattern.is_empty()
            || pattern.len() > 3
            || width == 0
            || width > 3
            || pattern.iter().any(|r| r.chars().count() != width)
        {
            self.diagnostics.push(
                Diagnostic::error(
                    "recipe",
                    file,
                    *span,
                    "a pattern has one to three rows of the same width, at most three wide",
                )
                .with_help("spaces are empty slots: `[\" D \", \" D \", \" S \"]`"),
            );
            return None;
        }
        Some(pattern)
    }

    fn data_enchantments(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<BTreeMap<String, u32>> {
        let mut out = BTreeMap::new();
        match value {
            DataValue::Array(entries, _) => {
                for entry in entries {
                    let (id, level, span) = self.data_enchantment_entry(module, file, entry)?;
                    if out.insert(id.clone(), level).is_some() {
                        self.error(
                            "duplicate",
                            file,
                            span,
                            format!("enchantment `{id}` is listed twice"),
                        );
                        return None;
                    }
                }
            }
            DataValue::Object(entries, _) => {
                let target = self
                    .pack
                    .as_ref()
                    .map_or(sand_version::LATEST_KNOWN, |pack| &pack.minecraft)
                    .to_owned();
                for entry in entries {
                    let id = &entry.key.name;
                    if !id.contains(':') || id.parse::<sand::ResourceLocation>().is_err() {
                        self.error(
                            "id",
                            file,
                            entry.key.span,
                            format!("invalid enchantment identity `{id}`"),
                        );
                        return None;
                    }
                    if id.starts_with("minecraft:")
                        && sand_version::registry::contains(&target, "minecraft:enchantment", id)
                            == Some(false)
                    {
                        self.error(
                            "id",
                            file,
                            entry.key.span,
                            format!("no enchantment `{id}` in Minecraft {target}"),
                        );
                        return None;
                    }
                    let level = self.data_int(module, file, &entry.value, 1, 255)?;
                    if out.insert(id.clone(), level as u32).is_some() {
                        self.error(
                            "duplicate",
                            file,
                            entry.key.span,
                            format!("enchantment `{id}` is listed twice"),
                        );
                        return None;
                    }
                }
            }
            _ => {
                self.error("type", file, value.span(), "expected a list of `{ enchantment: Enchantment::Sharpness, level: 5 }` entries");
                return None;
            }
        }
        Some(out)
    }

    fn data_enchantment_entry(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<(String, u32, Span)> {
        let DataValue::Object(fields, span) = value else {
            self.error(
                "type",
                file,
                value.span(),
                "expected an enchantment and level pair",
            );
            return None;
        };
        let mut enchantment = None;
        let mut level = None;
        for field in fields {
            match field.key.name.as_str() {
                "enchantment" if enchantment.is_none() => {
                    let expr = self.data_expr(file, &field.value, "a typed enchantment")?;
                    enchantment = Some(self.vanilla_value(module, file, expr, "Enchantment")?);
                }
                "level" if level.is_none() => {
                    level = Some(self.data_int(module, file, &field.value, 1, 255)? as u32);
                }
                "enchantment" | "level" => {
                    self.error(
                        "duplicate",
                        file,
                        field.key.span,
                        format!("duplicate enchantment entry field `{}`", field.key.name),
                    );
                    return None;
                }
                _ => {
                    self.error(
                        "field",
                        file,
                        field.key.span,
                        "an enchantment entry has only enchantment and level fields",
                    );
                    return None;
                }
            }
        }
        let (Some(id), Some(level)) = (enchantment, level) else {
            self.error(
                "field",
                file,
                *span,
                "an enchantment entry needs both enchantment and level",
            );
            return None;
        };
        Some((id, level, *span))
    }
}
