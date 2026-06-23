# Vanilla Limitations

- Storage is global; dynamic per-player NBT keys are runtime conventions.
- Advancement triggers do not cover every gameplay action or expose every event value.
- Damage tracking is a cumulative-stat approximation, not an event payload.
- Push/launch helpers are teleports/effects, not arbitrary physics velocity.
- Interaction entities react to right-click, not proximity.
- Exact successful shield-block detection and axe-disable behavior do not have a dedicated Sand event.

These are vanilla constraints, not missing Rust typing.
