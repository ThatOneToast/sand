//! [`ItemStack`] — a concrete, component-bearing item stack.
//!
//! Distinct from [`crate::item::matcher::ItemMatcher`]: a stack represents an
//! item that actually exists (something to give, craft, or place in a
//! container), not a condition used to detect one. See the
//! [`item`](crate::item) module docs for the full stack/matcher split.

use std::fmt;

use crate::error::{Result as SandResult, SandError};
use crate::item::{CustomItem, ItemComponent, ItemStackComponents};
use crate::raw::RawComponent;
use crate::registry::ItemId;
use crate::resource_location::ResourceLocation;

/// Vanilla's maximum stack size for any single item.
pub const MAX_STACK_SIZE: u32 = 99;

/// A concrete item stack: a typed item ID, a validated count, and zero or
/// more data components.
///
/// `ItemStack` is a thin, typed wrapper over the existing [`CustomItem`]
/// component model — every component and raw-escape-hatch API `CustomItem`
/// already has remains reachable here (via [`ItemStack::component`] and
/// [`ItemStack::raw_component`]), and stack identity (item ID) is tracked
/// separately from the free-form `base` string `CustomItem` uses, so callers
/// no longer need to re-derive or re-parse the item ID from a rendered
/// component string.
///
/// # Example
/// ```rust
/// use sand_components::item::stack::ItemStack;
/// use sand_components::item::ItemComponent;
/// use sand_components::registry::ItemId;
///
/// let stack = ItemStack::new(ItemId::minecraft("bow").unwrap())
///     .count(1)
///     .component(ItemComponent::custom_data_marker("special_bow"));
///
/// assert_eq!(stack.id().to_string(), "minecraft:bow");
/// assert_eq!(stack.count_value(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct ItemStack {
    id: ItemId,
    count: u32,
    item: CustomItem,
}

impl ItemStack {
    /// Create a stack of `1` of the given item, with no components.
    pub fn new(id: ItemId) -> Self {
        let item = CustomItem::new(id.to_string());
        Self { id, count: 1, item }
    }

    /// Set the stack count.
    ///
    /// Not validated at call time — matching the rest of Sand's builder
    /// APIs, out-of-range counts (`0` or greater than [`MAX_STACK_SIZE`])
    /// are reported clearly by [`ItemStack::validate`]/[`ItemStack::stack_components`]
    /// rather than by this setter.
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Add or merge a typed item component. See [`ItemComponent`].
    pub fn component(mut self, component: ItemComponent) -> Self {
        self.item = self.item.component(component);
        self
    }

    /// Add a raw item component escape hatch for components not covered by
    /// the typed [`ItemComponent`] API. See [`CustomItem::with_raw_component`].
    pub fn raw_component(mut self, component: RawComponent) -> Self {
        self.item = self.item.with_raw_component(component);
        self
    }

    /// The typed item ID this stack is built on.
    pub fn id(&self) -> &ItemId {
        &self.id
    }

    /// The current stack count.
    pub fn count_value(&self) -> u32 {
        self.count
    }

    /// Read-only access to the underlying [`CustomItem`] component model, for
    /// reusing existing helpers (e.g. [`CustomItem::item_predicate`]).
    pub fn custom_item(&self) -> &CustomItem {
        &self.item
    }

    /// Validate this stack's count and component invariants.
    ///
    /// Checks the count is in `1..=`[`MAX_STACK_SIZE`], then delegates
    /// component validation to [`CustomItem::validate`].
    pub fn validate(&self) -> SandResult<()> {
        if self.count == 0 || self.count > MAX_STACK_SIZE {
            return Err(SandError::ComponentValidation {
                location: item_stack_location(),
                kind: "item_stack".to_string(),
                field: "count".to_string(),
                message: format!(
                    "stack count must be in 1..={MAX_STACK_SIZE} (vanilla's item stack limit), \
                     got {}",
                    self.count
                ),
            });
        }
        self.item.validate()
    }

    /// Structured `(base_item, components)` form used by conversions that
    /// need JSON-shaped output (recipe results, loot results). Validates
    /// first — see [`ItemStack::validate`].
    pub fn stack_components(&self) -> SandResult<ItemStackComponents> {
        self.validate()?;
        self.item.stack_components()
    }
}

impl fmt::Display for ItemStack {
    /// Renders the underlying component-string form (`base[k=v,...]`), the
    /// same syntax accepted by `/give`. Count is not part of this syntax —
    /// pass [`ItemStack::count_value`] to whichever command API accepts a
    /// separate count argument (e.g. `give_count`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.item.fmt(f)
    }
}

fn item_stack_location() -> ResourceLocation {
    ResourceLocation::new("sand", "item_stack").expect("static resource location is always valid")
}

/// Converts a value into a concrete [`ItemStack`].
///
/// Implemented for `ItemStack` itself so APIs that accept `impl IntoItemStack`
/// can also accept an already-built stack directly.
pub trait IntoItemStack {
    fn into_item_stack(self) -> ItemStack;
}

impl IntoItemStack for ItemStack {
    fn into_item_stack(self) -> ItemStack {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ItemComponent;

    fn id(path: &str) -> ItemId {
        ItemId::minecraft(path).unwrap()
    }

    #[test]
    fn new_defaults_to_count_one_and_no_components() {
        let stack = ItemStack::new(id("bow"));
        assert_eq!(stack.count_value(), 1);
        assert_eq!(stack.to_string(), "minecraft:bow");
    }

    #[test]
    fn count_zero_fails_validation() {
        let stack = ItemStack::new(id("bow")).count(0);
        let err = stack.validate().unwrap_err();
        assert!(err.to_string().contains("count"));
    }

    #[test]
    fn count_over_max_fails_validation() {
        let stack = ItemStack::new(id("bow")).count(100);
        let err = stack.validate().unwrap_err();
        assert!(err.to_string().contains("count"));
    }

    #[test]
    fn count_at_max_is_valid() {
        let stack = ItemStack::new(id("arrow")).count(MAX_STACK_SIZE);
        assert!(stack.validate().is_ok());
    }

    #[test]
    fn typed_component_survives_stack_components_conversion() {
        let stack =
            ItemStack::new(id("bow")).component(ItemComponent::custom_data_marker("special_bow"));
        let components = stack.stack_components().unwrap();
        assert_eq!(components.base_item(), "minecraft:bow");
        assert!(!components.is_component_free());
    }

    #[test]
    fn raw_component_escape_hatch_survives_display() {
        let stack =
            ItemStack::new(id("bow")).raw_component(RawComponent::new("modded:widget", "1b"));
        assert_eq!(stack.to_string(), "minecraft:bow[modded:widget=1b]");
    }

    #[test]
    fn invalid_component_state_is_reported_by_validate() {
        // max_stack_size outside 1..=99 is an existing CustomItem invariant —
        // proves ItemStack::validate really delegates rather than only
        // checking count.
        let stack = ItemStack::new(id("bow")).component(ItemComponent::MaxStackSize(0));
        assert!(stack.validate().is_err());
    }
}
