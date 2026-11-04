//! Explicit player journal snapshots using canonical State, text and item slots.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    entity::{EntityText, EntityTextSegment, state::StateFieldReference},
    ir::{Actions, Cmd},
    item::{InventoryIndex, ItemLocation},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
    state::{StateDescriptor, StateScope},
};
use sand_commands::{
    CommandError, CommandResult, NbtCompound, NbtValue, ObjectiveName, TextComponent,
};
use sand_components::{
    CustomItem, ItemComponent, ItemId, ItemPredicate, ResourceLocation, WrittenBook,
};
use std::{collections::BTreeMap, sync::Arc};

/// A refreshed written-book snapshot in one explicitly selected player slot.
///
/// Pages reuse canonical EntityText. Numeric, enum and flag segments read current
/// attached State and materialize literal content before delivery, so later State
/// changes do not change this book until another successful refresh. Numeric
/// segments retain EntityText's stored integer rendering. There are 1..=100 pages
/// and at most 64 segments per page. Required fields and current schema membership
/// are checked before changing inventory. Unresolved canonical score/NBT/selector
/// components are rejected; use typed EntityText field segments for snapshots.
///
/// Empty slots admit by default. Explicit [`Self::replace_owned`] allows replacing
/// a single written book carrying this definition's unique custom-data marker.
/// The marker identifies the journal definition, not an original player or a
/// secret claim. Transferring a book allows its recipient to refresh it from their
/// own State. Unrelated items are preserved. There is no slot search, overflow
/// dropping, forced opening, persistent book cache or automatic update scheduler.
///
/// Native writes share Purchase/InventoryTransfer's pack inventory lock. Reentry
/// while locked is a silent no-op. After writing, the slot must still contain one
/// matching journal with exactly the staged written-book content; listener-added
/// unrelated components may remain. Changed/removed content or identity rejects
/// without undoing arbitrary listener changes. Callbacks run unlocked in isolated
/// player context. Native delivery failure never changes gameplay State itself.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal", module = "sand::systems",
 summary = "Refresh explicit player book snapshots from canonical State.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],

 example = "use sand::systems::Journal;")]
