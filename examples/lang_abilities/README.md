# Mana-powered spark

Build with `sand lang build examples/lang_abilities` for Minecraft 26.2.
Run `/function scout_magic:cast` as a player to launch a spark. Each admitted
cast spends two mana and starts a 20-tick cooldown. Run
`/function scout_magic:status` for admission diagnostics. Cancelling a flight
leaves the spent mana and the cooldown in the player's State.
