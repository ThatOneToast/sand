//! Typed authoring for Minecraft's data-driven Villager/Wandering Trader
//! trades (Minecraft Java 26.1+ / data pack version 95+).
//!
//! Minecraft models this as two registries, which this module keeps typed
//! and separate while presenting one cohesive authoring workflow:
//!
//! - `data/<namespace>/villager_trade/<id>.json` — a single reusable trade
//!   blueprint ([`VillagerTrade`]);
//! - `data/<namespace>/trade_set/<id>.json` — a trade-selection group that
//!   references one or more blueprints ([`TradeSet`]).
//!
//! # Three distinct author intents
//!
//! 1. **Define a reusable trade blueprint** — a standalone `#[datapack_component] fn
//!    ... -> VillagerTrade`. Exported once, referenced by ID wherever needed.
//! 2. **Create or replace a complete trade set** — [`TradeSet::new`] for a
//!    fresh custom set, or [`TradeSet::replace_target`] to explicitly
//!    replace a known vanilla profession/level/Wandering Trader pool. Both
//!    forms can own inline entries via [`TradeSet::entry`], reuse standalone
//!    trades via [`TradeSet::include`]/[`TradeSet::include_ref`], or select
//!    entirely by tag via [`TradeSet::source_tag`].
//! 3. **Append to an existing profession/Wandering Trader pool** —
//!    [`VillagerTradePoolPatch`]. This only ever *adds* a Villager Trade tag
//!    contribution (`replace: false`); it never replaces vanilla content.
//!
//! These are not interchangeable: creating a `TradeSet` does not, by itself,
//! attach it to any profession. Only [`TradeSet::replace_target`] (a known
//! vanilla target) or a [`VillagerTradePoolPatch`] (an additive tag
//! contribution) affects what a Villager profession/level or the Wandering
//! Trader actually offers.
//!
//! # Inline hoisting
//!
//! [`TradeSet::entry`] takes a stable string key and a closure that builds a
//! [`VillagerTrade`]. Sand hoists each entry into its own generated
//! `villager_trade` resource at `<namespace>:<trade_set path>/<entry key>`
//! (the same namespace and path as the owning `TradeSet`), and the exported
//! `trade_set` JSON references the generated IDs by that same deterministic
//! scheme. For a set at `rpg:blacksmith/novice` with an entry key
//! `"enchanted_pickaxe"`, Sand generates:
//!
//! ```text
//! data/rpg/villager_trade/blacksmith/novice/enchanted_pickaxe.json
//! data/rpg/trade_set/blacksmith/novice.json
//! ```
//!
//! [`VillagerTradePoolPatch::append`] hoists the same way, under the target
//! pool's tag path (e.g. `armorer/level_1/reinforced_helmet`) in an
//! explicitly supplied [`PackNamespace`] — see that type's constructors for
//! why the namespace must be explicit there.
//!
//! Both compound components are validated and collision-checked by the
//! export pipeline before any file is written — see
//! [`crate::component::DatapackComponent::nested_components`].
//!
//! # Scope of this first pass
//!
//! This module covers the common, stable vanilla trade shape: typed input
//! costs, a component-bearing result stack, `max_uses`, `reputation_discount`,
//! `merchant_xp`, and typed double-trade-price enchantment selection. Given
//! item modifiers (`given_item_modifiers`) and merchant predicates
//! (`merchant_predicate`) are accepted as explicit raw JSON escape hatches
//! ([`VillagerTrade::modify_given_item_raw`], [`VillagerTrade::offered_when_raw`])
//! pending the shared typed item-modifier/predicate reference work tracked by
//! #185/#204. Full vanilla trade-function/discount parity is intentionally
//! staged for a follow-up rather than attempted in one change.

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::enchantment_provider::EnchantmentSelection;
use crate::error::Result as SandResult;
use crate::item::stack::ItemStack;
use crate::loot_table::NumberProvider;
use crate::raw::RawJson;
use crate::registry::{ItemId, RandomSequenceId, TagId, VillagerTradeId};
use crate::resource_location::{PackNamespace, ResourceLocation};
use crate::validation;
use sand_version::ComponentFeature;

const VILLAGER_TRADE_DIR: &str = "villager_trade";
const TRADE_SET_DIR: &str = "trade_set";
const VILLAGER_TRADE_TAG_DIR: &str = "tags/villager_trade";

// ── TradeItem ────────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::TradeItem` for the canonical contract."]
/// A single accepted trade cost — the `wants` / `additional_wants` shape.
///
/// Not a concrete item stack: it describes an accepted item ID, an optional
/// count provider (default constant `1`), and an optional expected
/// component map.
#[derive(Debug, Clone, PartialEq)]
pub struct TradeItem {
    id: ItemId,
    count: NumberProvider,
    components_raw: Option<Value>,
}

impl TradeItem {
    /// Create a trade cost accepting one of `id`, with a default count of `1`.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeItem::new` for the canonical contract."]
    pub fn new(id: impl Into<ItemId>) -> Self {
        Self {
            id: id.into(),
            count: NumberProvider::Constant(1.0),
            components_raw: None,
        }
    }

    /// Set the accepted count (a constant or a dynamic number provider).
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeItem::count` for the canonical contract."]
    pub fn count(mut self, count: impl Into<NumberProvider>) -> Self {
        self.count = count.into();
        self
    }

    /// Raw escape hatch for the expected `components` object shape vanilla's
    /// `wants`/`additional_wants` accept. Validated only as "must be a JSON
    /// object" — see [`crate::item`] for the shared typed item/component
    /// model once a fallible `ItemMatcher` → trade-cost conversion lands.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeItem::components_raw` for the canonical contract."]
    pub fn components_raw(mut self, components: RawJson) -> Self {
        self.components_raw = Some(components.into_value());
        self
    }