pub struct Journal {
    id: ResourceLocation,
    title: String,
    author: String,
    slot: InventoryIndex,
    pages: Vec<EntityText>,
    replace: bool,
    success: Option<Arc<dyn Fn() -> Actions>>,
    rejected: Option<Arc<dyn Fn() -> Actions>>,
}
impl Journal {
    /// Define a book title/author and a destination numbered player slot.
    /// Delivery initially requires an empty slot. Register before refresh.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal::new", module = "sand::systems",
 summary = "Define an explicit-slot player journal.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],
 params(id = "Unique registered journal identity.", title = "Book title, at most 32 UTF-16 code units.", author = "Native book author label.", slot = "Numbered player slot used for delivery and optional replacement."),
 returns = "The configured journal or retained player-context actions.", example = "use sand::systems::Journal;")]
    pub fn new(
        id: ResourceLocation,
        title: impl Into<String>,
        author: impl Into<String>,
        slot: InventoryIndex,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            author: author.into(),
            slot,
            pages: vec![],
            replace: false,
            success: None,
            rejected: None,
        }
    }
    /// Append a canonical State-aware page, bounded to 64 segments.
    /// Dynamic field segments are resolved at refresh time, before delivery.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal::page", module = "sand::systems",
 summary = "Append a State-aware snapshot page.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],
 params(page = "Canonical EntityText literals and typed field segments."),
 returns = "The configured journal or retained player-context actions.", example = "use sand::systems::Journal;")]
    pub fn page(mut self, page: EntityText) -> Self {
        self.pages.push(page);
        self
    }
    /// Explicitly allow refreshing an existing single book with this journal's
    /// definition marker. Other items and differently identified books reject.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal::replace_owned", module = "sand::systems",
 summary = "Allow replacing this journal identity in its fixed slot.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],

 returns = "The configured journal or retained player-context actions.", example = "use sand::systems::Journal;")]
    pub fn replace_owned(mut self) -> Self {
        self.replace = true;
        self
    }
    /// Continue after native placement and content/identity revalidation.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal::on_success", module = "sand::systems",
 summary = "Continue after checked journal delivery.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],
 params(action = "Player-context actions after inventory scratch and lock release."),
 returns = "The configured journal or retained player-context actions.", example = "use sand::systems::Journal;")]
    pub fn on_success<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.success = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Run after admission, State validation, native delivery or post-write check
    /// rejection. Non-player and locked reentrant calls are silent no-ops.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal::on_rejected", module = "sand::systems",
 summary = "Handle rejected journal delivery.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],
 params(action = "Player-context actions after inventory scratch and lock release."),
 returns = "The configured journal or retained player-context actions.", example = "use sand::systems::Journal;")]
    pub fn on_rejected<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.rejected = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Render and deliver one fresh literal snapshot for the executing player.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Journal::refresh", module = "sand::systems",
 summary = "Render and deliver a checked player journal snapshot.", context = "Render literal book snapshots from canonical State through EntityText, then deliver to one explicit player slot. Empty slots admit by default; replacing the same journal identity requires explicit opt-in. Native delivery and post-write content checks precede callbacks.",
 minecraft = "Uses native written_book_content and canonical item slots under the shared inventory mutation lock. Author text never appears inside macro bodies; no forced opening or persistent journal cache.",
 use_when = ["Refreshing a player's quest journal from canonical State"], avoid_when = ["Arbitrary text input, forced book opening, or automatic persistent book tracking"],

 returns = "The configured journal or retained player-context actions.", example = "use sand::systems::Journal;")]
    pub fn refresh(&self) -> Actions {
        identity::call(&self.id, "journal_refresh")
    }

    fn marker(&self) -> CommandResult<String> {
        let id = crate::function::resolve_export_location(&self.id)?;
        Ok(format!(
            "sand.journal.{}",
            id.to_string()
                .bytes()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        ))
    }
    fn book(&self) -> CommandResult<CustomItem> {
        if self.pages.is_empty() || self.pages.len() > 100 {
            return Err(CommandError::new(
                "journal",
                "pages",
                "requires 1..=100 pages",
            ));
        }
        let mut book = WrittenBook::new(&self.title, &self.author);
        for page in &self.pages {
            if page.segments().len() > 64 {
                return Err(CommandError::new(
                    "journal",
                    "page",
                    "at most 64 segments per page",
                ));
            }
            let mut maximum = TextComponent::literal("");
            for segment in page.segments() {
                let (part, color) = match segment {
                    EntityTextSegment::Canonical { component } => {
                        crate::entity::text::literal_component(component)?;
                        (component.clone(), None)
                    }
                    EntityTextSegment::Literal { text, color } => {
                        (TextComponent::literal(text), *color)
                    }
                    EntityTextSegment::Numeric { color, .. } => {
                        (TextComponent::literal("-2147483648"), *color)
                    }
                    EntityTextSegment::Enum {
                        variants, color, ..
                    } => (
                        TextComponent::literal(
                            variants
                                .iter()
                                .map(|(_, name)| name)
                                .max_by_key(|name| {
                                    serde_json::to_string(name)
                                        .expect("Rust string JSON")
                                        .encode_utf16()
                                        .count()
                                })
                                .cloned()
                                .unwrap_or_default(),
                        ),
                        *color,
                    ),
                    EntityTextSegment::Flag {
                        enabled,
                        disabled,
                        color,
                        ..
                    } => (
                        TextComponent::literal(
                            if serde_json::to_string(enabled)
                                .expect("Rust string JSON")
                                .encode_utf16()
                                .count()
                                > serde_json::to_string(disabled)
                                    .expect("Rust string JSON")
                                    .encode_utf16()
                                    .count()
                            {
                                enabled
                            } else {
                                disabled
                            },
                        ),
                        *color,
                    ),
                };
                maximum = maximum.then(if let Some(color) = color {
                    part.color(color)
                } else {
                    part
                });
            }
            book = book.page(maximum);
        }
        let item = CustomItem::new(ItemId::minecraft("written_book").expect("vanilla item"))
            .custom_data(self.marker()?)
            .component(ItemComponent::WrittenBook(book));
        item.try_to_string()
            .map_err(|e| CommandError::new("journal", "book", e.to_string()))?;
        Ok(item)
    }
    fn references(&self) -> Vec<StateFieldReference> {
        self.pages
            .iter()
            .flat_map(|page| page.segments())
            .filter_map(|segment| match segment {
                EntityTextSegment::Numeric { field, .. }
                | EntityTextSegment::Enum { field, .. }
                | EntityTextSegment::Flag { field, .. } => Some(field.clone()),
                _ => None,
            })
            .collect()
    }
    fn schemas(&self) -> CommandResult<BTreeMap<String, &'static StateDescriptor<'static>>> {
        let mut schemas = BTreeMap::new();
        for reference in self.references() {
            let schema = inventory::iter::<StateDescriptor>
                .into_iter()
                .find(|schema| schema.id == reference.component)
                .ok_or_else(|| {
                    CommandError::new("journal", "state", "requires registered canonical State")
                })?;
            if !schema.fields.iter().any(|field| {
                ObjectiveName::logical(field.objective).as_str() == reference.objective
                    && field.field == reference.descriptor
            }) {
                return Err(CommandError::new(
                    "journal",
                    "state",
                    "field metadata does not match registered State",
                ));
            }
            schemas.insert(reference.component, schema);
        }
        Ok(schemas)
    }
    fn holder(schema: &StateDescriptor<'static>) -> &'static str {
        match schema.scope {
            StateScope::Global(holder) => holder,
            _ => "@s",
        }
    }
    fn build(&self, part: &str) -> CommandResult<Actions> {
        let book = self.book()?;
        let schemas = self.schemas()?;
        let id = crate::function::resolve_export_location(&self.id)?;
        let scratch = identity::resource(&id, "journal_scratch");
        let function = |part: &str| identity::resource(&id, &format!("journal_{part}"));
        let lock = super::inventory_transfer::inventory_lock(&id)?;
        let delivered = ObjectiveName::logical(format!("sand journal {id} delivery"));
        crate::function::request_numeric_objectives([delivered.clone()])?;
        let slot = ItemLocation::PlayerInventory(self.slot);
        let sand_commands::inventory::ItemCommandLocation::Entity {
            slot: native_slot, ..
        } = slot.item_target_slot().expect("numbered player slot")
        else {
            unreachable!()
        };
        let owned = ItemPredicate::new()
            .item(ItemId::minecraft("written_book").expect("vanilla item"))
            .custom_data_key(self.marker()?)
            .count_range(1, 1);
        let reject = || {
            Cmd::ReturnRun(Box::new(Cmd::Function(function("rejected").to_string())))
                .into_commands()
        };
        match part {
            "load"=>Ok(format!("scoreboard players set #busy {lock} 0\ndata remove storage {scratch} operation").into_commands()),
            "success"|"rejected"=>{
                let mut actions=format!("data remove storage {scratch} operation\nscoreboard players set #busy {lock} 0").into_commands();
                if let Some(callback)=if part=="success" {&self.success} else {&self.rejected} { actions.extend([runtime::isolated(callback())]); }
                Ok(actions)
            }
            "place"=>Ok(format!("$return run item replace entity @s {native_slot} with minecraft:written_book[minecraft:written_book_content=$(content),minecraft:custom_data={}] 1",NbtCompound::new().field(self.marker()?, 1i8)).into_commands()),
            "verify"=>Ok(format!("$execute if items entity @s {native_slot} minecraft:written_book[minecraft:written_book_content=$(content),minecraft:custom_data={}] run return 1\nreturn 0",NbtCompound::new().field(self.marker()?, 1i8)).into_commands()),
            "refresh"=>{
                let mut actions=format!("execute unless entity @s[type=minecraft:player] run return 0\nexecute if score #busy {lock} matches 1 run return 0\nscoreboard players set #busy {lock} 1\ndata remove storage {scratch} operation").into_commands();
                for schema in schemas.values() { actions.extend([format!("execute unless score {} {} matches {} run return run function {}",Self::holder(schema),ObjectiveName::logical(schema.presence_objective),schema.version,function("rejected"))]); }
                for reference in self.references() {
                    let holder=Self::holder(schemas[&reference.component]);
                    actions.extend([format!("execute unless score {holder} {} matches -2147483648..2147483647 run return run function {}",reference.objective,function("rejected"))]);
                    match reference.descriptor.kind {
                        crate::entity::StateFieldKind::Flag=>actions.extend([format!("execute unless score {holder} {} matches 0..1 run return run function {}",reference.objective,function("rejected"))]),
                        crate::entity::StateFieldKind::Enum(variants)=>{
                            let eligible=crate::condition::Condition::any(variants.iter().map(|variant|crate::condition::Condition::score(holder.into(),reference.objective.clone(),crate::condition::ScoreRange::Eq(variant.score))));
                            actions.extend([eligible.guard_actions(true,&[],reject())]);
                        }
                        _=>{}
                    }
                }
                let empty=slot.is_empty().expect("numbered slot");
                let admissible=if self.replace {empty.or(slot.matches(owned.clone()).expect("numbered slot"))} else {empty};
                actions.extend([admissible.guard_actions(true,&[],reject())]);
                let components=book.stack_components().map_err(|e|CommandError::new("journal","book",e.to_string()))?;
                let content=components.components().iter().find(|(key,_)|key=="minecraft:written_book_content").expect("canonical written book content").1.clone();
                actions.extend([format!("data modify storage {scratch} operation.content set value {}",json_nbt(&content)?) ]);
                for (index,page) in self.pages.iter().enumerate() {
                    let helper=identity::resource(&id,&format!("journal_page_{index}"));
                    let plan=crate::entity::text::materialize(page,&scratch.to_string(),&format!("operation.content.pages[{index}]"),&helper.to_string(),|field|Ok(Self::holder(schemas[&field.component]).into()))?;
                    for (location,body) in plan.helpers { crate::function::register_generated_component(crate::McFunction::new(location.parse().expect("compiler helper resource")).commands(body)); }
                    actions.extend([plan.commands]);
                }
                for part in ["place","verify"] {
                    actions.extend([format!("scoreboard players set #accepted {delivered} 0"), format!("execute store result score #accepted {delivered} run function {} with storage {scratch} operation",function(part)), format!("execute unless score #accepted {delivered} matches 1 run return run function {}",function("rejected"))]);
                }
                actions.extend([slot.matches(owned).expect("numbered slot").guard_actions(true,&[],reject()),format!("return run function {}",function("success")).into_commands()]);
                Ok(actions)
            }
            _=>unreachable!(),
        }
    }
}
fn json_nbt(value: &serde_json::Value) -> CommandResult<NbtValue> {
    Ok(match value {
        serde_json::Value::Null => {
            return Err(CommandError::new(
                "journal",
                "content",
                "null is not an NBT text component value",
            ));
        }
        serde_json::Value::Bool(value) => NbtValue::Bool(*value),
        // Canonical component strings may include book newlines. The generic
        // NbtValue::String intentionally rejects controls for data authoring;
        // quote this already-validated component payload exactly as the item
        // component serializer does, keeping the emitted command on one line.
        serde_json::Value::String(value) => {
            NbtValue::raw(serde_json::to_string(value).expect("Rust string JSON"))
        }
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                if let Ok(value) = i32::try_from(value) {
                    NbtValue::Int(value)
                } else {
                    NbtValue::Long(value)
                }
            } else {
                NbtValue::Double(value.as_f64().expect("JSON finite number"))
            }
        }
        serde_json::Value::Array(values) => NbtValue::List(
            values
                .iter()
                .map(json_nbt)
                .collect::<CommandResult<Vec<_>>>()?,
        ),
        serde_json::Value::Object(values) => {
            let mut compound = NbtCompound::new();
            for (key, value) in values {
                compound.insert(key, json_nbt(value)?);
            }
            NbtValue::Compound(compound)
        }
    })
}
impl IntoDatapack for Journal {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration = DatapackRegistration::new().lifecycle(LifecycleContribution::load(
            format!("function {}", identity::resource(&self.id, "journal_load")),
        ));
        for part in ["load", "refresh", "place", "verify", "success", "rejected"] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.id,
                &format!("journal_{part}"),
                move || definition.build(part),
            ));
        }
        registration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn journal() -> Journal {
        Journal::new(
            "journal_test:book".parse().unwrap(),
            "Journal",
            "Sand",
            InventoryIndex::new(0).unwrap(),
        )
    }
    #[test]
    fn validates_bounded_pages_and_largest_serialized_flag_label() {
        assert!(journal().book().unwrap_err().to_string().contains("pages"));
        let mut too_many = journal();
        for _ in 0..101 {
            too_many = too_many.page(EntityText::new().literal("Page"));
        }
        assert!(too_many.book().is_err());
        let mut crowded = EntityText::new();
        for _ in 0..65 {
            crowded = crowded.literal("x");
        }
        assert!(
            journal()
                .page(crowded)
                .book()
                .unwrap_err()
                .to_string()
                .contains("64")
        );
        // The enabled label has fewer visible characters but a larger escaped
        // component payload. It must determine worst-case page validation.
        let template = EntityText::new().flag(
            crate::entity::EntityFlag::new("journal_test", "flags", "on", false),
            "x".repeat(20_000),
            "\"".repeat(18_000),
        );
        assert!(
            journal()
                .page(template)
                .book()
                .unwrap_err()
                .to_string()
                .contains("32767")
        );
    }
    #[test]
    fn canonical_component_strings_stay_on_one_command_line() {
        let text = "quote \" slash \\ newline\n $(missing)";
        let encoded = json_nbt(&serde_json::json!({"text":text}))
            .unwrap()
            .to_string();
        assert!(!encoded.contains('\n'));
        assert!(encoded.contains("\\n"));
        assert!(encoded.contains("$(missing)"));
    }
}
