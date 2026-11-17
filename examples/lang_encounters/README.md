# Encounters (Sand language)

A two-module pack written in the [Sand language](../../book/src/language.md)
instead of Rust: per-player encounter phases driven by a flow, a shared world
clock, and rewards granted from another module.

```sh
sand lang check
sand lang build      # writes dist/demo
```

No `Cargo.toml` or `sand.toml` is needed; `pack.sand` is the package root.