    fn validate(&self, location: &ResourceLocation, kind: &str, field: &str) -> SandResult<()> {
        if let NumberProvider::Constant(v) = self.count
            && (!v.is_finite() || v < 1.0)
        {
            return Err(validation::error(
                location,
                kind,
                &format!("{field}.count"),
                &format!("trade cost count must be a positive constant (>=1); received {v}"),
            ));
        }
        if let Some(components) = &self.components_raw {
            validation::require_json_object(
                location,
                kind,
                &format!("{field}.components"),
                components,
            )?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("id".to_string(), Value::String(self.id.to_string()));
        map.insert(
            "count".to_string(),
            serde_json::to_value(&self.count).unwrap_or(Value::Null),
        );
        if let Some(components) = &self.components_raw {
            map.insert("components".to_string(), components.clone());
        }
        Value::Object(map)
    }
}

// ── VillagerTrade ────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade` for the canonical contract."]
/// A single `data/<namespace>/villager_trade/<id>.json` blueprint.
///
/// Used both as a standalone `#[datapack_component]` (a reusable trade referenced by
/// multiple [`TradeSet`]s) and as the value built inside
/// [`TradeSet::entry`]/[`VillagerTradePoolPatch::append`] closures, where
/// Sand overwrites the resource location with the deterministic generated
/// child ID after the closure runs.
#[derive(Debug, Clone)]
pub struct VillagerTrade {
    location: ResourceLocation,
    wants: Option<TradeItem>,
    additional_wants: Option<TradeItem>,
    gives: Option<ItemStack>,
    given_item_modifiers: Vec<Value>,
    max_uses: NumberProvider,
    reputation_discount: NumberProvider,
    merchant_xp: NumberProvider,
    merchant_predicate_raw: Option<Value>,
    double_trade_price_enchantments: Option<EnchantmentSelection>,
}

impl VillagerTrade {
    /// Create a new trade blueprint at `location`.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::new` for the canonical contract."]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            wants: None,
            additional_wants: None,
            gives: None,
            given_item_modifiers: Vec::new(),
            max_uses: NumberProvider::Constant(4.0),
            reputation_discount: NumberProvider::Constant(0.0),
            merchant_xp: NumberProvider::Constant(1.0),
            merchant_predicate_raw: None,
            double_trade_price_enchantments: None,
        }
    }

    /// Set the required first input cost (`wants`).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::wants` for the canonical contract."]
    pub fn wants(mut self, item: TradeItem) -> Self {
        self.wants = Some(item);
        self
    }

    /// Set the optional second input cost (`additional_wants`). Vanilla
    /// supports at most one additional cost — calling this again replaces
    /// the previous value rather than accumulating a list.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::and_wants` for the canonical contract."]
    pub fn and_wants(mut self, item: TradeItem) -> Self {
        self.additional_wants = Some(item);
        self
    }

    /// Set the resulting item stack (`gives`). Reuses the shared
    /// component-bearing [`ItemStack`] model, so custom data, names, lore,
    /// enchantments, and other result components survive.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::gives` for the canonical contract."]
    pub fn gives(mut self, stack: impl Into<ItemStack>) -> Self {
        self.gives = Some(stack.into());
        self
    }

    /// Append a raw item-modifier/loot-function JSON object applied to
    /// `gives` when an offer is generated (`given_item_modifiers`).
    ///
    /// Raw escape hatch: 26.1/26.2 only accept inline modifier shapes in
    /// this field (no `ItemModifierRef`); this method does not validate that
    /// constraint beyond "must be a JSON object" pending the typed loot
    /// item-modifier reference work (#185).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::modify_given_item_raw` for the canonical contract."]
    pub fn modify_given_item_raw(mut self, modifier: RawJson) -> Self {
        self.given_item_modifiers.push(modifier.into_value());
        self
    }

    /// Set the maximum number of times this trade can be used
    /// (constant or dynamic number provider; default constant `4`).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::max_uses` for the canonical contract."]
    pub fn max_uses(mut self, max_uses: impl Into<NumberProvider>) -> Self {
        self.max_uses = max_uses.into();
        self
    }

    /// Set how much reputation/demand/discounts affect the first cost
    /// (constant or dynamic number provider; default constant `0.0`).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::reputation_discount` for the canonical contract."]
    pub fn reputation_discount(mut self, discount: impl Into<NumberProvider>) -> Self {
        self.reputation_discount = discount.into();
        self
    }

    /// Set the merchant XP awarded when this trade completes
    /// (constant or dynamic number provider; default constant `1`).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::merchant_xp` for the canonical contract."]
    pub fn merchant_xp(mut self, xp: impl Into<NumberProvider>) -> Self {
        self.merchant_xp = xp.into();
        self
    }

    /// Raw escape hatch for the inline `merchant_predicate` object
    /// restricting which merchant entities may offer this trade.
    ///
    /// 26.1/26.2 do not support predicate references in this field; reuse
    /// the typed predicate model from [`crate::predicate`] once a
    /// predicate-context conversion lands (#204). Validated only as "must be
    /// a JSON object".
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::offered_when_raw` for the canonical contract."]
    pub fn offered_when_raw(mut self, predicate: RawJson) -> Self {
        self.merchant_predicate_raw = Some(predicate.into_value());
        self
    }

    /// Set the enchantment selection that doubles the additional trade cost
    /// when present in the generated result's stored enchantments.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::double_trade_price_enchantments` for the canonical contract."]
    pub fn double_trade_price_enchantments(mut self, selection: EnchantmentSelection) -> Self {
        self.double_trade_price_enchantments = Some(selection);
        self
    }

    /// The resource location this trade is (or will be) exported at.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::location` for the canonical contract."]
    pub fn location(&self) -> &ResourceLocation {
        &self.location
    }

    /// The typed ID this trade is exported at.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTrade::id` for the canonical contract."]
    pub fn id(&self) -> VillagerTradeId {
        VillagerTradeId::custom(self.location.clone())
    }

    /// Return a copy of this trade re-targeted at `location`. Used internally
    /// to hoist inline entries; kept `pub(crate)` since ordinary authors
    /// should use [`VillagerTrade::new`] with the final location directly.
    pub(crate) fn with_location(mut self, location: ResourceLocation) -> Self {
        self.location = location;
        self
    }

    fn validate_fields(
        &self,
        location: &ResourceLocation,
        kind: &str,
        prefix: &str,
    ) -> SandResult<()> {
        let Some(wants) = &self.wants else {
            return Err(validation::error(
                location,
                kind,
                &format!("{prefix}wants"),
                "a villager trade requires `wants`",
            ));
        };
        wants.validate(location, kind, &format!("{prefix}wants"))?;

        if let Some(additional) = &self.additional_wants {
            additional.validate(location, kind, &format!("{prefix}additional_wants"))?;
        }

        let Some(gives) = &self.gives else {
            return Err(validation::error(
                location,
                kind,
                &format!("{prefix}gives"),
                "a villager trade requires `gives`",
            ));
        };
        gives.stack_components().map_err(|e| {
            validation::error(location, kind, &format!("{prefix}gives"), &e.to_string())
        })?;

        for (index, modifier) in self.given_item_modifiers.iter().enumerate() {
            validation::require_json_object(
                location,
                kind,
                &format!("{prefix}given_item_modifiers[{index}]"),
                modifier,
            )?;
        }

        if let NumberProvider::Constant(v) = self.max_uses
            && (!v.is_finite() || v < 1.0)
        {
            return Err(validation::error(
                location,
                kind,
                &format!("{prefix}max_uses"),
                &format!("max_uses constant must be >= 1; received {v}"),
            ));
        }
        if let NumberProvider::Constant(v) = self.reputation_discount
            && (!v.is_finite() || v < 0.0)
        {
            return Err(validation::error(
                location,
                kind,
                &format!("{prefix}reputation_discount"),
                &format!("reputation_discount constant must be >= 0.0; received {v}"),
            ));
        }
        if let NumberProvider::Constant(v) = self.merchant_xp
            && (!v.is_finite() || v < 0.0)
        {
            return Err(validation::error(
                location,
                kind,
                &format!("{prefix}xp"),
                &format!("merchant_xp constant must be >= 0; received {v}"),
            ));
        }
        if let Some(predicate) = &self.merchant_predicate_raw {
            validation::require_json_object(
                location,
                kind,
                &format!("{prefix}merchant_predicate"),
                predicate,
            )?;
        }
        if let Some(selection) = &self.double_trade_price_enchantments {
            selection.validate_with(
                location,
                kind,
                &format!("{prefix}double_trade_price_enchantments"),
            )?;
        }
        Ok(())
    }

    fn to_json_value(&self) -> Value {
        let mut map = Map::new();
        if let Some(wants) = &self.wants {
            map.insert("wants".to_string(), wants.to_json());
        }
        if let Some(additional) = &self.additional_wants {
            map.insert("additional_wants".to_string(), additional.to_json());
        }
        if let Some(gives) = &self.gives
            && let Ok(components) = gives.stack_components()
        {
            let mut give_obj = Map::new();
            give_obj.insert(
                "id".to_string(),
                Value::String(components.base_item().to_string()),
            );
            give_obj.insert(
                "count".to_string(),
                Value::Number(gives.count_value().into()),
            );
            if !components.is_component_free() {
                let mut cmap = Map::new();
                for (key, value) in components.components() {
                    cmap.insert(key.clone(), value.clone());
                }
                give_obj.insert("components".to_string(), Value::Object(cmap));
            }
            map.insert("gives".to_string(), Value::Object(give_obj));
        }
        if !self.given_item_modifiers.is_empty() {
            map.insert(
                "given_item_modifiers".to_string(),
                Value::Array(self.given_item_modifiers.clone()),
            );
        }
        map.insert(
            "max_uses".to_string(),
            serde_json::to_value(&self.max_uses).unwrap_or(Value::Null),
        );
        map.insert(
            "reputation_discount".to_string(),
            serde_json::to_value(&self.reputation_discount).unwrap_or(Value::Null),
        );
        map.insert(
            "xp".to_string(),
            serde_json::to_value(&self.merchant_xp).unwrap_or(Value::Null),
        );
        if let Some(predicate) = &self.merchant_predicate_raw {
            map.insert("merchant_predicate".to_string(), predicate.clone());
        }
        if let Some(selection) = &self.double_trade_price_enchantments {
            map.insert(
                "double_trade_price_enchantments".to_string(),
                selection.to_json_value(),
            );
        }
        Value::Object(map)
    }
}

