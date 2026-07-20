# Introduction

This book teaches Sand by building one datapack from an empty directory to a
finished, running pack: **Trailforge**, a small equipment-and-traversal
gameplay mod. Trailforge adds a craftable **Grapple Core**, a pair of
upgraded boots called **Trail Striders**, a stamina resource that fuels a
grapple dash, and a small upgrade menu. Every chapter adds the next piece of
that pack and explains *why* it's built that way, not just *how* to call the
API.

Sand is a Rust authoring framework that compiles typed Rust declarations into
ordinary Minecraft Java datapack files — `.mcfunction` text, advancement and
recipe JSON, function tags, loot tables, dialogs, and more. It is not a
server mod: Sand does not run Rust inside Minecraft. Rust exists to catch
identifier, API, and composition mistakes before you ever load the world;
Minecraft still executes the generated commands and JSON exactly as vanilla
would.

## Why this book is one project

Earlier drafts of this book taught Sand as a flat API reference: one chapter
per builder type, with disconnected snippets. That made it hard to see how
the pieces fit together in a real pack — how a custom item interacts with an
advancement trigger, how a tick-driven timer feeds an event, how execution
context changes what a command means. This book instead follows Trailforge
end to end. Each chapter's code is a real, compiling fragment of
[`examples/book_project`](https://github.com/ThatOneToast/sand/tree/main/examples/book_project),
included directly from the source file via mdBook's `{{#include}}`
directive — the snippets in this book and the code in the repository can
never drift apart, because they're the same text.

## What Trailforge actually contains

By the end of this book you will have built (and understood the reasoning
behind) every system Trailforge demonstrates:

- **Load and tick components** — one-time setup and per-tick logic.
- **Custom items** — the Grapple Core (crafting material) and Trail Striders
  (upgraded boots with an attribute modifier).
- **A shaped recipe** — crafting the Grapple Core from string and an ender
  pearl.
- **Player state** — a scoreboard-backed stamina score, flags, a cooldown, a
  regen timer, and a storage-backed config value.
- **Typed commands and execution contexts** — `execute as/at`, conditions,
  and grouped `if_()`/`else_all()` branching.
- **Events** — vanilla events (`FirstJoin`, `OnDeath`), an
  advancement-backed custom event, a tick-backed custom event, and a chained
  event that composes off another event.
- **Equipment and attributes** — an `AttributeModifier` on armor.
- **A dialog** — a multi-button upgrade menu.
- **Particles and sounds** — a reusable `Vfx` sequence for the dash.
- **Optional systems** — the `systems-damage` feature's `DamageTracker`.

Nothing in this book is aspirational: if a chapter shows you an API, that API
is exercised by Trailforge's own compiling, tested source, and by
Trailforge's own `cargo test` suite.

## How to read this book

Read the chapters in order the first time — each one builds on state and
functions introduced earlier. After that, use it as a reference by jumping
straight to the system you need; each chapter is self-contained enough to
skim independently once you know the overall shape of the pack.

The [Vanilla Limitations](reference/vanilla-limitations.md) reference page
at the end lists gameplay signals vanilla Minecraft simply does not expose.
Sand cannot make an unsupported command or event exist; where a chapter
brushes against one of these limits, it says so.
