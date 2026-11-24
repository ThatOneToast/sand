# Checkpoint room markers

Build with `sand lang build examples/lang_world_ops` for Minecraft 26.2.
Stand in a clear test area and run `/function checkpoint_rooms:place` in the
Overworld or Nether. The pack places a lantern marker two blocks south and
clones it four blocks east. The masked clone preserves blocks at the destination
where the source region contains air.

Run `/function checkpoint_rooms:inspect_nether` from any dimension to check the
Nether from a scoped function call. The call keeps the same executor and the
invoker stays in their original dimension.
