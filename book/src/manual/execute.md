# Execute

`Execute` models low-level chains; `TypedExecute` provides common safe contexts.

```rust
TypedExecute::as_players_at_self().run(cmd::say("hello"));
Execute::new().as_(Selector::all_players()).at(Selector::self_()).run(cmd::say("hello"));
```

Execution context affects `@s`, coordinates, and local directions. Build context first, then run one command. Combine a typed condition through `TypedExecute::when`; do not concatenate raw `execute` fragments.
