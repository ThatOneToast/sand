# Exploration Log

A language-only datapack with a separate saved vector of visits for each player.
Mine stone or defeat zombies to record coordinates. Right-click the supplied
notebook to read the last eight visits.

```sh
sand lang build examples/lang_vectors
sand lang run examples/lang_vectors --offline
```

Each vector element is a record with a string and two fixed-point coordinates.
The example combines `Vec<T>` State, mutation and removal, snapshot iteration,
associated functions, borrowed methods, typed player properties, and runtime
text. The example keeps eight entries; language vectors support up to 1,024.
