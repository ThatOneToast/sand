# Ore Journal

Mine iron or gold ore to record a discovery and increment your personal count.
Use a carrot on a stick to read your journal in discovery order.

```sh
sand lang build examples/lang_maps
sand lang run examples/lang_maps --offline
```

The example bundles an editable copy of the module installed by
`sand lang add collections`. The map holds counts, and the set tracks first
discoveries. Both persist in player State.
