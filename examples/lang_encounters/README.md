# Encounters (Sand language)

A two-module pack written in the [Sand language](../../book/src/language.md)
instead of Rust: per-player encounter phases driven by a flow, a shared world
clock, and rewards granted from another module. The raid module also shows
how to select and equip nearby guards. The rewards module defines an amethyst
Sentry trim for armor at the smithing table and a native advancement for
acquiring amethyst or defeating a zombie.

```sh
sand lang check
sand lang build      # writes dist/demo
```

No `Cargo.toml` or `sand.toml` is needed; `pack.sand` is the package root.
