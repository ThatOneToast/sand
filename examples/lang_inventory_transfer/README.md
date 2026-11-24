# Camp supply transfer

Build with `sand lang build examples/lang_inventory_transfer` for Minecraft 26.2.
Place at least two items in your first hotbar slot and leave the first slot in
the top row of your main inventory empty. Run `/function camp_supplies:stow`
as the player to move two items. Run it again with the same stack to merge
another two into the destination.

Try with fewer than two items in the source or with a different item in the
destination to see the rejection callback.

Run `/function camp_supplies:supplies` to open the native menu. Select the move
directly or use Next to find the slot guide.
