# Function References

Sand accepts `IntoFunctionRef` where a function id is needed. Pass a registered function pointer rather than copying a reward-function string.

```rust
#[function] pub fn reward() { cmd::say("reward"); }
cmd::call(reward);
item.on_use_fn(ResourceLocation::new("arcane", "items/use").unwrap(), reward);
```

Use `ResourceLocation` or a string only for external datapacks. Dynamic branch helpers are intentionally private: call `when(...).then_all(...)` and let Sand register/deduplicate/export their functions.
