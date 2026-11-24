# Checkpoint room markers

Build with `sand lang build examples/lang_world_ops` for Minecraft 26.2.
Stand in a clear test area and run `/function checkpoint_rooms:place` in the
Overworld, Nether, or Workshop. The pack places a lantern marker two blocks south and
clones it four blocks east. The marker comes from the included
`structures/checkpoint_marker.nbt` template. The masked clone preserves blocks at the destination
where the source region contains air.

Run `/function checkpoint_rooms:inspect_nether` from any dimension to check the
Nether from a scoped function call. The call keeps the same executor and the
invoker stays in their original dimension.

Install the pack before starting the world to register its flat Workshop
dimension and warm, dry custom biome. Run `/function checkpoint_rooms:inspect_workshop` to check it from
another dimension. An operator can enter it using
`/execute in checkpoint_rooms:workshop run tp @s 0 -61 0`.
