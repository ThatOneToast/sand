# Counter golden provenance and pending acceptance

This eight-file pack is independently derived from existing Rust State and
export machinery, before reading any new portable compiler output. It includes the contract-derived direct-entry initialization prefix. Cross-frontend
byte comparisons and real-server evidence validate its behavior; independent
implementation review remains required before merge.

The derivation is:

- `State` derive constructs logical names `demo:counter.presence`,
  `demo:counter.suppressed`, `demo:counter.numeric_scratch`, and
  `demo:counter.value`. `ObjectiveName::logical` uses the canonical masked FNV-1a
  hash because each exceeds 16 characters. Their emitted names are respectively
  `s048b722efdc0917`, `sd5409a75751a99c`, `s02252f5fe4679ec`, and
  `sffe99bc0114fb8b`.
- `state/registry.rs::automatic_lifecycle` provisions all four objectives,
  including the numeric scratch objective even though this increment does not
  use it. The export pipeline emits these declarations sorted by objective.
- `state/registry.rs::emit_player` initializes missing field scores to zero,
  guarded by the suppression marker. It publishes presence revision 1 last.
  Neither a load nor a repeated init resets an existing field score.
- The lifecycle tick executes initialization as all players. User tick tag
  membership follows the lifecycle tick. `demo:tick` changes the executor with
  `as @a`; it does not introduce `at @s`.
- The integer State accessor emits one scoreboard add. The function references,
  tags and helpers retain the explicit semantic IDs from ADR 003.
- The CLI metadata writer pretty-prints the 26.2 verified pack format, 107,
  including its required `min_format` and `max_format` bounds. Resource bodies
  have no trailing newline, matching the existing writer.

Expected observations follow from these commands: first automatic tick sets a
new player's value to zero, publishes presence, then increments to one. Further
ticks increment independently; reloading leaves existing scores untouched.
These are contract deductions, not a claim that Minecraft runtime validation
has run.

## Direct-entry adaptation

The player-context `demo:increment` entry begins with
`function demo:__sand_lifecycle_init`, followed by the score add. Both the Rust
collector and portable compiler use the shared entry-initialization pass.
The prefix is derived from the lifecycle contract, not copied from portable
compiler output. This file records provenance rather than claiming independent
human review has already occurred.
