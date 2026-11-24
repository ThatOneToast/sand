# Reward Notes

Mine stone to see a personal points message. The player State stores a formatter
function with its own captured prefix. Mining invokes a copy of that callback
with the player's current points.

```sh
sand lang build examples/lang_callbacks
sand lang run examples/lang_callbacks --offline
```