impl DatapackComponent for VillagerTrade {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        self.to_json_value()
    }

    fn validate(&self) -> SandResult<()> {
        self.validate_fields(&self.location, VILLAGER_TRADE_DIR, "")
    }

    fn required_features(&self) -> &'static [ComponentFeature] {
        &[ComponentFeature::VillagerTrades]
    }

    fn component_dir(&self) -> &'static str {
        VILLAGER_TRADE_DIR
    }
}

// ── Trade source ─────────────────────────────────────────────────────────────

/// One entry in a [`TradeSet`]'s ordered source list — either an inline
/// entry hoisted into a generated `villager_trade` resource, or a reference
/// to an existing one.
#[derive(Debug, Clone)]
enum TradeSetItem {
    Inline {
        key: String,
        trade: Box<VillagerTrade>,
    },
    Reference(VillagerTradeId),
}

#[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradeRef` for the canonical contract."]
/// A reference to a [`VillagerTrade`] resource that is not owned/hoisted by
/// the referencing [`TradeSet`]/[`VillagerTradePoolPatch`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VillagerTradeRef(VillagerTradeId);

impl VillagerTradeRef {
    /// Reference a trade in another pack by its full `namespace:path` ID.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradeRef::external` for the canonical contract."]
    pub fn external(id: &str) -> SandResult<Self> {
        Ok(Self(id.parse()?))
    }

    /// The typed ID this reference points at.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradeRef::id` for the canonical contract."]
    pub fn id(&self) -> &VillagerTradeId {
        &self.0
    }
}

impl From<VillagerTradeId> for VillagerTradeRef {
    fn from(id: VillagerTradeId) -> Self {
        Self(id)
    }
}

fn valid_entry_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '-'))
}

// ── TradeSet ─────────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::TradeSet` for the canonical contract."]
/// A `data/<namespace>/trade_set/<id>.json` trade-selection group.
///
/// Owns zero or more inline entries (hoisted into generated `villager_trade`
/// resources — see [`TradeSet::entry`]), explicit references to standalone
/// trades ([`TradeSet::include`]/[`TradeSet::include_ref`]), or an exclusive
/// tag source ([`TradeSet::source_tag`]).
#[derive(Debug, Clone)]
pub struct TradeSet {
    location: ResourceLocation,
    items: Vec<TradeSetItem>,
    tag_source: Option<TagId<VillagerTradeId>>,
    amount: Option<NumberProvider>,
    allow_duplicates: bool,
    random_sequence: Option<RandomSequenceId>,
}

