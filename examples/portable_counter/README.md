# Portable counter: Rust frontend

This standalone crate depends only on the `sand` façade. Its semantic identities
match ADR 003: State `demo:counter`, revision 1, integer field `value` with default
0; server entry `demo:tick`; player entry `demo:increment`; and membership of
`demo:tick` in `minecraft:tick`.

`tick` establishes a player executor with `as @a` and calls `increment`. It does
not change positional context. Canonical automatic lifecycle work initializes
players before the tick entry runs. No raw command or manually named
initialization helper is used in the authored program.

Run `cargo run --manifest-path examples/portable_counter/Cargo.toml --bin sand_export`
to obtain the Rust collection records. The CLI packages these records with
`sand.toml` metadata through its shared output writer.

The independently derived expected pack is at
`tests/fixtures/program/golden`; see `GOLDEN.md` for provenance and review status.
The increment entry declares `context = player`, so canonical State initialization
also runs before direct calls. `scripts/check-program.sh` compares the complete
Rust-produced pack with constructed Program, JSON, Python and blueprint output.
