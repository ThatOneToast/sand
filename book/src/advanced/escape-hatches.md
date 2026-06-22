# Escape Hatches

Use escape hatches for:

- another datapack's documented public API
- modded commands
- snapshot syntax not modeled by Sand yet
- debugging generated output

Prefer `cmd::raw(...)` over bare strings so intent is explicit.
