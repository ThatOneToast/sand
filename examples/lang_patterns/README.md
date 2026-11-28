# Mining Streak

Mine stone to earn a personal reward every ten blocks. An exhaustive value match
selects a typed reward record, then a nested pattern unpacks its title and fixed
array of item amounts. Each player's streak persists separately.

```sh
sand lang build examples/lang_patterns
sand lang run examples/lang_patterns --offline
```

The example combines enum alternatives, block expressions, guarded destructuring
arms, nested array patterns, runtime item counts, and formatted strings.
