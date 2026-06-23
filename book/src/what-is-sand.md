# What Sand Is

Sand is a Rust authoring framework that compiles typed Rust declarations into ordinary Minecraft Java datapack and optional resource-pack files. It is not a server mod and it does not run Rust inside Minecraft. Rust catches identifier, API, and composition mistakes early; Minecraft still executes generated commands and JSON.

Use Sand when a datapack has enough functions, state, events, or generated data that raw command strings become difficult to maintain. Start with [A Tiny Datapack](walkthrough/01-tiny-datapack.md), then use the [Manual](manual/functions.md) as the API reference.

<div class="sand-experimental"><strong>Experimental APIs.</strong> Optional systems intentionally evolve behind feature gates. Sand cannot make an unsupported vanilla command or gameplay signal exist.</div>
