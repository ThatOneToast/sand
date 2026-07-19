# 12. Dialogs

Trailforge exposes its actions through a small in-game menu instead of
requiring players to memorize `/function` commands:

{{#include ../../examples/book_project/src/lib.rs:dialog}}

## `Dialog::multi_action_local`

Dialogs are a modern Minecraft Java feature (server-driven UI screens sent
to the client) — Sand's `Dialog` builder mirrors the underlying dialog JSON
schema: a `title`, a `body` (here, `DialogBody::text(...)`, plain
descriptive text), and one or more `button`s. `multi_action_local("trailhead")`
declares a dialog with a stable local ID (`trailhead`) that other code can
reference without holding the `Dialog` value itself.

Each `DialogButton::new(label).action(DialogAction::run_function(handle))`
pairs a button label with a typed reference to a Sand function —
`grapple` and `claim_striders`, the same functions chapter 8 walks through.
Passing the function *item* (not a `ResourceLocation` string) means the
dialog button target is checked against a function that actually exists at
compile time; renaming `claim_striders` without updating this line is a
compiler error, not a dialog button that silently does nothing at runtime.

## Opening the dialog

`trail:menu` (chapter 8) is the only place that opens this dialog:

```rust,ignore
#[function("trail:menu")]
pub fn open_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("trailhead"));
}
```

`DialogRef::local("trailhead")` references the dialog by its ID string
rather than requiring the `trailhead_dialog()` function be in scope —
useful when the dialog is opened from a different module than the one that
defines it, or (as here) opened from a plain function rather than another
component-returning builder. The ID string (`"trailhead"`) has to match
`multi_action_local`'s argument exactly; there's no compile-time link
between the two beyond that string, so keeping the definition and the
reference near each other (as Trailforge does — both live in `src/lib.rs`)
is a project-organization discipline worth keeping as a pack grows.

`on_obtained_grapple_core` (chapter 9) is Trailforge's other dialog
trigger, calling `cmd::call(open_menu)` — a typed function call (chapter 8's
`ResourceLocation` pattern, but resolved directly from the function item)
right after telling the player about the upgrade, so picking up a Grapple
Core immediately surfaces the menu that explains what to do with it.

## Why a dialog instead of chat text

A `tellraw` message can tell a player what to type; a dialog lets them
click a button instead, removing the "which exact command was that again"
friction — especially valuable for a small pack's discoverability. Reach
for a dialog when a pack has more than one or two player-facing actions
worth surfacing as UI, and reserve chat/actionbar text (chapters 3, 8, 13)
for status feedback that doesn't need to *request* player input.
