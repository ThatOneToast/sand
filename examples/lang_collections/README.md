# Quest board

A Sand-language pack with three saved personal quests: mine stone, mine oak logs,
and defeat zombies. Players receive a board item on first join; using it shows
progress. Each completed quest pays emeralds once.

The pack demonstrates arrays of structs in player State, mutable methods on
indexed elements, `[value; N]` defaults, snapshot `for` iteration, `len()`,
`contains()`, `get_or()`, and an indexed circular reward history. Each player's
quests and history remain separate. Struct receivers use the same call and
copying model as ordinary function values.

```sh
cargo run -p sand-cli -- lang check examples/lang_collections/pack.sand
cargo run -p sand-cli -- lang build examples/lang_collections/pack.sand --output dist/quest_board
cargo run -p sand-cli -- lang run examples/lang_collections/pack.sand
```

Run from the repository root. Targets Minecraft Java 26.2. See the
[language guide](../../book/src/language.md) for collection bounds and copy rules.