impl TradeSet {
    /// Create a fresh custom trade set at `location`.
    ///
    /// Defining a `TradeSet` never attaches it to a Villager
    /// profession/level or the Wandering Trader by itself — use
    /// [`TradeSet::replace_target`] to explicitly replace a known vanilla
    /// pool, or [`VillagerTradePoolPatch`] to additively extend one.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::new` for the canonical contract."]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            items: Vec::new(),
            tag_source: None,
            amount: None,
            allow_duplicates: false,
            random_sequence: None,
        }
    }

    /// Explicitly replace a known vanilla trade set (a profession/level,
    /// Common Smith level, or Wandering Trader pool).
    ///
    /// This is the only constructor that targets a vanilla `trade_set`
    /// resource location — making replacement of vanilla content visible in
    /// the call site rather than an implicit side effect of any ordinary
    /// `TradeSet::new` namespace choice.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::replace_target` for the canonical contract."]
    pub fn replace_target(pool: VillagerTradePool) -> Self {
        Self::new(pool.resource_location())
    }

    /// Add an inline trade entry under the stable key `key`, built by
    /// `build`. Sand hoists this into a generated `villager_trade` resource
    /// at `<namespace>:<trade_set path>/<key>` — see the module docs for the
    /// full deterministic scheme.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::entry` for the canonical contract."]
    pub fn entry(
        mut self,
        key: impl Into<String>,
        build: impl FnOnce(VillagerTrade) -> VillagerTrade,
    ) -> Self {
        let key = key.into();
        let placeholder = VillagerTrade::new(self.location.clone());
        let trade = build(placeholder);
        self.items.push(TradeSetItem::Inline {
            key,
            trade: Box::new(trade),
        });
        self
    }

    /// Reference an already-built standalone [`VillagerTrade`] value by its
    /// own resource location, without re-exporting it as a nested resource.
    ///
    /// The referenced trade should also be exported on its own (typically as
    /// a standalone `#[datapack_component] fn ... -> VillagerTrade`) — pass the
    /// value returned by calling that function directly, e.g.
    /// `.include(enchanted_pickaxe_trade())`.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::include` for the canonical contract."]
    pub fn include(mut self, trade: VillagerTrade) -> Self {
        self.items.push(TradeSetItem::Reference(trade.id()));
        self
    }

    /// Reference a trade by typed ID (e.g. an external-pack reference via
    /// [`VillagerTradeRef::external`]).
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::include_ref` for the canonical contract."]
    pub fn include_ref(mut self, reference: VillagerTradeRef) -> Self {
        self.items.push(TradeSetItem::Reference(reference.0));
        self
    }

    /// Select the trade source from a Villager Trade tag instead of inline
    /// entries/explicit references. Mutually exclusive with
    /// [`TradeSet::entry`]/[`TradeSet::include`]/[`TradeSet::include_ref`].
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::source_tag` for the canonical contract."]
    pub fn source_tag(mut self, tag: TagId<VillagerTradeId>) -> Self {
        self.tag_source = Some(tag);
        self
    }

    /// Set how many offers are selected from the source (constant or
    /// dynamic number provider). Required by vanilla; validated as present.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::amount` for the canonical contract."]
    pub fn amount(mut self, amount: impl Into<NumberProvider>) -> Self {
        self.amount = Some(amount.into());
        self
    }

    /// Allow the same trade blueprint to be selected more than once
    /// (default `false`).
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::allow_duplicates` for the canonical contract."]
    pub fn allow_duplicates(mut self, allow: bool) -> Self {
        self.allow_duplicates = allow;
        self
    }

    /// Set the named random sequence controlling trade selection.
    #[doc = "**API Contract:** Run `sand api show sand::component::TradeSet::random_sequence` for the canonical contract."]
    pub fn random_sequence(mut self, sequence: RandomSequenceId) -> Self {
        self.random_sequence = Some(sequence);
        self
    }

    fn child_location(&self, key: &str) -> ResourceLocation {
        ResourceLocation::new(
            self.location.namespace(),
            format!("{}/{}", self.location.path(), key),
        )
        .expect("parent trade_set location plus a validated entry key is always a valid path")
    }

    fn item_ref_string(&self, item: &TradeSetItem) -> String {
        match item {
            TradeSetItem::Inline { key, .. } => self.child_location(key).to_string(),
            TradeSetItem::Reference(id) => id.to_string(),
        }
    }
}

impl DatapackComponent for TradeSet {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        let trades = if let Some(tag) = &self.tag_source {
            Value::String(tag.to_tag_string())
        } else if self.items.len() == 1 {
            Value::String(self.item_ref_string(&self.items[0]))
        } else {
            Value::Array(
                self.items
                    .iter()
                    .map(|item| Value::String(self.item_ref_string(item)))
                    .collect(),
            )
        };
        map.insert("trades".to_string(), trades);
        if let Some(amount) = &self.amount {
            map.insert(
                "amount".to_string(),
                serde_json::to_value(amount).unwrap_or(Value::Null),
            );
        }
        if self.allow_duplicates {
            map.insert("allow_duplicates".to_string(), Value::Bool(true));
        }
        if let Some(sequence) = &self.random_sequence {
            map.insert(
                "random_sequence".to_string(),
                Value::String(sequence.to_string()),
            );
        }
        Value::Object(map)
    }

    fn validate(&self) -> SandResult<()> {
        let kind = TRADE_SET_DIR;
        if self.tag_source.is_some() && !self.items.is_empty() {
            return Err(validation::error(
                &self.location,
                kind,
                "trades",
                "a trade set cannot mix a tag source with inline entries or explicit references",
            ));
        }
        if self.tag_source.is_none() && self.items.is_empty() {
            return Err(validation::error(
                &self.location,
                kind,
                "trades",
                "a trade set requires at least one entry, reference, or a tag source",
            ));
        }

        let mut seen_keys = std::collections::HashSet::new();
        for item in &self.items {
            if let TradeSetItem::Inline { key, trade } = item {
                if !valid_entry_key(key) {
                    return Err(validation::error(
                        &self.location,
                        kind,
                        &format!("entries[{key}]"),
                        "entry keys must be non-empty and contain only [a-z0-9_-]",
                    ));
                }
                if !seen_keys.insert(key.clone()) {
                    return Err(validation::error(
                        &self.location,
                        kind,
                        &format!("entries[{key}]"),
                        "duplicate inline entry key",
                    ));
                }
                trade.validate_fields(&self.location, kind, &format!("entries[{key}]."))?;
            }
        }

        let Some(amount) = &self.amount else {
            return Err(validation::error(
                &self.location,
                kind,
                "amount",
                "a trade set requires `amount`",
            ));
        };
        if let NumberProvider::Constant(v) = amount
            && (!v.is_finite() || *v < 0.0)
        {
            return Err(validation::error(
                &self.location,
                kind,
                "amount",
                &format!("amount constant must be >= 0; received {v}"),
            ));
        }

        Ok(())
    }

    fn nested_components(&self) -> Vec<Box<dyn DatapackComponent>> {
        self.items
            .iter()
            .filter_map(|item| match item {
                TradeSetItem::Inline { key, trade } => {
                    let child = trade.clone().with_location(self.child_location(key));
                    Some(Box::new(child) as Box<dyn DatapackComponent>)
                }
                TradeSetItem::Reference(_) => None,
            })
            .collect()
    }

    fn required_features(&self) -> &'static [ComponentFeature] {
        &[ComponentFeature::VillagerTrades]
    }

    fn component_dir(&self) -> &'static str {
        TRADE_SET_DIR
    }
}

// ── Known pool targets ────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession` for the canonical contract."]
/// A villager profession with a vanilla trade table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VillagerProfession {
    #[doc = "Selects the armorer form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Armorer` for the canonical contract."]
    Armorer,
    #[doc = "Selects the butcher form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Butcher` for the canonical contract."]
    Butcher,
    #[doc = "Selects the cartographer form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Cartographer` for the canonical contract."]
    Cartographer,
    #[doc = "Selects the cleric form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Cleric` for the canonical contract."]
    Cleric,
    #[doc = "Selects the farmer form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Farmer` for the canonical contract."]
    Farmer,
    #[doc = "Selects the fisherman form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Fisherman` for the canonical contract."]
    Fisherman,
    #[doc = "Selects the fletcher form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Fletcher` for the canonical contract."]
    Fletcher,
    #[doc = "Selects the leatherworker form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Leatherworker` for the canonical contract."]
    Leatherworker,
    #[doc = "Selects the librarian form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Librarian` for the canonical contract."]
    Librarian,
    #[doc = "Selects the mason form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Mason` for the canonical contract."]
    Mason,
    #[doc = "Selects the shepherd form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Shepherd` for the canonical contract."]
    Shepherd,
    #[doc = "Selects the toolsmith form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Toolsmith` for the canonical contract."]
    Toolsmith,
    #[doc = "Selects the weaponsmith form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::Weaponsmith` for the canonical contract."]
    Weaponsmith,
}

impl VillagerProfession {
    /// The vanilla path segment for this profession (e.g. `"armorer"`).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerProfession::path` for the canonical contract."]
    pub fn path(self) -> &'static str {
        match self {
            Self::Armorer => "armorer",
            Self::Butcher => "butcher",
            Self::Cartographer => "cartographer",
            Self::Cleric => "cleric",
            Self::Farmer => "farmer",
            Self::Fisherman => "fisherman",
            Self::Fletcher => "fletcher",
            Self::Leatherworker => "leatherworker",
            Self::Librarian => "librarian",
            Self::Mason => "mason",
            Self::Shepherd => "shepherd",
            Self::Toolsmith => "toolsmith",
            Self::Weaponsmith => "weaponsmith",
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel` for the canonical contract."]
/// A villager trade level, `1` (Novice) through `5` (Master).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VillagerLevel {
    #[doc = "Selects the novice form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::Novice` for the canonical contract."]
    Novice,
    #[doc = "Selects the apprentice form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::Apprentice` for the canonical contract."]
    Apprentice,
    #[doc = "Selects the journeyman form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::Journeyman` for the canonical contract."]
    Journeyman,
    #[doc = "Selects the expert form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::Expert` for the canonical contract."]
    Expert,
    #[doc = "Selects the master form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::Master` for the canonical contract."]
    Master,
}

impl VillagerLevel {
    /// The vanilla `1..=5` level number.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::level_number` for the canonical contract."]
    pub fn level_number(self) -> u8 {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 2,
            Self::Journeyman => 3,
            Self::Expert => 4,
            Self::Master => 5,
        }
    }

