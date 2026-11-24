# Supply inspection

Build this Sand package with `sand lang build examples/lang_supply_job`. Run
`/function supply_job:begin_inspection` as a player. The job runs five indexed
steps, with at most two steps per loaded tick, then reports the player's total.
Run `/function supply_job:stop_inspection` to cancel that player's instances.
