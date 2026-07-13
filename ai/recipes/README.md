# Recipes

Small, verified, directly-usable patterns for common Sand tasks. Each recipe
compiled successfully against the current workspace as of its front matter
`last_verified`-equivalent (`ai/project-status.yaml`'s `last_reviewed`) —
verification method: added as a temporary `[[example]]` in `sand-example`,
checked with `cargo check -p sand-example --examples` against
`sand-core` built with `features = ["systems-all"]`, then removed (recipes
are reference snippets, not permanent example targets).

None of these recipes were validated against a live vanilla server — see
`ai/known-limitations.md` (`LIM-VAL-001`). "Compiles" is not "vanilla
reload-verified."

| Recipe | Capabilities | Use when |
|---|---|---|
| [basic-pack](basic-pack.md) | `functions`, `load-tick-components`, `score-state`, `text-and-ui` | Starting a new pack from nothing. |
| [stateful-system](stateful-system.md) | `score-state`, `flags`, `cooldowns`, `conditions`, `execute-chains` | A gated action with resource cost + cooldown (spells, abilities). |
| [custom-item](custom-item.md) | `custom-items`, `item-events` | A custom item with a use-triggered function. |
| [event-driven-feature](event-driven-feature.md) | `advancement-triggers`, `events-typed`, `score-state` | Reacting to a gameplay event (join, tick-based detection). |
| [raw-interop](raw-interop.md) | `raw-commands`, `raw-json-snbt` | Sand has no typed coverage for one specific command/field. |

Adapt these rather than inventing structure from scratch — they're drawn
from `examples/basic_typed.rs`, `examples/state_and_conditions.rs`,
`examples/player_join.rs`, `examples/interop_escape_hatches.rs`, and
`sand-components/src/item/mod.rs`'s own doc-tested example, cross-checked
against current source (some now-stale top-level `examples/*.rs` files
disagree with current APIs — see `LIM-DOC-005` below).
