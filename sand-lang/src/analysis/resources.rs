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
                let book = match get("book") {
                    Some(field) => {
                        if base != "minecraft:written_book" {
                            self.error(
                                "item",
                                file,
                                field.key.span,
                                "`book` requires `base: Item::WrittenBook`",
                            );
                            return;
                        }
                        let Some(content) = self.data_book(module, file, &field.value) else {
                            return;
                        };
                        Some(content)
                    }
                    None => None,
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
                    book,
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
            DataKind::Ability => {
                let Some(ticks) = get("ticks").and_then(|field| {
                    self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                }) else {
                    return;
                };
                let when = get("when")
                    .and_then(|field| self.data_boolean_state(module, file, &field.value));
                let start =
                    get("start").and_then(|field| self.data_callback(module, file, &field.value));
                let cost = get("cost").and_then(|field| {
                    let fields = self.data_object_fields(
                        file,
                        &field.value,
                        "ability cost",
                        &["balance", "amount"],
                        &[],
                    )?;
                    let balance =
                        self.data_ability_score(module, file, fields.get("balance")?, false)?;
                    let amount =
                        self.data_int(module, file, fields.get("amount")?, 0, i64::from(i32::MAX))?
                            as u32;
                    Some(pc::AbilityCost { balance, amount })
                });
                let cooldown = get("cooldown").and_then(|field| {
                    let fields = self.data_object_fields(
                        file,
                        &field.value,
                        "ability cooldown",
                        &["timer", "ticks"],
                        &[],
                    )?;
                    let timer =
                        self.data_ability_score(module, file, fields.get("timer")?, true)?;
                    let ticks =
                        self.data_int(module, file, fields.get("ticks")?, 1, i64::from(i32::MAX))?
                            as u32;
                    Some(pc::AbilityCooldown { timer, ticks })
                });
                let projectile = get("projectile").and_then(|field| {
                    let expr = self.data_expr(file, &field.value, "a projectile")?;
                    if let Some((_, id)) = self.declared(module, file, expr, DefKind::Projectile) {
                        id.parse().ok()
                    } else {
                        self.error("type", file, expr.span(), "expected a declared projectile");
                        None
                    }
                });
                self.pointers.push((
                    module_id,
                    format!("/abilities/{}", out.abilities.len()),
                    file,
                    data.name.span,
                ));
                out.abilities.push(pc::AbilityDefinition {
                    id: location,
                    ticks: ticks as u32,
                    when,
                    start,
                    cost,
                    cooldown,
                    projectile,
                });
            }
            DataKind::Progression => {
                let Some(objectives) = get("objectives")
                    .and_then(|field| self.data_progression_objectives(module, file, &field.value))
                else {
                    return;
                };
                let Some(progress) = get("progress").and_then(|field| {
                    self.data_progress_field(module, file, &field.value, objectives.len())
                }) else {
                    return;
                };
                let branches = get("branches")
                    .and_then(|field| {
                        self.data_progression_branches(module, file, &field.value, objectives.len())
                    })
                    .unwrap_or_default();
                self.pointers.push((
                    module_id,
                    format!("/progressions/{}", out.progressions.len()),
                    file,
                    data.name.span,
                ));
                out.progressions.push(pc::ProgressionDefinition {
                    id: location,
                    progress,
                    objectives,
                    branches,
                });
            }
            DataKind::Session => {
                let Some(progression) = get("progression").and_then(|field| {
                    let expr = self.data_expr(file, &field.value, "a progression")?;
                    if let Some((_, id)) = self.declared(module, file, expr, DefKind::Progression) {
                        id.parse().ok()
                    } else {
                        self.error("type", file, expr.span(), "expected a declared progression");
                        None
                    }
                }) else {
                    return;
                };
                let Some(ticks) = get("ticks").and_then(|field| {
                    self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                }) else {
                    return;
                };
                let every = get("every")
                    .and_then(|field| {
                        self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                    })
                    .map(|value| value as u32);
                self.pointers.push((
                    module_id,
                    format!("/sessions/{}", out.sessions.len()),
                    file,
                    data.name.span,
                ));
                out.sessions.push(pc::SessionDefinition {
                    id: location,
                    progression,
                    ticks: ticks as u32,
                    every,
                });
            }
            DataKind::Projectile => {
                let Some(ticks) = get("ticks").and_then(|field| {
                    self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                }) else {
                    return;
                };
                let mut number = |name, max: f64| {
                    let field = get(name)?;
                    let value = self.data_number(module, file, &field.value)?;
                    let min = if name == "damage" { 0.0 } else { 0.001 };
                    if !(min..=max).contains(&value) {
                        self.error(
                            "range",
                            file,
                            field.value.span(),
                            format!("`{name}` must be in {min}..={max}"),
                        );
                        return None;
                    }
                    Some(value)
                };
                let speed = number("speed", 16.0);
                let radius = number("radius", 8.0);
                let damage = number("damage", f64::from(f32::MAX));
                let target = get("target")
                    .and_then(|field| self.data_entity_type(module, file, &field.value));
                let hit =
                    get("hit").and_then(|field| self.data_callback(module, file, &field.value));
                self.pointers.push((
                    module_id,
                    format!("/projectiles/{}", out.projectiles.len()),
                    file,
                    data.name.span,
                ));
                out.projectiles.push(pc::ProjectileDefinition {
                    id: location,
                    ticks: ticks as u32,
                    speed,
                    radius,
                    target,
                    damage,
                    hit,
                });
            }
            DataKind::Menu => {
                let Some(title) =
                    get("title").and_then(|field| self.data_text(module, file, &field.value))
                else {
                    return;
                };
                let Some(ticks) = get("ticks").and_then(|field| {
                    self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                }) else {
                    return;
                };
                let page_size = get("page_size")
                    .and_then(|field| self.data_int(module, file, &field.value, 1, 64))
                    .map(|value| value as u32);
                let Some(choices) = get("choices")
                    .and_then(|field| self.data_menu_choices(module, file, &field.value))
                else {
                    return;
                };
                let views = match get("views") {
                    Some(field) => {
                        let Some(views) = self.data_menu_views(module, file, &field.value) else {
                            return;
                        };
                        views
                    }
                    None => Vec::new(),
                };
                let start = get("start").and_then(|field| self.data_menu_name(file, &field.value));
                let while_open = get("while_open")
                    .and_then(|field| self.data_boolean_state(module, file, &field.value));
                self.pointers.push((
                    module_id,
                    format!("/menus/{}", out.menus.len()),
                    file,
                    data.name.span,
                ));
                out.menus.push(pc::MenuDefinition {
                    id: location,
                    title,
                    ticks: ticks as u32,
                    page_size,
                    choices,
                    views,
                    start,
                    while_open,
                });
            }
            DataKind::Hud => {
                let Some(ticks) = get("ticks").and_then(|field| {
                    self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                }) else {
                    return;
                };
                if get("actionbar").is_none() && get("bossbar").is_none() {
                    self.error(
                        "hud",
                        file,
                        data.name.span,
                        "a HUD needs an actionbar or a boss bar",
                    );
                    return;
                }
                let every = get("every")
                    .and_then(|field| {
                        self.data_int(module, file, &field.value, 1, i64::from(i32::MAX))
                    })
                    .map(|value| value as u32);
                let actionbar = get("actionbar")
                    .and_then(|field| self.data_text(module, file, &field.value))
                    .unwrap_or_default();
                let bossbar = get("bossbar")
                    .and_then(|field| self.data_hud_bossbar(module, file, &field.value));
                self.pointers.push((
                    module_id,
                    format!("/huds/{}", out.huds.len()),
                    file,
                    data.name.span,
                ));
                out.huds.push(pc::HudDefinition {
                    id: location,
                    ticks: ticks as u32,
                    every,
                    actionbar,
                    bossbar,
                });
            }
            DataKind::InventoryTransfer => {
                use sand::inventory::InventoryIndex;
                let max = i64::from(InventoryIndex::MAX);
                let Some(from) =
                    get("from").and_then(|field| self.data_int(module, file, &field.value, 0, max))
                else {
                    return;
                };
                let Some(to) =
                    get("to").and_then(|field| self.data_int(module, file, &field.value, 0, max))
                else {
                    return;
                };
                if from == to {
                    self.error(
                        "inventory transfer",
                        file,
                        get("to").expect("checked").value.span(),
                        "source and destination must be distinct numbered player slots",
                    );
                    return;
                }
                let amount = get("amount")
                    .and_then(|field| self.data_int(module, file, &field.value, 1, 99))
                    .map(|value| value as u32);
                let success =
                    get("success").and_then(|field| self.data_callback(module, file, &field.value));
                let rejected = get("rejected")
                    .and_then(|field| self.data_callback(module, file, &field.value));
                self.pointers.push((
                    module_id,
                    format!("/inventory_transfers/{}", out.inventory_transfers.len()),
                    file,
                    data.name.span,
                ));
                out.inventory_transfers
                    .push(pc::InventoryTransferDefinition {
                        id: location,
                        from: InventoryIndex::new(from as u8).expect("checked slot index"),
                        to: InventoryIndex::new(to as u8).expect("checked slot index"),
                        amount,
                        success,
                        rejected,
                    });
            }
            DataKind::StructureTemplate => {
                use sand::registration::DatapackComponent;
                let Some(field) = get("source") else {
                    return;
                };
                let Some(Expr::Str {
                    value: source,
                    span,
                    ..
                }) = self.data_expr(file, &field.value, "a quoted relative .nbt path")
                else {
                    self.error(
                        "type",
                        file,
                        field.value.span(),
                        "expected a quoted relative .nbt path",
                    );
                    return;
                };
                let template = sand::component::StructureTemplate::new(location, source);
                if let Err(error) = template.validate() {
                    self.error("structure", file, *span, error.to_string());
                    return;
                }
                if let Some(Err(error)) = self.package.structure_assets.get(source) {
                    self.error(
                        "io",
                        file,
                        *span,
                        format!("cannot read structure asset `{source}`: {error}"),
                    );
                    return;
                }
                self.pointers.push((
                    module_id,
                    format!("/structure_templates/{}", out.structure_templates.len()),
                    file,
                    *span,
                ));
                out.structure_templates.push(template);
            }
            DataKind::Biome => {
                let precipitation =
                    get("precipitation").and_then(|field| self.data_bool(file, &field.value));
                let temperature = get("temperature")
                    .and_then(|field| self.data_biome_number(module, file, &field.value));
                let downfall = get("downfall").and_then(|field| {
                    let value = self.data_biome_number(module, file, &field.value)?;
                    if !(0.0..=1.0).contains(&value) {
                        self.error(
                            "range",
                            file,
                            field.value.span(),
                            "downfall must be between zero and one",
                        );
                        return None;
                    }
                    Some(value)
                });
                let mut color =
                    |name| get(name).and_then(|field| self.data_biome_color(file, &field.value));
                let biome = pc::CustomBiome {
                    id: location,
                    precipitation,
                    temperature,
                    downfall,
                    fog_color: color("fog_color"),
                    water_color: color("water_color"),
                    water_fog_color: color("water_fog_color"),
                    sky_color: color("sky_color"),
                    grass_color: color("grass_color"),
                    foliage_color: color("foliage_color"),
                    spawners: get("spawners")
                        .and_then(|field| self.data_biome_spawners(module, file, &field.value))
                        .unwrap_or_default(),
                    spawn_costs: get("spawn_costs")
                        .and_then(|field| self.data_biome_spawn_costs(module, file, &field.value))
                        .unwrap_or_default(),
                };
                self.pointers.push((
                    module_id,
                    format!("/biomes/{}", out.biomes.len()),
                    file,
                    data.name.span,
                ));
                out.biomes.push(biome);
            }
            DataKind::Dimension => {
                let Some(layers) = get("layers")
                    .and_then(|field| self.data_flat_layers(module, file, &field.value))
                else {
                    return;
                };
                let mut flat = sand::component::FlatGenerator::new(layers);
                if let Some(field) = get("biome") {
                    let Some(biome) = self
                        .data_expr(file, &field.value, "a biome")
                        .and_then(|expr| self.vanilla_value(module, file, expr, "Biome"))
                        .and_then(|id| id.parse::<sand::ResourceLocation>().ok())
                    else {
                        return;
                    };
                    flat = flat.biome(biome);
                }
                if let Some(field) = get("structures") {
                    let Some(structures) = self.data_bool(file, &field.value) else {
                        return;
                    };
                    flat = flat.with_structures(structures);
                }
                let dimension_type = if let Some(field) = get("type") {
                    let Some(id) = self
                        .data_expr(file, &field.value, "a dimension type")
                        .and_then(|expr| self.vanilla_value(module, file, expr, "DimensionType"))
                        .and_then(|id| id.parse::<sand::ResourceLocation>().ok())
                    else {
                        return;
                    };
                    id.into()
                } else {
                    sand::registry::DimensionTypeId::minecraft("overworld")
                        .expect("valid built-in id")
                };
                self.pointers.push((
                    module_id,
                    format!("/dimensions/{}", out.dimensions.len()),
                    file,
                    data.name.span,
                ));
                out.dimensions.push(pc::FlatDimension {
                    id: location,
                    dimension_type,
                    flat,
                });
            }
            DataKind::Dialog => {
                let Some(title) =
                    get("title").and_then(|field| self.data_text(module, file, &field.value))
                else {
                    return;
                };
                let body = match get("body").map(|field| &field.value) {
                    Some(DataValue::Array(lines, _)) => {
                        let Some(lines) = lines
                            .iter()
                            .map(|line| self.data_text(module, file, line))
                            .collect::<Option<Vec<_>>>()
                        else {
                            return;
                        };
                        lines
                    }
                    Some(other) => {
                        self.error(
                            "dialog",
                            file,
                            other.span(),
                            "dialog body must be a list of text blocks",
                        );
                        return;
                    }
                    None => Vec::new(),
                };
                let Some(button) = get("button")
                    .map(|field| self.data_text(module, file, &field.value))
                    .unwrap_or_else(|| Some(Vec::new()))
                else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/dialogs/{}", out.dialogs.len()),
                    file,
                    data.name.span,
                ));
                out.dialogs.push(pc::NoticeDialog {
                    id: location,
                    title,
                    body,
                    button,
                });
            }
            DataKind::Advancement => {
                let Some(criteria) = get("criteria")
                    .and_then(|field| self.data_advancement_criteria(module, file, &field.value))
                else {
                    return;
                };
                let display = match get("display") {
                    Some(field) => {
                        let Some(display) =
                            self.data_advancement_display(module, file, &field.value)
                        else {
                            return;
                        };
                        Some(display)
                    }
                    None => None,
                };
                let require_any = get("require_any")
                    .map(|field| self.data_bool(file, &field.value))
                    .unwrap_or(Some(false));
                let experience = get("experience")
                    .map(|field| self.data_int(module, file, &field.value, 0, i32::MAX as i64))
                    .unwrap_or(Some(0));
                let (Some(require_any), Some(experience)) = (require_any, experience) else {
                    return;
                };
                self.pointers.push((
                    module_id,
                    format!("/advancements/{}", out.advancements.len()),
                    file,
                    data.name.span,
                ));
                out.advancements.push(pc::Advancement {
                    id: location,
                    display,
                    criteria,
                    require_any,
                    experience: experience as i32,
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

    pub(super) fn native_data_id(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        kind: DefKind,
    ) -> Option<sand::ResourceLocation> {
        let label = kind.describe();
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
                format!("expected a namespaced {label} identity"),
            );
            return None;
        }
        let Expr::Path(path) = expr else {
            self.error(
                "type",
                file,
                expr.span(),
                format!("expected a declared {label}"),
            );
            return None;
        };
        let def = self.expect_path(
            module,
            path,
            &[kind],
            &format!("{} {label}", super::article(label)),
        )?;
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
                    pool.predicate =
                        Some(self.native_data_id(module, file, expr, DefKind::Predicate)?);
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

    fn data_object_fields<'a>(
        &mut self,
        file: FileId,
        value: &'a DataValue,
        kind: &str,
        required: &[&str],
        optional: &[&str],
    ) -> Option<BTreeMap<&'a str, &'a DataValue>> {
        let DataValue::Object(entries, span) = value else {
            self.error(
                "field",
                file,
                value.span(),
                format!("expected fields in braces for `{kind}`"),
            );
            return None;
        };
        let mut fields = BTreeMap::new();
        for field in entries {
            let name = field.key.name.as_str();
            if !required.contains(&name) && !optional.contains(&name) {
                self.error(
                    "field",
                    file,
                    field.key.span,
                    format!("`{kind}` has no field `{name}`"),
                );
            } else if fields.insert(name, &field.value).is_some() {
                self.error(
                    "field",
                    file,
                    field.key.span,
                    format!("duplicate `{kind}` field `{name}`"),
                );
            }
        }
        for name in required {
            if !fields.contains_key(name) {
                self.error("field", file, *span, format!("`{kind}` needs `{name}`"));
            }
        }
        Some(fields)
    }

    fn data_ability_score(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        timer: bool,
    ) -> Option<pc::FieldReference> {
        let expected = if timer {
            "a player State timer"
        } else {
            "a non-ticking player State number"
        };
        let expr = self.data_expr(file, value, expected)?;
        let Expr::Place(place) = expr else {
            self.error("state", file, value.span(), format!("expected {expected}"));
            return None;
        };
        let (state, field, ty) = self.resolve_place(module, file, place)?;
        if ty != Ty::Number
            || !matches!(
                self.defs[state.0].info,
                DefInfo::State {
                    scope: Scope::Player,
                    ..
                }
            )
            || !matches!(self.defs[field.0].info, DefInfo::Field { timer: found, .. } if found == timer)
        {
            self.error("state", file, value.span(), format!("expected {expected}"));
            return None;
        }
        self.field_reference(state, field)
    }

    fn data_progress_field(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        objectives: usize,
    ) -> Option<pc::FieldReference> {
        let expr = self.data_expr(file, value, "a player State number")?;
        let Expr::Place(place) = expr else {
            self.error(
                "state",
                file,
                value.span(),
                "expected a player State number",
            );
            return None;
        };
        let (state, field, ty) = self.resolve_place(module, file, place)?;
        if !self.types_agree(file, value.span(), Ty::Number, ty) {
            return None;
        }
        if !matches!(
            self.defs[state.0].info,
            DefInfo::State {
                scope: Scope::Player,
                ..
            }
        ) {
            self.error(
                "state",
                file,
                value.span(),
                "progression requires a player State number",
            );
            return None;
        }
        if let DefInfo::Field {
            default,
            bounds,
            timer,
            ..
        } = self.defs[field.0].info
        {
            if timer || default.unwrap_or(0) != 0 {
                self.error(
                    "state",
                    file,
                    value.span(),
                    "progression requires a non-ticking number with default zero",
                );
                return None;
            }
            if bounds.is_some_and(|(min, max)| min > 0 || max < objectives as i32) {
                self.error(
                    "state",
                    file,
                    value.span(),
                    "progression bounds must include zero through the objective count",
                );
                return None;
            }
        }
        self.field_reference(state, field)
    }

    fn data_progression_objectives(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<Vec<pc::ProgressionObjective>> {
        let DataValue::Array(entries, _) = value else {
            self.error(
                "progression",
                file,
                value.span(),
                "expected a list of objectives",
            );
            return None;
        };
        if !(1..=256).contains(&entries.len()) {
            self.error(
                "progression",
                file,
                value.span(),
                "a progression needs between 1 and 256 objectives",
            );
            return None;
        }
        entries
            .iter()
            .map(|entry| {
                let fields =
                    self.data_object_fields(file, entry, "objective", &[], &["when", "reward"])?;
                Some(pc::ProgressionObjective {
                    when: fields
                        .get("when")
                        .and_then(|value| self.data_boolean_state(module, file, value)),
                    reward: fields
                        .get("reward")
                        .and_then(|value| self.data_callback(module, file, value)),
                })
            })
            .collect()
    }

    fn data_progression_branches(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        objectives: usize,
    ) -> Option<Vec<pc::ProgressionBranch>> {
        let DataValue::Array(entries, _) = value else {
            self.error(
                "progression",
                file,
                value.span(),
                "expected a list of forward branches",
            );
            return None;
        };
        if entries.len() > 256 {
            self.error(
                "progression",
                file,
                value.span(),
                "a progression supports at most 256 branches",
            );
            return None;
        }
        entries
            .iter()
            .map(|entry| {
                let fields = self.data_object_fields(
                    file,
                    entry,
                    "progression branch",
                    &["from", "to", "when"],
                    &[],
                )?;
                let from =
                    self.data_int(module, file, fields.get("from")?, 0, objectives as i64 - 1)?;
                let to =
                    self.data_int(module, file, fields.get("to")?, from + 1, objectives as i64)?;
                let when = self.data_boolean_state(module, file, fields.get("when")?)?;
                Some(pc::ProgressionBranch {
                    from: from as u32,
                    to: to as u32,
                    when,
                })
            })
            .collect()
    }

    fn data_menu_choices(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<Vec<pc::MenuChoice>> {
        let DataValue::Array(entries, _) = value else {
            self.error(
                "menu",
                file,
                value.span(),
                "expected a list of menu choices",
            );
            return None;
        };
        if !(1..=256).contains(&entries.len()) {
            self.error(
                "menu",
                file,
                value.span(),
                "a menu needs between 1 and 256 choices",
            );
            return None;
        }
        entries
            .iter()
            .map(|choice| {
                let fields = self.data_object_fields(
                    file,
                    choice,
                    "menu choice",
                    &["label"],
                    &["action", "next", "when"],
                )?;
                if !fields.contains_key("action") && !fields.contains_key("next") {
                    self.error(
                        "menu",
                        file,
                        choice.span(),
                        "a menu choice needs an action or destination",
                    );
                    return None;
                }
                Some(pc::MenuChoice {
                    label: self.data_text(module, file, fields.get("label")?)?,
                    action: fields
                        .get("action")
                        .and_then(|value| self.data_callback(module, file, value)),
                    next: fields
                        .get("next")
                        .and_then(|value| self.data_menu_name(file, value)),
                    when: fields
                        .get("when")
                        .and_then(|value| self.data_boolean_state(module, file, value)),
                })
            })
            .collect()
    }

    fn data_menu_views(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<Vec<pc::MenuView>> {
        let DataValue::Array(views, _) = value else {
            self.error("menu", file, value.span(), "expected a list of menu views");
            return None;
        };
        views
            .iter()
            .map(|view| {
                let fields = self.data_object_fields(
                    file,
                    view,
                    "menu view",
                    &["name", "title", "choices"],
                    &[],
                )?;
                Some(pc::MenuView {
                    name: self.data_menu_name(file, fields.get("name")?)?,
                    title: self.data_text(module, file, fields.get("title")?)?,
                    choices: self.data_menu_choices(module, file, fields.get("choices")?)?,
                })
            })
            .collect()
    }

    fn data_menu_name(&mut self, file: FileId, value: &DataValue) -> Option<String> {
        if let Expr::Str { value, .. } = self.data_expr(file, value, "a quoted menu view name")? {
            return Some(value.clone());
        }
        self.error(
            "menu",
            file,
            value.span(),
            "expected a quoted menu view name",
        );
        None
    }

    fn data_boolean_state(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::FieldReference> {
        let expr = self.data_expr(file, value, "a boolean State field")?;
        let Expr::Place(place) = expr else {
            self.error("menu", file, value.span(), "expected a boolean State field");
            return None;
        };
        let (state, field, ty) = self.resolve_place(module, file, place)?;
        if !self.types_agree(file, value.span(), Ty::Bool, ty) {
            return None;
        }
        self.field_reference(state, field)
    }

    fn data_hud_bossbar(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::HudBossBar> {
        let fields =
            self.data_object_fields(file, value, "HUD boss bar", &["title", "value", "max"], &[])?;
        let title = self.data_text(module, file, fields.get("title")?)?;
        let max = self.data_int(module, file, fields.get("max")?, 1, i64::from(i32::MAX))? as u32;
        let value = self.data_expr(
            file,
            fields.get("value")?,
            "a constant or State number field",
        )?;
        let value = match value {
            Expr::Place(place) => {
                let (state, field, ty) = self.resolve_place(module, file, place)?;
                if !self.types_agree(file, value.span(), Ty::Number, ty) {
                    return None;
                }
                pc::ScoreOperand::Field(self.field_reference(state, field)?)
            }
            _ => pc::ScoreOperand::Value(self.eval_i32(
                module,
                file,
                value,
                Some(Ty::Number),
                None,
            )? as i32),
        };
        Some(pc::HudBossBar { title, value, max })
    }

    fn data_callback(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::FunctionReference> {
        let expr = self.data_expr(file, value, "a callback function name")?;
        let Expr::Path(path) = expr else {
            self.error(
                "type",
                file,
                value.span(),
                "expected a callback function name",
            );
            return None;
        };
        let def = self.expect_path(
            module,
            path,
            &[DefKind::Function, DefKind::ExternFunction],
            "a function",
        )?;
        if self.defs[def.0].kind == DefKind::Function
            && !self
                .fn_slots
                .get(&def)
                .is_some_and(|slots| slots.params.is_empty() && slots.ret.is_none())
        {
            self.error(
                "type",
                file,
                path.span,
                "expected a parameterless function with no return value",
            );
            return None;
        }
        self.function_reference(def)
    }

    fn data_biome_spawners(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<BTreeMap<sand::component::MobCategory, Vec<sand::component::SpawnEntry>>> {
        let DataValue::Object(categories, _) = value else {
            self.error(
                "biome",
                file,
                value.span(),
                "expected spawn categories such as `{ creature: [...] }`",
            );
            return None;
        };
        let mut table = BTreeMap::new();
        for category in categories {
            let Ok(kind) =
                serde_json::from_value(serde_json::Value::String(category.key.name.clone()))
            else {
                self.error(
                    "biome",
                    file,
                    category.key.span,
                    format!("unknown mob spawn category `{}`", category.key.name),
                );
                continue;
            };
            let DataValue::Array(entries, _) = &category.value else {
                self.error(
                    "biome",
                    file,
                    category.value.span(),
                    "expected a list of weighted spawn entries",
                );
                continue;
            };
            let entries = entries
                .iter()
                .map(|entry| {
                    let fields = self.data_object_fields(
                        file,
                        entry,
                        "spawn entry",
                        &["entity"],
                        &["weight", "min_count", "max_count"],
                    )?;
                    let entity = self.data_entity_type(module, file, fields.get("entity")?)?;
                    let mut number = |key, default, min| {
                        fields.get(key).map_or(Some(default), |value| {
                            self.data_int(module, file, value, min, i64::from(i32::MAX))
                        })
                    };
                    let weight = number("weight", 1, 1)?;
                    let min = number("min_count", 1, 1)?;
                    let max = number("max_count", min, min)?;
                    Some(sand::component::SpawnEntry::new(
                        entity,
                        weight as u32,
                        min as u32,
                        max as u32,
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            if table.insert(kind, entries).is_some() {
                self.error(
                    "biome",
                    file,
                    category.key.span,
                    format!("duplicate mob spawn category `{}`", category.key.name),
                );
            }
        }
        Some(table)
    }

    fn data_biome_spawn_costs(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<BTreeMap<String, sand::component::SpawnCost>> {
        let DataValue::Array(entries, _) = value else {
            self.error(
                "biome",
                file,
                value.span(),
                "expected a list of entity spawn costs",
            );
            return None;
        };
        let mut costs = BTreeMap::new();
        for entry in entries {
            let fields = self.data_object_fields(
                file,
                entry,
                "spawn cost",
                &["entity", "energy_budget", "charge"],
                &[],
            )?;
            let entity = self.data_entity_type(module, file, fields.get("entity")?)?;
            let budget = self.data_number(module, file, fields.get("energy_budget")?)?;
            let charge = self.data_number(module, file, fields.get("charge")?)?;
            if costs
                .insert(
                    entity.to_string(),
                    sand::component::SpawnCost::new(budget, charge),
                )
                .is_some()
            {
                self.error(
                    "biome",
                    file,
                    entry.span(),
                    "duplicate spawn cost for this entity type",
                );
            }
        }
        Some(costs)
    }

    fn data_entity_type(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<sand::registry::EntityTypeId> {
        let expr = self.data_expr(file, value, "an entity type")?;
        self.vanilla_value(module, file, expr, "Entity")?
            .parse::<sand::ResourceLocation>()
            .ok()
            .map(Into::into)
    }

    fn data_biome_number(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<f32> {
        let expr = self.data_expr(file, value, "a constant number")?;
        match self.eval(module, file, expr, None)? {
            (value, Ty::Number) => Some(value as f32),
            (value, Ty::Float) => Some((value as f64 / FLOAT_SCALE as f64) as f32),
            _ => {
                self.error("type", file, expr.span(), "expected a constant number");
                None
            }
        }
    }

    fn data_biome_color(&mut self, file: FileId, value: &DataValue) -> Option<u32> {
        let expr = self.data_expr(file, value, "an RGB color")?;
        if let Expr::Str { value, .. } = expr
            && let Some(hex) = value.strip_prefix('#')
            && hex.len() == 6
            && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return u32::from_str_radix(hex, 16).ok();
        }
        self.error(
            "color",
            file,
            expr.span(),
            "expected an RGB color such as `\"#3f76e4\"`",
        );
        None
    }

    fn data_flat_layers(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<Vec<sand::component::FlatLayer>> {
        let DataValue::Array(entries, _) = value else {
            self.error(
                "dimension",
                file,
                value.span(),
                "expected a list of terrain layers",
            );
            return None;
        };
        entries
            .iter()
            .map(|entry| {
                let fields =
                    self.data_object_fields(file, entry, "flat layer", &["block", "height"], &[])?;
                let block = self.data_expr(file, fields.get("block")?, "a block")?;
                let block = self
                    .vanilla_value(module, file, block, "Block")?
                    .parse::<sand::ResourceLocation>()
                    .ok()?;
                let height =
                    self.data_int(module, file, fields.get("height")?, 1, i64::from(u32::MAX))?;
                Some(sand::component::FlatLayer::new(block, height as u32))
            })
            .collect()
    }

    fn data_advancement_criteria(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<BTreeMap<String, pc::AdvancementCriterion>> {
        let DataValue::Object(entries, span) = value else {
            self.error(
                "advancement",
                file,
                value.span(),
                "expected a map of named advancement criteria",
            );
            return None;
        };
        if entries.is_empty() {
            self.error(
                "advancement",
                file,
                *span,
                "an advancement needs at least one criterion",
            );
            return None;
        }
        let mut criteria = BTreeMap::new();
        for field in entries {
            let DataValue::Object(triggers, _) = &field.value else {
                self.error(
                    "advancement",
                    file,
                    field.value.span(),
                    "expected one typed trigger in braces",
                );
                continue;
            };
            let [trigger] = triggers.as_slice() else {
                self.error(
                    "advancement",
                    file,
                    field.value.span(),
                    "an advancement criterion has exactly one trigger",
                );
                continue;
            };
            let criterion = match trigger.key.name.as_str() {
                "inventory_item" => self
                    .data_item_ref(module, file, &trigger.value, false)
                    .map(|item| pc::AdvancementCriterion::InventoryItem { item }),
                "place_block" => self
                    .data_expr(file, &trigger.value, "a Block value")
                    .and_then(|expr| self.vanilla_value(module, file, expr, "Block"))
                    .map(|block| pc::AdvancementCriterion::PlaceBlock { block }),
                "kill_entity" => self
                    .data_expr(file, &trigger.value, "an Entity value")
                    .and_then(|expr| self.vanilla_value(module, file, expr, "Entity"))
                    .map(|entity| pc::AdvancementCriterion::KillEntity { entity }),
                _ => {
                    self.error(
                        "advancement",
                        file,
                        trigger.key.span,
                        "use `inventory_item`, `place_block`, or `kill_entity`",
                    );
                    None
                }
            };
            if let Some(criterion) = criterion
                && criteria.insert(field.key.name.clone(), criterion).is_some()
            {
                self.error(
                    "advancement",
                    file,
                    field.key.span,
                    format!("duplicate criterion `{}`", field.key.name),
                );
            }
        }
        Some(criteria)
    }

    fn data_advancement_display(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::AdvancementDisplay> {
        let fields = self.data_object_fields(
            file,
            value,
            "display",
            &["icon", "title", "description"],
            &["background"],
        )?;
        let icon = self.data_item_ref(module, file, fields.get("icon")?, false)?;
        let title = self.data_text(module, file, fields.get("title")?)?;
        let description = self.data_text(module, file, fields.get("description")?)?;
        let background = match fields.get("background") {
            Some(background) => {
                let id = match background {
                    DataValue::Expr(Expr::Str { value, .. }) => value.parse().ok(),
                    _ => None,
                };
                let Some(id) = id else {
                    self.error(
                        "advancement",
                        file,
                        background.span(),
                        "expected a quoted namespaced background texture id",
                    );
                    return None;
                };
                Some(id)
            }
            None => None,
        };
        Some(pc::AdvancementDisplay {
            icon,
            title,
            description,
            background,
        })
    }

    fn data_book(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<pc::WrittenBookContent> {
        let fields =
            self.data_object_fields(file, value, "book", &["title", "author", "pages"], &[])?;
        let plain = |this: &mut Self, key: &str| match fields.get(key)? {
            DataValue::Expr(Expr::Str { value, .. }) => Some(value.clone()),
            other => {
                this.error(
                    "book",
                    file,
                    other.span(),
                    format!("book {key} must be a quoted string"),
                );
                None
            }
        };
        let title = plain(self, "title")?;
        let author = plain(self, "author")?;
        let DataValue::Array(pages, _) = fields.get("pages")? else {
            self.error(
                "book",
                file,
                fields["pages"].span(),
                "book pages must be a list of text values",
            );
            return None;
        };
        let count = pages.len();
        let pages = pages
            .iter()
            .map(|page| self.data_static_text(module, file, page, Some(count)))
            .collect::<Option<Vec<_>>>()?;
        Some(pc::WrittenBookContent {
            title,
            author,
            pages,
        })
    }

    fn data_text(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
    ) -> Option<Vec<pc::TextPart>> {
        self.data_static_text(module, file, value, None)
    }

    fn data_static_text(
        &mut self,
        module: ModuleId,
        file: FileId,
        value: &DataValue,
        book_pages: Option<usize>,
    ) -> Option<Vec<pc::TextPart>> {
        let expr = self.data_expr(file, value, "text in quotes")?.clone();
        let span = expr.span();
        let mut body = BodyCx::new(module, file, false);
        let arg = Arg {
            name: None,
            value: ArgValue::Expr(expr),
            span,
        };
        let parts =
            self.presentation_text(&mut body, &arg, None, book_pages.is_some(), book_pages, 0)?;
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
