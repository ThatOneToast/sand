# Sand Documentation

Sand is organized around typed Rust APIs that compile to vanilla Minecraft Java
datapacks and optional resource packs.

This directory contains focused reference docs, audits, and development notes.
The [mdBook guide](../book/src/introduction.md) is the user-facing project guide; this directory is focused reference, audit, and migration material.

## Beginner Path

- [Getting Started](getting-started.md)
- [Authoring Model](authoring-model.md)
- [Typed State](typed-state.md)
- [Typed Commands](typed-commands.md)
- [Conditions](conditions.md)
- [Typed Execute](typed-execute.md)
- [Events](events.md)
- [Damage](damage.md)
- [Storage And NBT](storage-nbt.md)
- [Version Capabilities](version-capabilities.md)

## Systems reference

The guide owns long tutorials. Its current manual pages cover [inventory](../book/src/manual/inventory.md), [movement](../book/src/manual/movement.md), [entities/interactables](../book/src/manual/entities.md), [custom items](../book/src/manual/custom-items.md), [item events](../book/src/manual/item-events.md), and [player data](../book/src/manual/player-data.md). These APIs are feature-gated and experimental; verify the target Minecraft version before depending on generated command/component syntax.

## Datapack Components

- [Components](components.md)
- [Dialogs](dialogs.md)
- [Advancement Events](advancement-events.md)
- Advancements, recipes, loot tables, predicates, item modifiers, and tags are
  covered from [Components](components.md) and [Examples](examples.md).

## Advanced

- [Storage And NBT](storage-nbt.md)
- [Version Support](version-support.md)
- [Version Capabilities](version-capabilities.md)
- [Architecture](architecture.md)
- [Testing](testing.md)
- [Escape Hatches](escape-hatches.md)
- [Examples](examples.md)

## Agent-facing guides

- [Typed state agent guide](agents/state-guide.md) — concise patterns for
  `GameState<S>`, transitions, enter/exit hooks, per-state tick, lifecycle
  registration, and the transition backend. Pairs with [Typed State](typed-state.md).
