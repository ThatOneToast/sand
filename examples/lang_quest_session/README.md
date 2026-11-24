# Trail quest

Build with `sand lang build examples/lang_quest_session` for Minecraft 26.2.
Run `/function scout_trial:begin` as a player to start the quest. The session
checks its next objective every 20 loaded ticks. Run the `shortcut` entry point
before the first objective to skip the glowstone reward. Use `cancel` to keep
the current progress, `begin` to continue, or `replay` to restart from the
first objective. Each player stores a separate quest index.
