# Training sparks

Build with `sand lang build examples/lang_projectiles` for Minecraft 26.2.
Run `/function scout_sparks:cast` as a player while facing a zombie. The spark
travels forward, stops on the first non-air block or matching entity, and damages
zombies on impact with credit to the caster. The impact callback emits particles
at the collision and sends an actionbar to the caster. Use
`/function scout_sparks:cancel` to cancel your active sparks.
