# Conversations and confirmation flows

Use one `Menu` for a bounded conversation graph. `view` and `start_at` take a
canonical `EntityStateEnum`, so routing uses the same typed finite values as
State. The menu's private page score is ephemeral UI state; store quest flags,
answers, and persistent progression in an ordinary State schema.

```rust,ignore
#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum View { Greeting = 0, Confirm = 1 }

fn conversation() -> Menu {
    Menu::new("demo:guide".parse().unwrap(), Text::new("Guide"), Ticks::new(200))
        .start_at(View::Greeting)
        .choice_to(Text::new("Buy the key"), Condition::all([]), View::Confirm,
            || ())
        .view(View::Confirm, Text::new("Spend three coins?"))
        .choice(Text::new("Confirm"), Wallet::coins.matches(3..).unwrap().condition(),
            || key_purchase().buy())
        .choice_to(Text::new("Back"), Condition::all([]), View::Greeting,
            || ())
}
```

Register the menu normally for player-position conversations, or give it to
`Interactable::opens_menu` to register the entire graph with an exact object
binding. Do not separately register a menu owned by an Interactable. Every view
shares that object identity and the `while_open` validity condition. Pagination
stays within the current view; transitions may revisit earlier views, but only
fresh player input advances the graph.

A transition consumes the current session before running its callback. The
callback runs as the player at the clicked object, then the destination rechecks
player-context validity and receives fresh tokens. Replayed buttons cannot act
on the destination. If the callback opens another menu, that newer session wins.
If the object is removed, unloaded, out of range or in another dimension, the
transition cannot reopen an authorized session. Callback effects are not rolled
back if destination admission fails.

A normal `choice` is terminal: use it for confirmation, declining, or explicitly
ending the conversation. Escape has no reliable vanilla callback; timeout,
explicit `close`, disconnect and reload invalidate the session as for any Menu.
Opening again starts at the configured entry view, not at an expired choice.
An empty entry or missing transition destination fails export. The complete
graph shares the existing 256-choice bound and at most 64 choices per page.

## Bounded choice and numeric forms

A choice callback can assign a canonical State enum, flag or score, then move to
a confirmation view. For a bounded numeric answer, create a finite choice for
each accepted value and assign that value in the server-side callback. Never
interpret the routing token as the answer: tokens authenticate the current
session only, and authoritative State guards still apply before every choice.
This route uses the existing survival-accessible trigger transport. Free-text
submission and arbitrary numeric dialog input remain unsupported until their
response transport is proven for the target version.

The maintained fixture `sand-core/tests/gameplay_menu.rs` includes an object-bound
conversation and export checks for 26.1, 26.2 and 26.3. The two-player native
`validate_interactions.py` proof exercises forward/back transitions, stale tokens,
a changed authorization condition, independent player views, object cleanup and
a callback that opens a different menu.

The complete two-player interaction fixture also passes on 26.3, including
conversation routing, cross-namespace callback precedence, token replay rejection, exact-object isolation, dimension
changes, disconnect and reload. See the [26.3 evidence](../testing/conversations-26.3-evidence.json).
The harness submits the native trigger in player context through server commands;
it does not claim network-driven dialog interaction or arbitrary text transport.
