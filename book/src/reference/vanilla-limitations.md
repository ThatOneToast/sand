# Vanilla Limitations

- Storage is global; dynamic per-player NBT keys are runtime conventions.
- Advancement triggers do not cover every gameplay action or expose every event value.
- Damage tracking is a cumulative-stat approximation, not an event payload.
- Push/launch helpers are teleports/effects, not arbitrary physics velocity.
- Interaction entities react to right-click, not proximity.
- Exact successful shield-block detection and axe-disable behavior do not have a dedicated Sand event.
- Freezing (powder snow) and drowning start/stop events are not provided: as of Minecraft Java 26.2, vanilla exposes freezing only via the raw `ticks_frozen`/`ticks_frozen_max` NBT ratio and drowning only via the raw `Air` NBT stat, with no boolean entity predicate flag or scoreboard criterion for either (unlike `flags.is_on_fire`, which backs `PlayerCaughtFireEvent`/`PlayerExtinguishedEvent`). Detecting them would require an author-chosen threshold sampled via `data get`, which is an inferred approximation, not an exact transition — Sand does not ship a misleadingly "exact"-looking event for either.
- `PlayerHealthChangedEvent`/`PlayerHealthLostEvent`/`PlayerHealthGainedEvent`/`PlayerLowHealthEvent` observe vanilla's `health` scoreboard criterion: an integer value (0-20 by default) that does **not** include absorption hearts, which vanilla tracks as a separate decaying overlay.
- `PlayerGamemodeChangedEvent` with a typed previous/current payload is not provided: the current event-handler-context model has no honest way to expose an enum-typed "previous state" value inside a handler body. Typed `PlayerEntered<Mode>Event`/`PlayerExited<Mode>Event` pairs are provided instead.

These are vanilla constraints, not missing Rust typing.
