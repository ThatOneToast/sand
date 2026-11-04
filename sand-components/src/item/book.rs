//! Typed written-book content shared by item commands and resource payloads.
use sand_commands::{CommandProfile, TextComponent};
use serde_json::{Value, json};

/// Written-book content with rich text pages and validated authoring limits.
/// Add it through `ItemComponent::WrittenBook` to an ordinary ItemStack or
/// CustomItem. Pages reuse TextComponent, including page navigation and hover
/// text. Content resolves when Minecraft first opens the book; replacing the
/// item is required to publish a new journal snapshot. This does not force a
/// player to open the book or grant command permissions to its click actions.
#[derive(Debug, Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::component::WrittenBook", module = "sand::component",
summary = "Typed written-book content using canonical rich text.", context = "Use ItemComponent::WrittenBook on canonical ItemStack or CustomItem. Export validates page text and authoring limits.", minecraft = "Emits native written_book_content with unresolved rich text pages and generation zero.", use_when = ["Authoring journals, manuals and quest books"], avoid_when = ["Capturing writable-book input or forcing a screen open"],
example = "use sand::component::WrittenBook;")]
pub struct WrittenBook {
    title: String,
    author: String,
    pages: Vec<TextComponent>,
}
impl WrittenBook {
    /// Start a book; title is limited to 32 UTF-16 code units at export.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::WrittenBook::new", module = "sand::component",
summary = "Start written-book content with a title and author.", context = "Use ItemComponent::WrittenBook on canonical ItemStack or CustomItem. Export validates page text and authoring limits.", minecraft = "Emits native written_book_content with unresolved rich text pages and generation zero.", use_when = ["Authoring journals, manuals and quest books"], avoid_when = ["Capturing writable-book input or forcing a screen open"],
params(title = "Title limited to 32 UTF-16 code units.", author = "Author displayed by the native book interface."), returns = "The configured book content.",
example = "use sand::component::WrittenBook;")]
    pub fn new(title: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            author: author.into(),
            pages: vec![],
        }
    }
    /// Append one rich text page, up to 100 pages. Navigation is one-indexed.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::WrittenBook::page", module = "sand::component",
summary = "Append a rich text page.", context = "Use ItemComponent::WrittenBook on canonical ItemStack or CustomItem. Export validates page text and authoring limits.", minecraft = "Emits native written_book_content with unresolved rich text pages and generation zero.", use_when = ["Authoring journals, manuals and quest books"], avoid_when = ["Capturing writable-book input or forcing a screen open"],
params(page = "Canonical rich text including page navigation."), returns = "The configured book content.",
example = "use sand::component::WrittenBook;")]
    pub fn page(mut self, page: TextComponent) -> Self {
        self.pages.push(page);
        self
    }
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.title.encode_utf16().count() > 32 {
            return Err("title exceeds 32 UTF-16 code units".into());
        }
        if self.pages.len() > 100 {
            return Err("pages exceeds the supported 100-page authoring limit".into());
        }
        for (index, page) in self.pages.iter().enumerate() {
            page.validate_at_path(
                &CommandProfile::unprofiled(),
                format!("written_book_content.pages[{index}]"),
            )
            .map_err(|e| e.to_string())?;
            if page.to_string().encode_utf16().count() > 32767 {
                return Err(format!("page {index} exceeds 32767 UTF-16 code units"));
            }
        }
        Ok(())
    }
    pub(super) fn json(&self) -> Value {
        json!({"title": self.title, "author": self.author,
            "pages": self.pages.iter().map(|p| serde_json::from_str::<Value>(&p.to_string()).expect("typed text JSON")).collect::<Vec<_>>(),
            "generation": 0, "resolved": false})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CustomItem, ItemComponent, ItemId};
    #[test]
    fn content_uses_rich_text_in_both_command_and_resource_forms() {
        let item = CustomItem::new(ItemId::minecraft("written_book").unwrap()).component(
            ItemComponent::WrittenBook(
                WrittenBook::new("Quest", "A \"writer\"")
                    .page(TextComponent::literal("Next\npage\twith controls").click_change_page(2)),
            ),
        );
        let command = item.try_to_string().unwrap();
        assert!(command.contains("written_book_content="));
        assert!(command.contains("change_page"));
        assert!(!command.contains('\n'));
        assert!(command.contains("\\n"));
        let content = item.stack_components().unwrap();
        let value = serde_json::to_value(content.components()).unwrap();
        assert!(value.to_string().contains("written_book_content"));
    }
    #[test]
    fn limits_and_nested_text_are_validated() {
        assert!(WrittenBook::new("x".repeat(33), "Sand").validate().is_err());
        assert!(
            WrittenBook::new("😀".repeat(17), "Sand")
                .validate()
                .is_err()
        );
        let mut book = WrittenBook::new("Pages", "Sand");
        for _ in 0..101 {
            book = book.page(TextComponent::literal("Page"));
        }
        assert!(book.validate().is_err());
        assert!(
            WrittenBook::new("Bad", "Sand")
                .page(TextComponent::literal("Bad").click_change_page(0))
                .validate()
                .is_err()
        );
    }
}