    /// The vanilla path segment for this level (e.g. `"level_1"`).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerLevel::path` for the canonical contract."]
    pub fn path(self) -> String {
        format!("level_{}", self.level_number())
    }
}

#[doc = "**API Contract:** Run `sand api show sand::component::WanderingTraderPool` for the canonical contract."]
/// A Wandering Trader trade pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WanderingTraderPool {
    #[doc = "Selects the buying form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::WanderingTraderPool::Buying` for the canonical contract."]
    Buying,
    #[doc = "Selects the special form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::WanderingTraderPool::Special` for the canonical contract."]
    Special,
    #[doc = "Selects the common form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::WanderingTraderPool::Common` for the canonical contract."]
    Common,
}

impl WanderingTraderPool {
    /// The vanilla path for this pool (e.g. `"wandering_trader/buying"`).
    #[doc = "**API Contract:** Run `sand api show sand::component::WanderingTraderPool::path` for the canonical contract."]
    pub fn path(self) -> &'static str {
        match self {
            Self::Buying => "wandering_trader/buying",
            Self::Special => "wandering_trader/special",
            Self::Common => "wandering_trader/common",
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool` for the canonical contract."]
/// A known Villager Trade pool target — a profession/level, Common Smith
/// level, Wandering Trader pool, or a custom Villager Trade tag.
///
/// Used by [`TradeSet::replace_target`] (explicit full replacement) and
/// [`VillagerTradePoolPatch`] (additive extension).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VillagerTradePool {
    #[doc = "Selects the profession form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::Profession` for the canonical contract."]
    Profession {
        /// `profession` provides the profession when the variant selects the profession form in this typed Minecraft component schema.
        #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::Profession::profession` for the canonical contract."]
        profession: VillagerProfession,
        /// `level` provides the level range when the variant selects the profession form in this typed Minecraft component schema.
        #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::Profession::level` for the canonical contract."]
        level: VillagerLevel,
    },
    #[doc = "Selects the common smith form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::CommonSmith` for the canonical contract."]
    CommonSmith {
        /// `level` provides the level range when the variant selects the common smith form in this typed Minecraft component schema.
        #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::CommonSmith::level` for the canonical contract."]
        level: VillagerLevel,
    },
    #[doc = "Selects the wandering trader form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::WanderingTrader` for the canonical contract."]
    WanderingTrader(
        #[doc = "The `WanderingTrader` variant carries the value described by its variant semantics: Selects the wandering trader form in this typed Minecraft component schema."]
        #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::WanderingTrader::0` for the canonical contract."]
        WanderingTraderPool,
    ),
    #[doc = "Selects the custom form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::Custom` for the canonical contract."]
    Custom(
        #[doc = "The `Custom` variant carries the value described by its variant semantics: Selects the custom form in this typed Minecraft component schema."]
        #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::Custom::0` for the canonical contract."]
        TagId<VillagerTradeId>,
    ),
}

impl VillagerTradePool {
    /// A profession/level pool, e.g. `armorer/level_1`.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::profession` for the canonical contract."]
    pub fn profession(profession: VillagerProfession, level: VillagerLevel) -> Self {
        Self::Profession { profession, level }
    }

    /// The Common Smith pool shared by Armorer/Toolsmith/Weaponsmith at a level.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::common_smith` for the canonical contract."]
    pub fn common_smith(level: VillagerLevel) -> Self {
        Self::CommonSmith { level }
    }

    /// A Wandering Trader pool.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::wandering_trader` for the canonical contract."]
    pub fn wandering_trader(pool: WanderingTraderPool) -> Self {
        Self::WanderingTrader(pool)
    }

    /// A custom Villager Trade tag target.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::custom` for the canonical contract."]
    pub fn custom(tag: TagId<VillagerTradeId>) -> Self {
        Self::Custom(tag)
    }

    fn path(&self) -> String {
        match self {
            Self::Profession { profession, level } => {
                format!("{}/{}", profession.path(), level.path())
            }
            Self::CommonSmith { level } => format!("common_smith/{}", level.path()),
            Self::WanderingTrader(pool) => pool.path().to_string(),
            Self::Custom(tag) => tag.as_resource_location().path().to_string(),
        }
    }

    /// The Villager Trade tag ID for this pool (`data/minecraft/tags/villager_trade/...`
    /// for known targets; the tag's own namespace for [`VillagerTradePool::Custom`]).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::tag_id` for the canonical contract."]
    pub fn tag_id(&self) -> TagId<VillagerTradeId> {
        match self {
            Self::Custom(tag) => tag.clone(),
            _ => TagId::minecraft(self.path())
                .expect("known villager trade pool paths are valid resource paths"),
        }
    }

    /// The `trade_set` resource location a full replacement of this pool
    /// targets (`data/minecraft/trade_set/...` for known targets).
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePool::resource_location` for the canonical contract."]
    pub fn resource_location(&self) -> ResourceLocation {
        match self {
            Self::Custom(tag) => tag.as_resource_location().clone(),
            _ => ResourceLocation::minecraft(self.path())
                .expect("known villager trade pool paths are valid resource paths"),
        }
    }
}

// ── VillagerTradePoolPatch ────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch` for the canonical contract."]
/// An additive extension of an existing Villager/Wandering Trader pool.
///
/// Unlike [`TradeSet::replace_target`], a pool patch never replaces vanilla
/// content: it only emits generated `villager_trade` resources plus a
/// `replace: false` contribution to the pool's Villager Trade tag.
///
/// The generated trades' namespace must be supplied explicitly
/// ([`PackNamespace`]) rather than inferred: `DatapackComponent` resolves a
/// component's resource location at construction time, and the target tag
/// itself always lives under the vanilla `minecraft` namespace, so there is
/// no other namespace this compound component could otherwise recover its
/// generated children's location from.
///
/// **Known limitation:** two separate `VillagerTradePoolPatch` components
/// targeting the *same* pool are not merged — each is a full `tags/villager_trade/...`
/// JSON record at the same path, so only one wins (export does not currently
/// detect this as a collision, unlike a generated trade colliding with an
/// explicit standalone component). Compose multiple `.append(...)`/`.include(...)`
/// calls on a single patch instead. Cross-patch tag merging (mirroring how
/// `#[datapack_component(Tag = "...")]` function tags already merge) is left as a
/// follow-up.
#[derive(Debug, Clone)]
pub struct VillagerTradePoolPatch {
    namespace: PackNamespace,
    pool: VillagerTradePool,
    /// The target Villager Trade tag's resource location, precomputed from
    /// `pool` at construction time so `DatapackComponent::resource_location`
    /// can return a plain borrow instead of needing to compute (and own) a
    /// fresh `ResourceLocation` per call.
    tag_location: ResourceLocation,
    items: Vec<TradeSetItem>,
}

impl VillagerTradePoolPatch {
    /// Target a profession/level pool.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch::profession` for the canonical contract."]
    pub fn profession(
        namespace: PackNamespace,
        profession: VillagerProfession,
        level: VillagerLevel,
    ) -> Self {
        Self::for_pool(namespace, VillagerTradePool::profession(profession, level))
    }

