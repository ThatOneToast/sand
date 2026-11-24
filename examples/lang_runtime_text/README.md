# Field Journal

A language-only datapack with a separate persistent journal for each player.
Mine stone or defeat zombies to add notes; right-click the supplied notebook to
read the last five entries. Values survive datapack reloads.

```sh
sand lang build examples/lang_runtime_text
sand lang run examples/lang_runtime_text --offline
```

The example combines string constants, concatenation, scalar `to_string`,
string fields in State, arrays of strings, snapshot iteration, borrowed State
methods, Unicode text, and styled runtime interpolation. Strings copy by value;
indices and `len` count Unicode scalar characters. Collection size is fixed in
this example. The notebook name and the chat layout use Minecraft translation
keys.
