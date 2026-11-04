# Menus, interactables, rewards and books

Use `sand::systems::{Menu, Interactable, Purchase}` to connect native interfaces
to gameplay. Register each definition through `#[datapack_component]` before
invoking its actions. These systems compose canonical State, Lifetime, item
stacks, rich text and world block handles.

## Player-owned menus

A `Menu` has an identity, title, positive tick timeout and up to 256 choices, with at most 64 gameplay choices per page:

```rust,ignore
Menu::new("demo:shop".parse()?, Text::new("Shop"), Ticks::new(200))
    .choice(Text::new("Buy"), Wallet::coins.matches(3..)?.condition(), || {
        purchase().buy()
    })
```

Call `open()` as a player. Each opening receives fresh action values. Opening a
menu in the same pack supersedes the player's previous menu. An action checks
its player session, timeout, disconnect generation and authorization condition,
then consumes the session before executing user code. A success callback can
open another registered menu for navigation or confirmation.

Choices remain visible even when their condition is false; eligibility is
checked at selection time. Tokens route actions; they are not secrets and do
not replace gameplay authorization. Use `while_open(condition)` to invalidate a session when player-context
eligibility changes. Relative queries in this guard use the current player
position. Close the session explicitly with `close()`
when its owner or context becomes invalid. Without a close callback, the
online-tick timeout provides cleanup. Reload and disconnect invalidate sessions.
The monotonic token allocator refuses exhaustion rather than recycling old
values. It persists across reloads. Do not manually reset its private scores.

This initial native menu model does not expose text input
or inventory-slot click cancellation. Literal `$(...)` macro delimiters in menu
text are rejected during export. Use ordinary text or a plain static Dialog.

Low-level `DialogAction::callback` remains useful for stateless actions. Its
pack-specific dispatcher consumes input before invoking callbacks, but it does
not prove that a player opened a menu. Use Menu when the action needs a session.

## Clickable world objects

Wrap a configured `Lifetime` in `Interactable`. `bounds(width, height, range)`
sets its native hitbox and maximum actor distance. `on_use` and `on_attack`
callbacks execute **as the recorded player at the object position**. Therefore
player State refers to the actor while relative block operations refer to the
object. The hitbox is stationary; provide blocks or display entities as visuals.

Lifetime capacity, required State, owner loss, expiration, cancellation and
parent-child ownership remain in effect. `start()` creates an owned instance;
`cancel()` cleans up the owner's instances. Only loaded instances are observed;
no chunks are force-loaded.

Minecraft stores the latest use and latest attack, not an event queue. Sand
consumes a record before its callback and prioritizes use if both are pending.
Multiple clicks between observations can coalesce. A managed hitbox does not
cancel the normal use of an unrelated native block or expose a generic chest
open event. `LootContainerOpenEvent` specifically observes loot generation.

## Purchases and reward containers

`Purchase::new(id, stack, InventoryIndex)` delivers a concrete item into one empty
player inventory slot. Optional `cost(balance, amount)` and `claim_once(flag)`
use registered State. Personal flags give personal claims; global flags give
shared claims. Reset a canonical claim flag through your gameplay logic to
restock a reward. The definition needs no private duplicate currency model.

Admission checks schema membership, balance, claim eligibility and destination
emptiness. The native slot write must succeed before charging or recording the
claim. A pack-owned transaction lock rejects reentrant purchases during native
placement. Both commits happen before `on_success`, which runs after unlocking. Failure invokes `on_rejected`
and leaves payment and claim unchanged. A full slot never spills a reward onto
the ground. `buy()` must execute as a player.

This is a deliberately bounded transaction: one stack and one destination slot.
It does not merge stacks, remove arbitrary item payments, transfer arbitrary
container contents or roll back user callbacks. A chest-shaped world object can
invoke Purchase without exposing a virtual inventory GUI. Ordinary block
containers remain available through `WorldContext::block(...).inventory()`.

## Written books

Attach `ItemComponent::WrittenBook(WrittenBook::new(title, author).page(text))`
to an ordinary `ItemStack` or `CustomItem`. Pages use the existing TextComponent,
including page-change links, styles and hover text. The same typed content
exports through commands and structured item-resource payloads.

Export validates a 32 UTF-16-unit title limit, a supported 100-page authoring
limit, page size and nested rich text. Books are emitted unresolved so native
text resolution happens on first opening. A journal is a snapshot: replacing
its item publishes updated content. Book links do not grant command permissions,
and giving a book does not force its screen open.

See `examples/interaction_hub` for a shop, key, journal, gate and personal reward
composed without handwritten gameplay commands. `scripts/validate_interactions.py`
provides the disposable two-player 26.2 runtime proof; export regressions also
cover 26.1 and 26.3.

### Paginated, object-bound menus

Use `Menu::page_size(8)` to split up to 256 choices into pages. Sand adds Previous
and Next controls and issues fresh tokens on every page change. Old buttons and
choices on other pages cannot authorize actions; navigation refreshes the timeout.

Use `Interactable::opens_menu(menu)` to register a menu and open it for the exact
hitbox clicked. Do not register that menu separately or also configure `on_use`.
The session stays bound across navigation. Removing or unloading the hitbox,
changing dimension, or leaving the interactable's configured range closes it.
Choice guards run at the player; successful callbacks retain the player executor
but run at the bound object's position. Calling `open()` on a bound definition
cannot bypass the interaction by selecting a nearby object.