    /// Target the Common Smith pool at a level.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch::common_smith` for the canonical contract."]
    pub fn common_smith(namespace: PackNamespace, level: VillagerLevel) -> Self {
        Self::for_pool(namespace, VillagerTradePool::common_smith(level))
    }

    /// Target a Wandering Trader pool.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch::wandering_trader` for the canonical contract."]
    pub fn wandering_trader(namespace: PackNamespace, pool: WanderingTraderPool) -> Self {
        Self::for_pool(namespace, VillagerTradePool::wandering_trader(pool))
    }

    /// Target a custom Villager Trade tag.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch::custom` for the canonical contract."]
    pub fn custom(namespace: PackNamespace, tag: TagId<VillagerTradeId>) -> Self {
        Self::for_pool(namespace, VillagerTradePool::custom(tag))
    }

    fn for_pool(namespace: PackNamespace, pool: VillagerTradePool) -> Self {
        let tag_location = pool.tag_id().as_resource_location().clone();
        Self {
            namespace,
            pool,
            tag_location,
            items: Vec::new(),
        }
    }

    /// Append an inline trade entry under key `key`, hoisted into a generated
    /// `villager_trade` resource under this patch's namespace and target
    /// pool path.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch::append` for the canonical contract."]
    pub fn append(
        mut self,
        key: impl Into<String>,
        build: impl FnOnce(VillagerTrade) -> VillagerTrade,
    ) -> Self {
        let key = key.into();
        let placeholder = VillagerTrade::new(self.child_location(&key));
        let trade = build(placeholder);
        self.items.push(TradeSetItem::Inline {
            key,
            trade: Box::new(trade),
        });
        self
    }

    /// Reference an already-built standalone [`VillagerTrade`] value,
    /// without re-exporting it — see [`TradeSet::include`] for the same
    /// pattern.
    #[doc = "**API Contract:** Run `sand api show sand::component::VillagerTradePoolPatch::include` for the canonical contract."]
    pub fn include(mut self, trade: VillagerTrade) -> Self {
        self.items.push(TradeSetItem::Reference(trade.id()));
        self
    }

    fn child_location(&self, key: &str) -> ResourceLocation {
        ResourceLocation::new(
            self.namespace.as_str(),
            format!("{}/{key}", self.pool.path()),
        )
        .expect("pool tag path plus a validated entry key is always a valid path")
    }
}

impl DatapackComponent for VillagerTradePoolPatch {
    fn resource_location(&self) -> &ResourceLocation {
        &self.tag_location
    }

    fn to_json(&self) -> Value {
        let values: Vec<Value> = self
            .items
            .iter()
            .map(|item| match item {
                TradeSetItem::Inline { key, .. } => {
                    Value::String(self.child_location(key).to_string())
                }
                TradeSetItem::Reference(id) => Value::String(id.to_string()),
            })
            .collect();
        serde_json::json!({ "replace": false, "values": values })
    }

    fn validate(&self) -> SandResult<()> {
        let kind = VILLAGER_TRADE_TAG_DIR;
        let location = &self.tag_location;
        if self.items.is_empty() {
            return Err(validation::error(
                location,
                kind,
                "values",
                "a villager trade pool patch requires at least one appended entry or reference",
            ));
        }
        let mut seen_keys = std::collections::HashSet::new();
        for item in &self.items {
            if let TradeSetItem::Inline { key, trade } = item {
                if !valid_entry_key(key) {
                    return Err(validation::error(
                        location,
                        kind,
                        &format!("entries[{key}]"),
                        "entry keys must be non-empty and contain only [a-z0-9_-]",
                    ));
                }
                if !seen_keys.insert(key.clone()) {
                    return Err(validation::error(
                        location,
                        kind,
                        &format!("entries[{key}]"),
                        "duplicate appended entry key",
                    ));
                }
                trade.validate_fields(location, kind, &format!("entries[{key}]."))?;
            }
        }
        Ok(())
    }

    fn nested_components(&self) -> Vec<Box<dyn DatapackComponent>> {
        self.items
            .iter()
            .filter_map(|item| match item {
                TradeSetItem::Inline { key, trade } => {
                    let child = trade.clone().with_location(self.child_location(key));
                    Some(Box::new(child) as Box<dyn DatapackComponent>)
                }
                TradeSetItem::Reference(_) => None,
            })
            .collect()
    }

    fn required_features(&self) -> &'static [ComponentFeature] {
        &[ComponentFeature::VillagerTrades]
    }

    fn component_dir(&self) -> &'static str {
        VILLAGER_TRADE_TAG_DIR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::EnchantmentId;

    fn rl(ns: &str, path: &str) -> ResourceLocation {
        ResourceLocation::new(ns, path).unwrap()
    }

    fn item(path: &str) -> ItemId {
        ItemId::minecraft(path).unwrap()
    }

    // ── TradeItem ──────────────────────────────────────────────────────────

    #[test]
    fn trade_item_defaults_count_to_one() {
        let cost = TradeItem::new(item("emerald"));
        let json = cost.to_json();
        assert_eq!(json["id"], "minecraft:emerald");
        assert_eq!(json["count"], 1.0);
    }

    #[test]
    fn trade_item_count_and_components_serialize() {
        let cost = TradeItem::new(item("diamond_sword"))
            .count(3)
            .components_raw(RawJson::new(
                serde_json::json!({"minecraft:custom_data": {"k": true}}),
            ));
        let json = cost.to_json();
        assert_eq!(json["count"], 3.0);
        assert_eq!(json["components"]["minecraft:custom_data"]["k"], true);
    }

    #[test]
    fn trade_item_rejects_non_positive_constant_count() {
        let cost = TradeItem::new(item("emerald")).count(0);
        let err = cost
            .validate(&rl("test", "loc"), "villager_trade", "wants")
            .unwrap_err();
        assert!(err.to_string().contains("wants.count"));
    }

    // ── VillagerTrade ──────────────────────────────────────────────────────

    fn basic_trade(location: ResourceLocation) -> VillagerTrade {
        VillagerTrade::new(location)
            .wants(TradeItem::new(item("emerald")).count(12))
            .gives(ItemStack::new(item("diamond_pickaxe")))
            .max_uses(2)
            .merchant_xp(30)
    }

    #[test]
    fn villager_trade_requires_wants() {
        let trade =
            VillagerTrade::new(rl("rpg", "trades/foo")).gives(ItemStack::new(item("emerald")));
        let err = trade.validate().unwrap_err();
        assert!(err.to_string().contains("wants"));
    }

    #[test]
    fn villager_trade_requires_gives() {
        let trade =
            VillagerTrade::new(rl("rpg", "trades/foo")).wants(TradeItem::new(item("emerald")));
        let err = trade.validate().unwrap_err();
        assert!(err.to_string().contains("gives"));
    }

