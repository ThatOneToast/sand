# Generic Mining History

One `History<T>` library stores both strings and numbers in separate saved
per-player histories. Its impl uses borrowed receivers, associated constructors,
`Self`, and a method with its own `U` parameter. A generic function returns a fixed
array, which the caller destructures into independent locals.

```sh
sand lang build examples/lang_generics
sand lang run examples/lang_generics --offline
```

Mine ten stone blocks to receive iron and read the latest entries. The generic
library lives in `history.sand`; it has the same module and privacy rules as the
rest of the project.
