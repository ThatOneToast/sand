# Bread baked to order

Build with `sand lang build examples/lang_waits` for Minecraft 26.2. Use a carrot
on a stick to place an order. A batch of bread arrives two seconds later while
the original chunk is loaded. Each order keeps its own batch number, including
when several players order at once or one player places several orders.

Run `/function bread_orders:cancel_order` as a player to cancel that player's
pending batches. The pack counts orders in player State and keeps pending
ingredients and batch numbers in each invocation's local variables.

Every third batch spends an extra 12 loaded ticks in the oven. Both branches
serve the original batch after its own timer completes.