    #[test]
    fn villager_trade_golden_json() {
        let trade = basic_trade(rl("rpg", "blacksmith/novice/special_pickaxe"));
        assert!(trade.validate().is_ok());
        let json = trade.to_json();
        assert_eq!(
            json,
            serde_json::json!({
                "wants": {"id": "minecraft:emerald", "count": 12.0},
                "gives": {"id": "minecraft:diamond_pickaxe", "count": 1},
                "max_uses": 2.0,
                "reputation_discount": 0.0,
                "xp": 30.0,
            })
        );
    }

    #[test]
    fn villager_trade_and_wants_and_modifiers_and_predicate_render() {
        let trade = basic_trade(rl("rpg", "trades/foo"))
            .and_wants(TradeItem::new(item("diamond")))
            .modify_given_item_raw(RawJson::new(
                serde_json::json!({"function": "minecraft:enchant_randomly"}),
            ))
            .offered_when_raw(RawJson::new(
                serde_json::json!({"condition": "minecraft:entity_properties"}),
            ))
            .double_trade_price_enchantments(EnchantmentSelection::one(
                EnchantmentId::minecraft("mending").unwrap(),
            ));
        assert!(trade.validate().is_ok());
        let json = trade.to_json();
        assert_eq!(json["additional_wants"]["id"], "minecraft:diamond");
        assert_eq!(
            json["given_item_modifiers"][0]["function"],
            "minecraft:enchant_randomly"
        );
        assert_eq!(
            json["merchant_predicate"]["condition"],
            "minecraft:entity_properties"
        );
        assert_eq!(json["double_trade_price_enchantments"], "minecraft:mending");
    }

    #[test]
    fn villager_trade_rejects_max_uses_below_one() {
        let trade = basic_trade(rl("rpg", "trades/foo")).max_uses(0);
        assert!(
            trade
                .validate()
                .unwrap_err()
                .to_string()
                .contains("max_uses")
        );
    }

    #[test]
    fn villager_trade_rejects_negative_reputation_discount() {
        let trade = basic_trade(rl("rpg", "trades/foo")).reputation_discount(-1.0);
        assert!(
            trade
                .validate()
                .unwrap_err()
                .to_string()
                .contains("reputation_discount")
        );
    }

    #[test]
    fn villager_trade_rejects_negative_merchant_xp() {
        let trade = basic_trade(rl("rpg", "trades/foo")).merchant_xp(-1);
        assert!(trade.validate().unwrap_err().to_string().contains("xp"));
    }

    #[test]
    fn villager_trade_requires_features_villager_trades() {
        let trade = basic_trade(rl("rpg", "trades/foo"));
        assert_eq!(
            trade.required_features(),
            &[ComponentFeature::VillagerTrades]
        );
    }

    #[test]
    fn villager_trade_component_dir_is_villager_trade() {
        assert_eq!(
            basic_trade(rl("rpg", "trades/foo")).component_dir(),
            "villager_trade"
        );
    }

    // ── TradeSet ───────────────────────────────────────────────────────────

    #[test]
    fn trade_set_single_inline_entry_hoists_deterministically() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .entry("special_pickaxe", |trade| {
                trade
                    .wants(TradeItem::new(item("emerald")).count(12))
                    .gives(ItemStack::new(item("diamond_pickaxe")))
                    .max_uses(2)
                    .merchant_xp(30)
            });
        assert!(set.validate().is_ok());
        let json = set.to_json();
        assert_eq!(json["trades"], "rpg:blacksmith/novice/special_pickaxe");
        assert_eq!(json["amount"], 1.0);

        let nested = set.nested_components();
        assert_eq!(nested.len(), 1);
        assert_eq!(
            nested[0].resource_location().to_string(),
            "rpg:blacksmith/novice/special_pickaxe"
        );
        assert_eq!(nested[0].component_dir(), "villager_trade");
    }

