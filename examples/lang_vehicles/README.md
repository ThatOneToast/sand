# Minecart shuttle

Build the pack with `sand lang build examples/lang_vehicles` and load it into a
Minecraft 26.2 world. Stand on a rail and run `/function cart_shuttle:place_cart`
to place a shuttle minecart. First-time players receive a carrot on a stick. If
you already joined the world, give yourself one with `/give @s carrot_on_a_stick`.

Use the item within eight blocks of a shuttle cart to board. Use it again to
dismount. The pack retains the nearest cart's identity while the query executes
as the cart, then mounts the original player using that reference. Vanilla
decides whether a cart has an available seat.
