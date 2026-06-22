# Interop Example

Interop is the right place for raw commands.

```rust
#[function]
pub fn call_other_pack() {
    cmd::raw("function other_pack:api/do_special_thing");
}
```

Keep interop functions small and named so raw commands are easy to audit.