    #[test]
    fn trade_set_multiple_entries_serialize_as_array_and_preserve_order() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(2)
            .entry("enchanted_pickaxe", |trade| {
                trade
                    .wants(TradeItem::new(item("emerald")).count(12))
                    .gives(ItemStack::new(item("diamond_pickaxe")))
            })
            .entry("coal_purchase", |trade| {
                trade
                    .wants(TradeItem::new(item("coal")).count(15))
                    .gives(ItemStack::new(item("emerald")))
            });
        let json = set.to_json();
        assert_eq!(
            json["trades"],
            serde_json::json!([
                "rpg:blacksmith/novice/enchanted_pickaxe",
                "rpg:blacksmith/novice/coal_purchase",
            ])
        );
    }

    #[test]
    fn trade_set_export_is_deterministic_across_repeated_calls() {
        let build = || {
            TradeSet::new(rl("rpg", "blacksmith/novice"))
                .amount(1)
                .entry("special_pickaxe", |trade| {
                    trade
                        .wants(TradeItem::new(item("emerald")).count(12))
                        .gives(ItemStack::new(item("diamond_pickaxe")))
                })
        };
        assert_eq!(build().to_json(), build().to_json());
        assert_eq!(
            build().nested_components()[0].resource_location(),
            build().nested_components()[0].resource_location()
        );
    }

    #[test]
    fn trade_set_include_references_without_duplicating_export() {
        let shared = basic_trade(rl("rpg", "trades/shared_pickaxe"));
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .include(shared.clone());
        assert!(set.validate().is_ok());
        assert_eq!(set.to_json()["trades"], "rpg:trades/shared_pickaxe");
        assert!(set.nested_components().is_empty());
    }

    #[test]
    fn trade_set_include_ref_external_pack() {
        let external = VillagerTradeRef::external("other_pack:trade").unwrap();
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .include_ref(external);
        assert_eq!(set.to_json()["trades"], "other_pack:trade");
    }

    #[test]
    fn trade_set_source_tag_renders_hash_prefixed_string() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .source_tag(TagId::minecraft("common_smith/level_1").unwrap());
        assert!(set.validate().is_ok());
        assert_eq!(set.to_json()["trades"], "#minecraft:common_smith/level_1");
    }

    #[test]
    fn trade_set_rejects_mixing_tag_source_with_entries() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .entry("foo", |t| {
                t.wants(TradeItem::new(item("emerald")))
                    .gives(ItemStack::new(item("diamond")))
            })
            .source_tag(TagId::minecraft("common_smith/level_1").unwrap());
        assert!(set.validate().unwrap_err().to_string().contains("trades"));
    }

    #[test]
    fn trade_set_rejects_empty_source() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice")).amount(1);
        assert!(set.validate().unwrap_err().to_string().contains("trades"));
    }

    #[test]
    fn trade_set_rejects_duplicate_entry_keys() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .entry("dup", |t| {
                t.wants(TradeItem::new(item("emerald")))
                    .gives(ItemStack::new(item("diamond")))
            })
            .entry("dup", |t| {
                t.wants(TradeItem::new(item("coal")))
                    .gives(ItemStack::new(item("emerald")))
            });
        assert!(
            set.validate()
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
    }

    #[test]
    fn trade_set_rejects_missing_amount() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice")).entry("foo", |t| {
            t.wants(TradeItem::new(item("emerald")))
                .gives(ItemStack::new(item("diamond")))
        });
        assert!(set.validate().unwrap_err().to_string().contains("amount"));
    }

    #[test]
    fn trade_set_error_path_points_through_owning_entry() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .entry("enchanted_pickaxe", |trade| {
                trade.wants(TradeItem::new(item("emerald")).count(0))
            });
        let err = set.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("entries[enchanted_pickaxe].wants.count")
        );
    }

    #[test]
    fn trade_set_replace_target_uses_known_vanilla_location() {
        let set = TradeSet::replace_target(VillagerTradePool::profession(
            VillagerProfession::Armorer,
            VillagerLevel::Novice,
        ))
        .amount(2)
        .entry("foo", |t| {
            t.wants(TradeItem::new(item("emerald")))
                .gives(ItemStack::new(item("diamond")))
        });
        assert_eq!(
            set.resource_location().to_string(),
            "minecraft:armorer/level_1"
        );
    }

    #[test]
    fn trade_set_component_bearing_result_survives_hoist() {
        let set = TradeSet::new(rl("rpg", "blacksmith/novice"))
            .amount(1)
            .entry("marked_pickaxe", |trade| {
                trade
                    .wants(TradeItem::new(item("emerald")).count(12))
                    .gives(ItemStack::new(item("diamond_pickaxe")).component(
                        crate::item::ItemComponent::custom_data_marker("special_pickaxe"),
                    ))
            });
        let nested = set.nested_components();
        let json = nested[0].to_json();
        assert_eq!(
            json["gives"]["components"]["minecraft:custom_data"]["special_pickaxe"],
            true
        );
    }

    // ── VillagerTradePool / typed target helpers ──────────────────────────

    #[test]
    fn villager_trade_pool_profession_path() {
        let pool =
            VillagerTradePool::profession(VillagerProfession::Weaponsmith, VillagerLevel::Master);
        assert_eq!(pool.tag_id().to_string(), "minecraft:weaponsmith/level_5");
        assert_eq!(
            pool.resource_location().to_string(),
            "minecraft:weaponsmith/level_5"
        );
    }

    #[test]
    fn villager_trade_pool_common_smith_path() {
        let pool = VillagerTradePool::common_smith(VillagerLevel::Expert);
        assert_eq!(pool.tag_id().to_string(), "minecraft:common_smith/level_4");
    }

    #[test]
    fn villager_trade_pool_wandering_trader_paths() {
        assert_eq!(
            VillagerTradePool::wandering_trader(WanderingTraderPool::Buying)
                .tag_id()
                .to_string(),
            "minecraft:wandering_trader/buying"
        );
        assert_eq!(
            VillagerTradePool::wandering_trader(WanderingTraderPool::Special)
                .tag_id()
                .to_string(),
            "minecraft:wandering_trader/special"
        );
        assert_eq!(
            VillagerTradePool::wandering_trader(WanderingTraderPool::Common)
                .tag_id()
                .to_string(),
            "minecraft:wandering_trader/common"
        );
    }

    #[test]
    fn villager_trade_pool_custom_uses_tag_namespace() {
        let tag = TagId::<VillagerTradeId>::custom(rl("mypack", "special_trades"));
        let pool = VillagerTradePool::custom(tag);
        assert_eq!(pool.tag_id().to_string(), "mypack:special_trades");
        assert_eq!(
            pool.resource_location().to_string(),
            "mypack:special_trades"
        );
    }

    // ── VillagerTradePoolPatch ─────────────────────────────────────────────

    fn ns(s: &str) -> PackNamespace {
        PackNamespace::new(s).unwrap()
    }

    #[test]
    fn pool_patch_profession_generates_children_and_tag() {
        let patch = VillagerTradePoolPatch::profession(
            ns("rpg"),
            VillagerProfession::Armorer,
            VillagerLevel::Novice,
        )
        .append("reinforced_helmet", |trade| {
            trade
                .wants(TradeItem::new(item("emerald")).count(8))
                .gives(ItemStack::new(item("iron_helmet")))
                .max_uses(4)
                .merchant_xp(10)
        });
        assert!(patch.validate().is_ok());
        assert_eq!(
            patch.resource_location().to_string(),
            "minecraft:armorer/level_1"
        );
        assert_eq!(patch.component_dir(), "tags/villager_trade");

        let json = patch.to_json();
        assert_eq!(json["replace"], false);
        assert_eq!(json["values"][0], "rpg:armorer/level_1/reinforced_helmet");

        let nested = patch.nested_components();
        assert_eq!(nested.len(), 1);
        assert_eq!(
            nested[0].resource_location().to_string(),
            "rpg:armorer/level_1/reinforced_helmet"
        );
    }

    #[test]
    fn pool_patch_wandering_trader_target() {
        let patch =
            VillagerTradePoolPatch::wandering_trader(ns("rpg"), WanderingTraderPool::Special)
                .append("gem", |trade| {
                    trade
                        .wants(TradeItem::new(item("emerald")))
                        .gives(ItemStack::new(item("diamond")))
                });
        assert_eq!(
            patch.resource_location().to_string(),
            "minecraft:wandering_trader/special"
        );
    }

    #[test]
    fn pool_patch_rejects_empty() {
        let patch = VillagerTradePoolPatch::common_smith(ns("rpg"), VillagerLevel::Novice);
        assert!(patch.validate().unwrap_err().to_string().contains("values"));
    }

    #[test]
    fn pool_patch_rejects_duplicate_keys() {
        let patch = VillagerTradePoolPatch::common_smith(ns("rpg"), VillagerLevel::Novice)
            .append("dup", |t| {
                t.wants(TradeItem::new(item("emerald")))
                    .gives(ItemStack::new(item("diamond")))
            })
            .append("dup", |t| {
                t.wants(TradeItem::new(item("coal")))
                    .gives(ItemStack::new(item("emerald")))
            });
        assert!(
            patch
                .validate()
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
    }

    #[test]
    fn pool_patch_include_references_without_nested_export() {
        let shared = basic_trade(rl("rpg", "trades/shared"));
        let patch = VillagerTradePoolPatch::custom(
            ns("rpg"),
            TagId::minecraft("custom_pool_placeholder").unwrap(),
        )
        .include(shared.clone());
        assert!(patch.validate().is_ok());
        assert!(patch.nested_components().is_empty());
        assert_eq!(patch.to_json()["values"][0], "rpg:trades/shared");
    }

    #[test]
    fn pool_patch_requires_villager_trades_feature() {
        let patch = VillagerTradePoolPatch::common_smith(ns("rpg"), VillagerLevel::Novice).append(
            "x",
            |t| {
                t.wants(TradeItem::new(item("emerald")))
                    .gives(ItemStack::new(item("diamond")))
            },
        );
        assert_eq!(
            patch.required_features(),
            &[ComponentFeature::VillagerTrades]
        );
    }

    // ── VillagerTradeRef / entry key validation ────────────────────────────

    #[test]
    fn entry_key_validation() {
        assert!(valid_entry_key("enchanted_pickaxe"));
        assert!(valid_entry_key("coal-purchase"));
        assert!(!valid_entry_key(""));
        assert!(!valid_entry_key("Bad Key"));
        assert!(!valid_entry_key("has/slash"));
    }
}
